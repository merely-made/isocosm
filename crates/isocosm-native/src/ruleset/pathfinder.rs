//! Pathfinder 2e at the table: creation as a saved draft (cases 1026, 1030)
//! and ordinary play (`play`). The seat that controls the body drafts and
//! plays its character; any controlling seat, a GM seat included, may
//! discard an unfinished draft. Nothing here assumes a single GM.

use super::*;
use cosmograph_overlay::Seat;
use isocosm_campaign::ItemInstance;
use isocosm_system::pf2e::{derive::derive, records};

pub(crate) mod play;

/// Every saved draft still walks, and every Pathfinder sheet has the
/// finished draft it was made from.
pub(super) fn check(state: &State) -> Result<(), String> {
    let r = records::builtin();
    for draft in state.drafts.values() {
        pf2e::draft::walk(r, draft)?;
    }
    for (id, sheet) in &state.sheets {
        if sheet.system == pf2e::SYSTEM {
            state
                .drafts
                .get(id)
                .ok_or("a Pathfinder 2e sheet has no creation record")?
                .finished(r)?;
        }
    }
    Ok(())
}

fn unfinished<'a>(state: &'a mut State, map: &MapDocument, id: TokenId) -> Result<&'a mut Draft, String> {
    if map.sheets.contains_key(&id) {
        return Err("this character is already finished".into());
    }
    state.drafts.get_mut(&id).ok_or_else(|| "this body has no creation draft".into())
}

/// A GM-authority seat; a table may have none, one or several.
pub(super) fn gm(access: OrdinaryAccess) -> Result<(), String> {
    (access.seat == Seat::Dm).then_some(()).ok_or_else(|| "this needs a GM seat".into())
}

/// The template a Pathfinder item instance names.
pub(super) const ITEM: &str = "pf2e:";

pub(super) fn apply(
    state: &mut State,
    map: &mut MapDocument,
    session: &mut Session,
    sections: &BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    action: &Action,
    payload: &[u8],
) -> Result<(String, Vec<RollRecord>), String> {
    use frame::authorize;
    let r = records::builtin();
    let note = match action {
        Action::Create(actor) => {
            let id = authorize(session, access, *actor)?;
            if map.sheets.contains_key(&id) || state.drafts.contains_key(&id) {
                return Err("this body already has a character or a draft".into());
            }
            state.drafts.insert(id, Draft::default());
            format!("Pathfinder 2e draft started for native body {actor}; experimental ruleset facts")
        },
        Action::Choose { actor, slot, value } => {
            let id = authorize(session, access, *actor)?;
            let made = unfinished(state, map, id)?.choose(r, slot, value)?;
            format!("Character {actor} chose {} for {}", made.label, made.title)
        },
        Action::Undo(actor) => {
            let id = authorize(session, access, *actor)?;
            let draft = unfinished(state, map, id)?;
            let last = pf2e::draft::walk(r, draft)?.made.pop().ok_or("nothing to take back")?;
            draft.undo()?;
            format!("Character {actor} took back {} for {}", last.label, last.title)
        },
        Action::Finish(actor) => {
            let id = authorize(session, access, *actor)?;
            let build = unfinished(state, map, id)?.finished(r)?;
            let derived = derive(r, &build);
            let mut sheet = SheetData::new(pf2e::SYSTEM);
            sheet.set_text("name", format!("Character {actor}"));
            sheet.set_int("level", derived.level);
            sheet.set_int("hp_current", derived.hp_max);
            sheet.set_int("coins_cp", build.coins);
            map.set_sheet(id, sheet);
            let inventory = state.inventories.entry(id).or_default();
            for item in &build.gear {
                let item_id = ItemId(format!("ruleset:pf2e:{}", state.next_item));
                state.next_item = state.next_item.checked_add(1).ok_or("item identity overflow")?;
                inventory.items.insert(item_id.clone(), ItemInstance {
                    id: item_id,
                    template: format!("{ITEM}{item}"),
                    name: r.items[item].name.clone(),
                    quantity: 1,
                    tags: vec![r.items[item].kind.clone()],
                    modifiers: vec![],
                    appearance_layers: vec![], material: None, make: None, condition: None,
                });
            }
            let ancestry = build.ancestry.as_ref().map_or("", |a| r.ancestries[a].name.as_str());
            let class = build.class.as_ref().map_or("", |c| r.classes[c].name.as_str());
            format!(
                "Character {actor} finished as a 1st-level {ancestry} {class} with {} Hit Points; gear is unmarked table equipment",
                derived.hp_max
            )
        },
        Action::Discard(actor) => {
            let id = authorize(session, access, *actor)?;
            unfinished(state, map, id)?;
            state.drafts.remove(&id);
            format!("Character {actor}'s unfinished draft was discarded")
        },
        _ => return play::apply(state, map, session, sections, access, action, payload),
    };
    Ok((note, Vec::new()))
}
