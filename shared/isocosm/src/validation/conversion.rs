// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Declared conversions (rulings 342 and 357), after Mesocosm's three. A
//! transform declaring one moves matter only, of some amount. World matter,
//! of a lineage of the world's kingdom (rulings 98 and 100), is synthesized
//! into one living lineage's matter; living matter is digested into one
//! living lineage's; and mineralization returns matter that includes living
//! matter to the world's. Synthesis and digestion yield the body's own
//! matter, which the act checks against the body it binds. Undeclared
//! transforms pass as before.

use super::matter;
use crate::{Result, rules::*, schema::*};
use std::collections::{BTreeMap, BTreeSet};

/// The kingdom whose lineages are the world's own matter.
const WORLD: &str = "kingdom:world";

/// What the rules alone can check: matter only, some of it, and a body to
/// synthesize or digest into.
pub(super) fn declared(rules: &Rules, id: &str, e: &Effect) -> Result<()> {
    if let Effect::Convert {
        who, conversion, ..
    } = e
    {
        let body = matches!(who, Binding::Actor | Binding::Target);
        if *conversion != Conversion::Mineralization && !body {
            return Err(format!(
                "{id} declares {conversion:?} for what is not a body"
            ));
        }
        return Ok(());
    }
    let Effect::Transform {
        who,
        take,
        give,
        conversion: Some(kind),
    } = e
    else {
        return Ok(());
    };
    for key in take.keys().chain(give.keys()) {
        matter(rules, key).map_err(|why| format!("{id} declares {kind:?}: {why}"))?;
    }
    if take.values().all(|v| *v == 0) {
        return Err(format!("{id} declares {kind:?} of nothing"));
    }
    let body = matches!(who, Binding::Actor | Binding::Target);
    if *kind != Conversion::Mineralization && !body {
        return Err(format!("{id} declares {kind:?} for what is not a body"));
    }
    Ok(())
}

/// What needs the lineages: which matter is the world's, and which living.
pub(crate) fn kinds(rules: &Rules, lineages: &BTreeMap<Key, Lineage>) -> Result<()> {
    // The lineage an account's matter is of, and whether it is the world's.
    let of = |key: &Key| match rules.accounts.get(key) {
        Some(AccountKind::Matter { lineage, .. }) => {
            let world = lineages.get(lineage).is_some_and(|l| l.kingdom == WORLD);
            Some((lineage, world))
        },
        _ => None,
    };
    for p in rules.processes.values() {
        let effects = p
            .commitments
            .iter()
            .chain(&p.effects)
            .chain(p.risk.iter().flat_map(|r| &r.effects));
        let effects = effects.flat_map(|e| std::iter::once(e).chain(e.branches()));
        for e in effects {
            let side = |l: &Ledger| -> Vec<(&Key, bool)> {
                let moved = l.iter().filter(|(_, v)| **v > 0);
                moved.filter_map(|(k, _)| of(k)).collect()
            };
            let keys =
                |ks: &'_ [Key]| -> Vec<(&Key, bool)> { ks.iter().filter_map(&of).collect() };
            let (kind, took, gave) = match e {
                Effect::Transform {
                    take,
                    give,
                    conversion: Some(kind),
                    ..
                } => (*kind, side(take), side(give)),
                Effect::Convert {
                    from,
                    to,
                    conversion,
                    ..
                } => (*conversion, keys(from), keys(std::slice::from_ref(to))),
                // Spending into another account returns living matter as
                // the world's (ruling 446).
                Effect::Spend {
                    from,
                    into: Some(into),
                    ..
                } => (
                    Conversion::Mineralization,
                    keys(from),
                    keys(std::slice::from_ref(into)),
                ),
                _ => continue,
            };
            let world = |s: &[(&Key, bool)]| s.iter().all(|(_, w)| *w);
            let living = |s: &[(&Key, bool)]| s.iter().all(|(_, w)| !*w);
            let one = gave.iter().map(|(l, _)| *l).collect::<BTreeSet<_>>().len() == 1;
            let fits = match kind {
                Conversion::Synthesis => world(&took) && living(&gave) && one,
                Conversion::Digestion => living(&took) && living(&gave) && one,
                Conversion::Mineralization => !world(&took) && world(&gave),
            };
            if !fits {
                return Err(format!(
                    "{} declares {kind:?} but moves the wrong kinds of matter",
                    p.id
                ));
            }
        }
    }
    Ok(())
}
