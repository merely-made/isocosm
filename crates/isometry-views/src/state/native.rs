// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::TextInput;
use isometry_campaign::founding::{Campaign, NativeWorld, Setup};
use isometry_campaign::GenerationRecord;

#[derive(Clone, Debug)]
pub struct Selection {
    pub setup: Setup,
    pub record: GenerationRecord,
    pub preview: NativeWorld,
    pub profile_path: String,
    pub attachment_path: String,
    pub sites: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct NativeUi {
    pub world: Option<NativeWorld>,
    pub table: Option<isonetry::NativeTable>,
    pub binding_request: Option<isonetry::NativeTableBindingInput>,
    pub admission_request: Option<isonetry::native_table::NativeTableAdmissionInput>,
    pub admission_path: TextInput,
    pub binding_path: TextInput,
    pub export_path: TextInput,
    pub file_request: Option<TableFileAction>,
    pub endpoints: Vec<[u8; 32]>,
    pub enabled: bool,
    pub profile_path: TextInput,
    pub attachment_path: TextInput,
    pub preview_request: bool,
    pub selection: Option<Selection>,
}

#[derive(Clone, Copy, Debug)]
pub enum TableFileAction { Admit, Bind, Export }

#[derive(Clone, Debug)]
pub enum Acceptance { TableOnly, Native(Selection) }

impl NativeUi {
    pub fn acceptance(&self, record: &GenerationRecord) -> Result<Acceptance, String> {
        if self.world.is_some() { return Err("this campaign already owns a native world".into()); }
        if !self.enabled { return Ok(Acceptance::TableOnly); }
        Ok(Acceptance::Native(self.selected(record)?.clone()))
    }
    /// A refusal cannot replace an earlier validated preview.
    pub fn preview(&mut self, bytes: &[u8], attachments: Option<&[u8]>, record: &GenerationRecord)
        -> Result<(), String> {
        let mut setup = Setup::decode(bytes)?;
        if let Some(bytes) = attachments { setup.replace_attachments(bytes)?; }
        let prepared = Campaign::from_setup(&setup, record, 16).map_err(|e| e.to_string())?;
        let preview = NativeWorld::from_campaign(&prepared.campaign)?;
        let sites = prepared.campaign.reading().authored_places().iter().map(|(key, (id, place))|
            format!("{}: site {id} ({key})", place.name)).collect();
        self.selection = Some(Selection { setup, record: record.clone(), preview, sites,
            profile_path: self.profile_path.text().trim().into(),
            attachment_path: self.attachment_path.text().trim().into() });
        Ok(())
    }
    pub fn selected(&self, record: &GenerationRecord) -> Result<&Selection, String> {
        let selected = self.selection.as_ref().ok_or("preview a world profile before accepting it")?;
        if &selected.record != record { return Err("world preview belongs to another generated draft".into()); }
        if selected.profile_path != self.profile_path.text().trim()
            || selected.attachment_path != self.attachment_path.text().trim()
        { return Err("world profile selection changed since preview".into()); }
        Ok(selected)
    }
}

impl super::UiState {
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

    pub(super) fn request_native_move(&mut self, id: isometry_core::TokenId, to: isometry_core::TileCoord) -> bool {
        if self.native.world.is_none() { return false; }
        self.net_outbox.push(isonetry::GameEvent::Map(isometry_core::SessionEvent::TokenMoved { id, to }));
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
    pub fn preview_native_world(&mut self) {
        if !self.can_edit_inventory { self.status = "world selection requires the host".into(); return; }
        if self.generator_preview.is_none() { self.status = "generate a campaign draft first".into(); return; }
        self.native.preview_request = true;
    }
}
