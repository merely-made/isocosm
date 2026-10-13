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
const TIME_VERSION: u32 = 3;
const HOLD_VERSION: u32 = 4;
const SHARED_VERSION: u32 = 5;
const JOURNEY_VERSION: u32 = 6;
const OVERMAP_VERSION: u32 = 7;
const HOOK_VERSION: u32 = 8;
const CATALOGUE_VERSION: u32 = 9;
const PHYSICAL_VERSION: u32 = 10;
const ITEM_VERSION: u32 = 11;
const CUSTODY_VERSION: u32 = 12;
const PROJECT_CARRY_VERSION: u32 = 13;
const COMPOSED_STORYLET_VERSION: u32 = 14;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_carry: Option<super::items::ProjectCarryConfig>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Input { Assertions(Vec<Assertion>), Admissions(AdmissionBatch), Play(VttIntentEnvelope),
    SharedRoster(super::SharedRosterInput), Journey(super::joint_journey::JourneyEnvelope),
    Overmap(super::overmap::OvermapInput), Pack(super::pack::PackEnvelope), Storylet(super::storylets::StoryletInput),
    Physical(super::physical_map::PhysicalMapInput), Institution(super::institutions::InstitutionInput),
    Item(super::items::ItemInput), Graft(super::graft::GraftInput),
    ProjectCarry(super::items::ProjectCarryConfig),
    ComposedStorylet(super::storylets::ComposedStoryletInput) }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    downtime: BTreeMap<Key, super::time::Proposal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    time_holds: Vec<super::time::Hold>,
    #[serde(default, skip_serializing_if = "is_zero")]
    roster_revision: u64,
    /// Absent is unconfigured, including historical games. Explicitly empty
    /// is an authored roster with no player holds, never a transient seat cut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shared_players: Option<BTreeSet<Id>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    journeys: BTreeMap<Key, super::joint_journey::Proposal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    overmap: Option<super::overmap::Presentation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    hooks: Vec<super::hooks::HookRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    catalogue: Option<super::pack::Catalogue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    physical_edits: Vec<super::physical_map::PhysicalMapRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    item_records: Vec<super::items::PhysicalItemRecord>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    project_carry: BTreeMap<Key, super::items::ProjectCarryPolicy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    composed_storylets: Vec<super::storylets::ComposedStoryletRecord>,
}

fn is_zero(value: &u64) -> bool { *value == 0 }

impl GameBindings {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if let Some(bytes) = bytes.strip_prefix(PREFIX) {
            let game: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
            if !matches!(game.version, VERSION | PLACEMENT_VERSION | TIME_VERSION | HOLD_VERSION | SHARED_VERSION | JOURNEY_VERSION
                | OVERMAP_VERSION | HOOK_VERSION | CATALOGUE_VERSION | PHYSICAL_VERSION | ITEM_VERSION | CUSTODY_VERSION | PROJECT_CARRY_VERSION | COMPOSED_STORYLET_VERSION)
                || game.version == VERSION && (!game.positions.is_empty() || !game.controls.is_empty())
                || game.version < TIME_VERSION && !game.downtime.is_empty()
                || game.version < HOLD_VERSION && (!game.time_holds.is_empty() || game.roster_revision != 0)
                || game.version < SHARED_VERSION && game.shared_players.is_some()
                || game.version < JOURNEY_VERSION && !game.journeys.is_empty()
                || game.version < OVERMAP_VERSION && game.overmap.is_some()
                || game.version < HOOK_VERSION && !game.hooks.is_empty()
                || game.version < CATALOGUE_VERSION && game.catalogue.is_some()
                || game.version < PHYSICAL_VERSION && !game.physical_edits.is_empty()
                || game.version < ITEM_VERSION && !game.item_records.is_empty()
                || game.version < CUSTODY_VERSION && game.item_records.iter().any(|record|
                    matches!(&record.input.action, super::items::ItemAction::Custody(_)))
                || game.version < COMPOSED_STORYLET_VERSION && !game.composed_storylets.is_empty()
                || game.version < PROJECT_CARRY_VERSION && (!game.project_carry.is_empty()
                    || game.item_records.iter().any(|record| matches!(&record.input.action,
                        super::items::ItemAction::ProjectCarry {..}) || record.project_custody.is_some()
                        || record.project_carrier_position.is_some()))
            { return Err("unsupported VTT character binding version".into()); }
            Ok(game)
        } else {
            // The previous product payload has no bindings owned by this
            // format. Preserve it without guessing its schema or migration.
            Ok(Self { version:VERSION, characters:BTreeMap::new(), positions:BTreeMap::new(),
                controls:BTreeMap::new(), payload:bytes.to_vec(), downtime:BTreeMap::new(),
                time_holds:Vec::new(), roster_revision:0, shared_players:None, journeys:BTreeMap::new(),
                overmap:None, hooks:Vec::new(), catalogue:None, physical_edits:Vec::new(), item_records:Vec::new(),
                project_carry:BTreeMap::new(), composed_storylets:Vec::new() })
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
    pub fn downtime(&self) -> &BTreeMap<Key, super::time::Proposal> { &self.downtime }
    pub fn time_holds(&self) -> &[super::time::Hold] { &self.time_holds }
    pub fn roster_revision(&self) -> u64 { self.roster_revision }
    pub fn shared_players(&self) -> Option<&BTreeSet<Id>> { self.shared_players.as_ref() }
    pub fn journeys(&self) -> &BTreeMap<Key, super::joint_journey::Proposal> { &self.journeys }
    pub fn overmap(&self) -> Option<&super::overmap::Presentation> { self.overmap.as_ref() }
    pub fn hooks(&self) -> &[super::hooks::HookRecord] { &self.hooks }
    pub fn catalogue(&self) -> Option<&super::pack::Catalogue> { self.catalogue.as_ref() }
    pub fn physical_edits(&self) -> &[super::physical_map::PhysicalMapRecord] { &self.physical_edits }
    pub fn item_records(&self) -> &[super::items::PhysicalItemRecord] { &self.item_records }
    pub fn project_carry(&self) -> &BTreeMap<Key, super::items::ProjectCarryPolicy> { &self.project_carry }
    pub fn composed_storylets(&self) -> &[super::storylets::ComposedStoryletRecord] { &self.composed_storylets }
    pub(super) fn record_composed_storylet(&mut self, record: super::storylets::ComposedStoryletRecord) {
        self.version = self.version.max(COMPOSED_STORYLET_VERSION);
        self.composed_storylets.push(record);
    }
    pub(super) fn record_project_carry(&mut self, policies:BTreeMap<Key,super::items::ProjectCarryPolicy>) {
        if !policies.is_empty() { self.version=self.version.max(PROJECT_CARRY_VERSION); }
        self.project_carry=policies;
    }
    pub(super) fn record_item(&mut self, record: super::items::PhysicalItemRecord) {
        let version = if matches!(&record.input.action, super::items::ItemAction::ProjectCarry {..})
            || record.project_carrier_position.is_some() { PROJECT_CARRY_VERSION }
        else if matches!(&record.input.action, super::items::ItemAction::Custody(_)) {
            CUSTODY_VERSION
        } else { ITEM_VERSION };
        self.version = self.version.max(version);
        self.item_records.push(record);
    }
    pub(super) fn record_physical_edit(&mut self, record: super::physical_map::PhysicalMapRecord) {
        self.version = self.version.max(PHYSICAL_VERSION);
        self.physical_edits.push(record);
    }
    pub(super) fn record_overmap(&mut self, presentation: super::overmap::Presentation) {
        self.version = self.version.max(OVERMAP_VERSION);
        self.overmap = Some(presentation);
    }
    pub(super) fn record_hook(&mut self, record: super::hooks::HookRecord) {
        self.version = self.version.max(HOOK_VERSION);
        self.hooks.push(record);
    }
    pub(super) fn record_catalogue(&mut self, catalogue: super::pack::Catalogue) {
        self.version = self.version.max(CATALOGUE_VERSION);
        self.catalogue = Some(catalogue);
    }
    pub(super) fn same_except_hooks(&self, other: &Self) -> bool {
        self.composed_storylets == other.composed_storylets && self.project_carry == other.project_carry && self.characters == other.characters && self.positions == other.positions && self.controls == other.controls
            && self.payload == other.payload && self.downtime == other.downtime && self.time_holds == other.time_holds
            && self.roster_revision == other.roster_revision && self.shared_players == other.shared_players
            && self.journeys == other.journeys && self.overmap == other.overmap && self.catalogue == other.catalogue && self.physical_edits == other.physical_edits && self.item_records == other.item_records
    }
    pub(super) fn same_except_catalogue(&self, other: &Self) -> bool {
        self.composed_storylets == other.composed_storylets && self.project_carry == other.project_carry && self.characters == other.characters && self.positions == other.positions && self.controls == other.controls
            && self.payload == other.payload && self.downtime == other.downtime && self.time_holds == other.time_holds
            && self.roster_revision == other.roster_revision && self.shared_players == other.shared_players
            && self.journeys == other.journeys && self.overmap == other.overmap && self.hooks == other.hooks
            && self.physical_edits == other.physical_edits && self.item_records == other.item_records
    }
    pub(super) fn same_except_physical_edits(&self, other: &Self) -> bool {
        self.composed_storylets == other.composed_storylets && self.project_carry == other.project_carry && self.characters == other.characters && self.positions == other.positions && self.controls == other.controls
            && self.payload == other.payload && self.downtime == other.downtime && self.time_holds == other.time_holds
            && self.roster_revision == other.roster_revision && self.shared_players == other.shared_players
            && self.journeys == other.journeys && self.overmap == other.overmap && self.hooks == other.hooks
            && self.catalogue == other.catalogue && self.item_records == other.item_records
    }
    pub(super) fn same_except_items(&self, other: &Self) -> bool {
        self.composed_storylets == other.composed_storylets && self.project_carry == other.project_carry && self.characters == other.characters && self.positions == other.positions && self.controls == other.controls
            && self.payload == other.payload && self.downtime == other.downtime && self.time_holds == other.time_holds
            && self.roster_revision == other.roster_revision && self.shared_players == other.shared_players
            && self.journeys == other.journeys && self.overmap == other.overmap && self.hooks == other.hooks
            && self.catalogue == other.catalogue && self.physical_edits == other.physical_edits
    }
    pub(super) fn record_journey(&mut self, key: Key, proposal: super::joint_journey::Proposal) {
        self.version = self.version.max(JOURNEY_VERSION);
        self.journeys.insert(key,proposal);
    }
    pub(super) fn same_except_journeys(&self, other: &Self) -> bool {
        self.composed_storylets == other.composed_storylets && self.project_carry == other.project_carry && self.characters == other.characters && self.positions == other.positions && self.controls == other.controls
            && self.payload == other.payload && self.downtime == other.downtime && self.time_holds == other.time_holds
            && self.roster_revision == other.roster_revision && self.shared_players == other.shared_players
            && self.overmap == other.overmap && self.hooks == other.hooks && self.catalogue == other.catalogue && self.physical_edits == other.physical_edits && self.item_records == other.item_records
    }

    pub fn shared_roster(&self) -> Option<super::time::Roster> {
        self.shared_players.as_ref().map(|players| super::time::Roster {
            players:players.clone(), preimage:isocosm::digest(players), revision:self.roster_revision })
    }

    pub(super) fn record_shared_players(&mut self, players: BTreeSet<Id>) -> Result<(), String> {
        if self.shared_players.as_ref().is_some_and(|prior| prior != &players) {
            self.roster_revision = self.roster_revision.checked_add(1).ok_or("native time roster revision overflow")?;
        }
        self.version = self.version.max(SHARED_VERSION);
        self.shared_players = Some(players);
        Ok(())
    }

    pub fn record_roster_revision(&mut self, revision: u64) {
        if revision != 0 { self.version = self.version.max(HOLD_VERSION); }
        self.roster_revision = revision;
    }

    pub(super) fn record_hold(&mut self, hold: super::time::Hold) {
        self.version = self.version.max(HOLD_VERSION);
        self.time_holds.push(hold);
    }

    pub(super) fn record_downtime(&mut self, key: Key, proposal: super::time::Proposal) {
        self.version = self.version.max(HOLD_VERSION);
        self.downtime.insert(key, proposal);
    }

    pub(super) fn same_except_time(&self, other: &Self) -> bool {
        self.composed_storylets == other.composed_storylets && self.project_carry == other.project_carry && self.characters == other.characters && self.positions == other.positions
            && self.controls == other.controls && self.payload == other.payload
            && self.roster_revision == other.roster_revision
            && self.shared_players == other.shared_players
            && self.journeys == other.journeys
            && self.overmap == other.overmap && self.hooks == other.hooks && self.catalogue == other.catalogue && self.physical_edits == other.physical_edits && self.item_records == other.item_records
    }

    pub(super) fn record_position(&mut self, character: Key, position: PositionBinding) {
        self.version = self.version.max(PLACEMENT_VERSION);
        self.positions.insert(character, position);
    }

    pub fn validate(&self, session: &Session) -> Result<(), String> {
        super::storylets::validate_composed_records(session, &self.composed_storylets,
            &self.physical_edits, &self.item_records)?;
        if self.shared_players.as_ref().is_some_and(|players| players.iter().any(|id|
            !session.sim.state().population.get(*id).is_some_and(isocosm::directing::is_participant))) {
            return Err("saved shared table roster names no actual native participant".into());
        }
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
        super::time::validate_bindings(session, &self.downtime, &self.time_holds)?;
        super::joint_journey::validate_bindings(session, &self.journeys)?;
        if let Some(presentation) = &self.overmap { super::overmap::validate_presentation(session, presentation)?; }
        super::hooks::validate_bindings(session, &self.hooks)?;
        super::pack::validate_bindings(session, self.catalogue.as_ref())?;
        super::physical_map::validate_records(session, &self.physical_edits)?;
        super::items::validate_policies(session, self)?;
        super::items::validate_item_records(session, &self.item_records)?;
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
    submit_game_access_with_roster(session, game, bytes, access, None)
}

/// A time input requires an explicit host-authenticated roster. Other table
/// inputs retain their existing access and byte-preservation behavior.
pub fn submit_game_access_with_roster(session: &mut Session, game: Vec<u8>, bytes: &[u8],
    access: OrdinaryAccess, roster: Option<&super::time::Roster>) -> Result<(String, Vec<u8>), String> {
    submit_access(session, game, bytes, access, roster, roster.is_none())
}

/// Standalone table peers are the external authority. These inputs cannot
/// author or fall back to the shared host's local roster metadata.
pub fn submit_external_table_access(session: &mut Session, game: Vec<u8>, bytes: &[u8], access: OrdinaryAccess)
    -> Result<(String, Vec<u8>), String> {
    submit_access(session, game, bytes, access, None, false)
}

fn submit_access(session: &mut Session, game: Vec<u8>, bytes: &[u8], access: OrdinaryAccess,
    roster: Option<&super::time::Roster>, shared: bool) -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    let input: Input = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    match input {
        Input::ProjectCarry(input) => Ok(("configured native project carry providers".into(),
            super::items::submit_project_carry_config(session, game, access, &input)?)),
        Input::Graft(input) => super::graft::submit_graft(session, game, access, &input),
        Input::Item(input) => super::items::submit_item(session, game, access, &input),
        Input::Institution(input) => super::institutions::submit_institution(session, game, access, &input),
        Input::Physical(input) => super::physical_map::submit_physical_map(session, game, access, &input),
        Input::Overmap(input) => super::overmap::submit_presentation(session, game, access, &input),
        Input::Pack(envelope) => super::pack::submit_pack(session, game, access, &BTreeSet::new(), &envelope),
        Input::Storylet(input) => super::storylets::submit_storylet(session, game, access, &input),
        Input::ComposedStorylet(input) => super::storylets::submit_composed_storylet(session, game, access, &input),
        Input::Journey(envelope) => {
            let local;
            let roster = if let Some(roster) = roster { roster }
                else if shared {
                    local = GameBindings::decode(&game)?.shared_roster()
                        .ok_or("DM must explicitly configure the saved native-player roster before joint travel")?;
                    &local
                } else { return Err("standalone joint travel requires its actual saved endpoint roster".into()); };
            super::joint_journey::submit_journey(session,game,access,roster,&envelope)
        },
        Input::Play(envelope) if matches!(&envelope.intent,
            isocosm_overlay::Intent::Game(isocosm_overlay::vtt::VttIntent::Time(_))) => {
                let local;
                let roster = if let Some(roster) = roster { roster }
                    else if shared {
                        local = GameBindings::decode(&game)?.shared_roster()
                            .ok_or("DM must explicitly configure the saved shared table player roster")?;
                        &local
                    } else { return Err("standalone native time requires its actual external table roster".into()); };
                super::submit_time(session, game, access, roster, &envelope)
            },
        Input::Play(envelope) if matches!(&envelope.intent,
            isocosm_overlay::Intent::Game(isocosm_overlay::vtt::VttIntent::Party(_))) =>
            super::party::submit_party(session, game, access, &envelope),
        Input::Play(envelope) if matches!(&envelope.intent,
            isocosm_overlay::Intent::Game(isocosm_overlay::vtt::VttIntent::Hook(_))) =>
            super::hooks::submit_hook(session, game, access, &envelope),
        Input::Play(envelope) => super::submit_table(session, game, access, &envelope),
        Input::SharedRoster(input) if shared => super::shared_roster::submit(session, game, access, &input),
        Input::SharedRoster(_) => Err("standalone native table players are authored through its endpoint bindings".into()),
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
        Input::SharedRoster(_) => return Err("shared roster authoring requires the actual DM seat cut".into()),
        Input::Journey(_) => return Err("joint journey requires actual native seat and player-roster admission".into()),
        Input::Overmap(_) => return Err("overmap presentation authoring requires the actual DM seat cut".into()),
        Input::Pack(_) => return Err("pack activation requires the actual DM seat cut".into()),
        Input::Storylet(_) => return Err("storylet activation requires the actual DM seat cut".into()),
        Input::ComposedStorylet(_) => return Err("storylet composition requires the actual DM seat cut".into()),
        Input::Physical(_) => return Err("physical map edits require the actual native seat and played body".into()),
        Input::Institution(_) => return Err("institution operations require the actual native seat and played body".into()),
        Input::Item(_) => return Err("physical item operations require the actual native seat and played body".into()),
        Input::Graft(_) => return Err("graft carrying requires the actual native seat and played body".into()),
        Input::ProjectCarry(_) => return Err("project carry configuration requires the actual DM seat cut".into()),
    };
    if !matches!(batch.version, VERSION | PLACEMENT_VERSION | TIME_VERSION)
        || batch.version == VERSION && (!batch.placements.is_empty() || !batch.controls.is_empty())
        || batch.version < TIME_VERSION && batch.project_carry.is_some()
    { return Err("unsupported VTT admission input version".into()); }
    if (!batch.controls.is_empty() || batch.project_carry.is_some()) && dm.is_none() {
        return Err("native character control needs the admitted DM participant".into());
    }
    let mut candidate = session.clone();
    let configured=if let Some(config)=&batch.project_carry {
        let mut bindings=GameBindings::decode(&game)?;
        if config.expected_game!=isocosm::digest(&game) {return Err("project carry admission has a stale game cut".into());}
        bindings.validate(session)?;
        super::items::configure_project_carry(session,&mut bindings,
            dm.ok_or("project carry configuration has no actual DM participant")?,config)?;
        Some(bindings)
    } else {None};
    submit_contract_assertions(&mut candidate, &batch.assertions).map_err(|error| error.to_string())?;
    if batch.characters.is_empty() && batch.placements.is_empty() && batch.controls.is_empty() && batch.project_carry.is_none() {
        *session = candidate;
        return Ok(("asserted".into(), game));
    }
    let mut bindings=match configured {Some(bindings)=>bindings,None=>GameBindings::decode(&game)?};
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
    if !batch.placements.is_empty() || !batch.controls.is_empty() { bindings.version = bindings.version.max(PLACEMENT_VERSION); }
    bindings.validate(&candidate)?;
    let next = bindings.encode()?;
    *session = candidate;
    if batch.project_carry.is_some() {accepted.push("configured native project carry providers".into());}
    let label = if batch.placements.is_empty() && batch.controls.is_empty() && batch.project_carry.is_none() { "developed" } else { "accepted" };
    Ok((format!("{label}: {}", accepted.join(", ")), next))
}
