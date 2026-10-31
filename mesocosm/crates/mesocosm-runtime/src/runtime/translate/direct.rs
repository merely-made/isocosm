// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The shared direct lowering used by owned and borrowed Mesocosm drivers.

use cosmograph::{Session, Command, directing::{Aim, Toward}, schema::Id};
use cosmograph_overlay::mesocosm::{DevIntent, MesocosmIntent, NudgeMeaning,
    NudgeTarget, PlayerActKind};
use crate::Refusal;

pub fn apply(session: &mut Session, participant: Id, round: u64,
    intent: &MesocosmIntent) -> Result<String, Refusal> {
    if matches!(intent, MesocosmIntent::Dev(DevIntent::EndEpoch)) {
        let epoch = session.sim.genesis().rules.epoch_ticks.max(1);
        let left = epoch - session.sim.state().tick % epoch;
        let ticks = left.saturating_sub(round);
        return session.advance(ticks).map(|_| format!("advanced {ticks}")).map_err(Refusal::Sim);
    }
    let playing = |entity| (session.sim.plays(participant) == Some(entity))
        .then_some(()).ok_or(Refusal::NotPlayed);
    let command = match intent {
        MesocosmIntent::Nudge(nudge) => {
            playing(nudge.entity.0)?;
            let (aim, act) = match &nudge.meaning {
                NudgeMeaning::Attend => (Aim::Attend, None),
                NudgeMeaning::Act(key) => (Aim::Act, Some(key.0.clone())),
            };
            let toward = match nudge.target {
                NudgeTarget::Place(place) => Toward::Site(place.0),
                NudgeTarget::Thing(thing) => Toward::Thing(thing.0),
            };
            Ok(Command::Nudge { participant, critter: nudge.entity.0, aim, toward, act })
        },
        MesocosmIntent::Act(act) => {
            playing(act.entity.0)?;
            let PlayerActKind::Speciate { name } = &act.kind;
            Ok(Command::Speciate { founder: act.entity.0, name: name.clone() })
        },
        MesocosmIntent::Checkpoint(_) => Err(Refusal::Unasked),
        MesocosmIntent::Dev(dev) => Ok(match *dev {
            DevIntent::EndEpoch => return Err(Refusal::Unbuilt("end-epoch lowering".into())),
            DevIntent::PlaceMatter { site, mass_mg } => Command::PlaceMatter {
                site: site.0, account: "world:soil".into(), amount: mass_mg,
            },
            DevIntent::ForceBirth { organism } => Command::ForceBirth { parent: organism.0 },
            DevIntent::Kill { organism } => Command::Kill { entity: organism.0 },
        }),
    }?;
    session.command(command).map_err(Refusal::Sim)
}
