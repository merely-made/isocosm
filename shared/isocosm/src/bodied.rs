// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bodies for worlds that ask for them (rulings 511, 512, 525, 531 and 547
//! to 550). Mesocosm's palette is the default kinds, every cell of a part
//! seeded to its role's one process; its recipe lottery draws flora as
//! producers and fauna as consumers, myco and micro waiting for territories
//! and surfaces; its eight authored bodies are presets; every recipe
//! reproduces in one cell of its root; and each lineage's life history is
//! drawn within what its body allows, all within the founding's bounds.

use crate::{
    Result, anatomy,
    development::{develop, soma},
    rules::*,
    schema::*,
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The life-history traits a lineage may carry (449, 520, 521, 527, 535).
pub const BROOD: &str = "strategy:brood";
pub const EGG: &str = "strategy:egg";
pub const BUD: &str = "strategy:bud";
pub const SEMELPAROUS: &str = "life:semelparous";
pub const MILK: &str = "care:milk";
pub const MOUTHFULS: &str = "care:mouthfuls";
pub const PROVISION_FIRST: &str = "growth:provision-first";
pub const CAPITAL: &str = "breeding:capital";
pub const TRAITS: [&str; 8] = [
    BROOD,
    EGG,
    BUD,
    SEMELPAROUS,
    MILK,
    MOUTHFULS,
    PROVISION_FIRST,
    CAPITAL,
];

/// What a founding draws bodies within (550), each an inclusive range.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bodies {
    pub variance: [u8; 2],
    /// A borne kind's absence odds, one in so many.
    pub absence: [u32; 2],
    pub clutch: [u32; 2],
    /// Whether flora and fauna take the authored bodies in turn rather
    /// than drawing (512).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub roster: bool,
}

impl Default for Bodies {
    /// Mesocosm's bounds where it has them, and a clutch of 1 to 4 (550).
    fn default() -> Self {
        Self {
            variance: [1, 2],
            absence: [12, 12],
            clutch: [1, 4],
            roster: false,
        }
    }
}

impl Bodies {
    pub fn check(&self) -> Result<()> {
        let ordered = self.variance[0] <= self.variance[1]
            && self.absence[0] <= self.absence[1]
            && self.clutch[0] <= self.clutch[1];
        if !ordered || self.absence[0] == 0 || self.clutch[0] == 0 {
            return Err("body bounds outside the declared generator domain".into());
        }
        Ok(())
    }
}

/// Mesocosm's palette (511, 758): the default founding's, read from the
/// datasheets, each box with all its cells given to its bank's function.
pub fn default_kinds() -> BTreeMap<Key, Template> {
    let palette = &crate::datasheet::sheets()
        .founding(crate::datasheet::native::default_founding())
        .expect("the default founding")
        .palette;
    crate::datasheet::native::kinds(palette).unwrap_or_else(|why| panic!("{why}"))
}

fn kind(name: &str) -> Key {
    format!("kind:{name}")
}

fn bare(segments: u8, segment: &str) -> Tagma {
    Tagma {
        segments,
        segment: kind(segment),
        bears: None,
        per_segment: 0,
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
    }
}

/// A tagma whose segments bear `borne`, `per` of them, at Mesocosm's socket:
/// a flank pair for limbs, eyes and covering plates, above for a lit plate,
/// below for a mouth.
fn bearing(segments: u8, segment: &str, borne: &str, per: u8, socket: Facing) -> Tagma {
    Tagma {
        bears: Some(kind(borne)),
        per_segment: per,
        socket,
        ..bare(segments, segment)
    }
}

const FLANK: Facing = Facing::Right;
const LIT: Facing = Facing::Above;
const MOUTH: Facing = Facing::Below;

/// Mesocosm's recipe lottery: a producer's stretches of 4 to 11 segments,
/// one of them and any a third of draws pick bearing fronds, under a bare
/// head; a consumer's of 1 to 6, bearing limbs, or an eye on the first,
/// under a head bearing a mouth, a jaw where it has limbs.
pub fn drawn(seed: u64, index: u64, producer: bool, b: &Bodies) -> Recipe {
    let r = |domain: &str, k: u64| crate::draw(seed, domain, &[index, k]);
    let stretches = 1 + r("recipe-stretches", 0) % if producer { 3 } else { 4 };
    let fixing = r("recipe-fixing", 0) % stretches;
    let mut tagmata = vec![];
    let mut limbed = false;
    for s in 0..stretches {
        let segments = match producer {
            true => 4 + r("recipe-segments", s) % 8,
            false => 1 + r("recipe-segments", s) % 6,
        } as u8;
        let borne = match producer {
            true => (s == fixing || r("recipe-plate", s) % 3 == 0).then_some(("frond", LIT)),
            false => match r("recipe-appendage", s) % 6 {
                0 | 1 => {
                    limbed = true;
                    Some(("rod", FLANK))
                },
                2 if s == 0 => Some(("eye", FLANK)),
                _ => None,
            },
        };
        let per = 1 + u8::from(r("recipe-pair", s) % 8 == 0);
        tagmata.push(match borne {
            Some((k, socket)) => bearing(segments, "block", k, per, socket),
            None => bare(segments, "block"),
        });
    }
    let head = match producer {
        true => bare(1, "block"),
        false => bearing(1, "block", if limbed { "rod" } else { "block" }, 1, MOUTH),
    };
    tagmata.insert(0, head);
    let [v0, v1] = b.variance;
    let variance = v0 + (r("recipe-variance", 0) % (u64::from(v1 - v0) + 1)) as u8;
    let [a0, a1] = b.absence;
    let one_in = a0 + (r("recipe-absence", 0) % (u64::from(a1 - a0) + 1)) as u32;
    Recipe {
        tagmata,
        variance,
        absence: [1, one_in],
        riff: [0, 1],
        vary: [0, 1],
    }
}

/// Mesocosm's authored bodies (512, 758): the default founding's, read
/// from the datasheets, by name, whether each is a producer, and its recipe.
pub fn roster() -> Vec<(String, bool, Recipe)> {
    let founding = crate::datasheet::native::default_founding();
    crate::datasheet::native::founding(founding).unwrap_or_else(|why| panic!("{why}"))
}

/// Gives reproduce one cell of `recipe`'s root kind (547), as a kind of its
/// own, so other segments of that kind keep all theirs.
pub fn reproducing(kinds: &mut BTreeMap<Key, Template>, recipe: &mut Recipe) -> Result<()> {
    let root = recipe.tagmata[0].segment.clone();
    let name = format!("{root}-reproducing");
    if !kinds.contains_key(&name) {
        let mut t = kinds
            .get(&root)
            .ok_or(format!("an unknown root kind {root}"))?
            .clone();
        let fullest = t
            .cells
            .iter()
            .max_by_key(|(_, n)| **n)
            .map(|(f, _)| f.clone());
        if let Some(f) = fullest {
            let n = t.cells.get_mut(&f).expect("found above");
            *n -= 1;
            if *n == 0 {
                t.cells.remove(&f);
            }
        }
        t.cells.insert(anatomy::REPRODUCE.into(), 1);
        kinds.insert(name.clone(), t);
    }
    recipe.tagmata[0].segment = name;
    Ok(())
}

/// A lineage's life history drawn by the founding seed (548): its
/// strategies, a choice none can refuse a body that reproduces; each of its
/// other traits either way; and its clutch within the founding's bound.
pub fn life(seed: u64, index: u64, b: &Bodies) -> (BTreeSet<Key>, u32, bool) {
    let r = |domain: &str| crate::draw(seed, domain, &[index]);
    let mut traits = BTreeSet::new();
    let strategies = 1 + r("life-strategies") % 7;
    for (bit, key) in [(1, BROOD), (2, EGG), (4, BUD)] {
        if strategies & bit != 0 {
            traits.insert(key.into());
        }
    }
    for (domain, key) in [
        ("life-order", PROVISION_FIRST),
        ("life-capital", CAPITAL),
        ("life-semelparity", SEMELPAROUS),
    ] {
        if r(domain) % 2 == 1 {
            traits.insert(key.into());
        }
    }
    match r("life-care") % 3 {
        1 => traits.insert(MILK.into()),
        2 => traits.insert(MOUTHFULS.into()),
        _ => false,
    };
    let anamorphic = r("life-anamorphic") % 2 == 1;
    let [c0, c1] = b.clutch;
    let clutch = c0 + (r("life-clutch") % (u64::from(c1 - c0) + 1)) as u32;
    (traits, clutch, anamorphic)
}

/// Gives a generated world's flora and fauna their recipes, provisions and
/// life histories, and develops their founders, a cohort sharing a soma and
/// its own matter given to its parts as far as they hold it.
pub(crate) fn embody(g: &mut Genesis, b: &Bodies) -> Result<()> {
    let mut kinds = default_kinds();
    let authored = roster();
    let (mut producers, mut consumers) = (0, 0);
    let affinity = Affinity::default();
    let mut added: BTreeMap<Key, BTreeSet<Key>> = BTreeMap::new();
    for (index, (key, lineage)) in g.lineages.iter_mut().enumerate() {
        let index = index as u64;
        let producer = match lineage.kingdom.as_str() {
            "kingdom:flora" => true,
            "kingdom:fauna" => false,
            _ => continue,
        };
        let mut recipe = match b.roster {
            false => drawn(g.seed, index, producer, b),
            true => {
                let turn = if producer {
                    &mut producers
                } else {
                    &mut consumers
                };
                let of: Vec<_> = authored.iter().filter(|a| a.1 == producer).collect();
                let pick = of[*turn % of.len()].2.clone();
                *turn += 1;
                pick
            },
        };
        reproducing(&mut kinds, &mut recipe)?;
        let (traits, clutch, anamorphic) = life(g.seed, index, b);
        lineage.traits.extend(traits.iter().cloned());
        added.insert(key.clone(), traits);
        let domain = (crate::draw(g.seed, "domain", &[index]) % u64::from(affinity.domains)) as u16;
        lineage.development = Some(Development {
            lexicon: recipe.kinds(),
            recipe,
            policy: Policy::default(),
            domain,
            clutch,
            anamorphic,
        });
        let provision = AccountKind::Matter {
            lineage: key.clone(),
            reserve: false,
            provision: true,
        };
        let name = key.trim_start_matches("lineage:");
        g.rules
            .accounts
            .insert(format!("provision:{name}"), provision);
    }
    // Bodies need the catalogue their kinds name (466) and the shapes (276).
    if g.rules.functions.is_empty() {
        g.rules.functions = default_functions();
    }
    if g.rules.shapes.is_empty() {
        g.rules.shapes = default_shapes();
    }
    g.rules.kinds = kinds;
    g.rules.affinity = Some(affinity);
    g.rules.traits.extend(TRAITS.map(String::from));
    let rules = g.rules.clone();
    let seed = g.seed;
    for (first, group) in g.population.groups.iter_mut() {
        let e = &mut group.entity;
        let lineage = g.lineages.get(&e.lineage);
        let Some(d) = lineage.and_then(|l| l.development.clone()) else {
            continue;
        };
        e.traits.extend(added[&e.lineage].iter().cloned());
        let drawn = soma(
            &rules,
            &d.recipe,
            crate::draw(seed, "founder-soma", &[*first]),
        );
        e.embody(develop(&rules, &d, &drawn)?);
        e.soma = drawn.segments;
        let own: Vec<Key> = e
            .accounts
            .keys()
            .filter(|k| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { lineage, .. }) if *lineage == e.lineage))
            .cloned()
            .collect();
        for k in own {
            let held = e.accounts.remove(&k).unwrap_or(0);
            let give = held.min(anatomy::room(e, &rules, &k));
            if give > 0 {
                anatomy::give(e, &rules, &k, give).ok_or("no parts to hold matter")??;
            }
        }
    }
    Ok(())
}
