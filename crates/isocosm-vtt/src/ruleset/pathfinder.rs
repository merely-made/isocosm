//! Pathfinder 2e at the table: creation as a saved draft (cases 1026, 1030).
//! The seat that controls the body drafts its character; any seat with that
//! control, a GM seat included, may discard an unfinished draft. Nothing here
//! assumes a single GM: no step needs one.

use super::*;
use isometry_system::pf2e::{derive::derive, records};

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

pub(super) fn apply(
    state: &mut State,
    map: &mut MapDocument,
    session: &Session,
    access: OrdinaryAccess,
    action: &Action,
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
            map.set_sheet(id, sheet);
            let ancestry = build.ancestry.as_ref().map_or("", |a| r.ancestries[a].name.as_str());
            let class = build.class.as_ref().map_or("", |c| r.classes[c].name.as_str());
            format!(
                "Character {actor} finished as a 1st-level {ancestry} {class} with {} Hit Points",
                derived.hp_max
            )
        },
        Action::Discard(actor) => {
            let id = authorize(session, access, *actor)?;
            unfinished(state, map, id)?;
            state.drafts.remove(&id);
            format!("Character {actor}'s unfinished draft was discarded")
        },
        _ => return Err("Pathfinder 2e play (equipment, actions, incapacitation) is the next increment".into()),
    };
    Ok((note, Vec::new()))
}
