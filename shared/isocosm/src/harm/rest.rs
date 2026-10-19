// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rest (the 2026-10-18 note to ruling 227): an ordinary Choice act of every
//! lineage that takes the tick's choice and heals one recorded cell, nearest
//! the root first, for a quarter of its mass (an eighth where the lineage
//! heals). It is feasible only while a
//! lost cell can be restored and paid, and in the chooser it serves a wound
//! as paid healing does.

use crate::{generate, rules::*};

pub const ID: &str = "body:rest";

/// The rest a world's bodies may take, its payment going to the soil `into`.
pub fn process(into: &str) -> Process {
    let mut rest = generate::process(
        ID,
        Causation::Choice,
        vec![Effect::Heal {
            into: into.into(),
            rest: true,
        }],
    );
    rest.period = Some(1);
    rest.requires.push(Query::Computed(Expr::AtLeast(
        Box::new(Expr::Read(Reading::Healing {
            who: Binding::Actor,
            rest: true,
        })),
        Box::new(Expr::Const(1)),
    )));
    rest
}
