// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ordinary view and the round's worker read one participant's walks
//! from the same accepted native world. The cache never enters a save.

use std::sync::{Arc, OnceLock};
use std::thread::{Builder, JoinHandle};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use cosmograph::{Session, Simulation, schema::Id};
use cosmograph_host::{
    drive::{Walk, walk_handbacks, world_foreground_walks},
    world::{Cut, NativeRevision, World},
};

use super::{Mesocosm, Runtime, session};

#[derive(Default)]
pub(super) struct Foreground {
    current: Option<(NativeRevision, Arc<MinuteWalks>)>,
    /// At most one preview is in flight. The next frame retries the current
    /// revision once it finishes; intermediate revisions never form a queue.
    preview: Option<JoinHandle<()>>,
}

impl Foreground {
    pub(super) fn cached(
        &self,
        revision: &NativeRevision,
        participant: Id,
    ) -> Option<Arc<MinuteWalks>> {
        let (at, minute) = self.current.as_ref()?;
        (at == revision && minute.participant == participant).then(|| Arc::clone(minute))
    }

    fn request(&mut self, minute: Arc<MinuteWalks>, world: &World) -> Result<(), String> {
        if self
            .preview
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
        {
            return Ok(());
        }
        let snapshot = world.session_snapshot()?;
        let preview = Builder::new()
            .name("mesocosm-foreground".into())
            .spawn(move || {
                minute.walks(&snapshot.sim);
            })
            .map_err(|why| why.to_string())?;
        self.preview = Some(preview);
        Ok(())
    }
}

/// The worker receives only this minute's slot. A later native publication
/// can replace the view's slot while the old worker finishes independently.
pub(crate) struct MinuteWalks {
    participant: Id,
    revision: u64,
    walks: OnceLock<Arc<[Walk]>>,
}

impl MinuteWalks {
    fn walks(&self, sim: &Simulation) -> &Arc<[Walk]> {
        self.walks.get_or_init(|| {
            static PROFILE: OnceLock<bool> = OnceLock::new();
            let profile = *PROFILE.get_or_init(|| {
                std::env::var_os("MESOCOSM_PROFILE_ACTIVE").is_some_and(|v| v == "1")
            });
            let start = profile.then(Instant::now);
            if profile {
                self.note(sim, "begin", 0);
            }
            let walks = world_foreground_walks(sim, self.participant).into();
            if let Some(start) = start {
                self.note(sim, "end", start.elapsed().as_micros());
            }
            walks
        })
    }

    fn note(&self, sim: &Simulation, phase: &str, elapsed_us: u128) {
        let at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());
        eprintln!(
            "[mesocosm-view] foreground plan {phase} tick={} revision={} participant={} slot={self:p} thread={} elapsed_us={elapsed_us} at {at_ms}ms",
            sim.state().tick,
            self.revision,
            self.participant,
            std::thread::current().name().unwrap_or("unnamed")
        );
    }

    pub(crate) fn matches(&self, session: &Session, participant: Id, accepted: Cut) -> bool {
        self.participant == participant
            && accepted.tick == session.sim.state().tick
            && accepted.entries == session.entries.len()
            && accepted.state_hash == session.sim.state_hash()
    }

    pub(crate) fn handbacks(
        &self,
        sim: &Simulation,
    ) -> Result<Vec<cosmograph::spatial::Handback>, String> {
        walk_handbacks(sim, self.walks(sim))
    }
}

impl Mesocosm {
    pub(super) fn foreground_minute(
        &mut self,
        world: &World,
    ) -> Result<Option<Arc<MinuteWalks>>, String> {
        let participant = self.lp.participant;
        let revision = world.native_revision()?;
        if let Some(minute) = self.foreground.cached(&revision, participant) {
            return Ok(Some(minute));
        }
        let session = session(world);
        let rules = &session.sim.genesis().rules;
        if !rules.foreground_walks || rules.speed.is_none() {
            self.foreground.current = None;
            return Ok(None);
        }
        let minute = Arc::new(MinuteWalks {
            participant,
            revision: revision.sequence(),
            walks: OnceLock::new(),
        });
        self.foreground.current = Some((revision, Arc::clone(&minute)));
        Ok(Some(minute))
    }
}

impl Runtime {
    /// Polls walks for a frame. While a plan is pending the host draws its
    /// native scene. Whichever worker starts first computes the shared slot.
    pub fn poll_foreground_walks(&mut self) -> Result<Option<Arc<[Walk]>>, String> {
        let world = self.driver.world().share();
        let product = self.driver.product_mut();
        let Some(minute) = product.foreground_minute(&world)? else {
            return Ok(None);
        };
        if let Some(walks) = minute.walks.get() {
            return Ok(Some(Arc::clone(walks)));
        }
        if !product
            .pending
            .as_ref()
            .is_some_and(|worker| worker.plans(&minute))
        {
            product.foreground.request(minute, &world)?;
        }
        Ok(None)
    }

    /// Reads a plan synchronously for scripts unless the clock's worker is
    /// preparing it. Frame callers use `poll_foreground_walks`.
    pub fn foreground_walks(&mut self) -> Result<Option<Arc<[Walk]>>, String> {
        let world = self.driver.world().share();
        let product = self.driver.product_mut();
        let Some(minute) = product.foreground_minute(&world)? else {
            return Ok(None);
        };
        if product
            .pending
            .as_ref()
            .is_some_and(|worker| worker.plans(&minute))
        {
            return Ok(minute.walks.get().cloned());
        }
        Ok(Some(Arc::clone(minute.walks(&session(&world).sim))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn a_busy_preview_skips_intermediate_requests_and_only_starts_the_latest() {
        let mut founded = crate::Founded::generated(9, 256).unwrap();
        founded.genesis.rules.foreground_walks = true;
        founded.genesis.rules.speed = Some(cosmograph::speed::Speed::froude(
            founded.genesis.world.base_unit_micrometres,
        ));
        let mut runtime = Runtime::bench(founded, 30).unwrap();
        let world = runtime.world().share();
        let first = runtime
            .driver
            .product_mut()
            .foreground_minute(&world)
            .unwrap()
            .unwrap();
        let obsolete = Arc::clone(&first);
        let snapshot = world.session_snapshot().unwrap();
        let (release, held) = mpsc::channel();
        let occupied = std::thread::spawn(move || {
            held.recv().unwrap();
            obsolete.walks(&snapshot.sim);
        });
        let id = occupied.thread().id();
        runtime.driver.product_mut().foreground.preview = Some(occupied);

        let mut requested = Vec::new();
        for _ in 0..3 {
            world.command(cosmograph::Command::Join).unwrap();
            assert!(runtime.poll_foreground_walks().unwrap().is_none());
            let foreground = &runtime.driver.product().foreground;
            assert_eq!(foreground.preview.as_ref().unwrap().thread().id(), id);
            requested.push(Arc::clone(&foreground.current.as_ref().unwrap().1));
        }
        release.send(()).unwrap();
        runtime
            .driver
            .product_mut()
            .foreground
            .preview
            .take()
            .unwrap()
            .join()
            .unwrap();
        assert!(first.walks.get().is_some());
        let latest = requested.pop().unwrap();
        assert!(
            runtime.poll_foreground_walks().unwrap().is_none(),
            "finished obsolete work stays hidden"
        );
        runtime
            .driver
            .product_mut()
            .foreground
            .preview
            .take()
            .unwrap()
            .join()
            .unwrap();
        let ready = runtime.poll_foreground_walks().unwrap().unwrap();
        assert!(Arc::ptr_eq(&ready, latest.walks.get().unwrap()));
        assert!(requested.iter().all(|older| older.walks.get().is_none()));
    }
}
