// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! P2: one played Paredros session as a Cambium document.
//!
//! `session` is a working name. The real one is a naming round, not a session
//! default; the plan's open decisions record it that way.
//!
//! The window is `cambium_genet_winit_host::run`. The root view is a viewport
//! leaf whose producer is P1's [`SceneProducer`], plus a subject sheet, an
//! equipment panel and a status panel, all projections of the one
//! `paredros_world::Session` held inside [`SceneModel`]. Because
//! `TimedActionSession` owns its session by value and exposes no mutable
//! handle on it, the wrapper goes *inside* the model's slot
//! (`SceneModel::timed`) rather than beside it — see `producer::Held`.
//!
//! # Declared limits
//!
//! - **No key releases.** `cambium_rootstock` routes presses only; the winit
//!   event source keeps releases for exactly one thing (letting go of Tab).
//!   So a held movement key is modelled as a *latch* refreshed by the
//!   platform's auto-repeat: a fresh press holds [`LATCH_FIRST`] to outlast
//!   the repeat delay, each repeat holds [`LATCH_REPEAT`]. Releasing a key
//!   therefore coasts for up to `LATCH_REPEAT`, and a single tap walks for
//!   `LATCH_FIRST`. Charging is a toggle for the same reason: Space begins it,
//!   Space again releases the strike. The left mouse button *does* carry
//!   press and release, so holding it over the scene is the faithful verb.
//! - **No function keys.** `cambium_rootstock::NamedKey` has no `F5`/`F9`
//!   case and `key_from_winit` lowers every unlisted named key to
//!   `NamedKey::Other`, so the two arrive indistinguishable. Save and load are
//!   bound to Ctrl+S and Ctrl+L here, and to the Save/Load buttons.
//! - **Picking ignores terrain**, inherited from
//!   [`SceneModelSource::pick_body`].

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use cambium_genet_winit_host::{CloseDisposition, HostHooks, HostOptions, Init, Key, NamedKey};
use isomere::{Binding, Keymap};
use isometer::core::PartId;
use paredros_client::producer::{SceneHandle, SceneModel, SceneModelSource, SceneProducer};
use paredros_identity::SubjectId;
use paredros_world::fixtures::session as session_fixture;
use paredros_world::glyphs::GlyphReading;
use paredros_world::timed_action::Direction;
use paredros_world::{CombatRules, ItemId};

#[path = "actions.rs"]
mod actions;
#[path = "panels.rs"]
mod panels;
#[path = "probe.rs"]
mod probe;
#[path = "smoke.rs"]
mod smoke;
#[path = "view.rs"]
mod view;

/// The one custom-leaf key this document owns. "PARD".
pub(crate) const LEAF_KEY: u64 = 0x5041_5244;

pub(super) const MOTION_INTERVAL: Duration = Duration::from_nanos(16_666_667);
pub(super) const CHARGE_INTERVAL: Duration = Duration::from_millis(100);
/// How long a fresh movement press stays latched: long enough to outlast a
/// platform auto-repeat delay, so a held key does not stutter.
const LATCH_FIRST: Duration = Duration::from_millis(700);
/// How long each auto-repeat extends the latch. Short, so a released key stops
/// promptly once the repeats stop arriving.
const LATCH_REPEAT: Duration = Duration::from_millis(150);

/// One key press as the declaration below compares it: a character key,
/// lower-cased, with its Ctrl state, or a named key.
///
/// The one normalization this host does. Named keys carry no Ctrl state
/// because the handler never read one for them, and lowering happens here so
/// the declaration is written the way a person would read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Press {
    Char(String, bool),
    Named(NamedKey),
}

impl Press {
    /// A plain character key.
    fn character(key: &str) -> Self {
        Self::Char(key.to_owned(), false)
    }

    /// The same key held with Ctrl.
    fn with_ctrl(key: &str) -> Self {
        Self::Char(key.to_owned(), true)
    }

    /// What the platform reported, as the declaration compares it. `None` for
    /// an unidentified or dead key, which claims nothing.
    fn of(key: &Key, ctrl: bool) -> Option<Self> {
        match key {
            Key::Character(character) => Some(Self::Char(character.to_ascii_lowercase(), ctrl)),
            Key::Named(named) => Some(Self::Named(*named)),
            _ => None,
        }
    }
}

/// What a press asks this session to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// W, S, A, D, by their latch index.
    Move(usize),
    Aim(Direction),
    Charge,
    Take,
    Rest,
    Injure,
    Join,
    Save,
    Load,
    Close,
}

/// The session's keys, declared once (M3 of the isomere plan).
///
/// This list is both halves of what used to be two: [`SessionApp::key`]
/// dispatches through [`Keymap::command`], and the header's control-help line
/// is [`Keymap::help`] over the same entries. A binding cannot move without
/// the sentence moving with it, and a key that is not here cannot be handled.
/// The pointer verb is declared with no press of its own because the help line
/// still has to name it; the handler for it is on the scene card.
pub(crate) static KEYMAP: LazyLock<Keymap<'static, Press, Command>> = LazyLock::new(|| {
    Keymap::new(vec![
        Binding::group(
            "WASD",
            "move",
            vec![
                (Press::character("w"), Command::Move(0)),
                (Press::character("s"), Command::Move(1)),
                (Press::character("a"), Command::Move(2)),
                (Press::character("d"), Command::Move(3)),
            ],
        ),
        Binding::group(
            "arrows",
            "aim",
            vec![
                (
                    Press::Named(NamedKey::ArrowUp),
                    Command::Aim(Direction::Forward),
                ),
                (
                    Press::Named(NamedKey::ArrowDown),
                    Command::Aim(Direction::Backward),
                ),
                (
                    Press::Named(NamedKey::ArrowLeft),
                    Command::Aim(Direction::Left),
                ),
                (
                    Press::Named(NamedKey::ArrowRight),
                    Command::Aim(Direction::Right),
                ),
            ],
        ),
        Binding::one(
            "Space",
            "charge, Space again to strike",
            Press::Named(NamedKey::Space),
            Command::Charge,
        ),
        Binding::said("left mouse hold on the scene", "to charge"),
        Binding::one("E", "take", Press::character("e"), Command::Take),
        Binding::one("R", "rest", Press::character("r"), Command::Rest),
        Binding::one("I", "injury", Press::character("i"), Command::Injure),
        Binding::one("J", "join part 2", Press::character("j"), Command::Join),
        Binding::one("Ctrl+S", "save", Press::with_ctrl("s"), Command::Save),
        Binding::one("Ctrl+L", "load", Press::with_ctrl("l"), Command::Load),
        Binding::one(
            "Esc",
            "close",
            Press::Named(NamedKey::Escape),
            Command::Close,
        ),
    ])
});

pub(crate) struct SessionApp {
    /// The one session, plus the presentation policy the producer reads.
    pub(crate) model: SceneHandle,
    pub(crate) scene: Rc<RefCell<SceneProducer>>,
    pub(crate) target: SubjectId,
    pub(crate) target_item: ItemId,
    pub(crate) combat_rules: CombatRules,
    pub(crate) direction: Direction,
    pub(crate) charging: bool,
    /// W, S, A, D latch expiry. See the module's declared limits.
    pub(crate) latched: [Option<Instant>; 4],
    pub(crate) last_motion: Instant,
    pub(crate) last_charge: Instant,
    /// The opt-in acquisition journal. `None` when the rules were refused, so
    /// a rejected canon costs the session nothing. Never in any save.
    pub(crate) glyphs: Option<GlyphReading>,
    pub(crate) status: Vec<String>,
    pub(crate) selected: Option<(SubjectId, PartId)>,
    pub(crate) published_error: Option<String>,
    pub(crate) save_path: PathBuf,
    /// Completed saves and loads. The scenario's `save-loaded` reading.
    pub(crate) saves: u32,
    pub(crate) loads: u32,
    pub(crate) closing: bool,
}

impl SessionApp {
    /// One key press. Returns `true` when this host consumed it, so ordinary
    /// document keys (Tab, typing) still reach the tree.
    ///
    /// Every branch below answers a [`Command`] the declared [`KEYMAP`] handed
    /// back, so an unclaimed key falls through untouched and a claimed one
    /// that grew no arm here is a compile error rather than a dead binding in
    /// the help line.
    fn key(&mut self, key: &Key, ctrl: bool, repeat: bool) -> bool {
        let Some(press) = Press::of(key, ctrl) else {
            return false;
        };
        let Some(command) = KEYMAP.command(&press).copied() else {
            return false;
        };
        match command {
            // A held movement key refreshes the latch rather than repeating.
            Command::Move(index) => {
                let hold = if repeat { LATCH_REPEAT } else { LATCH_FIRST };
                if self.latched[index].is_none_or(|until| until <= Instant::now()) {
                    self.last_motion = Instant::now();
                }
                actions::latch(&mut self.latched[index], hold);
            },
            // Swallow the repeat: a held Space must not toggle sixty times.
            Command::Charge if repeat => {},
            Command::Charge => {
                if self.charging {
                    self.release();
                } else {
                    self.begin_charge();
                }
            },
            Command::Aim(direction) => self.prepare(direction),
            Command::Close => self.closing = true,
            Command::Save => self.save(),
            Command::Load => self.load(),
            // The four world verbs are one press, one deed: an auto-repeat
            // falls through rather than queueing a dozen of them.
            _ if repeat => return false,
            Command::Take => self.take_dressing(),
            Command::Rest => self.rest(),
            Command::Injure => self.injure(),
            Command::Join => self.join_limb(PartId(2)),
        }
        true
    }

    /// Whether accepted events are waiting for the journal to read them.
    pub(crate) fn glyphs_pending(&self) -> bool {
        let model = self.model.borrow();
        self.glyphs
            .as_ref()
            .is_some_and(|reading| reading.pending(model.game()))
    }

    /// Reads whatever the world accepted this frame. The reading changes no
    /// world fact, so this is safe to run after every dispatched intent.
    pub(crate) fn advance_glyphs(&mut self) {
        let Some(reading) = self.glyphs.as_mut() else {
            return;
        };
        reading.advance(self.model.borrow().game());
    }

    /// True while a frame is still owed to an animation.
    fn animating(&self) -> bool {
        self.charging || self.motion_running()
    }
}

/// Builds the fixture world, opens the window, and returns the process code.
pub fn run() -> i32 {
    let Some(options) = probe::options() else {
        return 0;
    };
    let fixture = session_fixture::timed_action_world();
    let played = fixture.keeper;
    let target = fixture.target;
    let target_item = fixture.target_item;
    let combat_rules = fixture.combat_rules;
    let model = SceneModel::timed(fixture.action, played).into_handle();
    let glyphs = {
        let rules = session_fixture::demonstration_rules(played);
        match GlyphReading::new(rules, model.borrow().game()) {
            Ok(reading) => Some(reading),
            Err(why) => {
                eprintln!("session: acquisition journal disabled: {why}");
                None
            },
        }
    };
    let scene = Rc::new(RefCell::new(SceneProducer::new(SceneModelSource::new(
        model.clone(),
    ))));
    let saves = std::env::var_os("PAREDROS_SESSION_SAVES")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("paredros-session"));
    let smoking = std::env::var_os("PAREDROS_SESSION_SMOKE").is_some();
    let state = SessionApp {
        model,
        scene,
        target,
        target_item,
        combat_rules,
        direction: Direction::Right,
        charging: false,
        latched: [None; 4],
        glyphs,
        last_motion: Instant::now(),
        last_charge: Instant::now(),
        status: vec!["Ready: aim with the arrows, then charge and strike.".into()],
        selected: None,
        published_error: None,
        save_path: saves.join("session.save"),
        saves: 0,
        loads: 0,
        closing: false,
    };
    let exit = Rc::new(Cell::new(0));
    let lane = Rc::new(RefCell::new(
        (smoking && !options.driven).then(|| smoke::Lane::new(saves, exit.clone())),
    ));
    let driven = Rc::new(RefCell::new(options.driven.then(|| {
        probe::Lane::new(
            options.scenario,
            options.receipt,
            options.capture,
            exit.clone(),
            options.frames,
        )
    })));
    let close_lane = lane.clone();
    let after_lane = lane.clone();
    let close_driven = driven.clone();
    let after_driven = driven.clone();
    let frame_driven = driven.clone();
    let hooks: HostHooks<SessionApp, view::Logic, view::Child> = HostHooks {
        frame: Box::new(move |ctx| {
            if ctx.runner.state().closing {
                *ctx.close = true;
            }
            if !ctx.producers.contains(LEAF_KEY) {
                let scene = ctx.runner.state().scene.clone();
                ctx.producers
                    .register(LEAF_KEY, scene, &["color"])
                    .expect("the session document owns one producer key");
            }
            if ctx.runner.state().animating() {
                ctx.runner.update(|state| {
                    state.tick_motion();
                    state.tick_charge();
                });
            }
            if ctx.runner.state().glyphs_pending() {
                ctx.runner.update(SessionApp::advance_glyphs);
            }
            ctx.runner.state().animating()
                || lane.borrow().is_some()
                || frame_driven.borrow().is_some()
        }),
        after_dispatch: Box::new(|ctx| {
            if let Some(window) = ctx.window {
                window.request_redraw();
            }
        }),
        after_frame: Box::new(move |ctx| {
            let error = ctx
                .runner
                .state()
                .scene
                .borrow()
                .last_error()
                .map(str::to_owned)
                .or_else(|| {
                    ctx.producers
                        .error(LEAF_KEY)
                        .map(|why| format!("Viewport unavailable: {why:?}"))
                });
            if ctx.runner.state().published_error != error {
                ctx.runner.update(|state| state.published_error = error);
                if let Some(window) = ctx.window {
                    window.request_redraw();
                }
            }
            if let Some(lane) = after_lane.borrow_mut().as_mut() {
                lane.after_frame(ctx);
            }
            if let Some(lane) = after_driven.borrow_mut().as_mut() {
                lane.after_frame(ctx);
            }
        }),
        after_wake: Box::new(|_| {}),
        close_request: Box::new(move |ctx, _| {
            if let Some(lane) = close_lane.borrow_mut().as_mut() {
                lane.refuse();
            }
            if let Some(lane) = close_driven.borrow_mut().as_mut() {
                lane.request_close();
            }
            if let Some(window) = ctx.window {
                window.request_redraw();
            }
            CloseDisposition::Exit
        }),
        focused_text: Box::new(|_| None),
        key_intercept: Box::new(|runner, press| {
            let (key, ctrl, repeat) = (press.key.clone(), press.modifiers.ctrl, press.repeat);
            let mut consumed = false;
            runner.update(|state| consumed = state.key(&key, ctrl, repeat));
            consumed
        }),
    };
    let options = HostOptions {
        title: "Paredros · Session".into(),
        initial_logical_size: options.size,
        ..Default::default()
    };
    if let Err(why) = cambium_genet_winit_host::run(
        options,
        move |_, _, _| Init {
            state,
            logic: view::root as view::Logic,
            sheet: view::SHEET.clone(),
            fonts: Vec::new(),
            images: Vec::new(),
        },
        hooks,
    ) {
        eprintln!("session: event loop failed: {why}");
        return 1;
    }
    exit.get()
}
