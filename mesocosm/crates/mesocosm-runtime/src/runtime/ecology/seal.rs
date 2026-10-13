// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Saved admission evidence belongs to the exact native witness prefix.
//! It is not another live world or a new baseline selected during restore.

use isocosm::{Session,ecology::{Admission,JointWitness}};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seal {pub version:u32,pub witness:JointWitness,pub admission:Admission}

impl Seal {
    pub fn validate(&self,session:&Session,replay:bool)->Result<(),String> {
        let saved=&self.witness.native;
        if self.version!=1 || saved.genesis!=*session.sim.genesis() || saved.branch!=session.branch
            || saved.tick>session.sim.state().tick || !session.entries.starts_with(&saved.entries)
            || saved.movement!=Some(session.sim.state().movement)
            || self.witness.execution!=session.sim.mode() {
            return Err("ecology admission names another native source or witness prefix".into());
        }
        if replay {
            let source=session.prefix(saved.entries.len(),saved.tick)?;
            if source.save()!=*saved {return Err("ecology witness differs from the actual native prefix".into());}
            self.admission.validate(&self.witness)?;
        }
        Ok(())
    }
}
