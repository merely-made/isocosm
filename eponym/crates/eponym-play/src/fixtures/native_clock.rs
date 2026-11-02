// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The borrowed mode with a configured clock, in rounds (cases 1095, 1102,
//! 1107, 1108): the rounds timed-action world handed off with every subject
//! bound, the keeper's actions configured, and the clock configured over its
//! physical reading. Driven through the real envelopes, with a real
//! `OrdinaryAccess` and the participant's native `View`.

use crate::contract::mode::{
    self, Presentation, PoseBinding, SpatialBindingInput,
    actions::{self, Action, ActionConfig, ActionInput, CombatPolicy, FallPayment, MotionPolicy, Run, SubjectProfile, game_source},
    clock::{self, Config, Physical, Profile},
};
use crate::fixtures::{session, three_lives};
use crate::identity::SubjectId;
use crate::{GameIntent, MotionInput, MotionPose, MotionSolver};
use cosmograph::directing::readings::{self, Mode, View};
use cosmograph::operations::Window;
use cosmograph::schema::Id;
use cosmograph::Session;
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use isometer_space::places::Places;
use std::collections::BTreeMap;

/// A world volume's depth, as Eponym's world lifts it.
const DEPTH: i64 = 8;

/// The handed-off native session and the borrowed mode's bytes.
pub struct NativeClock {
    pub native: Session,
    pub game: Vec<u8>,
    pub keeper: SubjectId,
    pub entity: Id,
    pub participant: Id,
    /// The host's solver; still unless a test hands it another.
    pub solver: MotionSolver,
}

impl NativeClock {
    /// The keeper's player access, resolved as a window resolves it.
    pub fn access(&self) -> OrdinaryAccess {
        OrdinaryAccess { participant: ParticipantHandle(self.participant), seat: Seat::Player,
            subject: Some(EntityHandle(self.entity)) }
    }

    /// What the keeper's player sees.
    pub fn view(&self) -> View {
        readings::view(&self.native.sim, self.entity, Mode::Survival)
    }

    pub fn mode(&self) -> mode::ModeState {
        mode::ModeState::decode(&self.game).unwrap().expect("marked mode state")
    }

    pub fn binding(&self) -> PoseBinding {
        self.mode().bindings.into_iter().find(|b| b.subject == self.keeper).expect("keeper bound")
    }

    fn submit(&mut self, action: Action) -> Result<String, String> {
        let input = ActionInput { kind: "eponym:actions".into(), version: 1, expected_native: mode::source(&self.native),
            expected_game: game_source(&self.game), subject: self.keeper, action };
        let (access, view) = (self.access(), self.view());
        let (outcome, game) = actions::submit(&mut self.native, self.game.clone(), &input, access, &view, self.solver)?;
        self.game = game;
        Ok(outcome)
    }

    /// One fixed sixtieth-second motion step of the keeper.
    pub fn step(&mut self, input: MotionInput) -> Result<String, String> {
        let step = self.binding().pose.step + 1;
        self.submit(Action::Motion { step, input,
            payment: FallPayment { wounds: Vec::new(), hunger: Vec::new(), fatigue: Vec::new() } })
    }

    /// The keeper's pose after `runs`, stepped locally as a host steps it.
    pub fn local(&self, runs: &[Run]) -> MotionPose {
        let b = self.binding();
        let volume = self.native.sim.volume(b.site, b.window.min, b.window.max, b.window.depth).unwrap();
        let ground = volume.ground();
        let rules = actions::motion_rules(&self.native, &self.game, self.keeper).expect("keeper motion rules");
        runs.iter().flat_map(|r| std::iter::repeat_n(r.input, r.steps as usize))
            .fold(b.pose, |pose, input| self.solver.advance(&ground, pose, input, rules).expect("a local step").pose)
    }

    /// A round's minute of motion, its end pose stepped locally (case 1117).
    pub fn steps(&mut self, runs: Vec<Run>) -> Result<String, String> {
        let end = self.local(&runs);
        self.submit(Action::Steps { runs, end, falls: Vec::new() })
    }

    /// Submits any action for the keeper.
    pub fn act(&mut self, action: Action) -> Result<String, String> {
        self.submit(action)
    }

    /// The sim's minute, its handbacks taken by the configured clock.
    pub fn minute(&mut self) -> Result<String, String> {
        self.submit(Action::Advance { ticks: 1 })
    }
}

/// The rounds timed-action world, handed off and its clock configured.
pub fn native_clock_in_rounds() -> NativeClock {
    let mut f = session::timed_action_world_in_rounds();
    // The keeper tells the sim its patch once, with one still step (case
    // 1102); only the played body steps.
    let subjects: Vec<_> = f.action.session().game().world().subjects().collect();
    for subject in [f.keeper] {
        let game = f.action.session().game();
        let intent = GameIntent::AdvanceMotion { tick: game.next_tick(), subject,
            revision: game.bodies().get(subject).expect("body").revision,
            step: game.movement().pose(subject).expect("pose").step + 1,
            input: MotionInput { move_x: 0, move_z: 0 }, rules: session::motion_rules(game, subject) };
        f.action.apply_game_batch(&[intent]).expect("a still step");
    }
    let game = f.action.session().game();
    let world = game.world();
    let native = world.session();
    let side = world.config().extent as i64 * 2;
    let window = Window { min: [0, 0], max: [side, side], depth: DEPTH };
    let volume = native.sim.volume(world.site(), window.min, window.max, window.depth).unwrap();
    let places = Places::derive(&volume, Default::default()).unwrap();
    let mut bindings = Vec::new();
    let mut roots = BTreeMap::new();
    for (subject, entity) in &subjects {
        let pose = game.pose(*subject).expect("an accepted pose");
        let root = crate::walking::to_site(&volume, pose.cell().unwrap());
        let e = native.sim.state().population.get(*entity).unwrap();
        bindings.push(PoseBinding { subject: *subject, entity: *entity, pose, site: world.site(),
            patch: places.place_at(root).expect("a clear patch"), window: window.clone(),
            body_revision: e.body_revision, terrain_revision: native.sim.terrain_revision(),
            resident_since: e.arrived, places: None });
        roots.insert(*subject, root);
    }
    let presentation = Presentation { centre: [0.0; 3], forward: [0.0, 0.0, 1.0], half_height: 8.0, depth: 64.0,
        pan: [0.0; 3], scale: 1.0, material: 0, played_tint: [1.0; 3], other_tint: [0.5; 3] };
    let input = SpatialBindingInput { version: 1, expected_native: mode::source(native), bindings, presentation };
    let keeper = f.keeper;
    let entity = world.entity(keeper).unwrap();
    let profile = SubjectProfile { subject: keeper, species: three_lives::WETLAND_LINE,
        revision: game.bodies().get(keeper).unwrap().revision,
        native_revision: native.sim.state().population.get(entity).unwrap().body_revision,
        motion: Some(MotionPolicy { rules: session::motion_rules(game, keeper), profile: game.movement_profile(keeper).cloned() }),
        combat: Some(CombatPolicy { rules: f.combat_rules, timed: f.action.rules().clone(), network: f.action.network().clone() }) };
    let config = ActionConfig { version: 1, expected_native: mode::source(native),
        expected_game: f.action.mode_source(&input).unwrap(), profiles: vec![profile], project_carry: Vec::new() };
    let frame_origin = crate::walking::to_site(&volume, [0; 3]);
    let root = roots[&keeper];
    let handoff = f.action.into_native_mode(&input, &config).map_err(|(_, why)| why).expect("handoff");
    let physical = Physical { subject: keeper, yaw: Default::default(), eye: [root[0], root[1] + 1, root[2]],
        places: Default::default(), frame_origin };
    let clocked = Config { version: 1, expected_native: mode::source(&handoff.native),
        expected_game: game_source(&handoff.game),
        profile: Profile { version: 1, terrain: BTreeMap::new(), steps: BTreeMap::new(), arrivals: Vec::new(), births: Vec::new() },
        physical: vec![physical], sight: None };
    let game = clock::configure(&handoff.native, &handoff.game, &clocked).expect("clock configured");
    let participant = handoff.native.sim.state().population.groups.keys().copied()
        .find(|p| handoff.native.sim.plays(*p) == Some(entity)).expect("the keeper's participant");
    NativeClock { native: handoff.native, game, keeper, entity, participant, solver: MotionSolver::STILL }
}
