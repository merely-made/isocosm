// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native input receipt for Paredros's bounded timed limb action.
use mesocosm_core::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
use paredros_client::body_sheet::Hud;
use paredros_client::gpu::Composer;
use paredros_identity::Tick;
use paredros_world::fixtures::three_lives;
use paredros_world::timed_action::{Direction, TimedActionRules, TimedActionSession};
use paredros_world::{
    CombatRules, GameEvent, GameIntent, GameState, ItemKind, ItemLocation, Session, World,
    WorldConfig,
};
use std::{
    env,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::Window,
};
#[path = "actions.rs"]
mod actions;
#[path = "host.rs"]
mod host;
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
    pub(crate) target: paredros_identity::SubjectId,
    pub(crate) combat_rules: CombatRules,
    pub(crate) target_item: paredros_world::ItemId,
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

impl App {
    fn new() -> Self {
        let subject = three_lives::KEEPER;
        let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
        let at = game
            .items()
            .all()
            .find_map(|item| match item.location {
                paredros_world::ItemLocation::At(at) if item.kind == ItemKind::Dressing => Some(at),
                _ => None,
            })
            .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Generate {
            tick,
            subject,
            body_seed: 1,
            at,
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Name {
            tick,
            subject,
            name: paredros_world::Name::new("Keeper").unwrap(),
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::AdmitAnatomy {
            tick,
            subject,
            revision: paredros_identity::BodyRevisionId(0),
            document: Box::new(three_lives::three_lives()[0].body.clone()),
        })
        .unwrap();
        support::configure_keeper(&mut game, subject);
        let target = paredros_identity::SubjectId(702);
        let target_item = game
            .items()
            .all()
            .find(|item| {
                matches!(item.kind, ItemKind::Dressing) && item.location == ItemLocation::At(at)
            })
            .copied()
            .expect("generated world has a reachable forward dressing for the target");
        let target_spawn = match target_item.location {
            ItemLocation::At(at) => at,
            _ => unreachable!(),
        };
        let tick = game.next_tick();
        game.apply(GameIntent::Generate {
            tick,
            subject: target,
            body_seed: 2,
            at: target_spawn,
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Name {
            tick,
            subject: target,
            name: paredros_world::Name::new("Target").unwrap(),
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::AdmitAnatomy {
            tick,
            subject: target,
            revision: paredros_identity::BodyRevisionId(0),
            document: Box::new(target_body()),
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Take {
            tick,
            subject: target,
            item: target_item.id,
        })
        .unwrap();
        for _ in 0..12 {
            game.apply(GameIntent::Move {
                tick: game.next_tick(),
                subject: target,
                toward: [target_spawn[0] + 12, target_spawn[1], target_spawn[2]],
            })
            .unwrap();
        }
        let target_at = game.movement().position(target).unwrap();
        assert!(
            target_at[0] - at[0] >= 9,
            "fixture bodies must be spatially separated"
        );
        assert!(
            game.world().ground().sees(
                [at[0] + 4, at[1] + 2, at[2] + 2],
                [target_at[0] + 4, target_at[1] + 2, target_at[2] + 2],
            ),
            "fixture requires a clear limb-height ray"
        );
        let tick = game.next_tick();
        game.apply(GameIntent::AttachItem {
            tick,
            subject: target,
            item: target_item.id,
            part: mesocosm_core::PartId(2),
            revision: paredros_identity::BodyRevisionId(0),
        })
        .unwrap();
        let part = PartRef {
            subject: subject.0,
            part: 0,
        };
        let network = FunctionalNetwork::new(
            vec![
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Source {
                        part,
                        capacity: 24,
                        charge: 24,
                    },
                },
                Node {
                    id: NodeId(2),
                    kind: NodeKind::Effect {
                        part: PartRef {
                            subject: subject.0,
                            part: 1,
                        },
                    },
                },
                Node {
                    id: NodeId(3),
                    kind: NodeKind::Source {
                        part,
                        capacity: 24,
                        charge: 24,
                    },
                },
                Node {
                    id: NodeId(4),
                    kind: NodeKind::Effect {
                        part: PartRef {
                            subject: subject.0,
                            part: 2,
                        },
                    },
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
        .unwrap();
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
        let path = env::var_os("PAREDROS_TIMED_ACTION_SAVE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("timed_action.save"));
        let session = Session::begin(game, subject).unwrap();
        let combat_rules = CombatRules {
            max_reach: 64,
            sever_threshold: 12,
            ..CombatRules::default()
        };
        let mut app = Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            live: None,
            action: TimedActionSession::begin(session, network, rules).unwrap(),
            target,
            combat_rules,
            target_item: target_item.id,
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
        assert!(self.action.join(mesocosm_core::PartId(2)).is_ok());
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
                .any(|strike| matches!(strike.outcome, paredros_world::StrikeOutcome::Hit { .. }))
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
        self.rest();
        let player = self.action.session().control().played();
        let game = self.action.session().game();
        assert_eq!(game.bodies().get(player).unwrap().wound, 0);
        assert!(
            game.current_anatomy(player)
                .unwrap()
                .document
                .part(mesocosm_core::PartId(1))
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
        self.status = match self.action.join(mesocosm_core::PartId(2)) {
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
                            paredros_world::StrikeOutcome::Miss => {
                                format!("Strike {}: MISS (source {})", index + 1, strike.source.0)
                            },
                            paredros_world::StrikeOutcome::Hit {
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
    fn save(&mut self) {
        self.status = match self
            .action
            .save()
            .map_err(|e| format!("{e:?}"))
            .and_then(|bytes| {
                let pending = self.save_path.with_extension("pending");
                std::fs::write(&pending, bytes).map_err(|e| e.to_string())?;
                std::fs::rename(&pending, &self.save_path).map_err(|e| e.to_string())
            }) {
            Ok(()) => vec![format!("Saved {}", self.save_path.display())],
            Err(e) => vec![format!("Save failed: {e}")],
        };
        self.redraw();
    }
    fn load(&mut self) {
        self.status = match std::fs::read(&self.save_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| TimedActionSession::restore(&bytes).map_err(|e| format!("{e:?}")))
        {
            Ok(action) => {
                if action.session().game().bodies().get(self.target).is_none() {
                    vec!["Load failed: saved session has no selected target".into()]
                } else {
                    self.action = action;
                    self.held = false;
                    self.movement_keys = [false; 4];
                    self.last_motion = Instant::now();
                    self.last_charge = Instant::now();
                    vec![format!("Loaded {}", self.save_path.display())]
                }
            },
            Err(e) => vec![format!("Load failed: {e}")],
        };
        self.redraw();
    }
    fn redraw(&self) {
        if let Some(live) = &self.live {
            live.window.request_redraw();
        }
    }
}
