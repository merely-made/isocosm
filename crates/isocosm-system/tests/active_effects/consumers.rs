use super::super::*;

#[test]
fn granted_senses_change_actual_height_traced_visibility_and_respect_blindness() {
    let sight = HeightSightRules {
        radius: 18,
        eye_height: 1,
        obstacle_height: &|kind| if kind == "wall" { 10 } else { 0 },
    };
    for (edition, id, absolute) in [
        (Edition::Dnd5, "dnd5:darkvision", true),
        (Edition::Pathfinder2, "pf2:darkvision", true),
        (Edition::Ars5, "ars5:eyes-of-the-cat", false),
    ] {
        let mut scene = Scene::new(edition, 73);
        let request = scene.request(id, "vision");
        let observer = match request.target {
            Target::Creature(id) => id,
            _ => unreachable!(),
        };
        assert!(
            visible_tiles(&scene.map, observer, None, &sight, 5, &|_| {
                Lighting::NearDark
            })
            .unwrap()
            .is_empty()
        );
        scene.cast(&request, 0, false).unwrap();
        let near = visible_tiles(&scene.map, observer, None, &sight, 5, &|_| {
            Lighting::NearDark
        })
        .unwrap();
        assert!(near.contains(&(4, 2)));
        let dark = visible_tiles(&scene.map, observer, None, &sight, 5, &|_| {
            Lighting::AbsoluteDark
        })
        .unwrap();
        assert_eq!(dark.contains(&(4, 2)), absolute);
        if edition == Edition::Dnd5 {
            assert!(
                !near.contains(&(19, 2)),
                "the granted sense ends at sixty feet"
            );
        }
        scene.map.tile_kinds.push("wall".into());
        for row in 0..5 {
            scene.map.props.set(3, row, isocosm_core::TileKindId(1));
        }
        assert!(
            !visible_tiles(&scene.map, observer, None, &sight, 5, &|_| {
                Lighting::NearDark
            })
            .unwrap()
            .contains(&(4, 2))
        );
        scene
            .map
            .conditions
            .entry(observer)
            .or_default()
            .insert("blinded".into(), 1);
        assert!(
            visible_tiles(&scene.map, observer, None, &sight, 5, &|_| Lighting::Bright)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn size_projects_the_existing_graph_without_readdressing_parts_or_rounding_joints() {
    use isometer_core::{Attachment, VolumeRef, Yaw};
    let mut base = body();
    let root = base.root;
    let child = base
        .attach(
            VolumeRef::from_tag(2),
            [2, 4, 2],
            Attachment {
                parent: root,
                offset: [6, 0, 0],
                yaw: Yaw::Quarter,
            },
            None,
        )
        .unwrap();
    base.parts[child.0 as usize].origin = Some(44);
    let before = base.clone();
    let mut scene = Scene::new(Edition::Dnd5, 73);
    let request = scene.request("dnd5:enlarge-reduce", "grow");
    scene.cast(&request, 0, false).unwrap();
    let grown = project_body(&scene.map, BEARER, &base, None).unwrap();
    assert_eq!(grown.parts[0].half_extent, [8, 16, 8]);
    assert_eq!(grown.world_pivot(child), Some([12, 0, 0]));
    assert_eq!(grown.parts[1].origin, Some(44));
    assert_eq!(grown.parts[1].id, child);
    assert_eq!(scene.sheet(BEARER).int("weight_mg"), Some(512_000_000));
    assert_eq!(base, before);
    let mut reduce = request;
    reduce.request_id = "shrink".into();
    reduce.size_change = -1;
    scene.cast(&reduce, 1, false).unwrap();
    assert_eq!(
        project_body(&scene.map, BEARER, &base, None)
            .unwrap()
            .world_pivot(child),
        Some([3, 0, 0])
    );
    base.parts[1].attachment.as_mut().unwrap().offset[0] = 7;
    assert!(
        project_body(&scene.map, BEARER, &base, None)
            .unwrap_err()
            .contains("finer geometry")
    );

    let mut ars = Scene::new(Edition::Ars5, 73);
    let mut request = ars.request("ars5:preternatural-growth", "shrink2");
    request.size_change = -2;
    ars.cast(&request, 0, false).unwrap();
    assert_eq!(ars.sheet(BEARER).int("ars_size"), Some(-2));
    assert!(
        project_body(&ars.map, BEARER, &body(), None)
            .unwrap_err()
            .contains("authored physical scale")
    );
    assert_eq!(
        project_body(
            &ars.map,
            BEARER,
            &body(),
            Some(BodyChange {
                numerator: 1,
                denominator: 2
            })
        )
        .unwrap()
        .parts[0]
            .half_extent,
        [2, 4, 2]
    );
}

fn attack(
    system: &mut System,
    scene: &Scene,
    key: &str,
    seed: u64,
    distance: i32,
) -> isocosm_system::Resolution {
    let actor = system
        .effective_map_sheet(
            &scene.map,
            BEARER,
            scene.map.sheet(BEARER).unwrap(),
            scene.inventories.get(&BEARER),
        )
        .unwrap();
    let mut target = system.default_sheet();
    target.set_int("ac", 1);
    system
        .resolve_action(
            key,
            BEARER,
            &actor,
            (2, 2),
            TokenId(9),
            &target,
            (2 + distance, 2),
            &mut Rng::new(seed),
        )
        .unwrap()
}

#[test]
fn weapon_enhancement_and_size_are_consumed_by_the_existing_attack_resolver() {
    let mut dnd = Scene::new(Edition::Dnd5, 73);
    let mut system = srd_5e();
    let seed = (1..100)
        .find(|seed| {
            let r = attack(&mut system, &dnd, "attack", *seed, 1);
            r.hit && r.degree == 1
        })
        .unwrap();
    let ordinary = attack(&mut system, &dnd, "attack", seed, 1);
    let mut request = dnd.request("dnd5:magic-weapon", "magic");
    request.rank = 6;
    dnd.cast(&request, 0, false).unwrap();
    let magical = attack(&mut system, &dnd, "attack", seed, 1);
    assert_eq!(magical.attack.total, ordinary.attack.total + 3);
    assert_eq!(
        magical.damage.unwrap().total,
        ordinary.damage.unwrap().total + 3
    );
    advance(&mut dnd.map, clock(3600)).unwrap();
    let size = dnd.request("dnd5:enlarge-reduce", "grow");
    dnd.cast(&size, 3600, false).unwrap();
    let big = attack(&mut system, &dnd, "attack", seed, 1).damage.unwrap();
    assert!(big.expr.contains("+1d4"));
    assert_eq!(big.dice.len(), 2);
    let mut size = size;
    size.request_id = "small".into();
    size.size_change = -1;
    dnd.cast(&size, 3601, false).unwrap();
    let small = attack(&mut system, &dnd, "attack", seed, 1).damage.unwrap();
    assert!(small.expr.contains("-1d4"));
    assert!(small.total >= 1);

    let mut pf = Scene::new(Edition::Pathfinder2, 73);
    let mut system = pf2e_srd();
    let mut rune = pf.request("pf2:runic-weapon", "runes");
    rune.rank = 9;
    pf.cast(&rune, 0, false).unwrap();
    let result = attack(&mut system, &pf, "strike", seed, 1);
    assert!(result.damage.unwrap().expr.contains("4d8"));
    let mut enlarge = pf.request("pf2:enlarge", "large");
    enlarge.rank = 4;
    pf.cast(&enlarge, 1, false).unwrap();
    let sheet = pf.sheet(BEARER);
    assert_eq!(sheet.int("reach_ft"), Some(15));
    assert_eq!(sheet.int("clumsy"), Some(1));
    assert_eq!(sheet.int("melee_damage_status_bonus"), Some(4));
    assert!(attack(&mut system, &pf, "strike", seed, 3).hit);
    advance(&mut pf.map, clock(60)).unwrap();
    let after_rune = attack(&mut system, &pf, "strike", seed, 3).damage.unwrap();
    assert!(after_rune.expr.contains("1d8"));
    assert!(!after_rune.expr.contains("4d8"));
    advance(&mut pf.map, clock(301)).unwrap();
    let actor = pf.sheet(BEARER);
    assert!(
        system
            .resolve_action(
                "strike",
                BEARER,
                &actor,
                (2, 2),
                CASTER,
                pf.map.sheet(CASTER).unwrap(),
                (5, 2),
                &mut pf.rng
            )
            .is_err()
    );
}

#[test]
fn equipment_stacks_by_the_spell_rule_and_stays_with_the_item_identity() {
    let sword = ItemId("demo:sword".into());
    let mut pf = Scene::new(Edition::Pathfinder2, 73);
    pf.inventories
        .get_mut(&BEARER)
        .unwrap()
        .items
        .get_mut(&sword)
        .unwrap()
        .modifiers
        .push(ItemModifier {
            id: "existing:rune".into(),
            kind: ItemModifierKind::Enchantment,
            name: "Potency +3".into(),
            stats: [("attack_bonus".into(), 3)].into(),
            appearance_layer: None,
        });
    let request = pf.request("pf2:runic-weapon", "weaker");
    pf.cast(&request, 0, false).unwrap();
    let system = pf2e_srd();
    let effective = system
        .effective_map_sheet(
            &pf.map,
            BEARER,
            pf.map.sheet(BEARER).unwrap(),
            pf.inventories.get(&BEARER),
        )
        .unwrap();
    assert_eq!(effective.int("attack_bonus"), Some(3));
    assert_eq!(effective.int("effect_weapon_dice"), Some(2));
    let inventory = pf.inventories.get_mut(&BEARER).unwrap();
    inventory.equipped.remove(&EquipmentSlot::MainHand);
    assert_eq!(pf.sheet(BEARER).int("effect_weapon_dice"), None);
    assert_eq!(weapon(&pf.map, &sword).unwrap().damage_dice, 2);

    let mut ars = Scene::new(Edition::Ars5, 73);
    ars.inventories
        .get_mut(&BEARER)
        .unwrap()
        .items
        .get_mut(&sword)
        .unwrap()
        .modifiers
        .push(ItemModifier {
            id: "quality".into(),
            kind: ItemModifierKind::Quality,
            name: "Fine".into(),
            stats: [("damage_bonus".into(), 1)].into(),
            appearance_layer: None,
        });
    let spell = ars.request("ars5:edge-of-the-razor", "edge");
    ars.cast(&spell, 0, false).unwrap();
    let change = weapon(&ars.map, &sword).unwrap();
    assert_eq!(change.damage_bonus, 2);
    assert!(change.resisted_by_magic);
    assert!(change.penetration.is_some());
    assert_eq!(ars.sheet(BEARER).int("effect_weapon_damage"), Some(2));
}

#[test]
fn a_dnd_critical_rolls_twice_and_applies_magic_weapons_flat_bonus_once() {
    let mut scene = Scene::new(Edition::Dnd5, 73);
    let request = scene.request("dnd5:magic-weapon", "critical");
    scene.cast(&request, 0, false).unwrap();
    let seed = (1..1000)
        .find(|seed| Rng::new(*seed).die(20) == 20)
        .unwrap();
    let resolution = attack(&mut srd_5e(), &scene, "attack", seed, 1);
    assert_eq!(resolution.degree, 2);
    let damage = resolution.damage.unwrap();
    assert_eq!(damage.dice.len(), 2);
    assert_eq!(
        damage.total,
        damage.dice.iter().map(|die| i32::from(*die)).sum::<i32>() + 1
    );
}
