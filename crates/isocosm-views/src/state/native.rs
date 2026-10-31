// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::TextInput;
use isocosm_native::founding::{Campaign, NativeWorld, Offered, Purpose, Setup};
use isocosm_campaign::GenerationRecord;

#[derive(Clone, Debug)]
pub struct Selection {
    pub setup: Setup,
    pub record: GenerationRecord,
    pub preview: NativeWorld,
    pub profile_path: String,
    pub attachment_path: String,
    pub sites: Vec<String>,
    pub edition: String,
}

#[derive(Clone, Debug, Default)]
pub struct NativeUi {
    pub scene: crate::NativeSceneUi,
    pub institutions: crate::InstitutionUi,
    pub packs: crate::PackUi,
    pub party: crate::native_party::PartyUi,
    pub world: Option<NativeWorld>,
    pub table: Option<isonetry::NativeTable>,
    pub time_requests: Vec<cosmograph_overlay::vtt::TimeIntent>,
    pub binding_request: Option<isonetry::NativeTableBindingInput>,
    pub admission_request: Option<isonetry::native_table::NativeTableAdmissionInput>,
    pub physical_request: Option<PhysicalRequest>,
    pub physical_quote: Option<(isocosm_native::native::physical_map::PhysicalMapInput,
        isocosm_native::native::physical_map::PhysicalMapPreview)>,
    pub admission_path: TextInput,
    pub binding_path: TextInput,
    pub physical_path: TextInput,
    pub items_path: TextInput,
    pub items_file_request: Option<ItemsFileAction>,
    pub items_request: Option<ItemsRequest>,
    pub items_quote: Option<(isocosm_native::native::items::ItemInput,isocosm_native::native::items::ItemPreview)>,
    pub storylet_physical_path: TextInput,
    pub storylet_physical_file_request: Option<crate::StoryletPhysicalFileAction>,
    pub storylet_physical_request: Option<crate::StoryletPhysicalRequest>,
    pub storylet_physical_quote: Option<(isocosm_native::native::storylets::ComposedStoryletInput,
        isonetry::NativeComposedStoryletResolved)>,
    pub export_path: TextInput,
    pub file_request: Option<TableFileAction>,
    pub endpoints: Vec<[u8; 32]>,
    pub enabled: bool,
    pub profile_path: TextInput,
    pub attachment_path: TextInput,
    /// The edition the campaign will be founded under (case 1029); empty
    /// until chosen, then a key in `isocosm_native::founding::EDITIONS`.
    pub edition: String,
    pub preview_request: bool,
    pub selection: Option<Selection>,
    pub events:crate::NativeEventUi,
    /// Quick-start worlds the host can found in one step.
    pub quickstarts: Vec<QuickstartChoice>,
    /// The index of the quick-start world asked for, once.
    pub quickstart_request: Option<usize>,
}

/// A quick-start world on offer: its pack's datasheet text and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickstartChoice {
    pub name: String,
    /// "carried" for the sim's own, else the content pack's id.
    pub source: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug)]
pub enum TableFileAction { Admit, Bind, Export, QuotePhysical, ApplyPhysical }

#[derive(Clone, Debug)]
pub enum PhysicalRequest {
    Quote(isocosm_native::native::physical_map::PhysicalMapInput),
    Apply(isocosm_native::native::physical_map::PhysicalMapInput),
}

#[derive(Clone, Debug)]
pub enum PhysicalOutcome {
    Quote { input: isocosm_native::native::physical_map::PhysicalMapInput,
        preview: isocosm_native::native::physical_map::PhysicalMapPreview },
    Applied,
}

#[derive(Clone,Copy,Debug)]
pub enum ItemsFileAction { Quote,Apply }

#[derive(Clone,Debug)]
pub enum ItemsRequest { Quote(isocosm_native::native::items::ItemInput),Apply(isocosm_native::native::items::ItemInput) }

#[derive(Clone,Debug)]
pub enum ItemsOutcome {
    Quote {input:isocosm_native::native::items::ItemInput,preview:isocosm_native::native::items::ItemPreview},
    Applied,Submitted,
}

#[derive(Clone, Debug)]
pub enum Acceptance { TableOnly, Native(Selection) }

impl NativeUi {
    /// The chosen edition; a table that never chose founds the 5e default.
    pub fn chosen_edition(&self) -> String {
        if self.edition.is_empty() { isocosm_native::founding::default_edition() } else { self.edition.clone() }
    }
    pub fn acceptance(&self, record: &GenerationRecord) -> Result<Acceptance, String> {
        if self.world.is_some() { return Err("this campaign already owns a native world".into()); }
        if !self.enabled { return Ok(Acceptance::TableOnly); }
        Ok(Acceptance::Native(self.selected(record)?.clone()))
    }
    /// A refusal cannot replace an earlier validated preview.
    pub fn preview(&mut self, bytes: &[u8], attachments: Option<&[u8]>, record: &GenerationRecord)
        -> Result<(), String> {
        self.preview_for(bytes, attachments, record, Purpose::Head, None)
    }
    /// Found a preview only under the caller's explicit head or bench policy.
    pub fn preview_for(
        &mut self, bytes: &[u8], attachments: Option<&[u8]>, record: &GenerationRecord,
        purpose: Purpose, offered: Option<Offered<'_>>,
    ) -> Result<(), String> {
        let mut setup = Setup::decode(bytes)?;
        if let Some(bytes) = attachments { setup.replace_attachments(bytes)?; }
        let edition = self.chosen_edition();
        let prepared = Campaign::from_setup_for(&setup, record, 16, purpose, offered)
            .map_err(|e| e.to_string())?;
        let campaign = prepared.campaign.with_edition(&edition).map_err(|e| e.to_string())?;
        let preview = NativeWorld::from_campaign(&campaign)?;
        let sites = campaign.reading().authored_places().iter().map(|(key, (id, place))|
            format!("{}: site {id} ({key})", place.name)).collect();
        self.selection = Some(Selection { setup, record: record.clone(), preview, sites,
            profile_path: self.profile_path.text().trim().into(),
            attachment_path: self.attachment_path.text().trim().into(), edition });
        Ok(())
    }
    pub fn selected(&self, record: &GenerationRecord) -> Result<&Selection, String> {
        let selected = self.selection.as_ref().ok_or("preview a world profile before accepting it")?;
        if &selected.record != record { return Err("world preview belongs to another generated draft".into()); }
        if selected.profile_path != self.profile_path.text().trim()
            || selected.attachment_path != self.attachment_path.text().trim()
        { return Err("world profile selection changed since preview".into()); }
        if selected.edition != self.chosen_edition() {
            return Err("the campaign's edition changed since preview".into());
        }
        Ok(selected)
    }
}

impl super::UiState {
    pub fn request_native_items_file(&mut self,action:ItemsFileAction) {
        if self.native.world.is_none() || self.native.table.is_none() {
            self.status="item actuation needs an admitted native world and table".into();
            return;
        }
        self.native.items_file_request=Some(action);
    }
    /// Identity and native tick are filled by the actual session transport,
    /// never by token owner, UI player names or this request queue.
    pub fn request_native_time(&mut self, intent: cosmograph_overlay::vtt::TimeIntent) {
        if self.native.world.is_none() { self.status = "native time needs an accepted world".into(); return; }
        self.native.time_requests.push(intent);
        self.status = "requested native time decision".into();
    }

    pub fn request_native_table_file(&mut self, action: TableFileAction) {
        if !self.can_edit_inventory || self.native.world.is_none() {
            self.status = "native table authoring requires the host's existing world".into();
            return;
        }
        self.native.file_request = Some(action);
    }

    /// Explicit host-authored mappings; validation and publication belong to
    /// the existing authority callback, not to this request flag.
    pub fn request_native_table_binding(&mut self, input: isonetry::NativeTableBindingInput) {
        if !self.can_edit_inventory { self.status = "native table binding requires the host".into(); return; }
        self.native.binding_request = Some(input);
    }

    pub(super) fn request_native_move(&mut self, id: isocosm_core::TokenId, to: isocosm_core::TileCoord) -> bool {
        if self.native.world.is_none() { return false; }
        self.net_outbox.push(isonetry::GameEvent::Map(isocosm_core::SessionEvent::TokenMoved { id, to }));
        self.status = "requested native table move".into();
        true
    }

    pub fn choose_native_world(&mut self, enabled: bool) {
        if !self.can_edit_inventory { self.status = "world selection requires the host".into(); return; }
        if self.native.world.is_some() { self.status = "this campaign already owns a native world".into(); return; }
        self.native.enabled = enabled;
        self.native.preview_request = false;
        self.status = if enabled { "select a world profile and preview its site attachments" }
            else { "accepting table content with simulation off" }.into();
    }
    pub fn choose_edition(&mut self, edition: &str) {
        if !self.can_edit_inventory { self.status = "the edition is chosen by the host".into(); return; }
        if self.native.world.is_some() { self.status = "this campaign is already founded".into(); return; }
        if !isocosm_native::founding::EDITIONS.contains(&edition) { self.status = format!("unknown edition {edition}"); return; }
        self.native.edition = edition.to_owned();
        self.status = format!("the campaign will be founded under {}", isocosm_native::founding::edition_label(edition));
    }
    /// Ask the host to found the quick-start world at `index`.
    pub fn request_quickstart(&mut self, index: usize) {
        if !self.can_edit_inventory { self.status = "founding a world requires the host".into(); return; }
        if self.native.world.is_some() { self.status = "a native campaign is already founded".into(); return; }
        if index < self.native.quickstarts.len() { self.native.quickstart_request = Some(index); }
    }

    pub fn preview_native_world(&mut self) {
        if !self.can_edit_inventory { self.status = "world selection requires the host".into(); return; }
        if self.generator_preview.is_none() { self.status = "generate a campaign draft first".into(); return; }
        self.native.preview_request = true;
    }
}
