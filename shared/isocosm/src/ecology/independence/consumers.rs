// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::{Effect, Query}, simulation::Genesis};

pub(super) fn consumed(genesis: &Genesis, extra: &ExtraNoun) -> Result<()> {
    let process = genesis.rules.processes.get(&extra.consumer.process).ok_or("extra noun has no native process consumer")?;
    if extra.effect.process != process.id || !genesis.lineages.contains_key(&extra.consumer.actor_lineage)
        || extra.consumer.target_lineage.as_ref().is_some_and(|lineage| !genesis.lineages.contains_key(lineage)) {
        return Err("extra noun consumer has inconsistent process/lineage identity".into());
    }
    let (_, leaf) = project::leaf(genesis, &extra.effect)?;
    let effect_account = |key: &str| match leaf {
        Effect::Transfer { account, .. } => account == key,
        Effect::Transform { take, give, .. } => take.contains_key(key) || give.contains_key(key),
        Effect::Spend { from, into, .. } => from.iter().any(|k| k == key) || into.as_deref() == Some(key),
        Effect::Eat { into, of, .. } => into == key || of.iter().any(|k| k == key),
        Effect::Convert { from, to, .. } => to == key || from.iter().any(|k| k == key),
        Effect::Birth { provision } => provision.contains_key(key),
        _ => false,
    };
    let used = match &extra.noun {
        NativeReference::Process(key) => key == &process.id,
        NativeReference::Account(key) => effect_account(key) || process.need_account.as_ref() == Some(key)
            || process.requires.iter().any(|q| matches!(q, Query::Account { key:k,.. } | Query::Below { key:k,.. } if k == key)),
        NativeReference::Condition(key) => matches!(leaf, Effect::Condition { key:k,.. } if k == key)
            || process.requires.iter().any(|q| matches!(q, Query::Condition { key:k,.. } if k == key)),
        NativeReference::Trait(key) => process.requires.iter().any(|q| matches!(q, Query::Trait { key:k,.. } if k == key)),
        NativeReference::Function(key) => matches!(leaf, Effect::Allocate { from,to,.. } if to == key || from.as_ref() == Some(key))
            || process.requires.iter().any(|q| matches!(q, Query::Expresses {function} | Query::Routes {function,..} if function == key)),
        NativeReference::Lineage(key) => &extra.consumer.actor_lineage == key
            || extra.consumer.target_lineage.as_ref() == Some(key)
            || genesis.rules.accounts.iter().any(|(account,kind)| effect_account(account)
                && matches!(kind, crate::rules::AccountKind::Matter {lineage,..} if lineage == key)),
        NativeReference::Material(_) => return Err("material independence requires an actual priced terrain consumer; this axis is not implemented".into()),
        NativeReference::Kind(_) => return Err("kind independence requires actual recipe/physiology construction evidence; this axis is not implemented".into()),
        _ => return Err("changed native law/catalogue consumer is outside finite independence support".into()),
    };
    if !used { return Err("extra noun does not reach its named native mechanical consumer".into()); }
    Ok(())
}
