// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl NetBridge {
    pub fn native_time(&mut self, intent:cosmograph_overlay::vtt::TimeIntent) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeTime { request, intent }).then_some(request)
    }
    pub fn commit_native_table_admission(&mut self, input: isonetry::native_table::NativeTableAdmissionInput) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeTableAdmission { request, input }).then_some(request)
    }
    pub fn commit_native_table_binding(&mut self, input: isonetry::NativeTableBindingInput) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeTableBinding { request, input }).then_some(request)
    }
    pub fn commit_native_setup(
        &mut self, setup: isocosm_native::founding::Setup,
        expected: isocosm_native::founding::NativeWorld,
        record: GenerationRecord, item_owner: Option<TokenId>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeSetup {
            request, setup, expected, record, item_owner,
        }).then_some(request)
    }

    pub fn commit_native_quickstart(&mut self, pack: String, seed: u64, mode: cosmograph::Execution)
        -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeQuickstart { request, pack, seed, mode }).then_some(request)
    }

    pub fn commit_native_campaign(
        &mut self,
        profile: cosmograph::Founding,
        mode: cosmograph::Execution,
        record: GenerationRecord,
        item_owner: Option<TokenId>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeCampaign {
            request, profile, mode, record, item_owner,
        }).then_some(request)
    }
}

pub(super) async fn host_time(host:&HostNet, intent:cosmograph_overlay::vtt::TimeIntent) -> Result<(), String> {
    let snapshot = host.snapshot().await;
    let envelope = crate::native_time::envelope(&snapshot, isonetry::native_table::TableSender::Host, intent)?;
    host.native_time(&envelope).await
}

pub(super) async fn client_time(client:&ClientNet, intent:cosmograph_overlay::vtt::TimeIntent) -> Result<(), String> {
    let snapshot = client.state().await.ok_or("native time needs the authenticated host snapshot")?;
    let envelope = crate::native_time::envelope(&snapshot,
        isonetry::native_table::TableSender::Endpoint(client.endpoint_key()), intent)?;
    client.native_time(envelope).await
}
