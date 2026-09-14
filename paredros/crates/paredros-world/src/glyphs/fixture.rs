// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The scripted world the glyph tests read: the same seed-7 keeper and target
//! `paredros_client::session_fixture` builds for the host, rebuilt here
//! because that fixture lives in the client crate and cannot be reached from
//! the world crate. Same intents, same timed-action grammar.

use mesocosm_core::{Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use wing_functions::{Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules};
use wing_glyphs::{CanonSpec, GlyphDefinition};

use crate::fixtures::three_lives;
use crate::timed_action::{Direction, TimedActionRules, TimedActionSession};
use crate::{
    CombatRules, GameIntent, GameState, ItemId, ItemKind, ItemLocation, MOTION_SCALE,
    MotionEnvelope, MotionInput, MotionRules, MovementProfile, Session, SupportBand, World,
    WorldConfig,
};

use super::{AcceptedKind, EventGrant, GlyphRules};

pub(super) const TARGET: SubjectId = SubjectId(702);

pub(super) struct Fixture {
    pub action: TimedActionSession,
    pub keeper: SubjectId,
    pub target: SubjectId,
    pub combat_rules: CombatRules,
}

/// The demonstration canon these tests read against. Plain ids and effects;
/// authored evidence, not a world canon.
pub(super) fn canon() -> CanonSpec {
    let glyph = |id: &str, display: &str, effect: &str| GlyphDefinition {
        id: format!("paredros-test:{id}"),
        display: display.to_owned(),
        effect: format!("paredros-test:{effect}"),
    };
    CanonSpec {
        version: 1,
        id: "paredros-test:canon".into(),
        revision: 1,
        glyphs: vec![
            glyph("step", ".", "carry-weight"),
            glyph("strike", "/", "reach"),
            glyph("endure", "x", "hold-on"),
            glyph("mend", "~", "recover"),
            glyph("carry", "+", "hold"),
            glyph("wear", "o", "fasten"),
            glyph("end", "#", "close"),
        ],
        variants: Vec::new(),
        limits: Default::default(),
    }
}

pub(super) fn rules(subject: SubjectId) -> GlyphRules {
    let grant = |event, glyph: &str| EventGrant {
        event,
        glyph: format!("paredros-test:{glyph}"),
    };
    GlyphRules {
        canon: canon(),
        individual: "keeper".into(),
        subject,
        unlock_thresholds: vec![0, 0, 0, 0, 0, 0, 0],
        grants: vec![
            grant(AcceptedKind::MotionAdvanced, "step"),
            grant(AcceptedKind::VolleyResolved, "strike"),
            grant(AcceptedKind::Injured, "endure"),
            grant(AcceptedKind::Rested, "mend"),
            grant(AcceptedKind::Took, "carry"),
            grant(AcceptedKind::ItemAttached, "wear"),
            grant(AcceptedKind::Died, "end"),
        ],
    }
}

impl Fixture {
    /// The charged volley the host's smoke runs, which severs the target's
    /// authored limb.
    pub(super) fn volley(&mut self) {
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
    }

    /// The host's injury debug cut: a fall, then the reconciliation, applied
    /// as one batch so the open action repairs with it.
    pub(super) fn injure(&mut self) {
        let subject = self.keeper;
        let old = self.game().bodies().get(subject).unwrap().revision;
        let first = self.game().next_tick();
        self.action
            .apply_game_batch(&[
                GameIntent::Fall {
                    tick: first,
                    subject,
                    distance: 5,
                },
                GameIntent::ReconcileAnatomy {
                    tick: Tick(first.0 + 1),
                    subject,
                    from_revision: old,
                    revision: BodyRevisionId(old.0 + 1),
                    severed_parts: vec![PartId(1)],
                },
            ])
            .expect("injury cut");
    }

    pub(super) fn rest(&mut self) {
        self.action.rest().expect("rest");
    }

    /// The keeper takes the dressing under its feet, the host's `E` verb.
    pub(super) fn take_dressing(&mut self) -> ItemId {
        let subject = self.keeper;
        let at = self.game().movement().position(subject).expect("position");
        let item = self
            .game()
            .items()
            .at(at)
            .find(|item| item.kind == ItemKind::Dressing)
            .map(|item| item.id)
            .expect("a dressing lies where the keeper stands");
        let tick = self.game().next_tick();
        self.action
            .apply_game_batch(&[GameIntent::Take {
                tick,
                subject,
                item,
            }])
            .expect("pickup");
        item
    }

    /// The keeper wears what it picked up, the equipment panel's verb.
    pub(super) fn attach(&mut self, item: ItemId, part: PartId) {
        let subject = self.keeper;
        let revision = self.game().bodies().get(subject).unwrap().revision;
        let tick = self.game().next_tick();
        self.action
            .apply_game_batch(&[GameIntent::AttachItem {
                tick,
                subject,
                item,
                part,
                revision,
            }])
            .expect("attach");
    }

    /// One recorded fixed motion step, the host's movement key path.
    pub(super) fn step(&mut self, toward: [i32; 3]) {
        let subject = self.keeper;
        let pose = self.game().movement().pose(subject).expect("pose");
        let revision = self.game().bodies().get(subject).unwrap().revision;
        let tick = self.game().next_tick();
        self.action
            .apply_game_batch(&[GameIntent::AdvanceMotion {
                tick,
                subject,
                revision,
                step: pose.step + 1,
                input: MotionInput {
                    move_x: (toward[0] * 32767) as i16,
                    move_z: (toward[2] * 32767) as i16,
                },
                rules: MotionRules::default(),
            }])
            .expect("motion step");
    }

    pub(super) fn game(&self) -> &GameState {
        self.action.session().game()
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
    apply(game, |tick| GameIntent::ConfigureMovementProfile {
        tick,
        subject,
        revision: BodyRevisionId(0),
        profile: profile.clone(),
    });
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

pub(super) fn world() -> Fixture {
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

    apply(&mut game, |tick| GameIntent::Generate {
        tick,
        subject: TARGET,
        body_seed: 2,
        at,
    });
    apply(&mut game, |tick| GameIntent::Name {
        tick,
        subject: TARGET,
        name: crate::Name::new("Target").unwrap(),
    });
    apply(&mut game, |tick| GameIntent::AdmitAnatomy {
        tick,
        subject: TARGET,
        revision: BodyRevisionId(0),
        document: Box::new(target_body()),
    });
    // The target walks off its spawn cell and leaves the dressing there: the
    // keeper is the one who picks it up, so `Took` is the keeper's evidence.
    for _ in 0..12 {
        apply(&mut game, |tick| GameIntent::Move {
            tick,
            subject: TARGET,
            toward: [at[0] + 12, at[1], at[2]],
        });
    }
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
        target: TARGET,
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
