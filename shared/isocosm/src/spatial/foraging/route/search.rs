// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Bounded multi-goal search over native grounded poses. Expensive full swings
//! are checked on candidate paths, once per edge, before declaring reachability.
use super::*;
type Node = ([i64; 3], Yaw);
struct Vertex {
    pose: Pose,
    neighbours: Option<Vec<Node>>,
    distance: u64,
    food: Option<Id>,
}

pub(super) fn find(input: &Input) -> Result<RouteSearch> {
    let origin = (input.before.at, input.before.yaw);
    let mut graph = BTreeMap::from([(origin, vertex(input, input.before.clone())?)]);
    if graph[&origin].food.is_some() {
        return Ok(RouteSearch {
            nodes: 1,
            ..Default::default()
        });
    }
    let local = embodiment::boxes(
        &input.document,
        &Pose {
            at: [0; 3],
            yaw: input.before.yaw,
            feet: Default::default(),
        },
    )?;
    let mut grounding = embodiment::Grounding::new(&input.terrain.volume);
    let mut rejected = BTreeSet::new();
    let mut checked = BTreeSet::new();
    let mut report = RouteSearch::default();
    loop {
        let path = trial(
            input,
            origin,
            &mut graph,
            &mut grounding,
            &local,
            &rejected,
            &mut report.exhausted,
        )?;
        let Some(path) = path else {
            break;
        };
        let mut valid = true;
        for edge in path.windows(2) {
            let pair = (edge[0], edge[1]);
            if checked.contains(&pair) {
                continue;
            }
            report.checked_edges += 1;
            if let Err(why) = embodiment::sweep_clear(
                &input.terrain.volume,
                &input.document,
                &graph[&pair.0].pose,
                &graph[&pair.1].pose,
                &input.blocked,
            ) {
                rejected.insert(pair);
                report.refused_edges += 1;
                report.first_refusal.get_or_insert(why);
                valid = false;
                break;
            }
            checked.insert(pair);
        }
        if valid {
            report.route = Some(FoodRoute {
                target: graph[path.last().unwrap()].food.unwrap(),
                path: path.iter().map(|key| graph[key].pose.clone()).collect(),
            });
            break;
        }
    }
    report.nodes = graph.len();
    Ok(report)
}

fn vertex(input: &Input, pose: Pose) -> Result<Vertex> {
    let boxes = embodiment::boxes(&input.document, &pose)?;
    let distance = input
        .foods
        .iter()
        .map(|(_, food)| gap(&boxes, food))
        .min()
        .unwrap_or(u64::MAX);
    let food = input
        .foods
        .iter()
        .find(|(_, food)| gap(&boxes, food) <= input.reach)
        .map(|(id, _)| *id);
    Ok(Vertex {
        pose,
        neighbours: None,
        distance,
        food,
    })
}

fn trial(
    input: &Input,
    origin: Node,
    graph: &mut BTreeMap<Node, Vertex>,
    grounding: &mut embodiment::Grounding<'_>,
    local: &[embodiment::Box3],
    rejected: &BTreeSet<(Node, Node)>,
    exhausted: &mut bool,
) -> Result<Option<Vec<Node>>> {
    let mut costs = BTreeMap::from([(origin, 0u64)]);
    let mut parents = BTreeMap::new();
    let mut queue = BTreeSet::from([(
        graph[&origin].distance.saturating_sub(input.reach),
        0u64,
        origin,
    )]);
    while let Some((_, cost, key)) = queue.pop_first() {
        if costs.get(&key) != Some(&cost) {
            continue;
        }
        if graph[&key].food.is_some() {
            let mut path = vec![key];
            while let Some(parent) = parents.get(path.last().unwrap()) {
                path.push(*parent);
            }
            path.reverse();
            return Ok(Some(path));
        }
        if graph[&key].neighbours.is_none() {
            let pose = &graph[&key].pose;
            let candidates = if !pose.feet.is_empty() {
                embodiment::articulated_neighbours(
                    &input.terrain.places,
                    &input.profile,
                    &input.document,
                    pose,
                    grounding,
                )
            } else {
                embodiment::neighbours_from_boxes(
                    &input.terrain.places,
                    &input.profile,
                    local,
                    pose,
                    grounding,
                )
            };
            let mut neighbours = Vec::new();
            for next in candidates {
                let to = (next.at, next.yaw);
                if !graph.contains_key(&to) {
                    if graph.len() >= input.nodes {
                        *exhausted = true;
                        continue;
                    }
                    let boxes = embodiment::boxes(&input.document, &next)?;
                    if embodiment::intersects(&boxes, &input.blocked) {
                        continue;
                    }
                    graph.insert(to, vertex(input, next)?);
                }
                neighbours.push(to);
            }
            graph.get_mut(&key).unwrap().neighbours = Some(neighbours);
        }
        for &to in graph[&key].neighbours.as_ref().unwrap() {
            if rejected.contains(&(key, to)) {
                continue;
            }
            let next_cost = cost + 1;
            if costs.get(&to).is_some_and(|old| *old <= next_cost) {
                continue;
            }
            costs.insert(to, next_cost);
            parents.insert(to, key);
            let estimate =
                next_cost.saturating_add(graph[&to].distance.saturating_sub(input.reach));
            queue.insert((estimate, next_cost, to));
        }
    }
    Ok(None)
}
