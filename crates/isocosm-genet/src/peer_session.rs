//! Starting and pumping a table shared with invited peers (isocosm case 1060).
//!
//! `--host --handoff DIR` hosts the opened campaign: the owner's family card
//! goes into DIR, and each guest contact file found there is invited.
//! `--join DIR` joins as a guest from DIR. `--persona NAME` speaks as that
//! custody persona; without it, the persona in use. Identities come only from
//! the device's custodian (Djinn); no key is made here.

use super::*;
use crate::family_guest::FamilyGuest;
use std::sync::Arc;

const APP: &str = "isocosm";

/// `--handoff <dir>` and `--persona <name>` from the command line.
fn arg(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

fn identity() -> Result<castellan_client::custody_identity::CustodyIdentity, String> {
    let persona = arg("--persona").map(personae::vault::ProfileId);
    castellan_client::custody_identity::CustodyIdentity::connect_as(APP, persona)
        .map_err(|error| format!("the custody identity is unavailable: {error}"))
}

/// Begin hosting or joining, as the command line asked. The status says why not.
pub(crate) fn start(app: &mut App, ui: &mut UiState) {
    let Some(intent) = app.net_intent.take() else { return };
    let Some(wake) = app.family_wake.as_ref().map(|wake| wake.callback()) else {
        ui.status = "Peer sessions need the app's wake handle.".into();
        return;
    };
    ui.status = match intent {
        NetIntent::Host => match (app.family_table.as_mut(), arg("--handoff")) {
            (None, _) => "Open a native campaign before inviting players.".into(),
            (_, None) => "Hosting needs --handoff <folder> for the card and invitations.".into(),
            (Some(table), Some(dir)) => match identity().and_then(|identity|
                table.host_peers(Arc::new(identity), dir.clone().into(), wake)) {
                Ok(host) => {
                    app.family_host_net = Some(host);
                    format!("Hosting: the family card is in {dir}; guests' contact files there are invited")
                },
                Err(error) => format!("Hosting failed: {error}"),
            },
        },
        NetIntent::Join(dir) => match identity().and_then(|identity|
            FamilyGuest::start(Arc::new(identity), dir.clone().into(), wake)) {
            Ok((guest, contact)) => {
                ui.net_mode = NetMode::Remote;
                app.family_guest = Some(guest);
                format!("Joining: your contact is {}; waiting for the owner's invitation", contact.display())
            },
            Err(error) => format!("Joining failed: {error}"),
        },
    };
    // The VTT page has no status line; the log keeps what happened.
    eprintln!("[isocosm-peer] {}", ui.status);
}

impl App {
    /// The owner's hosting turn, ahead of the table's own request pumps.
    pub(crate) fn pump_peer_host(&mut self, ctx: &mut Ctx<'_>) {
        let (Some(host), Some(table), Some(wake)) = (self.family_host_net.as_mut(), self.family_table.as_mut(),
            self.family_wake.as_ref()) else { return };
        let (pumps, notes) = host.tick(table, wake.callback());
        self.family_request_pumps.extend(pumps);
        for note in &notes { eprintln!("[isocosm-peer] {note}"); }
        if let Some(note) = notes.last().cloned() {
            ctx.runner.update(|ui| ui.status = note);
        }
    }

    /// The guest's turn. Returns whether this process is a guest, which owns
    /// the page in place of a local table.
    pub(crate) fn pump_peer_guest(&mut self, ctx: &mut Ctx<'_>) -> bool {
        let Some(guest) = self.family_guest.as_mut() else { return false };
        let mut action = None;
        ctx.runner.update(|ui| action = ui.take_vtt_action_request());
        if let Some(action) = action { guest.invoke(action); }
        let turn = guest.tick();
        if let Some(status) = &turn.status { eprintln!("[isocosm-peer] {status}"); }
        ctx.runner.update(|ui| {
            if let Some(views) = turn.projections { ui.set_vtt_projections(views); }
            if let Some(status) = turn.status { ui.status = status; }
        });
        true
    }
}
