//! Isometry's half of the shared host: `init`, the key intercept, and the
//! per-frame work the assembly cannot know about.
//!
//! The host calls `init` once, after the window exists and before the first
//! frame, and hands back a runner over what it returns. Nothing here knows
//! about winit, wgpu, netrender or a layout engine — the boundary the
//! migration bought.
//!
//! M4 of the isomere plan took the seven closures this module used to build:
//! they are `isomere::host::Assembly` over the [`Isometry`] product impl now,
//! and what is left here is what the assembly cannot fill. [`App::frame_tick`]
//! is the frame hook's isometry half, [`key_intercept`] the keyboard policy,
//! [`focused_text`] the text seam, and `init` the boot the host calls once.
//!
//! [`Isometry`]: crate::product::Isometry

use std::cell::RefCell;
use std::rc::Rc;

use cambium_genet_winit_host::{HostWake, HostWindow, Init, Key, KeyPress, NamedKey};

use isometry_views::{BoardKey, NamedPress, Press};

use super::*;

mod text;

pub(crate) use text::focused_text;
use text::{Lane, focused_lane};

/// Whether `press` is this character with no command chord held.
fn plain_char(press: &KeyPress, want: &str) -> bool {
    matches!(&press.key, Key::Character(c) if c == want) && !press.modifiers.ctrl
}

/// Whether `press` is this named key.
fn named(press: &KeyPress, want: NamedKey) -> bool {
    matches!(&press.key, Key::Named(k) if *k == want)
}

/// What a key press means before the tree sees it.
///
/// Everything that used to live in the desktop host's `key`, in the same order,
/// because the order *is* the policy: an open transient surface outranks the
/// text lanes, which outrank the single-letter verbs. Returning `true` consumes
/// the press.
///
/// What M3 changed is *how* a lane is recognised. There used to be three flag
/// branches here, each rebuilding a draft one character at a time. There is one
/// now, and it asks the caret: only one field can hold it, so the flags cannot
/// disagree about who is typing. Escape and Enter stay app commands and
/// everything else falls through, so the host routes it into
/// `caret_text_field` — which is where the text, the caret, the selection and
/// IME live now.
pub(crate) fn key_intercept(runner: &mut Runner, press: &KeyPress) -> bool {
    // Escape backs out of target-pick before anything else reads it, so an
    // armed attack is always cancellable without spending a turn.
    if runner.state().picking_target() && named(press, NamedKey::Escape) {
        runner.update(|ui| ui.cancel_action_pick());
        return true;
    }
    // An open token menu is the most recent thing on screen and the first
    // thing Escape takes back. A press elsewhere is the `overlay_surface`'s
    // dismissal layer; this is the keyboard half, which the surface's own
    // passive listener cannot serve because nothing inside the menu is focused.
    if runner.state().context_menu.is_some() && named(press, NamedKey::Escape) {
        runner.update(|ui| ui.close_context_menu());
        return true;
    }
    // A focused text lane owns every key but its own two.
    if let Some((_, lane)) = focused_lane(runner) {
        if named(press, NamedKey::Escape) {
            match lane {
                Lane::Command => runner.update(|ui| ui.command_cancel()),
                Lane::Whisper => runner.update(|ui| ui.compose_cancel()),
                Lane::Search => runner.update(|ui| ui.compendium_escape()),
                Lane::CharacterName | Lane::CharacterOwner => {
                    runner.update(|ui| ui.close_character())
                },
            }
            return true;
        }
        if named(press, NamedKey::Enter) {
            match lane {
                Lane::Command => runner.update(|ui| ui.command_submit()),
                Lane::Whisper => runner.update(|ui| ui.compose_send()),
                // The filter has nothing to submit, but Enter must not reach
                // `end_turn` behind an open compendium either.
                Lane::Search => {},
                Lane::CharacterName | Lane::CharacterOwner => {
                    runner.update(|ui| ui.create_character())
                },
            }
            return true;
        }
        return false;
    }
    // The compendium with no field focused: an entry page is open, so there is
    // no filter to type into. It still swallows the press — a single-letter
    // verb firing on the board behind an open page was never the policy —
    // and Escape still steps back to the index.
    if runner.state().compendium_open {
        if named(press, NamedKey::Escape) {
            runner.update(|ui| ui.compendium_escape());
        }
        return true;
    }
    // The creation panel has no board verb behind it. Its name field normally
    // owns focus; this also covers a focus loss before Escape arrives.
    if runner.state().character_open {
        if named(press, NamedKey::Escape) {
            runner.update(|ui| ui.close_character());
        }
        return true;
    }
    // Everything below is a plain board verb, and every one of them is a row
    // in `isometry_views::KEYMAP` — the one declaration the side panel's key
    // crib is also read off (M3 of the isomere plan). A key that is not in it
    // falls through to the tree untouched, and a row that grew no arm here is
    // a compile error rather than a dead line in the crib.
    let Some(claimed) = board_press(press) else {
        return false;
    };
    let Some(command) = isometry_views::KEYMAP.command(&claimed).copied() else {
        return false;
    };
    match command {
        // The command sigil opens the > line, the way `w` opens a whisper. The
        // draft starts empty; the ">" is the prompt.
        BoardKey::Command => runner.update(|ui| ui.start_command()),
        BoardKey::Whisper => runner.update(|ui| ui.start_compose()),
        BoardKey::Face => runner.update(|ui| ui.rotate_selected()),
        // Cycle the fog viewer: omniscient, then each side. Lets the DM preview
        // a player's view (and drives single-window fog verification without a
        // session).
        BoardKey::FogView => runner.update(|ui| ui.cycle_viewer()),
        BoardKey::EndTurn => runner.update(|ui| ui.end_turn()),
        BoardKey::Undo => runner.update(|ui| ui.undo()),
        BoardKey::Redo => runner.update(|ui| ui.redo()),
        BoardKey::Pan(dc, dr) => runner.update(|ui| ui.pan_tiles(f32::from(dc), f32::from(dr))),
    }
    true
}

/// One press as the view layer's declaration compares it.
///
/// The only place this host lowers a platform key into the keymap's vocabulary.
/// Named keys drop the chord because the chain this replaced never read one for
/// them: a Ctrl-held arrow has always panned, and Ctrl+Enter has always ended
/// the turn.
fn board_press(press: &KeyPress) -> Option<Press> {
    match &press.key {
        Key::Character(character) if press.modifiers.ctrl => Some(Press::ctrl(character)),
        Key::Character(character) => Some(Press::plain(character)),
        Key::Named(NamedKey::Enter) => Some(Press::Named(NamedPress::Enter)),
        Key::Named(NamedKey::ArrowLeft) => Some(Press::Named(NamedPress::ArrowLeft)),
        Key::Named(NamedKey::ArrowRight) => Some(Press::Named(NamedPress::ArrowRight)),
        Key::Named(NamedKey::ArrowUp) => Some(Press::Named(NamedPress::ArrowUp)),
        Key::Named(NamedKey::ArrowDown) => Some(Press::Named(NamedPress::ArrowDown)),
        _ => None,
    }
}

/// Build the starting state: the map, the campaign checkpoint, the game system,
/// the generator catalog, and the session bridge. Runs once, inside the host,
/// after the window exists but before the first frame.
pub(crate) fn init(
    app: &Rc<RefCell<App>>,
    window: &dyn HostWindow,
    wake: &HostWake,
) -> Init<UiState, Logic> {
    let mut app = app.borrow_mut();
    // `ISOMETRY_SYNTH=<n>` loads an n x n synthetic stress board (n>1,
    // default 30 = the probe P2 board) instead of the demo skirmish;
    // large n exercises viewport windowing.
    let map = match std::env::var("ISOMETRY_SYNTH") {
        Ok(v) => {
            let n = v
                .trim()
                .parse::<u32>()
                .ok()
                .filter(|&n| n > 1)
                .unwrap_or(30);
            synth_map(n, n)
        },
        Err(_) => demo_map(),
    };
    let can_restore = !matches!(app.net_intent.as_ref(), Some(NetIntent::Join(_)));
    let mut restore_status = None;
    let mut restored_public = None;
    if can_restore {
        if let Some(name) = app.campaign_arg.take() {
            match CampaignRepository::open(campaign_path(&name))
                .and_then(|repository| repository.load_checkpoint())
            {
                Ok(Some(checkpoint)) => {
                    app.campaign = checkpoint.private;
                    app.journal = checkpoint.public.journal.clone();
                    app.history = checkpoint.history;
                    app.history_origin = checkpoint.history_origin;
                    app.source_history_len = None;
                    app.source_history_attached = false;
                    restored_public = Some(checkpoint.public);
                    restore_status = Some(format!("restored campaign {name}"));
                },
                Ok(None) => restore_status = Some(format!("campaign {name} has no checkpoint")),
                Err(error) => restore_status = Some(format!("campaign restore failed: {error}")),
            }
        }
    }
    let mut ui = UiState::new(map);
    if let Some(snapshot) = restored_public {
        ui.apply_snapshot(snapshot);
    }
    ui.generator_choices = app.generator_catalog.choices();
    for diagnostic in app.generator_catalog.diagnostics() {
        eprintln!("[isometry] content pack: {diagnostic}");
    }
    if let Some(status) = restore_status {
        ui.status = status;
    }
    // Start with the board roughly centered in the pane, and every
    // token in the turn order (a skirmish ready to play; drop
    // tokens out via the panel for free movement).
    ui.camera = (420.0, 140.0);
    // Seed the pane size so the view can window tile emission to the
    // viewport (the frame hook keeps it current on resize).
    //
    // The window's scale factor is the *device* scale and knows nothing about
    // the design fit, so dividing by it alone would hand the view a viewport a
    // whole zoom factor wrong for one frame. The host measures `fit_design`
    // against the pre-zoom surface and lays out at `surface / zoom`; `fit_zoom`
    // is public precisely so a consumer can compute the same number rather than
    // a near miss, and `DESIGN_SIZE` is the one `HostOptions` was handed.
    let (width, height) = window.inner_size();
    let device = window.scale_factor() as f32;
    let available = (width as f32 / device, height as f32 / device);
    let zoom = cambium_genet_winit_host::fit_zoom(DESIGN_SIZE, available);
    let (logical_w, logical_h) = (available.0 / zoom, available.1 / zoom);
    ui.viewport = ((logical_w - PANEL_W).max(0.0), logical_h);
    app.last_viewport = ui.viewport;
    // The board's pixel grid, before the first frame rather than after it: a
    // board laid out at the raw fractional zoom for one frame and re-laid out
    // on the next is a visible jump on a slow boot.
    ui.set_pixel_grid((device, zoom));
    let ids: Vec<_> = ui.map.tokens.iter().map(|t| t.id).collect();
    for id in ids {
        ui.turns.add(id);
    }
    let initial_snapshot = app.snapshot_of(&ui);
    if !matches!(app.net_intent.as_ref(), Some(NetIntent::Join(_))) {
        app.ensure_history_origin(&initial_snapshot);
    }
    if let Some(origin) = app.history_origin.clone() {
        ui.set_overmap_source_history(Some(isonetry::GameSourceHistory::new(
            origin,
            app.history.clone(),
        )));
        app.source_history_len = Some(app.history.len());
        app.source_history_attached = true;
    }

    // Session setup: host publishes this board; a client starts from
    // an empty view and fills in on the first snapshot. Either way the
    // view is Remote, so play routes through the session.
    //
    // The bridge's actor wakes this application through the host's own wake
    // handle, so a snapshot arriving on the network thread schedules one drain
    // turn (`after_wake`) and one redraw rather than waiting on a poll.
    match app.net_intent.take() {
        Some(NetIntent::Host) => {
            app.net_is_host = true;
            ui.net_mode = NetMode::Remote;
            let snapshot = GameSnapshot {
                map: ui.map.clone(),
                turns: ui.turns.clone(),
                roll_log: Vec::new(),
                journal: app.journal.clone(),
                inventories: ui.inventories.clone(),
                generations: ui.generations.clone(),
                maps: ui.campaign_maps.clone(),
                active_map: ui.active_map.clone(),
                world: ui.world.clone(),
                clocks: ui.clocks.clone(),

                party_cap: ui.party_cap,
                last_beats: Vec::new(),
                beat_seq: 0,
                applied_actions: Default::default(),
            };
            let campaign = app.campaign.clone();
            let history = app.history.clone();
            app.net = Some(NetBridge::spawn(
                Role::Host {
                    state: snapshot,
                    campaign,
                    history,
                },
                wake.callback(),
            ));
        },
        Some(NetIntent::Join(ticket)) => {
            ui.net_mode = NetMode::Remote;
            ui.can_edit_inventory = false;
            ui.status = "connecting...".to_owned();
            let name = app
                .viewer_arg
                .clone()
                .unwrap_or_else(|| "player".to_owned());
            app.net = Some(NetBridge::spawn(
                Role::Client { ticket, name },
                wake.callback(),
            ));
        },
        None => {},
    }
    // Boot clock. The net selftest waits on it, and so does the combat
    // selftest, which runs solo (there is no session to wait for).
    app.started = Some(Instant::now());
    // Fog viewer from `--as`. Applies in any mode: a client sees
    // through its player's tokens, and a solo run can preview a side.
    if let Some(v) = app.viewer_arg.take() {
        ui.viewer = Some(v);
        ui.recompute_fog();
    }
    // Seed the dice generator with real entropy so rolls differ per
    // launch (the clock is plenty for a friendly table).
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    ui.reseed(seed);

    // Load the game system (5e SRD) and hand the view its schema so it
    // can render sheets without knowing any rules.
    let system = srd_5e();
    ui.sheet_schema = schema_of(&system);
    ui.bestiary = bestiary_of();
    ui.emotes = app.pack_emotes.clone();
    ui.spells = spells_of();
    ui.items = items_of();
    app.system = Some(system);
    // B2's flag, read once here for the same reason every other `ISOMETRY_*`
    // hook is: the environment is the host's to read, and the view is handed a
    // field. With it unset nothing below is built and the DOM board stands.
    if scene_board::enabled() {
        ui.scene_board = true;
        app.scene_board = Some(scene_board::SceneBoard::new(&ui));
    }

    Init {
        state: ui,
        logic: board_root as Logic,
        sheet: std::mem::take(&mut app.sheet),
        fonts: Vec::new(),
        images: Vec::new(),
    }
}

impl App {
    /// Keep the view's pane size current so windowing culls to the actual
    /// viewport. Cheap-checked: writing the same two floats every frame would
    /// rebuild the retained tree for nothing.
    fn sync_viewport(&mut self, ctx: &mut Ctx<'_>) {
        let (width, height) = ctx.logical_size;
        let viewport = ((width - PANEL_W).max(0.0), height);
        if self.last_viewport == viewport {
            return;
        }
        self.last_viewport = viewport;
        ctx.runner.update(|ui| ui.viewport = viewport);
    }

    /// Keep the board's pixel grid current: the device scale the window is on
    /// and the zoom the host is laying out at.
    ///
    /// It belongs in a hook rather than in the view because only the host knows
    /// either number — `AppCtx::ui_zoom` is the effective zoom, fit times user
    /// offset, and the scale factor is the window's. `zoom_changed` is the edge
    /// the host raises, but a window dragged to a monitor of another density
    /// moves the device scale as well, so the compare is on the pair: two float
    /// tests per frame, and an update only when the board really must move.
    fn sync_board_scale(&mut self, ctx: &mut Ctx<'_>) {
        let device = ctx
            .window
            .map_or(1.0, |window| window.scale_factor() as f32);
        let grid = (device, ctx.ui_zoom);
        if ctx.runner.state().pixel_grid == grid {
            return;
        }
        ctx.runner.update(|ui| ui.set_pixel_grid(grid));
    }

    /// Register (or clear) the overmap's painted graph leaf so the view's
    /// `<custom-leaf>` gets nodes + edges.
    ///
    /// The swatch is only *built* while the surface is open (building it
    /// projects the world and runs the force layout — never pay that on an
    /// ordinary board frame), and the leaf is only *re-registered* when the
    /// swatch model changed: a fresh `GraphCanvas` is born dirty, so an
    /// unconditional insert would defeat the leaf-tier retention gate and
    /// repaint every frame. Both gates are load-bearing; read the 2026-07-20
    /// perf plan before loosening either.
    fn sync_overmap_leaf(&mut self, ctx: &mut Ctx<'_>) {
        if ctx.runner.state().overmap_open {
            match isometry_views::overmap_swatch(ctx.runner.state()) {
                Some(mut swatch) => {
                    self.sync_atlas_labels(&mut swatch);
                    if self.last_overmap_swatch.as_ref() != Some(&swatch) {
                        ctx.leaves.insert(
                            isometry_views::OVERMAP_LEAF_KEY,
                            Box::new(swatch.paint_leaf(overmap_node_color)),
                        );
                        self.last_overmap_swatch = Some(swatch);
                    }
                },
                None => {
                    ctx.leaves.remove(&isometry_views::OVERMAP_LEAF_KEY);
                    self.last_overmap_swatch = None;
                },
            }
        } else if self.last_overmap_swatch.is_some() {
            ctx.leaves.remove(&isometry_views::OVERMAP_LEAF_KEY);
            self.last_overmap_swatch = None;
        }
    }

    fn sync_atlas_labels(
        &mut self,
        swatch: &mut cambium::GraphCanvasSwatch<String, isometry_views::OvermapNodeKind>,
    ) {
        let Some(atlas) = swatch.atlas.as_mut() else {
            return;
        };
        let color = sprigging::ColorF::new(0.92, 0.94, 0.98, 1.0);
        self.atlas_label_cache.retain(|key, _| {
            swatch
                .graph
                .nodes
                .iter()
                .any(|node| key == &format!("{}\u{1f}{}", node.id, node.label))
        });
        let mut callouts = Vec::new();
        for node in &swatch.graph.nodes {
            let cache_key = format!("{}\u{1f}{}", node.id, node.label);
            let callout = self.atlas_label_cache.entry(cache_key).or_insert_with(|| {
                atlas_labels::atlas_label_from_default_host_font(
                    node.id.clone(),
                    &node.label,
                    11.0,
                    sceno::Vec2::new(7.0, -6.0),
                    color,
                )
            });
            callouts.push(callout.clone());
        }
        atlas.callouts = callouts;
    }

    /// Hold a staged beat for [`BEAT_HOLD`], then drop its classes. Returns
    /// whether frames should keep coming.
    ///
    /// The drop is what makes the *next* strike a genuine attribute change: an
    /// unchanged class restyles nothing, so without it the second swing would
    /// stand still.
    fn drive_beats(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.runner.state().beats.is_empty() {
            self.beat_until = None;
            return false;
        }
        match self.beat_until {
            Some(until) if Instant::now() >= until => {
                self.beat_until = None;
                ctx.runner.update(|ui| ui.clear_beats());
                false
            },
            Some(_) => true,
            None => {
                self.beat_until = Some(Instant::now() + BEAT_HOLD);
                true
            },
        }
    }

    /// Whether an armed self-test still needs frames to reach its deadline.
    ///
    /// A still board parks on `Wait`, which blocks until input arrives, so an
    /// armed selftest would never reach its own deadline. Asking for frames is
    /// how the shared host's hook says the same thing the old `WaitUntil` did.
    pub(crate) fn selftests_pending(&self) -> bool {
        (self.travel_selftest && !self.travel_fired)
            || (self.cmd_selftest && !self.cmd_fired)
            || (self.watchtower_selftest && !self.watchtower_fired)
            || (self.convince_selftest && !self.convince_fired)
            || (self.storylet_selftest && !self.storylet_fired)
            || (self.overmap_selftest && !self.overmap_fired)
            || (self.compendium_selftest && !self.compendium_fired)
            || (self.whisper_selftest && !self.whisper_fired)
            || (self.turns_selftest && !self.turns_fired)
            || (self.net.is_some() && self.net_selftest && !self.selftest_fired)
            || (self.combat_selftest && !(self.combat_swings == 0 && self.combat_emoted))
    }

    /// Run every armed self-test driver. Each one waits out its own warm-up.
    pub(crate) fn drive_selftests(&mut self, ctx: &mut Ctx<'_>) {
        self.maybe_combat_selftest(ctx);
        self.maybe_travel_selftest(ctx);
        self.maybe_cmd_selftest(ctx);
        self.maybe_watchtower_selftest(ctx);
        self.maybe_convince_selftest(ctx);
        self.maybe_storylet_selftest(ctx);
        self.maybe_overmap_selftest(ctx);
        self.maybe_compendium_selftest(ctx);
        self.maybe_whisper_selftest(ctx);
        self.maybe_turns_selftest(ctx);
        if self.net.is_some() {
            self.maybe_selftest(ctx);
        }
    }
}

/// The frame hook's isometry half, in the order the old closure ran it.
///
/// M4 moved the seven closures into `isomere::host::Assembly`; what the
/// assembly cannot know is this — the pane, the board's pixel grid, the atlas
/// motion, the overmap leaf, the scene board's snapshot and the beat hold.
/// Producer registration and capture arming used to sit in the middle of it
/// and are the assembly's now, on either side of this call.
impl App {
    pub(crate) fn frame_tick(&mut self, ctx: &mut Ctx<'_>) -> bool {
        // Before the viewport: culling measures the pane against the board
        // geometry, and the two must agree on the same frame.
        self.sync_board_scale(ctx);
        self.sync_viewport(ctx);
        let atlas_moving = self.drive_atlas_motion(ctx);
        self.sync_overmap_leaf(ctx);
        // After the viewport: the scene's camera is framed by the pane the
        // two calls above just settled.
        if let Some(mut board) = self.scene_board.take() {
            board.sync(ctx);
            self.scene_board = Some(board);
        }
        let selftests_pending = self.selftests_pending();
        let beating = self.drive_beats(ctx);
        beating || selftests_pending || atlas_moving
    }
}
