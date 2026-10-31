// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Retains one native geometry entry across product publications.
use cosmograph::{operations::Window, schema::Id, simulation::Simulation, spatial::SiteGeometry};
use isometer_space::{places::{Places, Rules}, volume::Volume};
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct GeometryCache {
    current: Option<Arc<SiteGeometry>>,
    #[cfg(test)]
    builds: usize,
}
impl GeometryCache {
    #[cfg(test)]
    pub(in crate::spatial) fn builds(&self) -> usize { self.builds }

    pub(crate) fn get(&mut self, sim: &Simulation, site: Id, window: &Window, rules: Rules)
        -> Result<(&Volume, &Places), String> {
        if !self.current.as_ref().is_some_and(|entry| entry.matches(sim, site, window, rules)) {
            let _span = crate::profile::Span::new("geometry.build");
            self.current = Some(sim.site_geometry(site, window, rules)?);
            #[cfg(test)] { self.builds += 1; }
        }
        let geometry = self.current.as_ref().ok_or("native geometry cache was not populated")?;
        Ok((&geometry.volume, &geometry.places))
    }
}

#[cfg(test)]
pub(super) fn physical_test_session() -> cosmograph::Session {
    use cosmograph::{
        Execution, Founding,
        directing::Played,
        map::{Grid, Layout},
    };
    let seed = 420_889;
    let genesis = Founding {
        seed,
        sites: 6,
        population: 24,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies: Some(Default::default()),
        played: Some(Played {
            lineage: 1,
            region_sites: 1,
        }),
        map: Some(Layout::Grid(Grid {
            width: 3,
            height: 2,
            ..Grid::drawn(seed)
        })),
        ..Default::default()
    }
    .generate()
    .unwrap();
    cosmograph::Session::new(genesis, Execution::Individuals).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmograph::operations::Window;
    use isometer_space::{Edit, Op, Shape};

    #[test]
    fn exact_geometry_reuses_and_terrain_edits_rebuild_it() {
        let session = physical_test_session();
        {
            let templates = session
                .sim
                .genesis()
                .rules
                .spatial_templates
                .as_ref()
                .unwrap();
            let (site, stand) = templates
                .stands
                .iter()
                .find_map(|(&site, stands)| stands.first().map(|stand| (site, stand)))
                .unwrap();
            let window = Window {
                min: [stand.at[0] - 1, stand.at[2] - 1],
                max: [stand.at[0] + 1, stand.at[2] + 1],
                depth: templates.window.depth,
            };
            let mut cache = GeometryCache::default();
            let (cached_volume, cached_places) = cache
                .get(&session.sim, site, &window, Rules::default())
                .unwrap();
            let direct_volume = session
                .sim
                .volume(site, window.min, window.max, window.depth)
                .unwrap();
            let direct_places = Places::derive(&direct_volume, Rules::default()).unwrap();
            assert_eq!(cached_places, &direct_places);
            assert_eq!(
                (cached_volume.min, cached_volume.max, cached_volume.datum),
                (direct_volume.min, direct_volume.max, direct_volume.datum)
            );
            let materials = cached_volume.materials;
            let edited_cell = (window.min[0]..window.max[0])
                .find_map(|x| {
                    (window.min[1]..window.max[1]).find_map(|z| {
                        (cached_volume.datum..cached_volume.ceiling()).find_map(|y| {
                            let cell = [x, y, z];
                            let material = cached_volume.material(cell)?;
                            (material == materials.air || material == materials.water)
                                .then_some((cell, material))
                        })
                    })
                })
                .expect("generated stand window contains an editable clear cell");
            for x in window.min[0]..window.max[0] {
                for z in window.min[1]..window.max[1] {
                    for y in cached_volume.datum..cached_volume.ceiling() {
                        assert_eq!(
                            cached_volume.material([x, y, z]),
                            direct_volume.material([x, y, z])
                        );
                    }
                }
            }
            let _ = cache
                .get(&session.sim, site, &window, Rules::default())
                .unwrap();
            assert_eq!(
                cache.builds, 1,
                "same exact geometry reuses its transient entry"
            );

            let mut edited = session.sim.clone();
            let (cell, before) = edited_cell;
            assert!(before == materials.air || before == materials.water);
            edited
                .edit(
                    site,
                    Edit {
                        op: Op::Fill(materials.rock),
                        shape: Shape::Box {
                            min: cell,
                            max: [cell[0] + 1, cell[1] + 1, cell[2] + 1],
                        },
                    },
                )
                .unwrap();
            let (after_volume, after_places) =
                cache.get(&edited, site, &window, Rules::default()).unwrap();
            let direct_volume = edited
                .volume(site, window.min, window.max, window.depth)
                .unwrap();
            let direct_places = Places::derive(&direct_volume, Rules::default()).unwrap();
            assert_eq!(after_places, &direct_places);
            assert_eq!(
                (after_volume.min, after_volume.max, after_volume.datum),
                (direct_volume.min, direct_volume.max, direct_volume.datum)
            );
            assert_eq!(
                after_volume.material(cell),
                Some(materials.rock),
                "the edit changes held geometry"
            );
            assert_eq!(
                cache.builds, 2,
                "an admitted terrain edit changes the key and rebuilds geometry"
            );

            let mut state_only_cache = GeometryCache::default();
            let (_, baseline_places) = state_only_cache
                .get(&session.sim, site, &window, Rules::default())
                .unwrap();
            let patch = baseline_places.places.keys().next().copied().unwrap();
            let terrain = session.sim.terrain_revision();
            let mut state_only = session.clone();
            let actor = state_only
                .sim
                .state()
                .population
                .groups
                .iter()
                .find_map(|(&id, group)| {
                    (group.entity.body.is_some() && group.entity.resident_site() == Some(site))
                        .then_some(id)
                })
                .expect("generated edit site has an embodied resident");
            state_only
                .command(cosmograph::Command::Patch {
                    entity: actor,
                    patch: Some(patch),
                })
                .unwrap();
            assert_eq!(state_only.sim.terrain_revision(), terrain);
            let (state_volume, state_places) = state_only_cache
                .get(&state_only.sim, site, &window, Rules::default())
                .unwrap();
            let direct_volume = state_only
                .sim
                .volume(site, window.min, window.max, window.depth)
                .unwrap();
            assert_eq!(
                state_places,
                &Places::derive(&direct_volume, Rules::default()).unwrap()
            );
            assert_eq!(
                state_volume.material(cell),
                direct_volume.material(cell),
                "native population changes leave held terrain geometry intact"
            );
            assert_eq!(
                state_only_cache.builds, 1,
                "same-terrain state changes reuse geometry"
            );
        }
    }

    #[test]
    #[ignore = "seed-9 geometry timing; run explicitly after the F physical gate"]
    fn seed_nine_generated_member_order_geometry_workload() {
        use crate::{Founded, spatial::Game};
        use cosmograph::{Execution, Session};
        use std::hint::black_box;
        use std::time::Instant;

        let founded = Founded::generated(9, 256).unwrap();
        let mut founding = founded.genesis.founding.clone().unwrap();
        founding.ecology_rates = cosmograph::ecology::Rates::A1;
        let genesis = founding.generate().unwrap();
        let mut session = Session::new(genesis, Execution::Individuals).unwrap();
        let mut game = Game::default();
        assert!(game.found_generated(&mut session).unwrap());
        let members: Vec<_> = game.members.values().collect();

        let started = Instant::now();
        for held in &members {
            for _ in 0..2 {
                let volume = session
                    .sim
                    .volume(
                        held.site.0,
                        held.window.min,
                        held.window.max,
                        held.window.depth,
                    )
                    .unwrap();
                let places = Places::derive(&volume, held.places.unwrap_or_default()).unwrap();
                black_box((volume.datum, places.places.len()));
            }
        }
        let uncached = started.elapsed();

        let mut cache = GeometryCache::default();
        let started = Instant::now();
        for held in &members {
            for _ in 0..2 {
                let (volume, places) = cache
                    .get(
                        &session.sim,
                        held.site.0,
                        &held.window,
                        held.places.unwrap_or_default(),
                    )
                    .unwrap();
                black_box((volume.datum, places.places.len()));
            }
        }
        let cached = started.elapsed();
        println!(
            "seed9 generated members={} geometry_requests={} cache_builds={} uncached={:?} cached={:?}",
            members.len(),
            members.len() * 2,
            cache.builds,
            uncached,
            cached
        );
    }
}

#[cfg(test)]
mod shared_tests {
    use super::*;

    #[test]
    fn native_and_product_reads_share_exact_immutable_geometry_across_clones() {
        let session = super::physical_test_session();
        let t = session.sim.genesis().rules.spatial_templates.as_ref().unwrap();
        let (&site, stands) = t.stands.iter().find(|(_, v)| !v.is_empty()).unwrap();
        let at = stands[0].at;
        let window = Window { min: [at[0]-1, at[2]-1], max: [at[0]+1, at[2]+1], depth: t.window.depth };
        let native = session.sim.site_geometry(site, &window, t.places).unwrap();
        let volume = session.sim.site_volume(site, &window).unwrap();
        assert!(Arc::ptr_eq(&volume, &native.volume), "arrival searches share the native volume");
        let clone = session.sim.clone();
        assert!(Arc::ptr_eq(&native, &clone.site_geometry(site, &window, t.places).unwrap()));
        let replayed = session.fork_at(session.sim.state().tick, "branch:cache-test".into()).unwrap();
        assert!(Arc::ptr_eq(&native, &replayed.sim.site_geometry(site, &window, t.places).unwrap()),
            "history replay retains immutable terrain, not actor or admission results");
        let mut product = GeometryCache::default();
        product.get(&clone, site, &window, t.places).unwrap();
        assert!(Arc::ptr_eq(&native, product.current.as_ref().unwrap()));
        let mut changed = window.clone();
        changed.depth += 1;
        assert!(!Arc::ptr_eq(&native, &clone.site_geometry(site, &changed, t.places).unwrap()));
        // Whole-site play retains 64 local windows. Vary depth to exercise
        // eviction without assuming this fixture has room for 64 lateral shifts.
        for extra in 2..68 {
            let w = Window { depth:window.depth+extra,..window.clone() };
            clone.site_geometry(site, &w, t.places).unwrap();
        }
        let rebuilt = clone.site_geometry(site, &window, t.places).unwrap();
        assert!(!Arc::ptr_eq(&native, &rebuilt), "bounded native cache evicts older entries");
        assert!(native.volume.same_cells(&rebuilt.volume));
        assert_eq!(native.places, rebuilt.places);
    }
}
