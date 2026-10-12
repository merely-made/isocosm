// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl HostNet {
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