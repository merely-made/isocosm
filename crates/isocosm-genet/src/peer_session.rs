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

#[path = "peer_custody.rs"]
mod custody;

/// `--handoff <dir>` and `--persona <name>` from the command line.
fn arg(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

use graphshell_host::invitation::Contact;

struct HostOptions {
    player: Option<(Contact, u64)>,
    replica: Option<Contact>,
}

fn host_options(args: &[String]) -> Result<HostOptions, String> {
    let value = |flag: &str| -> Result<Option<&str>, String> {
        let Some(i) = args.iter().position(|s| s == flag) else { return Ok(None) };
        args.get(i + 1).filter(|s| !s.starts_with("--")).map(|s| Some(s.as_str()))
            .ok_or_else(|| format!("{flag} needs a value"))
    };
    let read_contact = |file: &str| crate::family_table::host_net::read_json(std::path::Path::new(file));
    let player = match (value("--player-contact")?, value("--player-participant")?) {
        (None, None) => None,
        (Some(file), Some(id)) => Some((read_contact(file)?, id.parse().map_err(|_| "invalid player participant id")?)),
        _ => return Err("choose both --player-contact <file> and --player-participant <id>".into()),
    };
    Ok(HostOptions { player, replica: value("--replica-contact")?.map(read_contact).transpose()? })
}

/// Begin hosting or joining, as the command line asked. The status says why not.
pub(crate) fn start(app: &mut App, ui: &mut UiState) {
    let Some(intent) = app.net_intent.take() else { return };
    if matches!(intent, NetIntent::Join(_)) {
        ui.invited_table = true;
        ui.net_mode = NetMode::Remote;
        ui.set_vtt_projections(Vec::new());
    }
    let Some(wake) = app.family_wake.as_ref().map(|wake| wake.callback()) else {
        ui.status = "Peer sessions need the app's wake handle.".into();
        return;
    };
    let before = app.family_table.as_ref().map(crate::family_table::FamilyTable::sequence);
    ui.status = match intent {
        NetIntent::Host => match (app.family_table.as_mut(), arg("--handoff")) {
            (None, _) => "Open a native campaign before inviting players.".into(),
            (_, None) => "Hosting needs --handoff <folder> for the card and invitations.".into(),
            (Some(table), Some(dir)) => match host_options(&std::env::args().collect::<Vec<_>>()).and_then(|HostOptions { player, replica }| {
                let identity = Arc::new(custody::identity(true, replica.is_some())?);
                if let Some((contact, participant)) = player {
                    let now = crate::family_table::invite::now_ms();
                    table.invite_player(&*identity, contact, Some(participant), now,
                        now + crate::family_table::host_net::INVITATION_LIFETIME_MS)?;
                }
                table.host_peers(identity, dir.clone().into(), replica, wake)
            }) {
                Ok(host) => {
                    app.family_host_net = Some(host);
                    format!("Hosting: the family card is in {dir}; guests' contact files there are invited")
                },
                Err(error) => format!("Hosting failed: {error}"),
            },
        },
        NetIntent::Join(dir) => match custody::identity(false, false).and_then(|identity|
            FamilyGuest::start(Arc::new(identity), dir.clone().into(),
                std::env::args().any(|s| s == "--replicate"), wake)) {
            Ok((guest, contact)) => {
                app.family_guest = Some(guest);
                format!("Joining: your contact is {}; waiting for the owner's invitation", contact.display())
            },
            Err(error) => format!("Joining failed: {error}"),
        },
    };
    // Binding a prepared seat can advance the world during startup, before
    // the first event-loop turn records its starting sequence.
    if let Some(table) = app.family_table.as_mut().filter(|t| Some(t.sequence()) != before) {
        match table.projections() {
            Ok(views) => ui.set_vtt_projections(views),
            Err(error) => ui.status.push_str(&format!(". Table refresh failed: {error}")),
        }
    }
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
        let Some(guest) = self.family_guest.as_mut() else { return ctx.runner.state().invited_table };
        if ctx.runner.state().vtt_action_request.is_some() {
            ctx.runner.update(|ui| {
                if let Some(action) = ui.take_vtt_action_request() { guest.invoke(action); }
            });
        }
        let turn = guest.tick();
        if let Some(status) = &turn.status { eprintln!("[isocosm-peer] {status}"); }
        if turn.projections.is_some() || turn.status.is_some() {
            ctx.runner.update(|ui| {
                if let Some(views) = turn.projections { ui.set_vtt_projections(views); }
                if let Some(status) = turn.status { ui.status = status; }
            });
        }
        true
    }
}
