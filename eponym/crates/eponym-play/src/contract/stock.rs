// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Borrowed, Lowered};
use crate::WorldError;
use cosmograph::{Command, schema::Id};
use cosmograph_overlay::eponym::StockAct;

impl Borrowed<'_> {
    pub(super) fn stock_act(&mut self, participant: Option<Id>, act: &StockAct)
        -> Result<Lowered, WorldError>
    {
        let participant = participant.ok_or(WorldError::NotPlayed)?;
        if self.sim().plays(participant) != Some(act.subject.0) {
            return Err(WorldError::NotPlayed);
        }
        let selections: Vec<cosmograph::projects::DebitSelection> =
            serde_json::from_slice(&act.selections).map_err(|_| WorldError::Decode)?;
        let outcome = self.run(Command::ActOwned { participant, actor:act.subject.0,
            target:act.target.map(|target| target.0), process:act.process.0.clone(),
            cause:None, selections })?;
        let result: cosmograph::projects::OwnedActed =
            serde_json::from_str(&outcome).map_err(|_| WorldError::Decode)?;
        if !result.receipt.accepted() { return Err(WorldError::Refused); }
        Ok(Lowered { outcomes:vec![outcome], ..Default::default() })
    }
}
