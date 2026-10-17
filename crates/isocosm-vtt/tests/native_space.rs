// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "native_founding/fixture.rs"]
mod fixture;
use isocosm::{Command, Execution, Founding, Session, map::{Grid, Layout}};
use isocosm_vtt::founding::Campaign;
use isometer_space::{Edit, Op, Shape};

#[test]
fn native_space_lifts_admitted_edits_and_replays_exact_site_bytes() {
    let grid = Grid::drawn(59);
    let profile = Founding {
        seed: 59, sites: grid.width * grid.height, population: 8, cohort_size: 2,
        lineages: 2, map: Some(Layout::Grid(grid)), ..Default::default()
    };
    let campaign = Campaign::from_profile_bench(&profile, &fixture::record(), 16, Execution::Grouped)
        .unwrap().campaign;
    let before = campaign.reading().lift(0, 0, [0, 0]).unwrap();
    let y = before.surface[0];
    let command = Command::Edit { site: 0, by: None, edit: Edit {
        op: Op::Carve, shape: Shape::Box { min: [0, y, 0], max: [1, y + 1, 1] },
    }};
    let edited = campaign.with_commands(&[command], 0).unwrap();
    let after = edited.reading().lift(0, 0, [0, 0]).unwrap();
    assert_ne!(before, after, "native lift omitted admitted edit history");
    assert_eq!(after.material(0, 0, y), after.materials.air);
    assert_eq!(campaign.reading().lift(0, 0, [0, 0]).unwrap(), before);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let native = Session::load(edited.save().native.clone(), mode).unwrap();
        let replay = Campaign::load(edited.save(), mode).unwrap();
        assert_eq!(replay.reading().lift(0, 0, [0, 0]).unwrap(), native.sim.lift(0, 0, [0, 0]).unwrap());
        assert_eq!(replay.reading().lift(0, 0, [0, 0]).unwrap(), after);
        let volume = replay.reading().volume(0, [0, 0], [2, 2], 2).unwrap();
        assert_eq!(volume.site, 0);
    }
}

#[test]
fn native_space_refuses_authored_board_position_as_native_geometry() {
    let campaign = Campaign::from_draft_bench(fixture::genesis(), &fixture::draft(), Execution::Grouped)
        .unwrap().campaign;
    let (site, authored) = campaign.reading().authored_site("ford").unwrap();
    assert_eq!(authored.authored.as_ref().unwrap().position, Some((-3, 8)));
    assert_eq!(campaign.reading().position(site), None,
        "authored board position was promoted to native geometry");
    assert!(campaign.reading().lift(site, 0, [0, 0]).unwrap_err().contains("without a footprint"));
}
