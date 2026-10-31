//! Tokens as live bodies: each token's look, resolved and meshed.
//!
//! A token's look is its class plus its overrides plus its equipped items'
//! layers, resolved by [`Library::resolve`] (doctrine VTT 7). The DOM board
//! bakes the same resolution to sprite pixels, and the creator's preview reads
//! it too, so the three cannot disagree. A body is built once per resolved
//! key and shared by every token with that look. A look that does not resolve
//! draws as a one-voxel magenta stand-in and is counted, rather than a silently missing
//! piece on the board.
//!
//! **Scale.** A look is authored in its own voxels; the board's unit is the
//! tile, which the subdivision ruling made [`VOXELS_PER_TILE`] world units
//! wide. The rig is stood [`FOOTPRINT`] of a tile wide either way, which puts
//! the 8-voxel-wide humanoid at three quarters of a diamond, the same pixels
//! as before the subdivision, because a tile is still a tile.

use std::collections::HashMap;
use std::rc::Rc;

use isometer::mesh::VolumeSource;
use isometer::mesh::bake::Palette;
use isometer::mesh::voxel::Voxels;
use isometer::{PaletteColour, TokenBody, VolumeMap, material_colours};
use isometry_core::{Facing, TokenId};

use super::terrain::VOXELS_PER_TILE;
use crate::appearance::{Library, LookInput};

/// A token's footprint as a fraction of a tile's diagonal.
pub const FOOTPRINT: f32 = 0.75;

struct Built {
    body: TokenBody,
    colours: Vec<PaletteColour>,
}

/// Every token body the board draws, plus the one volume map the frame
/// resolves them through.
pub struct TokenBodies {
    library: Rc<Library>,
    /// Bodies by resolved key: one per distinct look, however many wear it.
    built: HashMap<u64, Built>,
    /// Each token's last look and what it resolved to; `None` is the
    /// placeholder.
    keys: HashMap<TokenId, (LookInput, Option<u64>)>,
    placeholder: Built,
    /// The scene's volume source: every distinct look, collapsed by content.
    pub volumes: VolumeMap,
}

impl Default for TokenBodies {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenBodies {
    /// A table over the core pack's looks.
    pub fn new() -> Self {
        Self::with_library(Rc::new(Library::core()))
    }

    pub fn with_library(library: Rc<Library>) -> Self {
        // The stand-in: one voxel of palette index 0, which becomes material
        // 1 and takes the placeholder colour.
        let mut cell = Voxels::new(1, 1, 1);
        cell.set(0, 0, 0, 0);
        let placeholder = Built {
            body: TokenBody::from_voxels(&cell),
            colours: material_colours(&Palette::new(vec![PLACEHOLDER_COLOUR])),
        };
        let mut volumes = VolumeMap::new();
        add_volume(&mut volumes, &placeholder.body);
        Self {
            library,
            built: HashMap::new(),
            keys: HashMap::new(),
            placeholder,
            volumes,
        }
    }

    /// Takes the bindings the board now resolves through. A different
    /// library re-resolves every token on its next [`Self::sync`].
    pub fn set_library(&mut self, library: &Rc<Library>) {
        if !Rc::ptr_eq(&self.library, library) {
            self.library = library.clone();
            self.keys.clear();
        }
    }

    pub fn library(&self) -> &Rc<Library> {
        &self.library
    }

    /// Brings `id`'s body up to `look`, building it if no token wore that
    /// look before, and returns the key its geometry and colours go by.
    pub fn sync(&mut self, id: TokenId, look: &LookInput) -> Option<u64> {
        if let Some((held, key)) = self.keys.get(&id) {
            if held == look {
                return *key;
            }
        }
        let items = look.items.iter().map(String::as_str);
        let key = match self.library.resolve(&look.sprite, look.appearance.as_ref(), items) {
            Ok(resolved) => {
                let key = resolved.key;
                if !self.built.contains_key(&key) {
                    let body = TokenBody::from_voxels(&resolved.voxels);
                    add_volume(&mut self.volumes, &body);
                    let colours = material_colours(&resolved.palette);
                    self.built.insert(key, Built { body, colours });
                }
                Some(key)
            },
            Err(_) => None,
        };
        self.keys.insert(id, (look.clone(), key));
        key
    }

    /// The key `id`'s look last resolved to; `None` is the placeholder.
    pub fn key(&self, id: TokenId) -> Option<u64> {
        self.keys.get(&id).and_then(|(_, key)| *key)
    }

    fn built(&self, id: TokenId) -> (&Built, bool) {
        match self.key(id).and_then(|key| self.built.get(&key)) {
            Some(built) => (built, false),
            None => (&self.placeholder, true),
        }
    }

    /// The body `id` draws as, and whether it is the placeholder.
    pub fn body(&self, id: TokenId) -> (&TokenBody, bool) {
        let (built, placeholder) = self.built(id);
        (&built.body, placeholder)
    }

    /// The colour table `id`'s materials index into: its palette shifted by
    /// one, as [`material_colours`] gives it.
    pub fn colours(&self, id: TokenId) -> Vec<PaletteColour> {
        self.built(id).0.colours.clone()
    }

    /// The scale that stands `id` [`FOOTPRINT`] of a tile wide. A tile is
    /// [`VOXELS_PER_TILE`] world units, so the on-screen size is what it was
    /// before the grid was subdivided.
    pub fn scale(&self, id: TokenId) -> f32 {
        let size = self.body(id).0.size();
        let widest = size[0].max(size[2]).max(1) as f32;
        FOOTPRINT * VOXELS_PER_TILE as f32 / widest
    }
}

fn add_volume(volumes: &mut VolumeMap, body: &TokenBody) {
    if let Some(volume) = body.volumes.volume(body.volume) {
        volumes.insert(body.volume, volume.clone());
    }
}

/// Magenta: a placeholder that reads as a mistake at a glance.
const PLACEHOLDER_COLOUR: [u8; 3] = [220, 40, 200];

/// The yaw a four-way facing turns a body to.
///
/// The board's axes are world `+x` for a column and world `+z` for a row, so
/// South (row increasing) is the zero and each quarter turn follows the
/// compass anticlockwise about world up. The rig is symmetric enough that this
/// reads as a turn rather than as a specific pose; B3 owns whatever facing art
/// wants beyond it.
pub fn yaw_of(facing: Facing) -> f32 {
    use std::f32::consts::FRAC_PI_2;
    match facing {
        Facing::South => 0.0,
        Facing::West => FRAC_PI_2,
        Facing::North => std::f32::consts::PI,
        Facing::East => 3.0 * FRAC_PI_2,
    }
}

/// The tint an owner's pieces are drawn in.
///
/// Sides are named by the map, not by the substrate, so the tint is a hash of
/// the owner string folded into a light pastel: distinguishable, and never a
/// colour that swamps the recipe's own palette. No owner draws untinted.
pub fn owner_tint(owner: Option<&str>) -> [f32; 3] {
    let Some(owner) = owner else {
        return [1.0; 3];
    };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in owner.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    [0, 1, 2].map(|channel| 0.72 + ((hash >> (channel * 8)) & 0xff) as f32 / 255.0 * 0.28)
}
