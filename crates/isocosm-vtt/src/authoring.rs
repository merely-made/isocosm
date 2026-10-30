// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native operation authoring: the DM's context for writing an operation,
//! previews of a supplied operation of each kind, and its application
//! through the campaign's one admission path. The operation is the payload.

use chirograph::IntentEffect;
use isocosm_host::endpoint::disclosure::{Item, action, card, value};
use isocosm_overlay::Seat;
use crate::native::{GameBindings, graft, institutions, items, party, physical_map};
use serde::de::DeserializeOwned;

use super::{Act, Built, Cx, VttEndpoint};

const OPERATION: &str = r#"{"type":"object"}"#;
const ASSERTIONS: &str = r#"{"type":"array"}"#;
const KINDS: [&str; 6] = ["Preview institution operation", "Preview physical map edit", "Preview item or project",
    "Preview graft carrying", "Preview knowledge statement", "Preview Location grouping"];

fn input<T: DeserializeOwned>(payload: &[u8]) -> Result<T, String> {
    serde_json::from_slice(payload).map_err(|e| e.to_string())
}

fn pretty(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

/// What applying `payload`, an operation of `kind`, would do; nothing is submitted.
pub(crate) fn quote(endpoint: &VttEndpoint, kind: u8, payload: &[u8]) -> Result<String, String> {
    let access = endpoint.window.projection()?.access;
    if access.seat != Seat::Dm { return Err("Native authoring requires the DM seat.".into()); }
    endpoint.read(|session, game| match kind {
        0 => pretty(&institutions::prepare_institution(session, access, &input(payload)?)?),
        1 => pretty(&physical_map::prepare_physical_map(session, game, access, &input(payload)?)?),
        2 => pretty(&items::prepare_item(session, game, access, &input(payload)?)?),
        3 => pretty(&graft::prepare_graft(session, access, &input(payload)?)?),
        4 => pretty(&party::statement_admission(session, game, access, &input(payload)?)?),
        5 => {
            let mut candidate = session.clone();
            let (outcome, _) = party::submit_location(&mut candidate, game.to_vec(), access, &input(payload)?)?;
            Ok(outcome)
        },
        _ => Err("unknown operation kind".into()),
    })
}

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let mut built = Built::new();
    let access = cx.cut.access;
    let context = if access.seat != Seat::Dm { Err("Native authoring context requires the DM seat.".to_string()) }
        else { endpoint.read(|session, game| {
            let institutions = session.sim.state().polities.keys().map(|id|
                session.sim.institution_reading(isocosm::institutions::PolityId(*id))).collect::<Result<Vec<_>, _>>()?;
            Ok(serde_json::json!({
                "institution_source": institutions::source(session), "physical_source": physical_map::physical_source(session),
                "item_source": items::source(session), "graft_source": institutions::source(session),
                "game_source": isocosm::digest(&game), "terrain_revision": session.sim.terrain_revision(),
                "tick": session.sim.state().tick, "participant": access.participant.0, "played_body": access.subject,
                "institutions": institutions, "bindings": GameBindings::decode(game)?,
            }))
        }) };
    let context = match context {
        Ok(context) => context,
        Err(reason) => {
            built.push(Item { id: "authoring".into(), card: card("Native operation authoring",
                vec![value("Note", reason)], &["note"]), actions: Vec::new() }, Vec::new());
            return Ok(built);
        },
    };
    let values = context.as_object().into_iter().flatten().map(|(key, field)| match field {
        serde_json::Value::String(text) => value(key, text),
        other => value(key, other),
    }).collect();
    let mut actions: Vec<_> = KINDS.iter().enumerate().map(|(kind, label)| action(&format!("vtt.preview-{kind}"), label,
        "Quote what this supplied operation would do.", OPERATION, IntentEffect::Curation)).collect();
    let mut acts: Vec<_> = (0..KINDS.len() as u8).map(Act::Quote).collect();
    actions.push(action("vtt.apply", "Apply supplied operation", "Submit this operation to the shared world.", OPERATION,
        IntentEffect::DomainTruth));
    acts.push(Act::Submit(Vec::new()));
    actions.push(action("vtt.assert", "Assert table contract", "Submit these contract assertions to the shared world.",
        ASSERTIONS, IntentEffect::DomainTruth));
    acts.push(Act::Assert);
    built.push(Item { id: "authoring".into(), card: card("Native operation authoring", values, &["authoring"]), actions },
        acts);
    if let Some(quote) = cx.quote {
        built.push(Item { id: "quote".into(), card: card("Last preview", vec![value("Quote", quote)], &["quote"]),
            actions: Vec::new() }, Vec::new());
    }
    built.push(Item {id:"ruleset-scale".into(),card:card("Ruleset physical scale",vec![value("Scale","Native cells per five-foot square")],&["authoring"]),
        actions:vec![action("vtt.ruleset-scale","Set ruleset physical scale","Declare scale for measured weapon reach.",
            r#"{"type":"object","properties":{"cells_per_square":{"type":"integer","minimum":1}},"required":["cells_per_square"],"additionalProperties":false}"#,
            IntentEffect::DomainTruth),action("vtt.ruleset-rulings","Use DM reach rulings","Use your recorded reach decisions for this table.",isocosm_host::endpoint::disclosure::NULL,IntentEffect::DomainTruth)]},
            vec![Act::Ruleset(crate::ruleset::Action::Calibrate),Act::Ruleset(crate::ruleset::Action::UseRulings)]);
    Ok(built)
}
