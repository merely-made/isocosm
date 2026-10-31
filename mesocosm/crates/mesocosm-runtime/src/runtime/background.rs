// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round worked on another thread, so the window keeps drawing the last
//! world while the next tick computes. The result commits against the cut it
//! started from; if the world moved meanwhile, it is dropped and run again.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use cosmograph::Session;
use cosmograph_host::world::{Cut, World};
use super::{Mesocosm, Runtime, session};
use crate::{continuation::VerifiedQuestion, spatial::{Game, GeometryCache}};

type Out = Result<(String, Vec<cosmograph::flows::Flow>, Option<VerifiedQuestion>), String>;

pub(crate) struct Pending { cut: Cut, done: Receiver<Done> }

struct Done { session: Session, game: Game, geometry: GeometryCache, out: Out }

impl Mesocosm {
    /// Starts the round on a worker from the world as it stands.
    pub(super) fn begin_round(&mut self, world: &World) -> Result<(), String> {
        if self.pending.is_some() { return Ok(()); }
        self.sync_loop(world);
        let cut = world.cut()?;
        let mut session = session(world).clone();
        let mut game = self.spatial.clone();
        let mut geometry = std::mem::take(&mut self.geometry);
        let participant = self.lp.participant;
        let (send, done) = mpsc::channel();
        std::thread::Builder::new().name("mesocosm-round".into()).spawn(move || {
            let out = crate::continuation::round::advance_with_flows_cached(
                &mut session, &mut game, participant, &mut geometry).map_err(|why| format!("{why:?}"));
            let _ = send.send(Done { session, game, geometry, out });
        }).map_err(|why| why.to_string())?;
        self.pending = Some(Pending { cut, done });
        Ok(())
    }

    /// A finished round committed: `None` while it works or when the world
    /// moved under it, else the round's outcome.
    fn collect_round(&mut self, world: &World) -> Option<Result<(), String>> {
        let pending = self.pending.as_ref()?;
        let done = match pending.done.try_recv() {
            Ok(done) => done,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                return Some(self.refused(world, "the round's worker stopped".into()));
            },
        };
        let cut = pending.cut;
        self.pending = None;
        self.geometry = done.geometry;
        let (_, flows, verified) = match done.out {
            Ok(out) => out,
            Err(why) => return Some(self.refused(world, why)),
        };
        let mut computed = Some(done.session);
        let landed = world.product(cut, |session| {
            *session = computed.take().ok_or("the round commits once")?;
            Ok(())
        });
        landed.ok()?;
        Some(self.finish_round(world, done.game, flows, verified))
    }
}

impl Runtime {
    /// Runs rounds on a worker thread from the clock's `advance`; `step`
    /// stays synchronous for scripts.
    pub fn with_background(mut self) -> Self {
        self.driver.product_mut().background = true;
        self
    }

    /// The clock's advance while a worker round is out: commit it if it is
    /// done, and bank the time that passed.
    pub(super) fn advance_pending(&mut self, elapsed_us: u64) -> Option<u64> {
        self.driver.product().pending.as_ref()?;
        self.banked_us = self.banked_us.saturating_add(elapsed_us);
        let world = self.driver.world().share();
        Some(match self.driver.product_mut().collect_round(&world) {
            None => 0,
            Some(outcome) => {
                if let Err(why) = outcome { self.driver.restore_fault(Some(why)); }
                self.publish();
                1
            },
        })
    }
}
