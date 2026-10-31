//! Looks: a pack's class bindings, and a token's look resolved from them.
//!
//! Doctrine VTT 7 (rulings 1019, 1022, 1028): a look is an isometer
//! `Appearance` of layers and a palette; a pack binds class names (a token's
//! `sprite`) to layers per slot and a palette; a token keeps only its
//! overrides ([`TokenAppearance`]), so whatever it does not override follows
//! the pack. The creator's preview and both boards resolve through
//! [`Library::resolve`], which is what makes them agree.
//!
//! The file format is the pack's `appearance.toml`; the core pack's copy is
//! built in, and a pack found at boot overlays it ([`Library::overlay`]).

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use isometer_mesh::bake::Palette;
use isometer_mesh::voxel::{Rgb, Voxels};
use isocosm_core::TokenAppearance;
use serde::Deserialize;

mod creator;
mod items;
#[cfg(test)]
mod tests;

pub use creator::{Bakes, LookDraft, bake_facing_of};
pub use items::ItemLook;
use items::{ITEM_PALETTE, ItemTables, chipped};

/// The core pack's looks, compiled in so a table with no packs on disk still
/// draws its classes, and so the board's stylesheet can be baked statically.
const CORE: &str = include_str!("../../isocosm-system/examples/packs/core/appearance.toml");

/// The files the core pack's `appearance.toml` names, compiled in with it.
const CORE_FILES: &[(&str, &[u8])] = &[(
    "appearance/hood.vox",
    include_bytes!("../../isocosm-system/examples/packs/core/appearance/hood.vox"),
)];

/// A palette entry the creator offers to recolour.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Swatch {
    pub index: u8,
    pub label: String,
    #[serde(deserialize_with = "colours::deserialize")]
    pub colours: Vec<Rgb>,
}

/// One token's look, resolved: the composed volume, its palette, and a key
/// that changes exactly when either does.
#[derive(Clone, Debug)]
pub struct Resolved {
    pub voxels: Voxels,
    pub palette: Palette,
    pub key: u64,
}

/// Everything a token's look is resolved from: its class, its overrides and
/// the items it has equipped, in slot order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct LookInput {
    pub sprite: String,
    pub appearance: Option<TokenAppearance>,
    pub items: Vec<ItemLook>,
}

impl LookInput {
    /// A bare class, as a token with no overrides and no equipment has it.
    pub fn class(sprite: &str) -> Self {
        Self {
            sprite: sprite.to_owned(),
            ..Self::default()
        }
    }

    /// `token`'s look as the table holds it now.
    pub fn of(ui: &crate::state::UiState, token: &isocosm_core::Token) -> Self {
        let items = ui
            .inventories
            .get(&token.id)
            .map(|inventory| {
                inventory
                    .equipped
                    .iter()
                    .filter_map(|(slot, id)| Some(ItemLook::of(inventory.items.get(id)?, *slot)))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            sprite: token.sprite.clone(),
            appearance: ui.map.appearances.get(&token.id).cloned(),
            items,
        }
    }

    /// Whether this look is its class's alone, which the stylesheet's class
    /// rule already draws.
    pub fn is_bare(&self) -> bool {
        self.appearance.is_none() && self.items.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
struct LayerSource {
    fills: Vec<[i32; 7]>,
    points: Vec<[i32; 4]>,
    /// A layer read from a file, already in palette indices.
    loaded: Option<Voxels>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PaletteEntry {
    #[serde(default, deserialize_with = "colours::deserialize")]
    colours: Vec<Rgb>,
    from: Option<String>,
    #[serde(default)]
    set: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassEntry {
    palette: String,
    #[serde(default)]
    layers: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerEntry {
    #[serde(default)]
    fills: Vec<[i32; 7]>,
    #[serde(default)]
    points: Vec<[i32; 4]>,
    file: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileFormat {
    #[allow(dead_code)]
    format: Option<u32>,
    size: Option<[i32; 3]>,
    slots: Option<Vec<String>>,
    #[serde(default)]
    palettes: BTreeMap<String, PaletteEntry>,
    #[serde(default)]
    classes: BTreeMap<String, ClassEntry>,
    #[serde(default)]
    choices: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    swatches: Vec<Swatch>,
    #[serde(default)]
    item_sorts: BTreeMap<String, String>,
    #[serde(default)]
    sorts: BTreeMap<String, items::SortEntry>,
    #[serde(default)]
    materials: BTreeMap<String, String>,
    #[serde(default)]
    makes: BTreeMap<String, items::MakeEntry>,
    #[serde(default)]
    conditions: BTreeMap<String, items::ConditionEntry>,
    #[serde(default)]
    layers: BTreeMap<String, LayerEntry>,
}

/// Every class, layer and palette the loaded packs declare.
#[derive(Clone, Debug)]
pub struct Library {
    size: [i32; 3],
    slots: Vec<String>,
    palettes: BTreeMap<String, PaletteEntry>,
    classes: BTreeMap<String, ClassEntry>,
    choices: BTreeMap<String, Vec<String>>,
    swatches: Vec<Swatch>,
    items: ItemTables,
    layers: BTreeMap<String, LayerSource>,
    /// Moves with every overlay, so a resolved key never outlives the
    /// bindings it was resolved under.
    fingerprint: u64,
}

impl Default for Library {
    fn default() -> Self {
        Self::core()
    }
}

impl Library {
    fn empty() -> Self {
        Self {
            size: [1, 1, 1],
            slots: Vec::new(),
            palettes: BTreeMap::new(),
            classes: BTreeMap::new(),
            choices: BTreeMap::new(),
            swatches: Vec::new(),
            items: ItemTables::default(),
            layers: BTreeMap::new(),
            fingerprint: 0,
        }
    }

    /// The core pack's looks. The built-in copy is checked by the tests, so
    /// a broken core file fails them rather than a board.
    pub fn core() -> Self {
        let mut library = Self::empty();
        library
            .overlay(CORE, &|path: &str| {
                CORE_FILES
                    .iter()
                    .find(|(name, _)| *name == path)
                    .map(|(_, bytes)| bytes.to_vec())
                    .ok_or_else(|| format!("the core pack has no file {path}"))
            })
            .expect("the core pack's appearance.toml parses");
        library
    }

    /// Lay one pack's `appearance.toml` over what is loaded: every name it
    /// declares replaces the same name, and everything else stays. `read`
    /// fetches a `file` relative to that pack's root.
    pub fn overlay(
        &mut self,
        text: &str,
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    ) -> Result<(), String> {
        let file: FileFormat = toml::from_str(text).map_err(|error| error.to_string())?;
        let mut layers = BTreeMap::new();
        for (name, entry) in file.layers {
            let loaded = match &entry.file {
                Some(path) => Some(load_layer(path, &read(path)?)?),
                None => None,
            };
            layers.insert(
                name,
                LayerSource {
                    fills: entry.fills,
                    points: entry.points,
                    loaded,
                },
            );
        }
        if let Some(size) = file.size {
            if size.iter().any(|d| *d <= 0) {
                return Err("an appearance frame needs a positive size".into());
            }
            self.size = size;
        }
        if let Some(slots) = file.slots {
            self.slots = slots;
        }
        self.palettes.extend(file.palettes);
        self.classes.extend(file.classes);
        self.choices.extend(file.choices);
        self.items.extend(ItemTables {
            item_sorts: file.item_sorts,
            sorts: file.sorts,
            materials: file.materials,
            makes: file.makes,
            conditions: file.conditions,
        });
        self.layers.extend(layers);
        for swatch in file.swatches {
            self.swatches.retain(|held| held.index != swatch.index);
            self.swatches.push(swatch);
        }
        self.swatches.sort_by_key(|swatch| swatch.index);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (self.fingerprint, text).hash(&mut hasher);
        self.fingerprint = hasher.finish();
        Ok(())
    }

    /// Moves whenever the loaded bindings do.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    /// The paper-doll order a class's layers draw in.
    pub fn slots(&self) -> &[String] {
        &self.slots
    }

    /// Every class the loaded packs bind, in name order.
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.classes.keys().map(String::as_str)
    }

    pub fn has_class(&self, sprite: &str) -> bool {
        self.classes.contains_key(sprite)
    }

    /// What the creator offers in `slot`; `""` is nothing.
    pub fn choices(&self, slot: &str) -> &[String] {
        self.choices.get(slot).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn swatches(&self) -> &[Swatch] {
        &self.swatches
    }

    /// The layer `slot` shows for `sprite`, with `overrides` taken first.
    pub fn slot_layer<'a>(
        &'a self,
        sprite: &str,
        overrides: Option<&'a TokenAppearance>,
        slot: &str,
    ) -> Option<&'a str> {
        overrides
            .and_then(|look| look.layers.get(slot))
            .or_else(|| self.classes.get(sprite)?.layers.get(slot))
            .map(String::as_str)
    }

    /// The colour palette entry `index` has for `sprite` under `overrides`.
    pub fn colour(&self, sprite: &str, overrides: Option<&TokenAppearance>, index: u8) -> Option<Rgb> {
        if let Some(colour) = overrides.and_then(|look| look.palette.get(&index)) {
            return Some(*colour);
        }
        let class = self.classes.get(sprite)?;
        self.palette(&class.palette, 0).ok()?.get(index as usize).copied()
    }

    /// Resolve a token's look: its class's layers with `overrides` taken
    /// first, then each equipped item's derived look (armour under what is
    /// held), over the class palette with `overrides`' entries replaced and
    /// each hand's item colours after it.
    ///
    /// A class or layer no pack declares is an error, so the caller draws the
    /// visible placeholder rather than a token silently missing its hat.
    pub fn resolve(
        &self,
        sprite: &str,
        overrides: Option<&TokenAppearance>,
        items: &[ItemLook],
    ) -> Result<Resolved, String> {
        let class = self
            .classes
            .get(sprite)
            .ok_or_else(|| format!("no pack binds the class `{sprite}`"))?;
        let mut names: Vec<&str> = Vec::new();
        for slot in &self.slots {
            if let Some(name) = self.slot_layer(sprite, overrides, slot) {
                if !name.is_empty() {
                    names.push(name);
                }
            }
        }
        let mut colours = self.palette(&class.palette, 0)?;
        if let Some(look) = overrides {
            for (index, colour) in &look.palette {
                set_colour(&mut colours, *index as usize, *colour);
            }
        }
        let [dx, dy, dz] = self.size;
        let mut voxels = Voxels::new(dx, dy, dz);
        for name in &names {
            let layer = self.layer(name)?;
            draw(&mut voxels, layer, |x, y, z, index| Some((x, y, z, index)));
        }
        let drawn = self.draw_items(&mut voxels, &mut colours, items)?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (self.fingerprint, &names, &drawn, &colours).hash(&mut hasher);
        Ok(Resolved {
            voxels,
            palette: Palette::new(colours),
            key: hasher.finish(),
        })
    }

    /// One item alone, as the variant strip shows it.
    pub fn resolve_item(&self, item: &ItemLook) -> Result<Resolved, String> {
        let [dx, dy, dz] = self.size;
        let mut voxels = Voxels::new(dx, dy, dz);
        let mut colours = Vec::new();
        let drawn = self.draw_items(&mut voxels, &mut colours, std::slice::from_ref(item))?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (self.fingerprint, &drawn, &colours).hash(&mut hasher);
        Ok(Resolved {
            voxels,
            palette: Palette::new(colours),
            key: hasher.finish(),
        })
    }

    /// `item` again under other material, make and condition keys, `count`
    /// of them, walking the pack's own vocabularies so every pack shows its
    /// range. The first is the item as it is.
    pub fn item_variants(&self, item: &ItemLook, count: usize) -> Vec<ItemLook> {
        let materials = self.variant_materials(item);
        let makes: Vec<&str> = self.items.makes.keys().map(String::as_str).collect();
        let conditions: Vec<Option<&str>> = std::iter::once(None)
            .chain(self.items.conditions.keys().map(|key| Some(key.as_str())))
            .collect();
        let mut out = vec![item.clone()];
        for i in 1..count {
            let mut variant = item.clone();
            if !materials.is_empty() {
                variant.material = Some(materials[i % materials.len()].to_owned());
            }
            if !makes.is_empty() {
                variant.grade = Some(makes[i % makes.len()].to_owned());
            }
            variant.condition = conditions[i % conditions.len()].map(str::to_owned);
            variant.seed = item.seed.wrapping_add((i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
            out.push(variant);
        }
        out
    }

    fn layer(&self, name: &str) -> Result<&LayerSource, String> {
        self.layers
            .get(name)
            .ok_or_else(|| format!("no pack declares the layer `{name}`"))
    }

    /// Draw each item over `voxels` in paper-doll order and set its colours;
    /// returns what was drawn, for the look's key.
    fn draw_items(
        &self,
        voxels: &mut Voxels,
        colours: &mut Vec<Rgb>,
        items: &[ItemLook],
    ) -> Result<Vec<(String, bool, u8, u64)>, String> {
        if colours.len() < ITEM_PALETTE {
            colours.resize(ITEM_PALETTE, [255, 0, 255]);
        }
        let mut ordered: Vec<&ItemLook> = items.iter().collect();
        ordered.sort_by_key(|item| item.order());
        let mut drawn = Vec::new();
        let dx = self.size[0];
        for item in ordered {
            let Some(look) = self.item_draw(item)? else {
                continue;
            };
            for (i, colour) in look.colours.iter().enumerate() {
                set_colour(colours, look.base as usize + i, *colour);
            }
            let layer = self.layer(look.layer)?;
            draw(voxels, layer, |x, y, z, index| {
                if index == 0 && chipped(look.seed, x, y, z, look.chip) {
                    return None;
                }
                let x = if look.mirror { dx - 1 - x } else { x };
                Some((x, y, z, look.base + index.min(3)))
            });
            drawn.push((look.layer.to_owned(), look.mirror, look.base, look.seed));
        }
        Ok(drawn)
    }

    /// A named palette with its `from` chain applied. The depth bound keeps a
    /// cycle an error instead of a hang.
    fn palette(&self, name: &str, depth: usize) -> Result<Vec<Rgb>, String> {
        if depth > 16 {
            return Err(format!("the palette `{name}` derives from itself"));
        }
        let entry = self
            .palettes
            .get(name)
            .ok_or_else(|| format!("no pack declares the palette `{name}`"))?;
        let mut colours = match &entry.from {
            Some(base) => self.palette(base, depth + 1)?,
            None => Vec::new(),
        };
        for (i, colour) in entry.colours.iter().enumerate() {
            if colours.len() <= i {
                colours.resize(i + 1, [255, 0, 255]);
            }
            colours[i] = *colour;
        }
        for (index, colour) in &entry.set {
            let i: usize = index
                .parse()
                .map_err(|_| format!("palette `{name}` sets `{index}`, which is not an index"))?;
            if colours.len() <= i {
                colours.resize(i + 1, [255, 0, 255]);
            }
            colours[i] = parse_colour(colour)?;
        }
        Ok(colours)
    }
}

fn set_colour(colours: &mut Vec<Rgb>, i: usize, colour: Rgb) {
    if colours.len() <= i {
        colours.resize(i + 1, [255, 0, 255]);
    }
    colours[i] = colour;
}

/// Every voxel of `layer`, passed through `place` (which may move, recolour
/// or drop it) onto `voxels`.
fn draw(
    voxels: &mut Voxels,
    layer: &LayerSource,
    place: impl Fn(i32, i32, i32, u8) -> Option<(i32, i32, i32, u8)>,
) {
    let mut put = |x: i32, y: i32, z: i32, index: u8| {
        if let Some((x, y, z, index)) = place(x, y, z, index) {
            voxels.set(x, y, z, index);
        }
    };
    if let Some(loaded) = &layer.loaded {
        for (x, y, z, index) in loaded.iter() {
            put(x, y, z, index);
        }
    }
    for [x0, x1, y0, y1, z0, z1, index] in &layer.fills {
        for z in *z0..*z1 {
            for y in *y0..*y1 {
                for x in *x0..*x1 {
                    put(x, y, z, *index as u8);
                }
            }
        }
    }
    for [x, y, z, index] in &layer.points {
        put(*x, *y, *z, *index as u8);
    }
}

/// A `file` layer: another TOML layer (fills and points) or a `.vox`.
fn load_layer(path: &str, bytes: &[u8]) -> Result<Voxels, String> {
    if path.ends_with(".vox") {
        // `dot_vox` already reads a file's colour slot n as index n - 1, so a
        // .vox's first colour is palette index 0 here; the file's own colours
        // are not used.
        let (voxels, _) = isometer_mesh::bake::load_vox(bytes)
            .ok_or_else(|| format!("`{path}` is not a readable .vox"))?;
        return Ok(voxels);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| format!("`{path}` is not text"))?;
    let entry: LayerEntry = toml::from_str(text).map_err(|error| format!("{path}: {error}"))?;
    if entry.file.is_some() {
        return Err(format!("`{path}` names another file; a layer file holds its own voxels"));
    }
    let width = entry.fills.iter().map(|f| f[1]).chain(entry.points.iter().map(|p| p[0] + 1));
    let height = entry.fills.iter().map(|f| f[3]).chain(entry.points.iter().map(|p| p[1] + 1));
    let depth = entry.fills.iter().map(|f| f[5]).chain(entry.points.iter().map(|p| p[2] + 1));
    let mut voxels = Voxels::new(
        width.max().unwrap_or(1).max(1),
        height.max().unwrap_or(1).max(1),
        depth.max().unwrap_or(1).max(1),
    );
    for [x0, x1, y0, y1, z0, z1, index] in &entry.fills {
        voxels.fill(*x0, *x1, *y0, *y1, *z0, *z1, *index as u8);
    }
    for [x, y, z, index] in &entry.points {
        voxels.set(*x, *y, *z, *index as u8);
    }
    Ok(voxels)
}

/// `#rrggbb` to a colour.
pub fn parse_colour(text: &str) -> Result<Rgb, String> {
    let hex = text.strip_prefix('#').unwrap_or(text);
    let channel = |i: usize| {
        hex.get(i..i + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .ok_or_else(|| format!("`{text}` is not a #rrggbb colour"))
    };
    if hex.len() != 6 {
        return Err(format!("`{text}` is not a #rrggbb colour"));
    }
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

/// A colour as `#rrggbb`, for CSS.
pub fn colour_hex([r, g, b]: Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

mod colours {
    use super::*;
    use serde::Deserializer;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Rgb>, D::Error> {
        let texts = Vec::<String>::deserialize(d)?;
        texts
            .iter()
            .map(|text| parse_colour(text).map_err(serde::de::Error::custom))
            .collect()
    }
}
