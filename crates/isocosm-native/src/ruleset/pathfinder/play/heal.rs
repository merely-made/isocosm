//! Healing (case 1050): Battle Medicine as one encounter action; Treat
//! Wounds (10 minutes) and rest (8 hours) begun at the table and completed
//! once the table's consented native downtime has passed.

use super::*;
use isocosm_system::pf2e::play::{Activity, Immunity, medicine_dice};

const TOOLKIT: &str = "healers-toolkit";
/// The trained DC Player Core sets for Treat Wounds and Battle Medicine.
const MEDICINE_DC: i64 = 15;

/// Native ticks covering `seconds`.
pub(crate) fn ticks(session: &Session, seconds: u64) -> u64 {
    let micro = session.sim.genesis().rules.tick_microseconds().max(1);
    (seconds * 1_000_000).div_ceil(micro)
}

fn toolkit(state: &State, id: TokenId) -> Result<(), String> {
    let wanted = format!("{ITEM}{TOOLKIT}");
    state.inventories.get(&id).is_some_and(|inv| inv.items.values().any(|i| i.template == wanted))
        .then_some(()).ok_or_else(|| "needs a healer's toolkit".into())
}

fn immune(state: &State, kind: &str, healer: TokenId, target: TokenId, now: u64) -> Result<(), String> {
    match state.immunities.iter().find(|i| i.kind == kind && i.healer == healer.0 && i.target == target.0 && i.until > now) {
        Some(i) => Err(format!("character {} cannot benefit from this {kind} again until native tick {}", target.0, i.until)),
        None => Ok(()),
    }
}

/// Restore Hit Points: a dying creature stops dying (wounded rises), and
/// one with Hit Points again wakes.
fn restore(map: &mut MapDocument, r: &records::Records, build: &Build, id: TokenId, amount: i64) -> Result<String, String> {
    let max = pf2e::derive::derive(r, build).hp_max;
    let sheet = map.sheets.get_mut(&id).ok_or("no sheet")?;
    if sheet.int("dead").unwrap_or(0) > 0 {
        return Err("the dead are not healed by Medicine".into());
    }
    let hp = (sheet.int("hp_current").unwrap_or(0) + amount).min(max);
    sheet.set_int("hp_current", hp);
    let mut note = format!("{amount} HP restored, {hp} / {max}");
    if hp > 0 {
        if sheet.int("dying").unwrap_or(0) > 0 {
            sheet.set_int("dying", 0);
            sheet.set_int("wounded", sheet.int("wounded").unwrap_or(0) + 1);
            note.push_str("; no longer dying, wounded rises");
        }
        sheet.set_int("unconscious", 0);
    }
    Ok(note)
}

/// A Medicine check at DC 15 and its healing or harm.
fn medicine(state: &mut State, map: &mut MapDocument, session: &mut Session, healer: TokenId, target: TokenId,
    rolls: &mut Vec<RollRecord>) -> Result<(String, Degree), String> {
    let hb = build(state, healer)?;
    if !hb.trained("medicine") {
        return Err("needs training in Medicine".into());
    }
    let modifier = rules::skill(&hb, &gear(state, healer), "medicine");
    let (die, face) = d20(state)?;
    let result = rules::degree(die + modifier, MEDICINE_DC, face);
    rolls.push(RollRecord { by: format!("{} Medicine", healer.0), expr: format!("1d20{modifier:+} vs DC {MEDICINE_DC}"),
        dice: vec![face], total: (die + modifier) as i32 });
    let (dice, heals) = medicine_dice(result);
    if dice == "0" {
        return Ok((format!("{}: no effect", result.name()), result));
    }
    let (amount, faces) = isocosm_core::roll(dice, &mut state.rng).ok_or("healing dice")?;
    rolls.push(RollRecord { by: format!("{} {}", healer.0, if heals { "healing" } else { "harm" }),
        expr: dice.into(), dice: faces, total: amount });
    let note = if heals {
        restore(map, records::builtin(), &build(state, target)?, target, i64::from(amount))?
    } else {
        hurt(state, map, session, target, i64::from(amount), false)?
    };
    Ok((format!("{}: {note}", result.name()), result))
}

pub(super) fn battle_medicine(state: &mut State, map: &mut MapDocument, session: &mut Session,
    sections: &BTreeMap<String, Vec<u8>>, access: OrdinaryAccess, actor: u64, target: u64,
    rolls: &mut Vec<RollRecord>) -> Result<String, String> {
    let a = frame::authorize(session, access, actor)?;
    let t = token(target)?;
    if down(map, a) {
        return Err("this character cannot act".into());
    }
    if !build(state, a)?.feats.iter().any(|f| f == "battle-medicine") {
        return Err("needs the Battle Medicine feat".into());
    }
    toolkit(state, a)?;
    let now = session.sim.state().tick;
    immune(state, "Battle Medicine", a, t, now)?;
    let adjacent = match state.cells_per_square {
        Some(scale) if a != t => {
            let feet = combat::feet(frame::point(session, sections, actor)?, frame::point(session, sections, target)?, scale);
            if feet > 5 {
                return Err(format!("{feet} feet away; Battle Medicine needs an adjacent creature"));
            }
            format!("adjacent, {feet} feet")
        },
        Some(_) => "on yourself".into(),
        None if a == t => "on yourself".into(),
        None => {
            gm(access)?;
            "GM ruled the creature adjacent".into()
        },
    };
    if state.encounter.as_ref().is_some_and(|e| e.turn(a.0).is_some()) {
        spend(state, a, 1)?;
    }
    let (note, _) = medicine(state, map, session, a, t, rolls)?;
    state.immunities.push(Immunity { kind: "Battle Medicine".into(), healer: a.0, target: t.0,
        until: now + ticks(session, 86_400) });
    Ok(format!("Character {actor} uses Battle Medicine on {target} ({adjacent}): {note}"))
}

pub(super) fn begin(state: &mut State, map: &MapDocument, session: &Session, access: OrdinaryAccess,
    actor: u64, target: u64, kind: &str) -> Result<String, String> {
    let a = frame::authorize(session, access, actor)?;
    let t = token(target)?;
    if state.encounter.is_some() {
        return Err("Treat Wounds and rest are for outside encounters".into());
    }
    if state.activities.iter().any(|x| x.actor == a.0) {
        return Err("this character is already busy".into());
    }
    let now = session.sim.state().tick;
    let seconds = match kind {
        "treat-wounds" => {
            if !build(state, a)?.trained("medicine") {
                return Err("needs training in Medicine".into());
            }
            toolkit(state, a)?;
            immune(state, "Treat Wounds", a, t, now)?;
            600
        },
        "rest" if a == t && !down(map, a) => 8 * 3600,
        "rest" => return Err("a character rests for themselves".into()),
        _ => return Err("unknown activity".into()),
    };
    let until = now + ticks(session, seconds);
    state.activities.push(Activity { kind: kind.into(), actor: a.0, target: t.0, until });
    Ok(format!("Character {actor} begins {} for {target}; completes after native tick {until}",
        if kind == "rest" { "8 hours' rest" } else { "Treat Wounds (10 minutes)" }))
}

pub(super) fn complete(state: &mut State, map: &mut MapDocument, session: &mut Session, access: OrdinaryAccess,
    actor: u64, rolls: &mut Vec<RollRecord>) -> Result<String, String> {
    let a = frame::authorize(session, access, actor)?;
    let now = session.sim.state().tick;
    let index = state.activities.iter().position(|x| x.actor == a.0).ok_or("no activity is under way")?;
    let activity = state.activities[index].clone();
    if now < activity.until {
        return Err(format!("{} native ticks remain; propose downtime first", activity.until - now));
    }
    state.activities.remove(index);
    let t = TokenId(activity.target);
    if activity.kind == "rest" {
        let b = build(state, t)?;
        let per = b.modifier("con").max(1) * rules::LEVEL;
        let amount = if b.feats.iter().any(|f| f == "fast-recovery") { per * 2 } else { per };
        let note = restore(map, records::builtin(), &b, t, amount)?;
        return Ok(format!("Character {actor} rests 8 hours: {note}"));
    }
    let (note, result) = medicine(state, map, session, a, t, rolls)?;
    if result >= Degree::Success {
        map.sheets.get_mut(&t).ok_or("no sheet")?.set_int("wounded", 0);
    }
    state.immunities.push(Immunity { kind: "Treat Wounds".into(), healer: a.0, target: t.0,
        until: now + ticks(session, 3600) });
    Ok(format!("Character {actor} Treats {}'s Wounds: {note}{}", t.0,
        if result >= Degree::Success { "; wounded removed" } else { "" }))
}
