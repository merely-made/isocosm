// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl NetBridge {
    pub fn commit_native_setup(
        &mut self, setup: isometry_campaign::founding::Setup,
        expected: isometry_campaign::founding::NativeWorld,
        record: GenerationRecord, item_owner: Option<TokenId>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeSetup {
            request, setup, expected, record, item_owner,
        }).then_some(request)
    }

    pub fn commit_native_campaign(
        &mut self,
        profile: isocosm::Founding,
        mode: isocosm::Execution,
        record: GenerationRecord,
        item_owner: Option<TokenId>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeCampaign {
            request, profile, mode, record, item_owner,
        }).then_some(request)
    }
}