// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Disposable effect/material experiment. This does not modify a World, assign
//! capabilities to organs, or implement a tabletop ruleset. Coordinates are a
//! normalized integer demonstration plane; they are not collision results.

use crate::rng::Rng;
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;
pub const RECEIVER_CENTER: [i32; 2] = [700, 500];
pub const RECEIVER_RADIUS: i32 = 100;

macro_rules! choices {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub fn label(self) -> &'static str {
                match self { $(Self::$variant => $label),+ }
            }
        }
    };
}

choices!(Glyph { Quotes => "quotes", Slashes => "slashes", Backticks => "backticks" });
choices!(Behavior { Stream => "stream", Enclose => "enclose", Inscribe => "inscribe" });
choices!(Receiver { Stone => "stone", Metal => "metal", Moss => "moss" });
choices!(Profile { Guaranteed => "guaranteed", Generated => "generated" });
choices!(Response { Reflect => "reflect", Absorb => "absorb", Bind => "bind", Split => "split" });
choices!(RuleOrigin { Guaranteed => "guaranteed", Generated => "generated" });
choices!(Connection { Separate => "separate", Pairs => "pairs", String => "string" });

impl Glyph {
    pub fn text(self) -> &'static str {
        match self {
            Self::Quotes => "\"",
            Self::Slashes => "/",
            Self::Backticks => "`",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub world_seed: u64,
    pub appearance_seed: u64,
    pub glyph: Glyph,
    pub behavior: Behavior,
    pub receiver: Receiver,
    pub profile: Profile,
    pub count: u16,
    pub duration_ticks: u16,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            version: VERSION,
            world_seed: 7,
            appearance_seed: 42,
            glyph: Glyph::Quotes,
            behavior: Behavior::Enclose,
            receiver: Receiver::Stone,
            profile: Profile::Guaranteed,
            count: 24,
            duration_ticks: 120,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub response: Response,
    pub origin: RuleOrigin,
    pub connection: Connection,
    pub explanation: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Experiment {
    pub request: Request,
    pub report: Report,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub id: u16,
    pub glyph: char,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub angle: i32,
    pub scale: u16,
    pub opacity: u8,
    pub group: Option<u16>,
}

impl Mark {
    pub fn text(&self) -> &'static str {
        match self.glyph {
            '"' => "\"",
            '/' => "/",
            _ => "`",
        }
    }
}

impl Request {
    pub fn prepare(&self) -> Result<Experiment, String> {
        if self.version != VERSION {
            return Err(format!(
                "unsupported effect experiment version {}",
                self.version
            ));
        }
        if !(2..=128).contains(&self.count) {
            return Err("mark count must be 2..128".into());
        }
        if !(10..=3600).contains(&self.duration_ticks) {
            return Err("duration must be 10..3600 ticks".into());
        }
        // Domain-separated lookup, independent of appearance, mark count and
        // sampling order. These are authored demonstration laws, not 5e/PF2e.
        let (response, origin) = match self.profile {
            Profile::Guaranteed => (
                match self.receiver {
                    Receiver::Stone => Response::Reflect,
                    Receiver::Metal => Response::Split,
                    Receiver::Moss => Response::Bind,
                },
                RuleOrigin::Guaranteed,
            ),
            Profile::Generated => {
                let key = match self.receiver {
                    Receiver::Stone => 0x5354_4f4e,
                    Receiver::Metal => 0x4d45_544c,
                    Receiver::Moss => 0x4d4f_5353,
                };
                let mut rng = Rng::from_seed(self.world_seed ^ key ^ 0x4546_4645_4354_0001);
                (
                    Response::ALL[rng.below(Response::ALL.len() as u64) as usize],
                    RuleOrigin::Generated,
                )
            },
        };
        let connection = match (self.glyph, self.behavior, response) {
            (Glyph::Quotes, Behavior::Enclose, Response::Bind | Response::Reflect) => {
                Connection::Pairs
            },
            (Glyph::Backticks, Behavior::Inscribe, Response::Bind) => Connection::String,
            _ => Connection::Separate,
        };
        let explanation = format!(
            "{} material rule: {} → {}. {} with {} produces {}. Demonstration only; no world changes.",
            origin.label(),
            self.receiver.label(),
            response.label(),
            self.glyph.label(),
            self.behavior.label(),
            connection.label()
        );
        Ok(Experiment {
            request: self.clone(),
            report: Report {
                response,
                origin,
                connection,
                explanation,
            },
        })
    }
}

impl Experiment {
    /// Stateless sample at an absolute age. At or after the duration, no marks
    /// remain. Hosts choose playback clocks; sampling never advances an RNG.
    pub fn sample(&self, tick: u16) -> Vec<Mark> {
        let r = &self.request;
        if tick >= r.duration_ticks {
            return Vec::new();
        }
        let age = i32::from(tick) * 1000 / i32::from(r.duration_ticks);
        let mut marks = Vec::with_capacity(usize::from(r.count) * 2);
        for id in 0..r.count {
            // A stream emits a sequence over the first 18% of its lifetime.
            // Each mark reaches and reacts to the receiver at its own age.
            let mark_age = if r.behavior == Behavior::Stream {
                let delay = i32::from(id) * 180 / i32::from(r.count);
                if age < delay {
                    continue;
                }
                age - delay
            } else {
                age
            };
            let mut rng = Rng::from_seed(r.appearance_seed ^ (u64::from(id) << 32) ^ 0x4d41_524b);
            let jitter = rng.range_i32(-70, 70);
            let (mut x, mut y) = match r.behavior {
                Behavior::Stream => (100 + mark_age * 8 / 10, 500 + jitter),
                Behavior::Enclose => {
                    let phase = (i32::from(id) * 4000 / i32::from(r.count) + age * 2) % 4000;
                    let radius = 170 - age * 60 / 1000;
                    let p = phase % 1000;
                    let (dx, dy) = match phase / 1000 {
                        0 => (1000 - p, p),
                        1 => (-p, 1000 - p),
                        2 => (-1000 + p, -p),
                        _ => (p, -1000 + p),
                    };
                    (700 + dx * radius / 1000, 500 + dy * radius / 1000)
                },
                Behavior::Inscribe => (
                    620 + i32::from(id) * 160 / i32::from(r.count - 1),
                    500 + jitter * (1000 - age) / 1000,
                ),
            };
            let contact = match r.behavior {
                Behavior::Stream => 625,
                _ => 500,
            };
            let after = (mark_age - contact).max(0);
            let mut opacity = 255;
            if after > 0 {
                match self.report.response {
                    Response::Reflect => match r.behavior {
                        Behavior::Stream => {
                            x = 600 - after * 8 / 10;
                            y += after / 4;
                        },
                        _ => {
                            x = 700 + (x - 700) * (1000 + after * 2) / 1000;
                            y = 500 + (y - 500) * (1000 + after * 2) / 1000;
                        },
                    },
                    Response::Absorb => {
                        let remaining = (1000 - after * 1000 / (1000 - contact)).max(0);
                        x = 700 + (x - 700) * remaining / 1000;
                        y = 500 + (y - 500) * remaining / 1000;
                        opacity = (remaining * 255 / 1000) as u8;
                    },
                    Response::Bind => {
                        x = x.clamp(620, 780);
                        y = y.clamp(420, 580);
                    },
                    Response::Split => {
                        y -= after / 3;
                    },
                }
            }
            let mark = Mark {
                id,
                glyph: r.glyph.text().chars().next().unwrap(),
                x,
                y,
                z: 0,
                angle: if r.behavior == Behavior::Enclose {
                    (i32::from(id) * 360 / i32::from(r.count) + age / 3) % 360
                } else {
                    jitter / 4
                },
                scale: 800 + rng.below(401) as u16,
                opacity,
                group: match self.report.connection {
                    Connection::Separate => None,
                    Connection::Pairs => Some(id / 2),
                    Connection::String => Some(0),
                },
            };
            if self.report.response == Response::Split && after > 0 {
                let mut other = mark.clone();
                other.id += r.count;
                other.y += after * 2 / 3;
                // Split copies no longer claim the original connection.
                other.group = None;
                marks.push(other);
            }
            marks.push(mark);
        }
        marks
    }
}

#[cfg(test)]
mod tests;
