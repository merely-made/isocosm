// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The seed-7 timed-action fixture world: one home for every consumer.
//!
//! `bin/timed_action/model.rs` builds this world inside the binary, where no
//! other target can reach it. P2 promoted a copy into the client crate; P5
//! moved it here, because the construction depends only on this crate's own
//! dependencies and the glyph tests need it too. The producer tests, the
//! `session` host and `glyphs::tests` all read this one. The timed-action bin
//! keeps its own copy until it retires. Seed 7, keeper plus target, identical
//! intent order.

use isometer_core::{Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
use eponym_identity::{BodyRevisionId, SubjectId, Tick};
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

use crate::fixtures::three_lives;
use crate::glyphs::{AcceptedKind, CanonSpec, EventGrant, GlyphDefinition, GlyphRules};
use crate::timed_action::{Direction, TimedActionRules, TimedActionSession};
use crate::{
    CanonRevisionCause, CombatRules, GameIntent, GameState, ItemId, ItemKind, ItemLocation,
    MOTION_SCALE, MotionEnvelope, MotionRules, MovementProfile, Session, SupportBand, World,
    WorldConfig,
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

impl Fixture {
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
        name: crate::Name::new("Keeper").unwrap(),
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
        name: crate::Name::new("Target").unwrap(),
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
