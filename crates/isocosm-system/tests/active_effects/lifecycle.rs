use super::super::*;

#[test]
fn concentration_replacement_damage_and_incapacitation_end_real_effects() {
    let mut scene = Scene::new(Edition::Dnd5, 37);
    let size = scene.request("dnd5:enlarge-reduce", "size");
    let first = scene.cast(&size, 0, false).unwrap().effect.unwrap();
    let sword = scene.request("dnd5:magic-weapon", "sword");
    let next = scene.cast(&sword, 1, false).unwrap().effect.unwrap();
    let records = Records::read(&scene.map).unwrap().unwrap();
    assert_eq!(records.effects[&first].ended, Some((1, End::Concentration)));
    assert_eq!(scene.sheet(BEARER).int("size_category"), Some(2));
    scene
        .map
        .sheets
        .get_mut(&CASTER)
        .unwrap()
        .set_int("con_save_bonus", -100);
    let (total, dc) = concentration_damage(&mut scene.map, CASTER, 50, clock(2), &mut scene.rng)
        .unwrap()
        .unwrap();
    assert!(total < dc);
    assert_eq!(dc, 25);
    assert_eq!(
        weapon(&scene.map, &ItemId("demo:sword".into())).unwrap(),
        WeaponChange::default()
    );
    assert_eq!(
        Records::read(&scene.map).unwrap().unwrap().effects[&next].ended,
        Some((2, End::Concentration))
    );
    let mut size = size;
    size.request_id = "again".into();
    let third = scene.cast(&size, 3, false).unwrap().effect.unwrap();
    scene
        .map
        .conditions
        .entry(CASTER)
        .or_default()
        .insert("incapacitated".into(), 1);
    assert_eq!(advance(&mut scene.map, clock(4)).unwrap(), vec![third]);
    assert_eq!(
        Records::read(&scene.map).unwrap().unwrap().effects[&third].ended,
        Some((4, End::Incapacitated))
    );
}

#[test]
fn independent_suppression_and_dispel_do_not_erase_each_other_or_base_changes() {
    let mut scene = Scene::new(Edition::Pathfinder2, 19);
    let spell = scene.request("pf2:runic-weapon", "rune");
    let id = scene.cast(&spell, 0, false).unwrap().effect.unwrap();
    let sword = ItemId("demo:sword".into());
    assert_eq!(weapon(&scene.map, &sword).unwrap().damage_dice, 2);
    for source in ["ward:a", "ward:b"] {
        set_suppression(&mut scene.map, id, clock(1), source, true).unwrap();
    }
    set_suppression(&mut scene.map, id, clock(2), "ward:a", false).unwrap();
    assert!(!weapon(&scene.map, &sword).unwrap().magical);
    set_suppression(&mut scene.map, id, clock(3), "ward:b", false).unwrap();
    assert_eq!(weapon(&scene.map, &sword).unwrap().damage_dice, 2);
    interrupt(
        &mut scene.map,
        id,
        clock(4),
        End::Dispelled {
            ruling: "counteract success: effect 1".into(),
        },
    )
    .unwrap();
    set_suppression(&mut scene.map, id, clock(5), "ward:a", false).unwrap();
    assert!(!weapon(&scene.map, &sword).unwrap().magical);
    assert!(
        interrupt(
            &mut scene.map,
            id,
            clock(5),
            End::Dispelled {
                ruling: String::new()
            }
        )
        .is_err()
    );
}

#[test]
fn stronger_same_spell_yields_to_the_longer_weaker_one_after_expiry() {
    let mut scene = Scene::new(Edition::Pathfinder2, 23);
    let mut high = scene.request("pf2:runic-weapon", "high");
    high.rank = 9;
    scene.cast(&high, 0, false).unwrap();
    let low = scene.request("pf2:runic-weapon", "low");
    scene.cast(&low, 10, false).unwrap();
    let sword = ItemId("demo:sword".into());
    assert_eq!(weapon(&scene.map, &sword).unwrap().damage_dice, 4);
    advance(&mut scene.map, clock(60)).unwrap();
    assert_eq!(weapon(&scene.map, &sword).unwrap().damage_dice, 2);
    advance(&mut scene.map, clock(70)).unwrap();
    assert_eq!(weapon(&scene.map, &sword).unwrap(), WeaponChange::default());
}

#[test]
fn exact_replay_and_saved_definitions_do_not_follow_later_catalogue_edits() {
    let execute = || {
        let mut scene = Scene::new(Edition::Ars5, 97);
        for (n, id) in [
            "ars5:eyes-of-the-cat",
            "ars5:preternatural-growth",
            "ars5:edge-of-the-razor",
        ]
        .iter()
        .enumerate()
        {
            let request = scene.request(id, &n.to_string());
            scene.cast(&request, n as u64, false).unwrap();
        }
        scene
    };
    let mut first = execute();
    let second = execute();
    assert_eq!(first.map, second.map);
    first
        .definitions
        .0
        .get_mut("ars5:eyes-of-the-cat")
        .unwrap()
        .duration = Duration::Seconds(1);
    let encoded = serde_json::to_vec(&first.map).unwrap();
    let loaded: MapDocument = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        Records::read(&loaded).unwrap(),
        Records::read(&second.map).unwrap()
    );
    assert!(
        Records::at(&loaded, clock(1))
            .unwrap_err()
            .contains("backwards")
    );
    let bad_clock = Clock {
        tick_microseconds: 6_000_000,
        ..clock(2)
    };
    assert!(
        Records::at(&loaded, bad_clock)
            .unwrap_err()
            .contains("units")
    );
    assert!(clock(0).seconds(60).is_ok());
    assert!(
        Clock {
            tick_microseconds: 7_000_000,
            ..clock(0)
        }
        .seconds(60)
        .is_err()
    );
}
