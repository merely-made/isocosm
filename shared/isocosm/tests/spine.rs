// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP1 of the spatial spine (the place-graph engine plan's §A.5): world maps
//! with geometry, skeletons and edge profiles, over grids drawn from ruling
//! 399's space. Each check runs beside a control that must fail.

use isocosm::{
    Founding,
    map::{Grid, Layout, SHAPES},
    schema::Border,
    simulation::Genesis,
    terrain::{EdgeProfile, SPANS, View, check},
};

/// Worlds drawn across every shape, kept as regression pins; the bench's
/// `--map-draws` draws from a seed nobody chose.
fn drawn() -> Vec<(Founding, Genesis)> {
    let mut worlds = Vec::new();
    for (s, shape) in SHAPES.iter().enumerate() {
        for i in 0..6u64 {
            let seed = isocosm::draw(0x5b1e, "spine-test", &[s as u64, i]);
            let grid = Grid {
                shape: (*shape).into(),
                ..Grid::drawn(seed)
            };
            let founding = Founding {
                seed,
                sites: grid.width * grid.height,
                map: Some(Layout::Grid(grid)),
                ..Founding::default()
            };
            let genesis = founding.generate().unwrap();
            worlds.push((founding, genesis));
        }
    }
    worlds
}

#[test]
fn worlds_without_a_map_serialize_as_before() {
    let json = serde_json::to_string(&Founding::default().generate().unwrap()).unwrap();
    for key in [
        "\"map\"",
        "\"border\"",
        "\"footprint\"",
        "\"skeleton\"",
        "\"flipped\"",
    ] {
        assert!(
            !json.contains(key),
            "{key} appeared in a world without a map"
        );
    }
}

#[test]
fn drawn_maps_lay_every_site_and_found_the_same_world_twice() {
    for (founding, genesis) in drawn() {
        let Some(Layout::Grid(grid)) = &founding.map else {
            unreachable!()
        };
        assert_eq!(genesis.sites.len() as u32, grid.width * grid.height);
        assert_eq!(genesis.world.shape, grid.shape);
        assert_eq!(
            isocosm::digest(&founding.generate().unwrap()),
            isocosm::digest(&genesis)
        );
    }
}

#[test]
fn every_border_reads_the_same_from_either_side() {
    for (_, genesis) in drawn() {
        let view = View::of(&genesis).unwrap();
        assert!(check::profiles(&view, |v, s, k| v.edge_profile(s, k)).unwrap() > 0);
    }
}

#[test]
fn a_profile_keyed_by_the_ordered_pair_fails_the_symmetry_check() {
    let ordered = |v: &View<'_>, s: u64, k: u8| -> isocosm::Result<EdgeProfile> {
        let mut profile = v.edge_profile(s, k)?;
        profile.heights[SPANS as usize / 2] += s as i64 * 4 + i64::from(k) + 1;
        Ok(profile)
    };
    for (_, genesis) in drawn() {
        let view = View::of(&genesis).unwrap();
        assert!(check::profiles(&view, ordered).is_err());
    }
}

#[test]
fn corners_agree_and_every_border_ends_at_its_corners() {
    for (_, genesis) in drawn() {
        let view = View::of(&genesis).unwrap();
        assert!(check::corners(&view, |v, s, c| v.corner_height(s, c)).unwrap() > 0);
    }
}

#[test]
fn corner_walks_find_every_corner_of_the_grid_once() {
    for (founding, genesis) in drawn() {
        let Some(Layout::Grid(grid)) = &founding.map else {
            unreachable!()
        };
        let view = View::of(&genesis).unwrap();
        let mut corners = std::collections::BTreeSet::new();
        let mut slots = 0;
        for &site in genesis.sites.keys() {
            for corner in 0..4 {
                let class = view.corner_class(site, corner);
                assert!(class.contains(&isocosm::terrain::CornerKey { site, corner }));
                corners.insert(class);
                slots += 1;
            }
        }
        let (w, h) = (grid.width as usize, grid.height as usize);
        let expected = match grid.shape.as_str() {
            "shape:plane" => (w + 1) * (h + 1),
            "shape:ring" => w * (h + 1),
            _ => w * h,
        };
        assert_eq!(corners.len(), expected, "{} {w}x{h}", grid.shape);
        let members: usize = corners.iter().map(Vec::len).sum();
        assert_eq!(members, slots, "every slot belongs to exactly one corner");
    }
}

#[test]
fn a_corner_height_keyed_per_site_fails_corner_agreement() {
    let per_site = |v: &View<'_>, s: u64, c: u8| -> isocosm::Result<i64> {
        Ok(v.corner_height(s, c)? + s as i64 + 1)
    };
    for (_, genesis) in drawn() {
        let view = View::of(&genesis).unwrap();
        assert!(check::corners(&view, per_site).is_err());
    }
}

#[test]
fn a_map_whose_sites_disagree_with_its_founding_is_refused() {
    let grid = Grid {
        width: 3,
        height: 3,
        ..Grid::drawn(7)
    };
    let founding = Founding {
        seed: 7,
        sites: 10,
        map: Some(Layout::Grid(grid)),
        ..Founding::default()
    };
    assert!(founding.generate().unwrap_err().contains("disagree"));
}

#[test]
fn broken_borders_and_missing_skeletons_are_refused() {
    let (_, genesis) = drawn().remove(0);
    let bordered = |g: &mut Genesis| {
        g.sites
            .get_mut(&0)
            .unwrap()
            .routes
            .iter_mut()
            .find(|r| r.border.is_some())
            .unwrap()
            .clone()
    };

    let mut g = genesis.clone();
    let route = g
        .sites
        .get_mut(&0)
        .unwrap()
        .routes
        .iter_mut()
        .find(|r| r.border.is_some());
    let border = route.unwrap().border.as_mut().unwrap();
    border.enters = (border.enters + 1) % 4;
    assert!(g.validate().unwrap_err().contains("reverse"));

    let mut g = genesis.clone();
    let route = g
        .sites
        .get_mut(&0)
        .unwrap()
        .routes
        .iter_mut()
        .find(|r| r.border.is_some());
    route.unwrap().border.as_mut().unwrap().side = 9;
    assert!(g.validate().unwrap_err().contains("outside"));

    let mut g = genesis.clone();
    let twin = bordered(&mut g);
    g.sites.get_mut(&0).unwrap().routes.push(twin);
    assert!(g.validate().unwrap_err().contains("two borders"));

    let mut g = genesis.clone();
    g.world.footprint = None;
    assert!(g.validate().unwrap_err().contains("without a footprint"));

    let mut g = genesis;
    g.sites
        .get_mut(&0)
        .unwrap()
        .conditions
        .remove("terrain:elevation");
    assert!(g.validate().unwrap_err().contains("without its skeleton"));
}

#[test]
fn the_new_types_round_trip_through_bytes() {
    let (founding, genesis) = drawn().remove(0);
    let bytes = serde_json::to_vec(&genesis).unwrap();
    assert_eq!(serde_json::from_slice::<Genesis>(&bytes).unwrap(), genesis);
    let bytes = serde_json::to_vec(&founding).unwrap();
    assert_eq!(
        serde_json::from_slice::<Founding>(&bytes).unwrap(),
        founding
    );

    let view = View::of(&genesis).unwrap();
    let profile = view.edge_profile(0, 1).unwrap();
    let bytes = serde_json::to_vec(&profile).unwrap();
    assert_eq!(
        serde_json::from_slice::<EdgeProfile>(&bytes).unwrap(),
        profile
    );

    for flipped in [false, true] {
        let border = Border {
            side: 1,
            enters: 3,
            flipped,
        };
        let json = serde_json::to_string(&border).unwrap();
        assert_eq!(json.contains("flipped"), flipped);
        assert_eq!(serde_json::from_str::<Border>(&json).unwrap(), border);
    }
}
