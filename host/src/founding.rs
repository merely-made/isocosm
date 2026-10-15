// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Founding a world (sim host plan H1). A product builds its genesis and
//! the core founds the world under 885 and 897: a head founds a generated
//! genesis only with the joint witness (875) and admission (879) that bind
//! it, while the bench, a dev instrument, and authored worlds need neither.

use isocosm::ecology::{Admission, JointWitness};
use isocosm::simulation::Genesis;
use isocosm::{Execution, Session};

use crate::world::World;

/// Who founds: a product head, or the dev bench.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Head,
    Bench,
}

/// The witness and admission a product ran over its generated candidate.
#[derive(Clone, Copy)]
pub struct Offered<'a> {
    pub witness: &'a JointWitness,
    pub admission: &'a Admission,
}

pub fn found(genesis: Genesis, execution: Execution, purpose: Purpose, offered: Option<Offered<'_>>)
    -> Result<World, String> {
    if genesis.founding.is_some() && purpose == Purpose::Head {
        let offered = offered
            .ok_or("a head founds a generated world only with its admitted joint witness (875, 885)")?;
        if isocosm::digest(&genesis) != offered.admission.genesis_digest() {
            return Err("the admitted joint witness names another generated genesis".into());
        }
        offered.admission.validate(offered.witness)?;
    }
    Ok(World::new(Session::new(genesis, execution)?))
}
