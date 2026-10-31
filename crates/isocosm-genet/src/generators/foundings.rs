// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl App {
    pub(crate) fn pump_world_preview(&mut self, ctx: &mut Ctx<'_>) {
        let (request, can_edit, record, profile, attachments) = {
            let ui = ctx.runner.state();
            (ui.native.preview_request, ui.can_edit_inventory, ui.generator_preview.clone(),
                ui.native.profile_path.text().trim().to_owned(),
                ui.native.attachment_path.text().trim().to_owned())
        };
        if !request { return; }
        ctx.runner.update(|ui| ui.native.preview_request = false);
        let result: Result<_, String> = (|| {
            if !can_edit { return Err("world selection requires the host".into()); }
            if profile.is_empty() { return Err("select a world profile file".into()); }
            let record = record.as_ref().ok_or("generate a campaign draft first")?;
            let bytes = std::fs::read(&profile).map_err(|e| format!("world profile: {e}"))?;
            let attachments = if attachments.is_empty() { None } else {
                Some(std::fs::read(&attachments).map_err(|e| format!("site attachments: {e}"))?)
            };
            let mut candidate = ctx.runner.state().native.clone();
            candidate.preview(&bytes, attachments.as_deref(), record)?;
            Ok(candidate.selection)
        })();
        ctx.runner.update(|ui| match result {
            Ok(selection) => {
                ui.native.selection = selection;
                ui.status = "world and native site attachments ready to accept".into();
            },
            Err(error) => ui.status = error,
        });
    }
}