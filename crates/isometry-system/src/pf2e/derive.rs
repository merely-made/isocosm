//! Numbers derived from a build (VTT 5's derived display): never stored, so
//! they cannot drift from the choices that produce them.

use super::draft::Build;
use super::records::{self, Records};

/// A proficiency bonus: untrained adds nothing, otherwise twice the rank plus
/// the level.
pub fn proficiency(rank: u8, level: i64) -> i64 {
    if rank == 0 { 0 } else { 2 * i64::from(rank) + level }
}

pub fn rank_name(rank: u8) -> &'static str {
    ["untrained", "trained", "expert", "master", "legendary"]
        .get(usize::from(rank))
        .copied()
        .unwrap_or("legendary")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Derived {
    pub level: i64,
    pub hp_max: i64,
    pub dying_max: i64,
    pub recovery_dc: i64,
    pub lines: Vec<Line>,
}

pub fn derive(r: &Records, b: &Build) -> Derived {
    let level = 1;
    let rank = |k: &str| b.ranks.get(k).copied().unwrap_or(0);
    let m = |a: &str| b.modifier(a);
    let mut lines = Vec::new();
    let mut line = |label: &str, value: String| lines.push(Line { label: label.into(), value });
    let ancestry = b.ancestry.as_ref().map(|a| &r.ancestries[a]);
    let class = b.class.as_ref().map(|c| &r.classes[c]);
    let hp_max = ancestry.map_or(0, |a| a.hp)
        + class.map_or(0, |c| c.hp + m("con") + b.hp_per_level) * level;
    if let Some(a) = ancestry {
        let heritage = b.heritage.as_ref().map_or(String::new(), |h| format!(" ({})", r.heritages[h].name));
        line("Ancestry", format!("{}{heritage}", a.name));
    }
    if let Some(background) = &b.background {
        line("Background", r.backgrounds[background].name.clone());
    }
    if let Some(c) = class {
        let racket = b.racket.as_ref().map_or(String::new(), |x| format!(" ({})", r.feats[x].name));
        line("Class", format!("{}{racket}", c.name));
    }
    let attributes = records::ATTRIBUTES
        .iter()
        .map(|a| format!("{} {:+}", &records::attribute_name(a)[..3], m(a)))
        .collect::<Vec<_>>()
        .join(", ");
    line("Attributes", attributes);
    if let Some(key) = &b.key {
        line("Key attribute", records::attribute_name(key).into());
    }
    line("Hit Points", hp_max.to_string());
    line("Armor Class (unarmored)", (10 + m("dex") + proficiency(rank("unarmored"), level)).to_string());
    let checks = [("Perception", "perception", "wis"), ("Fortitude", "fortitude", "con"),
        ("Reflex", "reflex", "dex"), ("Will", "will", "wis")];
    for (label, key, attribute) in checks {
        line(label, format!("{:+} ({})", m(attribute) + proficiency(rank(key), level), rank_name(rank(key))));
    }
    if b.initiative != 0 {
        line("Initiative bonus", format!("{:+}", b.initiative));
    }
    if let Some(key) = &b.key {
        line("Class DC", (10 + m(key) + proficiency(rank("class_dc"), level)).to_string());
    }
    if let Some(a) = ancestry {
        line("Size, Speed, senses", format!("{}, {} feet, {}", a.size, a.speed + b.speed_bonus, a.vision));
        line("Languages", a.languages.join(", "));
    }
    let skills = records::SKILLS
        .iter()
        .filter(|(s, _)| b.trained(s))
        .map(|(s, a)| format!("{} {:+}", records::title(s), m(a) + proficiency(b.skills[*s], level)))
        .collect::<Vec<_>>()
        .join(", ");
    line("Trained skills", skills);
    if !b.lores.is_empty() {
        let lore = m("int") + proficiency(1, level);
        line("Lores", b.lores.iter().map(|l| format!("{l} {lore:+}")).collect::<Vec<_>>().join(", "));
    }
    let attacks = ["unarmed", "simple", "martial", "advanced"].map(|k| format!("{k} {}", rank_name(rank(k)))).join(", ");
    line("Attacks", attacks);
    let defenses = ["unarmored", "light", "medium", "heavy"].map(|k| format!("{k} {}", rank_name(rank(k)))).join(", ");
    line("Defenses", defenses);
    let feats = b.feats.iter().map(|f| r.feats[f].name.clone()).collect::<Vec<_>>().join(", ");
    line("Feats and features", feats);
    for (feat, value) in &b.options {
        line(feat, records::title(value));
    }
    if !b.resistances.is_empty() {
        let amount = (level / 2).max(1);
        line("Resistances", b.resistances.iter().map(|k| format!("{k} {amount}")).collect::<Vec<_>>().join(", "));
    }
    let dying_max = b.dying_max.unwrap_or(4);
    let recovery_dc = 10 + b.recovery_dc;
    Derived { level, hp_max, dying_max, recovery_dc, lines }
}
