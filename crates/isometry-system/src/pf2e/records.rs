//! Player Core Remaster records for character creation. Each record names its
//! source; the loader refuses any record that is not from a Remaster book
//! under the ORC licence (case 1016). Summaries are original; the fields are
//! what the draft executes.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const ATTRIBUTES: [&str; 6] = ["str", "dex", "con", "int", "wis", "cha"];

/// The sixteen skills and the attribute each one keys off.
pub const SKILLS: [(&str, &str); 16] = [
    ("acrobatics", "dex"),
    ("arcana", "int"),
    ("athletics", "str"),
    ("crafting", "int"),
    ("deception", "cha"),
    ("diplomacy", "cha"),
    ("intimidation", "cha"),
    ("medicine", "wis"),
    ("nature", "wis"),
    ("occultism", "int"),
    ("performance", "cha"),
    ("religion", "wis"),
    ("society", "int"),
    ("stealth", "dex"),
    ("survival", "wis"),
    ("thievery", "dex"),
];

pub fn attribute_name(key: &str) -> &'static str {
    match key {
        "str" => "Strength",
        "dex" => "Dexterity",
        "con" => "Constitution",
        "int" => "Intelligence",
        "wis" => "Wisdom",
        "cha" => "Charisma",
        _ => "Unknown attribute",
    }
}

pub fn skill_attribute(skill: &str) -> Option<&'static str> {
    SKILLS.iter().find(|(s, _)| *s == skill).map(|(_, a)| *a)
}

/// Upper-cases the first letter of a key for display.
pub fn title(key: &str) -> String {
    let mut chars = key.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub book: String,
    pub license: String,
    pub remaster: bool,
}

/// What a record or a choice adds to the build.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Grant {
    /// Trained in a skill; already trained, a free skill pick instead.
    Train(String),
    /// Trained in `count` skills, from `from` (any untrained skill if empty).
    TrainChoice {
        count: u8,
        #[serde(default)]
        from: Vec<String>,
    },
    /// Trained in a skill; already trained, the feat with that choice instead.
    TrainElseFeat {
        skill: String,
        feat: String,
        choice: String,
    },
    Lore(String),
    Feat(String),
    /// A feat of a category, chosen.
    FeatChoice(String),
    /// A named option, chosen from a list and recorded.
    Choice {
        key: String,
        options: Vec<String>,
    },
    /// One of `options` (a save or perception) raised to `rank`.
    RankChoice {
        options: Vec<String>,
        rank: u8,
    },
    /// A proficiency (perception, a save, an attack or defense category)
    /// raised to at least `rank`.
    Rank {
        target: String,
        rank: u8,
    },
    /// One of several named options, each with its own grants.
    Pick(BTreeMap<String, Vec<Grant>>),
    /// A feat whose own choice is already made (Assurance with Athletics).
    FeatWith {
        feat: String,
        value: String,
    },
    /// A skill the build is trained in, recorded for the granting feat.
    TrainedSkillChoice,
    /// The next armour category the build is untrained in.
    ArmorStep,
    /// Another ancestry from these records, chosen.
    OtherAncestry,
    Speed(i64),
    HitPointsPerLevel(i64),
    DyingMax(i64),
    RecoveryDc(i64),
    Initiative(i64),
    /// Resistance equal to half the level, minimum 1.
    Resistance(String),
    /// Learn this many more languages.
    Languages(u8),
}

/// A weapon, armour or shield. Fields the kind does not use are absent.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub source: Source,
    /// weapon, armor or shield.
    pub kind: String,
    /// In copper pieces.
    pub price: i64,
    pub bulk: String,
    pub rarity: String,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub damage: Option<String>,
    #[serde(default)]
    pub damage_type: Option<String>,
    #[serde(default)]
    pub hands: Option<u8>,
    #[serde(default)]
    pub range: Option<i64>,
    #[serde(default)]
    pub ac: Option<i64>,
    #[serde(default)]
    pub dex_cap: Option<i64>,
    #[serde(default)]
    pub check: Option<i64>,
    #[serde(default)]
    pub speed: Option<i64>,
    #[serde(default)]
    pub strength: Option<i64>,
    #[serde(default)]
    pub hardness: Option<i64>,
    #[serde(default)]
    pub hp: Option<i64>,
    pub summary: String,
}

impl Item {
    pub fn has(&self, trait_: &str) -> bool {
        self.traits.iter().any(|t| t == trait_)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ancestry {
    pub id: String,
    pub name: String,
    pub source: Source,
    pub hp: i64,
    pub size: String,
    pub speed: i64,
    pub vision: String,
    /// One entry per boost; an empty entry is a free boost.
    pub boosts: Vec<Vec<String>>,
    pub flaws: Vec<String>,
    pub languages: Vec<String>,
    /// Languages offered for extra picks; empty offers the common list.
    #[serde(default)]
    pub additional_languages: Vec<String>,
    /// Extra language picks before Intelligence.
    #[serde(default)]
    pub extra_languages: i64,
    /// Items every member starts with (a dwarf's clan dagger).
    #[serde(default)]
    pub items: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heritage {
    pub id: String,
    pub name: String,
    pub source: Source,
    pub ancestry: String,
    #[serde(default)]
    pub grants: Vec<Grant>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Background {
    pub id: String,
    pub name: String,
    pub source: Source,
    /// The pair one boost must come from; the other boost is free.
    pub boosts: Vec<String>,
    #[serde(default)]
    pub skill: Option<String>,
    /// The Lore it trains; with several, one is chosen.
    pub lore: Vec<String>,
    #[serde(default)]
    pub feat: Option<String>,
    #[serde(default)]
    pub grants: Vec<Grant>,
    /// Out of this slice's reach, with the reason shown to the player.
    #[serde(default)]
    pub unavailable: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub source: Source,
    pub hp: i64,
    pub key: Vec<String>,
    /// Starting proficiency ranks: perception, saves, class DC, attacks,
    /// defenses (0 untrained to 4 legendary).
    pub ranks: BTreeMap<String, u8>,
    #[serde(default)]
    pub skills: Vec<String>,
    /// One trained skill chosen from these.
    #[serde(default)]
    pub skill_choice: Vec<String>,
    pub additional_skills: i64,
    /// A group of class features of which one is chosen (the rogue's racket).
    #[serde(default)]
    pub option: Option<String>,
    pub skill_feat_at_first: bool,
    #[serde(default)]
    pub grants: Vec<Grant>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Requirement {
    #[serde(rename = "trained")]
    Trained(String),
    #[serde(rename = "attribute")]
    Attribute { key: String, at_least: i64 },
    #[serde(rename = "feat")]
    Feat(String),
    /// Trained in a named Lore.
    #[serde(rename = "lore")]
    Lore(String),
    #[serde(rename = "any_lore")]
    AnyLore,
    /// Trained in at least one of these skills (any skill if empty).
    #[serde(rename = "any_trained")]
    AnyTrained(Vec<String>),
    #[serde(rename = "any_of")]
    AnyOf(Vec<Requirement>),
    /// Out of this slice's reach, with the reason shown to the player.
    #[serde(rename = "unavailable")]
    Unavailable(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feat {
    pub id: String,
    pub name: String,
    pub source: Source,
    /// ancestry, class, general, skill, or feature (granted, never chosen).
    pub category: String,
    /// The ancestry or class whose feat this is.
    #[serde(default)]
    pub owner: Option<String>,
    /// The class option group this feature belongs to (a racket).
    #[serde(default)]
    pub option: Option<String>,
    /// Extra key attributes this feature allows its class.
    #[serde(default)]
    pub key: Vec<String>,
    /// May be taken more than once (with a different choice each time).
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub requires: Vec<Requirement>,
    #[serde(default)]
    pub grants: Vec<Grant>,
    pub summary: String,
}

impl Feat {
    /// Skill feats are general feats as well.
    pub fn in_category(&self, category: &str) -> bool {
        self.category == category || (category == "general" && self.category == "skill")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u32,
    #[serde(default)]
    ancestry: Vec<Ancestry>,
    #[serde(default)]
    heritage: Vec<Heritage>,
    #[serde(default)]
    background: Vec<Background>,
    #[serde(default)]
    class: Vec<Class>,
    #[serde(default)]
    feat: Vec<Feat>,
    #[serde(default)]
    item: Vec<Item>,
    #[serde(default)]
    common_languages: Vec<String>,
    #[serde(default)]
    step: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default)]
pub struct Records {
    pub ancestries: BTreeMap<String, Ancestry>,
    pub heritages: BTreeMap<String, Heritage>,
    pub backgrounds: BTreeMap<String, Background>,
    pub classes: BTreeMap<String, Class>,
    pub feats: BTreeMap<String, Feat>,
    pub items: BTreeMap<String, Item>,
    /// Languages any character may learn.
    pub common_languages: Vec<String>,
    /// The explanation shown with each kind of step.
    pub steps: BTreeMap<String, String>,
}

const SOURCES: [&str; 8] = [
    include_str!("../../data/pf2e/ancestries.toml"),
    include_str!("../../data/pf2e/backgrounds.toml"),
    include_str!("../../data/pf2e/classes.toml"),
    include_str!("../../data/pf2e/feats.toml"),
    include_str!("../../data/pf2e/general.toml"),
    include_str!("../../data/pf2e/skills.toml"),
    include_str!("../../data/pf2e/equipment.toml"),
    include_str!("../../data/pf2e/steps.toml"),
];

/// The built-in slice, parsed and checked once.
pub fn builtin() -> &'static Records {
    static RECORDS: OnceLock<Records> = OnceLock::new();
    RECORDS.get_or_init(|| Records::parse(&SOURCES).expect("built-in Pathfinder 2e records load"))
}

fn source_ok(name: &str, source: &Source) -> Result<(), String> {
    if source.license != "ORC" || !source.remaster {
        return Err(format!(
            "{name}: only Remaster records under the ORC licence are kept ({}, {}, remaster {})",
            source.book, source.license, source.remaster
        ));
    }
    Ok(())
}

fn insert<T>(map: &mut BTreeMap<String, T>, id: &str, value: T) -> Result<(), String> {
    if map.insert(id.to_owned(), value).is_some() {
        return Err(format!("duplicate record {id}"));
    }
    Ok(())
}

impl Records {
    pub fn parse(sources: &[&str]) -> Result<Self, String> {
        let mut records = Records::default();
        for text in sources {
            let file: File = toml::from_str(text).map_err(|e| format!("pf2e records: {e}"))?;
            if file.version != 1 {
                return Err("unsupported pf2e record file".into());
            }
            for r in file.ancestry {
                source_ok(&r.name, &r.source)?;
                insert(&mut records.ancestries, &r.id.clone(), r)?;
            }
            for r in file.heritage {
                source_ok(&r.name, &r.source)?;
                insert(&mut records.heritages, &r.id.clone(), r)?;
            }
            for r in file.background {
                source_ok(&r.name, &r.source)?;
                insert(&mut records.backgrounds, &r.id.clone(), r)?;
            }
            for r in file.class {
                source_ok(&r.name, &r.source)?;
                insert(&mut records.classes, &r.id.clone(), r)?;
            }
            for r in file.feat {
                source_ok(&r.name, &r.source)?;
                insert(&mut records.feats, &r.id.clone(), r)?;
            }
            for r in file.item {
                source_ok(&r.name, &r.source)?;
                insert(&mut records.items, &r.id.clone(), r)?;
            }
            records.common_languages.extend(file.common_languages);
            records.steps.extend(file.step);
        }
        records.check()?;
        Ok(records)
    }

    /// Every cross-reference resolves, so a draft never meets a dangling id.
    fn check(&self) -> Result<(), String> {
        let attribute = |a: &String| {
            ATTRIBUTES
                .contains(&a.as_str())
                .then_some(())
                .ok_or_else(|| format!("unknown attribute {a}"))
        };
        let skill = |s: &String| {
            skill_attribute(s)
                .map(|_| ())
                .ok_or_else(|| format!("unknown skill {s}"))
        };
        let feat = |f: &String| {
            self.feats
                .contains_key(f)
                .then_some(())
                .ok_or_else(|| format!("unknown feat {f}"))
        };
        fn grants(list: &[Grant], skill: &dyn Fn(&String) -> Result<(), String>,
            feat: &dyn Fn(&String) -> Result<(), String>) -> Result<(), String> {
            for g in list {
                match g {
                    Grant::Pick(options) => {
                        for inner in options.values() {
                            grants(inner, skill, feat)?;
                        }
                    },
                    Grant::FeatWith { feat: f, .. } => feat(f)?,
                    Grant::Train(s) => skill(s)?,
                    Grant::TrainChoice { from, .. } => from.iter().try_for_each(skill)?,
                    Grant::TrainElseFeat { skill: s, feat: f, .. } => {
                        skill(s)?;
                        feat(f)?
                    },
                    Grant::Feat(f) => feat(f)?,
                    _ => {},
                }
            }
            Ok(())
        }
        let grants = |list: &[Grant]| grants(list, &skill, &feat);
        for a in self.ancestries.values() {
            a.boosts.iter().flatten().chain(&a.flaws).try_for_each(attribute)?;
            for i in &a.items {
                if !self.items.contains_key(i) {
                    return Err(format!("{}: unknown item {i}", a.name));
                }
            }
        }
        for h in self.heritages.values() {
            if !self.ancestries.contains_key(&h.ancestry) {
                return Err(format!("{}: unknown ancestry", h.name));
            }
            grants(&h.grants)?;
        }
        for b in self.backgrounds.values() {
            b.boosts.iter().try_for_each(attribute)?;
            b.skill.iter().try_for_each(skill)?;
            b.feat.iter().try_for_each(feat)?;
            grants(&b.grants)?;
        }
        for c in self.classes.values() {
            c.key.iter().try_for_each(attribute)?;
            c.skills.iter().chain(&c.skill_choice).try_for_each(skill)?;
            grants(&c.grants)?;
        }
        for f in self.feats.values() {
            f.key.iter().try_for_each(attribute)?;
            grants(&f.grants)?;
            let mut pending: Vec<&Requirement> = f.requires.iter().collect();
            while let Some(r) = pending.pop() {
                match r {
                    Requirement::Trained(s) => skill(s)?,
                    Requirement::AnyTrained(list) => list.iter().try_for_each(skill)?,
                    Requirement::Feat(x) => feat(x)?,
                    Requirement::Attribute { key, .. } => attribute(key)?,
                    Requirement::AnyOf(list) => pending.extend(list),
                    Requirement::Lore(_) | Requirement::AnyLore | Requirement::Unavailable(_) => {},
                }
            }
        }
        Ok(())
    }

    pub fn step(&self, kind: &str) -> &str {
        self.steps.get(kind).map_or("", String::as_str)
    }
}
