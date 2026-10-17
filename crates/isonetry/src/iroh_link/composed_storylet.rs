// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_vtt::native::storylets::ComposedStoryletInput;

impl HostNet {
    pub async fn quote_composed_storylet(&self, input: &ComposedStoryletInput, item_owner: Option<isometry_core::TokenId>)
        -> Result<crate::NativeComposedStoryletResolved, String> {
        self.session.lock().await.quote_composed_storylet(input, item_owner)
    }
    pub async fn commit_composed_storylet(&self, input: &ComposedStoryletInput, item_owner: Option<isometry_core::TokenId>)
        -> Result<(), String> {
        let out = self.session.lock().await.commit_composed_storylet(input, item_owner)?;
        dispatch(&self.peers, out).await;
        Ok(())
    }
}
