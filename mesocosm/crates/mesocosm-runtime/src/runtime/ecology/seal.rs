// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Saved admission evidence belongs to the exact native witness prefix.
//! It is not another live world or a new baseline selected during restore.

use isocosm::{Session,ecology::{Admission,JointWitness}};
use serde::{Deserialize,Serialize};
use std::{ops::{Deref,DerefMut},sync::{Arc,OnceLock}};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealData {pub version:u32,pub witness:JointWitness,pub admission:Admission}

/// Large admission evidence is shared until explicitly edited. Its JSON is a
/// derivative cache only; decoding and mutable access always start uncached.
#[derive(Clone)]
pub struct Seal(Arc<Shared>);
struct Shared {data:SealData,json:OnceLock<Vec<u8>>}
impl Clone for Shared {
    fn clone(&self)->Self {Self {data:self.data.clone(),json:OnceLock::new()}}
}
impl Deref for Seal {
    type Target=SealData;
    fn deref(&self)->&SealData {&self.0.data}
}
impl DerefMut for Seal {
    fn deref_mut(&mut self)->&mut SealData {
        let shared=Arc::make_mut(&mut self.0);
        shared.json=OnceLock::new();
        &mut shared.data
    }
}
impl std::fmt::Debug for Seal {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {self.0.data.fmt(f)}
}
impl PartialEq for Seal {
    fn eq(&self,other:&Self)->bool {Arc::ptr_eq(&self.0,&other.0)||self.0.data==other.0.data}
}
impl Eq for Seal {}
impl Serialize for Seal {
    fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error> {
        self.0.data.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Seal {
    fn deserialize<D:serde::Deserializer<'de>>(deserializer:D)->Result<Self,D::Error> {
        SealData::deserialize(deserializer).map(Self::from_data)
    }
}

impl Seal {
    pub fn new(witness:JointWitness,admission:Admission)->Self {
        Self::from_data(SealData {version:1,witness,admission})
    }
    fn from_data(data:SealData)->Self {Self(Arc::new(Shared {data,json:OnceLock::new()}))}
    pub(crate) fn json(&self)->&[u8] {
        self.0.json.get_or_init(||serde_json::to_vec(&self.0.data)
            .expect("ecology seal serializes"))
    }
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
