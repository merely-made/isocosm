//! Shield Block (case 1050): a hit on a raised shield pauses; the defender's
//! seat chooses. The shield stops damage up to its Hardness, and the shield
//! and its bearer each take the rest. A shield at half its Hit Points or
//! fewer is broken and neither blocks nor raises.

use super::*;

fn shield_key(item: &ItemId) -> String {
    format!("shield_hp:{}", item.0)
}

/// The held shield, its item id and current Hit Points.
fn held(state: &State, map: &MapDocument, id: TokenId) -> Option<(ItemId, &'static Item, i64)> {
    let item = state.inventories.get(&id)?.equipped.get(&EquipmentSlot::OffHand)?.clone();
    let record = item_of(state, id, &item).filter(|i| i.kind == "shield")?;
    let hp = map.sheet(id).and_then(|s| s.int(&shield_key(&item))).unwrap_or(record.hp.unwrap_or(0));
    Some((item, record, hp))
}

pub(crate) fn broken(state: &State, map: &MapDocument, id: TokenId) -> bool {
    held(state, map, id).is_some_and(|(_, record, hp)| hp * 2 <= record.hp.unwrap_or(0))
}

/// Whether the defender may Shield Block now: the feat, a raised and
/// unbroken shield, and an unspent reaction.
pub(super) fn available(state: &State, map: &MapDocument, id: TokenId) -> bool {
    let turn = state.encounter.as_ref().and_then(|e| e.turn(id.0));
    turn.is_some_and(|t| t.shield_raised && t.reaction)
        && build(state, id).is_ok_and(|b| b.feats.iter().any(|f| f == "shield-block"))
        && held(state, map, id).is_some() && !broken(state, map, id)
}

pub(super) fn decide(state: &mut State, map: &mut MapDocument, session: &mut Session, access: OrdinaryAccess,
    target: u64, block: bool) -> Result<String, String> {
    let id = frame::authorize(session, access, target)?;
    let pending = state.pending.clone().ok_or("no hit is waiting on a reaction")?;
    if pending.target != id.0 {
        return Err("the waiting hit is on another character".into());
    }
    state.pending = None;
    let mut amount = pending.amount;
    let mut note = String::new();
    if block {
        let (item, record, hp) = held(state, map, id).ok_or("no shield is held")?;
        let stopped = amount.min(record.hardness.unwrap_or(0));
        let rest = amount - stopped;
        let left = (hp - rest).max(0);
        map.sheets.get_mut(&id).ok_or("no sheet")?.set_int(&shield_key(&item), left);
        let encounter = state.encounter.as_mut().ok_or("no encounter")?;
        if let Some(turn) = encounter.order.iter_mut().find(|t| t.token == id.0) {
            turn.reaction = false;
            if left * 2 <= record.hp.unwrap_or(0) {
                turn.shield_raised = false;
            }
        }
        amount = rest;
        note = format!("Shield Block: the {} stops {stopped}, takes {rest} ({left} HP left{}); ",
            record.name, if left * 2 <= record.hp.unwrap_or(0) { ", broken" } else { "" });
    }
    let hurt = if amount > 0 { hurt(state, map, session, id, amount, pending.critical)? } else { "no damage".into() };
    Ok(format!("Character {target} {}: {note}{hurt}", if block { "blocks" } else { "takes the hit" }))
}
