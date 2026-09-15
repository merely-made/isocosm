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

pub mod palette;
pub mod sheet;

pub use palette::{Palette, Picked, Seeds, css_vars, derive, text_contrast};
pub use sheet::{Sizes, css_sizes, from_palette, shared, sheet, sheet_with};

/// Re-exported so a product names one crate, not two, when it spells a seed.
pub use tinct::{Srgb, best_on, color_from_hex, color_to_hex, contrast, mix};

#[cfg(test)]
mod tests;
