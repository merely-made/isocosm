//! The small executable ruleset tranche: nine specific procedures over one
//! saved, addressable lifecycle. Edition state lives beside the VTT sheets;
//! native tissue, inventories and anatomy are not silently rewritten.

use isocosm_campaign::{Inventory, ItemId};
use isocosm_core::{FieldValue, MapDocument, Rng, SheetData, TokenId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod casting;
mod damage;
mod definitions;
mod geometry;
mod lifecycle;
mod procedures;
mod projection;

pub use casting::cast;
pub(crate) use damage::damage_expression;
pub use definitions::{Definition, Definitions, Duration, Edition, Kind};
pub use geometry::{BodyChange, project_body};
pub use lifecycle::{advance, concentration_damage, concentration_check, ConcentrationCheck, dismiss, interrupt, set_suppression};
pub use projection::{
    Lighting, Perceived, WeaponChange, perceive, project_sheet, visible_tiles, weapon,
};

const KEY: &str = "isometry:active-effects";
pub const VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    pub tick: u64,
    pub tick_microseconds: u64,
    /// The accepted calendar's next sunrise or sunset. Not a fixed duration.
    pub next_sun: Option<u64>,
    pub next_daily_preparation: Option<u64>,
}

impl Clock {
    pub fn seconds(self, seconds: u64) -> Result<u64, String> {
        let micros = seconds.checked_mul(1_000_000).ok_or("duration overflow")?;
        if self.tick_microseconds == 0 || micros % self.tick_microseconds != 0 {
            return Err("world clock cannot represent this duration exactly".into());
        }
        self.tick
            .checked_add(micros / self.tick_microseconds)
            .ok_or("duration overflow".into())
    }

    /// Use the native sky and clock. Polar/tidally locked skies with no crossing
    /// in this day leave Sun duration to an explicit recorded calendar ruling.
    pub fn from_native(sim: &cosmograph::Simulation, site: Option<u64>) -> Result<Self, String> {
        let rules = &sim.genesis().rules;
        let unit = rules.tick_microseconds();
        if unit == 0 {
            return Err("zero world clock unit".into());
        }
        let tick = sim.state().tick;
        let day = cosmograph::light::DAY_MICROSECONDS / unit;
        let at = site.and_then(|id| sim.state().sites.get(&id));
        let up = |time| cosmograph::light::at(rules, time, at).sun.is_some();
        let initial = up(tick);
        // Bound this optional calendar lookup independently of any wall clock.
        let next_sun = if day <= 86_400 && !rules.sun.as_ref().is_some_and(|s| s.tidal_lock) {
            (1..=day).find_map(|delta| {
                let time = tick.checked_add(delta)?;
                (up(time) != initial).then_some(time)
            })
        } else {
            None
        };
        Ok(Self {
            tick,
            tick_microseconds: unit,
            next_sun,
            next_daily_preparation: None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Creature(TokenId),
    Item(ItemId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    Expired,
    Dismissed,
    Concentration,
    Incapacitated,
    Dispelled { ruling: String },
    Replaced { ruling: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Active {
    pub id: u64,
    pub definition: Definition,
    pub caster: TokenId,
    pub target: Target,
    pub rank: u8,
    pub size_change: i8,
    pub penetration: Option<i64>,
    pub started: u64,
    pub expires: u64,
    pub concentration: bool,
    pub suppressed_by: BTreeSet<String>,
    pub ended: Option<(u64, End)>,
}

impl Active {
    pub fn running(&self, tick: u64) -> bool {
        self.started <= tick && tick < self.expires && self.ended.is_none()
    }
    pub fn applies(&self, tick: u64) -> bool {
        self.running(tick) && self.suppressed_by.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records {
    pub version: u32,
    pub tick: u64,
    pub tick_microseconds: u64,
    pub next_id: u64,
    pub effects: BTreeMap<u64, Active>,
    pub casts: Vec<Receipt>,
}

impl Records {
    /// Geometry and sight caches can bind the same accepted effect revision.
    pub fn revision(&self) -> String {
        cosmograph::digest(self)
    }
    pub fn read(map: &MapDocument) -> Result<Option<Self>, String> {
        let Some(value) = map.system_data.get(KEY) else {
            return Ok(None);
        };
        let text = value
            .as_text()
            .ok_or("active effect state is not encoded text")?;
        let records: Self =
            serde_json::from_str(text).map_err(|e| format!("active effect state: {e}"))?;
        records.validate()?;
        Ok(Some(records))
    }
    pub fn at(map: &MapDocument, clock: Clock) -> Result<Self, String> {
        if clock.tick_microseconds == 0 {
            return Err("zero world clock unit".into());
        }
        let records = Self::read(map)?.unwrap_or(Self {
            version: VERSION,
            tick: clock.tick,
            tick_microseconds: clock.tick_microseconds,
            next_id: 1,
            effects: BTreeMap::new(),
            casts: Vec::new(),
        });
        if records.tick > clock.tick || records.tick_microseconds != clock.tick_microseconds {
            return Err("effect clock moved backwards or changed units".into());
        }
        Ok(records)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION
            || self.tick_microseconds == 0
            || self.next_id == 0
            || self.effects.len() > 4096
            || self.casts.len() > 4096
        {
            return Err("unsupported or invalid active effect state".into());
        }
        let mut requests = BTreeSet::new();
        for cast in &self.casts {
            if !requests.insert(&cast.request.request_id)
                || cast.tick > self.tick
                || cast
                    .effect
                    .is_some_and(|id| !self.effects.contains_key(&id))
            {
                return Err("inconsistent cast history".into());
            }
            if let Some(id) = cast.effect {
                let effect = &self.effects[&id];
                if cast.request.definition != effect.definition.id
                    || cast.request.caster != effect.caster
                    || cast.request.target != effect.target
                    || cast.request.rank != effect.rank
                    || cast.tick != effect.started
                    || cast.definition_revision != effect.definition.revision()
                {
                    return Err("cast history disagrees with accepted effect".into());
                }
            }
        }
        for (id, effect) in &self.effects {
            if *id != effect.id
                || *id >= self.next_id
                || effect.expires <= effect.started
                || effect.started > self.tick
                || effect
                    .ended
                    .as_ref()
                    .is_some_and(|(tick, _)| *tick < effect.started || *tick > self.tick)
            {
                return Err("inconsistent active effect identity or lifetime".into());
            }
            effect.definition.validate()?;
            if effect.concentration != effect.definition.concentration
                || effect.rank < effect.definition.level
                || (effect.definition.kind == Kind::Weapon)
                    != matches!(effect.target, Target::Item(_))
            {
                return Err("active effect disagrees with its accepted procedure".into());
            }
        }
        Ok(())
    }
    fn store(&self, map: &mut MapDocument) -> Result<(), String> {
        self.validate()?;
        map.system_data.insert(
            KEY.into(),
            FieldValue::Text(serde_json::to_string(self).map_err(|e| e.to_string())?),
        );
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cast {
    pub request_id: String,
    pub definition: String,
    pub caster: TokenId,
    pub target: Target,
    pub rank: u8,
    /// D&D enlarge/reduce and Ars growth +1 or shrinking -1/-2.
    pub size_change: i8,
    pub willing: bool,
    /// Explicit GM decision where resistance/counteracting lies outside this
    /// small procedure. Retained in the receipt, never silently assumed.
    pub ruling: Option<String>,
    pub resisted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub request: Cast,
    pub tick: u64,
    pub effect: Option<u64>,
    pub costs: BTreeMap<String, i64>,
    pub dice: Vec<u16>,
    pub casting_total: Option<i64>,
    pub outcome: String,
    pub definition_revision: String,
    /// These specific procedures have executable evidence, not a calibrated
    /// complete edition. Kept on every receipt as required by VTT 5.
    pub experimental: bool,
}

/// Geometry comes from the host's current terrain/visibility machinery. The
/// plugin owns range rules, not another line tracer or a second position store.
pub struct CastContext<'a> {
    pub clock: Clock,
    pub feet_per_tile: u32,
    pub in_combat: bool,
    pub line_of_effect: &'a dyn Fn(isocosm_core::TileCoord, isocosm_core::TileCoord) -> bool,
}

pub(crate) fn add(sheet: &mut SheetData, field: &str, amount: i64) -> Result<(), String> {
    let old = sheet.int(field).unwrap_or(0);
    sheet.set_int(
        field,
        old.checked_add(amount).ok_or("effect statistic overflow")?,
    );
    Ok(())
}
