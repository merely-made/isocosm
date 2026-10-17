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
use crate::save::{WorldSave, FORMAT_VERSION};

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
    validate(&genesis, purpose, offered)?;
    Ok(World::new(Session::new(genesis, execution)?))
}

/// Continue the exact admitted witness trajectory, rather than resetting it
/// to the candidate's tick zero. Product setup happens after this core load.
pub fn from_admitted(execution: Execution, offered: Offered<'_>) -> Result<World, String> {
    validate_offered(&offered.witness.native.genesis, offered)?;
    if execution != offered.witness.execution {
        return Err("the offered execution differs from its admitted witness".into());
    }
    World::resume(WorldSave { format: FORMAT_VERSION, native: offered.witness.native.clone(),
        sections: Default::default() }, execution)
}

fn validate(genesis: &Genesis, purpose: Purpose, offered: Option<Offered<'_>>) -> Result<(), String> {
    if genesis.founding.is_some() && purpose == Purpose::Head {
        let offered = offered
            .ok_or("a head founds a generated world only with its admitted joint witness (875, 885)")?;
        validate_offered(genesis, offered)?;
    }
    Ok(())
}

fn validate_offered(genesis: &Genesis, offered: Offered<'_>) -> Result<(), String> {
    if isocosm::digest(genesis) != offered.admission.genesis_digest() {
        return Err("the admitted joint witness names another generated genesis".into());
    }
    offered.admission.validate(offered.witness)
}
