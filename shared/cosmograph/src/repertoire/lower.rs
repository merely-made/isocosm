// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Lowering one catalog part to a kind (see `repertoire`): geometry from its
//! shape, its cells to its process, its port from what it admits and unlocks.
//!
//! Lowered: a part that acts at the meal, which is a mouth (process intake,
//! contract or grip, with an effect at the eating site, admitting living or
//! dead matter) or an organ whose traits unlock a tolerance (dead aged or
//! toxic flesh, a decomposer's). Left, each with its reason: parts whose
//! flows name no matter an intake port can (a host's tissue, soil, light);
//! those on a proposed process; those that wait on a mechanic; and shapes a
//! box cannot show (joint, branch). Armour, spine and gland parts lower only
//! where the world sets the guard rule (`ecology::guard`), as an organ whose
//! port carries the strengths the catalog's traits give it ([`guard`]); and
//! with the rule a mouth's port carries its attack by kind, and a detox organ
//! its tolerance of toxin. Without it they are left, as a lowered kind would
//! be inert.
//!
//! Parts that run through the categories rule lower only in a world that sets
//! it, with their dead classes and grinding on the port ([`port`]); parts
//! that deposit or eat dung lower only where the world also sets the detritus
//! rule (`ecology::detritus`), a dung mouth being a dead-stock port limited to
//! the dung class.

use crate::{
    catalog::{Catalog, Part, Role, Site, Status, Kind, refs},
    process::{DeadClass, IntakePort, NisKind, ProcessDef, ProcessId, Registry, Strength, Tolerance, Trade},
    rules::Template,
    schema::Key,
};
use isometer_core::classify;
use std::collections::BTreeMap;

/// A lowered part: its kind, and where it is the root of a body, the same
/// with one cell of reproduce.
#[derive(Clone, Debug)]
pub struct Lowering {
    pub template: Template,
    pub root: Option<Template>,
}

/// The box a catalog shape reads as, and the shape a kind declares where a
/// box cannot show it (tube, shell); the primitive palette's sizes.
fn boxed(shape: &str) -> Result<([i32; 3], &'static str), String> {
    Ok(match shape {
        "part-shape:lump" => ([2, 2, 2], ""),
        "part-shape:rod" => ([4, 1, 1], ""),
        "part-shape:sheet" => ([4, 4, 1], ""),
        "part-shape:point" => ([1, 1, 1], ""),
        "part-shape:tube" => ([2, 1, 1], "part-shape:tube"),
        "part-shape:shell" => ([2, 2, 2], "part-shape:shell"),
        other => return Err(format!("a box cannot show {other}")),
    })
}

fn def(process: &str) -> Result<&'static ProcessDef, String> {
    Registry::native()
        .get(&ProcessId::new(refs::NATIVE_NAMESPACE, refs::process_name(process)))
        .ok_or_else(|| format!("{process} is not in the native registry"))
}

/// The port the part's flows give, none if it names no matter and no
/// tolerance a port can carry. Under the categories rule (`cats`) the
/// flows `live:tough` and `dead:wood` give a mouth for tough plant matter
/// that grinds it, and the dead flows (litter, wood, the flesh ages) limit a
/// dead-stock mouth to those classes (`ecology::categories`); without it
/// every dead flow is a mouth for all dead stock, as before.
fn port(part: &Part, c: &Catalog, cats: bool, guard: bool) -> Result<Option<IntakePort>, String> {
    if needs_mutualism(part) {
        return Ok(Some(exchange(part).supported_by(def(&part.process)?.reference())));
    }
    let name = refs::process_name(&part.process);
    if guard && let Some(organ) = guard_organ(part, c) {
        return Ok(Some(organ.supported_by(def(&part.process)?.reference())));
    }
    let mut matter = IntakePort::none();
    let mut eats = false;
    for flow in &part.admits {
        match flow.as_str() {
            "live:tough" if cats => matter = matter.with_live(NisKind::Producer),
            "dead:litter" if cats => matter = matter.with_deadstock().taking(DeadClass::Litter),
            "dead:wood" if cats => matter = matter.with_deadstock().taking(DeadClass::Wood),
            "dead:fresh" | "dead:aged" | "dead:toxic" if cats => {
                matter = matter.with_deadstock().taking(DeadClass::Carrion);
            },
            "dead:dung" if cats => matter = matter.with_deadstock().taking(DeadClass::Dung),
            "live:producer" => matter = matter.with_live(NisKind::Producer),
            "live:consumer" => matter = matter.with_live(NisKind::Consumer),
            "live:decomposer" => matter = matter.with_live(NisKind::Decomposer),
            f if f.starts_with("dead:") => matter = matter.with_deadstock(),
            _ => continue,
        }
        eats = true;
    }
    let mouth = eats && part.effects.contains_key(&Site::Eating) && matches!(name, "intake" | "contract" | "grip");
    // A mouth tolerates what it eats of the aged and the toxic; any organ
    // what its traits unlock.
    let unlocked = part.traits.iter().filter_map(|t| c.trait_by_key(t)).flat_map(|t| &t.unlocks);
    let mut tolerant = IntakePort::none();
    let flows = unlocked.chain(part.admits.iter().filter(|_| mouth));
    for flow in flows {
        tolerant = match flow.as_str() {
            "dead:aged" => tolerant.tolerating(Tolerance::AgedFlesh),
            "dead:toxic" => tolerant.tolerating(Tolerance::ToxicFlesh),
            "live:decomposer" => tolerant.tolerating(Tolerance::DecomposerFlesh),
            "live:tough" | "dead:wood" if cats => tolerant.tolerating(Tolerance::ToughPlant),
            _ => tolerant,
        };
    }
    if guard {
        let strengths = part.traits.iter().filter_map(|t| c.trait_by_key(t)).flat_map(|t| &t.strengths);
        if strengths.clone().any(|s| s.role == Role::Defend && s.kind == Kind::Toxin) {
            tolerant = tolerant.tolerating(Tolerance::Toxin);
        }
        // A mouth bites by the attacks its traits give it.
        if mouth {
            let attack = strengths.filter(|s| s.role == Role::Attack).fold(Strength::NONE, |a, s| match s.kind {
                Kind::Crush => a.max(Strength { crush: strength(s.amount), ..Strength::NONE }),
                Kind::Pierce => a.max(Strength { pierce: strength(s.amount), ..Strength::NONE }),
                _ => a,
            });
            tolerant = tolerant.with_strength(attack);
        }
    }
    let bits = tolerant.tolerance_bits();
    if !mouth && bits == 0 {
        return Ok(None);
    }
    let base = if mouth { matter } else { IntakePort::none() };
    Ok(Some(base.with_tolerance_bits(bits).with_strength(tolerant.strength()).supported_by(def(&part.process)?.reference())))
}

fn strength(amount: u32) -> u8 {
    u8::try_from(amount.min(10)).unwrap_or(10)
}

/// The contact strength a spine or a sting gets where its traits give none.
const SPINE: u8 = 3;

/// The port of a guard organ (the guard rule): an armour, a spined or a
/// stinging part, or a gland, read from the effects the part has and the
/// strengths its traits give: armour's crush and pierce defence, the
/// attack of a contact effect as the wound it deals, and a toxin attack as
/// the toxin of a gland. None where the part is none of these.
fn guard_organ(part: &Part, c: &Catalog) -> Option<IntakePort> {
    let has = |site: Site, effect: &str| part.effects.get(&site).is_some_and(|keys| keys.iter().any(|k| k == effect));
    let strengths: Vec<_> = part.traits.iter().filter_map(|t| c.trait_by_key(t)).flat_map(|t| &t.strengths).collect();
    let best = |role: Role, kinds: &[Kind]| {
        strengths.iter().filter(|s| s.role == role && kinds.contains(&s.kind)).map(|s| strength(s.amount)).max().unwrap_or(0)
    };
    let mut s = Strength::NONE;
    if has(Site::Eaten, "effect:armour-threshold") {
        s.crush = best(Role::Defend, &[Kind::Crush]);
        s.pierce = best(Role::Defend, &[Kind::Pierce]);
    }
    if has(Site::Contact, "effect:spine-prick") || has(Site::Contact, "effect:sting") {
        s.spine = best(Role::Attack, &[Kind::Pierce, Kind::Inject]).max(SPINE);
    }
    if has(Site::Eaten, "effect:toxin-bite") {
        s.toxin = best(Role::Attack, &[Kind::Toxin]).max(1);
    }
    (!s.is_none()).then(|| IntakePort::none().with_strength(s))
}

/// The port of an exchange organ (the mutualism rule): it gives what the
/// part emits and takes what it admits, of sugar and soil nutrient.
fn exchange(part: &Part) -> IntakePort {
    let goods = |flows: &[String]| flows.iter().any(|f| matches!(f.as_str(), "sugar" | "exudate" | "soil:nutrient" | "host:matter"));
    let port = IntakePort::none();
    let port = if goods(&part.emits) { port.trading(Trade::Gives) } else { port };
    if goods(&part.admits) { port.trading(Trade::Takes) } else { port }
}

/// Whether `part` runs only through the mutualism rule.
#[must_use]
pub fn needs_mutualism(part: &Part) -> bool {
    matches!(&part.status, Status::Expressible { via } if via.iter().any(|v| v == "rule:mutualism"))
}

/// Whether `part` runs only through the guard rule.
#[must_use]
pub fn needs_guard(part: &Part) -> bool {
    matches!(&part.status, Status::Expressible { via } if via.iter().any(|v| v == "rule:guard"))
}

/// Whether `part` runs only through the categories rule.
#[must_use]
pub fn needs_categories(part: &Part) -> bool {
    matches!(&part.status, Status::Expressible { via } if via.iter().any(|v| v == "rule:categories"))
}

/// Lowers `part` for a world without the categories rule.
///
/// # Errors
/// As [`lowering_with`].
pub fn lowering(part: &Part) -> Result<Lowering, String> {
    lowering_with(part, false)
}

/// Lowers `part` for a world with the categories rule if `cats` and,
/// likewise, the guard rule: `lowering_for(part, cats, false)` is
/// [`lowering_with`].
///
/// # Errors
/// As [`lowering_with`], and a part that runs only through the guard rule
/// where the world sets none.
pub fn lowering_for(part: &Part, cats: bool, guard: bool) -> Result<Lowering, String> {
    lowering_in(part, cats, guard, false)
}

/// As [`lowering_for`], for a world with the mutualism rule if `mutual`.
///
/// # Errors
/// As [`lowering_for`], and a part that runs only through the mutualism
/// rule where the world sets none.
pub fn lowering_in(part: &Part, cats: bool, guard: bool, mutual: bool) -> Result<Lowering, String> {
    lowering_all(part, cats, guard, mutual, false)
}

/// As [`lowering_in`], for a world with the detritus rule if `dung`.
///
/// # Errors
/// As [`lowering_in`], and a part that runs only through the detritus rule
/// where the world sets none.
pub fn lowering_all(part: &Part, cats: bool, guard: bool, mutual: bool, dung: bool) -> Result<Lowering, String> {
    if needs_detritus(part) && !dung {
        return Err("it needs the detritus rule".into());
    }
    if needs_guard(part) && !guard {
        return Err("it needs the guard rule".into());
    }
    if needs_mutualism(part) && !mutual {
        return Err("it needs the mutualism rule".into());
    }
    lowered(part, cats, guard)
}

/// Whether `part` runs only through the detritus rule.
#[must_use]
pub fn needs_detritus(part: &Part) -> bool {
    matches!(&part.status, Status::Expressible { via } if via.iter().any(|v| v == "rule:detritus"))
}

/// Lowers `part`; with `cats`, as a world with the categories rule has it.
///
/// # Errors
/// Names why it cannot: not runnable, a part that runs only through the
/// categories rule where the world sets none, a shape no box can show or the
/// process cannot be held on, or nothing a port can carry.
pub fn lowering_with(part: &Part, cats: bool) -> Result<Lowering, String> {
    lowering_for(part, cats, false)
}

fn lowered(part: &Part, cats: bool, guard: bool) -> Result<Lowering, String> {
    if needs_categories(part) && !cats {
        return Err("it needs the categories rule".into());
    }
    if !matches!(part.status, Status::Expressible { .. }) {
        return Err(format!("it is not runnable ({})", part.status.word()));
    }
    let (half_extent, shape) = boxed(&part.shape)?;
    let d = def(&part.process)?;
    if !d.admits(classify(half_extent)) || !refs::fits(&part.process, &part.shape) {
        return Err(format!("{} cannot hold {}", part.shape, part.process));
    }
    let Some(port) = port(part, crate::catalog::catalog(), cats, guard)? else {
        return Err("it names no matter or tolerance an intake port can carry".into());
    };
    let function = format!("function:{}", d.id.name);
    let capacity = crate::anatomy::capacity(half_extent);
    let make = |cells: BTreeMap<Key, u32>| Template { half_extent, cells, shape: shape.into(), port: Some(port) };
    let template = make(BTreeMap::from([(function.clone(), capacity)]));
    let root = (port.admits_deadstock() && capacity >= 2).then(|| {
        make(BTreeMap::from([(function, capacity - 1), (crate::anatomy::REPRODUCE.into(), 1)]))
    });
    Ok(Lowering { template, root })
}

/// The function the kind's port rests on.
pub(super) fn function_of(t: &Template) -> Option<Key> {
    crate::mosaic::ports::supporting(t.port?)
}

pub(super) fn is_mouth(t: &Template) -> bool {
    t.port.is_some_and(|p| p.has_live() || p.admits_deadstock())
}
