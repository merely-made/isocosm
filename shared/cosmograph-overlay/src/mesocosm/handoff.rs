// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// Geometry returned for a native chosen operation; this declares no new act.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MesocosmHandoff { Terrain(super::TerrainGeometry) }
