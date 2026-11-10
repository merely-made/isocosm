// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Time flow (case 1216): the world runs ahead, unwatched, in the middle
//! of play. It is the unattended deep-time path, adapting every line at
//! each boundary, run on the live session; nothing in it is changed.

use super::{Interim, deep, readings::{self, Raised}};
use crate::{Result, history::Command, rules::DeepTimeSpan};

impl Interim {
    /// Runs the world ahead `epochs` boundaries of its clock with nobody at
    /// the keyboard, every line adapting at each (the deep-time path:
    /// `run_deep_time_adapting`), and says what its conditions raised
    /// (case 1216). From mid-epoch the first boundary comes sooner than a
    /// whole epoch. Every step is a logged command, so a saved session
    /// replays to the same hash. The epoch's ending begins again at the new
    /// tick. If the played critter is gone from the world, its first heir
    /// is taken up; a critter that is only dead is left to the next
    /// `round`, whose succession handles it as it does any death.
    ///
    /// # Errors
    /// Nobody is played and nothing is held, the world's epoch is not timed, the pace's scoring
    /// span is nought, the played critter is gone with no heir, or the run
    /// is refused; the loop is then untouched. A holder with no critter has
    /// no heir to take up and runs the same path.
    pub fn ahead(&mut self, epochs: u32) -> Result<Vec<Raised>> {
        let played = self.critter();
        if played.is_none() && self.session.sim.holds(self.participant).is_empty() {
            return Err("nobody is played and nothing is held".into());
        }
        let before = self.session.sim.clone();
        let mut run = self.session.clone();
        deep::run_deep_time_adapting(&mut run, DeepTimeSpan { epochs }, self.pace.scoring)?;
        if let Some(played) = played.filter(|played| run.sim.state().population.get(*played).is_none()) {
            let heir = readings::heirs(&run.sim, played).first().copied()
                .ok_or("the played critter is gone and has no heir")?;
            run.command(Command::Take { participant: self.participant, critter: heir })?;
        }
        self.session = run;
        if epochs != 0 { self.ending.begin(self.session.sim.state().tick); }
        Ok(readings::raised(&before, &self.session.sim))
    }
}
