// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Births spend the provision (rulings 447, 448, 518, 524, 530, 552 and
//! 554), as both runners make them. A brood develops its lineage's whole
//! recipe from a soma each child draws; a clutch lays eggs, each the
//! recipe's root alone, sharing the provision; and a bud is the provision
//! poured into a part grown at the reproducing part, severing into a body
//! of its own once it holds a provision's worth. A child's tissue is its
//! parent's provision moved, never spawned (TD6). How a child draws its
//! soma, and what else a birth writes (identities, relations, the flow
//! record), is each runner's own.

use super::*;
use crate::{
    anatomy,
    development::{self, Soma},
    growth,
    rules::expressing,
};

/// What a birth works from: the actor's lineage's development, and its
/// provision and tissue accounts.
pub(crate) struct Lineal {
    pub d: Development,
    pub provision: Key,
    pub tissue: Key,
}

fn lineal(p: &mut impl Parties, rules: &Rules) -> Result<Lineal> {
    let lineage = p.body(Binding::Actor)?.lineage.clone();
    let d = p.development(&lineage)?;
    let own = anatomy::own(rules, &lineage);
    let (Some(provision), Some(tissue)) = (own.provision, own.tissue) else {
        return Err("a lineage without a provision bears nothing".into());
    };
    Ok(Lineal {
        d,
        provision,
        tissue,
    })
}

/// A brood or a clutch: the actor's whole provision taken and shared among
/// one child or the lineage's clutch, the remainder to the first, none
/// empty. Returns what the children develop from and each one's share.
pub(crate) fn bear(
    p: &mut impl Parties,
    rules: &Rules,
    clutch: bool,
) -> Result<(Lineal, Vec<u64>)> {
    let l = lineal(p, rules)?;
    let amount = p.held(Binding::Actor, &l.provision)?;
    if amount == 0 {
        return Err("nothing provisioned".into());
    }
    p.take(Binding::Actor, &l.provision, amount)?;
    let eggs = if clutch { u64::from(l.d.clutch) } else { 1 };
    let (base, extra) = (amount / eggs, amount % eggs);
    let shares = (0..eggs)
        .map(|k| base + u64::from(k < extra))
        .filter(|s| *s > 0)
        .collect();
    Ok((l, shares))
}

/// A child of `parent` born at `tick`, holding nothing, with `parts` and
/// the segments its soma drew.
pub(crate) fn newborn(
    parent: &Entity,
    parts: BTreeMap<Id, Part>,
    soma: Vec<u8>,
    tick: Tick,
) -> Entity {
    let mut child = parent.clone();
    child.parts = parts;
    child.soma = soma;
    child.accounts = Ledger::new();
    child.born = tick;
    child.arrived = tick;
    child.visits.clear();
    child.skills = BTreeMap::new();
    child.provenance = Provenance::Born(child.lineage.clone());
    child
}

/// A brood's or an egg's child of `parent`: the parts `soma` develops, an
/// egg its recipe's root alone, given `share` of tissue by their room and
/// carrying `young`. Returns it and where its tissue went.
pub(crate) fn hatch(
    (parent, l): (&Entity, &Lineal),
    rules: &Rules,
    soma: Soma,
    (clutch, share): (bool, u64),
    (tick, young): (Tick, Option<&Key>),
) -> Result<(Entity, Vec<(Id, u64)>)> {
    let mut parts = development::develop(rules, &l.d, &soma)?;
    if clutch {
        parts.retain(|_, p| p.situs == Some([0, 0, 0]));
    }
    let seed = soma.seed;
    let mut child = newborn(parent, parts, soma.segments, tick);
    born(&mut child, rules, &l.d.recipe, seed);
    child.traits.extend(young.cloned());
    let given =
        anatomy::give(&mut child, rules, &l.tissue, share).ok_or("a child with no parts")??;
    Ok((child, given))
}

/// A bud's pour: what the conversion took of the provision, how much, and
/// the bud, by its id on the parent, where it severed.
pub(crate) struct Budding {
    pub lineal: Lineal,
    pub taken: Ledger,
    pub pour: u64,
    pub severed: Option<(Id, Part)>,
}

/// Pours the provision into the actor's bud marked `mark`, growing one at
/// its reproducing part where there is none, and severs it once it holds a
/// provision's worth (552): what the parent's reproduce cells hold, or the
/// bud's adult mass where that is less.
pub(crate) fn bud(p: &mut impl Parties, rules: &Rules, mark: &Key) -> Result<Budding> {
    let l = lineal(p, rules)?;
    let amount = p.held(Binding::Actor, &l.provision)?;
    if amount == 0 {
        return Err("nothing provisioned".into());
    }
    let actor = p.body(Binding::Actor)?;
    let marked = actor
        .parts
        .iter()
        .find(|(_, q)| !q.severed && q.traits.contains(mark));
    let bud = match marked.map(|(id, _)| *id) {
        Some(bud) => bud,
        None => {
            let host = expressing(actor, anatomy::REPRODUCE).ok_or("nothing reproduces")?;
            let root = &l.d.recipe.tagmata[0].segment;
            let kind = rules.kinds.get(root).ok_or("an unknown root kind")?;
            let offset = growth::seat(actor, &l.d.policy, host, kind.half_extent, None)
                .ok_or("no room to bud")?;
            let id = actor.parts.keys().next_back().map_or(0, |last| last + 1);
            let cells: BTreeMap<Key, u32> = kind
                .cells
                .iter()
                .filter(|(_, n)| **n > 0)
                .map(|(f, n)| (f.clone(), *n))
                .collect();
            let part = Part {
                parent: Some(host),
                traits: BTreeSet::from([mark.clone()]),
                shape: kind.shape.clone(),
                functions: cells.keys().cloned().collect(),
                half_extent: kind.half_extent,
                offset,
                cells,
                ..Default::default()
            };
            actor.parts.insert(id, part);
            id
        },
    };
    let worth: u64 = actor
        .parts
        .iter()
        .filter(|(id, q)| **id != bud && !q.severed)
        .map(|(_, q)| anatomy::bound(q, rules, &l.provision))
        .sum();
    let part = &actor.parts[&bud];
    let full = anatomy::ceiling(part, rules.body()).min(worth);
    let need = full.saturating_sub(value(&part.matter, &l.tissue));
    // Its parts full, the body's room for tissue is the bud's; what the bud
    // does not need stays provisioned.
    let pour = amount.min(need).min(anatomy::room(actor, rules, &l.tissue));
    let mut taken = Ledger::new();
    if pour > 0 {
        let from = std::slice::from_ref(&l.provision);
        let to = (from, &l.tissue);
        taken = convert(p, rules, Binding::Actor, to, pour, Conversion::Digestion)?;
    }
    let actor = p.body(Binding::Actor)?;
    let held = value(&actor.parts[&bud].matter, &l.tissue);
    let severed = match full > 0 && held >= full {
        true => Some((bud, actor.parts.remove(&bud).expect("found above"))),
        false => None,
    };
    Ok(Budding {
        lineal: l,
        taken,
        pour,
        severed,
    })
}

/// A severed bud as a body of its own, born at `tick`: its part the root,
/// its mark gone, and the segments its soma drew.
pub(crate) fn seedling(
    (parent, l): (&Entity, &Lineal),
    rules: &Rules,
    (mut part, mark): (Part, &Key),
    soma: Soma,
    tick: Tick,
) -> Entity {
    part.parent = None;
    part.offset = [0; 3];
    part.traits.remove(mark);
    part.situs = Some([0, 0, 0]);
    let mut child = newborn(parent, BTreeMap::from([(0, part)]), soma.segments, tick);
    born(&mut child, rules, &l.d.recipe, soma.seed);
    child
}

/// What a child carries beyond its recipe (568, 577 and 583): its parent's
/// systems, and its parent's varied cells where its soma develops their
/// parts, applied to those it has; then its own varied cell and its riff,
/// both by its soma's seed.
fn born(child: &mut Entity, rules: &Rules, recipe: &crate::rules::Recipe, seed: u64) {
    let soma = &child.soma;
    let within = |v: &Varied| {
        let [t, s, _] = v.situs;
        soma.get(usize::from(t)).is_some_and(|n| s < *n)
    };
    let varied: Vec<Varied> = child.varied.iter().filter(|v| within(v)).cloned().collect();
    child.varied = varied;
    crate::systems::inherit(&mut child.parts, &child.varied);
    crate::systems::vary(child, rules, recipe, seed);
    crate::systems::riff(child, recipe, seed);
}
