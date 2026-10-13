// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{projects::{Acted, DebitPreview, DebitSelection, Owner, ProjectId}, schema::{Id,Key,Ledger}};
use isometer_space::places::{PlaceId,Rules};
use serde::{Deserialize,Serialize};
use crate::ItemId;
use super::super::{PlacementWindow,PositionBinding};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub enum EntityRef { Existing(Id), SelectedProjectOutput }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemPlacement {
    pub site:Id,pub patch:PlaceId,pub at:[i64;3],pub window:PlacementWindow,pub places:Rules,
    pub body_revision:u64,pub terrain_revision:Key,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemBindingInput {
    pub item:ItemId,pub entity:EntityRef,pub lineage:Key,pub stocks:Ledger,
    pub development_seed:Option<u64>,pub placement:ItemPlacement,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub enum ItemAction {
    AuthorBind(ItemBindingInput),
    Project { process:Key,target:Option<Id>,project:Option<ProjectId>,lots:Vec<Key>,
        selections:Vec<DebitSelection>,output:Option<ItemBindingInput> },
    Consent { project:ProjectId,owner:Owner,worker:Id,grant:bool,process:Option<Key>,cause:Option<Key> },
    Act { process:Key,target:Option<Id>,cause:Option<Key>,selections:Vec<DebitSelection> },
    AuthorOwn(isocosm::projects::StockLot),
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemInput {
    pub version:u32,pub expected_native:Key,pub expected_game:Key,
    pub actor:Id,pub body_revision:u64,pub action:ItemAction,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct BoundItem { pub item:ItemId,pub entity:Id,pub placement:ItemPlacement,pub patch_entry:Key,pub resident_since:u64 }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct ProjectPosition {
    pub project:ProjectId,pub site:Id,pub at:[i64;3],pub created_site:Id,pub created_at:[i64;3],
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalItemRecord {
    pub input:ItemInput,pub participant:Id,pub actor_position:PositionBinding,
    pub entries:Vec<Key>,pub outcomes:Vec<String>,pub binding:Option<BoundItem>,pub project:Option<ProjectPosition>,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct ItemPreview {
    pub debits:Option<DebitPreview>,pub complete:bool,pub refusal:Option<String>,
    pub project:Option<Acted>,pub binding:Option<BoundItem>,
}
