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
}

impl Default for Climate {
    /// A first cut, to be tuned.
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
            // A tenth of a matter a tick for each degree outside.
            strain: [1, 10],
        }
    }
}

impl Climate {
    /// # Errors
    /// Fails if a spell, rise, reference or denominator is nought, or the
    /// band's width is below nought.
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
        if self.strain[1] == 0 {
            return Err("a climate's strain needs a denominator".into());
        }
        if self.width < 0 || self.weather < 0 {
            return Err("a climate's width and weather cannot be below nought".into());
        }
        Ok(())
    }
}
