//! Ordinary Pathfinder 2e play: equipment, encounter mode's three actions,
//! Strikes with the multiple attack penalty, skill checks, Raise a Shield and
//! the dying track. Hit Points stay edition state, read as vigour; a
//! critical hit and dropping to dying each wound the native body, and death
//! kills it, in the same accepted transaction (case 1020).

use super::*;
use cosmograph::Command;
use isocosm_campaign::EquipmentSlot;
use isocosm_system::pf2e::{
    draft::Build,
    play::{self as rules, Degree, Encounter, Gear, Turn},
    records::Item,
};

mod stride;
pub(crate) use stride::{ground, position, speed, squares};

/// Native cells one wound takes. Provisional: the size is an open fork.
const WOUND_CELLS: u32 = 1;
/// Player Core's simple DC for a 1st-level task.
const DEFAULT_DC: i64 = 15;

fn build(state: &State, id: TokenId) -> Result<Build, String> {
    state.drafts.get(&id).ok_or("no Pathfinder 2e character")?.finished(records::builtin())
}

/// The record behind an inventory item, if it is a Pathfinder one.
pub(crate) fn item_of(state: &State, id: TokenId, item: &ItemId) -> Option<&'static Item> {
    let instance = state.inventories.get(&id)?.items.get(item)?;
    records::builtin().items.get(instance.template.strip_prefix(ITEM)?)
}

pub(crate) fn gear(state: &State, id: TokenId) -> Gear<'static> {
    let slot = |s| state.inventories.get(&id).and_then(|inv| inv.equipped.get(&s)).and_then(|i| item_of(state, id, i));
    let off = slot(EquipmentSlot::OffHand);
    Gear {
        weapon: slot(EquipmentSlot::MainHand),
        armor: slot(EquipmentSlot::Body),
        shield: off.filter(|i| i.kind == "shield"),
    }
}

fn int(map: &MapDocument, id: TokenId, key: &str) -> i64 {
    map.sheet(id).and_then(|s| s.int(key)).unwrap_or(0)
}

/// Out of the fight: unconscious, dying or dead.
fn down(map: &MapDocument, id: TokenId) -> bool {
    int(map, id, "unconscious") > 0 || int(map, id, "dying") > 0 || int(map, id, "dead") > 0
}

fn d20(state: &mut State) -> Result<(i64, u16), String> {
    let (_, dice) = isocosm_core::roll("1d20", &mut state.rng).ok_or("d20 unavailable")?;
    Ok((i64::from(dice[0]), dice[0]))
}

/// The acting creature's turn, which must be the current one, spending
/// `cost` actions.
fn spend(state: &mut State, id: TokenId, cost: u8) -> Result<(), String> {
    let encounter = state.encounter.as_mut().ok_or("this needs encounter mode")?;
    let current = encounter.current;
    let turn = encounter.order.get_mut(current).ok_or("no current turn")?;
    if turn.token != id.0 {
        return Err("it is not this character's turn".into());
    }
    if turn.actions < cost {
        return Err(format!("{cost} action(s) needed; {} left this turn", turn.actions));
    }
    turn.actions -= cost;
    Ok(())
}

fn native(session: &mut Session, command: Command) -> Result<String, String> {
    session.command(command).map_err(|e| format!("native body refused: {e}"))
}

/// Damage to a Pathfinder character, with the dying track and its native
/// relations. Returns a note for the result log.
fn hurt(state: &mut State, map: &mut MapDocument, session: &mut Session, id: TokenId, amount: i64,
    critical: bool) -> Result<String, String> {
    if int(map, id, "dead") > 0 {
        return Err("the target is already dead".into());
    }
    let r = records::builtin();
    let derived = pf2e::derive::derive(r, &build(state, id)?);
    let entity = u64::from(id.0);
    let mut notes = Vec::new();
    if critical {
        native(session, Command::Wound { entity, part: None, cells: WOUND_CELLS })?;
        notes.push("the critical hit wounds the native body".to_owned());
    }
    let sheet = map.sheets.get_mut(&id).ok_or("target has no sheet")?;
    let hp = sheet.int("hp_current").unwrap_or(0);
    let wounded = sheet.int("wounded").unwrap_or(0);
    let dying = sheet.int("dying").unwrap_or(0);
    let mut new_dying = dying;
    if hp > 0 {
        let left = (hp - amount).max(0);
        sheet.set_int("hp_current", left);
        notes.push(format!("{amount} damage, {left} HP left"));
        if left == 0 {
            new_dying = rules::dying_gain(critical, wounded, false);
            sheet.set_int("unconscious", 1);
        }
    } else {
        new_dying = dying + rules::dying_gain(critical, wounded, true);
    }
    sheet.set_int("dying", new_dying);
    if new_dying > dying {
        notes.push(format!("dying {new_dying}"));
        if dying == 0 && hp > 0 {
            native(session, Command::Wound { entity, part: None, cells: WOUND_CELLS })?;
            notes.push("going dying wounds the native body".into());
        }
    }
    if new_dying >= derived.dying_max {
        die(map, session, id)?;
        notes.push("dead; the native body dies with them".into());
    }
    Ok(notes.join("; "))
}

fn die(map: &mut MapDocument, session: &mut Session, id: TokenId) -> Result<(), String> {
    let entity = u64::from(id.0);
    if session.sim.state().population.get(entity).is_some_and(|b| b.alive) {
        native(session, Command::Kill { entity })?;
    }
    let sheet = map.sheets.get_mut(&id).ok_or("no sheet")?;
    sheet.set_int("dead", 1);
    Ok(())
}

/// Start of a turn: actions refresh unless down; a dying character attempts
/// its recovery check.
fn begin_turn(state: &mut State, map: &mut MapDocument, session: &mut Session, rolls: &mut Vec<RollRecord>)
    -> Result<String, String> {
    let encounter = state.encounter.as_ref().ok_or("no encounter")?;
    let id = TokenId(encounter.current().ok_or("no turn")?.token);
    let mut note = format!("round {}, character {}'s turn", encounter.round, id.0);
    let dying = int(map, id, "dying");
    if dying > 0 && int(map, id, "dead") == 0 {
        let derived = pf2e::derive::derive(records::builtin(), &build(state, id)?);
        let dc = derived.recovery_dc + dying;
        let (total, face) = d20(state)?;
        let result = rules::degree(total, dc, face);
        rolls.push(RollRecord { by: format!("{} recovery", id.0), expr: format!("flat check vs DC {dc}"),
            dice: vec![face], total: total as i32 });
        let after = rules::recovery(dying, result);
        let sheet = map.sheets.get_mut(&id).ok_or("no sheet")?;
        sheet.set_int("dying", after);
        note.push_str(&format!("; recovery check {}: dying {after}", result.name()));
        if after == 0 {
            sheet.set_int("wounded", sheet.int("wounded").unwrap_or(0) + 1);
            note.push_str("; stable, wounded rises");
        } else if after >= derived.dying_max {
            die(map, session, id)?;
            note.push_str("; dead, and the native body dies");
        }
    }
    let actions = if down(map, id) { 0 } else { 3 };
    let encounter = state.encounter.as_mut().ok_or("no encounter")?;
    let current = encounter.current;
    let turn = &mut encounter.order[current];
    *turn = Turn { actions, reaction: actions > 0, attacks: 0, shield_raised: false, ..turn.clone() };
    Ok(note)
}

pub(super) fn apply(
    state: &mut State,
    map: &mut MapDocument,
    session: &mut Session,
    sections: &mut BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    action: &Action,
    payload: &[u8],
) -> Result<(String, Vec<RollRecord>), String> {
    use frame::authorize;
    let mut rolls = Vec::new();
    let note = match action {
        Action::Stride { actor, to } => stride::stride(state, map, session, sections, access, *actor, *to, false)?,
        Action::RuleStride { actor, to } => stride::stride(state, map, session, sections, access, *actor, *to, true)?,
        Action::Calibrate => {
            gm(access)?;
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Scale {
                cells_per_square: u32,
            }
            let scale: Scale = serde_json::from_slice(payload).map_err(|_| "enter native cells per five-foot square")?;
            if scale.cells_per_square == 0 {
                return Err("physical scale must be positive".into());
            }
            state.cells_per_square = Some(scale.cells_per_square);
            format!("GM set {} native cells per five-foot square", scale.cells_per_square)
        },
        Action::UseRulings => {
            gm(access)?;
            state.cells_per_square = None;
            "GM chose recorded rulings for distance".into()
        },
        Action::Equip { actor, item } => {
            let id = authorize(session, access, *actor)?;
            let record = item_of(state, id, item).ok_or("not a Pathfinder item in this character's inventory")?;
            let slot = match record.kind.as_str() {
                "weapon" => EquipmentSlot::MainHand,
                "shield" => EquipmentSlot::OffHand,
                "armor" => EquipmentSlot::Body,
                _ => return Err(format!("{} is carried, not readied", record.name)),
            };
            if state.encounter.as_ref().is_some_and(|e| e.turn(id.0).is_some()) {
                if slot == EquipmentSlot::Body {
                    return Err("donning armour takes minutes, not an encounter action".into());
                }
                spend(state, id, 1)?;
            }
            let two_handed = |state: &State| gear(state, id).weapon.is_some_and(|w| w.hands == Some(2));
            if slot == EquipmentSlot::OffHand && two_handed(state) {
                return Err("both hands hold a two-handed weapon".into());
            }
            let inventory = state.inventories.get_mut(&id).ok_or("no inventory")?;
            if record.hands == Some(2) {
                inventory.equipped.remove(&EquipmentSlot::OffHand);
            }
            inventory.equipped.insert(slot, item.clone());
            format!("Character {actor} readied {}", record.name)
        },
        Action::StartEncounter => {
            gm(access)?;
            if state.encounter.is_some() {
                return Err("an encounter is already running".into());
            }
            let ids: Vec<TokenId> = map.sheets.iter()
                .filter(|(id, s)| s.system == pf2e::SYSTEM && s.int("dead").unwrap_or(0) == 0
                    && session.sim.state().population.get(u64::from(id.0)).is_some_and(|b| b.alive))
                .map(|(id, _)| *id).collect();
            let mut order = Vec::new();
            for id in ids {
                let b = build(state, id)?;
                let modifier = rules::perception(&b) + b.initiative;
                let (die, face) = d20(state)?;
                rolls.push(RollRecord { by: format!("{} initiative", id.0),
                    expr: format!("Perception 1d20{modifier:+}"), dice: vec![face], total: (die + modifier) as i32 });
                order.push(Turn { token: id.0, initiative: die + modifier, actions: 3, reaction: true,
                    attacks: 0, shield_raised: false });
            }
            if order.is_empty() {
                return Err("no living Pathfinder 2e character to take part".into());
            }
            order.sort_by(|a, b| b.initiative.cmp(&a.initiative).then(a.token.cmp(&b.token)));
            state.encounter = Some(Encounter { round: 1, current: 0, order });
            format!("Encounter begins; {}", begin_turn(state, map, session, &mut rolls)?)
        },
        Action::EndEncounter => {
            gm(access)?;
            state.encounter.take().ok_or("no encounter is running")?;
            "Encounter ends".into()
        },
        Action::EndTurn(actor) => {
            let id = authorize(session, access, *actor)?;
            let encounter = state.encounter.as_mut().ok_or("no encounter is running")?;
            if encounter.current().map(|t| t.token) != Some(id.0) {
                return Err("it is not this character's turn".into());
            }
            encounter.current += 1;
            if encounter.current >= encounter.order.len() {
                encounter.current = 0;
                encounter.round += 1;
            }
            format!("Character {actor} ends the turn; {}", begin_turn(state, map, session, &mut rolls)?)
        },
        Action::RaiseShield(actor) => {
            let id = authorize(session, access, *actor)?;
            let shield = gear(state, id).shield.ok_or("no shield is held")?;
            spend(state, id, 1)?;
            let encounter = state.encounter.as_mut().ok_or("no encounter")?;
            let current = encounter.current;
            encounter.order[current].shield_raised = true;
            format!("Character {actor} raises the {} (+{} AC until the next turn)", shield.name, shield.ac.unwrap_or(0))
        },
        Action::Check { actor, skill } => {
            let id = authorize(session, access, *actor)?;
            if down(map, id) {
                return Err("this character cannot act".into());
            }
            let b = build(state, id)?;
            if state.encounter.as_ref().is_some_and(|e| e.turn(id.0).is_some()) {
                spend(state, id, 1)?;
            }
            let modifier = rules::skill(&b, &gear(state, id), skill);
            let dc = state.check_dc.unwrap_or(DEFAULT_DC);
            let (die, face) = d20(state)?;
            let result = rules::degree(die + modifier, dc, face);
            rolls.push(RollRecord { by: format!("{actor} {skill}"), expr: format!("1d20{modifier:+} vs DC {dc}"),
                dice: vec![face], total: (die + modifier) as i32 });
            format!("Character {actor} attempts {skill}: {}", result.name())
        },
        Action::SetDc(delta) => {
            gm(access)?;
            let dc = (state.check_dc.unwrap_or(DEFAULT_DC) + delta).clamp(1, 60);
            state.check_dc = Some(dc);
            format!("GM ruling: skill checks are against DC {dc}")
        },
        Action::Attack { actor, target } | Action::RuleAttack { actor, target } => {
            let a = authorize(session, access, *actor)?;
            let t = token(*target)?;
            if down(map, a) {
                return Err("this character cannot act".into());
            }
            let ruled = matches!(action, Action::RuleAttack { .. });
            let ruling: &[u8] = if ruled {
                gm(access)?;
                br#"{"squares":1,"reason":"GM ruled the target within reach with a clear path"}"#
            } else {
                payload
            };
            let (_, reach) = frame::reach(state, session, &*sections, access, *actor, *target, ruling)?;
            let ab = build(state, a)?;
            let weapon = gear(state, a).weapon.cloned().unwrap_or_else(rules::fist);
            if weapon.range.is_some() && !weapon.traits.iter().any(|x| x.starts_with("thrown")) {
                return Err("ranged Strikes come with ranged measurement".into());
            }
            spend(state, a, 1)?;
            let attacks = state.encounter.as_ref().and_then(|e| e.current()).map_or(0, |turn| turn.attacks);
            let map_penalty = rules::multiple_attack_penalty(&weapon, attacks);
            let bonus = rules::attack(&ab, &weapon) + map_penalty;
            let tb = build(state, t)?;
            let raised = state.encounter.as_ref().and_then(|e| e.turn(t.0)).is_some_and(|turn| turn.shield_raised);
            let ac = rules::armor_class(&tb, &gear(state, t), raised);
            let (die, face) = d20(state)?;
            let result = rules::degree(die + bonus, ac, face);
            rolls.push(RollRecord { by: format!("{actor} Strike"), expr: format!("1d20{bonus:+} vs AC {ac} (MAP {map_penalty})"),
                dice: vec![face], total: (die + bonus) as i32 });
            let encounter = state.encounter.as_mut().ok_or("no encounter")?;
            let current = encounter.current;
            encounter.order[current].attacks += 1;
            let mut note = format!("Character {actor} Strikes {target} with {}: {}", weapon.name, result.name());
            if result >= Degree::Success {
                let critical = result == Degree::CriticalSuccess;
                let (dice, flat, extra) = rules::damage(&ab, &weapon, critical);
                let (rolled, faces) = isocosm_core::roll(&dice, &mut state.rng).ok_or("weapon damage dice")?;
                let mut amount = (i64::from(rolled) + flat).max(1);
                if critical {
                    amount *= 2;
                }
                let mut all = faces;
                if let Some(extra) = extra {
                    let (more, faces) = isocosm_core::roll(&extra, &mut state.rng).ok_or("extra damage dice")?;
                    amount += i64::from(more);
                    all.extend(faces);
                }
                rolls.push(RollRecord { by: format!("{actor} damage"), expr: format!("{dice}{flat:+}{}",
                    if critical { " doubled" } else { "" }), dice: all, total: amount as i32 });
                note.push_str(&format!("; {}", hurt(state, map, session, t, amount, critical)?));
            }
            format!("{note}; {reach}")
        },
        Action::Damage { target, amount } => {
            gm(access)?;
            if *amount == 0 {
                return Err("damage needs a positive amount".into());
            }
            let t = token(*target)?;
            let note = hurt(state, map, session, t, i64::from(*amount), false)?;
            format!("GM ruled {amount} damage to {target}: {note}")
        },
        _ => return Err("not built for Pathfinder 2e".into()),
    };
    Ok((note, rolls))
}
