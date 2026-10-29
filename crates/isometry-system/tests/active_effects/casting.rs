use super::super::*;

#[test]
fn nine_procedures_activate_cost_save_and_expire_without_reverting_later_edits() {
    let definitions = Definitions::builtin().unwrap();
    assert_eq!(definitions.0.len(), 9);
    for def in definitions.0.values() {
        let mut scene = Scene::new(def.edition, 73);
        let request = scene.request(&def.id, "first");
        let before = scene.map.clone();
        let receipt = scene.cast(&request, 0, true).unwrap();
        let id = receipt.effect.unwrap();
        let saved = serde_json::to_vec(&scene.map).unwrap();
        scene.map = serde_json::from_slice(&saved).unwrap();
        assert_eq!(serde_json::to_vec(&scene.map).unwrap(), saved);
        let records = Records::read(&scene.map).unwrap().unwrap();
        let active = &records.effects[&id];
        assert_eq!(active.definition, *def);
        assert_eq!(receipt.definition_revision, def.revision());
        assert!(active.applies(0));
        assert!(!receipt.costs.is_empty());
        assert_ne!(scene.map, before);
        scene
            .map
            .sheets
            .get_mut(&BEARER)
            .unwrap()
            .set_int("hp_current", 17);
        scene
            .map
            .sheets
            .get_mut(&BEARER)
            .unwrap()
            .set_int("str", 18);
        let inventory = scene.inventories.clone();
        let expires = active.expires;
        assert!(
            advance(&mut scene.map, clock(expires - 1))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            advance(&mut scene.map, clock(expires + 5)).unwrap(),
            vec![id]
        );
        assert_eq!(scene.sheet(BEARER).int("hp_current"), Some(17));
        assert_eq!(scene.sheet(BEARER).int("str"), Some(18));
        assert_eq!(scene.inventories, inventory);
        assert_eq!(
            Records::read(&scene.map).unwrap().unwrap().effects[&id].ended,
            Some((expires, End::Expired))
        );
        assert_eq!(scene.sheet(BEARER), *scene.map.sheet(BEARER).unwrap());
    }
}

#[test]
fn refusal_is_atomic_for_geometry_components_resources_and_request_conflicts() {
    let mut scene = Scene::new(Edition::Dnd5, 7);
    let request = scene.request("dnd5:enlarge-reduce", "one");
    let before = serde_json::to_vec(&scene.map).unwrap();
    let mut expected_rng = scene.rng.clone();
    let context = CastContext {
        clock: clock(0),
        feet_per_tile: 5,
        in_combat: false,
        line_of_effect: &|_, _| false,
    };
    assert!(
        cast(
            &mut scene.map,
            &scene.inventories,
            &scene.definitions,
            &request,
            &context,
            &mut scene.rng
        )
        .is_err()
    );
    assert_eq!(serde_json::to_vec(&scene.map).unwrap(), before);
    assert_eq!(scene.rng.die(1000), expected_rng.die(1000));
    scene
        .map
        .sheets
        .get_mut(&CASTER)
        .unwrap()
        .fields
        .insert("can_speak".into(), FieldValue::Bool(false));
    let before = scene.map.clone();
    assert!(
        scene
            .cast(&request, 0, true)
            .unwrap_err()
            .contains("can_speak")
    );
    assert_eq!(scene.map, before);
    scene
        .map
        .sheets
        .get_mut(&CASTER)
        .unwrap()
        .fields
        .insert("can_speak".into(), FieldValue::Bool(true));
    let receipt = scene.cast(&request, 0, true).unwrap();
    let after = scene.map.clone();
    assert_eq!(scene.cast(&request, 0, true).unwrap(), receipt);
    assert_eq!(scene.map, after);
    let mut conflicting = request.clone();
    conflicting.size_change = -1;
    assert!(
        scene
            .cast(&conflicting, 0, true)
            .unwrap_err()
            .contains("conflicts")
    );
    assert_eq!(scene.map, after);
}

#[test]
fn target_rules_saves_and_duration_distinctions_are_not_interchangeable() {
    let mut pf = Scene::new(Edition::Pathfinder2, 11);
    let mut sight = pf.request("pf2:darkvision", "sight");
    sight.target = Target::Creature(BEARER);
    assert!(pf.cast(&sight, 0, false).unwrap_err().contains("personal"));
    sight.rank = 3;
    let id = pf.cast(&sight, 0, false).unwrap().effect.unwrap();
    assert_eq!(
        Records::read(&pf.map).unwrap().unwrap().effects[&id].expires,
        3600
    );
    sight.request_id = "sight5".into();
    sight.rank = 5;
    let id = pf.cast(&sight, 0, false).unwrap().effect.unwrap();
    assert_eq!(
        Records::read(&pf.map).unwrap().unwrap().effects[&id].expires,
        86_400
    );

    let mut dnd = Scene::new(Edition::Dnd5, 12);
    let mut size = dnd.request("dnd5:enlarge-reduce", "save");
    size.willing = false;
    dnd.map
        .sheets
        .get_mut(&BEARER)
        .unwrap()
        .set_int("con_save_bonus", 100);
    let receipt = dnd.cast(&size, 0, false).unwrap();
    assert_eq!(receipt.effect, None);
    assert_eq!(receipt.dice.len(), 1);
    assert_eq!(dnd.map.sheet(CASTER).unwrap().int("slot:2"), Some(11));
    let mut ars = Scene::new(Edition::Ars5, 13);
    let spell = ars.request("ars5:eyes-of-the-cat", "sun");
    let id = ars.cast(&spell, 100, false).unwrap().effect.unwrap();
    assert_eq!(
        Records::read(&ars.map).unwrap().unwrap().effects[&id].expires,
        21_600
    );
    let context = CastContext {
        clock: Clock {
            next_sun: None,
            ..clock(100)
        },
        feet_per_tile: 5,
        in_combat: false,
        line_of_effect: &|_, _| true,
    };
    let mut other = spell.clone();
    other.request_id = "no-sun".into();
    assert!(
        cast(
            &mut ars.map,
            &ars.inventories,
            &ars.definitions,
            &other,
            &context,
            &mut ars.rng
        )
        .unwrap_err()
        .contains("Sun duration")
    );
}

#[test]
fn formulaic_totals_fatigue_and_failures_use_the_existing_dice_stream() {
    for (score, success, fatigue) in [(30, true, 0), (10, true, 1), (-30, false, 1)] {
        let mut scene = Scene::new(Edition::Ars5, 15);
        let caster = scene.map.sheets.get_mut(&CASTER).unwrap();
        caster.set_int("stamina", score);
        caster.set_int("muto", 0);
        caster.set_int("terram", 0);
        let request = scene.request("ars5:edge-of-the-razor", "formulaic");
        let mut rng = scene.rng.clone();
        let die = rng.die(10);
        let receipt = scene.cast(&request, 0, false).unwrap();
        assert_eq!(receipt.dice, vec![die as u16]);
        assert_eq!(receipt.casting_total, Some(score + i64::from(die)));
        assert_eq!(receipt.effect.is_some(), success);
        assert_eq!(
            scene.map.sheet(CASTER).unwrap().int("fatigue_lost"),
            Some(fatigue)
        );
        assert_eq!(scene.rng.die(100), rng.die(100));
    }
}

#[test]
fn spell_casting_shares_pf2_strikes_action_budget() {
    let mut scene = Scene::new(Edition::Pathfinder2, 71);
    scene
        .map
        .turn_counters
        .entry(CASTER)
        .or_default()
        .insert("actions_spent".into(), 2);
    let request = scene.request("pf2:darkvision", "no-budget");
    let before = scene.map.clone();
    assert!(
        scene
            .cast(&request, 0, true)
            .unwrap_err()
            .contains("not enough actions")
    );
    assert_eq!(scene.map, before);
    scene
        .map
        .turn_counters
        .get_mut(&CASTER)
        .unwrap()
        .insert("actions_spent".into(), 0);
    scene.cast(&request, 0, true).unwrap();
    assert_eq!(scene.map.turn_counters[&CASTER]["actions_spent"], 2);
    let mut system = pf2e_srd();
    let actor = sheet_with_turn_counters(
        scene.map.sheet(CASTER).unwrap(),
        scene.map.turn_counters[&CASTER].iter(),
    );
    let resolution = system
        .resolve_action(
            "strike",
            CASTER,
            &actor,
            (1, 2),
            BEARER,
            scene.map.sheet(BEARER).unwrap(),
            (2, 2),
            &mut scene.rng,
        )
        .unwrap();
    assert!(
        resolution
            .turn_counters
            .contains(&(CASTER, "actions_spent".into(), 1))
    );
}

#[test]
fn the_native_calendar_supplies_sun_duration_without_advancing_the_world() {
    let genesis = isocosm::Founding {
        seed: 10,
        sites: 1,
        population: 2,
        lineages: 1,
        cohort_size: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let sim = isocosm::Simulation::new(genesis, isocosm::Execution::Individuals).unwrap();
    let before = sim.state().clone();
    let calendar = Clock::from_native(&sim, None).unwrap();
    assert_eq!(calendar.tick, sim.state().tick);
    assert_eq!(
        calendar.tick_microseconds,
        sim.genesis().rules.tick_microseconds()
    );
    let boundary = calendar
        .next_sun
        .expect("ordinary sky has a solar crossing");
    let initial = isocosm::light::at(&sim.genesis().rules, calendar.tick, None)
        .sun
        .is_some();
    assert_ne!(
        isocosm::light::at(&sim.genesis().rules, boundary, None)
            .sun
            .is_some(),
        initial
    );
    assert_eq!(*sim.state(), before);
}
