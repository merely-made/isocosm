// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isometry_views::{StoryletPhysicalRequest, StoryletPhysicalOutcome};

impl NetBridge {
    pub fn native_composed_storylet(&mut self, input: StoryletPhysicalRequest) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor.command(BridgeCommand::NativeComposedStorylet { request, input }).then_some(request)
    }
    pub fn take_storylet_physical_results(&mut self) -> Vec<Correlated<Result<StoryletPhysicalOutcome, String>>> {
        std::mem::take(&mut self.storylet_physical_results)
    }
}

pub(super) async fn host_request(host: &HostNet, request: StoryletPhysicalRequest) -> Result<StoryletPhysicalOutcome, String> {
    match request {
        StoryletPhysicalRequest::Quote { input, item_owner } => {
            let result = host.quote_composed_storylet(&input, item_owner).await?;
            Ok(StoryletPhysicalOutcome::Quote { input, result })
        },
        StoryletPhysicalRequest::Apply { input, item_owner } => {
            host.commit_composed_storylet(&input, item_owner).await?;
            Ok(StoryletPhysicalOutcome::Applied)
        },
    }
}

pub(super) async fn client_request(_: &ClientNet, _: StoryletPhysicalRequest) -> Result<StoryletPhysicalOutcome, String> {
    Err("physical storylet authoring and private eligibility require the actual host".into())
}
