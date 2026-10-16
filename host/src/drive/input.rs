// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Live input through the core (sim-host plan H4, rulings 954 to 957). The
//! window lowers its platform key into a [`Press`] and its pixel pick into a
//! [`Pick`] stamped with the world revision of the frame it was drawn from.
//! The product's keymap turns a press, and a current pick, into an envelope;
//! keys that only move host state never come here (955). A pick from a frame
//! the world has moved past is refused (956), and live input is refused once
//! the backlog reaches its cap (957); a replay queueing directly is not.

use super::{Driver, Product};

/// Default live-input backlog: envelopes queued before a new press is refused.
pub const DEFAULT_MAX_BACKLOG: usize = 10;

/// A key, lowered from the window's platform key in one place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Press {
    Named(Named),
    /// A printable key, lowercased.
    Char(char),
}

impl Press {
    /// The press for `c`, lowercased.
    pub fn char(c: char) -> Self { Self::Char(c.to_ascii_lowercase()) }
}

/// The named keys a keymap can claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Named { Space, Enter, Tab, Escape, ArrowUp, ArrowDown, ArrowLeft, ArrowRight }

/// What the window found under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick { Thing(u64), Place(u64), Nothing }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button { Primary, Secondary }

/// A pick and the world revision of the frame it was picked from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pointed {
    pub pick: Pick,
    pub button: Button,
    pub revision: u64,
}

/// Host state a keymap reads but does not own: the followed thing, the
/// selected row of a standing list, and whether dev keys are live.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Context {
    pub focus: Option<u64>,
    pub row: usize,
    pub dev: bool,
}

/// Why live input was not taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputRefusal {
    /// The backlog is at its cap.
    Backlog { queued: usize },
    /// The pick came from a frame the world has moved past.
    Stale { picked: u64, current: u64 },
    /// The world could not be read.
    World(String),
}

impl<P: Product> Driver<P> {
    pub fn max_backlog(&self) -> usize { self.max_backlog }
    pub fn set_max_backlog(&mut self, max_backlog: usize) { self.max_backlog = max_backlog; }

    /// Queues live input unless the backlog is at its cap.
    pub fn offer(&mut self, envelope: P::Envelope) -> Result<(), InputRefusal> {
        if self.queued.len() >= self.max_backlog {
            return Err(InputRefusal::Backlog { queued: self.queued.len() });
        }
        self.queue(envelope);
        Ok(())
    }

    /// Dispatches a press through the product's keymap. `Ok(false)` when the
    /// keymap does not claim it.
    pub fn press(&mut self, press: &Press, context: &Context) -> Result<bool, InputRefusal> {
        match self.product.press(&self.world, press, context) {
            Some(envelope) => self.offer(envelope).map(|()| true),
            None => Ok(false),
        }
    }

    /// The pick, if its frame is still the world's current revision.
    pub fn current(&self, pointed: &Pointed) -> Result<Pick, InputRefusal> {
        let current = self.world.revision().map_err(InputRefusal::World)?;
        if pointed.revision != current {
            return Err(InputRefusal::Stale { picked: pointed.revision, current });
        }
        Ok(pointed.pick)
    }

    /// Dispatches a current pick through the product's keymap. `Ok(false)`
    /// when the product makes no envelope of it.
    pub fn point(&mut self, pointed: &Pointed, context: &Context) -> Result<bool, InputRefusal> {
        let pick = self.current(pointed)?;
        match self.product.point(&self.world, pick, pointed.button, context) {
            Some(envelope) => self.offer(envelope).map(|()| true),
            None => Ok(false),
        }
    }
}
