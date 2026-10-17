// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The seed-7 timed-action fixture world: one home for every consumer.
//!
//! The producer tests, session host, timed-action host and glyph tests
//! share this native founding. Its level practice lane is an accepted
//! material edit; no body position or contact result is supplied by hand.

use crate::identity::{BodyRevisionId, SubjectId, Tick};
use isocosm::lineage::SpeciesId;
use isometer_core::PartId;
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

use crate::fixtures::three_lives;
use crate::glyphs::{AcceptedKind, CanonSpec, EventGrant, GlyphDefinition, GlyphRules};
use crate::timed_action::{Direction, TimedActionRules, TimedActionSession};
use crate::{
    CanonRevisionCause, CombatRules, GameIntent, GameState, ItemId, ItemKind, ItemLocation,
    MOTION_SCALE, MotionEnvelope, MotionRules, MovementProfile, Session, SupportBand, World,
    WorldConfig, part_bounds,
};

pub struct Fixture {
    pub action: TimedActionSession,
    pub keeper: SubjectId,
    pub target: SubjectId,
    /// The dressing the target carries, then wears on part 2. A strike that
    /// severs that part drops it at the target's feet.
    pub target_item: ItemId,
    pub combat_rules: CombatRules,
}

fn lateral_radius(game: &GameState, subject: SubjectId) -> i32 {
    let body = &game.current_anatomy(subject).unwrap().document;
    body.parts
        .iter()
        .filter_map(|part| part_bounds(body, part.id))
        .map(|bounds| bounds.min[0].abs().max(bounds.max[0].abs()))
        .max()
        .unwrap_or(0)
}

impl Fixture {
    /// The charged volley the binary's smoke sequence runs, which severs the
    /// target's native limb. Returns the parts the target lost.
    pub fn sever_target_limb(&mut self) -> Vec<PartId> {
        self.action.prepare(Direction::Right).expect("prepare");
        // Part 1 is the keeper's native +X limb. Part 2 extends toward -X,
        // so a rightward sweep from it can stop short despite body clearance.
        self.action.join(PartId(2)).expect("join second limb; prepare already selected part 1");
        let start = self.action.action().expect("open action").last_tick;
        for step in 1..=4 {
            self.action
                .charge(Tick(start.0 + step))
                .expect("charge tick");
        }
        let tick = self.action.action().expect("open action").last_tick;
        let events = self.action
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
        let severed = document
            .parts
            .iter()
            .filter(|part| part.severed)
            .map(|part| part.id)
            .collect::<Vec<_>>();
        if severed.is_empty() {
            let game = self.action.session().game();
            let source = game.current_anatomy(self.keeper).unwrap();
            let strikes = events.iter().find_map(|event| match event {
                crate::GameEvent::VolleyResolved { strikes, .. } => Some(strikes),
                _ => None,
            });
            panic!(
                "native fixture volley missed: keeper_position={:?}, target_position={:?}, keeper_pose={:?}, target_pose={:?}, source_part1={:?}, target_parts={:?}, strikes={strikes:?}, charge=8, reach={}",
                game.movement().position(self.keeper),
                game.movement().position(self.target),
                game.movement().pose(self.keeper),
                game.movement().pose(self.target),
                part_bounds(&source.document, PartId(1)),
                document.parts.iter().filter_map(|part| part_bounds(document, part.id).map(|bounds| (part.id, bounds))).collect::<Vec<_>>(),
                1 + 8 / self.combat_rules.charge_per_reach as i32,
            );
        }
        severed
    }
}

fn configure_keeper(game: &mut GameState, subject: SubjectId) {
    let body = &game.current_anatomy(subject).unwrap().document;
    let profile = MovementProfile {
        revision: crate::MOVEMENT_PROFILE_REVISION,
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
    build_timed_action_world(crate::MotionSolver::STILL, None)
}

/// The same world, its motion solved by `solver`; the plain fixture admits
/// motion steps without moving, the sim linking no solver (ruling 597).
pub fn timed_action_world_solving(solver: crate::MotionSolver) -> Fixture {
    build_timed_action_world(solver, None)
}

/// Opt-in variant for native external-volley tests. The default fixture stays
/// exactly two bodies so its occlusion and severance controls remain stable.
pub fn timed_action_world_solving_with_practice_striker(solver: crate::MotionSolver) -> Fixture {
    build_timed_action_world(solver, Some(false))
}

pub fn timed_action_world_with_practice_striker() -> Fixture {
    build_timed_action_world(crate::MotionSolver::STILL, Some(false))
}

/// Opt-in practice actor founded at the non-played target for world-harm
/// tests; ordinary shared fixture geometry remains unchanged.
pub fn timed_action_world_with_practice_striker_at_target() -> Fixture {
    build_timed_action_world(crate::MotionSolver::STILL, Some(true))
}

fn build_timed_action_world(
    solver: crate::MotionSolver,
    practice_at_target: Option<bool>,
) -> Fixture {
    let keeper = three_lives::KEEPER;
    let mut world = World::generate(7, WorldConfig::default()).unwrap();
    // Keep the native bodies apart on a level practice lane. The generated
    // dressing starts on a hill; widening the old spacing otherwise puts the
    // target three cells below a horizontal strike. Author the actual Ground
    // before arrivals, so items, stances, contact and replay agree.
    let top = world.ground().surface(0, 0).unwrap();
    let floor = (0..=40).filter_map(|x| world.ground().surface(x, 0)).min().unwrap() + 1;
    let edit = isometer_space::Edit {
        op: isometer_space::Op::Carve,
        shape: isometer_space::edit::Shape::Box {
            min: crate::walking::to_site(world.volume(), [-4, floor, -4]),
            max: crate::walking::to_site(world.volume(), [41, top + 8, 4]),
        },
    };
    world.apply(crate::WorldIntent::MaterialEdit { tick: Tick(0), edit })
        .expect("native practice lane");
    let mut game = GameState::new(world).with_motion_solver(solver);
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
        name: crate::Name::new("Keeper").unwrap(),
    });
    let keeper_anatomy = game
        .native_anatomy(keeper, three_lives::WETLAND_LINE)
        .unwrap();
    apply(&mut game, |tick| GameIntent::AdmitAnatomy {
        tick,
        subject: keeper,
        revision: BodyRevisionId(0),
        document: Box::new(keeper_anatomy),
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
        name: crate::Name::new("Target").unwrap(),
    });
    let target_anatomy = game.native_anatomy(target, SpeciesId(73)).unwrap();
    apply(&mut game, |tick| GameIntent::AdmitAnatomy {
        tick,
        subject: target,
        revision: BodyRevisionId(0),
        document: Box::new(target_anatomy),
    });
    apply(&mut game, |tick| GameIntent::Take {
        tick,
        subject: target,
        item: target_item.id,
    });
    let body_clearance = lateral_radius(&game, keeper) + lateral_radius(&game, target);
    let keeper_at = game.movement().position(keeper).unwrap();
    let target_steps = (keeper_at[0] + body_clearance - target_spawn[0]).max(0);
    for _ in 0..target_steps {
        apply(&mut game, |tick| GameIntent::Move {
            tick,
            subject: target,
            toward: [
                keeper_at[0] + body_clearance,
                target_spawn[1],
                target_spawn[2],
            ],
        });
    }
    let keeper_at = game.movement().position(keeper).unwrap();
    let target_at = game.movement().position(target).unwrap();
    assert!(
        target_at[0] - keeper_at[0] >= lateral_radius(&game, keeper) + lateral_radius(&game, target),
        "native fixture bodies must stand clear by their measured lateral bounds: keeper={keeper_at:?}, target={target_at:?}, radii=({}, {})",
        lateral_radius(&game, keeper),
        lateral_radius(&game, target),
    );
    if let Some(at_target) = practice_at_target {
        let actor_at = if at_target { target_at } else { keeper_at };
        crate::fixtures::native_severing::found_practice_striker(&mut game, actor_at);
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
        target_item: target_item.id,
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

/// The movement-rules revision the keeper's authored profile selects, as
/// `bin/timed_action/support.rs` computes it.
pub fn motion_rules(game: &GameState, subject: SubjectId) -> MotionRules {
    MotionRules {
        revision: if game.movement_profile(subject).is_some() {
            2
        } else {
            1
        },
        ..MotionRules::default()
    }
}

/// One recorded fixed motion step for a subject, the same path the
/// timed-action host's movement keys take.
pub fn advance_motion(session: &mut Session, subject: SubjectId, input: crate::MotionInput) {
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

/// Publishes one canon revision into accepted history, the world fact a
/// hagioglyph reading follows. The cause is the caller's: an authored label
/// is a fixture control, while a promotion needs its condition receipt.
pub fn revise_canon(session: &mut Session, revision: u64, seed: u64, cause: CanonRevisionCause) {
    let tick = session.game().next_tick();
    session
        .apply_game(GameIntent::ReviseCanon {
            tick,
            revision,
            seed,
            cause,
        })
        .expect("accepted canon revision");
}

/// The demonstration glyph canon the session host's acquisition journal reads
/// against, and the rules that bind it to the played subject.
///
/// **This is a fixture, not a world canon.** Eponym has no authored glyph
/// correspondence; these seven ids and effects are plain placeholder words
/// chosen to make the journal legible in the window, and the effect ids are
/// references the reading never executes. A real canon is a design decision
/// this lane does not make.
pub fn demonstration_canon() -> CanonSpec {
    let glyph = |id: &str, display: &str, effect: &str| GlyphDefinition {
        id: format!("paredros-fixture:{id}"),
        display: display.to_owned(),
        effect: format!("paredros-fixture:{effect}"),
    };
    CanonSpec {
        version: 1,
        id: "paredros-fixture:canon".into(),
        revision: 1,
        glyphs: vec![
            glyph("step", ".", "travel"),
            glyph("strike", "/", "reach"),
            glyph("carry", "+", "hold"),
            glyph("wear", "o", "fasten"),
            glyph("endure", "x", "hold-on"),
            glyph("mend", "~", "recover"),
            glyph("end", "#", "close"),
        ],
        variants: Vec::new(),
        limits: Default::default(),
    }
}

/// One grant per accepted event kind the played subject can produce here.
pub fn demonstration_rules(subject: SubjectId) -> GlyphRules {
    let grant = |event, glyph: &str| EventGrant {
        event,
        glyph: format!("paredros-fixture:{glyph}"),
    };
    GlyphRules {
        canon: demonstration_canon(),
        individual: "Keeper".into(),
        subject,
        unlock_thresholds: vec![0, 0, 0, 0, 0, 0, 0],
        grants: vec![
            grant(AcceptedKind::MotionAdvanced, "step"),
            grant(AcceptedKind::VolleyResolved, "strike"),
            grant(AcceptedKind::Took, "carry"),
            grant(AcceptedKind::ItemAttached, "wear"),
            grant(AcceptedKind::Injured, "endure"),
            grant(AcceptedKind::Rested, "mend"),
            grant(AcceptedKind::Died, "end"),
        ],
    }
}
