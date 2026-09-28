// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP2 of the spatial spine (the place-graph engine plan's §A.5 and §A.9):
//! the lift, over grids drawn from ruling 399's space. Each check runs beside
//! a control that must fail.

use isocosm::{
    Founding,
    map::{Grid, Layout, SHAPES},
    simulation::Genesis,
    terrain::{CHUNK, View, check, fading},
};

/// Worlds across every shape, with sites of `side` base units, kept as
/// regression pins; the bench's `--lift-draws` draws from a seed nobody
/// chose.
fn drawn(side: u64) -> Vec<Genesis> {
    let mut worlds = Vec::new();
    for (s, shape) in SHAPES.iter().enumerate() {
        for i in 0..3u64 {
            let seed = isocosm::draw(0x11f7, "lift-test", &[s as u64, i]);
            let drawn = Grid::drawn(seed);
            let grid = Grid {
                shape: (*shape).into(),
                width: 2 + drawn.width % 5,
                height: 2 + drawn.height % 5,
                side,
                elevation: [0, side as i64],
                relief: [0, side as i64 / 8],
                ..drawn
            };
            let founding = Founding {
                seed,
                sites: grid.width * grid.height,
                map: Some(Layout::Grid(grid)),
                ..Founding::default()
            };
            worlds.push(founding.generate().unwrap());
        }
    }
    worlds
}

#[test]
fn a_site_lifts_to_the_same_bytes_twice_and_in_a_second_process() {
    let seed = isocosm::draw(0x11f7, "lift-process", &[]);
    let here = isocosm::bench::lift_digest(seed, 2).unwrap();
    assert_eq!(isocosm::bench::lift_digest(seed, 2).unwrap(), here);
    for _ in 0..2 {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_isocosm-bench"))
            .args(["--seed", &seed.to_string(), "--lift-digest", "2"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), here);
    }
}

#[test]
fn neighbours_meet_exactly_at_every_border_point() {
    for genesis in drawn(512) {
        let view = View::of(&genesis).unwrap();
        assert!(check::borders(&view, |v, s| v.lattice(s)).unwrap() > 0);
    }
}

#[test]
fn detail_that_does_not_fade_parts_the_borders() {
    for genesis in drawn(512) {
        let view = View::of(&genesis).unwrap();
        let unfaded = |v: &View<'_>, s: u64| v.lattice_with(s, |_| 256, true);
        assert!(check::borders(&view, unfaded).is_err());
    }
}

#[test]
fn a_lifted_sites_mean_surface_is_its_elevation_exactly() {
    for genesis in drawn(256) {
        let view = View::of(&genesis).unwrap();
        let sites: Vec<u64> = genesis.sites.keys().copied().collect();
        assert_eq!(
            check::means(&view, &sites, |v, s| v.lattice(s)).unwrap(),
            sites.len() as u64
        );
    }
}

#[test]
fn dropping_the_correction_misses_the_mean() {
    for genesis in drawn(256) {
        let view = View::of(&genesis).unwrap();
        let sites: Vec<u64> = genesis.sites.keys().copied().collect();
        let uncorrected = |v: &View<'_>, s: u64| v.lattice_with(s, fading, false);
        assert!(check::means(&view, &sites, uncorrected).is_err());
    }
}

#[test]
fn detail_stays_within_relief() {
    for genesis in drawn(512) {
        let view = View::of(&genesis).unwrap();
        assert_eq!(
            check::reliefs(&view, |v, s| v.lattice(s)).unwrap(),
            genesis.sites.len() as u64
        );
    }
}

#[test]
fn detail_drawn_past_the_window_breaks_the_relief_bound() {
    for genesis in drawn(512) {
        let view = View::of(&genesis).unwrap();
        let loud = |v: &View<'_>, s: u64| v.lattice_with(s, |k| 2 * fading(k), true);
        assert!(check::reliefs(&view, loud).is_err());
    }
}

#[test]
fn water_fills_to_its_level_and_soil_lies_over_rock() {
    let mut drowned = 0;
    for genesis in drawn(512) {
        let view = View::of(&genesis).unwrap();
        for &site in genesis.sites.keys() {
            let chunk = view.lift(site, 0, [0, 0]).unwrap();
            let m = chunk.materials;
            for z in 0..chunk.columns[1] {
                for x in 0..chunk.columns[0] {
                    let top = chunk.surface[(z * chunk.columns[0] + x) as usize];
                    assert_eq!(chunk.material(x, z, top), m.soil);
                    assert_eq!(chunk.material(x, z, top - chunk.soil), m.rock);
                    let above = chunk.material(x, z, top + 1);
                    assert_eq!(above, if top < chunk.water { m.water } else { m.air });
                    assert_eq!(chunk.material(x, z, chunk.water.max(top) + 1), m.air);
                    drowned += u64::from(top < chunk.water);
                }
            }
        }
    }
    assert!(
        drowned > 0,
        "no drawn column lay under water, so water went untested"
    );
}

#[test]
fn coarse_levels_round_the_same_surface_down() {
    for genesis in drawn(512) {
        let view = View::of(&genesis).unwrap();
        let site = *genesis.sites.keys().next().unwrap();
        let fine = view.lift(site, 0, [0, 0]).unwrap();
        for level in 1..=5u8 {
            let coarse = view.lift(site, level, [0, 0]).unwrap();
            let step = 1u32 << level;
            for z in 0..coarse.columns[1].min(CHUNK / step) {
                for x in 0..coarse.columns[0].min(CHUNK / step) {
                    let base = fine.surface[(z * step * fine.columns[0] + x * step) as usize];
                    let top = coarse.surface[(z * coarse.columns[0] + x) as usize];
                    assert_eq!(top, base.div_euclid(i64::from(step)));
                }
            }
        }
    }
}

#[test]
fn chunks_tile_every_site_at_every_level() {
    let genesis = drawn(512).remove(0);
    let view = View::of(&genesis).unwrap();
    let side = genesis.world.footprint.unwrap().side;
    for level in 0..=9u8 {
        let n = view.chunks(level);
        let mut columns = 0u64;
        for z in 0..n {
            for x in 0..n {
                let chunk = view.lift(0, level, [x, z]).unwrap();
                columns += u64::from(chunk.columns[0]) * u64::from(chunk.columns[1]);
            }
        }
        let cells = side.div_ceil(1 << level);
        assert_eq!(columns, cells * cells, "level {level}");
        assert!(view.lift(0, level, [n, 0]).is_err());
    }
}

#[test]
fn lifting_changes_nothing_in_the_world() {
    let genesis = drawn(512).remove(0);
    let before = isocosm::digest(&genesis);
    let view = View::of(&genesis).unwrap();
    for &site in genesis.sites.keys() {
        view.lift(site, 2, [0, 0]).unwrap();
    }
    assert_eq!(isocosm::digest(&genesis), before);
}

#[test]
fn a_footprint_outside_the_exact_range_is_refused() {
    let mut genesis = drawn(512).remove(0);
    genesis.world.footprint.as_mut().unwrap().side = 520;
    let view = View::of(&genesis).unwrap();
    assert!(view.lift(0, 0, [0, 0]).is_err());
}
