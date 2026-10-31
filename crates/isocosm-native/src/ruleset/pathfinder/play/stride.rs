//! Stride (case 1042): one action, carrying the table's native cell batch
//! in the same transaction. The move is checked against Speed at the
//! campaign's cells per square; with no scale set, a GM seat's recorded
//! ruling stands in. Only characters admitted with a table position move.

use super::*;
use crate::native::{GameBindings, PositionBinding, submit_table};
use cosmograph_overlay::{
    EntityHandle, Intent, PlaceHandle, Tick, WorldPoint,
    vtt::{Cell, TableAct, TableActKind, TableBatch, VttIntent},
};

/// The admitted table position of a native body.
pub(crate) fn position(sections: &BTreeMap<String, Vec<u8>>, entity: u64) -> Result<PositionBinding, String> {
    let game = GameBindings::decode(sections.get("vtt").map_or(&[], Vec::as_slice))?;
    let (key, _) = game.characters().iter().find(|(_, recipe)| recipe.entity == entity)
        .ok_or("Stride moves admitted characters only; this body has no native recipe admission")?;
    game.positions().get(key).cloned()
        .ok_or_else(|| "Stride moves placed characters only; this body has no table position".into())
}

/// Speed in feet: the build's land Speed less armour and shield penalties
/// (armour's eased by 5 feet when its Strength is met; Unburdened Iron
/// ignores it).
pub(crate) fn speed(build: &Build, gear: &Gear) -> i64 {
    let r = records::builtin();
    let base = build.ancestry.as_ref().map_or(25, |a| r.ancestries[a].speed) + build.speed_bonus;
    let armour = gear.armor.map_or(0, |a| {
        let penalty = a.speed.unwrap_or(0);
        if build.feats.iter().any(|f| f == "unburdened-iron") { 0 }
        else if build.modifier("str") >= a.strength.unwrap_or(i64::MAX) { (penalty + 5).min(0) }
        else { penalty }
    });
    let shield = gear.shield.and_then(|s| s.speed).unwrap_or(0);
    (base + armour + shield).max(5)
}

/// The terrain a position stands in: its window's volume and patches,
/// lifted once for any number of landings.
pub(crate) struct Ground {
    volume: isometer_space::volume::Volume,
    places: isometer_space::places::Places,
}

std::thread_local! {
    /// The last ground lifted, keyed by site, window, places and terrain
    /// revision: card snapshots ask for it on every frame of a turn.
    static GROUND: std::cell::RefCell<Option<(String, std::rc::Rc<Ground>)>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn ground(session: &Session, from: &PositionBinding) -> Option<std::rc::Rc<Ground>> {
    let p = &from.placement;
    let key = format!("{}:{:?}:{:?}:{}:{:?}:{}", p.site, p.window.min, p.window.max, p.window.depth, p.places,
        session.sim.terrain_revision());
    if let Some(hit) = GROUND.with(|g| g.borrow().as_ref().filter(|(k, _)| *k == key).map(|(_, v)| v.clone())) {
        return Some(hit);
    }
    let volume = session.sim.volume(p.site, p.window.min, p.window.max, p.window.depth).ok()?;
    let places = isometer_space::places::Places::derive(&volume, p.places).ok()?;
    let ground = std::rc::Rc::new(Ground { volume, places });
    GROUND.with(|g| *g.borrow_mut() = Some((key, ground.clone())));
    Some(ground)
}

impl Ground {
    /// The highest clear cell of the column at `x, z`, if its patch derives.
    pub(crate) fn landing(&self, x: i64, z: i64) -> Option<[i64; 3]> {
        let v = &self.volume;
        if !v.holds(x, z) {
            return None;
        }
        (v.datum..v.ceiling()).rev().map(|y| [x, y, z])
            .find(|at| v.material(*at) == Some(v.materials.air) && self.places.place_at(*at).is_some())
    }
}

pub(super) fn stride(state: &mut State, map: &MapDocument, session: &mut Session,
    sections: &mut BTreeMap<String, Vec<u8>>, access: OrdinaryAccess, actor: u64, to: [i64; 3], ruled: bool)
    -> Result<String, String> {
    let id = frame::authorize(session, access, actor)?;
    if down(map, id) {
        return Err("this character cannot act".into());
    }
    let from = position(sections, actor)?;
    let start = from.current(session)?;
    let feet = speed(&build(state, id)?, &gear(state, id));
    let measure = match (state.cells_per_square, ruled) {
        (Some(scale), false) => {
            let needed = combat::feet(start, to, scale);
            if needed > feet as u64 {
                return Err(format!("{needed} feet exceeds Speed {feet} feet"));
            }
            format!("{needed} feet of Speed {feet}, measured at {scale} cells per square, diagonals 5 then 10")
        },
        (Some(_), true) => return Err("with a physical scale set, Stride is measured".into()),
        (None, false) => return Err("no physical scale is set; a GM seat rules the Stride".into()),
        (None, true) => {
            gm(access)?;
            format!("GM ruled the move within Speed {feet} feet")
        },
    };
    spend(state, id, 1)?;
    let to32 = to.map(|v| i32::try_from(v).map_err(|_| "destination out of range")).map(|v| v.unwrap_or(i32::MAX));
    let envelope = cosmograph_overlay::vtt::VttIntentEnvelope {
        tick: Tick(session.sim.state().tick),
        participant: access.participant,
        intent: Intent::Game(VttIntent::Table(TableBatch { acts: vec![TableAct {
            actor: EntityHandle(actor),
            kind: TableActKind::Move { to: Cell { site: PlaceHandle(from.placement.site), at: WorldPoint(to32) } },
        }] })),
    };
    let game = sections.get("vtt").cloned().unwrap_or_default();
    let (moved, next) = submit_table(session, game, access, &envelope)?;
    sections.insert("vtt".into(), next);
    Ok(format!("Character {actor} Strides: {moved}; {measure}"))
}
