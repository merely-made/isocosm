use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edition {
    Dnd5,
    Pathfinder2,
    Ars5,
}

impl Edition {
    pub fn system(self) -> &'static str {
        match self {
            Self::Dnd5 => "5e-srd",
            Self::Pathfinder2 => "pf2e-srd",
            Self::Ars5 => "ars5",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Sight,
    Size,
    Weapon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Duration {
    Seconds(u64),
    Sun,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub edition: Edition,
    pub kind: Kind,
    pub level: u8,
    pub range_ft: u32,
    pub personal: bool,
    pub duration: Duration,
    pub concentration: bool,
    pub actions: u8,
    pub bonus_action: bool,
    pub requisite: Option<String>,
    pub form: Option<String>,
    pub source: String,
    pub license: String,
    pub credit: String,
}

impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.level == 0
            || self.source.is_empty()
            || self.license.is_empty()
            || self.credit.is_empty()
            || self.actions > 3
            || matches!(self.duration, Duration::Seconds(0))
            || (self.edition == Edition::Ars5 && self.form.is_none())
        {
            return Err("invalid effect procedure definition".into());
        }
        Ok(())
    }
    pub fn revision(&self) -> String {
        isocosm::digest(self)
    }
}

#[derive(Clone, Debug)]
pub struct Definitions(pub BTreeMap<String, Definition>);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pack {
    version: u32,
    effects: Vec<Definition>,
}

impl Definitions {
    pub fn builtin() -> Result<Self, String> {
        let mut entries = BTreeMap::new();
        for source in [
            include_str!("../../data/effects/dnd5.toml"),
            include_str!("../../data/effects/pf2.toml"),
            include_str!("../../data/effects/ars5.toml"),
        ] {
            let pack: Pack =
                toml::from_str(source).map_err(|e| format!("effect procedures: {e}"))?;
            if pack.version != 1 {
                return Err("unsupported effect procedure pack".into());
            }
            for definition in pack.effects {
                definition.validate()?;
                if entries.insert(definition.id.clone(), definition).is_some() {
                    return Err("duplicate effect procedure".into());
                }
            }
        }
        Ok(Self(entries))
    }
    pub fn get(&self, id: &str) -> Result<&Definition, String> {
        self.0
            .get(id)
            .ok_or_else(|| format!("unknown effect procedure {id}"))
    }
}
