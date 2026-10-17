// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native input receipt for Eponym's bounded timed limb action.
use self::hud::Hud;
use eponym_client::gpu::Composer;
use eponym_play::fixtures::native_severing;
use eponym_play::identity::Tick;
use eponym_play::timed_action::{Direction, TimedActionSession};
use eponym_play::{
    CombatRules, GameEvent, GameIntent, ItemKind, ItemLocation,
};
use std::{
    env,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::Window,
};
#[path = "persistence.rs"]
mod persistence;
#[path = "actions.rs"]
mod actions;
#[path = "host.rs"]
mod host;
#[path = "hud.rs"]
mod hud;
#[path = "support.rs"]
mod support;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
#[path = "view.rs"]
mod view;

pub fn run() {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("timed action host");
    if app.smoke {
        assert!(app.presented, "smoke ended without a presented frame");
        println!("timed action: lifecycle assertions and presented frame passed");
    }
}
pub(crate) struct App {
    pub(crate) instance: wgpu::Instance,
    pub(crate) live: Option<Live>,
    pub(crate) action: TimedActionSession,
    pub(crate) target: eponym_play::identity::SubjectId,
    pub(crate) combat_rules: CombatRules,
    pub(crate) target_item: eponym_play::ItemId,
    pub(crate) attack_direction: Direction,
    pub(crate) held: bool,
    pub(crate) movement_keys: [bool; 4],
    pub(crate) last_motion: Instant,
    pub(crate) last_charge: Instant,
    pub(crate) status: Vec<String>,
    pub(crate) smoke: bool,
    pub(crate) presented: bool,
    pub(crate) opened_at: Instant,
    pub(crate) save_path: PathBuf,
}
pub(crate) struct Live {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) composer: Composer,
    pub(crate) hud: Hud,
}

impl App {
    fn new() -> Self {
        let fixture = eponym_play::fixtures::session::timed_action_world_solving_with_practice_striker(eponym_motion::SOLVER);
        let path = env::var_os("PAREDROS_TIMED_ACTION_SAVE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("timed_action.save"));
        let mut app = Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            live: None,
            action: fixture.action,
            target: fixture.target,
            combat_rules: fixture.combat_rules,
            target_item: fixture.target_item,
            attack_direction: Direction::Right,
            held: false,
            movement_keys: [false; 4],
            last_motion: Instant::now(),
            last_charge: Instant::now(),
            status: vec!["Ready: choose a direction, then hold Space or left mouse.".into()],
            smoke: env::var_os("PAREDROS_TIMED_ACTION_SMOKE").is_some(),
            presented: false,
            opened_at: Instant::now(),
            save_path: path,
        };
        if app.smoke {
            app.smoke_sequence();
        }
        app
    }
    fn smoke_sequence(&mut self) {
        self.move_player([1, 0, 0]);
        assert!(self.action.prepare(self.attack_direction).is_ok());
        // Prepare selects part 1; add the other native limb to the volley.
        assert!(self.action.join(isometer::core::PartId(2)).is_ok());
        let start = self.action.action().unwrap().last_tick;
        assert_eq!(self.action.charge(Tick(start.0 + 1)).unwrap().len(), 2);
        assert_eq!(self.action.charge(Tick(start.0 + 2)).unwrap().len(), 2);
        assert_eq!(self.action.charge(Tick(start.0 + 3)).unwrap().len(), 2);
        assert_eq!(self.action.charge(Tick(start.0 + 4)).unwrap().len(), 2);
        let tick = self.action.action().unwrap().last_tick;
        let events = self
            .action
            .release_against(self.target, tick, self.combat_rules)
            .unwrap();
        let volley = events
            .iter()
            .find_map(|event| match event {
                GameEvent::VolleyResolved { strikes, .. } => Some(strikes),
                _ => None,
            })
            .expect("release records a volley");
        assert!(
            volley
                .iter()
                .any(|strike| matches!(strike.outcome, eponym_play::StrikeOutcome::Hit { .. }))
        );
        let target = self
            .action
            .session()
            .game()
            .current_anatomy(self.target)
            .unwrap();
        assert!(
            target.document.parts.iter().any(|part| part.severed),
            "smoke volley severs a target part"
        );
        let target_at = self
            .action
            .session()
            .game()
            .movement()
            .position(self.target)
            .unwrap();
        assert_eq!(
            self.action
                .session()
                .game()
                .items()
                .get(self.target_item)
                .map(|item| item.location),
            Some(ItemLocation::At(target_at)),
            "the attached target dressing is released at the target"
        );
        self.take_dressing();
        self.injure();
        let wound_before_rest = self
            .action
            .session()
            .game()
            .bodies()
            .get(self.action.session().control().played())
            .unwrap()
            .wound;
        self.rest();
        let player = self.action.session().control().played();
        let game = self.action.session().game();
        let wound_after_rest = game.bodies().get(player).unwrap().wound;
        assert!(
            wound_after_rest == wound_before_rest,
            "current Rest law recovers no wound: before={wound_before_rest}, after={wound_after_rest}"
        );
        assert!(game.events().iter().any(|event| matches!(event,
            GameEvent::Rested { subject, recovered: 0, .. } if *subject == player
        )), "rest records zero wound recovery under the current law");
        assert!(
            game.current_anatomy(player)
                .unwrap()
                .document
                .part(isometer::core::PartId(1))
                .unwrap()
                .severed
        );
        assert_eq!(
            game.items()
                .carried_by(player)
                .filter(|item| item.kind == ItemKind::Dressing)
                .count(),
            0
        );
        self.move_player([1, 0, 0]);
        assert_eq!(
            self.action
                .session()
                .game()
                .movement()
                .pose(player)
                .unwrap()
                .step,
            2
        );
        let after_release = self.action.save().unwrap();
        self.save();
        assert!(
            self.status
                .first()
                .is_some_and(|line| line.starts_with("Saved "))
        );
        self.move_player([0, 0, -1]);
        self.load();
        assert!(
            self.status
                .first()
                .is_some_and(|line| line.starts_with("Loaded "))
        );
        assert_eq!(
            self.action.save().unwrap(),
            after_release,
            "load restores exact post-hit consequences"
        );
        self.status = vec![format!(
            "Smoke: volley resolved with {} strike(s); target anatomy and dropped dressing inspected.",
            volley.len()
        )];
    }
    fn tick(&mut self) {
        self.tick_motion();
        if !self.held || self.last_charge.elapsed() < Duration::from_millis(100) {
            return;
        }
        let tick = self
            .action
            .action()
            .map(|a| Tick(a.last_tick.0 + 1))
            .unwrap_or_else(|| self.action.session().game().next_tick());
        match self.action.charge(tick) {
            Ok(outcomes) => {
                self.status = vec![
                    format!("Charging tick {}: {:?}", tick.0, outcomes),
                    format!(
                        "Contributors: {}",
                        self.action.action().map_or(0, |a| a.contributors.len())
                    ),
                ]
            },
            Err(e) => self.status = vec![format!("Charge paused: {e:?}")],
        }
        self.last_charge = Instant::now();
        self.redraw();
    }
    fn dispatch(&mut self, code: KeyCode, pressed: bool, loop_: &ActiveEventLoop) {
        let movement = match code {
            KeyCode::KeyW => Some(0),
            KeyCode::KeyS => Some(1),
            KeyCode::KeyA => Some(2),
            KeyCode::KeyD => Some(3),
            _ => None,
        };
        if let Some(index) = movement {
            if !self.motion_running() {
                self.last_motion = Instant::now();
            }
            self.movement_keys[index] = pressed;
            return;
        }
        if !pressed {
            if code == KeyCode::Space {
                self.release();
            }
            return;
        }
        match code {
            KeyCode::Escape => loop_.exit(),
            KeyCode::ArrowUp => self.prepare(Direction::Forward),
            KeyCode::ArrowDown => self.prepare(Direction::Backward),
            KeyCode::ArrowLeft => self.prepare(Direction::Left),
            KeyCode::ArrowRight => self.prepare(Direction::Right),
            KeyCode::Space => {
                if self.action.action().is_none() {
                    self.prepare(self.attack_direction);
                }
                self.held = true;
                self.last_charge = Instant::now();
            },
            KeyCode::KeyJ => self.join_limb(),
            KeyCode::KeyI => self.injure(),
            KeyCode::KeyR => self.rest(),
            KeyCode::KeyE => self.take_dressing(),
            KeyCode::F5 => self.save(),
            KeyCode::F9 => self.load(),
            _ => return,
        }
        self.redraw();
    }
    fn prepare(&mut self, direction: Direction) {
        self.status = match self.action.prepare(direction) {
            Ok(action) => vec![
                format!(
                    "Prepared {:?} at tick {}",
                    action.direction, action.started_at.0
                ),
                format!("Joined limbs: {}", action.contributors.len()),
            ],
            Err(e) => vec![format!("Prepare failed: {e:?}")],
        };
    }
    fn join_limb(&mut self) {
        self.status = match self.action.join(isometer::core::PartId(2)) {
            Ok(()) => vec!["Joined limb part 2 into authoritative action.".into()],
            Err(e) => vec![format!("Limb join failed: {e:?}")],
        };
    }
    fn release(&mut self) {
        self.held = false;
        if self.action.action().is_none() {
            return;
        }
        let tick = self.action.action().map(|a| a.last_tick).unwrap();
        self.status = match self
            .action
            .release_against(self.target, tick, self.combat_rules)
        {
            Ok(events) => {
                let mut lines = vec![format!(
                    "Released volley against target: {} event(s)",
                    events.len()
                )];
                lines.extend(self.combat_lines(&events));
                lines
            },
            Err(e) => vec![format!("Release failed: {e:?}")],
        };
        self.redraw();
    }
    fn combat_lines(&self, events: &[GameEvent]) -> Vec<String> {
        events
            .iter()
            .find_map(|event| match event {
                GameEvent::VolleyResolved { strikes, .. } => Some(
                    strikes
                        .iter()
                        .enumerate()
                        .map(|(index, strike)| match strike.outcome {
                            eponym_play::StrikeOutcome::Miss => {
                                format!("Strike {}: MISS (source {})", index + 1, strike.source.0)
                            },
                            eponym_play::StrikeOutcome::Hit {
                                part,
                                quality,
                                harm,
                            } => format!(
                                "Strike {}: HIT target part {} quality {} harm {}",
                                index + 1,
                                part.0,
                                quality,
                                harm
                            ),
                        })
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_else(|| vec!["No volley resolution recorded".into()])
    }
    fn redraw(&self) {
        if let Some(live) = &self.live {
            live.window.request_redraw();
        }
    }
}
