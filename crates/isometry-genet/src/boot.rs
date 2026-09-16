// SPDX-License-Identifier: MIT OR Apache-2.0

//! Everything the command line and the environment decided, before a window
//! exists.
//!
//! Split out of `main.rs` at the 600-line ceiling when M4 of the isomere plan
//! added the product impl. The seam is the one the file already had: these are
//! the functions that run once, from `App::boot`, with no window, no runner
//! and no host in sight — the document-slug paths a dispatch resolves later,
//! the pack roots, the three command-line parsers, and the constructor that
//! reads the environment. Nothing per-frame is here; `hooks.rs` next door is
//! what that is for.

use super::*;

/// Search the bundled example, a project-local pack root, and user-selected
/// roots. Entries may be pack directories or directories containing packs.
///
/// (This doc had drifted two functions away from what it documents; the M4
/// split put it back.)
pub(crate) fn generator_pack_roots() -> Vec<std::path::PathBuf> {
    // The `core` pack ships the default beat vocabulary (strike, recoil, fall,
    // cheer...). It is a pack like any other, so a campaign overrides a beat
    // simply by declaring the same name: the app owns no choreography.
    let mut roots = vec![
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../isometry-system/examples/packs/core"),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../isometry-system/examples/packs/demo"),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../isometry-system/examples/packs/watchtower"),
    ];
    let local = std::path::PathBuf::from("packs");
    if local.is_dir() {
        roots.push(local);
    }
    if let Some(paths) = std::env::var_os("ISOMETRY_PACK_DIRS") {
        roots.extend(std::env::split_paths(&paths));
    }
    roots
}

/// Parse `--host` or `--join <ticket>` from the command line.
pub(crate) fn parse_net_intent() -> Option<NetIntent> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--host") {
        Some(NetIntent::Host)
    } else if let Some(i) = args.iter().position(|a| a == "--join") {
        args.get(i + 1).map(|t| NetIntent::Join(t.clone()))
    } else {
        None
    }
}

/// Parse `--as <player>` from the command line.
pub(crate) fn parse_viewer() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--as")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Parse `--campaign <name>` for checkpoint restore. The name shares the map
/// slug convention, so `--campaign "Demo Skirmish"` resolves its paired store.
pub(crate) fn parse_campaign() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--campaign")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

pub(crate) fn document_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// The public, reviewable map document.
pub(crate) fn map_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("maps").join(format!("{}.json", document_slug(name)))
}

/// The private GM store paired with a map. Muniment's redb backend makes the
/// slot durable; it is intentionally outside the map's shareable JSON.
pub(crate) fn campaign_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("campaigns").join(format!("{}.redb", document_slug(name)))
}

impl App {
    /// Everything the command line and the environment decided, before a window
    /// exists. The content packs are read here because the stylesheet the host
    /// is handed in `init` is the app sheet plus whatever choreography they
    /// declared.
    pub(crate) fn boot() -> Self {
        let generator_catalog = GeneratorCatalog::discover(generator_pack_roots());
        // Choreography is pack data: the stylesheet the packs supply is appended
        // to the app's, and the emote menu is built from whichever beats they
        // marked emotable. A table with no packs still plays a correct game; it
        // just plays it without flourishes, which is safe precisely because no
        // rule may read a beat.
        let (pack_beats, beat_diagnostics) = generator_catalog.choreography();
        for diagnostic in &beat_diagnostics {
            eprintln!("[isometry] choreography: {diagnostic}");
        }
        let mut sheet = board_css();
        for beat in &pack_beats {
            sheet.push('\n');
            sheet.push_str(&beat.css);
        }
        let pack_emotes: Vec<(String, String)> = pack_beats
            .iter()
            .filter_map(|b| b.emote.clone().map(|label| (b.name.clone(), label)))
            .collect();
        Self {
            campaign: CampaignStore::new(),
            journal: Vec::new(),
            history: Journal::new(),
            history_origin: None,
            source_history_len: None,
            source_history_attached: false,
            last_overmap_swatch: None,
            atlas_label_cache: HashMap::new(),
            overmap_frame_at: None,
            last_storylet_inputs: None,
            // A fixed seed keeps a solo session reproducible and makes the headed
            // verification deterministic. A real table seeds this per session.
            action_rng: Rng::new(0x15D_0BE),
            own_requests: 0,
            beat_until: None,
            sheet,
            last_viewport: (0.0, 0.0),
            scene_board: None,
            profile: std::env::var_os("ISOMETRY_PROFILE").is_some(),
            net_intent: parse_net_intent(),
            net_is_host: false,
            viewer_arg: parse_viewer(),
            campaign_arg: parse_campaign(),
            net: None,
            last_net_version: 0,
            net_selftest: std::env::var_os("ISOMETRY_NET_SELFTEST").is_some(),
            travel_selftest: std::env::var_os("ISOMETRY_TRAVEL_SELFTEST").is_some(),
            travel_fired: false,
            cmd_selftest: std::env::var_os("ISOMETRY_CMD_SELFTEST").is_some(),
            cmd_fired: false,
            watchtower_selftest: std::env::var_os("ISOMETRY_WATCHTOWER_SELFTEST").is_some(),
            watchtower_fired: false,
            convince_selftest: std::env::var_os("ISOMETRY_CONVINCE_SELFTEST").is_some(),
            convince_fired: false,
            storylet_selftest: std::env::var_os("ISOMETRY_STORYLET_SELFTEST").is_some(),
            storylet_fired: false,
            overmap_selftest: std::env::var_os("ISOMETRY_OVERMAP_SELFTEST").is_some(),
            overmap_fired: false,
            overmap_source_time_selftest: std::env::var_os("ISOMETRY_OVERMAP_SOURCE_TIME_SELFTEST")
                .is_some(),
            compendium_selftest: std::env::var_os("ISOMETRY_COMPENDIUM_SELFTEST").is_some(),
            compendium_fired: false,
            whisper_selftest: std::env::var_os("ISOMETRY_WHISPER_SELFTEST").is_some(),
            whisper_fired: false,
            turns_selftest: std::env::var_os("ISOMETRY_TURNS_SELFTEST").is_some(),
            select_selftest: std::env::var_os("ISOMETRY_SELECT_SELFTEST").is_some(),
            select_fired: false,
            overlay_selftest: std::env::var("ISOMETRY_OVERLAY_SELFTEST")
                .ok()
                .map(|value| crate::selftest::OverlayArm::of(&value)),
            overlay_fired: false,
            turns_fired: false,
            combat_selftest: std::env::var_os("ISOMETRY_COMBAT_SELFTEST").is_some(),
            combat_swings: 4,
            last_swing: None,
            combat_emoted: false,
            travel_emitted: Vec::new(),
            started: None,
            selftest_fired: false,
            system: None,
            last_sheet_open: None,
            // `ISOMETRY_GEN_SEED` fixes the generator tape so `>gen` previews and
            // rerolls are reproducible (headed verification, and a table that wants
            // a deterministic session); otherwise the wall clock seeds it as before.
            generation_tape: EntropyTape::from_seed(
                std::env::var("ISOMETRY_GEN_SEED")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|duration| duration.as_nanos() as u64)
                            .unwrap_or(1)
                    }),
            ),
            generation_ordinal: 0,
            generator_catalog,
            last_generator_selection: None,
            faction_turn_batch: Vec::new(),
            pack_emotes,
        }
    }
}
