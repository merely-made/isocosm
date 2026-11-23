// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ordinary view and the round's worker read one participant's walks
//! from the same accepted native world. The cache never enters a save.

use std::sync::{Arc, OnceLock};

use cosmograph::{Session, Simulation, schema::Id};
use cosmograph_host::{
    drive::{Walk, walk_handbacks, world_foreground_walks},
    world::{Cut, NativeRevision, World},
};

use super::{Mesocosm, Runtime, session};

#[derive(Default)]
pub(super) struct Foreground {
    current: Option<(NativeRevision, Arc<MinuteWalks>)>,
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
}

/// The worker receives only this minute's slot. A later accepted publication
/// can replace the view's slot while the old worker finishes independently.
pub(crate) struct MinuteWalks {
    participant: Id,
    cut: Cut,
    walks: OnceLock<Arc<[Walk]>>,
}

impl MinuteWalks {
    fn walks(&self, sim: &Simulation) -> &Arc<[Walk]> {
        self.walks
            .get_or_init(|| world_foreground_walks(sim, self.participant).into())
    }

    pub(crate) fn matches(&self, session: &Session, participant: Id) -> bool {
        self.participant == participant
            && self.cut.tick == session.sim.state().tick
            && self.cut.entries == session.entries.len()
            && self.cut.state_hash == session.sim.state_hash()
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
            cut: world.cut()?,
            walks: OnceLock::new(),
        });
        self.foreground.current = Some((revision, Arc::clone(&minute)));
        Ok(Some(minute))
    }
}

impl Runtime {
    /// The current native foreground for this participant. While the worker
    /// prepares that same minute a frame can keep drawing native positions.
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
