// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! # isomere — the wing's GUI layer
//!
//! **The ruling (Mark, 2026-09-14).** `isometer` is the game-world scene
//! family; **isomere** is the wing-unique GUI layer — scene-plus-host chrome,
//! overlays, graphs, and the applications of Cambium that belong to the wing
//! rather than to one product. *isometer never names a Cambium widget;
//! **isomere never names a voxel.*** A viewport card in this crate holds a
//! leaf key and a size, never a scene and never a body, and this crate
//! depends on `shared/isometer` for nothing at all. That is the boundary, and
//! the 2026-09-15 inventory confirms it holds.
//!
//! **What lives here.** What the three products each built over Cambium and
//! netrender that is *not* product rules: the wing's shared stylesheet and
//! its tokens, the panels the three GUIs write three times under three names,
//! and the host assembly they share. Promoted once from an existing product
//! implementation — never written fresh — and consumed back by the product it
//! came from.
//!
//! **What does not live here.** The scene (isometer), the scenario lane
//! (mesquite), any product's rules or vocabulary, and anything a second
//! non-game consumer could want: that is Cambium's, and goes upstream to
//! mere.
//!
//! ## M0: the sheet
//!
//! The bench sheet and the session sheet were the same sheet in two palettes,
//! rule for rule and in the same order; only colours, four font sizes and
//! three box numbers differed. [`sheet`] is that sheet once, with every
//! difference lifted into an `--isomere-*` custom property, and a product
//! supplies [`Seeds`] plus its own remaining rules.
//!
//! Mark's ruling of 2026-09-15 on derivation: **seed from today's exact hex**
//! so the capture sets hold, and let `tinct` fill only the states nobody hand
//! picked. [`Picked`] is where a hand-picked state goes; everything left
//! `None` derives.
//!
//! ```
//! use isomere::{Seeds, Srgb, sheet};
//! let seeds = Seeds::anchors(
//!     Srgb::rgb(0xEE, 0xEA, 0xE1),
//!     Srgb::rgb(0x27, 0x33, 0x2E),
//!     Srgb::rgb(0x65, 0x73, 0x6A),
//!     Srgb::rgb(0x31, 0x5C, 0x3E),
//! );
//! let css = sheet(&seeds, ".bench { padding:24px; }");
//! assert!(css.contains("--isomere-accent: #315C3E;"));
//! assert!(css.contains(".part.selected"));
//! ```

//! ## M1: the viewport card and the error line
//!
//! [`viewport`] is the card the bench and the session each wrote by hand: a
//! `custom_leaf` with `role="img"` and a label, inside a `.scene-card`, with an
//! optional overlay over it, and the error line under the column. What a
//! product hands over is a leaf key and a box — never a scene — plus the four
//! things the two genuinely differed on (leaf class, id, description, and the
//! bench's transform style). The Isometry overmap is *not* a consumer: its leaf
//! is Cambium's own `GraphCanvasSwatch`, placed in a panel rather than a card.
//!
//! ## M2: the examiner
//!
//! [`examiner`] is the parts palette and the reading column the session's
//! subject sheet and the bench's parts examiner each wrote by hand:
//! `button.part` chips with a label, a selected state and a condition class,
//! over `.field` name-value rows. A product hands over an
//! [`ExaminerModel`] built from its own projection — a row is a `u64` and a
//! label, never a part — and a selection handler over its own state. Isometry's
//! `sheet.rs` is not a consumer: it is a character-sheet overlay of
//! `.sheet-row` lines with no palette, so §1's row is corrected there too.

pub mod examiner;
pub mod palette;
pub mod sheet;
pub mod viewport;

pub use examiner::{
    ExaminerModel, ExaminerRow, FIELD_CLASS, FIELD_NAME_CLASS, FIELD_VALUE_CLASS, PART_CLASS,
    PARTS_CLASS, READING_CLASS, examiner, field, field_cells, parts_palette, reading_column,
};
pub use palette::{Palette, Picked, Seeds, css_vars, derive, text_contrast};
pub use sheet::{Sizes, css_sizes, from_palette, shared, sheet, sheet_with};
pub use viewport::{
    CARD_CLASS, ERROR_CLASS, LEAF_CLASS, ViewportCard, error_attrs, error_line, scene_card,
    viewport_card, viewport_leaf,
};

/// Re-exported so a product names one crate, not two, when it spells a seed.
pub use tinct::{Srgb, best_on, color_from_hex, color_to_hex, contrast, mix};

#[cfg(test)]
mod tests;
