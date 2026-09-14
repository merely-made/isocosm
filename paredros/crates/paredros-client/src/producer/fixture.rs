// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The timed-action fixture world, as `bin/timed_action/model.rs` builds it.
//!
//! Duplicated deliberately and minimally: `App::new`, `support::configure_keeper`
//! and `target_body` all live inside the binary and are not reachable from the
//! library. Everything else the fixture needs — the three-lives bodies, the
//! limb roles, the world generator — comes from `paredros_world::fixtures`.
//! Seed 7, keeper plus target, identical intent order.

use mesocosm_core::{Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use paredros_world::fixtures::three_lives;
use paredros_world::timed_action::{Direction, TimedActionRules, TimedActionSession};
use paredros_world::{
    CombatRules, GameIntent, GameState, ItemKind, ItemLocation, MOTION_SCALE, MotionEnvelope,
    MotionRules, MovementProfile, Session, SupportBand, World, WorldConfig,
};
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

use super::{SceneHandle, SceneModel};

pub struct Fixture {
    pub action: TimedActionSession,
    pub keeper: SubjectId,
    pub target: SubjectId,
    pub combat_rules: CombatRules,
}

impl Fixture {
    /// A handle over a copy of the current session, framed on the keeper.
    pub fn scene(&self) -> SceneHandle {
        SceneModel::new(self.action.session().clone(), self.keeper).into_handle()
    }

    /// The charged volley the binary's smoke sequence runs, which severs the
    /// target's authored limb. Returns the parts the target lost.
    pub fn sever_target_limb(&mut self) -> Vec<PartId> {
        self.action.prepare(Direction::Right).expect("prepare");
        self.action.join(PartId(2)).expect("join");
        let start = self.action.action().expect("open action").last_tick;
        for step in 1..=4 {
            self.action
                .charge(Tick(start.0 + step))
                .expect("charge tick");
        }
        let tick = self.action.action().expect("open action").last_tick;
        self.action
            .release_against(self.target, tick, self.combat_rules)
            .expect("volley resolves");
        let document = &self
            .action
            .session()
            .game()
            .anatomies()
            .get(self.target)
            .expect("target anatomy")
            .document;
        document
            .parts
            .iter()
            .filter(|part| part.severed)
            .map(|part| part.id)
            .collect()
    }
}

fn target_body() -> BodyDocument {
    let mut body = BodyDocument::new(SpeciesId(73), VolumeRef::from_tag(73), 48_000, [2; 3]);
    for (tag, offset) in [(1, [3, 0, 0]), (2, [-3, 2, 0])] {
        body.attach(
            VolumeRef::from_tag(tag),
            2_000,
            [1; 3],
            Attachment {
                parent: body.root,
                offset,
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("authored target limb attaches");
    }
    body
}

fn configure_keeper(game: &mut GameState, subject: SubjectId) {
    let body = &game.current_anatomy(subject).unwrap().document;
    let profile = MovementProfile {
        revision: paredros_world::MOVEMENT_PROFILE_REVISION,
        source_revision: BodyRevisionId(0),
        envelope: MotionEnvelope {
            anchor: body.root,
            half_width: MOTION_SCALE / 4,
            height: 2 * MOTION_SCALE,
        },
        supports: three_lives::limb_parts(body).to_vec(),
        support_band: SupportBand { min_y: 0, max_y: 2 },
    };
    game.apply(GameIntent::ConfigureMovementProfile {
        tick: game.next_tick(),
        subject,
        revision: BodyRevisionId(0),
        profile,
    })
    .expect("authored keeper movement profile");
}

fn network(subject: SubjectId) -> FunctionalNetwork {
    let source = PartRef {
        subject: subject.0,
        part: 0,
    };
    let effect = |part| PartRef {
        subject: subject.0,
        part,
    };
    FunctionalNetwork::new(
        vec![
            Node {
                id: NodeId(1),
                kind: NodeKind::Source {
                    part: source,
                    capacity: 24,
                    charge: 24,
                },
            },
            Node {
                id: NodeId(2),
                kind: NodeKind::Effect { part: effect(1) },
            },
            Node {
                id: NodeId(3),
                kind: NodeKind::Source {
                    part: source,
                    capacity: 24,
                    charge: 24,
                },
            },
            Node {
                id: NodeId(4),
                kind: NodeKind::Effect { part: effect(2) },
            },
        ],
        vec![
            Edge {
                from: NodeId(1),
                to: NodeId(2),
                capacity: 24,
            },
            Edge {
                from: NodeId(3),
                to: NodeId(4),
                capacity: 24,
            },
        ],
    )
    .expect("authored network")
}

pub fn timed_action_world() -> Fixture {
    let keeper = three_lives::KEEPER;
    let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) if item.kind == ItemKind::Dressing => Some(at),
            _ => None,
        })
        .unwrap();
    apply(&mut game, |tick| GameIntent::Generate {
        tick,
        subject: keeper,
        body_seed: 1,
        at,
    });
    apply(&mut game, |tick| GameIntent::Name {
        tick,
        subject: keeper,
        name: paredros_world::Name::new("Keeper").unwrap(),
    });
    apply(&mut game, |tick| GameIntent::AdmitAnatomy {
        tick,
        subject: keeper,
        revision: BodyRevisionId(0),
        document: Box::new(three_lives::three_lives()[0].body.clone()),
    });
    configure_keeper(&mut game, keeper);

    let target = SubjectId(702);
    let target_item = game
        .items()
        .all()
        .find(|item| {
            matches!(item.kind, ItemKind::Dressing) && item.location == ItemLocation::At(at)
        })
        .copied()
        .expect("generated world has a reachable forward dressing for the target");
    let ItemLocation::At(target_spawn) = target_item.location else {
        unreachable!()
    };
    apply(&mut game, |tick| GameIntent::Generate {
        tick,
        subject: target,
        body_seed: 2,
        at: target_spawn,
    });
    apply(&mut game, |tick| GameIntent::Name {
        tick,
        subject: target,
        name: paredros_world::Name::new("Target").unwrap(),
    });
    apply(&mut game, |tick| GameIntent::AdmitAnatomy {
        tick,
        subject: target,
        revision: BodyRevisionId(0),
        document: Box::new(target_body()),
    });
    apply(&mut game, |tick| GameIntent::Take {
        tick,
        subject: target,
        item: target_item.id,
    });
    for _ in 0..12 {
        apply(&mut game, |tick| GameIntent::Move {
            tick,
            subject: target,
            toward: [target_spawn[0] + 12, target_spawn[1], target_spawn[2]],
        });
    }
    apply(&mut game, |tick| GameIntent::AttachItem {
        tick,
        subject: target,
        item: target_item.id,
        part: PartId(2),
        revision: BodyRevisionId(0),
    });
    let rules = TimedActionRules {
        max_contributors: 8,
        max_elapsed_ticks: 60,
        charge_per_tick: 2,
        max_charge_per_limb: 12,
        evaluation: WorldRules {
            allowed_operators: [Operator::Strengthen].into_iter().collect(),
            allowed_costs: [1, 2, 3, 5, 8, 12].into_iter().collect(),
            max_range: 0,
            max_hops: 4,
        },
    };
    let session = Session::begin(game, keeper).unwrap();
    Fixture {
        action: TimedActionSession::begin(session, network(keeper), rules).unwrap(),
        keeper,
        target,
        combat_rules: CombatRules {
            max_reach: 64,
            sever_threshold: 12,
            ..CombatRules::default()
        },
    }
}

fn apply(game: &mut GameState, intent: impl FnOnce(Tick) -> GameIntent) {
    let tick = game.next_tick();
    game.apply(intent(tick)).expect("fixture intent");
}

/// One recorded fixed motion step for a subject, the same path the
/// timed-action host's movement keys take.
pub fn advance_motion(
    session: &mut Session,
    subject: SubjectId,
    input: paredros_world::MotionInput,
) {
    let game = session.game();
    let pose = game.movement().pose(subject).expect("pose");
    let revision = game.bodies().get(subject).expect("body").revision;
    let tick = game.next_tick();
    session
        .apply_game(GameIntent::AdvanceMotion {
            tick,
            subject,
            revision,
            step: pose.step + 1,
            input,
            rules: MotionRules::default(),
        })
        .expect("recorded motion step");
}
