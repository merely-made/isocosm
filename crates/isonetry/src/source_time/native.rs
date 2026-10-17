// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{GameSourceHistory, GameSourceTimeError};
use isocosm::{Execution, schema::Id};
use isocosm_vtt::founding::{Campaign, NativeCursor, NativeWorld};

/// One native site at an exact public history prefix, never the live world's geometry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeSiteVersion {
    pub cursor: NativeCursor,
    pub site: Id,
    pub world: NativeWorld,
}

impl NativeSiteVersion {
    pub fn load(&self, mode: Execution) -> Result<Campaign, String> {
        let campaign = self.world.load(mode)?;
        if self.world.cursor()? != self.cursor || !campaign.reading().state().sites.contains_key(&self.site) {
            return Err("native site version identity mismatch".into());
        }
        Ok(campaign)
    }
}

impl GameSourceHistory {
    /// Select an admitted source prefix. This method grants no participant disclosure.
    pub fn native_site_at(
        &self, cursor: u64, site: Id, mode: Execution,
    ) -> Result<NativeSiteVersion, GameSourceTimeError> {
        let snapshot = self.snapshot_at(cursor)?;
        let failure = |reason| GameSourceTimeError::Native { cursor, reason };
        let world = snapshot.native.ok_or_else(|| failure("no native campaign at prefix".into()))?;
        let campaign = world.load(mode).map_err(failure)?;
        if !campaign.reading().state().sites.contains_key(&site) {
            return Err(failure("unknown native site at prefix".into()));
        }
        Ok(NativeSiteVersion { cursor: world.cursor().map_err(failure)?, site, world })
    }
}