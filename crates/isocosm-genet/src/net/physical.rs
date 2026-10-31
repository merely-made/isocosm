// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_views::{PhysicalRequest, PhysicalOutcome};

impl NetBridge {
    pub fn native_physical(&mut self, input: PhysicalRequest) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativePhysical { request, input }).then_some(request)
    }
    pub fn take_physical_results(&mut self) -> Vec<Correlated<Result<PhysicalOutcome, String>>> {
        std::mem::take(&mut self.physical_results)
    }
}

pub(super) async fn host_request(host: &HostNet, request: PhysicalRequest) -> Result<PhysicalOutcome, String> {
    match request {
        PhysicalRequest::Quote(input) => {
            let preview = host.quote_native_physical(&input).await?;
            Ok(PhysicalOutcome::Quote { input, preview })
        },
        PhysicalRequest::Apply(input) => {
            host.native_physical(&input).await?;
            Ok(PhysicalOutcome::Applied)
        },
    }
}

pub(super) async fn client_request(_: &ClientNet, _: PhysicalRequest) -> Result<PhysicalOutcome, String> {
    Err("physical map authoring requires the actual host".into())
}
