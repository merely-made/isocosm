// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Bounded multi-goal search over native grounded poses. Expensive full swings
//! are checked on candidate paths, once per edge, before declaring reachability.
use super::*;
#[cfg(test)]
mod cold_tests;
#[cfg(test)]
pub(super) mod turn_tests;
type Node = ([i64; 3], Yaw, [u8; 32]);
fn node(input: &Input, pose: &Pose) -> Node {
    key(
        pose,
        input.walking.is_some_and(|r| r.stances > 0 || r.turns),
    )
}
fn key(pose: &Pose, stances: bool) -> Node {
    use sha2::{Digest, Sha256};
    let mut layout = [0; 32];
    if stances {
        let mut hash = Sha256::new();
        for (id, p) in &pose.feet {
            hash.update(id.0.to_le_bytes());
            for v in p {
                hash.update(v.to_le_bytes());
            }
        }
        layout = hash.finalize().into();
    }
    (pose.at, pose.yaw, layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinct_stances_at_one_root_are_distinct_routes_and_old_rules_keep_the_old_key() {
        let mut a = Pose::default();
        a.feet.insert(isometer_core::PartId(1), [0; 3]);
        let mut b = a.clone();
        b.feet.get_mut(&isometer_core::PartId(1)).unwrap()[0] = 1024;
        assert_ne!(key(&a, true), key(&b, true));
        assert_eq!(key(&a, false), key(&b, false));
        assert_eq!(key(&a, false).2, [0; 32]);
    }
}
struct Vertex {
    pose: Pose,
    neighbours: Option<Vec<Node>>,
    distance: u64,
    food: Option<Id>,
}

pub(super) fn find(input: &Input) -> Result<RouteSearch> {
    let mut strides =
        embodiment::StrideChecks::new(&input.terrain.volume, &input.document, &input.blocked);
    find_using(input, &mut strides)
}
fn find_using(input: &Input, strides: &mut embodiment::StrideChecks<'_>) -> Result<RouteSearch> {
    let origin = node(input, &input.before);
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
    let mut grounding =
        embodiment::Grounding::new(&input.terrain.volume).with_walking(input.walking);
    let mut rejected = BTreeSet::new();
    let mut checked = BTreeMap::new();
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
            if checked.contains_key(&pair) {
                continue;
            }
            report.checked_edges += 1;
            let plan = strides.step(&graph[&pair.0].pose, &graph[&pair.1].pose, input.walking);
            if let Err(why) = &plan {
                rejected.insert(pair);
                report.refused_edges += 1;
                report.first_refusal.get_or_insert_with(|| why.clone());
                valid = false;
                break;
            }
            checked.insert(pair, plan.unwrap());
        }
        if valid {
            report.route = Some(FoodRoute {
                strides: if input.walking.is_some() {
                    path.windows(2)
                        .map(|e| checked[&(e[0], e[1])].clone())
                        .collect()
                } else {
                    Vec::new()
                },
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
    let mut costs = BTreeMap::from([(origin, (0u64, 0u64))]);
    let mut parents = BTreeMap::new();
    let mut queue = BTreeSet::from([(
        graph[&origin].distance.saturating_sub(input.reach),
        0u64,
        0u64,
        origin,
    )]);
    while let Some((_, sideways, cost, key)) = queue.pop_first() {
        if costs.get(&key) != Some(&(cost, sideways)) {
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
                let to = node(input, &next);
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
            let next_sideways = sideways
                + u64::from(
                    input.walking.is_some_and(|r| r.turns)
                        && sidestep(&graph[&key].pose, &graph[&to].pose),
                );
            if costs
                .get(&to)
                .is_some_and(|old| *old <= (next_cost, next_sideways))
            {
                continue;
            }
            costs.insert(to, (next_cost, next_sideways));
            parents.insert(to, key);
            let estimate =
                next_cost.saturating_add(graph[&to].distance.saturating_sub(input.reach));
            queue.insert((estimate, next_sideways, next_cost, to));
        }
    }
    Ok(None)
}
fn sidestep(from: &Pose, to: &Pose) -> bool {
    let d = [to.at[0] - from.at[0], to.at[2] - from.at[2]];
    if d == [0, 0] {
        return false;
    }
    let forward = match to.yaw {
        Yaw::Zero => [0, -1],
        Yaw::Quarter => [-1, 0],
        Yaw::Half => [0, 1],
        Yaw::ThreeQuarter => [1, 0],
    };
    d != forward
}
