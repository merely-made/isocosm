// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A deposit (lineage-rounds D42, D47, D48): matter a body lays down at its
//! place, a kind of it an account of its own that the place holds. The body
//! pays it from its reserve at the rule's rate; the place lets it decay to
//! the soil at the rule's rate. The grammar's `Rule::Deposit` names the
//! account and the two rates; this lowers one to the two processes that move
//! the matter, as the rot and the detritus rules lay theirs, so the budget
//! stays closed and the flow record has its legs.

use crate::{Result, generate::process, rules::*, simulation::Genesis};
use std::collections::BTreeMap;

/// The soil account decay returns matter to.
const SOIL: &str = "world:soil";

/// After the meals (priority 10 at most), as detritus's turns are.
const AFTER_MEALS: i32 = 11;

/// What lays a deposit down: the trait that marks its bodies and the reserve
/// account they pay from.
pub(crate) struct Payer<'a> {
    pub marked: &'a str,
    pub reserve: &'a str,
}

/// Lays a deposit of `account` on the world: the account, each site's empty
/// stock, the payment by `payer` at `rate` a tick, and the decay at `decay`.
///
/// # Errors
/// Fails if a rate is nought, the account is already some other kind, or the
/// processes pass the world's limit.
pub(crate) fn install(g: &mut Genesis, payer: &Payer, account: &str, rate: u64, decay: u64) -> Result<()> {
    if rate == 0 || decay == 0 {
        return Err("a deposit that pays or decays nothing".into());
    }
    let kind = AccountKind::Matter { lineage: "world:ground".into(), reserve: false, provision: false };
    if g.rules.accounts.get(account).is_some_and(|k| *k != kind) {
        return Err(format!("{account} is already an account of another kind"));
    }
    let mut pay = process(
        &format!("deposit:{}:{account}", payer.marked),
        Causation::Transition,
        vec![Effect::Spend {
            from: vec![payer.reserve.into()],
            to: Binding::Place,
            amount: rate.into(),
            into: Some(account.into()),
        }],
    );
    pay.requires.extend([
        Query::Trait { who: Binding::Actor, key: payer.marked.into() },
        Query::Account { who: Binding::Actor, key: payer.reserve.into(), at_least: rate },
    ]);
    pay.period = Some(1);
    pay.priority = AFTER_MEALS;
    let mut rot = process(
        &format!("deposit:decay:{account}"),
        Causation::Agentless,
        vec![Effect::Transform {
            who: Binding::Place,
            take: BTreeMap::from([(account.into(), decay)]),
            give: BTreeMap::from([(SOIL.into(), decay)]),
            conversion: None,
        }],
    );
    rot.requires.push(Query::Account { who: Binding::Place, key: account.into(), at_least: decay });
    rot.period = Some(1);
    rot.priority = 50;
    let made = [pay, rot].into_iter().filter(|p| !g.rules.processes.contains_key(&p.id)).collect::<Vec<_>>();
    if g.rules.processes.len() + made.len() > g.rules.limits.processes {
        return Err("a deposit's processes exceed the world's process limit".into());
    }
    g.rules.processes.extend(made.into_iter().map(|p| (p.id.clone(), p)));
    g.rules.accounts.insert(account.into(), kind);
    for site in g.sites.values_mut() {
        site.accounts.entry(account.into()).or_insert(0);
    }
    Ok(())
}
