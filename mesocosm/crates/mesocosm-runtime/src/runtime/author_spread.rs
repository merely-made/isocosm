// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit author startup, separate from ordinary product packets.

use super::{Founded, Runtime};
use crate::spatial::Game;
use isocosm::{Command, Session, asserted::Assertion,
    projects::{DebitSelection, SpreadPreview, StockLot}, spread::{self, Identity, Substrate}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    pub version: u32,
    pub founded: Founded,
    pub ownership: Vec<Ownership>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ownership { pub identity: Identity, pub lot: StockLot }

/// The author supplies every selected lot portion and unowned quantity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Step {
    Overflow { identity: Identity, at: Substrate, selections: Vec<DebitSelection> },
    ForkStrain { parent: String, at: Substrate, amount: u64, traits: BTreeSet<String>,
        selections: Vec<DebitSelection> },
}

impl Step {
    fn command(&self) -> (Command, &[DebitSelection]) {
        match self {
            Self::Overflow { identity, at, selections } =>
                (Command::Spread(spread::Command::Overflow { identity: identity.clone(), at: *at }), selections),
            Self::ForkStrain { parent, at, amount, traits, selections } =>
                (Command::Spread(spread::Command::ForkStrain { parent: parent.clone(), at: *at,
                    amount: *amount, traits: traits.clone() }), selections),
        }
    }
}

impl Setup {
    pub fn check(&self) -> Result<(), String> {
        if self.version != VERSION { return Err("unsupported authored Mesocosm spread setup".into()); }
        if self.founded.pace.round == 0 || self.founded.pace.scoring == 0 {
            return Err("authored Mesocosm pace must have positive round and scoring ticks".into());
        }
        self.founded.genesis.validate()?;
        let mut keys = BTreeSet::new();
        for owner in &self.ownership {
            if !keys.insert(&owner.lot.key) { return Err("authored spread setup repeats an ownership key".into()); }
        }
        Ok(())
    }

    fn candidate(&self, session: &Session) -> Result<Session, String> {
        self.check()?;
        if session.sim.genesis() != &self.founded.genesis {
            return Err("authored spread setup names another native genesis".into());
        }
        let mut native = session.clone();
        for owner in &self.ownership {
            native.command(Command::Assert(Assertion::SpreadOwnership {
                identity: owner.identity.clone(), lot: owner.lot.clone() }))?;
        }
        Ok(native)
    }

    /// Read the exact candidate after assertions and admitted earlier selected steps.
    pub fn prepare_step(&self, session: &Session, index: usize) -> Result<SpreadPreview, String> {
        let step = self.steps.get(index).ok_or("authored spread step index is absent")?;
        let mut native = self.candidate(session)?;
        for earlier in &self.steps[..index] { admit_step(&mut native, earlier)?; }
        native.prepare_owned_spread(&step.command().0)
    }

    /// The shared host verifies this founding; it never replaces its native world.
    pub fn apply(&self, session: &mut Session, game: &mut Game) -> Result<Vec<String>, String> {
        game.validate(session)?;
        let mut native = self.candidate(session)?;
        let mut spatial = game.clone();
        let mut outcomes = Vec::new();
        for step in &self.steps {
            outcomes.push(admit_step(&mut native, step)?);
        }
        spatial.refresh(&mut native)?;
        crate::continuation::rebind(&native, &mut spatial);
        spatial.validate(&native)?;
        *session = native;
        *game = spatial;
        Ok(outcomes)
    }
}

fn admit_step(native: &mut Session, step: &Step) -> Result<String, String> {
    let (command, selections) = step.command();
    let preview = native.prepare_owned_spread(&command)?;
    if let Some(why) = preview.debits.refusal { return Err(why); }
    if preview.debits.requests.len() != selections.len() || preview.debits.requests.iter()
        .any(|request| selections.iter().filter(|selection| &selection.request == request).count() != 1) {
        return Err("authored spread step must select every exact current debit request".into());
    }
    native.command(Command::Owned { command: Box::new(command), selections: selections.to_vec() })
}

impl Runtime {
    /// Founds from explicit native inputs and publishes only fully admitted startup.
    pub fn from_authored_spread(setup: &Setup, ticks_per_second: u32) -> Result<Self, String> {
        setup.check()?;
        let mut runtime = Self::new(setup.founded.clone(), ticks_per_second)?;
        setup.apply(&mut runtime.interim.session, &mut runtime.spatial)?;
        Ok(runtime)
    }
}
