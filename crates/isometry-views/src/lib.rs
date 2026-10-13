//! Isometry's Cambium view layer.
//!
//! View functions project [`isometry_core`] state into DOM-shaped views:
//! every visible tile, prop, and token is an element positioned by the
//! iso math, appearance bound through CSS class vocabulary so tilesets
//! arrive as stylesheets. Host-agnostic: the desktop host and the later
//! web host both drive [`board_root`].

mod board;
mod character;
mod command;
mod compendium;
mod demo;
mod downtime;
mod generator;
mod governance;
mod keymap;
mod overmap;
mod native_party;
pub use native_party::{PartyUi, PartyRequest, PartyView};
mod native_journey;
pub use native_journey::{JourneyUi,JourneyRequest,JourneyQuote};
mod native_pack;
pub use native_pack::{PackUi,PackRequest,PackReading};
mod native_overmap;
pub use native_overmap::{native_overmap_atlas,native_overmap_swatch,native_overmap_cell_id};
mod native_hooks;
mod native_events;
pub use native_events::{NativeEventStream,NativeEventUi};
mod native_items;
mod native_composed_storylet;
pub use native_composed_storylet::{StoryletPhysicalFileAction, StoryletPhysicalRequest, StoryletPhysicalOutcome};
mod native_institutions;
mod native_scene;
pub use native_scene::{NativeSceneInput,NativeSceneUi,SceneFileAction};
pub use native_institutions::{InstitutionUi,InstitutionFileAction,InstitutionRequest,InstitutionOutcome};
pub use native_hooks::{native_goal_lines,native_polity_lines};
mod panel;
mod projection;
mod scene;
mod sheet;
mod state;
mod storylet;
mod theme;
mod widgets;

pub use board::{UiChild, board_root};
pub use demo::{SYNTH_PARTY, demo_map, synth_map, synth_world};
pub use keymap::{BoardKey, KEYMAP, NamedPress, Press, key_hint};
pub use overmap::{
    AtlasBounds, AtlasProjection, AtlasRoute, AtlasSite, AtlasTerrainCell,
    ISOMETRY_OVERMAP_ADAPTER, OVERMAP_CANVAS, OVERMAP_LEAF_KEY, OvermapNodeKind, atlas_projection,
    overmap_atlas, overmap_positions, overmap_score, overmap_swatch,
};
pub use projection::{
    ISOMETRY_TILE_BOARD_ADAPTER, ISOMETRY_TILE_BOARD_BACKDROP, tile_board_cells, tile_board_scene,
    tile_board_score,
};
pub use scene::{
    BOARD_SCENE_LEAF_KEY, BoardBricks, BoardGround, BoardHandle, BoardPalette, BoardPick,
    BoardProducer, BoardSource, BoardView, BoardWorld, ColumnChange, ELEVATION_PX, GroundCost,
    MapTerrain, Overlays, RESIDENCY_MARGIN, ResidencySettings, SEA_LEVEL, ScenePick, TileColumns,
    Tint, TokenBodies, VOID_SURFACE, VOXELS_PER_STEP, VOXELS_PER_TILE, WORLD_PX, elevation_px,
    focus_top, material_of, owner_tint, surface_of, terrain_palette, world_px, yaw_of,
};
pub use state::{
    ActionRow, BOARD_UNIT, CharacterCreateRequest, CompendiumTab, EditMode, FactionMoveRow,
    FogLevel, GenerationRequest, GeneratorSelectionRequest, GovernanceBindingRow,
    GovernanceConflict, GovernanceResolutionRequest, InitiativeMode, InventoryRequest, ItemRow,
    MESSAGES_CAP, MonsterRow, NetMode, PANEL_W, SheetSchema, SpellRow, StoryletRow, UiState,
    WHEEL_BOARD_TILES, WHEEL_NOTCH_PX,
};
pub use state::{OvermapMotionState, OvermapMotionTick};
pub use state::native::{Acceptance as NativeAcceptance, NativeUi, TableFileAction, PhysicalRequest, PhysicalOutcome,
    ItemsRequest, ItemsOutcome, ItemsFileAction};
pub use state::{PACE_PCTS, STANCE_KEYS, mode_items, pace_items, stance_items};
pub use theme::{board_css, tile_kind_colour};
