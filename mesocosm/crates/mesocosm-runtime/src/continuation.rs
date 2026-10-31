// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Saved product holds over the host's actual native Session.

mod types;
mod read;
pub(crate) mod admission;
pub(crate) mod round;
pub(crate) mod scoring;
pub(crate) mod restore;
mod verified;
pub(crate) use verified::{VerifiedQuestion, validate_replayed_question};
pub(crate) use admission::answer_verified;
pub use types::*;
pub use read::read_loop;
pub use admission::{answer_loop, configure_loop, submit_loop};
pub use round::advance_loop;

use crate::spatial::Game;
use cosmograph::Session;

pub const VERSION: u32 = 1;
pub const KIND: &str = "mesocosm:loop";

pub(crate) fn physical(game: &Game) -> Game {
    let mut physical = game.clone();
    physical.continuation = None;
    physical.version = physical.current_version();
    physical
}

pub fn cut(session: &Session, game: &Game) -> LoopCut {
    LoopCut { native: cosmograph::digest(&session.save()), game: cosmograph::digest(game),
        question: game.continuation.as_ref().and_then(Continuation::question) }
}

impl Continuation {
    pub(crate) fn seal(&mut self, session: &Session, game: &Game) {
        let _span = crate::profile::Span::new("continuation.seal");
        self.native = crate::profile::measure("seal.native", || prefix(session));
        self.native_entries = session.entries.len() as u64;
        self.native_tick = session.sim.state().tick;
        self.physical = crate::profile::measure("seal.physical", || game.physical_digest());
    }

    pub fn question(&self) -> Option<String> {
        self.checkpoint.as_ref().map(|_| cosmograph::digest(&(
            &self.native, self.participant, &self.phase, &self.checkpoint,
            &self.review, &self.proposed)))
    }

    pub fn validate_session(&self, session: &Session, game: &Game) -> Result<(), String> {
        let _span = crate::profile::Span::new("continuation.validate");
        let count = usize::try_from(self.native_entries).map_err(|_| "loop source count overflow")?;
        if self.version != VERSION || self.native_tick != session.sim.state().tick
            || count > session.entries.len()
            || self.native != cosmograph::digest(&(&session.branch, session.sim.genesis(),
                &session.entries[..count], self.native_tick))
            || session.entries[count..].iter().any(|entry| !matches!(&entry.command, cosmograph::Command::Window(_)))
            || self.physical != game.physical_digest() {
            return Err("saved loop names another native/product cut".into());
        }
        if self.settings.pace.round == 0 || self.settings.pace.scoring == 0
            || !session.sim.state().population.get(self.participant)
                .is_some_and(cosmograph::directing::is_participant) {
            return Err("saved loop participant or explicit pace invalid".into());
        }
        restore::validate_question(self, session)
    }
}

impl Game {
    /// Exact digest of the existing physical Game encoding. The final ecology
    /// field is unchanged admission evidence, so serialize it once per value.
    /// Every mutable Seal access invalidates that derivative byte cache.
    pub fn physical_digest(&self) -> String {
        use sha2::{Digest,Sha256};
        let mut physical = physical(self);
        let ecology = physical.ecology.take();
        let prefix = serde_json::to_vec(&physical).expect("physical game serializes");
        let mut hash = Sha256::new();
        hash.update(&prefix[..prefix.len()-1]);
        if let Some(ecology) = ecology {
            hash.update(b",\"ecology\":");
            hash.update(ecology.json());
        }
        hash.update(b"}");
        format!("{hash:x}",hash=hash.finalize())
    }
}

pub(crate) fn prefix(session: &Session) -> String {
    cosmograph::digest(&(&session.branch, session.sim.genesis(), &session.entries,
        session.sim.state().tick))
}

pub(crate) fn rebind(session: &Session, game: &mut Game) {
    if let Some(mut saved) = game.continuation.take() {
        saved.seal(session, game);
        game.continuation = Some(saved);
        game.version = game.current_version();
    }
}
