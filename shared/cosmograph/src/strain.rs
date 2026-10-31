// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Strain worked out when read (the execution plan's E7, D16). Where every
//! need reads only the actor's own body, mood changes only when the body is
//! written, so a body's strain account holds its value as of `settled` and
//! each tick since builds or bleeds it by the mood the body has now. A read
//! works it out; a write settles it first, by the body as it stood.

use crate::{
    meaning::{Named, Scene},
    rules::{Rate, Rules},
    schema::*,
};

/// The strain account and its rate, where the world keeps strain as one.
fn rated(rules: &Rules) -> Option<(&Key, Rate)> {
    let mind = rules.mind.as_ref()?;
    Some((&mind.strain, mind.rate?))
}

/// Whether `key` is strain kept as a rate.
pub(crate) fn is_rated(rules: &Rules, key: &str) -> bool {
    rated(rules).is_some_and(|(k, _)| k == key)
}

/// `e`'s strain at `tick`.
pub(crate) fn at(e: &Entity, rules: &Rules, tick: Tick) -> u64 {
    let Some((key, rate)) = rated(rules) else {
        return e.accounts.get(key_of(rules)).copied().unwrap_or(0);
    };
    let held = e.accounts.get(key).copied().unwrap_or(0);
    let ticks = tick.saturating_sub(e.settled.unwrap_or(e.born));
    if ticks == 0 || !e.alive || e.method == Method::Inert {
        return held;
    }
    let related = |_: &Key| -> crate::Result<bool> { Err("no target".into()) };
    let scene = Scene {
        actor: Some(e),
        target: Named::Unnamed,
        part: None,
        site: None,
        tick,
        related: &related,
        rules,
        lineages: None,
    };
    let low = crate::meaning::mood(&scene).is_ok_and(|m| m < -(rate.low_mood as i64));
    match low {
        true => held.saturating_add(rate.build.saturating_mul(ticks)),
        false => held.saturating_sub(rate.bleed.saturating_mul(ticks)),
    }
}

fn key_of(rules: &Rules) -> &str {
    rules.mind.as_ref().map_or("", |m| m.strain.as_str())
}

/// What `e`'s strain has gained (or, below nought, bled) since it was last
/// settled, as of `tick`; none where the world keeps no rated strain.
pub(crate) fn gained(e: &Entity, rules: &Rules, tick: Tick) -> Option<i128> {
    let (key, _) = rated(rules)?;
    let before = e.accounts.get(key).copied().unwrap_or(0);
    Some(at(e, rules, tick) as i128 - before as i128)
}

/// Carries what strain `gained` onto `e`, settled as of `tick`.
pub(crate) fn carry(e: &mut Entity, rules: &Rules, gained: i128, tick: Tick) {
    let Some((key, _)) = rated(rules) else {
        return;
    };
    let held = e.accounts.get(key).copied().unwrap_or(0) as i128;
    match (held + gained).clamp(0, u64::MAX as i128) as u64 {
        0 => e.accounts.remove(key),
        v => e.accounts.insert(key.clone(), v),
    };
    e.settled = Some(tick);
}
