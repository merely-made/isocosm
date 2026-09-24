// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use mesocosm_core::{Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
use eponym_identity::SubjectId;
use eponym_world::timed_action::{Direction, TimedActionRules, TimedActionSession};
use eponym_world::{
    GameIntent, GameState, ItemId, ItemKind, ItemLocation, Name, Session, World, WorldConfig,
};
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

pub const ACTOR: SubjectId = SubjectId(1);
pub const TARGET: SubjectId = SubjectId(2);

pub fn prepared() -> (TimedActionSession, ItemId) {
    let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) if item.kind == ItemKind::Dressing => Some(at),
            _ => None,
        })
        .unwrap();
    for (subject, seed, name) in [(ACTOR, 1, "Actor"), (TARGET, 2, "Target")] {
        game.apply(GameIntent::Generate {
            tick: game.next_tick(),
            subject,
            body_seed: seed,
            at,
        })
        .unwrap();
        game.apply(GameIntent::Name {
            tick: game.next_tick(),
            subject,
            name: Name::new(name).unwrap(),
        })
        .unwrap();
        game.apply(GameIntent::AdmitAnatomy {
            tick: game.next_tick(),
            subject,
            revision: game.bodies().get(subject).unwrap().revision,
            document: Box::new(body(subject == ACTOR)),
        })
        .unwrap();
    }
    let item = game
        .items()
        .at(at)
        .find(|item| item.kind == ItemKind::Dressing)
        .unwrap()
        .id;
    game.apply(GameIntent::Take {
        tick: game.next_tick(),
        subject: TARGET,
        item,
    })
    .unwrap();
    let goal = [at[0] + 12, at[1], at[2]];
    for _ in 0..12 {
        game.apply(GameIntent::Move {
            tick: game.next_tick(),
            subject: TARGET,
            toward: goal,
        })
        .unwrap();
    }
    let target_at = game.movement().position(TARGET).unwrap();
    assert!(target_at[0] - at[0] >= 9, "fixture bodies must not overlap");
    assert!(
        game.world().ground().sees(
            [at[0] + 4, at[1] + 2, at[2] + 2],
            [target_at[0] + 4, target_at[1] + 2, target_at[2] + 2]
        ),
        "fixture requires a clear limb-height ray"
    );
    game.apply(GameIntent::AttachItem {
        tick: game.next_tick(),
        subject: TARGET,
        item,
        part: PartId(2),
        revision: game.bodies().get(TARGET).unwrap().revision,
    })
    .unwrap();

    let mut session =
        TimedActionSession::begin(Session::begin(game, ACTOR).unwrap(), network(), rules())
            .unwrap();
    session.prepare(Direction::Right).unwrap();
    session.join(PartId(2)).unwrap();
    (session, item)
}

fn body(actor: bool) -> BodyDocument {
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [2; 3]);
    let offsets = if actor {
        [[3, 0, 0], [4, 0, 0]]
    } else {
        [[3, 0, 0], [-3, 2, 0]]
    };
    for offset in offsets {
        body.attach(
            VolumeRef::from_tag(2),
            20,
            [1; 3],
            Attachment {
                parent: PartId(0),
                offset,
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    }
    body
}

fn network() -> FunctionalNetwork {
    FunctionalNetwork::new(
        vec![
            Node {
                id: NodeId(0),
                kind: NodeKind::Source {
                    part: PartRef {
                        subject: ACTOR.0,
                        part: 0,
                    },
                    capacity: 80,
                    charge: 80,
                },
            },
            Node {
                id: NodeId(1),
                kind: NodeKind::Effect {
                    part: PartRef {
                        subject: ACTOR.0,
                        part: 1,
                    },
                },
            },
            Node {
                id: NodeId(2),
                kind: NodeKind::Effect {
                    part: PartRef {
                        subject: ACTOR.0,
                        part: 2,
                    },
                },
            },
        ],
        vec![
            Edge {
                from: NodeId(0),
                to: NodeId(1),
                capacity: 80,
            },
            Edge {
                from: NodeId(0),
                to: NodeId(2),
                capacity: 80,
            },
        ],
    )
    .unwrap()
}

fn rules() -> TimedActionRules {
    TimedActionRules {
        max_contributors: 2,
        max_elapsed_ticks: 8,
        charge_per_tick: 4,
        max_charge_per_limb: 8,
        evaluation: WorldRules {
            allowed_operators: BTreeSet::from([Operator::Strengthen]),
            allowed_costs: BTreeSet::from([4]),
            max_range: 0,
            max_hops: 2,
        },
    }
}
