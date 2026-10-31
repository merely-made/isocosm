//! Item looks, derived from what the item is (case 1035, doctrine VTT 7):
//! shape from its sort, palette from its material, finish from its make's
//! grade, wear from its condition, variation from its seed, every table read
//! from the pack (case 1043: open keys, a pack mapping them).
//!
//! An item layer is authored in item colours 0 to 3 (material, secondary,
//! accent, detail). Each equip hand owns four palette entries past the
//! wearer's own, so a sword and a shield of different materials never share
//! a colour.

use super::*;

/// An equipped item, as its look is derived from it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ItemLook {
    pub template: String,
    /// Its kind tag, the sort's fallback (`kind:<tag>` in the pack).
    pub kind: Option<String>,
    /// Where it is worn: `main`, `off`, `body`, or anything else.
    pub hand: String,
    pub material: Option<String>,
    pub grade: Option<String>,
    pub condition: Option<String>,
    /// From the item's identity, so one item always varies the same way.
    pub seed: u64,
}

impl ItemLook {
    /// `item` worn in `slot`.
    pub fn of(item: &isocosm_campaign::ItemInstance, slot: isocosm_campaign::EquipmentSlot) -> Self {
        use isocosm_campaign::EquipmentSlot;
        let hand = match slot {
            EquipmentSlot::MainHand => "main",
            EquipmentSlot::OffHand => "off",
            EquipmentSlot::Body => "body",
            _ => "other",
        };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        item.id.0.hash(&mut hasher);
        Self {
            template: item.template.clone(),
            kind: item.tags.first().cloned(),
            hand: hand.to_owned(),
            material: item.material.clone(),
            grade: item.make.as_ref().and_then(|make| make.grade.clone()),
            condition: item.condition.clone(),
            seed: hasher.finish(),
        }
    }

    /// Paper-doll order: armour under what is held.
    pub(super) fn order(&self) -> u8 {
        match self.hand.as_str() {
            "body" => 0,
            "other" => 1,
            "off" => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SortEntry {
    hand: String,
    shapes: Vec<String>,
    material: Option<String>,
    /// What the sort is made of; its variants walk these.
    #[serde(default)]
    materials: Vec<String>,
    #[serde(deserialize_with = "colour::deserialize")]
    secondary: Rgb,
    #[serde(deserialize_with = "colour::deserialize")]
    accent: Rgb,
    #[serde(deserialize_with = "colour::deserialize")]
    detail: Rgb,
    #[serde(default)]
    jitter: u8,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MakeEntry {
    #[serde(default)]
    shade: i16,
    accent: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConditionEntry {
    #[serde(default)]
    shade: i16,
    #[serde(default)]
    fade: f32,
    #[serde(default)]
    chip: f32,
}

/// The pack's item tables, overlaid name by name like the rest.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ItemTables {
    #[serde(default)]
    pub(super) item_sorts: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) sorts: BTreeMap<String, SortEntry>,
    #[serde(default)]
    pub(super) materials: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) makes: BTreeMap<String, MakeEntry>,
    #[serde(default)]
    pub(super) conditions: BTreeMap<String, ConditionEntry>,
}

impl ItemTables {
    pub(super) fn extend(&mut self, other: ItemTables) {
        self.item_sorts.extend(other.item_sorts);
        self.sorts.extend(other.sorts);
        self.materials.extend(other.materials);
        self.makes.extend(other.makes);
        self.conditions.extend(other.conditions);
    }

    /// The material keys the pack draws, in name order.
    pub(super) fn material_keys(&self) -> Vec<&str> {
        self.materials.keys().map(String::as_str).collect()
    }
}

/// One item's derived look: the shape to draw, where, and in what colours.
pub(super) struct ItemDraw<'a> {
    pub(super) layer: &'a str,
    pub(super) mirror: bool,
    pub(super) base: u8,
    pub(super) colours: [Rgb; 4],
    pub(super) chip: f32,
    pub(super) seed: u64,
}

/// The first palette entry each hand's four item colours take.
pub(super) fn hand_base(hand: &str) -> u8 {
    match hand {
        "main" => 10,
        "off" => 14,
        "body" => 18,
        _ => 22,
    }
}

/// The palette length every look is padded to, so item colours always exist.
pub(super) const ITEM_PALETTE: usize = 26;

impl Library {
    /// The materials `item`'s variants walk: its sort's own, else every one
    /// the pack draws.
    pub(super) fn variant_materials(&self, item: &ItemLook) -> Vec<&str> {
        let sort = self.item_sort(item).and_then(|name| self.items.sorts.get(name));
        match sort {
            Some(sort) if !sort.materials.is_empty() => sort.materials.iter().map(String::as_str).collect(),
            _ => self.items.material_keys(),
        }
    }

    /// The sort an item draws as: its template's, else its kind tag's.
    pub fn item_sort(&self, item: &ItemLook) -> Option<&str> {
        self.items
            .item_sorts
            .get(&item.template)
            .or_else(|| self.items.item_sorts.get(&format!("kind:{}", item.kind.as_deref()?)))
            .map(String::as_str)
    }

    /// Derive an item's look; `None` when no pack binds its sort.
    pub(super) fn item_draw(&self, item: &ItemLook) -> Result<Option<ItemDraw<'_>>, String> {
        let Some(sort_name) = self.item_sort(item) else {
            return Ok(None);
        };
        let sort = self
            .items
            .sorts
            .get(sort_name)
            .ok_or_else(|| format!("no pack declares the sort `{sort_name}`"))?;
        if sort.shapes.is_empty() {
            return Err(format!("the sort `{sort_name}` has no shapes"));
        }
        let layer = sort.shapes[(item.seed % sort.shapes.len() as u64) as usize].as_str();
        // An open key the pack does not map falls back to the sort's own.
        let material = item
            .material
            .as_deref()
            .and_then(|key| self.items.materials.get(key))
            .or_else(|| sort.material.as_deref().and_then(|key| self.items.materials.get(key)))
            .map(|text| parse_colour(text))
            .transpose()?
            .unwrap_or([255, 0, 255]);
        let make = item.grade.as_deref().and_then(|key| self.items.makes.get(key));
        let accent = match make.and_then(|make| make.accent.as_deref()) {
            Some(text) => parse_colour(text)?,
            None => sort.accent,
        };
        let jitter = sort.jitter as i16;
        let offset = |shift: u32| {
            if jitter == 0 {
                0
            } else {
                ((item.seed >> shift) % (2 * jitter as u64 + 1)) as i16 - jitter
            }
        };
        let mut primary = shade(material, make.map_or(0, |make| make.shade));
        for (channel, shift) in [(0, 8), (1, 16), (2, 24)] {
            primary[channel] = (primary[channel] as i16 + offset(shift)).clamp(0, 255) as u8;
        }
        let mut colours = [primary, sort.secondary, accent, sort.detail];
        let condition = item.condition.as_deref().and_then(|key| self.items.conditions.get(key));
        if let Some(condition) = condition {
            for colour in &mut colours {
                *colour = fade(shade(*colour, condition.shade), condition.fade);
            }
        }
        Ok(Some(ItemDraw {
            layer,
            mirror: sort.hand != item.hand && matches!(item.hand.as_str(), "main" | "off"),
            base: hand_base(&item.hand),
            colours,
            chip: condition.map_or(0.0, |condition| condition.chip),
            seed: item.seed,
        }))
    }
}

fn shade(colour: Rgb, by: i16) -> Rgb {
    colour.map(|c| (c as i16 + by).clamp(0, 255) as u8)
}

fn fade(colour: Rgb, by: f32) -> Rgb {
    let by = by.clamp(0.0, 1.0);
    let grey = (colour[0] as f32 * 0.3 + colour[1] as f32 * 0.59 + colour[2] as f32 * 0.11) as f32;
    colour.map(|c| (c as f32 + (grey - c as f32) * by).round() as u8)
}

/// Whether wear has taken the voxel at `(x, y, z)`: a fixed draw per voxel
/// and seed, so the same item always loses the same bits.
pub(super) fn chipped(seed: u64, x: i32, y: i32, z: i32, chip: f32) -> bool {
    if chip <= 0.0 {
        return false;
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (seed, x, y, z).hash(&mut hasher);
    (hasher.finish() % 1000) as f32 / 1000.0 < chip
}

mod colour {
    use super::*;
    use serde::Deserializer;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Rgb, D::Error> {
        let text = String::deserialize(d)?;
        parse_colour(&text).map_err(serde::de::Error::custom)
    }
}
