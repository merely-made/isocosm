// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A new founding migrates a recipe's port and its meal together. Saved
//! unmarked definitions and bare ecology never pass through this migration.

use crate::{
    Result,
    mosaic::ports::admission::Source,
    process::{IntakePort, NisKind, Process as Native, Registry},
    rules::*,
    schema::{Key, Lineage},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct FoundingPort {
    pub(super) function: Key,
    pub(super) port: IntakePort,
}

pub(super) fn declare(
    kinds: &mut BTreeMap<Key, Template>,
    recipe: &mut Recipe,
    lineage: &str,
    seed: Option<IntakePort>,
) -> Result<FoundingPort> {
    let head = recipe
        .tagmata
        .first_mut()
        .ok_or("a feeding recipe has no root")?;
    let (original, mouth) = match &head.bears {
        Some(kind) if head.socket == Facing::Below => (kind.clone(), true),
        _ => (head.segment.clone(), false),
    };
    let mut kind = kinds
        .get(&original)
        .ok_or("feeding kind is absent")?
        .clone();
    let native = if !mouth || kind.cells.get("function:intake").is_some_and(|n| *n > 0) {
        Native::Intake
    } else if kind.cells.get("function:contract").is_some_and(|n| *n > 0) {
        Native::Contract
    } else {
        return Err("mouth has no supported native feeding function".into());
    };
    let port = seed
        .unwrap_or_else(|| match (mouth, native) {
            (false, _) => IntakePort::deadstock(),
            (true, Native::Intake) => IntakePort::live(NisKind::Producer),
            (true, _) => IntakePort::live(NisKind::Consumer),
        })
        .supported_by(Registry::native().of_native(native).reference());
    // An explicit kind declaration precedes geometry's founding fallback.
    let port = *kind.port.get_or_insert(port);
    let function = match port.support() {
        Some(_) => crate::mosaic::ports::supporting(port).ok_or("unknown port support")?,
        None => format!("function:{}", Registry::native().of_native(native).id.name),
    };
    // Distinct mouth kinds do not turn every limb or mass segment into an
    // intake organ. Geometry and the exact cell allocation are unchanged.
    let key = format!("{original}-port-{}", lineage.trim_start_matches("lineage:"));
    kinds.insert(key.clone(), kind);
    if mouth {
        head.bears = Some(key);
    } else {
        head.segment = key;
    }
    Ok(FoundingPort { function, port })
}

/// Pair a newly founded ordinary meal with its organ's declared foods (832).
/// The selector names matter provenance; the body's lineage is no substitute
/// for the sources of mixed holdings or incorporated parts.
pub(super) fn configure(
    p: &mut Process,
    intake: FoundingPort,
    rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>,
    aligned: bool,
) -> Result<()> {
    let alive = match (intake.port.has_live(), intake.port.admits_deadstock()) {
        (true, true) => None,
        (false, true) => Some(false),
        _ => Some(true),
    };
    let mut sources = BTreeSet::new();
    for (account, kind) in &rules.accounts {
        if !matches!(kind, AccountKind::Matter { .. }) {
            continue;
        }
        for living in [true, false] {
            if alive.is_some_and(|wanted| wanted != living) {
                continue;
            }
            let source = Source::of(account, living, rules, lineages)?;
            if source.admitted_by(intake.port).is_ok() {
                sources.insert(source.lineage().to_owned());
            }
        }
    }
    let target = p
        .target
        .as_mut()
        .ok_or("ordinary feeding has no target selector")?;
    target.alive = alive;
    target.lineage = None;
    target.among.clear();
    // Even an empty admitted set means none. Existing prospective/runtime
    // port checks enforce this gate before drawing or accepting a meal.
    p.port_feeding = Some(PortFeeding {
        function: intake.function,
        sources: FeedingSources::Only(sources),
        meal: None,
    });
    if aligned {
        let meals: Vec<_> = p
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Eat {
                    amount: Amount::Fixed(n),
                    into,
                    ..
                } => Some((*n, into.clone())),
                _ => None,
            })
            .collect();
        let [(kept, retained), (lost, waste)] = meals.as_slice() else {
            return Err("world-time meal is not a declared two-part ratio".into());
        };
        let bite = kept.checked_add(*lost).ok_or("meal ratio overflow")?;
        let common = crate::rules::gcd(bite, *kept);
        let f = p.port_feeding.as_mut().unwrap();
        f.sources = FeedingSources::All;
        f.meal = Some(MealPartition {
            units: bite / common,
            kept: kept / common,
            ingesta: crate::ecology::world_time::INGESTA.into(),
            waste: waste.clone(),
        });
        p.target.as_mut().unwrap().alive = None;
        p.effects = vec![Effect::Eat {
            from: Binding::Target,
            amount: bite.into(),
            into: retained.clone(),
            of: vec![],
            whole: false,
        }];
        // Packet size no longer rejects a smaller, real admitted offering.
        p.requires.retain(|q| {
            !matches!(
                q,
                Query::Holds {
                    who: Binding::Target,
                    ..
                }
            )
        });
        p.requires.push(Query::Holds {
            who: Binding::Target,
            at_least: 1,
        });
    }
    Ok(())
}

pub(super) fn seed(role: crate::ecology::Role) -> IntakePort {
    use crate::ecology::Role;
    match role {
        Role::Predator => IntakePort::live(NisKind::Consumer),
        Role::Omnivore => IntakePort::live(NisKind::Producer).with_live(NisKind::Consumer),
        _ => IntakePort::live(NisKind::Producer),
    }
}
