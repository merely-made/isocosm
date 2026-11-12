// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world rule of temperature and what it costs a body outside its
//! comfort (case 1215; lineage-rounds D13 to D15). Unset, a world has no
//! temperature and hashes as before.

use crate::{Result, schema::Tick};
use serde::{Deserialize, Serialize};

/// Temperature and what it costs a body outside its comfort (case 1215;
/// lineage-rounds D13 to D15). Degrees are whole and of no particular scale.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Climate {
    /// Degrees at the equator, at sea level, at the year's mean.
    pub base: i64,
    /// Degrees lost from the equator to a pole.
    pub poleward: i64,
    /// Degrees lost for each `rise` of height above the sea.
    pub lapse: i64,
    pub rise: i64,
    /// Degrees the year swings either side of its mean at a pole (nought at the equator).
    pub season: i64,
    /// Degrees weather moves it either way, and how many ticks one spell lasts.
    pub weather: i64,
    pub spell: Tick,
    /// The comfort band: its centre for a body of `reference` adult mass, its
    /// half-width, and the degrees the centre falls for each doubling of mass.
    pub centre: i64,
    pub width: i64,
    pub reference: u64,
    pub doubling: i64,
    /// Matter a tick (numerator, denominator) for each degree outside the band.
    pub strain: [u32; 2],
    /// Water: the least `world:water` (0 to 100) a body with no store cells
    /// is comfortable with, how far that falls for a body all of whose cells
    /// store, and the matter a tick (numerator, denominator) for each point
    /// of water below it (lineage-rounds D14).
    pub thirst: i64,
    pub buffer: i64,
    pub drought: [u32; 2],
    /// Light: what a body all of whose cells fix needs, on the scale the
    /// ecology stages sunlight at (nought to its full credit, 500), and the
    /// matter a tick (numerator, denominator) for each unit of light below it.
    pub shade: i64,
    pub dim: [u32; 2],
    /// The most degrees a holder of a world-class lineage may shift every
    /// temperature by, either way (cases 1216, 1217); none, cycles cannot be
    /// turned, which serializes and hashes as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<i32>,
}

impl Default for Climate {
    /// Tuned to a survival target (lineage-rounds D26): with the crowding
    /// brake and `Limits::default()` on, about nine seeds in ten found and keep
    /// the played line alive 600 ticks. The three strain rates are the first
    /// cut's divided by thirty. At the first cut's rates only 18 of 40 seeds
    /// founded and 2 kept the played line; a tenth of them kept 31, a
    /// fifteenth and a twentieth fell short on seeds 40 to 59, a thirtieth kept 38.
    fn default() -> Self {
        Self {
            // Warm at the equator, so the band's centre (20) is met mid-latitude.
            base: 30,
            // A pole is 50 colder than the equator.
            poleward: 50,
            // Sites stand a few hundred base units high: 6 a 512 is mild.
            lapse: 6,
            rise: 512,
            // The year swings 15 either side of its mean at a pole.
            season: 15,
            // Weather is a few degrees, over three days at a minute a tick.
            weather: 4,
            spell: 4_320,
            // The centre is mild and the band 20 wide.
            centre: 20,
            width: 10,
            // The body rules' reference mass in mg.
            reference: 100,
            // Two degrees colder for each doubling of mass.
            doubling: 2,
            // A three-hundredth of a matter a tick for each degree outside.
            strain: [1, 300],
            // Dry below 40, and a body of store cells copes with 30 less.
            thirst: 40,
            buffer: 30,
            drought: [1, 600],
            // Half of the full credit of 500; a body of fixing cells in the
            // dark pays 250 a 150,000 (a six-hundredth of a matter) a tick,
            // about 1.2 matter through a night.
            shade: 250,
            dim: [1, 150_000],
            turn: None,
        }
    }
}

impl Climate {
    /// # Errors
    /// Fails if a spell, rise, reference or denominator is nought, the band's
    /// width is below nought, or thirst, buffer or shade is out of range.
    pub fn validate(&self) -> Result<()> {
        if self.spell == 0 {
            return Err("a climate's spell must last some ticks".into());
        }
        if self.rise <= 0 {
            return Err("a climate's rise must be some height".into());
        }
        if self.reference == 0 {
            return Err("a climate's reference mass must be some matter".into());
        }
        if self.strain[1] == 0 || self.drought[1] == 0 || self.dim[1] == 0 {
            return Err("a climate's strain, drought and dim need denominators".into());
        }
        if !(0..=100).contains(&self.thirst) || self.buffer < 0 || self.shade < 0 {
            return Err("a climate's thirst must be within 0 to 100, its buffer and shade not below nought".into());
        }
        if self.turn.is_some_and(|bound| bound < 0) {
            return Err("a climate's turn bound cannot be below nought".into());
        }
        if self.width < 0 || self.weather < 0 {
            return Err("a climate's width and weather cannot be below nought".into());
        }
        Ok(())
    }
}
