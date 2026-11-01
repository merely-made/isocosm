// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The table's seat cards (case 1059): who holds the GM grant, open claims
//! and the yes each needs, and this window's claim, release or yes.

use chirograph::IntentEffect;
use cosmograph_host::endpoint::disclosure::{Item, NULL, action, card, value};
use cosmograph_overlay::Seat;
use crate::native::{GameBindings, SeatAction};

use super::{Act, Built, Cx};

const DOMAIN: IntentEffect = IntentEffect::DomainTruth;

fn ids<'a>(ids: impl IntoIterator<Item = &'a u64>) -> String {
    let list: Vec<String> = ids.into_iter().map(u64::to_string).collect();
    if list.is_empty() { "none".into() } else { list.join(", ") }
}

pub(crate) fn build(built: &mut Built, cx: &Cx, game: &GameBindings) {
    let access = cx.cut.access;
    let me = access.participant.0;
    let Some(seats) = game.seats() else {
        let (actions, acts) = if access.seat == Seat::Dm {
            (vec![action("vtt.seats-configure", "Keep GM seats as grants",
                "Record this participant as the table's first GM grant.", NULL, DOMAIN)],
                vec![Act::Seat(SeatAction::Configure)])
        } else { (Vec::new(), Vec::new()) };
        built.push(Item { id: "seats".into(), card: card("Seats", vec![value("GM grants", "not kept yet")],
            &["seats"]), actions }, acts);
        return;
    };
    let mut actions = Vec::new();
    let mut acts = Vec::new();
    if seats.holds_gm(me) {
        if access.seat == Seat::Dm {
            actions.push(action("vtt.seat-release", "Release my GM seat", "Give up this participant's GM grant.",
                NULL, DOMAIN));
            acts.push(Act::Seat(SeatAction::Release));
        }
    } else if seats.claims.contains_key(&me) {
        actions.push(action("vtt.seat-withdraw", "Withdraw my GM claim", "Withdraw the open claim.", NULL, DOMAIN));
        acts.push(Act::Seat(SeatAction::Withdraw));
    } else if access.seat != Seat::CreativeObserver {
        actions.push(action("vtt.seat-claim", "Claim the GM seat",
            "Ask the GMs seated, or the table when none is, for the GM grant.", NULL, DOMAIN));
        acts.push(Act::Seat(SeatAction::Claim));
    }
    built.push(Item { id: "seats".into(), card: card("Seats", vec![value("GM grants", ids(&seats.gms)),
        value("Bound players", ids(game.shared_players().into_iter().flatten())),
        value("Revision", seats.revision)], &["seats"]), actions }, acts);
    for (claimant, claim) in &seats.claims {
        let current = seats.current(claim, game.roster_revision());
        let wanted = if claim.consenters == seats.gms { Seat::Dm } else { Seat::Player };
        let (actions, acts) = if current && access.seat == wanted && claim.consenters.contains(&me)
            && !claim.yeses.contains(&me) {
            (vec![action("vtt.seat-consent", "Give my yes", "Consent to this GM claim.", NULL, DOMAIN)],
                vec![Act::Seat(SeatAction::Consent { claimant: *claimant })])
        } else { (Vec::new(), Vec::new()) };
        built.push(Item { id: format!("claim:{claimant}"), card: card(format!("GM claim by {claimant}"),
            vec![value("Needs", ids(&claim.consenters)), value("Yes from", ids(&claim.yeses)),
                value("Status", if current { "open" } else { "stale: claim again" })], &["claim"]),
            actions }, acts);
    }
}
