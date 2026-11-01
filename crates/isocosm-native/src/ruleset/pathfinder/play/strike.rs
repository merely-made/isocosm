//! A Strike (cases 1047, 1050): melee within reach or ranged within six
//! increments, counted with Player Core's alternating diagonals; off-guard
//! from a GM ruling, unconsciousness or a party ally's flank; Sneak Attack;
//! ammunition spent; and a hit on a raised shield paused for Shield Block.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn strike(state: &mut State, map: &mut MapDocument, session: &mut Session,
    sections: &BTreeMap<String, Vec<u8>>, access: OrdinaryAccess, action: &Action, payload: &[u8],
    rolls: &mut Vec<RollRecord>) -> Result<String, String> {
    use frame::authorize;
    let (Action::Attack { actor, target } | Action::RuleAttack { actor, target }) = action else {
        return Err("not a Strike".into());
    };

    let a = authorize(session, access, *actor)?;
    let t = token(*target)?;
    if down(map, a) {
        return Err("this character cannot act".into());
    }
    let ab = build(state, a)?;
    let weapon = gear(state, a).weapon.cloned().unwrap_or_else(rules::fist);
    let ranged = weapon.range.is_some();
    let ruled = matches!(action, Action::RuleAttack { .. });
    let ruling: &[u8] = if ruled {
        gm(access)?;
        if ranged {
            br#"{"squares":1,"reason":"GM ruled the target within the first range increment with a clear line"}"#
        } else {
            br#"{"squares":1,"reason":"GM ruled the target within reach with a clear path"}"#
        }
    } else {
        payload
    };
    let mut distance = None;
    let (_, reach) = frame::reach_with(state, session, &*sections, access, *actor, *target, ruling,
        &mut |start, end, scale| {
            let feet = combat::feet(start, end, scale);
            if !ranged && !combat::in_reach(&weapon, start, end, scale) {
                return Err(format!("{feet} feet is beyond the {}'s {}-foot reach", weapon.name, combat::reach(&weapon)));
            }
            distance = Some(feet);
            Ok((u32::try_from(feet / 5).unwrap_or(u32::MAX), format!("{feet} feet, measured at {scale} cells per square")))
        })?;
    let range = match (ranged, distance) {
        (true, Some(feet)) => combat::range_penalty(&weapon, feet)?,
        _ => 0,
    };
    spend(state, a, 1)?;
    let spent = spend_ammunition(state, a, &weapon)?;
    let attacks = state.encounter.as_ref().and_then(|e| e.current()).map_or(0, |turn| turn.attacks);
    let map_penalty = rules::multiple_attack_penalty(&weapon, attacks);
    let bonus = rules::attack(&ab, &weapon) + map_penalty + range;
    let tb = build(state, t)?;
    let raised = state.encounter.as_ref().and_then(|e| e.turn(t.0)).is_some_and(|turn| turn.shield_raised);
    let unconscious = int(map, t, "unconscious") > 0;
    let flanker = if ranged || distance.is_none() { None } else { flanking(state, map, session, sections, a, t)? };
    let off_guard = unconscious || int(map, t, "off_guard") > 0 || flanker.is_some();
    let ac = rules::armor_class(&tb, &gear(state, t), raised) - if off_guard { 2 } else { 0 }
        - if unconscious { 4 } else { 0 };
    let (die, face) = d20(state)?;
    let result = rules::degree(die + bonus, ac, face);
    rolls.push(RollRecord { by: format!("{actor} Strike"),
        expr: format!("1d20{bonus:+} vs AC {ac} (MAP {map_penalty}, range {range}{}{})",
            if off_guard { ", off-guard" } else { "" },
            flanker.map_or(String::new(), |ally| format!(", flanked with {ally}"))),
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
        let mut all = faces;
        let mut expr = format!("{dice}{flat:+}");
        if off_guard && combat::sneak_weapon(&ab, &weapon) {
            let (sneak, faces) = isocosm_core::roll("1d6", &mut state.rng).ok_or("sneak attack dice")?;
            amount += i64::from(sneak);
            all.extend(faces);
            expr.push_str(" +1d6 precision (Sneak Attack)");
        }
        if critical {
            amount *= 2;
            expr.push_str(" doubled");
        }
        if let Some(extra) = extra {
            let (more, faces) = isocosm_core::roll(&extra, &mut state.rng).ok_or("extra damage dice")?;
            amount += i64::from(more);
            all.extend(faces);
            expr.push_str(&format!(" +{extra}"));
        }
        rolls.push(RollRecord { by: format!("{actor} damage"), expr, dice: all, total: amount as i32 });
        if shield::available(state, map, t) {
            state.pending = Some(rules::Pending { attacker: a.0, target: t.0, amount, critical });
            note.push_str(&format!("; {amount} damage waits on character {target}'s Shield Block decision"));
        } else {
            note.push_str(&format!("; {}", hurt(state, map, session, t, amount, critical)?));
        }
    }
    Ok(format!("{note}; {spent}{reach}"))
}

/// Spend one piece of ammunition, or the thrown weapon itself, for a ranged
/// Strike. Returns a note for the log.
fn spend_ammunition(state: &mut State, id: TokenId, weapon: &Item) -> Result<String, String> {
    let thrown = weapon.range.is_some() && weapon.traits.iter().any(|t| t.starts_with("thrown"));
    let wanted = match (combat::ammunition(weapon), thrown) {
        (Some(ammo), _) => format!("{ITEM}{ammo}"),
        (None, true) => format!("{ITEM}{}", weapon.id),
        (None, false) => return Ok(String::new()),
    };
    let inventory = state.inventories.get_mut(&id).ok_or("no inventory")?;
    let (item_id, instance) = inventory.items.iter_mut()
        .find(|(_, i)| i.template == wanted && i.quantity > 0)
        .ok_or_else(|| format!("no {} left", wanted.trim_start_matches(ITEM)))?;
    let item_id = item_id.clone();
    instance.quantity -= 1;
    let (name, left) = (instance.name.clone(), instance.quantity);
    if thrown {
        inventory.equipped.retain(|_, held| *held != item_id);
    }
    if left == 0 {
        inventory.items.remove(&item_id);
    }
    Ok(if thrown { format!("the {name} lands near the target; ") } else { format!("{name}: {left} left; ") })
}

/// A party ally flanking `target` with `actor`: both able to act, both with
/// the target in melee reach, on opposite sides of its square (case 1050).
fn flanking(state: &State, map: &MapDocument, session: &Session, sections: &BTreeMap<String, Vec<u8>>,
    actor: TokenId, target: TokenId) -> Result<Option<u32>, String> {
    let Some(scale) = state.cells_per_square else { return Ok(None) };
    let sim = &session.sim;
    let (a, t) = (u64::from(actor.0), u64::from(target.0));
    let mut allies = std::collections::BTreeSet::new();
    for party in sim.genesis().rules.party_relations.keys() {
        let members = cosmograph::party::members(sim, party).map_err(|e| e.to_string())?;
        if members.iter().any(|m| m.member == a) {
            allies.extend(members.iter().map(|m| m.member).filter(|m| *m != a && *m != t));
        }
    }
    let square = |p: [i64; 3]| [p[0].div_euclid(i64::from(scale)), p[2].div_euclid(i64::from(scale))];
    let (from, at) = (frame::point(session, sections, a)?, frame::point(session, sections, t)?);
    for ally in allies {
        let Ok(id) = token(ally) else { continue };
        if map.sheet(id).is_none_or(|s| s.system != pf2e::SYSTEM) || down(map, id)
            || !sim.state().population.get(ally).is_some_and(|b| b.alive) {
            continue;
        }
        let Ok(there) = frame::point(session, sections, ally) else { continue };
        let weapon = gear(state, id).weapon.filter(|w| w.range.is_none()).cloned().unwrap_or_else(rules::fist);
        if combat::in_reach(&weapon, there, at, scale) && combat::flanks(square(from), square(there), square(at)) {
            return Ok(Some(id.0));
        }
    }
    Ok(None)
}

