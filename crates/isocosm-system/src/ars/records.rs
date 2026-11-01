//! Ars Magica's records, read from the bundled `ars5` edition pack (cases
//! 1063, 1064): creation records, learnable spells and the play numbers the
//! compiled rules read.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The pack these records come from.
pub const PACK: &str = "ars5";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub characteristic_points: i64,
    pub characteristic_costs: BTreeMap<String, i64>,
    pub native_language_xp: u32,
    pub childhood_xp: u32,
    pub childhood_years: u32,
    pub later_life_xp_per_year: u32,
    pub apprenticeship_years: u32,
    pub apprenticeship_xp: u32,
    pub apprenticeship_spell_levels: u32,
    pub virtue_points_max: u32,
    pub grog_minor_max: u32,
    pub magus_minimums: BTreeMap<String, u32>,
    pub ability_caps: Vec<[u32; 2]>,
    pub magus_ages: Vec<u32>,
    pub grog_ages: Vec<u32>,
    pub affinity_cap_bonus: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Named {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Art {
    pub id: String,
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct House {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub benefit: String,
    pub targets: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    pub xp: u32,
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default)]
    pub abilities: Vec<String>,
}

/// A Virtue or a Flaw.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    pub id: String,
    pub name: String,
    /// `free`, `minor`, `major`, or `either` for a Flaw taken at either weight.
    pub weight: String,
    pub kind: String,
    pub summary: String,
    #[serde(rename = "for")]
    pub types: Vec<String>,
    #[serde(default)]
    pub automatic: bool,
    #[serde(default)]
    pub opens: Option<String>,
    #[serde(default)]
    pub pool: Option<Pool>,
    /// `ability`, `art` or `form`.
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub max: Option<usize>,
    /// Offered but refused, with this reason.
    #[serde(default)]
    pub refused: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub childhood: bool,
    #[serde(default)]
    pub gift: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spell {
    pub id: String,
    pub name: String,
    pub technique: String,
    pub form: String,
    #[serde(default)]
    pub requisites: Vec<String>,
    pub level: u32,
    pub range: String,
    pub duration: String,
    pub target: String,
    #[serde(default)]
    pub ritual: bool,
    pub summary: String,
    /// The effect record this spell casts through, when one is built.
    #[serde(default)]
    pub procedure: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Time {
    pub seasons_per_year: u64,
    pub season_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Study {
    pub max_sources_per_season: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wounds {
    pub first_band: i64,
    pub order: Vec<String>,
    pub penalties: BTreeMap<String, i64>,
    pub typical_part_share_percent: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fatigue {
    pub levels: Vec<String>,
    pub penalties: Vec<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreationFile {
    version: u32,
    rules: Rules,
    characteristic: Vec<Named>,
    art: Vec<Art>,
    house: Vec<House>,
    virtue: Vec<Trait>,
    flaw: Vec<Trait>,
    ability: Vec<Ability>,
    step: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellFile {
    version: u32,
    spell: Vec<Spell>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayFile {
    version: u32,
    time: Time,
    study: Study,
    wounds: Wounds,
    fatigue: Fatigue,
}

#[derive(Clone, Debug)]
pub struct Records {
    pub rules: Rules,
    pub characteristics: Vec<Named>,
    pub arts: Vec<Art>,
    pub houses: Vec<House>,
    pub virtues: Vec<Trait>,
    pub flaws: Vec<Trait>,
    pub abilities: Vec<Ability>,
    pub spells: Vec<Spell>,
    pub steps: BTreeMap<String, String>,
    pub time: Time,
    pub study: Study,
    pub wounds: Wounds,
    pub fatigue: Fatigue,
}

fn one<'a>(texts: &[&'a str], kind: &str) -> Result<&'a str, String> {
    match texts {
        [text] => Ok(text),
        _ => Err(format!("the Ars pack must carry exactly one {kind} file")),
    }
}

impl Records {
    pub fn parse(creation: &str, spells: &str, play: &str) -> Result<Self, String> {
        let c: CreationFile = toml::from_str(creation).map_err(|e| format!("Ars creation records: {e}"))?;
        let s: SpellFile = toml::from_str(spells).map_err(|e| format!("Ars spells: {e}"))?;
        let p: PlayFile = toml::from_str(play).map_err(|e| format!("Ars play numbers: {e}"))?;
        if c.version != 1 || s.version != 1 || p.version != 1 {
            return Err("unsupported Ars record version".into());
        }
        let records = Self {
            rules: c.rules, characteristics: c.characteristic, arts: c.art, houses: c.house,
            virtues: c.virtue, flaws: c.flaw, abilities: c.ability, spells: s.spell, steps: c.step,
            time: p.time, study: p.study, wounds: p.wounds, fatigue: p.fatigue,
        };
        records.check()?;
        Ok(records)
    }

    fn check(&self) -> Result<(), String> {
        let mut ids = std::collections::BTreeSet::new();
        let all = self.characteristics.iter().map(|c| &c.id)
            .chain(self.arts.iter().map(|a| &a.id))
            .chain(self.abilities.iter().map(|a| &a.id));
        for id in all {
            if !ids.insert(id) {
                return Err(format!("Ars record id {id} is used twice"));
            }
        }
        for spell in &self.spells {
            for art in std::iter::once(&spell.technique).chain([&spell.form]).chain(&spell.requisites) {
                self.art(art).ok_or_else(|| format!("spell {} names unknown Art {art}", spell.id))?;
            }
        }
        for house in &self.houses {
            self.trait_(&house.benefit).ok_or_else(|| format!("House {} grants unknown {}", house.id, house.benefit))?;
        }
        if self.time.seasons_per_year == 0 || self.wounds.first_band <= 0 {
            return Err("Ars play numbers out of range".into());
        }
        Ok(())
    }

    pub fn art(&self, id: &str) -> Option<&Art> {
        self.arts.iter().find(|a| a.id == id)
    }
    pub fn ability(&self, id: &str) -> Option<&Ability> {
        self.abilities.iter().find(|a| a.id == id)
    }
    pub fn trait_(&self, id: &str) -> Option<&Trait> {
        self.virtues.iter().chain(&self.flaws).find(|t| t.id == id)
    }
    pub fn spell(&self, id: &str) -> Option<&Spell> {
        self.spells.iter().find(|s| s.id == id)
    }
    pub fn house(&self, id: &str) -> Option<&House> {
        self.houses.iter().find(|h| h.id == id)
    }
    /// A target's display name: an Ability or an Art.
    pub fn name(&self, id: &str) -> String {
        self.ability(id).map(|a| a.name.clone())
            .or_else(|| self.art(id).map(|a| a.name.clone()))
            .or_else(|| self.characteristics.iter().find(|c| c.id == id).map(|c| c.name.clone()))
            .unwrap_or_else(|| id.to_owned())
    }
    pub fn step(&self, kind: &str) -> &str {
        self.steps.get(kind).map_or("", String::as_str)
    }
}

/// The bundled records, parsed and checked once.
pub fn builtin() -> &'static Records {
    static RECORDS: OnceLock<Records> = OnceLock::new();
    RECORDS.get_or_init(|| {
        let read = |kind: &str| isocosm_campaign::bundled_records(PACK, kind);
        (|| {
            let (creation, spells, play) = (read("creation")?, read("spells")?, read("play")?);
            Records::parse(one(&creation, "creation")?, one(&spells, "spells")?, one(&play, "play")?)
        })()
        .expect("bundled Ars Magica records load")
    })
}
