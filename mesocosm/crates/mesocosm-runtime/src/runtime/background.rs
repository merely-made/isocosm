// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round worked on another thread, so the window keeps drawing the last
//! world while the next tick computes. The result commits against the cut it
//! started from; if the world moved meanwhile, it is dropped and run again.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use cosmograph::Session;
use cosmograph_host::world::{Cut, NativeRevision, World};
use super::{Mesocosm, Runtime, foreground::MinuteWalks};
use crate::{continuation::VerifiedQuestion, spatial::{Game, GeometryCache}};

type Out = Result<(String, Vec<cosmograph::flows::Flow>, Option<VerifiedQuestion>), String>;

pub(crate) struct Pending { native: NativeRevision, cut: Cut, minute: Option<Arc<MinuteWalks>>, done: Receiver<Done> }

impl Pending {
    pub(super) fn plans(&self, minute: &Arc<MinuteWalks>) -> bool {
        self.minute.as_ref().is_some_and(|held| Arc::ptr_eq(held, minute))
    }
}

struct Done { session: Session, game: Game, geometry: GeometryCache, out: Out }

impl Mesocosm {
    /// Starts the round on a worker from the world as it stands.
    pub(super) fn begin_round(&mut self, world: &World) -> Result<(), String> {
        if self.pending.is_some() { return Ok(()); }
        self.sync_loop(world);
        let native = world.native_revision()?;
        let cut = world.cut()?;
        let minute = self.foreground_minute(world)?;
        let worker_minute = minute.clone();
        let snapshot = world.session_snapshot()?;
        let mut game = self.spatial.clone();
        let mut geometry = std::mem::take(&mut self.geometry);
        let participant = self.lp.participant;
        let (send, done) = mpsc::channel();
        std::thread::Builder::new().name("mesocosm-round".into()).spawn(move || {
            let mut session = snapshot.as_ref().clone();
            let first = worker_minute.as_deref().map(|minute| (minute, cut));
            let out = crate::continuation::round::advance_with_flows_cached(
                &mut session, &mut game, participant, &mut geometry, first).map_err(|why| format!("{why:?}"));
            let _ = send.send(Done { session, game, geometry, out });
        }).map_err(|why| why.to_string())?;
        self.pending = Some(Pending { native, cut, minute, done });
        Ok(())
    }

    fn finish_computation(&mut self, world: &World, cut: Cut, done: Done) -> Result<(), String> {
        let (_, flows, verified) = done.out?;
        world.product(cut, |session| {
            *session = done.session;
            Ok(())
        })?;
        self.geometry = done.geometry;
        self.finish_round(world, done.game, flows, verified)
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
        let pending = self.driver.product().pending.as_ref()?;
        self.banked_us = self.banked_us.saturating_add(elapsed_us);
        let result = match pending.done.try_recv() {
            Ok(done) => Ok(done),
            Err(TryRecvError::Empty) => return Some(0),
            Err(TryRecvError::Disconnected) => Err("the round's worker stopped".to_owned()),
        };
        let pending = self.driver.product_mut().pending.take().expect("the completed round");
        let world = self.driver.world().share();
        let native = match world.native_revision() {
            Ok(native) => native,
            Err(why) => {
                self.driver.restore_fault(Some(why));
                return Some(0);
            },
        };
        if native != pending.native {
            // A branch or saved native policy can change at an equal Cut.
            // Retry the authorised round through normal envelope admission.
            // A frame reading the old section after a native fork may have
            // recorded a transient validation fault. The saved loop carries
            // the accepted fault, which is the one the retry must respect.
            let fault = self.product().spatial.continuation.as_ref().and_then(|s| s.fault.clone());
            self.driver.product_mut().fault = fault.clone();
            self.driver.restore_fault(fault);
            let taken = self.driver.step(1);
            self.publish();
            return Some(if self.driver.product().pending.is_some() { 0 } else { taken });
        }
        let product = self.driver.product_mut();
        let outcome = result.and_then(|done| product.finish_computation(&world, pending.cut, done));
        if let Err(why) = outcome {
            let _ = product.refused(&world, why.clone());
            self.driver.restore_fault(Some(why));
        }
        self.publish();
        Some(1)
    }
}

#[cfg(test)]
mod tests;
