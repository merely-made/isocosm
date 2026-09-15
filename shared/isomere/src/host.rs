// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The host assembly (M4 of the isomere plan).
//!
//! `cambium-genet-winit-host` already owns the window, the surface, layout,
//! paint, input routing and accessibility. What it deliberately does *not*
//! own is the seven [`HostHooks`] closures, and the 2026-09-15 inventory found
//! the three products filling the same five things into them by hand:
//!
//! 1. **Producer registration for viewport leaves** — `if !contains(key) {
//!    register(key, producer, &["color"]) }`, written once per leaf in the
//!    bench and once in the session.
//! 2. **The mesquite scenario lane**, ticked in `after_frame` and told about a
//!    close in `close_request`, with a redraw so the deferred close can present
//!    its last frame.
//! 3. **Capture arming** — Isometry's bounded `ISOMETRY_CAPTURE_DIR` policy,
//!    which is the same policy under any other env name.
//! 4. **The error line**, published from the product's own scene error or the
//!    first producer error among its leaves, compared against what is already
//!    on screen so an unchanged string rebuilds no tree.
//! 5. **The frame profile line**, printed from `AppCtx::frame_profile` behind
//!    the product's own switch.
//!
//! [`Assembly`] is those five filled once. What is left is [`Product`]: the
//! product's state, its root view, its own frame work, its keymap dispatch and
//! its actions. The plan's §6 finding sets the expectation — mesquite left each
//! product about 400 lines of "what is about this host", and so does this.
//!
//! # What Cambium already offered, and what it did not
//!
//! Read at 876320fd and **not** adopted, because each solves a different
//! problem: `cambium_genet_winit_host::Harness` is the *windowless* host for
//! unit tests, and `inert_hooks` is a `HostHooks` with every closure empty —
//! a starting point for a test, not an assembly. `mesquite::Lane` is the
//! scenario lifecycle and is adopted wholesale here rather than reimplemented;
//! what this module adds is the *wiring* of it into two of the seven hooks,
//! which mesquite deliberately leaves to the consumer and which all three
//! products therefore wrote by hand. `IdlePolicy` and the frame-pacing
//! machinery are the host's and are not touched.
//!
//! Nothing here is upstream-able as it stands: the five parts above are joined
//! by *what the wing's products happen to do*, not by anything a second
//! non-game Cambium consumer would want. §3's boundary rule keeps it here.
//!
//! # Why the feature gate
//!
//! `cambium-genet-winit-host` drags wgpu, netrender and winit in. The panels
//! above emit elements and nothing else, and `isometry-views` and
//! `mesocosm-views` consume them, so this module is behind the `host` feature
//! and those two crates never pay for a renderer. The three hosts turn it on.

use std::cell::RefCell;
use std::rc::Rc;

use cambium_genet_winit_host::{AppCtx, HostHooks, TextureProducer};
use cambium_rootstock::meristem_bounds::RootView;

mod capture;

pub use capture::{Capture, CaptureNames};

/// Re-exported so a [`Product`] impl names one crate rather than two for the
/// handful of host types its own signatures have to spell. The same courtesy
/// `isomere` already does for `tinct`'s `Srgb`.
pub use cambium_genet_winit_host::{
    CloseDisposition, CloseRequest, FocusedTextSlot, HostOptions, HostWake, HostWindow, Init,
    KeyPress, ProducerError, Runner, WindowCommands,
};

/// The host context a [`Product`]'s hooks are handed, spelled once. The same
/// shape and the same reason as [`mesquite::Ctx`].
pub type Ctx<'a, P> =
    AppCtx<'a, <P as Product>::State, <P as Product>::Logic, <P as Product>::View>;

/// The words a producer error reads as when a product does not say otherwise.
/// Both the bench and the session wrote this sentence.
pub const VIEWPORT_ERROR: &str = "Viewport unavailable";

/// A product with no viewport producer of its own.
///
/// Isometry registers painted leaves rather than same-device producers, so its
/// [`Product::Producer`] is this: a type that satisfies the bound and is never
/// constructed, because [`Product::viewport_keys`] names no key.
pub enum NoProducer {}

impl TextureProducer for NoProducer {
    fn render(
        &mut self,
        _cx: &cambium_genet_winit_host::ProducerContext<'_>,
    ) -> Option<cambium_genet_winit_host::ProducedTexture> {
        match *self {}
    }
}

/// A scenario lane the assembly drives.
///
/// The two calls all three products already make on `mesquite::Lane`, named as
/// a trait so the assembly never has to name the lane's own product type. A
/// host with two lanes — Paredros's scripted smoke and its driven acceptance —
/// hands over two, and a host with none hands over none.
pub trait ScenarioLane<P: Product> {
    /// Tick the lane on a presented frame.
    fn after_frame(&mut self, ctx: &mut Ctx<'_, P>);

    /// A close arrived. The lane decides what that means to it; the host's
    /// disposition stays [`Product::close_request`]'s.
    fn request_close(&mut self) {}

    /// Whether this lane still needs frames delivered. A lane that exists at
    /// all does, which is what every product's frame hook already said.
    fn wants_frames(&self) -> bool {
        true
    }
}

impl<P, Q> ScenarioLane<P> for mesquite::Lane<Q>
where
    P: Product,
    Q: mesquite::Product<State = P::State, Logic = P::Logic, View = P::View>,
{
    fn after_frame(&mut self, ctx: &mut Ctx<'_, P>) {
        mesquite::Lane::after_frame(self, ctx);
    }

    fn request_close(&mut self) {
        mesquite::Lane::request_close(self);
    }
}

/// What is about *this* host, and nothing the assembly can fill.
///
/// Every method but the three associated types has a defensible default, so a
/// product that wants only the error line and a lane implements none of them.
pub trait Product: Sized + 'static {
    /// The application state the runner owns.
    type State: 'static;
    /// The view logic the runner diffs into a DOM.
    type Logic: FnMut(&Self::State) -> Self::View + 'static;
    /// The root view that logic produces.
    type View: RootView<Self::State>;
    /// The producer behind this product's viewport leaves. [`NoProducer`] for
    /// a product that registers none.
    type Producer: TextureProducer + 'static;

    /// The prefix on every line the assembly prints.
    const LOG_PREFIX: &'static str;

    // --- viewport leaves -----------------------------------------------------

    /// The leaf keys whose producers this product owns *this frame*, appended
    /// to `keys` (which arrives empty). A key that is already registered is
    /// skipped, so a product names the keys its tree currently holds and lets
    /// the assembly work out which are new.
    fn viewport_keys(&self, ctx: &Ctx<'_, Self>, keys: &mut Vec<u64>) {
        let _ = (ctx, keys);
    }

    /// The producer for `key`. `None` refuses the registration for this frame,
    /// which is how a leaf whose world has not been built yet stays unclaimed.
    fn producer(&self, ctx: &Ctx<'_, Self>, key: u64) -> Option<Self::Producer> {
        let _ = (ctx, key);
        None
    }

    /// The properties a registration declares. `&["color"]` everywhere so far.
    fn producer_properties(&self, key: u64) -> &'static [&'static str] {
        let _ = key;
        &["color"]
    }

    // --- the error line ------------------------------------------------------

    /// An error the product itself knows about, which outranks a producer's.
    /// The session's scene reports its own refusals this way.
    fn product_error(&self, ctx: &Ctx<'_, Self>) -> Option<String> {
        let _ = ctx;
        None
    }

    /// How this product words a producer error. `None` is [`VIEWPORT_ERROR`]
    /// plus the debug form, which is the sentence both sources already wrote.
    fn viewport_error(&self, key: u64, why: ProducerError) -> Option<String> {
        let _ = (key, why);
        None
    }

    /// The error line currently on screen, so an unchanged one rebuilds
    /// nothing.
    fn published_error(state: &Self::State) -> Option<&str> {
        let _ = state;
        None
    }

    /// Put `error` on screen. Called inside a `runner.update`, and only when it
    /// differs from [`published_error`](Product::published_error).
    fn publish_error(state: &mut Self::State, error: Option<String>) {
        let _ = (state, error);
    }

    // --- the seven hooks, minus what the assembly fills ----------------------

    /// Whether the product has asked to end. Read at the top of every frame.
    fn closing(state: &Self::State) -> bool {
        let _ = state;
        false
    }

    /// The product's own frame work: animation ticks, leaf syncing, backend
    /// polling. Return `true` to keep frames coming; a live lane already does.
    fn frame(&mut self, ctx: &mut Ctx<'_, Self>) -> bool {
        let _ = ctx;
        false
    }

    /// Whether a bounded capture may be armed now — a product's "nothing is
    /// still settling". Ignored when the assembly holds no [`Capture`].
    fn capture_ready(&self, ctx: &Ctx<'_, Self>) -> bool {
        let _ = ctx;
        true
    }

    /// The tail of every input dispatch.
    fn after_dispatch(&mut self, ctx: &mut Ctx<'_, Self>) {
        let _ = ctx;
    }

    /// Per-presented-frame product work, before the lanes tick.
    fn after_frame(&mut self, ctx: &mut Ctx<'_, Self>) {
        let _ = ctx;
    }

    /// Drain the product's own worker channel.
    fn after_wake(&mut self, ctx: &mut Ctx<'_, Self>) {
        let _ = ctx;
    }

    /// What a close means. The lanes have already been told.
    fn close_request(
        &mut self,
        ctx: &mut Ctx<'_, Self>,
        request: CloseRequest,
    ) -> CloseDisposition {
        let _ = (ctx, request);
        CloseDisposition::Exit
    }

    /// The text seam: which field has focus.
    fn focused_text(
        runner: &Runner<Self::State, Self::Logic, Self::View>,
    ) -> Option<FocusedTextSlot<Self::State>> {
        let _ = runner;
        None
    }

    /// The pre-dispatch keyboard intercept. `true` consumes the press. This is
    /// where a product dispatches through its [`Keymap`](crate::Keymap).
    fn key(
        &mut self,
        runner: &mut Runner<Self::State, Self::Logic, Self::View>,
        press: &KeyPress,
    ) -> bool {
        let _ = (runner, press);
        false
    }

    /// Whether to print the frame profile line. Isometry's `ISOMETRY_PROFILE=1`.
    fn profiling(&self) -> bool {
        false
    }
}

/// The product, its lanes and its capture policy, assembled into the seven
/// hooks.
pub struct Assembly<P: Product> {
    product: P,
    lanes: Vec<Box<dyn ScenarioLane<P>>>,
    capture: Option<Capture>,
    /// Reused across frames so leaf naming allocates once rather than per
    /// frame. The frame hook clears it before every use.
    keys: Vec<u64>,
}

impl<P: Product> Assembly<P> {
    /// An assembly over `product` with no lane and no capture policy.
    pub fn new(product: P) -> Self {
        Self {
            product,
            lanes: Vec::new(),
            capture: None,
            keys: Vec::new(),
        }
    }

    /// Add a lane. Lanes tick in the order they were added.
    pub fn with_lane(mut self, lane: impl ScenarioLane<P> + 'static) -> Self {
        self.lanes.push(Box::new(lane));
        self
    }

    /// Add a lane a flag may have declined to build.
    pub fn with_optional_lane(self, lane: Option<impl ScenarioLane<P> + 'static>) -> Self {
        match lane {
            Some(lane) => self.with_lane(lane),
            None => self,
        }
    }

    /// Arm bounded captures through this policy. `None` leaves the capture slot
    /// alone, which is what a product whose lane owns capture wants.
    pub fn with_capture(mut self, capture: Option<Capture>) -> Self {
        self.capture = capture;
        self
    }

    /// Register every named leaf that is not registered yet.
    fn register_leaves(&mut self, ctx: &mut Ctx<'_, P>) {
        self.keys.clear();
        self.product.viewport_keys(ctx, &mut self.keys);
        for index in 0..self.keys.len() {
            let key = self.keys[index];
            if ctx.producers.contains(key) {
                continue;
            }
            let Some(producer) = self.product.producer(ctx, key) else {
                continue;
            };
            let properties = self.product.producer_properties(key);
            if let Err(why) = ctx.producers.register(key, producer, properties) {
                eprintln!("[{}] producer {key:#x}: {why:?}", P::LOG_PREFIX);
            }
        }
    }

    /// This frame's error line: the product's own, else the first producer
    /// error among its leaves.
    fn error(&mut self, ctx: &mut Ctx<'_, P>) -> Option<String> {
        if let Some(error) = self.product.product_error(ctx) {
            return Some(error);
        }
        self.keys.clear();
        self.product.viewport_keys(ctx, &mut self.keys);
        for index in 0..self.keys.len() {
            let key = self.keys[index];
            let Some(why) = ctx.producers.error(key) else {
                continue;
            };
            return Some(
                self.product
                    .viewport_error(key, why)
                    .unwrap_or_else(|| viewport_error(why)),
            );
        }
        None
    }

    /// Publish the error line when it moved, and ask for the frame that shows
    /// it. An unchanged string is not written: writing it would rebuild the
    /// retained tree for nothing.
    fn publish(&mut self, ctx: &mut Ctx<'_, P>) {
        let error = self.error(ctx);
        if P::published_error(ctx.runner.state()) == error.as_deref() {
            return;
        }
        ctx.runner.update(|state| P::publish_error(state, error));
        if let Some(window) = ctx.window {
            window.request_redraw();
        }
    }

    /// The seven closures, over one shared assembly.
    pub fn hooks(self) -> HostHooks<P::State, P::Logic, P::View> {
        let this = Rc::new(RefCell::new(self));
        let (frame, dispatch, after, wake, close, key) = (
            this.clone(),
            this.clone(),
            this.clone(),
            this.clone(),
            this.clone(),
            this.clone(),
        );
        HostHooks {
            frame: Box::new(move |ctx| {
                let mut this = frame.borrow_mut();
                if P::closing(ctx.runner.state()) {
                    *ctx.close = true;
                }
                this.register_leaves(ctx);
                let busy = this.product.frame(ctx);
                if let Some(capture) = &this.capture {
                    let ready = this.product.capture_ready(ctx);
                    capture.arm::<P>(ctx, ready);
                }
                busy || this.lanes.iter().any(|lane| lane.wants_frames())
            }),
            after_dispatch: Box::new(move |ctx| {
                dispatch.borrow_mut().product.after_dispatch(ctx);
            }),
            after_frame: Box::new(move |ctx| {
                let mut this = after.borrow_mut();
                this.publish(ctx);
                this.product.after_frame(ctx);
                for index in 0..this.lanes.len() {
                    this.lanes[index].after_frame(ctx);
                }
                if this.product.profiling() {
                    if let Some(profile) = ctx.frame_profile {
                        eprintln!("[{}] frame {}", P::LOG_PREFIX, profile.summary());
                    }
                }
            }),
            after_wake: Box::new(move |ctx| {
                wake.borrow_mut().product.after_wake(ctx);
            }),
            close_request: Box::new(move |ctx, request| {
                let mut this = close.borrow_mut();
                for index in 0..this.lanes.len() {
                    this.lanes[index].request_close();
                }
                // The deferred close needs one more presented frame for the
                // last capture and the receipt. Both sources asked for it here.
                if let Some(window) = ctx.window {
                    window.request_redraw();
                }
                this.product.close_request(ctx, request)
            }),
            focused_text: Box::new(P::focused_text),
            key_intercept: Box::new(move |runner, press| {
                key.borrow_mut().product.key(runner, press)
            }),
        }
    }

    /// Open the window and run to completion.
    ///
    /// The error is a string rather than winit's: naming `EventLoopError` would
    /// put winit in this crate's public surface for a value every consumer
    /// immediately prints.
    pub fn run(
        self,
        options: HostOptions,
        init: impl FnOnce(&dyn HostWindow, &WindowCommands, &HostWake) -> Init<P::State, P::Logic>
        + 'static,
    ) -> Result<(), String> {
        cambium_genet_winit_host::run(options, init, self.hooks()).map_err(|why| why.to_string())
    }
}

/// The shared wording for a producer error.
pub fn viewport_error(why: ProducerError) -> String {
    format!("{VIEWPORT_ERROR}: {why:?}")
}
