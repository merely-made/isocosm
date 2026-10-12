// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! VTT admission metadata beside the mode's existing uninterpreted game data.
//! Native history/body/control remain exclusively in the borrowed Session.

use super::{CharacterAdmission, RecipeBinding, admit_character, submit_contract_assertions};
use isocosm::{Session, schema::Key};
use isocosm_overlay::vtt::Assertion;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PREFIX: &[u8] = b"ISOCOSM-VTT-CHARACTERS\n";
const VERSION: u32 = 1;

/// The actual host input after its DM/seat/current-subject checks. Assertions
/// precede the character admissions, which name their existing authored keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionBatch {
    pub version: u32,
    pub assertions: Vec<Assertion>,
    pub characters: Vec<CharacterAdmission>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Input { Assertions(Vec<Assertion>), Admissions(AdmissionBatch) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameBindings {
    version: u32,
    characters: BTreeMap<Key, RecipeBinding>,
    /// The prior table/game payload stays byte-exact and grants no authority.
    payload: Vec<u8>,
}

impl GameBindings {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if let Some(bytes) = bytes.strip_prefix(PREFIX) {
            let game: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
            if game.version != VERSION { return Err("unsupported VTT character binding version".into()); }
            Ok(game)
        } else {
            // The previous product payload has no bindings owned by this
            // format. Preserve it without guessing its schema or migration.
            Ok(Self { version:VERSION, characters:BTreeMap::new(), payload:bytes.to_vec() })
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let mut bytes = PREFIX.to_vec();
        bytes.extend(serde_json::to_vec(self).map_err(|error| error.to_string())?);
        Ok(bytes)
    }

    pub fn characters(&self) -> &BTreeMap<Key, RecipeBinding> { &self.characters }
    pub fn payload(&self) -> &[u8] { &self.payload }

    pub fn validate(&self, session: &Session) -> Result<(), String> {
        for (key, binding) in &self.characters { binding.validate(session, key)?; }
        Ok(())
    }
}

/// Returns native accepted state plus product bytes for World::product_game.
/// The caller publishes the pair atomically; failure changes neither input.
/// Existing assertion arrays retain their old behavior and exact game bytes.
pub fn submit_game(session: &mut Session, game: Vec<u8>, bytes: &[u8])
    -> Result<(String, Vec<u8>), String> {
    let input: Input = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let batch = match input {
        Input::Assertions(assertions) => {
            submit_contract_assertions(session, &assertions).map_err(|error| error.to_string())?;
            return Ok(("asserted".into(), game));
        },
        Input::Admissions(batch) => batch,
    };
    if batch.version != VERSION { return Err("unsupported VTT admission input version".into()); }
    let mut candidate = session.clone();
    submit_contract_assertions(&mut candidate, &batch.assertions).map_err(|error| error.to_string())?;
    if batch.characters.is_empty() {
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
    bindings.validate(&candidate)?;
    let next = bindings.encode()?;
    *session = candidate;
    Ok((format!("developed: {}", accepted.join(", ")), next))
}
