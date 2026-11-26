// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Founders draw from the organs the world's rules can actually express.
//! Draw a native capability first: many catalogue names for the same active
//! port must not make that strategy more likely.
use super::*;
use crate::{Founding, ecology::Role, process::{Carried, IntakePort, NisKind, Trade}, repertoire::Lowered};

pub(super) mod growth;

/// The least light a founder that fixes for its living must catch when
/// grown, in the units of `trophic::canopy_light`: four cells' worth in the
/// open, a quarter of the full rate at the starting recipe's budget.
const LEAST_LIGHT: u64 = 4 * 64;

/// Whether a body grown from `tagmata` could make the living `role` asks of
/// it: read off the grown body, as the sim will read it. A founder that
/// fixes must hold fixing tissue its own parts leave in the light; one that
/// eats must hold its mouth under its head. What else it can do is free.
fn lives(tagmata: Vec<Tagma>, kinds: &BTreeMap<Key, Template>, role: Role) -> bool {
    let soma = crate::development::Soma { segments: tagmata.iter().map(|t| t.segments).collect(), absent: vec![], seed: 0 };
    let d = Development {
        lexicon: BTreeSet::new(),
        recipe: Recipe { tagmata, variance: 0, absence: [0, 1], riff: [0, 1], vary: [0, 1] },
        policy: Policy::default(),
        domain: 0,
        clutch: 1,
        anamorphic: false,
        tracts: vec![],
        systems: Default::default(),
        comfort: 0,
        grammar: None,
    };
    let Ok(body) = crate::development::develop_of(kinds, &d, &soma) else { return false };
    match role {
        Role::Producer => crate::trophic::canopy_light::exposure(&body) >= LEAST_LIGHT,
        Role::Decomposer => true,
        _ => crate::trophic::mouth_part(&body.doc).is_some(),
    }
}

/// Whether `grammar`, with `map`'s bindings, unfolds to a body the world
/// accepts in every stage its switches can bring about, and one that could
/// make the living `role` asks of it.
fn grows(map: &BodyMap, grammar: &Grammar, kinds: &BTreeMap<Key, Template>, lowered: &Lowered, kingdom: &str, role: Role) -> bool {
    let probe = BodyMap { grammar: grammar.clone(), ..map.clone() };
    let Ok(source) = probe.resolve(kinds, Some(lowered), kingdom) else { return false };
    let held = source.rules.iter().flat_map(|rule| match rule {
        Rule::Switch { rules, .. } => rules.iter().collect(),
        rule => vec![rule],
    });
    let laid = held.filter_map(|rule| match rule {
        Rule::Accrete { how: Accretion::Harden { kind, .. } | Accretion::Cover { kind, .. }, .. } => Some(kind),
        _ => None,
    });
    if laid.into_iter().any(|kind| !kinds.contains_key(kind)) {
        return false;
    }
    let switches: Vec<usize> = crate::rules::switches(&source).into_iter().map(|(i, _)| i).collect();
    let mut stages: Vec<Vec<usize>> = vec![vec![]];
    stages.extend(switches.iter().map(|i| vec![*i]));
    if switches.len() > 1 {
        stages.push(switches);
    }
    stages.iter().all(|on| {
        crate::rules::unfold_on(&source, on).is_ok_and(|body| {
            crate::validation::tagmata(&body).is_none()
                && body.iter().map(|run| usize::from(run.segments)).sum::<usize>() <= growth::MOST_SEGMENTS
                && lives(body, kinds, role)
        })
    })
}

type Pool<'a> = BTreeMap<IntakePort, Vec<&'a str>>;

pub(crate) fn maps(
    f: &Founding,
    lowered: &Lowered,
    kinds: &BTreeMap<Key, Template>,
) -> Result<BTreeMap<Key, BodyMap>> {
    let mut pools: BTreeMap<(&str, bool), Pool> = BTreeMap::new();
    let share_parts = lowered.across_kingdoms;
    // An organ that takes from the ground hangs below its run.
    let ground = lowered.goods.iter().enumerate().filter(|(_, good)| good.carried == Carried::Ground);
    let ground = ground.fold(0u16, |bits, (i, _)| bits | 1 << i);
    let rooted = |port: &IntakePort| {
        port.trades(Trade::Draws) || port.trades(Trade::Takes) || (port.emitted() | port.admitted()) & ground != 0
    };
    // Keep the native pool and the world's whole pool indexed by capability.
    // A draw can favor natal forms without making the kingdom a hard boundary.
    let mut common: BTreeMap<bool, Pool> = BTreeMap::new();
    for (kind, entry) in &lowered.kinds {
        if entry.root_of.is_some() {
            continue;
        }
        if entry.mouth
            && !matches!(
                entry.function.as_str(),
                "function:intake" | "function:contract"
            )
        {
            continue;
        }
        if let Some(port) = kinds[kind].port {
            if share_parts {
                common
                    .entry(entry.mouth)
                    .or_default()
                    .entry(port)
                    .or_default()
                    .push(&entry.part);
            }
            pools
                .entry((&entry.kingdom, entry.mouth))
                .or_default()
                .entry(port)
                .or_default()
                .push(&entry.part);
        }
    }
    let mut out = BTreeMap::new();
    let roster = f
        .repertoire
        .as_ref()
        .is_some_and(|r| r.kingdom_draw.is_some());
    let mut seen: BTreeMap<&str, Vec<BodyMap>> = BTreeMap::new();
    for i in 0..f.lineages {
        let role = f.role(i);
        let kingdom = f.kingdom_key(i);
        // A mouth that suits the living drawn, where the pool holds one.
        let suits = |port: &IntakePort| {
            let plants = port.admits_live(NisKind::Producer);
            let flesh = port.admits_live(NisKind::Consumer) || port.admits_live(NisKind::Decomposer);
            match role {
                Role::Grazer => plants && !flesh,
                Role::Predator => flesh && !plants,
                Role::Omnivore => plants && flesh,
                Role::Producer | Role::Decomposer => true,
            }
        };
        let mut accepted = false;
        for attempt in 0..256 {
            let seed = if attempt == 0 {
                f.seed
            } else {
                crate::draw(f.seed, "distinct-founder", &[i.into(), attempt])
            };
            let draw = |name: &str| crate::draw(seed, name, &[i.into()]);
            let pool_for = |mouth| {
                let natal = pools.get(&(kingdom, mouth));
                let world = common.get(&mouth);
                if share_parts
                    && draw(if mouth {
                        "mouth-repertoire"
                    } else {
                        "organ-repertoire"
                    }) % 2
                        == 0
                {
                    world.or(natal)
                } else {
                    natal.or(world)
                }
            };
            let pick = |mouth: bool, domain: &str| -> Option<Key> {
                let pool = pool_for(mouth)?;
                let mut fitting: Vec<_> = pool.iter().filter(|(port, _)| suits(port)).map(|(_, parts)| parts).collect();
                if fitting.is_empty() {
                    fitting = pool.values().collect();
                }
                let n = draw(domain);
                let entries = fitting.get((n % fitting.len().max(1) as u64) as usize)?;
                Some(entries[((n / fitting.len() as u64) % entries.len() as u64) as usize].to_string())
            };
            let mut map = BodyMap::single_run();
            let root = &mut map.grammar.seed[0];
            if role == Role::Producer {
                root.bears = Some("kind:frond".into());
                root.socket = Facing::Above;
                root.per_segment = 1;
            } else if role.consumer() {
                let mouth = pick(true, "founder-mouth").unwrap_or_else(|| "kind:block".into());
                map.regions.insert("region:mouth".into(), mouth);
                root.bears = Some("region:mouth".into());
                root.socket = Facing::Below;
                root.per_segment = 1;
            }
            // A producer keeps its fixing surfaces, a consumer its feeding and
            // locomotor tissue. Accessory organs draw distinct capabilities.
            let mut axis = super::bare(1, "block");
            axis.parent = Some(0);
            if role.consumer() {
                axis.bears = Some("kind:rod".into());
                axis.per_segment = 2;
                axis.socket = Facing::Right;
            }
            map.grammar.seed.push(axis);
            let mut organs: Vec<_> = pool_for(false)
                .into_iter()
                .flat_map(|pool| pool.iter())
                .collect();
            let count = 1 + (draw("founder-organ-count") % 3) as usize;
            let right = draw("founder-facing") % 2 == 0;
            for slot in 0..count.min(organs.len()) {
                let n = crate::draw(seed, "founder-organ", &[i.into(), slot as u64]);
                let (port, names) = organs.remove((n % organs.len() as u64) as usize);
                let name = names[(crate::draw(seed, "founder-organ-form", &[i.into(), slot as u64])
                    % names.len() as u64) as usize];
                let region = format!("region:organ-{}", slot + 1);
                let mut run = super::bare(1, "block");
                run.parent = Some(1);
                run.facing = if (slot == 0) == right {
                    Facing::Right
                } else {
                    Facing::Left
                };
                run.socket = if rooted(port) { Facing::Below } else { Facing::Above };
                run.bears = Some(region.clone());
                run.per_segment = 1;
                map.regions.insert(region, name.to_owned());
                map.grammar.seed.push(run);
            }
            let first = usize::from(role.consumer());
            let runs = map.grammar.seed.len() - first;
            let grown = first + (draw("founder-allocation") % runs as u64) as usize;
            map.grammar.rules.push(Rule::Repeat {
                tagma: grown as u8,
                by: (1 + draw("founder-length") % 3) as u8,
            });
            map.variance = (draw("founder-variance") % 2) as u8;
            map.absence = [1, (3 + draw("founder-absence") % 7) as u32];
            if roster {
                // Allocation alters the count of working tissue or locomotor parts.
                let axis = &mut map.grammar.seed[1];
                axis.segments = 1 + (draw("founder-axis-length") % 4) as u8;
                axis.facing = [Facing::Front, Facing::Right, Facing::Left, Facing::Above]
                    [(draw("founder-axis-facing") % 4) as usize];
            }
            if let Some(weights) = crate::catalog::catalog().kingdom(kingdom) {
                let mut grammar = std::mem::replace(&mut map.grammar, Grammar { seed: Vec::new(), rules: Vec::new() });
                let valid = |g: &Grammar| grows(&map, g, kinds, lowered, kingdom, role);
                let draw = |domain: &str, slot: u64| crate::draw(seed, domain, &[i.into(), slot]);
                growth::attach(&mut grammar, &draw, &valid);
                growth::Sampler { kingdom: weights, draw, valid, seasons: false }.compose(&mut grammar);
                map.grammar = grammar;
            }
            // A draw whose body could not make its living is drawn again.
            if !grows(&map, &map.grammar, kinds, lowered, kingdom, role) {
                continue;
            }
            if roster {
                let mut signature = map.clone();
                signature.variance = 0;
                signature.absence = [0, 1];
                for reference in signature.regions.values_mut() {
                    let kind = crate::repertoire::kind_key(reference);
                    if let Some(port) = kinds.get(&kind).and_then(|t| t.port) {
                        *reference = format!("{port:?}");
                    }
                }
                let previous = seen.entry(kingdom).or_default();
                if previous.contains(&signature) {
                    continue;
                }
                previous.push(signature);
            }
            out.insert(format!("lineage:{i}"), map);
            accepted = true;
            break;
        }
        if !accepted {
            return Err(format!(
                "{kingdom} exhausted distinct founder recipes in this repertoire"
            ));
        }
    }
    Ok(out)
}
impl Founding {
    /// Whether founders are composed from the world's catalogue organs, so
    /// that what a line does for a living is read from its body: any body
    /// with fixing tissue fixes, any with a fitting organ trades.
    #[must_use]
    pub fn composed(&self) -> bool {
        self.bodies.is_some() && self.repertoire.as_ref().is_some_and(|r| r.founders)
    }

    /// Generated, editable defaults. Explicit entries in `body_maps` take
    /// precedence; the original seed is sufficient to recover the defaults.
    pub fn catalogue_body_maps(&self) -> Result<BTreeMap<Key, BodyMap>> {
        if !self.repertoire.as_ref().is_some_and(|r| r.founders) {
            return Ok(BTreeMap::new());
        }
        // The kinds a founding of this recipe declares before it composes.
        let mut kinds = match self.bodies.as_ref().and_then(|b| b.founding.as_ref()) {
            Some(name) => crate::datasheet::native::kinds(&crate::datasheet::sheets().founding(name)?.palette)?,
            None => default_kinds(),
        };
        let lowered = crate::repertoire::declare(self, &mut kinds)?.ok_or("missing repertoire")?;
        growth::inert(&mut kinds);
        let prepared = self.drawn_founding()?;
        maps(prepared.as_ref().map_or(self, |(f, _)| f), &lowered, &kinds)
    }
}
