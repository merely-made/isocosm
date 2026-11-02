// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A world played in rounds (cases 1095, 1098 to 1100, 1102). A rounds
//! marker in the founding's intent log declares spatial templates in the
//! sim, so its bodies hold native poses; motion stays the game's, and the
//! sim ticks only at a minute intent, where each body's pose is handed back
//! and admitted at cell grain. A world without the marker plays and replays
//! exactly as before.

use super::*;
use cosmograph::operations::Window;
use cosmograph::spatial::{ArrivalGeometry, BirthResolution, Handback, Pose, SpatialAdvance, Templates};

/// Root pivot to eye for a handed-back stance, in cells.
const EYE: [i64; 3] = [0, 1, 0];

impl World {
    /// Whether this world plays in rounds.
    pub fn rounds(&self) -> bool {
        self.intents.iter().any(|i| matches!(i, WorldIntent::Rounds))
    }

    /// Re-founds this world in rounds: the same founding, the sim's genesis
    /// declaring templates, and the marker appended to its founding intents.
    /// Only a freshly founded world, before any play, may take it.
    pub fn in_rounds(self) -> Result<Self, WorldError> {
        let profile = |i: &WorldIntent| matches!(i, WorldIntent::MaterialProfile { .. }
            | WorldIntent::OwnedMaterialProfile { .. } | WorldIntent::SpreadMaterialProfile { .. });
        if self.rounds() || !self.intents.iter().all(profile) || self.last_tick.is_some() {
            return Err(WorldError::Refused);
        }
        let mut genesis = self.session.sim.genesis().clone();
        let side = self.config.extent as i64 * 2;
        genesis.rules.spatial_templates = Some(Templates {
            version: 1,
            window: Window { min: [0, 0], max: [side, side], depth: DEPTH },
            places: Default::default(),
            eye: EYE,
            stands: BTreeMap::new(),
            crossings: Vec::new(),
            foraging: BTreeMap::new(),
            body_spacing: 0,
            walking: None,
        });
        genesis.validate().map_err(|_| WorldError::Refused)?;
        let mut session = Session::new_with_movement(genesis, Execution::Individuals, self.session.sim.state().movement)
            .map_err(|_| WorldError::Refused)?;
        for entry in &self.session.entries {
            let outcome = session.command(entry.command.clone()).map_err(|_| WorldError::Refused)?;
            if outcome != entry.outcome {
                return Err(WorldError::Refused);
            }
        }
        let base_hash = hash_bytes(cosmograph::digest(session.sim.genesis()).as_bytes());
        let mut intents = self.intents.clone();
        intents.push(WorldIntent::Rounds);
        Ok(Self { base_hash, session, intents, ..self })
    }

    /// The sim's minute in a round: each handback checked and admitted, then
    /// one native tick. Birth placements are the game's to make later; a
    /// born body arrives unplaced.
    pub fn minute(&mut self, handbacks: &[Handback]) -> Result<Vec<cosmograph::spatial::HandbackReport>, WorldError> {
        if !self.rounds() {
            return Err(WorldError::Refused);
        }
        let mut births = Vec::new();
        loop {
            match self.session.try_advance_round_spatial(1, &[], &[], &[], &births, handbacks)
                .map_err(|_| WorldError::Refused)? {
                SpatialAdvance::Advanced(_) => break,
                SpatialAdvance::Birth(pending) => births.push(BirthResolution {
                    request: pending.request, geometry: Err("Eponym places births in play".into()) }),
                _ => return Err(WorldError::Refused),
            }
        }
        Ok(self.session.sim.spatial_handbacks().to_vec())
    }

    /// The handback for a body standing at site cell `at`.
    pub fn handback(&self, entity: Id, at: [i64; 3]) -> Option<Handback> {
        let t = self.session.sim.genesis().rules.spatial_templates.as_ref()?;
        let e = self.session.sim.state().population.get(entity)?;
        Some(Handback {
            actor: entity, site: self.site, body_revision: e.body_revision,
            terrain_revision: self.session.sim.terrain_revision(),
            geometry: ArrivalGeometry {
                window: t.window.clone(), places: t.places,
                pose: Pose { at, yaw: Default::default(), feet: Default::default() },
                eye: std::array::from_fn(|k| at[k] + t.eye[k]),
            },
            contacts: Vec::new(),
        })
    }
}
