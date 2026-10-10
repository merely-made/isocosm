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
    geometry::{Body, Frame},
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

/// A child of `parent` born at `tick`, holding nothing, with `body` and
/// the segments its soma drew.
pub(crate) fn newborn(parent: &Entity, body: Body, soma: Vec<u8>, tick: Tick) -> Entity {
    let mut child = parent.clone();
    child.embody(body);
    child.soma = soma;
    child.accounts = Ledger::new();
    child.born = tick;
    child.arrived = tick;
    child.visits.clear();
    child.skills = BTreeMap::new();
    child.provenance = Provenance::Born(child.lineage.clone());
    child.authored = None;
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
) -> Result<(Entity, Vec<(PartId, u64)>)> {
    let mut body = development::develop(rules, &l.d, &soma)?;
    if clutch {
        // An egg is its recipe's root alone, which development puts first.
        let root = body
            .parts
            .remove(&PartId(0))
            .ok_or("a recipe with no root")?;
        let frame = Frame {
            situs: Some([0, 0, 0]),
            ..frame_of(&body.doc, PartId(0))
        };
        body = Body {
            doc: crate::geometry::document(&frame),
            parts: BTreeMap::from([(PartId(0), root)]),
        };
    }
    let seed = soma.seed;
    let mut child = newborn(parent, body, soma.segments, tick);
    born(&mut child, rules, &l.d, seed);
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
    pub severed: Option<(PartId, (Frame, Part))>,
}

/// Part `id`'s geometry in `doc`, as a frame.
fn frame_of(doc: &BodyDocument, id: PartId) -> Frame {
    let g = doc.part(id);
    Frame {
        parent: g.and_then(|g| g.attachment).map(|a| a.parent),
        half_extent: g.map_or([0; 3], |g| g.half_extent),
        offset: g.and_then(|g| g.attachment).map_or([0; 3], |a| a.offset),
        situs: g.and_then(|g| g.situs),
        shape: g.map_or(String::new(), |g| g.shape.clone()),
    }
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
    let marked = actor.living().find(|(_, q)| q.traits.contains(mark));
    let bud = match marked.map(|(id, _)| id) {
        Some(bud) => bud,
        None => {
            let host = expressing(actor, anatomy::REPRODUCE).ok_or("nothing reproduces")?;
            let root = &l.d.recipe.tagmata[0].segment;
            let kind = rules.kinds.get(root).ok_or("an unknown root kind")?;
            let offset = growth::seat(actor, &l.d.policy, host, kind.half_extent, None)
                .ok_or("no room to bud")?;
            let cells: BTreeMap<Key, u32> = kind
                .cells
                .iter()
                .filter(|(_, n)| **n > 0)
                .map(|(f, n)| (f.clone(), *n))
                .collect();
            let frame = Frame {
                parent: Some(host),
                half_extent: kind.half_extent,
                offset,
                situs: None,
                shape: kind.shape.clone(),
            };
            let part = Part {
                traits: BTreeSet::from([mark.clone()]),
                functions: cells.keys().cloned().collect(),
                cells,
                ..Default::default()
            };
            actor.add_part(&frame, part)?
        },
    };
    let worth: u64 = actor
        .living()
        .filter(|(id, _)| *id != bud)
        .map(|(id, q)| anatomy::bound(actor.extent(id), q, rules, &l.provision))
        .sum();
    let part = &actor.parts[&bud];
    let full = anatomy::ceiling(actor.extent(bud), rules.body()).min(worth);
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
        true => Some((bud, actor.take_part(bud).expect("found above"))),
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
    ((frame, mut part), mark): ((Frame, Part), &Key),
    soma: Soma,
    tick: Tick,
) -> Entity {
    part.traits.remove(mark);
    let root = Frame {
        parent: None,
        offset: [0; 3],
        situs: Some([0, 0, 0]),
        ..frame
    };
    let body = Body {
        doc: crate::geometry::document(&root),
        parts: BTreeMap::from([(PartId(0), part)]),
    };
    let mut child = newborn(parent, body, soma.segments, tick);
    born(&mut child, rules, &l.d, soma.seed);
    child
}

/// What a child carries beyond its recipe (568, 577 and 583): its parent's
/// systems, or those its line folded in at a boundary (765), and its
/// parent's varied cells where its soma develops their parts, applied to
/// those it has; then its own varied cell and its riff, both by its soma's
/// seed.
fn born(child: &mut Entity, rules: &Rules, d: &Development, seed: u64) {
    let recipe = &d.recipe;
    if !d.systems.is_empty() {
        child.systems = d.systems.clone();
    }
    let soma = &child.soma;
    let within = |v: &Varied| {
        let [t, s, _] = v.situs;
        soma.get(usize::from(t)).is_some_and(|n| s < *n)
    };
    let varied: Vec<Varied> = child.varied.iter().filter(|v| within(v)).cloned().collect();
    child.varied = varied;
    crate::systems::inherit(child);
    crate::systems::vary(child, rules, recipe, seed);
    crate::systems::riff(child, recipe, seed);
}
