// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exchange between any two bodies whose living organs fit. A body's organs
//! give off and take up goods (`process::IntakePort`), named over the world's
//! table of goods (`repertoire::Lowered::goods`, from the catalogue's goods
//! sheet); two bodies at one place can trade when each gives something the
//! other takes. Nothing here names a good, a kingdom or a partner: a lineage
//! index identifies a tissue ledger and no more, so which lines trade is
//! whatever their bodies make possible, and it moves as bodies change.
//!
//! What the exchange knows is how a good is carried ([`Carried`]). One act
//! settles both legs at once, so neither side pays unless the other does:
//!
//! - a good carried in **tissue** leaves the giver's own tissue for the
//!   taker, who makes it its own in its next turn and pays its fee to the
//!   ground then;
//! - a good carried from the **ground** is drawn by the giver's organ from
//!   the place both stand on, in the mix the taker's kind needs, and the
//!   taker fixes it with the place's light, so only a body with living
//!   fixing tissue can use it.
//!
//! The act is the tissue-payer's: it fixes with what the ground-giver draws
//! and pays for it. A further carrier (a giver that carries the taker's
//! young, say) is a variant and its legs here; a further good is a row.

use super::{Mutualism, SOIL, SUN, matter};
use crate::{
    Founding, Result,
    ecology::nutrients,
    generate::process,
    process::Carried,
    rules::*,
    schema::{Entity, Ledger, Tick},
    simulation::Genesis,
};

/// Tissue in passage from its giver to its taker: no lineage's own until the
/// taker makes it so.
pub const IN_PASSAGE: &str = "world:exchange";

/// The goods `e`'s living organs give off and take up, as bits over the
/// world's table.
#[must_use]
pub fn goods(e: &Entity) -> [u16; 2] {
    let ports = e.living().filter_map(|(_, p)| crate::mosaic::ports::active(p));
    ports.fold([0, 0], |[gives, takes], port| [gives | port.emitted(), takes | port.admitted()])
}

/// The goods carried so that `from` gives off and `to` takes up, by key.
#[must_use]
pub fn supplies<'r>(rules: &'r Rules, from: &Entity, to: &Entity, carried: Carried) -> Vec<&'r str> {
    let Some(table) = rules.repertoire.as_ref().map(|r| &r.goods) else { return Vec::new() };
    let passing = goods(from)[0] & goods(to)[1];
    let named = table.iter().enumerate().filter(|(i, good)| good.carried == carried && passing & (1 << i) != 0);
    named.map(|(_, good)| good.key.as_str()).collect()
}

/// What `a` and `b` could exchange, as `[what a pays, what b draws for it]`
/// by key: empty unless both legs can settle.
#[must_use]
pub fn bargain<'r>(rules: &'r Rules, a: &Entity, b: &Entity) -> Option<[Vec<&'r str>; 2]> {
    let legs = [supplies(rules, a, b, Carried::Tissue), supplies(rules, b, a, Carried::Ground)];
    legs.iter().all(|leg| !leg.is_empty()).then_some(legs)
}

pub(super) fn install(f: &Founding, g: &mut Genesis, rule: Mutualism) -> Result<()> {
    let turns = 2 * f.lineages as usize;
    if g.rules.processes.len() + turns > g.rules.limits.processes {
        return Err("partner exchanges exceed the world's process limit".into());
    }
    g.rules.accounts.insert(
        IN_PASSAGE.into(),
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

/// The mix of the ground a fixer of `line` needs for `lots` of tissue, with
/// what the ground is then owed on each nutrient axis.
fn ground_mix(f: &Founding, line: u32, rule: &Mutualism, lots: u64) -> Result<(Ledger, Vec<Effect>)> {
    let mut mix = Ledger::new();
    let mut returns = Vec::new();
    if let Some(soil) = f.nutrients {
        for axis in 0..soil.axes {
            let n = soil.need(line, axis).checked_mul(lots).ok_or("exchange quantity overflow")?;
            if n > 0 {
                mix.insert(nutrients::axis(axis), n);
                if axis > 0 {
                    returns.push(Effect::Condition {
                        key: nutrients::owed(axis),
                        delta: i64::try_from(n).map_err(|_| "exchange quantity overflow")?,
                    });
                }
            }
        }
    } else {
        mix.insert(SOIL.into(), rule.nutrient);
    }
    Ok((mix, returns))
}

/// The tissue-payer's act: it pays `rule.sugar` of its tissue to a body
/// that takes it, and fixes what that body's organ draws from the ground.
fn exchange(f: &Founding, line: u32, rule: &Mutualism, every: Tick) -> Result<Process> {
    let tissue = matter(line);
    let lots = (rule.nutrient / 2).max(1);
    let (mix, returns) = ground_mix(f, line, rule, lots)?;
    let fixed = mix.values().try_fold(0u64, |sum, &n| sum.checked_add(n)).ok_or("exchange yield overflow")?;
    let mut exchange = process(&format!("mutualism:exchange-{line}"), Causation::Transition, vec![]);
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
        Query::Supplies { from: Binding::Actor, to: Binding::Target, carried: Carried::Tissue },
        Query::Supplies { from: Binding::Target, to: Binding::Actor, carried: Carried::Ground },
        Query::Account { who: Binding::Actor, key: tissue.clone(), at_least: rule.sugar },
        Query::Account { who: Binding::Place, key: SUN.into(), at_least: lots },
    ]);
    exchange.requires.push(crate::ecology::canopy_requirement(f, line, every).unwrap_or(Query::FixingCanopy));
    for (account, &n) in &mix {
        exchange.requires.push(Query::Account { who: Binding::Place, key: account.clone(), at_least: n });
        // The ground is debited, never the partner: a credit lent to the
        // partner within the act could not be read as pass-start stock.
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
            give: [(IN_PASSAGE.into(), rule.sugar)].into(),
            conversion: None,
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Target,
            account: IN_PASSAGE.into(),
            amount: rule.sugar.into(),
        },
        Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: SUN.into(), amount: lots.into() },
        Effect::Transform {
            who: Binding::Actor,
            take: mix,
            give: [(tissue.clone(), fixed)].into(),
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

/// The taker's turn: what it was paid becomes its own tissue, and it pays
/// the ground its fee.
fn assimilate(line: u32, rule: &Mutualism, every: Tick) -> Process {
    let tissue = matter(line);
    let mut consume = process(
        &format!("mutualism:assimilate-{line}"),
        Causation::Transition,
        vec![
            Effect::Transform {
                who: Binding::Actor,
                take: [(IN_PASSAGE.into(), rule.sugar)].into(),
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
        Query::Account { who: Binding::Actor, key: IN_PASSAGE.into(), at_least: rule.sugar },
    ]);
    if rule.fee > rule.sugar {
        consume.requires.push(Query::Account { who: Binding::Actor, key: tissue, at_least: rule.fee - rule.sugar });
    }
    consume.period = Some(every);
    consume.priority = 2;
    consume
}
