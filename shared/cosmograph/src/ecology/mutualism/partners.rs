// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Local nutrient-for-sugar exchange. A living fixer that exports sugar can
//! transact with any other resident able to supply nutrients and accept sugar.
//! All legs settle in the same act. The recipient consumes the sugar in its
//! own turn. Lineage indexes here identify tissue ledgers, never a partner.

use super::{Mutualism, SOIL, SUN, matter};
use crate::{
    Founding, Result,
    ecology::nutrients,
    generate::process,
    process::Trade,
    rules::*,
    schema::{Ledger, Tick},
    simulation::Genesis,
};

const SUGAR: &str = "world:exchange-sugar";

pub(super) fn install(f: &Founding, g: &mut Genesis, rule: Mutualism) -> Result<()> {
    let turns = 2 * f.lineages as usize;
    if g.rules.processes.len() + turns > g.rules.limits.processes {
        return Err("partner exchanges exceed the world's process limit".into());
    }
    g.rules.accounts.insert(
        SUGAR.into(),
        AccountKind::Matter { lineage: "world:ground".into(), reserve: false, provision: false },
    );
    let every = minutes_ticks(rule.period, g.rules.tick_microseconds());
    for line in 0..f.lineages {
        for p in [exchange(f, line, &rule, every)?, assimilate(line, &rule, every)] {
            g.rules.processes.insert(p.id.clone(), p);
        }
    }
    g.rules.mutualism = Some(rule);
    Ok(())
}

fn lineage_actor(line: u32) -> Query {
    Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{line}") }
}

/// Draw the local mineral mix a fixer needs, retaining what the ground must
/// eventually recover on each nutrient axis.
fn mineral_mix(
    f: &Founding,
    line: u32,
    rule: &Mutualism,
    lots: u64,
) -> Result<(Ledger, Vec<Effect>)> {
    let mut nutrients = Ledger::new();
    let mut returns = Vec::new();
    if let Some(soil) = f.nutrients {
        for axis in 0..soil.axes {
            let n = soil.need(line, axis).checked_mul(lots).ok_or("exchange quantity overflow")?;
            if n > 0 {
                nutrients.insert(nutrients::axis(axis), n);
                if axis > 0 {
                    returns.push(Effect::Condition {
                        key: nutrients::owed(axis),
                        delta: i64::try_from(n).map_err(|_| "exchange quantity overflow")?,
                    });
                }
            }
        }
    } else {
        nutrients.insert(SOIL.into(), rule.nutrient);
    }
    Ok((nutrients, returns))
}

fn exchange(f: &Founding, line: u32, rule: &Mutualism, every: Tick) -> Result<Process> {
    let tissue = matter(line);
    let lots = (rule.nutrient / 2).max(1);
    let (nutrients, returns) = mineral_mix(f, line, rule, lots)?;
    let yield_matter = nutrients
        .values()
        .try_fold(0u64, |sum, &n| sum.checked_add(n))
        .ok_or("exchange yield overflow")?;
    let mut exchange =
        process(&format!("mutualism:exchange-{line}"), Causation::Transition, vec![]);
    exchange.target = Some(Target {
        same_place: true,
        alive: Some(true),
        lineage: None,
        among: Default::default(),
        weighted: true,
    });
    exchange.period = Some(every);
    exchange.priority = 1;
    exchange.requires.extend([
        lineage_actor(line),
        Query::Trades { who: Binding::Actor, what: Trade::GivesSugar },
        Query::Trades { who: Binding::Target, what: Trade::GivesNutrient },
        Query::Trades { who: Binding::Target, what: Trade::TakesSugar },
        Query::Account { who: Binding::Actor, key: tissue.clone(), at_least: rule.sugar },
        Query::Account { who: Binding::Place, key: SUN.into(), at_least: lots },
    ]);
    exchange
        .requires
        .push(crate::ecology::canopy_requirement(f, line, every).unwrap_or(Query::FixingCanopy));
    for (account, &n) in &nutrients {
        exchange.requires.push(Query::Account {
            who: Binding::Place,
            key: account.clone(),
            at_least: n,
        });
        // The partner extracts from the patch into the fixer's intake.
        // Only the actual substrate is debited as shared ground. A
        // temporary credit on the partner cannot be used as pass-start stock.
        exchange.effects.push(Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: account.clone(),
            amount: n.into(),
        });
    }
    exchange.effects.extend([
        Effect::Transform {
            who: Binding::Actor,
            take: [(tissue.clone(), rule.sugar)].into(),
            give: [(SUGAR.into(), rule.sugar)].into(),
            conversion: None,
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Target,
            account: SUGAR.into(),
            amount: rule.sugar.into(),
        },
        Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: SUN.into(),
            amount: lots.into(),
        },
        Effect::Transform {
            who: Binding::Actor,
            take: nutrients,
            give: [(tissue.clone(), yield_matter)].into(),
            conversion: None,
        },
        Effect::Transform {
            who: Binding::Actor,
            take: [(SUN.into(), lots)].into(),
            give: Ledger::new(),
            conversion: None,
        },
    ]);
    exchange.effects.extend(returns);

    Ok(exchange)
}

fn assimilate(line: u32, rule: &Mutualism, every: Tick) -> Process {
    let tissue = matter(line);
    let mut consume = process(
        &format!("mutualism:assimilate-{line}"),
        Causation::Transition,
        vec![
            Effect::Transform {
                who: Binding::Actor,
                take: [(SUGAR.into(), rule.sugar)].into(),
                give: [(tissue.clone(), rule.sugar)].into(),
                conversion: None,
            },
            Effect::Spend {
                from: vec![tissue.clone()],
                to: Binding::Place,
                amount: rule.fee.into(),
                into: Some(SOIL.into()),
            },
        ],
    );
    consume.requires.extend([
        lineage_actor(line),
        Query::Trades { who: Binding::Actor, what: Trade::TakesSugar },
        Query::Account { who: Binding::Actor, key: SUGAR.into(), at_least: rule.sugar },
    ]);
    if rule.fee > rule.sugar {
        consume.requires.push(Query::Account {
            who: Binding::Actor,
            key: tissue,
            at_least: rule.fee - rule.sugar,
        });
    }
    consume.period = Some(every);
    consume.priority = 2;
    consume
}
