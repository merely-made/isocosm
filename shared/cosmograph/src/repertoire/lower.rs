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
//! the dung class. Parts that take the nutrient the ground is scarcest in lower
//! only where the world sets the nutrient rule (`ecology::nutrients`), as an
//! organ whose port draws. A resprouting store can refill a grazed fixing
//! organ with matching tissue already held in that store. Parts that sense
//! lower only where bodies sense, as tissue behind a port that carries
//! nothing ([`tissue`]): the world reads their cells off the body.

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
    let base = feeding_port(part, c, cats, guard)?;
    if refs::process_name(&part.process) != "fix" || !part.admits.iter().any(|f| f == "light") {
        return Ok(base);
    }
    let mut responses = part.traits.iter().filter_map(|k| c.trait_by_key(k)?.light_response);
    let Some(leaf) = responses.next() else { return Ok(base) };
    if responses.any(|other| other != leaf) {
        return Err("the leaf traits prescribe different light responses".into());
    }
    let port = base.unwrap_or_else(IntakePort::none).with_light_response(leaf);
    Ok(Some(port.supported_by(def(&part.process)?.reference())))
}

fn feeding_port(part: &Part, c: &Catalog, cats: bool, guard: bool) -> Result<Option<IntakePort>, String> {
    if refs::process_name(&part.process) == "store"
        && part.traits.iter().any(|t| t == "trait:resprouting")
        && part.effects.get(&Site::Eaten)
            .is_some_and(|effects| effects.iter().any(|e| e == "effect:regrow-after-bite"))
    {
        let store = IntakePort::none().with_resprouting().supported_by(def(&part.process)?.reference());
        return Ok(Some(store));
    }
    if needs_mutualism(part) {
        let organ = exchange(part);
        let organ = if needs_nutrients(part) { organ.trading(Trade::Draws) } else { organ };
        return Ok(Some(organ.supported_by(def(&part.process)?.reference())));
    }
    if needs_nutrients(part) {
        return Ok(Some(IntakePort::none().trading(Trade::Draws).supported_by(def(&part.process)?.reference())));
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

/// `port`, giving off and taking up each good of `table` the part emits and
/// admits. The table decides what is a good; the part only names flows.
pub(super) fn trading(mut port: IntakePort, part: &Part, table: &[crate::catalog::Good]) -> IntakePort {
    for (index, good) in table.iter().enumerate() {
        if part.emits.contains(&good.key) {
            port = port.emitting(index);
        }
        if part.admits.contains(&good.key) {
            port = port.admitting(index);
        }
    }
    port
}

/// Whether `part` is runnable and names `rule` among those it runs through.
#[must_use]
pub fn runs_through(part: &Part, rule: &str) -> bool {
    matches!(&part.status, Status::Expressible { via } if via.iter().any(|v| v == rule))
}

/// Whether `part` runs only through the mutualism rule.
#[must_use]
pub fn needs_mutualism(part: &Part) -> bool {
    runs_through(part, "rule:mutualism")
}

/// Whether `part` runs only through the guard rule.
#[must_use]
pub fn needs_guard(part: &Part) -> bool {
    runs_through(part, "rule:guard")
}

/// Whether `part` runs only through the categories rule.
#[must_use]
pub fn needs_categories(part: &Part) -> bool {
    runs_through(part, "rule:categories")
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
    lowering_full(part, cats, guard, mutual, (dung, false))
}

/// As [`lowering_all`], for a world with the nutrient rule if `rules.1`; the
/// detritus rule is `rules.0`.
///
/// # Errors
/// As [`lowering_all`], and a part that runs only through the nutrient rule
/// where the world sets none.
pub fn lowering_full(part: &Part, cats: bool, guard: bool, mutual: bool, rules: (bool, bool)) -> Result<Lowering, String> {
    let (dung, soil) = rules;
    if needs_detritus(part) && !dung {
        return Err("it needs the detritus rule".into());
    }
    if needs_guard(part) && !guard {
        return Err("it needs the guard rule".into());
    }
    if needs_mutualism(part) && !mutual {
        return Err("it needs the mutualism rule".into());
    }
    if needs_nutrients(part) && !soil {
        return Err("it needs the nutrients rule".into());
    }
    lowered(part, cats, guard)
}

/// Whether `part` runs only through the nutrient rule.
#[must_use]
pub fn needs_nutrients(part: &Part) -> bool {
    runs_through(part, "rule:nutrients")
}

/// Whether `part` runs only through the detritus rule.
#[must_use]
pub fn needs_detritus(part: &Part) -> bool {
    runs_through(part, "rule:detritus")
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

/// Why a part that names no matter is left.
const NO_PORT: &str = "it names no matter or tolerance an intake port can carry";

/// Lowers `part` as tissue that works where it is: every cell given to its
/// process, and a port that carries nothing and rests on that process. For a
/// part [`lowering_full`] leaves only because it names no matter, in a world
/// that sets a rule reading such tissue off the body (`rule:senses`).
///
/// # Errors
/// `why` where the part was left for another reason, and as [`lowering_with`].
pub fn tissue(part: &Part, why: String) -> Result<Lowering, String> {
    if why != NO_PORT {
        return Err(why);
    }
    let (half_extent, shape, d) = held(part)?;
    Ok(built(half_extent, shape, d, IntakePort::none().supported_by(d.reference())))
}

/// The box `part` lowers to and the process its cells hold.
fn held(part: &Part) -> Result<([i32; 3], &'static str, &'static ProcessDef), String> {
    if !matches!(part.status, Status::Expressible { .. }) {
        return Err(format!("it is not runnable ({})", part.status.word()));
    }
    let (half_extent, shape) = boxed(&part.shape)?;
    let d = def(&part.process)?;
    if !d.admits(classify(half_extent)) || !refs::fits(&part.process, &part.shape) {
        return Err(format!("{} cannot hold {}", part.shape, part.process));
    }
    Ok((half_extent, shape, d))
}

fn lowered(part: &Part, cats: bool, guard: bool) -> Result<Lowering, String> {
    if needs_categories(part) && !cats {
        return Err("it needs the categories rule".into());
    }
    let (half_extent, shape, d) = held(part)?;
    let Some(port) = port(part, crate::catalog::catalog(), cats, guard)? else {
        return Err(NO_PORT.into());
    };
    Ok(built(half_extent, shape, d, port))
}

fn built(half_extent: [i32; 3], shape: &str, d: &ProcessDef, port: IntakePort) -> Lowering {
    let function = format!("function:{}", d.id.name);
    let capacity = crate::anatomy::capacity(half_extent);
    let make = |cells: BTreeMap<Key, u32>| Template { half_extent, cells, shape: shape.into(), port: Some(port) };
    let template = make(BTreeMap::from([(function.clone(), capacity)]));
    let root = (port.admits_deadstock() && capacity >= 2).then(|| {
        make(BTreeMap::from([(function, capacity - 1), (crate::anatomy::REPRODUCE.into(), 1)]))
    });
    Lowering { template, root }
}

/// The function the kind's port rests on.
pub(super) fn function_of(t: &Template) -> Option<Key> {
    crate::mosaic::ports::supporting(t.port?)
}

pub(super) fn is_mouth(t: &Template) -> bool {
    t.port.is_some_and(|p| p.has_live() || p.admits_deadstock())
}
