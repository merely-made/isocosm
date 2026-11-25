// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use mesocosm_runtime::{Runtime, continuation::LoopPhase};
use std::collections::BTreeSet;

#[test]
fn same_projection_lives_away_from_view_and_reopens_at_selected_site() {
    let dir = tempfile::tempdir().unwrap();
    for seed in [9, 104] {
        let draft = Panel::starter(seed);
        let mut projection = Panel::new(draft.clone()).unwrap();
        let mut reference = Session::new(draft.generate().unwrap(), Execution::Grouped).unwrap();
        let at_site = |s: &Session, site| {
            s.sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(_, g)| g.entity.resident_site() == Some(site))
                .map(|(id, g)| (*id, g.clone()))
                .collect::<Vec<_>>()
        };
        let away=reference.sim.state().population.groups.values()
            .find_map(|g|g.entity.body.as_ref().and_then(|_|g.entity.resident_site()).filter(|&id|id!=1))
            .expect("a founded body outside the watched site");
        let original = at_site(&reference, away);
        projection.site = 1;
        projection.seek(12).unwrap();
        reference.advance(12).unwrap();
        assert_ne!(at_site(&reference, away), original);
        assert_eq!(at_site(&projection.session, away), at_site(&reference, away));
        assert_eq!(
            projection.session.sim.state_hash(),
            reference.sim.state_hash()
        );
        let transfers = projection.transfers.clone();
        projection.seek(4).unwrap();
        projection.seek(12).unwrap();
        assert_eq!(projection.transfers, transfers);
        assert_eq!(
            projection.session.sim.state_hash(),
            reference.sim.state_hash()
        );

        let folder = Panel::save(&projection.session, dir.path()).unwrap();
        let entry = folder.join("entry.json");
        let saved = std::fs::read(&entry).unwrap();
        let runtime = Runtime::enter_founding_at(&saved, 4, None, Some(1)).unwrap();
        assert_eq!(runtime.current_loop_phase(), Some(LoopPhase::Watching));
        assert_eq!(runtime.view().sites, BTreeSet::from([1]));
        assert_eq!(runtime.tick(), 12);
        assert_eq!(runtime.save().entries.len(), reference.entries.len() + 1);
        let reopened = Runtime::resume_game_with_mode(runtime.save_game(), 4, None).unwrap();
        assert_eq!(reopened.view(), runtime.view());
        assert_eq!(reopened.sim().state_hash(), runtime.sim().state_hash());

        projection.path = TextInput::new(entry.to_string_lossy());
        projection.reopen().unwrap();
        assert_eq!(projection.transfers, transfers);
        assert_eq!(
            projection.session.sim.state_hash(),
            reference.sim.state_hash()
        );
        let archived: Founding =
            toml::from_str(&std::fs::read_to_string(folder.join("founding.toml")).unwrap())
                .unwrap();
        assert_eq!(archived, draft);
    }
}

#[test]
fn independent_body_maps_are_authored_imported_and_expressed() {
    let dir = tempfile::tempdir().unwrap();
    let authored:Founding=toml::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../shared/cosmograph/examples/optional-gland.toml"))).unwrap();
    let mut p=Panel::new(Founding {seed:12,..authored}).unwrap();
    let initial = p.session.sim.state_hash();
    let first = p.map().unwrap().clone();
    p.run = 1;
    p.bind("kind:frond".into());
    p.repeat(true);
    p.cohort = field(1);
    p.population = field(48);
    p.seed = field(104);
    p.project().unwrap();
    assert_ne!(p.session.sim.state_hash(), initial);
    assert_eq!(p.session.sim.genesis().seed, 104);
    assert_eq!(
        p.session
            .sim
            .genesis()
            .founding
            .as_ref()
            .unwrap()
            .cohort_size,
        1
    );
    assert_eq!(p.map().unwrap().regions["region:gland"], "kind:frond");
    let new_map = p.map().unwrap().clone();
    assert_ne!(new_map, first);
    let folder = Panel::save(&p.session, dir.path()).unwrap();
    p.lineage = "lineage:2".into();
    p.path = TextInput::new(folder.join("lineage-0.toml").to_string_lossy());
    p.load_map().unwrap();
    assert_eq!(p.map().unwrap(), &new_map);
    let i = p.map_mut().branch_from(0).unwrap();
    assert_eq!(p.map().unwrap().grammar.seed[i].parent, Some(0));
    p.project().unwrap();
    let sim = &p.session.sim;
    assert!(
        sim.state()
            .population
            .groups
            .values()
            .any(|g| g.entity.lineage == "lineage:2" && g.entity.body.is_some())
    );
    assert_eq!(sim.genesis().founding.as_ref().unwrap().body_maps.len(), 2);
    let hash = sim.state_hash();
    p.cohort = field(0);
    assert!(p.project().is_err());
    assert_eq!(p.session.sim.state_hash(), hash);
}

#[test]
fn comparing_edits_keeps_the_original_at_the_displayed_tick() {
    let base = tempfile::tempdir().unwrap();
    let mut panel = Panel::new(Panel::starter(104)).unwrap();
    panel.seek(12).unwrap();
    panel.site = 1;
    let unchanged = panel.session.clone();
    panel.repeat(true);
    panel.compare_edit().unwrap();
    let edited_hash = panel.session.sim.state_hash();
    assert_ne!(unchanged.sim.state_hash(), edited_hash);
    assert_eq!(
        panel.original.as_ref().unwrap().sim.state_hash(),
        unchanged.sim.state_hash()
    );
    assert_eq!(panel.site, 1);
    panel.seek(0).unwrap();
    panel.seek(12).unwrap();
    assert_eq!(
        panel.original.as_ref().unwrap().sim.state_hash(),
        unchanged.sim.state_hash()
    );
    assert_eq!(panel.session.sim.state_hash(), edited_hash);

    panel.next_preset();
    panel.compare_edit().unwrap();
    assert_ne!(panel.session.sim.state_hash(), edited_hash);
    let mut reference = unchanged.clone();
    reference.advance(6).unwrap();
    panel.seek(18).unwrap();
    assert_eq!(
        panel.original.as_ref().unwrap().sim.state_hash(),
        reference.sim.state_hash()
    );
    let mut edited = Session::new(
        panel.recipe().unwrap().generate().unwrap(),
        Execution::Grouped,
    )
    .unwrap();
    edited.advance(18).unwrap();
    assert_eq!(panel.session.sim.state_hash(), edited.sim.state_hash());
    for world in [panel.original.as_ref().unwrap(), &panel.session] {
        let dir = Panel::save(world, base.path()).unwrap();
        let bytes = std::fs::read(dir.join("entry.json")).unwrap();
        let saved = Session::load_json(&bytes, Execution::Grouped).unwrap();
        assert_eq!(saved.sim.state_hash(), world.sim.state_hash());
        let entered = Runtime::enter_founding_at(&bytes, 4, None, Some(panel.site)).unwrap();
        assert_eq!(entered.tick(), 18);
        assert_eq!(entered.view().sites, BTreeSet::from([1]));
    }
    panel.cohort = field(0);
    assert!(panel.compare_edit().is_err());
    assert_eq!(
        panel.original.as_ref().unwrap().sim.state_hash(),
        reference.sim.state_hash()
    );
    assert_eq!(panel.session.sim.state_hash(), edited.sim.state_hash());
}

#[test]
fn watched_site_pages_the_whole_ground_and_keeps_representatives_on_it() {
    let mut draft = Panel::starter(9);
    let Some(cosmograph::map::Layout::Grid(ref mut grid)) = draft.map else { unreachable!() };
    grid.side = 512;
    let mut panel = Panel::new(draft).unwrap();
    panel.seek(12).unwrap();
    let mut representatives = BTreeMap::new();
    for group in panel.session.sim.state().population.groups.values() {
        let body = &group.entity;
        if body.alive && body.body.is_some() && let Some(site) = body.resident_site() {
            *representatives.entry(site).or_insert(0usize) += 1;
        }
    }
    let (&site, _) = representatives.iter().max_by_key(|(_, n)| *n).expect("populated site");
    let saved = serde_json::to_vec(&panel.session.save()).unwrap();
    let runtime = Runtime::enter_founding_at(&saved, 4, None, Some(site)).unwrap();
    let sim = runtime.sim();
    let before = sim.state_hash();
    let scene = crate::played::scene::watched_site(&sim, site, None, &runtime.view()).unwrap();
    assert!(scene.window.is_paged());
    assert_eq!((scene.window.min, scene.window.max), ([0; 2], [512; 2]));
    assert!(scene.ground.keys().next().is_none());
    assert!(scene.bodies.len() > 1);
    let mut footprints = Vec::new();
    for body in &scene.bodies {
        let bounds = body.document.aabb();
        let [x, _, z] = body.at;
        let rect = [x + bounds.min[0], z + bounds.min[2], x + bounds.max[0], z + bounds.max[2]];
        assert!(rect[0] >= -256 && rect[1] >= -256 && rect[2] <= 256 && rect[3] <= 256);
        assert!(footprints.iter().all(|r: &[i32; 4]| rect[2] <= r[0] || r[2] <= rect[0]
            || rect[3] <= r[1] || r[3] <= rect[1]));
        footprints.push(rect);
    }
    assert_eq!(sim.state_hash(), before);
}
