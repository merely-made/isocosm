// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an intent names: its tick, its subject, and who may author it.

use super::*;

impl GameIntent {
    pub const fn tick(&self) -> Tick {
        match self {
            Self::Generate { tick, .. }
            | Self::GenerateNativeRecipe { tick, .. }
            | Self::Drop { tick, .. }
            | Self::ProjectAct { tick, .. }
            | Self::ProjectCarry { tick, .. }
            | Self::ProjectReserve { tick, .. }
            | Self::StockOwnership { tick, .. }
            | Self::ProjectConsent { tick, .. }
            | Self::StockAct { tick, .. }
            | Self::EatOwned { tick, .. }
            | Self::FoundSpreadActor { tick, .. }
            | Self::SpreadOwnership { tick, .. }
            | Self::AuthoredSpread { tick, .. }
            | Self::ExternalVolley { tick, .. }
            | Self::Minute { tick }
            | Self::Name { tick, .. }
            | Self::Move { tick, .. }
            | Self::Observe { tick, .. }
            | Self::Take { tick, .. }
            | Self::Eat { tick, .. }
            | Self::Rest { tick, .. }
            | Self::Fall { tick, .. }
            | Self::AdmitAnatomy { tick, .. }
            | Self::ReconcileAnatomy { tick, .. }
            | Self::AttachItem { tick, .. }
            | Self::DetachItem { tick, .. }
            | Self::ResolveVolley { tick, .. }
            | Self::AdvanceMotion { tick, .. }
            | Self::Steps { tick, .. }
            | Self::ConfigureMovementProfile { tick, .. }
            | Self::ReviseCanon { tick, .. }
            | Self::Wait { tick, .. } => *tick,
        }
    }

    /// The subject this intent is for, or `None` for a world-level intent
    /// that belongs to no body.
    pub const fn subject(&self) -> Option<SubjectId> {
        match self {
            Self::Generate { subject, .. }
            | Self::GenerateNativeRecipe { subject, .. }
            | Self::Drop { subject, .. }
            | Self::ProjectAct { subject, .. }
            | Self::ProjectCarry { subject, .. }
            | Self::ProjectReserve { subject, .. }
            | Self::ProjectConsent { subject, .. }
            | Self::StockAct { subject, .. }
            | Self::EatOwned { subject, .. }
            | Self::Name { subject, .. }
            | Self::Move { subject, .. }
            | Self::Observe { subject, .. }
            | Self::Take { subject, .. }
            | Self::Eat { subject, .. }
            | Self::Rest { subject, .. }
            | Self::Fall { subject, .. }
            | Self::AdmitAnatomy { subject, .. }
            | Self::ReconcileAnatomy { subject, .. }
            | Self::AttachItem { subject, .. }
            | Self::DetachItem { subject, .. }
            | Self::AdvanceMotion { subject, .. }
            | Self::Steps { subject, .. }
            | Self::ConfigureMovementProfile { subject, .. }
            | Self::Wait { subject, .. } => Some(*subject),
            Self::ResolveVolley { actor, .. } => Some(*actor),
            Self::ExternalVolley { .. }
            | Self::Minute { .. }
            | Self::ReviseCanon { .. }
            | Self::StockOwnership { .. }
            | Self::FoundSpreadActor { .. }
            | Self::SpreadOwnership { .. }
            | Self::AuthoredSpread { .. } => None,
        }
    }

    pub const fn is_author_only(&self) -> bool {
        matches!(
            self,
            Self::StockOwnership { .. }
                | Self::FoundSpreadActor { .. }
                | Self::SpreadOwnership { .. }
                | Self::AuthoredSpread { .. }
        )
    }
}
