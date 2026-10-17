// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One core runtime, one native picture, and Cambium controls in Assembly.

use super::{Host, HostConfig};
use crate::borrowed_section::{Presentation, SceneHandle, producer::SectionProducer};
use isomere::host::{
    Assembly, CloseDisposition, CloseRequest, Ctx, HostOptions, Init, KeyPress, Product, Runner,
};
use mesocosm_runtime::{Button, Pick, Pointed};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use winit::keyboard::{Key, NamedKey};

mod probe;
mod view;

const LEAF: u64 = 0x4d45_534f;
const MAP: u64 = 0x4d45_4d50;

pub(super) struct State {
    host: Host,
    picture: SceneHandle,
    producer: Rc<RefCell<SectionProducer>>,
    error: Option<String>,
    notice: Option<String>,
    last_point: Option<Pick>,
    vitals: mesocosm_views::Vitals,
    board: mesocosm_views::Board,
    question: mesocosm_views::Succession,
    dev: mesocosm_views::Dev,
    question_open: bool,
    frame_us: Vec<u64>,
    commands: Option<isomere::host::WindowCommands>,
    map_key: Option<(u64, Option<u64>)>,
    minimap: Option<mesocosm_views::MinimapLeaf>,
}

impl State {
    fn new(config: HostConfig) -> Self {
        let picture = SceneHandle::default();
        let producer = Rc::new(RefCell::new(SectionProducer::new(picture.clone())));
        Self {
            host: Host::new(config),
            picture,
            producer,
            error: None,
            notice: None,
            last_point: None,
            vitals: Default::default(),
            board: Default::default(),
            question: empty_question(),
            dev: Default::default(),
            question_open: false,
            frame_us: Vec::new(),
            commands: None,
            map_key: None,
            minimap: None,
        }
    }

    fn refresh(&mut self) {
        let host = &mut self.host;
        host.refresh_scene();
        host.update_follow();
        host.refresh_continuation();
        host.refresh_directing();
        let r = &host.runtime;
        let sim = r.sim();
        let map_key = (r.revision().unwrap_or(0), r.critter());
        if self.map_key != Some(map_key) {
            self.minimap = Some(mesocosm_views::minimap_leaf(&sim, r.critter()));
            self.map_key = Some(map_key);
        }
        self.vitals = mesocosm_views::vitals_of_with_terms(
            &sim,
            r.critter(),
            30,
            r.fault().map(str::to_owned),
            Some(&r.trend()),
            r.terms(),
        );
        self.board = r
            .review()
            .map(|review| crate::review::board_of(review, r.proposed(), &r.trend(), host.board_row))
            .unwrap_or_default();
        let question = host
            .continuation_reading()
            .or_else(|| host.directing_reading())
            .or_else(|| {
                r.checkpoint()
                    .filter(|_| r.review().is_none())
                    .map(|c| crate::succession::succession_of(&sim, c))
            });
        self.question_open = question.is_some();
        self.question = question.unwrap_or_else(empty_question);
        if let Some(dev) = host.dev_reading() {
            self.dev = dev;
        }
        let half = crate::section::half_height_or_default(host.config.slab_half_height);
        let at = host.follow_at();
        if let Some(scene) = &host.scene {
            let presentation = Presentation {
                forward: host.config.camera.forward(),
                centre: crate::section::centre_on(at, host.pan, half, host.config.camera),
                half_height: half,
                bodies: host.config.body_mode,
                budget: host.config.body_budget,
                light: host.config.body_light,
                terrain: host.config.terrain_style,
                terrain_y: at[1] as f32,
                focus: host.config.dev.then(|| host.followed()).flatten(),
                selection: host.inspection.selected,
            };
            if let Err(why) =
                self.picture
                    .present(scene, r.revision().unwrap_or(0), presentation)
            {
                self.error = Some(why);
            }
        } else {
            self.picture.clear();
        }
    }

    fn press(&mut self, key: &Key) {
        if self.host.try_continuation_key(key) || self.host.try_directing_key(key) {
            self.refresh();
            return;
        }
        if matches!(key, Key::Named(NamedKey::Escape)) && !self.host.directing_open() {
            if let Some(commands) = &self.commands {
                commands.close();
            }
            return;
        }
        if self.host.try_camera_key(key) {
            self.refresh();
            return;
        }
        let followed = self.host.followed();
        if !matches!(
            key,
            Key::Named(
                NamedKey::ArrowLeft
                    | NamedKey::ArrowRight
                    | NamedKey::ArrowUp
                    | NamedKey::ArrowDown
            )
        ) && self.host.inspection.key(
            key,
            self.host.config.dev,
            followed,
            |id, current, backwards| {
                self.producer
                    .borrow_mut()
                    .select_part(id, current, backwards)
            },
            |selection| self.producer.borrow_mut().validate_selection(selection),
        ) {
            self.refresh();
            return;
        }
        match key {
            Key::Named(NamedKey::ArrowLeft) => self.host.pan.x -= crate::section::PAN_STEP,
            Key::Named(NamedKey::ArrowRight) => self.host.pan.x += crate::section::PAN_STEP,
            Key::Named(NamedKey::ArrowUp)
                if !self.host.continuation_open() && !self.host.directing_open() =>
            {
                self.host.pan.y += crate::section::PAN_STEP
            },
            Key::Named(NamedKey::ArrowDown)
                if !self.host.continuation_open() && !self.host.directing_open() =>
            {
                self.host.pan.y -= crate::section::PAN_STEP
            },
            _ => self.host.press_key(key),
        }
        self.refresh();
    }

    fn point(&mut self, event: cambium::PointerEvent) {
        if event.phase != cambium::PointerPhase::Down
            || self.host.runtime.checkpoint().is_some()
            || self.host.continuation_open()
        {
            return;
        }
        let button = match event.button {
            cambium::PointerButton::Primary => Button::Primary,
            cambium::PointerButton::Secondary => Button::Secondary,
            _ => return,
        };
        let result = self.producer.borrow_mut().pick(
            (event.local.0 as f64, event.local.1 as f64),
            (event.size.0 as f64, event.size.1 as f64),
        );
        let result = result.and_then(|(pick, revision)| {
            let pick = match pick {
                Some(isometer::Pick::Body(body)) => Pick::Thing(body.address.subject.0),
                Some(isometer::Pick::Terrain(_)) => {
                    Pick::Place(self.host.scene.as_ref().ok_or("picture withdrawn")?.site)
                },
                None => return Ok(()),
            };
            let pointed = Pointed {
                pick,
                button,
                revision,
            };
            self.host
                .runtime
                .current(&pointed)
                .map_err(|why| format!("{why:?}"))?;
            let target = match pick {
                Pick::Thing(id) => {
                    isocosm_overlay::mesocosm::NudgeTarget::Thing(isocosm_overlay::EntityHandle(id))
                },
                Pick::Place(id) => {
                    isocosm_overlay::mesocosm::NudgeTarget::Place(isocosm_overlay::PlaceHandle(id))
                },
                Pick::Nothing => return Ok(()),
            };
            if button == Button::Secondary {
                self.host.open_directing_in_view(target);
            } else {
                self.host.try_directing_key(&Key::Named(NamedKey::Escape));
                let context = self.host.input_context();
                self.host
                    .runtime
                    .point(&pointed, &context)
                    .map_err(|why| format!("{why:?}"))?;
            }
            self.last_point = Some(pick);
            Ok(())
        });
        if let Err(why) = result {
            self.notice = Some(why);
        } else {
            self.notice = None;
        }
        self.refresh();
    }
}

fn empty_question() -> mesocosm_views::Succession {
    mesocosm_views::Succession {
        headline: String::new(),
        facts: Vec::new(),
        stay: String::new(),
        take: None,
    }
}

struct WindowProduct;
impl Product for WindowProduct {
    type State = State;
    type Logic = view::Logic;
    type View = view::Child;
    type Producer = Rc<RefCell<SectionProducer>>;
    const LOG_PREFIX: &'static str = "mesocosm";
    fn viewport_keys(&self, _: &Ctx<'_, Self>, keys: &mut Vec<u64>) {
        keys.push(LEAF);
    }
    fn producer(&self, ctx: &Ctx<'_, Self>, _: u64) -> Option<Self::Producer> {
        Some(ctx.runner.state().producer.clone())
    }
    fn published_error(s: &State) -> Option<&str> {
        s.error.as_deref()
    }
    fn publish_error(s: &mut State, e: Option<String>) {
        s.error = e;
    }
    fn product_error(&self, ctx: &Ctx<'_, Self>) -> Option<String> {
        ctx.runner.state().picture.last_error()
    }
    fn frame(&mut self, ctx: &mut Ctx<'_, Self>) -> bool {
        ctx.runner.update(|s| {
            let before = std::time::Instant::now();
            s.host.advance();
            s.host.note_outcomes();
            s.refresh();
            if s.frame_us.len() < 10000 {
                s.frame_us.push(before.elapsed().as_micros() as u64);
            }
        });
        let mut minimap = None;
        ctx.runner.update(|s| minimap = s.minimap.take());
        if let Some(minimap) = minimap {
            ctx.leaves.insert(MAP, Box::new(minimap));
        }
        true
    }
    fn after_frame(&mut self, ctx: &mut Ctx<'_, Self>) {
        ctx.runner.update(|s| {
            s.host.frames += 1;
            s.host.mark_continuation_presented();
            let followed = s.host.followed();
            s.host.inspection.update(followed, |selection| {
                s.producer.borrow_mut().validate_selection(selection)
            });
        });
    }
    fn key(
        &mut self,
        runner: &mut Runner<State, view::Logic, view::Child>,
        press: &KeyPress,
    ) -> bool {
        if press.repeat {
            use cambium_genet_winit_host::{Key as K, NamedKey as N};
            let host = &runner.state().host;
            let pan = matches!(press.key, K::Named(N::ArrowLeft | N::ArrowRight))
                || (matches!(press.key, K::Named(N::ArrowUp | N::ArrowDown))
                    && !host.continuation_open()
                    && !host.directing_open());
            if !pan {
                return true;
            }
        }
        let key = match &press.key {
            cambium_genet_winit_host::Key::Character(c) => Key::Character(c.clone().into()),
            cambium_genet_winit_host::Key::Named(n) => {
                use cambium_genet_winit_host::NamedKey as N;
                Key::Named(match n {
                    N::Space => NamedKey::Space,
                    N::Enter => NamedKey::Enter,
                    N::Tab => NamedKey::Tab,
                    N::Escape => NamedKey::Escape,
                    N::ArrowUp => NamedKey::ArrowUp,
                    N::ArrowDown => NamedKey::ArrowDown,
                    N::ArrowLeft => NamedKey::ArrowLeft,
                    N::ArrowRight => NamedKey::ArrowRight,
                    _ => return false,
                })
            },
            _ => return false,
        };
        runner.update(|s| s.press(&key));
        true
    }
    fn close_request(&mut self, _: &mut Ctx<'_, Self>, _: CloseRequest) -> CloseDisposition {
        CloseDisposition::KeepVisible
    }
}

pub(super) fn run(config: HostConfig) -> i32 {
    let frames = config.frames;
    let capture = config.capture.clone();
    let receipt = config.receipt.clone();
    let options = HostOptions {
        title: if config.bench_world {
            "Mesocosm · Bench world"
        } else {
            "Mesocosm"
        }
        .into(),
        initial_logical_size: (config.width as f64, config.height as f64),
        ..Default::default()
    };
    let mut state = State::new(config);
    state.refresh();
    let scenario = state.host.scenario.take();
    let exit = Rc::new(Cell::new(0));
    let lane = mesquite::Lane::new(probe::Probe, scenario, receipt, capture, exit.clone())
        .with_frame_limit(frames);
    if let Err(why) =
        Assembly::new(WindowProduct)
            .with_lane(lane)
            .run(options, move |_, commands, _| {
                state.commands = Some(commands.clone());
                Init {
                    state,
                    logic: view::root as view::Logic,
                    sheet: view::sheet(),
                    fonts: Vec::new(),
                    images: Vec::new(),
                }
            })
    {
        eprintln!("mesocosm: {why}");
        return 1;
    }
    exit.get()
}
