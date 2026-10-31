//! Ordinary play numbers for a finished build: Armor Class, Strikes with the
//! multiple attack penalty, skill checks, degrees of success and the dying
//! track. Pure functions over the build and its equipped items; the table
//! owns the dice and the state they change.

use super::derive::proficiency;
use super::draft::Build;
use super::records::{self, Item, Source};
use serde::{Deserialize, Serialize};

pub const LEVEL: i64 = 1;

/// Encounter mode (case 1026): turns in initiative order, each with three
/// actions, a reaction and an attack count for the multiple attack penalty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    pub round: u32,
    /// Index into `order` of the creature whose turn it is.
    pub current: usize,
    pub order: Vec<Turn>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Turn {
    /// The native body's table id.
    pub token: u32,
    pub initiative: i64,
    pub actions: u8,
    pub reaction: bool,
    pub attacks: u8,
    pub shield_raised: bool,
}

impl Encounter {
    pub fn turn(&self, token: u32) -> Option<&Turn> {
        self.order.iter().find(|t| t.token == token)
    }
    pub fn current(&self) -> Option<&Turn> {
        self.order.get(self.current)
    }
}

/// The four degrees of success, worst first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Degree {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}

impl Degree {
    pub fn name(self) -> &'static str {
        match self {
            Degree::CriticalFailure => "critical failure",
            Degree::Failure => "failure",
            Degree::Success => "success",
            Degree::CriticalSuccess => "critical success",
        }
    }
}

/// Beat the DC by 10 for a critical success, miss by 10 for a critical
/// failure; a natural 20 or 1 then moves the result one step.
pub fn degree(total: i64, dc: i64, die: u16) -> Degree {
    let mut step: i8 = if total >= dc + 10 { 3 } else if total >= dc { 2 } else if total <= dc - 10 { 0 } else { 1 };
    if die == 20 { step += 1 }
    if die == 1 { step -= 1 }
    match step.clamp(0, 3) {
        0 => Degree::CriticalFailure,
        1 => Degree::Failure,
        2 => Degree::Success,
        _ => Degree::CriticalSuccess,
    }
}

/// What a character has equipped.
#[derive(Clone, Copy, Debug, Default)]
pub struct Gear<'a> {
    pub weapon: Option<&'a Item>,
    pub armor: Option<&'a Item>,
    pub shield: Option<&'a Item>,
}

fn rank(b: &Build, key: &str) -> u8 {
    b.ranks.get(key).copied().unwrap_or(0)
}

/// A fist: every character's unarmed attack.
pub fn fist() -> Item {
    Item {
        id: "fist".into(),
        name: "Fist".into(),
        source: Source { book: "Pathfinder Player Core".into(), license: "ORC".into(), remaster: true },
        kind: "weapon".into(),
        price: 0,
        bulk: "-".into(),
        rarity: "common".into(),
        traits: ["agile", "finesse", "nonlethal", "unarmed"].map(String::from).to_vec(),
        category: Some("unarmed".into()),
        group: Some("brawling".into()),
        damage: Some("1d4".into()),
        damage_type: Some("bludgeoning".into()),
        hands: Some(1),
        range: None,
        ac: None, dex_cap: None, check: None, speed: None, strength: None, hardness: None, hp: None,
        summary: "Unarmed attack: 1d4 bludgeoning; agile, finesse, nonlethal.".into(),
    }
}

/// Armor Class: 10, Dexterity up to the armour's cap, the armour's
/// proficiency and bonus, and a raised shield's circumstance bonus.
pub fn armor_class(b: &Build, gear: &Gear, shield_raised: bool) -> i64 {
    let (category, bonus, cap) = gear.armor.map_or(("unarmored", 0, i64::MAX), |a| {
        (a.category.as_deref().unwrap_or("unarmored"), a.ac.unwrap_or(0), a.dex_cap.unwrap_or(i64::MAX))
    });
    let shield = if shield_raised { gear.shield.and_then(|s| s.ac).unwrap_or(0) } else { 0 };
    10 + b.modifier("dex").min(cap) + proficiency(rank(b, category), LEVEL) + bonus + shield
}

pub fn perception(b: &Build) -> i64 {
    b.modifier("wis") + proficiency(rank(b, "perception"), LEVEL)
}

/// A skill check modifier; armour's check penalty applies to Strength and
/// Dexterity skills unless the wearer meets its Strength.
pub fn skill(b: &Build, gear: &Gear, skill: &str) -> i64 {
    let attribute = records::skill_attribute(skill).unwrap_or("int");
    let penalty = gear.armor.filter(|a| b.modifier("str") < a.strength.unwrap_or(0))
        .and_then(|a| a.check)
        .filter(|_| matches!(attribute, "str" | "dex"))
        .unwrap_or(0);
    b.modifier(attribute) + proficiency(b.skills.get(skill).copied().unwrap_or(0), LEVEL) + penalty
}

/// The proficiency rank a weapon uses, with Dwarven Weapon Familiarity
/// treating dwarf weapons (and the battle axe, pick and warhammer) as one
/// category simpler.
fn weapon_rank(b: &Build, w: &Item) -> u8 {
    let mut category = w.category.as_deref().unwrap_or("simple");
    let familiar = b.feats.iter().any(|f| f == "dwarven-weapon-familiarity")
        && (w.has("dwarf") || ["battle-axe", "pick", "warhammer"].contains(&w.id.as_str()));
    if familiar {
        category = match category { "advanced" => "martial", "martial" => "simple", c => c };
    }
    rank(b, category)
}

/// The multiple attack penalty for the next attack this turn.
pub fn multiple_attack_penalty(w: &Item, attacks: u8) -> i64 {
    let step = if w.has("agile") { 4 } else { 5 };
    -step * i64::from(attacks.min(2))
}

fn melee(w: &Item) -> bool {
    w.range.is_none()
}

/// The Strike's attack modifier before the multiple attack penalty.
pub fn attack(b: &Build, w: &Item) -> i64 {
    let attribute = if !melee(w) {
        b.modifier("dex")
    } else if w.has("finesse") {
        b.modifier("str").max(b.modifier("dex"))
    } else {
        b.modifier("str")
    };
    attribute + proficiency(weapon_rank(b, w), LEVEL)
}

/// Damage dice and flat modifier; a critical hit doubles the total, and
/// deadly or fatal add their die on top.
pub fn damage(b: &Build, w: &Item, critical: bool) -> (String, i64, Option<String>) {
    let mut dice = w.damage.clone().unwrap_or_else(|| "1".into());
    let flat = if melee(w) || w.traits.iter().any(|t| t.starts_with("thrown")) { b.modifier("str") } else { 0 };
    let mut extra = None;
    if critical {
        for t in &w.traits {
            if let Some(die) = t.strip_prefix("deadly-") {
                extra = Some(format!("1{die}"));
            }
            if let Some(die) = t.strip_prefix("fatal-") {
                dice = dice.split('d').next().map_or(dice.clone(), |n| format!("{n}{die}"));
                extra = Some(format!("1{die}"));
            }
        }
    }
    (dice, flat, extra)
}

/// The dying track after a recovery check (DC 10 plus dying, lowered by
/// Toughness): critical success -2, success -1, failure +1, critical
/// failure +2.
pub fn recovery(dying: i64, result: Degree) -> i64 {
    let change = match result {
        Degree::CriticalSuccess => -2,
        Degree::Success => -1,
        Degree::Failure => 1,
        Degree::CriticalFailure => 2,
    };
    (dying + change).max(0)
}

/// Dying gained on dropping to 0 Hit Points, or on taking damage while
/// dying: 1, or 2 from a critical hit, plus the wounded value when first
/// knocked out.
pub fn dying_gain(critical: bool, wounded: i64, already_dying: bool) -> i64 {
    let base = if critical { 2 } else { 1 };
    if already_dying { base } else { base + wounded }
}
