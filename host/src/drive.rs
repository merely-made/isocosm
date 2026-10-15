// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The core's driver (sim-host plan H3, rulings 926 and 927). A product's
//! envelopes queue and apply in order before each round, each kept in the
//! trace with what it came to; a ceiling bounds the rounds one advance runs;
//! rounds are paced by the core's clock or only by explicit steps. While a
//! question holds the run nothing else runs: answers at the front of the
//! queue drain, everything else waits, and no time banks. The product says
//! what an envelope does, whether a question holds and what answers it, and
//! what one round is; the world is the core's.

use std::collections::VecDeque;

use crate::world::World;

pub mod clock;
pub use clock::{Advance, Clock};

/// Default ceiling on rounds one advance runs.
pub const DEFAULT_MAX_STEPS: u64 = 8;

/// How a driver's rounds are paced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pacing {
    /// Fixed steps at this rate: `advance` runs the rounds elapsed wall time authorises.
    Clock { ticks_per_second: u32 },
    /// Rounds run only through `step`.
    Explicit,
}

/// What a product supplies to the core's driver.
pub trait Product {
    type Envelope: Clone;
    type Refusal: Clone;
    /// Applies one envelope to the world; the trace keeps what it came to.
    fn apply(&mut self, world: &World, envelope: &Self::Envelope) -> Result<String, Self::Refusal>;
    /// Whether a question holds the run.
    fn held(&self, world: &World) -> bool;
    /// Whether `envelope` answers the question holding the run.
    fn answers(&self, world: &World, envelope: &Self::Envelope) -> bool;
    /// One round of the product's loop. An error stops the run until a
    /// restored fault clears it.
    fn round(&mut self, world: &World) -> Result<(), String>;
}

pub struct Driver<P: Product> {
    world: World,
    product: P,
    clock: Option<Clock>,
    queued: VecDeque<P::Envelope>,
    trace: Vec<(P::Envelope, Result<String, P::Refusal>)>,
    fault: Option<String>,
    max_steps: u64,
}

impl<P: Product> Driver<P> {
    pub fn new(world: &World, product: P, pacing: Pacing) -> Self {
        let clock = match pacing {
            Pacing::Clock { ticks_per_second } => Some(Clock::new(ticks_per_second)),
            Pacing::Explicit => None,
        };
        Self { world: world.share(), product, clock, queued: VecDeque::new(), trace: Vec::new(), fault: None,
            max_steps: DEFAULT_MAX_STEPS }
    }

    pub fn with_max_steps(mut self, max_steps: u64) -> Self {
        assert!(max_steps > 0, "at least one step per advance");
        self.max_steps = max_steps;
        self
    }

    pub fn world(&self) -> &World { &self.world }
    pub fn product(&self) -> &P { &self.product }
    pub fn product_mut(&mut self) -> &mut P { &mut self.product }
    pub fn clock(&self) -> Option<&Clock> { self.clock.as_ref() }

    pub fn queue(&mut self, envelope: P::Envelope) { self.queued.push_back(envelope); }
    pub fn queued(&self) -> impl Iterator<Item = &P::Envelope> { self.queued.iter() }
    pub fn queued_len(&self) -> usize { self.queued.len() }
    /// Replaces the queue, as a resumed save restores it.
    pub fn restore_queue(&mut self, queued: impl IntoIterator<Item = P::Envelope>) {
        self.queued = queued.into_iter().collect();
    }

    /// Every envelope applied, in order, with what it came to.
    pub fn trace(&self) -> &[(P::Envelope, Result<String, P::Refusal>)] { &self.trace }

    /// Why a round was refused, if one was: the run has stopped there.
    pub fn fault(&self) -> Option<&str> { self.fault.as_deref() }
    /// Restores a saved fault, which keeps the run stopped, or clears it.
    pub fn restore_fault(&mut self, fault: Option<String>) { self.fault = fault; }

    /// Runs the rounds the elapsed time authorises. A held run drains its
    /// answers and banks no time; an explicitly paced driver runs none here.
    pub fn advance(&mut self, elapsed_us: u64) -> u64 {
        if self.held() {
            self.drain_answers();
            return 0;
        }
        let Some(clock) = &mut self.clock else { return 0 };
        let steps = clock.advance(elapsed_us, self.max_steps).steps;
        self.step(steps)
    }

    /// Runs up to `steps` rounds; fewer while a question holds or after a fault.
    pub fn step(&mut self, steps: u64) -> u64 {
        let mut taken = 0;
        for _ in 0..steps {
            if !self.step_once() { break; }
            taken += 1;
        }
        taken
    }

    fn held(&self) -> bool { self.product.held(&self.world) }

    fn apply(&mut self, envelope: P::Envelope) {
        let outcome = self.product.apply(&self.world, &envelope);
        self.trace.push((envelope, outcome));
    }

    /// Applies queued answers while a question holds; anything else waits.
    fn drain_answers(&mut self) {
        while let Some(front) = self.queued.front() {
            if !self.product.answers(&self.world, front) { return; }
            let envelope = self.queued.pop_front().expect("a front");
            self.apply(envelope);
        }
    }

    /// One round after the queued envelopes; `false` while a question holds
    /// that nothing queued answers, or after a fault. An envelope that opens
    /// a question ends the step there, before the round.
    fn step_once(&mut self) -> bool {
        if self.fault.is_some() { return false; }
        if self.held() {
            self.drain_answers();
            if self.held() { return false; }
        }
        while let Some(envelope) = self.queued.pop_front() {
            self.apply(envelope);
            if self.held() { return true; }
        }
        match self.product.round(&self.world) {
            Ok(()) => true,
            Err(why) => { self.fault = Some(why); false },
        }
    }
}
