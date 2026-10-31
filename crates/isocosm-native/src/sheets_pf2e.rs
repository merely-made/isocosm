//! Pathfinder 2e character cards: a creation draft shown one explained
//! decision at a time, then the finished sheet with its derived numbers.
use super::{Act, Built};
use crate::ruleset::{self, Action, State};
use chirograph::IntentEffect;
use cosmograph_host::endpoint::disclosure::{Item, NULL, action, card, value};
use isocosm_core::TokenId;
use crate::ruleset::pathfinder::play;
use isocosm_system::pf2e::{
    self,
    derive::derive,
    draft::{Progress, money, walk},
    play as rules,
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

/// What the viewing seat may do with one body.
pub(crate) struct Seat {
    pub control: bool,
    pub gm: bool,
    /// The native body lives.
    pub alive: bool,
    /// Reach is measured on native positions rather than ruled.
    pub measured: bool,
    /// Bodies with living Pathfinder sheets this one could Strike.
    pub targets: Vec<u64>,
}

/// The card for one native body: start, draft or finished sheet.
pub(crate) fn body(built: &mut Built, state: &State, map: &isocosm_core::MapDocument, id: u64,
    token: TokenId, title: &str, seat: &Seat) -> Result<(), String> {
    let control = seat.control;
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
        let flag = |k: &str| sheet.int(k).unwrap_or(0);
        let status = if !seat.alive { "dead (the native body has died)".to_owned() }
            else if flag("dead") > 0 { "dead".to_owned() }
            else if flag("dying") > 0 { format!("dying {}, unconscious", flag("dying")) }
            else if flag("unconscious") > 0 { "unconscious".to_owned() }
            else { "able to act".to_owned() };
        values.push(value("Condition", status));
        values.push(value("Hit Points", format!("{} / {}", flag("hp_current"), derived.hp_max)));
        values.push(value("Wounded", flag("wounded")));
        let gear = play::gear(state, token);
        let raised = state.encounter.as_ref().and_then(|e| e.turn(token.0)).is_some_and(|t| t.shield_raised);
        values.push(value("Armor Class", rules::armor_class(&walked.build, &gear, raised)));
        let weapon = gear.weapon.cloned().unwrap_or_else(rules::fist);
        values.push(value("Strike", format!("{} {:+}, {} {}", weapon.name, rules::attack(&walked.build, &weapon),
            weapon.damage.as_deref().unwrap_or("1"), weapon.damage_type.as_deref().unwrap_or(""))));
        values.push(value("Coins", money(flag("coins_cp"))));
        if let Some(turn) = state.encounter.as_ref().and_then(|e| e.turn(token.0)) {
            values.push(value("This turn", format!("{} actions left, {} attacks made, reaction {}",
                turn.actions, turn.attacks, if turn.reaction { "ready" } else { "spent" })));
        }
        values.push(value("Level", derived.level));
        values.extend(derived.lines.iter().filter(|l| l.label != "Hit Points" && l.label != "Coins")
            .map(|l| value(&l.label, &l.value)));
        if control && seat.alive && flag("dead") == 0 {
            actions(&mut choices, state, id, token, &walked.build, seat);
        }
        if seat.gm && seat.alive && flag("dead") == 0 {
            for amount in [1, 5, 10] {
                choices.push((format!("vtt.sheet-damage-{amount}"), format!("Rule {amount} damage"),
                    "A GM ruling: damage from outside a Strike, with the dying track.".into(),
                    Action::Damage { target: id, amount }));
            }
        }
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

type Choices = Vec<(String, String, String, Action)>;

/// The controlling seat's actions for a finished character.
fn actions(choices: &mut Choices, state: &State, id: u64, token: TokenId, build: &pf2e::draft::Build, seat: &Seat) {
    let r = records::builtin();
    if let Some(inventory) = state.inventories.get(&token) {
        for (item_id, instance) in &inventory.items {
            if inventory.equipped.values().any(|e| e == item_id) {
                continue;
            }
            let Some(item) = play::item_of(state, token, item_id)
                .filter(|i| matches!(i.kind.as_str(), "weapon" | "armor" | "shield")) else { continue };
            choices.push((format!("vtt.pf2e-equip-{}", item_id.0), format!("Ready {}", instance.name),
                item.summary.clone(), Action::Equip { actor: id, item: item_id.clone() }));
        }
    }
    let encounter = state.encounter.as_ref();
    let my_turn = encounter.and_then(|e| e.current()).is_some_and(|t| t.token == token.0 && t.actions > 0);
    if my_turn {
        for target in &seat.targets {
            let (intent, label, act) = if seat.measured {
                (format!("vtt.pf2e-strike-{target}"), format!("Strike {target}"), Action::Attack { actor: id, target: *target })
            } else if seat.gm {
                (format!("vtt.pf2e-rule-strike-{target}"), format!("Rule within reach and Strike {target}"),
                    Action::RuleAttack { actor: id, target: *target })
            } else {
                continue;
            };
            choices.push((intent, label,
                "One action: an attack roll against Armor Class, with the multiple attack penalty after the first.".into(), act));
        }
        if play::gear(state, token).shield.is_some() {
            choices.push(("vtt.pf2e-raise-shield".into(), "Raise a Shield".into(),
                "One action: the shield's bonus to AC until your next turn starts.".into(), Action::RaiseShield(id)));
        }
    }
    if encounter.is_none() || my_turn {
        for (skill, _) in records::SKILLS.iter().filter(|(s, _)| build.trained(s)) {
            choices.push((format!("vtt.pf2e-check-{skill}"), format!("{} check", records::title(skill)),
                format!("{} against the table's DC; one action in an encounter.", r.step("check")),
                Action::Check { actor: id, skill: (*skill).into() }));
        }
    }
    if encounter.and_then(|e| e.current()).is_some_and(|t| t.token == token.0) {
        choices.push(("vtt.pf2e-end-turn".into(), "End turn".into(), "Pass to the next in initiative order.".into(),
            Action::EndTurn(id)));
    }
}

/// The table's encounter and DC card.
pub(crate) fn table(built: &mut Built, state: &State, gm: bool) {
    let mut values = vec![value("Ruleset", RULESET), value("Skill check DC", state.check_dc.unwrap_or(15))];
    let mut choices: Choices = Vec::new();
    match &state.encounter {
        Some(e) => {
            values.push(value("Round", e.round));
            for (n, t) in e.order.iter().enumerate() {
                values.push(value(format!("{} Character {}", if n == e.current { "Now:" } else { "Then:" }, t.token),
                    format!("initiative {}, {} actions", t.initiative, t.actions)));
            }
            if gm {
                choices.push(("vtt.pf2e-end-encounter".into(), "End encounter".into(), "Leave encounter mode.".into(),
                    Action::EndEncounter));
            }
        },
        None => {
            values.push(value("Mode", "free play"));
            if gm {
                choices.push(("vtt.pf2e-start-encounter".into(), "Start encounter".into(),
                    "Roll Perception for initiative; each turn has three actions and a reaction.".into(),
                    Action::StartEncounter));
            }
        },
    }
    if gm {
        for delta in [-1, 1] {
            choices.push((format!("vtt.pf2e-dc-{delta}"), format!("DC {delta:+}"),
                "A GM ruling on how hard checks are.".into(), Action::SetDc(delta)));
        }
    }
    let actions = choices.iter().map(|(intent, label, why, _)| action(intent, label, why, NULL, IntentEffect::DomainTruth))
        .collect();
    built.push(Item { id: "pf2e-table".into(), card: card("Encounter", values, &["experimental"]), actions },
        choices.into_iter().map(|(_, _, _, act)| Act::Ruleset(act)).collect());
}

/// Whether this table plays Pathfinder 2e.
pub(crate) fn plays(sections: &std::collections::BTreeMap<String, Vec<u8>>) -> Result<bool, String> {
    Ok(ruleset::edition(sections)? == pf2e::SYSTEM)
}
