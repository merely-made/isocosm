//! Ars Magica character cards (case 1063): the creation draft one explained
//! decision at a time, then the sheet with its casting, study and Wounds,
//! and the covenant's library.
use super::{Act, Built};
use crate::ruleset::{Action, State, ars::season_ticks};
use chirograph::IntentEffect;
use cosmograph_host::endpoint::disclosure::{Item, NULL, action, card, value};
use cosmograph_overlay::vtt::{TimeIntent, VttIntent};
use isocosm_core::{MapDocument, TokenId};
use isocosm_system::active_effects::{Definitions, Kind};
use isocosm_system::ars::{self, draft::{Progress, walk}, records};

const RULESET: &str = "Ars Magica Fifth Edition; experimental";

type Choices = Vec<(String, String, String, Act)>;

fn push(built: &mut Built, id: String, title: &str, values: Vec<chirograph::CardValueV1>, choices: Choices,
    schema: Option<&'static str>) {
    let actions = choices.iter().map(|(intent, label, why, _)|
        action(intent, label, why, schema.filter(|_| intent == "vtt.ars-add-summa").unwrap_or(NULL), IntentEffect::DomainTruth)).collect();
    built.push(Item { id, card: card(title, values, &["experimental"]), actions },
        choices.into_iter().map(|(_, _, _, act)| act).collect());
}

fn rule(intent: impl Into<String>, label: impl Into<String>, why: impl Into<String>, action: Action) -> (String, String, String, Act) {
    (intent.into(), label.into(), why.into(), Act::Ruleset(action))
}

/// What the viewing seat may do with one body.
pub(crate) struct Seat {
    pub control: bool,
    pub gm: bool,
    pub alive: bool,
    /// The table has a saved roster, so downtime can be proposed.
    pub propose: bool,
    pub tick: u64,
}

fn line(r: &records::Records, sheet: &isocosm_core::SheetData, ids: impl Iterator<Item = String>) -> String {
    let parts: Vec<String> = ids.filter_map(|id| {
        let score = sheet.int(&ars::field(&id)).unwrap_or(0);
        let puissant = sheet.int(&ars::field(&format!("puissant:{id}"))).unwrap_or(0);
        (score > 0).then(|| if puissant > 0 { format!("{} {score} (+{puissant})", r.name(&id)) } else { format!("{} {score}", r.name(&id)) })
    }).collect();
    if parts.is_empty() { "none".into() } else { parts.join(", ") }
}

pub(crate) fn body(built: &mut Built, state: &State, map: &MapDocument, id: u64,
    token: TokenId, title: &str, seat: &Seat) -> Result<(), String> {
    let r = records::builtin();
    let mut values = vec![value("Native body", id), value("Ruleset", RULESET)];
    let mut choices: Choices = Vec::new();
    let Some(d) = state.drafts.get(&token) else {
        values.push(value("Character", "not started"));
        if seat.control {
            choices.push(rule("vtt.sheet-create", "Start Ars Magica character",
                "Begin a saved draft; each choice is explained and can be taken back.", Action::Create(id)));
        }
        push(built, format!("sheet:{id}"), title, values, choices, None);
        return Ok(());
    };
    let walked = walk(r, d)?;
    if let Some(sheet) = map.sheet(token) {
        let int = |key: &str| sheet.int(&ars::field(key)).unwrap_or(0);
        let build = &walked.build;
        let kind = if build.magus() { "Magus" } else { "Grog" };
        let house = build.house.as_deref().and_then(|h| r.house(h)).map_or(String::new(), |h| format!(" of House {}", h.name));
        values.push(value("Character", format!("{kind}{house}, {} years old", int("age"))));
        values.push(value("Characteristics", r.characteristics.iter()
            .map(|c| format!("{} {:+}", c.name, int(&c.id))).collect::<Vec<_>>().join(", ")));
        if build.magus() {
            values.push(value("Arts", line(r, sheet, r.arts.iter().map(|a| a.id.clone()))));
        }
        values.push(value("Abilities", line(r, sheet, r.abilities.iter().map(|a| a.id.clone()))));
        let traits: Vec<String> = build.traits.iter().map(|t| {
            let name = r.trait_(&t.id).map_or(t.id.clone(), |x| x.name.clone());
            t.target.as_ref().map_or(name.clone(), |target| format!("{name} ({})", r.name(target)))
        }).collect();
        values.push(value("Virtues and Flaws", traits.join(", ")));
        let fatigue = usize::try_from(int("fatigue_lost")).unwrap_or(0);
        values.push(value("Fatigue", r.fatigue.levels.get(fatigue).cloned().unwrap_or_else(|| "Unconscious".into())));
        let wounds: Vec<String> = r.wounds.order.iter().filter_map(|w| {
            let n = int(&format!("wounds:{w}"));
            (n > 0).then(|| format!("{n} {w}"))
        }).collect();
        values.push(value("Wounds", if int("dead") > 0 { "dead".into() } else if wounds.is_empty() { "none".into() } else { wounds.join(", ") }));
        let alive = seat.alive && int("dead") == 0;
        let studying = state.covenant.as_ref().and_then(|c| c.studies.get(&token).map(|s| (s.clone(), c.library[s.summa].clone())));
        match &studying {
            Some((study, book)) => values.push(value("This season", format!("studying the summa on {}; {} native ticks to go",
                r.name(&book.subject), study.until.saturating_sub(seat.tick)))),
            None => values.push(value("This season", "free")),
        }
        let defs = Definitions::builtin()?;
        for spell in build.spells.iter().filter_map(|s| r.spell(s)) {
            // A weapon procedure needs Ars equipment, which this slice lacks.
            let weapon = spell.procedure.as_deref().and_then(|p| defs.get(p).ok()).map(|d| d.kind == Kind::Weapon);
            let castable = weapon == Some(false);
            let status = match weapon {
                Some(false) => "castable here",
                Some(true) => "learned; it needs a wielded metal weapon, and Ars equipment is not in this slice",
                None => "learned; its procedure is not built at this table",
            };
            values.push(value(format!("Spell: {}", spell.name), format!("{} {} {}; {status}", r.name(&spell.technique),
                r.name(&spell.form), spell.level)));
            if seat.control && alive && castable {
                choices.push(rule(format!("vtt.ars-cast-{}", spell.id), format!("Cast {}", spell.name),
                    "Formulaic casting: Stamina, Technique, Form and a die against the spell's level; falling short costs Fatigue.",
                    Action::CastSpell { actor: id, spell: spell.id.clone() }));
            }
        }
        if seat.control && alive {
            match &studying {
                Some((study, _)) if seat.tick >= study.until => choices.push(rule("vtt.ars-complete", "Complete the season of study",
                    "Gain the summa's Quality in experience, never past its Level.", Action::Complete(id))),
                Some(_) => {},
                None => for (n, book) in state.covenant.iter().flat_map(|c| c.library.iter()).enumerate() {
                    choices.push(rule(format!("vtt.ars-study-{n}"), format!("Study the summa on {} for a season", r.name(&book.subject)),
                        "One source a season; the season passes as consented native downtime.", Action::Study { actor: id, summa: n }));
                },
            }
        }
        if let Some((study, _)) = &studying && seat.gm && seat.propose && seat.tick < study.until {
            choices.push(("vtt.ars-propose-season".into(), "Propose downtime for the rest of the season".into(),
                "Propose elapsed time through the table's ordinary consent path.".into(),
                Act::Send(VttIntent::Time(TimeIntent::Downtime { ticks: study.until - seat.tick }))));
        }
        if seat.gm && alive {
            for amount in [3, 8, 15] {
                choices.push(rule(format!("vtt.sheet-damage-{amount}"), format!("Rule a Damage Total of {amount}"),
                    "A GM ruling: damage past Soak becomes a Wound, and every Wound wounds the native body.",
                    Action::Damage { target: id, amount }));
            }
        }
        push(built, format!("sheet:{id}"), title, values, choices, None);
        return Ok(());
    }
    values.push(value("Choices made", walked.made.len()));
    for made in walked.made.iter().rev().take(4).rev() {
        values.push(value(&made.title, &made.label));
    }
    match &walked.progress {
        Progress::Pending(slot) => {
            values.push(value("Next decision", &slot.title));
            values.push(value("What it decides", r.step(slot.kind)));
            for o in slot.options.iter().filter(|o| o.refused.is_some()).take(6) {
                values.push(value(format!("Not available: {}", o.label), o.refused.as_deref().unwrap_or("")));
            }
            if seat.control {
                for o in slot.options.iter().filter(|o| o.refused.is_none()) {
                    let why = if o.summary.is_empty() { r.step(slot.kind).to_owned() } else { o.summary.clone() };
                    choices.push(rule(format!("vtt.draft-{}", o.value), o.label.clone(), why,
                        Action::Choose { actor: id, slot: slot.id.clone(), value: o.value.clone() }));
                }
            }
        },
        Progress::Complete => {
            values.push(value("Next decision", "none: ready to finish"));
            values.push(value("What it decides", r.step("finish")));
            if seat.control {
                choices.push(rule("vtt.draft-finish", "Finish character", r.step("finish"), Action::Finish(id)));
            }
        },
    }
    if seat.control && !walked.made.is_empty() {
        choices.push(rule("vtt.draft-undo", "Take back last choice",
            "Undo the most recent choice; later steps follow from what remains.", Action::Undo(id)));
    }
    if seat.control || seat.gm {
        choices.push(rule("vtt.draft-discard", "Discard draft", "Abandon this unfinished character.", Action::Discard(id)));
    }
    push(built, format!("sheet:{id}"), title, values, choices, None);
    Ok(())
}

/// The covenant's library and the season's length.
pub(crate) fn covenant(built: &mut Built, state: &State, session: &cosmograph::Session, gm: bool) {
    let r = records::builtin();
    let mut values = vec![value("Ruleset", RULESET), value("A season", format!("{} native ticks", season_ticks(session)))];
    let library: Vec<_> = state.covenant.iter().flat_map(|c| c.library.iter()).collect();
    if library.is_empty() {
        values.push(value("Library", "empty"));
    }
    for book in library {
        values.push(value(format!("Summa on {}", r.name(&book.subject)), format!("Level {}, Quality {}", book.level, book.quality)));
    }
    let mut choices: Choices = Vec::new();
    if gm {
        choices.push(rule("vtt.ars-add-summa", "Add a summa to the library",
            "A GM ruling: the subject (an Art or Ability id), its Level and its Quality.", Action::AddSumma));
    }
    push(built, "ars-covenant".into(), "Covenant", values, choices,
        Some(r#"{"type":"object","properties":{"subject":{"type":"string"},"level":{"type":"integer","minimum":1},"quality":{"type":"integer","minimum":1}},"required":["subject","level","quality"],"additionalProperties":false}"#));
}
