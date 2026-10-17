// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The stable intent discriminants, with only anatomy's wire parameterized.

use crate::identity::{BodyRevisionId, SubjectId, Tick};
use crate::*;
use isocosm::lineage::LineageBody;
use isometer_core::PartId;
use serde::{Deserialize, Serialize};

macro_rules! intents {
    ($($before:ident { $($bf:ident : $bt:ty),* }),*;
     $($after:ident { $($af:ident : $at:ty),* }),* $(,)?) => {
        #[derive(Serialize, Deserialize)]
        pub(super) enum Wire<D> {
            $($before { $($bf:$bt),* },)*
            AdmitAnatomy { tick:Tick, subject:SubjectId, revision:BodyRevisionId, document:Box<D> },
            $($after { $($af:$at),* },)*
        }
        impl<D> Wire<D> {
            pub(super) fn map<E>(self, convert:impl FnOnce(D)->Result<E,String>) -> Result<Wire<E>,String> {
                Ok(match self {
                    $(Self::$before { $($bf),* } => Wire::$before { $($bf),* },)*
                    Self::AdmitAnatomy { tick, subject, revision, document } =>
                        Wire::AdmitAnatomy { tick, subject, revision, document:Box::new(convert(*document)?) },
                    $(Self::$after { $($af),* } => Wire::$after { $($af),* },)*
                })
            }
        }
        impl Wire<LineageBody> {
            pub(super) fn from_game(intent:GameIntent) -> Self {
                match intent {
                    $(GameIntent::$before { $($bf),* } => Self::$before { $($bf),* },)*
                    GameIntent::AdmitAnatomy { tick, subject, revision, document } =>
                        Self::AdmitAnatomy { tick, subject, revision, document },
                    $(GameIntent::$after { $($af),* } => Self::$after { $($af),* },)*
                }
            }
            pub(super) fn into_game(self) -> GameIntent {
                match self {
                    $(Self::$before { $($bf),* } => GameIntent::$before { $($bf),* },)*
                    Self::AdmitAnatomy { tick, subject, revision, document } =>
                        GameIntent::AdmitAnatomy { tick, subject, revision, document },
                    $(Self::$after { $($af),* } => GameIntent::$after { $($af),* },)*
                }
            }
        }
    }
}

// Order is the original postcard discriminant order. New vocabulary stays
// decoded for the existing version guards to reject, never silently lowered.
intents! {
    Generate { tick:Tick, subject:SubjectId, body_seed:u64, at:[i32;3] },
    Name { tick:Tick, subject:SubjectId, name:Name },
    Move { tick:Tick, subject:SubjectId, toward:[i32;3] },
    Observe { tick:Tick, subject:SubjectId, target:[i32;3] },
    Take { tick:Tick, subject:SubjectId, item:ItemId },
    Eat { tick:Tick, subject:SubjectId, item:ItemId },
    Rest { tick:Tick, subject:SubjectId },
    Fall { tick:Tick, subject:SubjectId, distance:i32 },
    Wait { tick:Tick, subject:SubjectId };
    ReconcileAnatomy { tick:Tick, subject:SubjectId, from_revision:BodyRevisionId,
        revision:BodyRevisionId, severed_parts:Vec<PartId> },
    AttachItem { tick:Tick, subject:SubjectId, item:ItemId, part:PartId, revision:BodyRevisionId },
    DetachItem { tick:Tick, subject:SubjectId, item:ItemId },
    ResolveVolley { tick:Tick, actor:SubjectId, target:SubjectId,
        strikes:Vec<crate::timed_action::StrikeReceipt>, rules:CombatRules },
    AdvanceMotion { tick:Tick, subject:SubjectId, revision:BodyRevisionId,
        step:u64, input:MotionInput, rules:MotionRules },
    ConfigureMovementProfile { tick:Tick, subject:SubjectId, revision:BodyRevisionId, profile:MovementProfile },
    ReviseCanon { tick:Tick, revision:u64, seed:u64, cause:CanonRevisionCause },
    GenerateNativeRecipe { tick:Tick, subject:SubjectId, body_seed:u64, at:[i32;3], recipe:NativeFixtureRecipe },
    Drop { tick:Tick, subject:SubjectId, item:ItemId },
    ProjectAct { tick:Tick, subject:SubjectId, process:String,
        target:Option<isocosm::schema::Id>, project:Option<isocosm::projects::ProjectId> },
    ProjectCarry { tick:Tick, subject:SubjectId, project:isocosm::projects::ProjectId,
        carried:bool },
    ProjectReserve { tick:Tick, subject:SubjectId, process:String,
        target:Option<isocosm::schema::Id>, lots:Vec<String> },
    StockOwnership { tick:Tick, lot:isocosm::projects::StockLot },
    ProjectConsent { tick:Tick, subject:SubjectId, project:isocosm::projects::ProjectId,
        owner:isocosm::projects::Owner, worker:SubjectId, grant:bool, process:Option<String> },
    StockAct { tick:Tick, subject:SubjectId, process:String,
        target:Option<isocosm::schema::Id>, selections:Vec<isocosm::projects::DebitSelection> },
    EatOwned { tick:Tick, subject:SubjectId, item:ItemId,
        selections:Vec<isocosm::projects::DebitSelection> },
    FoundSpreadActor { tick:Tick, key:String, recipe:String, site:isocosm::schema::Id },
    SpreadOwnership { tick:Tick, identity:crate::IdentityRef, lot:crate::AuthorSpreadLot },
    AuthoredSpread { tick:Tick, step:crate::AuthorSpreadStep },
    ExternalVolley { tick:Tick, actor:SubjectId, actor_revision:BodyRevisionId,
        target:SubjectId, strikes:Vec<crate::timed_action::StrikeReceipt>, rules:CombatRules },
}
