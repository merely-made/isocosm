// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bodies for worlds that ask for them (rulings 511, 512, 525, 531 and 547
//! to 550). Mesocosm's palette is the default kinds, every cell of a part
//! seeded to its role's one process; its recipe lottery draws flora as
//! producers and fauna as consumers. Authored fungal maps also develop bodies;
//! unmapped fungi and microbes await territories and surfaces; its eight authored bodies are presets; every recipe
//! reproduces in one cell of its root; and each lineage's life history is
//! drawn within what its body allows, all within the founding's bounds.

use crate::{
    Result, anatomy,
    development::{develop, lineage_soma},
    rules::*,
    schema::*,
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod catalogue;
mod feeding;
mod map;
mod proportions;
mod rooted;
mod tips;
pub use tips::tips;
pub use map::BodyMap;
pub(crate) use catalogue::growth::{BOUNDS as GROWTH_BOUNDS, OPERATORS as GROWTH_OPERATORS, Sampler as GrowthSampler};

/// The life-history traits a lineage may carry (449, 520, 521, 527, 535).
pub const BROOD: &str = "strategy:brood";
pub const EGG: &str = "strategy:egg";
pub const BUD: &str = "strategy:bud";
pub const SEMELPAROUS: &str = "life:semelparous";
pub const MILK: &str = "care:milk";
pub const MOUTHFULS: &str = "care:mouthfuls";
pub const PROVISION_FIRST: &str = "growth:provision-first";
pub const CAPITAL: &str = "breeding:capital";
pub const TRAITS: [&str; 10] = [
    BROOD,
    EGG,
    BUD,
    SEMELPAROUS,
    MILK,
    MOUTHFULS,
    PROVISION_FIRST,
    CAPITAL,
    crate::harm::FRAGMENT,
    crate::harm::HEAL,
];

/// What a founding draws bodies within (550), each an inclusive range.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bodies {
    /// Explicit named founding; absent in existing generated worlds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub founding: Option<Key>,
    pub variance: [u8; 2],
    /// A borne kind's absence odds, one in so many.
    pub absence: [u32; 2],
    pub clutch: [u32; 2],
    /// Whether flora and fauna take the authored bodies in turn rather
    /// than drawing (512).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub roster: bool,
    /// Draw proportions from the named sheet and the lineage's life history.
    /// Opt-in founding data: old worlds retain their exact recipe lottery.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub proportions: bool,
    /// Saved inclusive geometry bounds. Omission keeps historical chain sizes.
    #[serde(default, skip_serializing_if = "unset")]
    pub limb_scale: [u8; 2],
    /// Whether a drawn consumer keeps the historical producer-admitting
    /// grazing mouth (ruling 905, the default) rather than a limbed jaw
    /// admitting fauna (832). New bodied WorldTime Diets founding instead
    /// seeds the mouth's admission from its initial diet; the actual port
    /// remains authoritative thereafter. A saved founding that omits this
    /// field keeps `false`, its recorded meaning and its digest.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub grazing: bool,
    /// The agentless hazard of ruling 704, drawn per world (the 2026-10-18
    /// note to ruling 227): its odds a tick as a numerator over 64, and the
    /// cells a hit takes. Odds of nought never wound: the control. A saved
    /// founding that does not name them was founded without, so they read as
    /// nought and are not rewritten with them.
    #[serde(default, skip_serializing_if = "unset")]
    pub hazard: [u64; 2],
    #[serde(default, skip_serializing_if = "unset")]
    pub wound: [u32; 2],
    /// Whether every active form senses (1071, Bodies 15): animals by touch
    /// and a smell radius, plants by light, soil and crowding, territories by
    /// their gradient, the world by its site. A saved founding that does not
    /// name it was founded without, so it reads as `false`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub sensing: bool,
    /// Producer growth needs a living, exposed fixing plate. Omitted in
    /// historical templates, which keep their lineage-based growth.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub canopy_growth: bool,
    /// Exposed, biomass-backed fixing cells needed for full growth per scheduled act.
    #[serde(default, skip_serializing_if = "unset")]
    pub fixing_budget: u32,
    /// Inclusive bounds, in base cells, on the smell radius each animal
    /// lineage draws; a world may widen them.
    #[serde(default, skip_serializing_if = "unset")]
    pub smell: [u64; 2],
}

fn unset<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

impl Default for Bodies {
    /// Mesocosm's bounds where it has them, and a clutch of 1 to 4 (550); no
    /// hazard, which Mesocosm's own founding turns on.
    fn default() -> Self {
        Self {
            founding: None,
            variance: [1, 2],
            absence: [12, 12],
            clutch: [1, 4],
            roster: false,
            proportions: false,
            limb_scale: [0, 0],
            grazing: true,
            hazard: [0, 0],
            wound: [0, 0],
            sensing: true,
            canopy_growth: false,
            fixing_budget: 0,
            smell: crate::senses::SMELL,
        }
    }
}

impl Bodies {
    pub fn check(&self) -> Result<()> {
        let ordered = self.variance[0] <= self.variance[1]
            && self.absence[0] <= self.absence[1]
            && self.clutch[0] <= self.clutch[1]
            && self.hazard[0] <= self.hazard[1]
            && self.hazard[1] <= MAX_DRAW
            && self.wound[0] <= self.wound[1];
        if !ordered || self.absence[0] == 0 || self.clutch[0] == 0
            || (self.limb_scale != [0,0] && !(1..=2).contains(&self.limb_scale[0]))
            || (self.limb_scale != [0,0] && (self.limb_scale[1] < self.limb_scale[0] || self.limb_scale[1] > 2)) {
            return Err("body bounds outside the declared generator domain".into());
        }
        if self.hazard[1] > 0 && self.wound[0] == 0 {
            return Err("a hazard that hits for no cells".into());
        }
        if self.sensing && (self.smell[0] == 0 || self.smell[0] > self.smell[1]) {
            return Err("a smell radius outside its bounds".into());
        }
        Ok(())
    }

    /// The hazard a world draws, on the bodied lineages `among`, where its
    /// odds are not all nought.
    fn drawn_hazard(&self, seed: u64, among: BTreeSet<Key>) -> Option<Process> {
        if self.hazard[1] == 0 || among.is_empty() {
            return None;
        }
        let pick = |domain: &str, r: [u64; 2]| r[0] + crate::draw(seed, domain, &[0]) % (r[1] - r[0] + 1);
        let rate = pick("body-hazard", self.hazard) as i64;
        let cells = pick("body-wound-cells", self.wound.map(u64::from));
        Some(crate::harm::hazard::process(rate, cells, among))
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
        chain: Vec::new(),
        segments,
        segment: kind(segment),
        bears: None,
        per_segment: 0,
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
        symmetry: None,
        harden: None,
        pool: None,
        slot: None,
    }
}

/// A tagma whose segments bear `borne`, `per` of them, at Mesocosm's socket:
/// a flank pair for limbs, eyes and covering plates, above for a lit plate,
/// below for a mouth.
fn bearing(segments: u8, segment: &str, borne: &str, per: u8, socket: Facing) -> Tagma {
    Tagma {
        chain: Vec::new(),
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
    let span = if producer { ([1, 3], [4, 11]) } else { ([1, 4], [1, 6]) };
    drawn_within(seed, index, producer, b, span)
}

/// The same within `(stretches, segments)`, each an inclusive range: a
/// world-time plant's are narrower (1115).
pub(crate) fn drawn_within(seed: u64, index: u64, producer: bool, b: &Bodies, span: ([u64; 2], [u64; 2])) -> Recipe {
    let r = |domain: &str, k: u64| crate::draw(seed, domain, &[index, k]);
    let ([t0, t1], [s0, s1]) = span;
    let stretches = t0 + r("recipe-stretches", 0) % (t1 - t0 + 1);
    let fixing = r("recipe-fixing", 0) % stretches;
    let mut tagmata = vec![];
    let mut limbed = false;
    for s in 0..stretches {
        let segments = (s0 + r("recipe-segments", s) % (s1 - s0 + 1)) as u8;
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
        false => bearing(1, "block", if limbed && !b.grazing { "rod" } else { "block" }, 1, MOUTH),
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
        ("life-fragment", crate::harm::FRAGMENT),
        ("life-heal", crate::harm::HEAL),
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
    embody_within(g, b, None)
}

/// A world-time founding may supply the bounds for its producer-only bodies.
pub(crate) fn embody_within(
    g: &mut Genesis,
    b: &Bodies,
    plants: Option<crate::ecology::world_time::Plants>,
) -> Result<()> {
    let root_stores = !g.rules.root_stores.is_empty();
    let aligned_food = root_stores && g.founding.as_ref().is_some_and(|f| f.web == crate::ecology::Web::Diets);
    let bounded;
    let b = if root_stores { bounded = rooted::bounds(b); &bounded } else { b };
    // The accepted rooted founding turns sensing on. Recheck the effective
    // bounds so an older, nonsensing input with unset smell fails cleanly.
    b.check()?;
    if root_stores && let Some(founding) = &mut g.founding {
        founding.bodies = Some(b.clone());
    }
    let (mut kinds, authored) = if let Some(name) = &b.founding {
        let sheets = crate::datasheet::sheets();
        let palette = &sheets.founding(name)?.palette;
        (
            {
                let mut kinds = crate::datasheet::native::kinds(palette)?;
                // Native collision/volume frames require positive half-extents.
                // A sheet's zero-width blade becomes the smallest paid solid link,
                // only in this explicit new-world route. Historical kinds stay intact.
                for kind in kinds.values_mut() {
                    kind.half_extent = kind.half_extent.map(|h| h.max(1));
                }
                kinds
            },
            crate::datasheet::native::founding(name)?,
        )
    } else {
        (default_kinds(), roster())
    };
    let maps = g.founding.as_ref().map(|f| &f.body_maps);
    let mut generated=BTreeMap::new();
    if let Some(founding)=g.founding.as_ref() {
        let compose=founding.repertoire.as_ref().is_some_and(|rule|rule.founders);
        if compose || !founding.body_maps.is_empty() {
            g.rules.repertoire=crate::repertoire::declare(founding,&mut kinds)?;
        }
        if compose {
            catalogue::growth::inert(&mut kinds);
        }
        if compose && let Some(lowered)=&g.rules.repertoire {
            generated=catalogue::maps(founding,lowered,&kinds)?;
        }
    }
    let (mut producers, mut consumers) = (0, 0);
    let mut senses = crate::senses::Senses::default();
    let affinity = g.rules.affinity.clone().unwrap_or_default();
    affinity.validate()?;
    let mut added: BTreeMap<Key, BTreeSet<Key>> = BTreeMap::new();
    let mut bodied: BTreeSet<Key> = BTreeSet::new();
    // Kingdom/provenance facts do not change when recipes and life traits
    // are assigned below; keep them available during the mutable line walk.
    let source_lineages = g.lineages.clone();
    for (index, (key, lineage)) in g.lineages.iter_mut().enumerate() {
        let index = if g.founding.as_ref().is_some_and(|f|f.repertoire.as_ref().is_some_and(|r|r.kingdom_draw.is_some())) {
            key.strip_prefix("lineage:").and_then(|n|n.parse().ok()).unwrap_or(index as u64)
        } else { index as u64 };
        let map = maps.and_then(|maps| maps.get(key)).or_else(||generated.get(key));
        // Where kingdoms are drawn a line's living is drawn too, and its
        // body is founded for that living whatever kingdom it is of.
        let living = g.founding.as_ref().filter(|f| !f.forms().is_empty()).map(|f| f.role(index as u32));
        let producer = match (living, lineage.kingdom.as_str()) {
            (Some(role), "kingdom:flora" | "kingdom:fauna" | "kingdom:myco") if plants.is_none() && map.is_some() => {
                role == crate::ecology::Role::Producer
            },
            (_, "kingdom:flora") => true,
            (_, "kingdom:fauna") if plants.is_none() => false,
            (_, "kingdom:myco") if plants.is_none() && map.is_some() => false,
            _ => continue,
        };
        bodied.insert(key.clone());
        let animal = living.map_or(lineage.kingdom == "kingdom:fauna", crate::ecology::Role::consumer);
        if b.sensing && animal {
            // An animal's smell reaches a radius its lineage draws (1071).
            let [s0, s1] = b.smell;
            senses.smell.insert(key.clone(), s0 + crate::draw(g.seed, "smell", &[index]) % (s1 - s0 + 1));
        }
        let (traits, mut clutch, mut anamorphic) = life(g.seed, index, b);
        let varied = (b.proportions && (producer || animal)).then(||
            proportions::drawn(g.seed, index, producer, &traits, &authored)).flatten();
        let mut recipe = match varied {
            Some(recipe) => recipe,
            None => match (producer || animal) && (b.roster || b.founding.is_some()) {
            false => match &plants {
                Some(p) => drawn_within(g.seed, index, producer, b, p.span),
                None => drawn(g.seed, index, producer, b),
            },
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
            },
        };
        let mut grammar = None;
        if let Some(map) = map {
            if producer && (root_stores || plants.is_some()) {
                return Err(format!("{key}: the world-time producer requires its native root recipe"));
            }
            let mut source = map.resolve(&kinds, g.rules.repertoire.as_ref(), &lineage.kingdom)
                .map_err(|why| format!("{key}: {why}"))?;
            recipe = map.recipe(&mut source);
            grammar = Some(source);
        }
        if root_stores && producer {
            recipe = rooted::recipe(g.seed, index, &mut kinds)?;
            anamorphic = false;
            clutch = 1;
        }
        reproducing(&mut kinds, &mut recipe)?;
        if let Some(p) = &plants {
            p.storing(&mut kinds, &mut recipe)?;
        }
        if !producer && map.is_none() {
            proportions::limbs(g.seed, index, b.limb_scale, &mut kinds, &mut recipe)?;
        }
        if animal {
            let seed = if aligned_food {
                let i = key.trim_start_matches("lineage:").parse::<u32>().map_err(|_| "drawn food lineage has no index")?;
                Some(feeding::seed(crate::ecology::Role::of(crate::ecology::Web::Diets, i)))
            } else { None };
            let intake = feeding::declare(&mut kinds, &mut recipe, key, seed)?;
            let feed = format!("ecology:feed-{}", key.trim_start_matches("lineage:"));
            if let Some(mut p) = g.rules.processes.get(&feed).cloned() {
                feeding::configure(&mut p, intake, &g.rules, &source_lineages, aligned_food)?;
                g.rules.processes.insert(feed, p);
                if aligned_food && let Some(birth) = g.rules.processes.get_mut(&format!("ecology:birth-{}", key.trim_start_matches("lineage:"))) {
                    for q in &mut birth.requires {
                        if let Query::MembersPerHead { at_least, per_members, .. } = q {
                            *q = Query::FoodPerHead { feed: format!("ecology:feed-{}", key.trim_start_matches("lineage:")),
                                at_least: *at_least, per_members: *per_members };
                        }
                    }
                }
            }
        }
        if let Some(source) = &mut grammar {
            source.seed = std::mem::take(&mut recipe.tagmata);
            recipe.tagmata = unfold(source).map_err(|why| format!("{key}: {why}"))?;
            if let Some(why) = crate::validation::tagmata(&recipe.tagmata) {
                return Err(format!("{key}: body map unfolds to a recipe that {why}"));
            }
        }
        lineage.traits.extend(traits.iter().cloned());
        added.insert(key.clone(), traits);
        let domain = (crate::draw(g.seed, "domain", &[index]) % u64::from(affinity.domains)) as u16;
        if grammar.is_none() && g.rules.inherited_variation != 0 {
            let generated = crate::inherit::grammar(&recipe);
            recipe.tagmata = crate::rules::unfold(&generated)?;
            grammar = Some(generated);
        }
        lineage.development = Some(Development {
            lexicon: recipe.kinds(),
            recipe,
            policy: Policy::default(),
            domain,
            clutch,
            anamorphic,
            tracts: vec![],
            systems: Default::default(),
            comfort: 0,
            grammar,
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
    if let Some(maps) = maps {
        for key in maps.keys() {
            if !bodied.contains(key) || !g.population.groups.values().any(|g| g.entity.lineage == *key) {
                return Err(format!("{key}: body map needs a lineage with bodied founders"));
            }
        }
    }
    // A lineage without a body has no cells to wound. A world that wounds
    // also rests.
    if let Some(hazard) = b.drawn_hazard(g.seed, bodied) {
        g.rules.processes.insert(hazard.id.clone(), hazard);
        let rest = crate::harm::rest::process("world:soil");
        g.rules.processes.insert(rest.id.clone(), rest);
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
    if b.sensing {
        g.rules.senses = Some(senses);
    }
    let rules = g.rules.clone();
    let seed = g.seed;
    let bodies = crate::inherit::Cache::default();
    for (first, group) in g.population.groups.iter_mut() {
        let members=group.count;
        let e = group.entity_mut();
        let lineage = g.lineages.get(&e.lineage);
        let Some((line, d)) = lineage.and_then(|l| l.development.as_ref().map(|d| (l, d))) else {
            continue;
        };
        e.traits.extend(added[&e.lineage].iter().cloned());
        let drawn = lineage_soma(
            &rules,
            d,
            crate::draw(seed, "founder-soma", &[*first]),
        );
        if rules.inherited_variation == 0 {
            e.embody(develop(&rules, d, &drawn)?);
        } else {
            bodies.embody(&rules, (line.revision, d), &drawn, e)?;
        }
        let mapped = generated.contains_key(&e.lineage) || maps.is_some_and(|maps| maps.contains_key(&e.lineage));
        // Generic births need the same draw to regrow the parent's optional organs.
        e.absent = if mapped { drawn.absent } else { crate::development::recorded(&rules, &drawn) };
        e.soma = drawn.segments;
        let own: Vec<Key> = e
            .accounts
            .keys()
            .filter(|k| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { lineage, .. }) if *lineage == e.lineage))
            .cloned()
            .collect();
        for k in own {
            let held = e.accounts.remove(&k).unwrap_or(0);
            let room = anatomy::room(e, &rules, &k);
            if mapped && held > room {
                if generated.contains_key(&e.lineage) && !maps.is_some_and(|m|m.contains_key(&e.lineage))
                    && g.founding.as_ref().is_some_and(|f|f.repertoire.as_ref().is_some_and(|r|r.kingdom_draw.is_some())) {
                    // Found only what fits the phenotype. The unallocated
                    // starting material stays in this cohort's home soil.
                    let unused=(held-room).checked_mul(members).ok_or("founder supply overflow")?;
                    let soil=g.sites.get_mut(&e.place).ok_or("founder has no home site")?.accounts.entry("world:soil".into()).or_default();
                    *soil=soil.checked_add(unused).ok_or("founder soil overflow")?;
                } else {
                    return Err(format!("{}: body map founder {first} cannot hold its {held} of {k} (room {room})", e.lineage));
                }
            }
            let give = held.min(room);
            if give > 0 {
                if anatomy::root_store::allocation(e, &rules, &k).is_some() {
                    anatomy::root_store::initialize(e, &rules, &k, give).ok_or("no rooted allocation")??;
                } else {
                    anatomy::give(e, &rules, &k, give).ok_or("no parts to hold matter")??;
                }
            }
        }
    }
    Ok(())
}
