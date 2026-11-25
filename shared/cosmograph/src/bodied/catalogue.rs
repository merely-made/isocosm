// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Founders draw from the organs the world's rules can actually express.
//! Draw a native capability first: many catalogue names for the same active
//! port must not make that strategy more likely.
use super::*;
use crate::{Founding, ecology::Role, process::IntakePort, repertoire::Lowered};

mod growth;

type Pool<'a> = BTreeMap<IntakePort, Vec<&'a str>>;

pub(crate) fn maps(
    f: &Founding,
    lowered: &Lowered,
    kinds: &BTreeMap<Key, Template>,
) -> Result<BTreeMap<Key, BodyMap>> {
    let mut pools: BTreeMap<(&str, bool), Pool> = BTreeMap::new();
    let generative = f.repertoire.as_ref().is_some_and(|r| r.generative_ecology);
    let share_parts = generative && lowered.across_kingdoms;
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
        let kingdom = role.kingdom();
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
                let n = draw(domain);
                let entries = pool.values().nth((n % pool.len() as u64) as usize)?;
                Some(entries[((n / pool.len() as u64) % entries.len() as u64) as usize].to_string())
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
            let count = usize::from(generative) + (draw("founder-organ-count") % 3) as usize;
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
                run.socket = if port.trades(crate::process::Trade::Draws)
                    || port.trades(crate::process::Trade::Takes)
                {
                    Facing::Below
                } else {
                    Facing::Above
                };
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
            if generative {
                growth::compose(&mut map.grammar, seed, i, grown);
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
    /// Generated, editable defaults. Explicit entries in `body_maps` take
    /// precedence; the original seed is sufficient to recover the defaults.
    pub fn catalogue_body_maps(&self) -> Result<BTreeMap<Key, BodyMap>> {
        if !self.repertoire.as_ref().is_some_and(|r| r.founders) {
            return Ok(BTreeMap::new());
        }
        let mut kinds = BTreeMap::new();
        let lowered = crate::repertoire::declare(self, &mut kinds)?.ok_or("missing repertoire")?;
        let prepared = self.drawn_founding()?;
        maps(prepared.as_ref().map_or(self, |(f, _)| f), &lowered, &kinds)
    }
}
