//! Pathfinder 2e character cards: a creation draft shown one explained
//! decision at a time, then the finished sheet with its derived numbers.
use super::{Act, Built};
use crate::ruleset::{self, Action, State};
use chirograph::IntentEffect;
use isocosm_host::endpoint::disclosure::{Item, NULL, action, card, value};
use isometry_core::TokenId;
use isometry_system::pf2e::{
    self,
    derive::derive,
    draft::{Progress, walk},
    records,
};

const RULESET: &str = "Pathfinder 2e Player Core (Remaster); experimental";

fn push(built: &mut Built, id: u64, title: &str, values: Vec<chirograph::CardValueV1>,
    choices: Vec<(String, String, String, Action)>) {
    let actions = choices.iter().map(|(intent, label, why, _)|
        action(intent, label, why, NULL, IntentEffect::DomainTruth)).collect();
    built.push(Item { id: format!("sheet:{id}"), card: card(title, values, &["experimental"]), actions },
        choices.into_iter().map(|(_, _, _, act)| Act::Ruleset(act)).collect());
}

/// The card for one native body: start, draft or finished sheet.
pub(crate) fn body(built: &mut Built, state: &State, map: &isometry_core::MapDocument, id: u64,
    token: TokenId, title: &str, control: bool) -> Result<(), String> {
    let r = records::builtin();
    let mut values = vec![value("Native body", id), value("Ruleset", RULESET)];
    let mut choices = Vec::new();
    let Some(draft) = state.drafts.get(&token) else {
        values.push(value("Character", "not started"));
        if control {
            choices.push(("vtt.sheet-create".into(), "Start Pathfinder 2e character".into(),
                "Begin a saved draft; each choice is explained and can be taken back.".into(), Action::Create(id)));
        }
        push(built, id, title, values, choices);
        return Ok(());
    };
    let walked = walk(r, draft)?;
    if let Some(sheet) = map.sheet(token) {
        let derived = derive(r, &walked.build);
        values.push(value("Hit Points", format!("{} / {}", sheet.int("hp_current").unwrap_or(0), derived.hp_max)));
        values.push(value("Level", derived.level));
        values.extend(derived.lines.iter().filter(|l| l.label != "Hit Points").map(|l| value(&l.label, &l.value)));
        push(built, id, title, values, choices);
        return Ok(());
    }
    values.push(value("Choices made", walked.made.len()));
    for made in &walked.made {
        let shown = if made.summary.is_empty() { made.label.clone() } else { format!("{}. {}", made.label, made.summary) };
        values.push(value(&made.title, shown));
    }
    match &walked.progress {
        Progress::Pending(slot) => {
            values.push(value("Next decision", &slot.title));
            values.push(value("What it decides", r.step(slot.kind)));
            for o in &slot.options {
                if let Some(why) = &o.refused {
                    values.push(value(format!("Not available: {}", o.label), why));
                }
            }
            if control {
                for o in slot.options.iter().filter(|o| o.refused.is_none()) {
                    let why = if o.summary.is_empty() { r.step(slot.kind).to_owned() } else { o.summary.clone() };
                    choices.push((format!("vtt.draft-{}", o.value), o.label.clone(), why,
                        Action::Choose { actor: id, slot: slot.id.clone(), value: o.value.clone() }));
                }
            }
        },
        Progress::Complete => {
            values.push(value("Next decision", "none: ready to finish"));
            values.push(value("What it decides", r.step("finish")));
            values.extend(derive(r, &walked.build).lines.iter().map(|l| value(&l.label, &l.value)));
            if control {
                choices.push(("vtt.draft-finish".into(), "Finish character".into(),
                    r.step("finish").into(), Action::Finish(id)));
            }
        },
    }
    if control {
        if !walked.made.is_empty() {
            choices.push(("vtt.draft-undo".into(), "Take back last choice".into(),
                "Undo the most recent choice; later steps follow from what remains.".into(), Action::Undo(id)));
        }
        choices.push(("vtt.draft-discard".into(), "Discard draft".into(),
            "Abandon this unfinished character.".into(), Action::Discard(id)));
    }
    push(built, id, title, values, choices);
    Ok(())
}

/// Whether this table plays Pathfinder 2e.
pub(crate) fn plays(sections: &std::collections::BTreeMap<String, Vec<u8>>) -> Result<bool, String> {
    Ok(ruleset::edition(sections)? == pf2e::SYSTEM)
}
