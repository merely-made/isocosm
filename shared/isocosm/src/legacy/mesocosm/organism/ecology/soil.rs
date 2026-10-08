// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ecology's receipt for soil-owned mineralization.

use crate::legacy::mesocosm::flow::{FlowEvent, Records};
use crate::legacy::mesocosm::places::Soil;

pub(super) fn mineralize(soil: &mut Soil, dose_mg: u64, records: &mut Records<'_>) {
    let mut converted = Vec::new();
    soil.mineralize(dose_mg, |column, paid| converted.push((column, paid)));
    for (column, paid) in converted {
        records.flow(
            soil.position_of(column),
            FlowEvent::soil_mineralization(paid),
        );
    }
}
