// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::spatial::{embodiment::articulation_tests::quadruped, embodiment::tests::terrain};

fn cells(boxes: &[embodiment::Box3]) -> Vec<embodiment::Box3> {
    let mut out = Vec::new();
    for [lo, hi] in boxes {
        for x in lo[0]..hi[0] {
            for y in lo[1]..hi[1] {
                for z in lo[2]..hi[2] {
                    out.push([[x, y, z], [x + 1, y + 1, z + 1]]);
                }
            }
        }
    }
    out
}
pub(in crate::spatial::foraging::route) fn control_input(sim: &Simulation) -> Input {
    let doc = quadruped();
    let rules = embodiment::Walking::default();
    let terrain = sim
        .site_geometry(
            0,
            &crate::operations::Window {
                min: [350; 2],
                max: [418; 2],
                depth: 16,
            },
            Default::default(),
        )
        .unwrap();
    let mut g = embodiment::Grounding::new(&terrain.volume)
        .with_articulation(true)
        .with_walking(Some(rules));
    let from = g.at(&doc, 384, 384, Yaw::Zero).unwrap();
    let source = embodiment::boxes(&doc, &from).unwrap();
    let to = g.at(&doc, 383, 384, Yaw::Quarter).unwrap();
    let straight = g.at(&doc, 383, 384, Yaw::Zero).unwrap();
    let stride = embodiment::plan_stride(&terrain.volume, &doc, &from, &to, rules, &[]).unwrap();
    let end = embodiment::boxes(&doc, &to).unwrap();
    let swept: Vec<_> = (0..=stride.samples())
        .flat_map(|i| {
            embodiment::stride_boxes(&doc, &from, &to, &stride, i * 1024 / stride.samples())
                .unwrap()
        })
        .collect();
    let obstacle = cells(&embodiment::boxes(&doc, &straight).unwrap())
        .into_iter()
        .find(|b| !embodiment::intersects(&[*b], &swept))
        .expect("unturned and turning bodies must occupy different cells");
    assert!(!embodiment::intersects(&[obstacle], &source));
    let lo = [0, 1, 2].map(|k| end.iter().map(|b| b[0][k]).min().unwrap() - 1);
    let hi = [0, 1, 2].map(|k| end.iter().map(|b| b[1][k]).max().unwrap() + 1);
    let food = cells(&[[lo, hi]])
        .into_iter()
        .find(|b| {
            gap(&source, &[*b]) > 0
                && gap(&end, &[*b]) == 0
                && !embodiment::intersects(&[*b], &end)
                && !embodiment::intersects(&[*b], &swept)
        })
        .unwrap();
    Input {
        actor: 1,
        site: 0,
        terrain: terrain.clone(),
        document: doc.clone(),
        before: from.clone(),
        profile: StepProfile {
            process: "walk".into(),
            cells: crate::rules::Amount::Fixed(64),
            places: Default::default(),
            body: isometer_space::places::Body {
                width: 1,
                height: 1,
                climb: 1,
                wades: false,
            },
            stance_offset: [0; 3],
        },
        foods: vec![(2, vec![food])],
        blocked: vec![obstacle, food],
        reach: 0,
        nodes: 256,
        walking: Some(rules),
    }
}

#[test]
fn food_route_turns_past_an_obstacle_that_blocks_the_unturned_body() {
    let sim = terrain();
    let input = control_input(&sim);
    let straight = embodiment::Grounding::new(&input.terrain.volume)
        .with_articulation(true)
        .with_walking(input.walking)
        .at(&input.document, 383, 384, Yaw::Zero)
        .unwrap();
    assert!(embodiment::intersects(
        &embodiment::boxes(&input.document, &straight).unwrap(),
        &input.blocked
    ));
    let report = find(&input).unwrap();
    let route = report
        .route
        .expect("paid pivot must provide a food route around the obstacle");
    assert_eq!(route.target, 2);
    assert!(route.strides.iter().flatten().any(|s| s.turn != 0));
    for (i, pair) in route.path.windows(2).enumerate() {
        embodiment::check_walk(
            &input.terrain.volume,
            &input.document,
            &pair[0],
            &pair[1],
            input.walking,
            route.strides[i].as_ref(),
            &input.blocked,
        )
        .unwrap();
    }
}
