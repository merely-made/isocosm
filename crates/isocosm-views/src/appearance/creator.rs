//! The creator's working look, and the baked sprites the DOM side shows.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use isometer_mesh::bake::{BakeParams, bake_facing};
use isocosm_core::{Facing, TokenAppearance, TokenId};

use super::{Library, LookInput};

/// What the creator is editing: a token's class and overrides, held apart
/// from the table until accepted, so cancelling leaves the table untouched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LookDraft {
    /// The token being edited; `None` while creating a new one.
    pub token: Option<TokenId>,
    pub sprite: String,
    pub appearance: TokenAppearance,
    /// The equipped item templates the token wears, so the preview shows what
    /// the board will.
    pub items: Vec<String>,
    /// The preview's quarter turn; never written to the table.
    pub facing: Facing,
}

impl LookDraft {
    pub fn new(sprite: &str) -> Self {
        Self {
            token: None,
            sprite: sprite.to_owned(),
            appearance: TokenAppearance::default(),
            items: Vec::new(),
            facing: Facing::South,
        }
    }

    /// The draft as a look to resolve.
    pub fn input(&self) -> LookInput {
        LookInput {
            sprite: self.sprite.clone(),
            appearance: (!self.appearance.is_empty()).then(|| self.appearance.clone()),
            items: self.items.clone(),
        }
    }

    /// Show `layer` in `slot`. The class's own choice clears the override, so
    /// a reskin of the class reaches the slot again.
    pub fn set_layer(&mut self, library: &Library, slot: &str, layer: &str) {
        if library.slot_layer(&self.sprite, None, slot) == Some(layer) {
            self.appearance.layers.remove(slot);
        } else {
            self.appearance.layers.insert(slot.to_owned(), layer.to_owned());
        }
    }

    /// Recolour palette entry `index`. The class's own colour clears the
    /// override, for the same reason as [`Self::set_layer`].
    pub fn set_colour(&mut self, library: &Library, index: u8, colour: [u8; 3]) {
        if library.colour(&self.sprite, None, index) == Some(colour) {
            self.appearance.palette.remove(&index);
        } else {
            self.appearance.palette.insert(index, colour);
        }
    }

    /// One quarter turn anticlockwise, the board's own facing order.
    pub fn turn(&mut self) {
        self.facing = match self.facing {
            Facing::South => Facing::East,
            Facing::East => Facing::North,
            Facing::North => Facing::West,
            Facing::West => Facing::South,
        };
    }
}

/// The bake facing a board facing shows. Measured in the running game
/// (2026-10-30, four headed previews): bake 0 shows the figure's back with its
/// front toward the screen's upper right, which is north (row decreasing), and
/// each step turns it a quarter clockwise as seen from above.
pub fn bake_facing_of(facing: Facing) -> u8 {
    match facing {
        Facing::North => 0,
        Facing::East => 1,
        Facing::South => 2,
        Facing::West => 3,
    }
}

/// Baked sprites as data URIs, by look, facing and stamp. A view fn runs on
/// every state change, so a look is baked once and its URI reused.
#[derive(Default)]
pub struct Bakes {
    held: RefCell<HashMap<(LookInput, u8, i32, u64), Rc<str>>>,
}

impl Bakes {
    /// `look` baked at `facing` with `params`, or `None` if it does not
    /// resolve.
    pub fn uri(
        &self,
        library: &Library,
        look: &LookInput,
        facing: u8,
        params: &BakeParams,
    ) -> Option<Rc<str>> {
        let key = (look.clone(), facing, params.half_w, library.fingerprint());
        if let Some(uri) = self.held.borrow().get(&key) {
            return Some(uri.clone());
        }
        let items = look.items.iter().map(String::as_str);
        let resolved = library.resolve(&look.sprite, look.appearance.as_ref(), items).ok()?;
        let uri: Rc<str> = bake_facing(&resolved.voxels, &resolved.palette, facing, params)
            .to_png_data_uri()
            .into();
        let mut held = self.held.borrow_mut();
        // A creator session can try many looks; keep the cache bounded.
        if held.len() > 512 {
            held.clear();
        }
        held.insert(key, uri.clone());
        Some(uri)
    }
}
