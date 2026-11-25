// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::rules::{Accretion, Condition, Rule, Symmetry};

pub(super) fn describe(rule: &Rule) -> String {
    let run = |n: &u8| format!("run {}", u16::from(*n) + 1);
    match rule {
        Rule::Repeat { tagma, by } => format!("Extend {} by {by} segments", run(tagma)),
        Rule::Branch {
            tagma,
            every,
            depth,
            ..
        } => {
            format!(
                "Branch {} at intervals of {every} segments, {depth} levels deep",
                run(tagma)
            )
        },
        Rule::Differentiate { tagma, at, kind } => {
            format!(
                "Grow {} from segment {} of {}",
                super::small(kind),
                u16::from(*at) + 1,
                run(tagma)
            )
        },
        Rule::Accrete { tagma, how } => match how {
            Accretion::Harden { oldest, kind } => format!(
                "Harden {oldest} old segments of {} into {}",
                run(tagma),
                super::small(kind)
            ),
            Accretion::Cover { kind, .. } => {
                format!("Cover {} with {}", run(tagma), super::small(kind))
            },
        },
        Rule::Radiate { tagma, arms, fused } => {
            let reserves = if *fused { " with a shared reserve" } else { "" };
            format!("Grow {arms} arms around {}{reserves}", run(tagma))
        },
        Rule::Partner { tagma, kept } => kept.as_ref().map_or_else(
            || {
                format!(
                    "Reserve a socket on {} for an incorporated part",
                    run(tagma)
                )
            },
            |kind| {
                format!(
                    "Carry {} in the socket on {}",
                    super::small(kind),
                    run(tagma)
                )
            },
        ),
        Rule::Fuse { runs } => format!(
            "Pool reserves across {}",
            runs.iter().map(run).collect::<Vec<_>>().join(", ")
        ),
        Rule::Deposit {
            account,
            rate,
            decay,
        } => format!(
            "Deposit {rate} units of {} per tick, decaying by {decay}",
            super::small(account)
        ),
        Rule::Symmetry { tagma, symmetry } => match symmetry {
            Symmetry::None => format!("Place each appendage of {} once", run(tagma)),
            Symmetry::Bilateral => format!("Mirror the appendages on {}", run(tagma)),
            Symmetry::Radial { arms } => {
                format!("Arrange the appendages of {} in {arms} arms", run(tagma))
            },
        },
        Rule::Switch { when, rules } => {
            let condition = match when {
                Condition::Age { ticks } => format!("from age {ticks} ticks"),
                Condition::Crowding { bodies } => {
                    format!("with at least {bodies} members of the lineage present")
                },
                Condition::FullReserves => "with full reserves".to_owned(),
                Condition::Season { from, to } => {
                    format!("during the seasonal interval {from}–{to}‰ of the year")
                },
            };
            format!(
                "{} ({condition})",
                rules.iter().map(describe).collect::<Vec<_>>().join(", ")
            )
        },
        Rule::Allocate { total } => format!("Share a budget of {total} segments across the body"),
        Rule::Partition(part) => format!(
            "Divide the organ on {} into {} pieces",
            run(&part.tagma),
            part.pieces
        ),
    }
}
