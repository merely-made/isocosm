// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! VTT admission metadata beside the mode's existing uninterpreted game data.
//! Native history/body/control remain exclusively in the borrowed Session.

use super::{CharacterAdmission, CharacterControl, CharacterPlacement, ControlBinding,
    PositionBinding, RecipeBinding, admit_character, submit_contract_assertions,
    placement::{place_character, take_character}};
use isocosm::{Session, schema::{Id, Key}};
use isocosm_overlay::{OrdinaryAccess, Seat, vtt::{Assertion, VttIntentEnvelope}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const PREFIX: &[u8] = b"ISOCOSM-VTT-CHARACTERS\n";
const VERSION: u32 = 1;
const PLACEMENT_VERSION: u32 = 2;

/// The actual host input after its DM/seat/current-subject checks. Assertions
/// precede the character admissions, which name their existing authored keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionBatch {
    pub version: u32,
    pub assertions: Vec<Assertion>,
    pub characters: Vec<CharacterAdmission>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub placements: Vec<CharacterPlacement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<CharacterControl>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Input { Assertions(Vec<Assertion>), Admissions(AdmissionBatch), Play(VttIntentEnvelope) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameBindings {
    version: u32,
    characters: BTreeMap<Key, RecipeBinding>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    positions: BTreeMap<Key, PositionBinding>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    controls: BTreeMap<Id, ControlBinding>,
    /// The prior table/game payload stays byte-exact and grants no authority.
    payload: Vec<u8>,
}

impl GameBindings {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if let Some(bytes) = bytes.strip_prefix(PREFIX) {
            let game: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
            if !matches!(game.version, VERSION | PLACEMENT_VERSION)
                || game.version == VERSION && (!game.positions.is_empty() || !game.controls.is_empty())
            { return Err("unsupported VTT character binding version".into()); }
            Ok(game)
        } else {
            // The previous product payload has no bindings owned by this
            // format. Preserve it without guessing its schema or migration.
            Ok(Self { version:VERSION, characters:BTreeMap::new(), positions:BTreeMap::new(),
                controls:BTreeMap::new(), payload:bytes.to_vec() })
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let mut bytes = PREFIX.to_vec();
        bytes.extend(serde_json::to_vec(self).map_err(|error| error.to_string())?);
        Ok(bytes)
    }

    pub fn characters(&self) -> &BTreeMap<Key, RecipeBinding> { &self.characters }
    pub fn positions(&self) -> &BTreeMap<Key, PositionBinding> { &self.positions }
    pub fn controls(&self) -> &BTreeMap<Id, ControlBinding> { &self.controls }
    pub fn payload(&self) -> &[u8] { &self.payload }

    pub(super) fn record_position(&mut self, character: Key, position: PositionBinding) {
        self.version = PLACEMENT_VERSION;
        self.positions.insert(character, position);
    }

    pub fn validate(&self, session: &Session) -> Result<(), String> {
        for (key, binding) in &self.characters { binding.validate(session, key)?; }
        for (key, binding) in &self.positions {
            let recipe = self.characters.get(key).ok_or("position has no admitted character binding")?;
            binding.validate(session, key, recipe)?;
        }
        for (participant, binding) in &self.controls {
            if *participant != binding.participant { return Err("control participant key differs".into()); }
            let (key, recipe) = self.characters.iter().find(|(_, recipe)| recipe.entity == binding.entity)
                .ok_or("control has no admitted character binding")?;
            recipe.validate(session, key)?;
            binding.validate(session, recipe)?;
        }
        Ok(())
    }
}

/// Returns native accepted state plus product bytes for World::product_game.
/// The caller publishes the pair atomically; failure changes neither input.
/// Existing assertion arrays retain their old behavior and exact game bytes.
pub fn submit_game(session: &mut Session, game: Vec<u8>, bytes: &[u8])
    -> Result<(String, Vec<u8>), String> {
    submit(session, game, bytes, None)
}

/// Actual DM access supplies its native participant identity; owner metadata
/// is not a substitute. The shared host admits the seat before calling here.
pub fn submit_game_as(session: &mut Session, game: Vec<u8>, bytes: &[u8], dm: Id)
    -> Result<(String, Vec<u8>), String> {
    if !session.sim.state().population.get(dm).is_some_and(isocosm::directing::is_participant) {
        return Err("VTT admission caller is not a native participant".into());
    }
    submit(session, game, bytes, Some(dm))
}

/// The actual shared host supplies its already admitted seat and native cut.
/// Player play is separate from DM assertion/body/control authoring.
pub fn submit_game_access(session: &mut Session, game: Vec<u8>, bytes: &[u8], access: OrdinaryAccess)
    -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    let input: Input = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    match input {
        Input::Play(envelope) => super::submit_table(session, game, access, &envelope),
        _ if access.seat != Seat::Dm => Err("native table assertions and admissions require the DM seat".into()),
        input => submit_decoded(session, game, input, Some(access.participant.0)),
    }
}

fn submit(session: &mut Session, game: Vec<u8>, bytes: &[u8], dm: Option<Id>)
    -> Result<(String, Vec<u8>), String> {
    let input: Input = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    submit_decoded(session, game, input, dm)
}

fn submit_decoded(session: &mut Session, game: Vec<u8>, input: Input, dm: Option<Id>)
    -> Result<(String, Vec<u8>), String> {
    let batch = match input {
        Input::Assertions(assertions) => {
            submit_contract_assertions(session, &assertions).map_err(|error| error.to_string())?;
            return Ok(("asserted".into(), game));
        },
        Input::Admissions(batch) => batch,
        Input::Play(_) => return Err("table play requires a resolved native seat cut".into()),
    };
    if !matches!(batch.version, VERSION | PLACEMENT_VERSION)
        || batch.version == VERSION && (!batch.placements.is_empty() || !batch.controls.is_empty())
    { return Err("unsupported VTT admission input version".into()); }
    if !batch.controls.is_empty() && dm.is_none() {
        return Err("native character control needs the admitted DM participant".into());
    }
    let mut candidate = session.clone();
    submit_contract_assertions(&mut candidate, &batch.assertions).map_err(|error| error.to_string())?;
    if batch.characters.is_empty() && batch.placements.is_empty() && batch.controls.is_empty() {
        *session = candidate;
        return Ok(("asserted".into(), game));
    }
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut accepted = Vec::new();
    for (index, character) in batch.characters.iter().enumerate() {
        if bindings.characters.contains_key(&character.character) {
            return Err(format!("character {index}: a recipe admission is already recorded for this key"));
        }
        let binding = admit_character(&mut candidate, character)
            .map_err(|reason| format!("character {index} ({}): {reason}", character.character))?;
        accepted.push(format!("{} ({})", character.character, binding.entity));
        bindings.characters.insert(character.character.clone(), binding);
    }
    let mut positioned = BTreeSet::new();
    for (index, placement) in batch.placements.iter().enumerate() {
        if !positioned.insert(&placement.character) { return Err("duplicate character placement".into()); }
        let recipe = bindings.characters.get(&placement.character)
            .ok_or("placement needs an explicitly admitted character")?;
        let binding = place_character(&mut candidate, placement, recipe)
            .map_err(|reason| format!("placement {index} ({}): {reason}", placement.character))?;
        accepted.push(format!("placed {}", placement.character));
        bindings.positions.insert(placement.character.clone(), binding);
    }
    for (index, control) in batch.controls.iter().enumerate() {
        let recipe = bindings.characters.get(&control.character)
            .ok_or("control needs an explicitly admitted character")?;
        let binding = take_character(&mut candidate, control, recipe,
            dm.ok_or("native control has no admitted DM participant")?)
            .map_err(|reason| format!("control {index} ({}): {reason}", control.character))?;
        accepted.push(format!("participant:{} took {}", binding.participant, control.character));
        bindings.controls.insert(binding.participant, binding);
    }
    if !batch.placements.is_empty() || !batch.controls.is_empty() { bindings.version = PLACEMENT_VERSION; }
    bindings.validate(&candidate)?;
    let next = bindings.encode()?;
    *session = candidate;
    let label = if batch.placements.is_empty() && batch.controls.is_empty() { "developed" } else { "accepted" };
    Ok((format!("{label}: {}", accepted.join(", ")), next))
}
