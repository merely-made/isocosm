//! B4's headed receipt: put every overlay on the board and leave it standing.
//!
//! `ISOMETRY_OVERLAY_SELFTEST` arms it, and its *value* chooses what the
//! capture shows, because the three groups hide each other: fog would swallow
//! the tints it is meant to be seen beside, and a focus elevation cuts the
//! hill they stand on.
//!
//! | value | what stands |
//! |-------|-------------|
//! | `1` (or anything else) | selection, reach, path, a door and an encounter site |
//! | `fog` | the same board through a player's eyes: the shroud and the dark |
//! | `focus` | the same board cut to one elevation |
//!
//! Everything is driven through the state's own verbs — `select_token`,
//! `hover_tile_enter`, `cycle_viewer`, `cycle_focus_elevation` — so the
//! capture proves the shipping path rather than a second one. With the scene
//! flag off it drives the identical board through the DOM, which is the
//! positive control: the DOM arm shows what the scene arm is being measured
//! against.

use isometry_campaign::{CampaignMap, EncounterAnchor, MapPoint, MapScale, MapTransition};

use super::*;

/// What the armed capture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlayArm {
    Tints,
    Fog,
    Focus,
}

impl OverlayArm {
    /// Reads the arm off the flag's value. Anything unrecognised is the tints,
    /// so `=1` means what it always means.
    pub(crate) fn of(value: &str) -> Self {
        match value {
            "fog" => Self::Fog,
            "focus" => Self::Focus,
            _ => Self::Tints,
        }
    }
}

impl App {
    /// `ISOMETRY_OVERLAY_SELFTEST`: lay the board's overlays on and hold them.
    pub(crate) fn maybe_overlay_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(arm) = self.overlay_selftest else {
            return;
        };
        if self.overlay_fired {
            return;
        }
        // Later than the panel lanes, like the select receipt: the scene has
        // to have drawn a frame before a pick can read one.
        if !self
            .started
            .is_some_and(|started| started.elapsed() > Duration::from_secs(3))
        {
            return;
        }
        self.overlay_fired = true;
        ctx.runner.update(|ui| {
            seed_sites(ui);
            // A selected tile, chosen away from the party so its tint is its
            // own rather than a marker's.
            ui.mode = EditMode::Select;
            ui.click_tile((8, 16));
            // Play mode with a piece up: reach tints its whole field, and the
            // hover paints the path through it.
            ui.mode = EditMode::Play;
            ui.select_token(TokenId(1));
            let step = ui
                .reach
                .keys()
                .copied()
                .max_by_key(|at| at.0 + at.1)
                .unwrap_or((12, 12));
            ui.hover_tile_enter(Some(step));
            eprintln!(
                "[isometry] overlay selftest ({arm:?}): selected {:?}, reach {} tiles, path \
                 to {step:?}, door at {DOOR:?} among {:?}, encounter site at {ENCOUNTER:?}",
                ui.selected,
                ui.reach.len(),
                ui.door_tiles(),
            );
            match arm {
                OverlayArm::Tints => {},
                // The DM looks through the party's eyes: everything out of
                // sight is remembered or dark.
                // The DM looks through the party's eyes, and then the light
                // shortens: what was seen a moment ago is remembered rather
                // than lit, which is the only way to get all three fog levels
                // on one board — explored memory is folded in the instant a
                // tile is seen, so a first look has no shroud in it at all.
                OverlayArm::Fog => {
                    ui.cycle_viewer();
                    ui.sight_radius = 3;
                    ui.recompute_fog();
                    eprintln!(
                        "[isometry] overlay selftest: viewer {:?} at sight {}, {} visible, \
                         {} explored, {} shrouded",
                        ui.viewer,
                        ui.sight_radius,
                        ui.visible.len(),
                        ui.explored.len(),
                        ui.explored.difference(&ui.visible).count(),
                    );
                },
                OverlayArm::Focus => {
                    ui.cycle_focus_elevation();
                    eprintln!(
                        "[isometry] overlay selftest: focus {:?} — {}",
                        ui.focus_elevation, ui.status
                    );
                },
            }
        });
    }
}

/// The tile the seeded encounter site stands on.
const ENCOUNTER: (i32, i32) = (15, 10);
/// The tile the seeded door stands on.
const DOOR: (i32, i32) = (9, 9);

/// Register the live board as a campaign map carrying one door and one
/// encounter site, which is the only way those two tints exist at all: both
/// are authored campaign facts rather than map layers.
fn seed_sites(ui: &mut UiState) {
    let field = CampaignMap {
        id: "field".to_owned(),
        scale: MapScale::Local,
        document: ui.map.clone(),
        spawn_zones: Vec::new(),
        transitions: vec![MapTransition {
            id: "field-gate".to_owned(),
            at: MapPoint {
                col: DOOR.0 as u32,
                row: DOOR.1 as u32,
            },
            target_map: "hut".to_owned(),
            target_entry: None,
        }],
        encounter_anchors: vec![EncounterAnchor {
            id: "wolf-den".to_owned(),
            at: MapPoint {
                col: ENCOUNTER.0 as u32,
                row: ENCOUNTER.1 as u32,
            },
            tags: vec!["beast".to_owned()],
        }],
    };
    ui.campaign_maps.insert("field".to_owned(), field);
    ui.active_map = Some("field".to_owned());
}
