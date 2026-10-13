// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl HostNet {
    pub async fn native_journey(&self, input: &isometry_campaign::native::joint_journey::JourneyEnvelope) -> Result<(), String> {
        let out = self.session.lock().await.native_journey(crate::native_table::TableSender::Host, input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn native_party(&self, envelope: &isocosm_overlay::vtt::VttIntentEnvelope) -> Result<(), String> {
        let out = self.session.lock().await.native_party(crate::native_table::TableSender::Host, envelope)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn native_time(&self, envelope: &isocosm_overlay::vtt::VttIntentEnvelope) -> Result<(), String> {
        let out = self.session.lock().await.native_time(crate::native_table::TableSender::Host, envelope)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
    pub async fn commit_native_table_admission(&self, input: &crate::native_table::NativeTableAdmissionInput) -> Result<(), String> {
        let out = self.session.lock().await.commit_native_table_admission(input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn authenticated_endpoints(&self) -> Vec<[u8; 32]> {
        self.session.lock().await.authenticated_endpoints()
    }

    pub async fn commit_native_table_binding(&self, input: &crate::NativeTableBindingInput) -> Result<(), String> {
        let out = self.session.lock().await.commit_native_table_binding(input)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
    pub async fn commit_native_setup(
        &self, setup: &isometry_campaign::founding::Setup,
        expected: &isometry_campaign::founding::NativeWorld,
        record: &isometry_campaign::GenerationRecord,
        item_owner: Option<isometry_core::TokenId>,
    ) -> Result<(), String> {
        let out = self.session.lock().await.commit_native_setup(setup, expected, record, item_owner)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }

    pub async fn commit_native_campaign(
        &self,
        profile: &isocosm::Founding,
        record: &isometry_campaign::GenerationRecord,
        item_owner: Option<isometry_core::TokenId>,
        mode: isocosm::Execution,
    ) -> Result<(), String> {
        let out = self.session.lock().await
            .commit_native_campaign(profile, record, item_owner, mode)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
}
