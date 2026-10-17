// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{Execution, Founding, map::{Grid, Layout}, site_authoring::Attachment};
use isocosm_vtt::founding::Setup;
use std::collections::BTreeMap;

pub fn setup(mode: Execution) -> Setup {
    let founding = Founding { seed: 83, sites: 4, population: 4, lineages: 2, cohort_size: 2,
        map: Some(Layout::Grid(Grid { shape: "shape:plane".into(), width: 2, height: 2,
            side: 256, elevation: [0, 0], relief: [0, 0], sea_per_mille: 0 })),
        ..Default::default() };
    let genesis = founding.generate().unwrap();
    let mut geometry = genesis.sites[&0].clone();
    geometry.accounts.clear(); geometry.routes.clear(); geometry.authored = None;
    geometry.terrain_seed = 991;
    Setup { name: "River world".into(), founding, execution: mode,
        party_relations: Default::default(), pace_profiles: Default::default(),
        paced_processes: Default::default(), authored_processes: Default::default(),
        authored_note_kinds: Default::default(), institutions: Default::default(),
        material_profile: None, item_custody: None, institution_producers: Default::default(),
        attachments: BTreeMap::from([("ford".into(), Attachment::Existing(0)),
            ("shrine".into(), Attachment::New { site: 1000, geometry })]) }
}
