// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Founding;
use std::collections::{BTreeMap, BTreeSet};

impl Default for Founding {
    fn default() -> Self {
        Self {
            seed: 1,
            sites: 6,
            population: 256,
            lineages: 4,
            cohort_size: 32,
            stock_min: 8,
            stock_max: 64,
            period_max: 5,
            base_unit_micrometres: 1000,
            ecology: false,
            map: None,
            preset: None,
            bodies: None,
            played: None,
            affinity: None,
            ecology_rates: Default::default(),
            web: Default::default(),
            experimental_food_contract: false,
            dormancy: None,
            mass_law: None,
            limits: None,
            rot: None,
            spoilage: None,
            repertoire: None,
            categories: None,
            guard: None,
            inherit: false,
            inherited_variation: 0,
            population_bands: None,
            cohort_births: false,
            mutualism: None,
            pocket: None,
            detritus: None,
            nutrients: None,
            body_maps: BTreeMap::new(),
            unstocked_lineages: BTreeSet::new(),
        }
    }
}
