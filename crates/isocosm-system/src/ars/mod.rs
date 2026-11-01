//! Ars Magica Fifth Edition (cases 1063, 1064). The records ship in the
//! bundled `ars5` pack; creation, study and Wounds are compiled here and read
//! their numbers from it. Sheet fields use Ars's own words under `ars:`.

pub mod draft;
pub mod records;
pub mod rules;
pub mod sample;

use isocosm_core::{FieldValue, SheetData};

/// The system key, the same as the effect procedures' edition key.
pub const SYSTEM: &str = "ars5";
pub const LABEL: &str = "Ars Magica Fifth Edition";

/// A field under Ars's prefix.
pub fn field(name: &str) -> String {
    format!("ars:{name}")
}

/// The sheet a finished build records: Characteristics, Art and Ability
/// scores with the experience behind them, Virtue and Flaw effects the
/// casting and study rules read, and the spells known.
pub fn sheet(r: &records::Records, build: &draft::Build, name: &str) -> SheetData {
    let mut sheet = SheetData::new(SYSTEM);
    sheet.set_text("name", name);
    sheet.set_text("body_form", "corpus");
    sheet.set_text(field("type"), build.kind.as_str());
    if let Some(house) = &build.house {
        sheet.set_text(field("house"), house.as_str());
    }
    sheet.set_int(field("age"), i64::from(build.age));
    sheet.set_int(field("size"), 0);
    sheet.set_int(field("fatigue_lost"), 0);
    for (id, value) in &build.characteristics {
        sheet.set_int(field(id), *value);
    }
    for art in &r.arts {
        sheet.set_int(field(&art.id), i64::from(build.art(&art.id)));
    }
    for ability in &r.abilities {
        sheet.set_int(field(&ability.id), i64::from(build.ability(&ability.id)));
    }
    for (id, xp) in &build.xp {
        sheet.set_int(field(&format!("xp:{id}")), i64::from(*xp));
    }
    for taken in &build.traits {
        sheet.fields.insert(field(&format!("trait:{}", taken.id)), FieldValue::Bool(true));
        let Some(target) = &taken.target else { continue };
        match taken.id.as_str() {
            "puissant-art" => sheet.set_int(field(&format!("puissant:{target}")), 3),
            "puissant-ability" => sheet.set_int(field(&format!("puissant:{target}")), 2),
            "affinity-with-art" | "affinity-with-ability" => {
                sheet.fields.insert(field(&format!("affinity:{target}")), FieldValue::Bool(true));
            },
            "deficient-form" => {
                sheet.fields.insert(field(&format!("deficient:{target}")), FieldValue::Bool(true));
            },
            _ => {},
        }
    }
    sheet.set_int(field("penetration"), i64::from(build.ability("penetration")));
    for spell in &build.spells {
        sheet.fields.insert(field(&format!("spell:{spell}")), FieldValue::Bool(true));
        // The F11 procedure reads which definitions a sheet knows.
        if let Some(procedure) = r.spell(spell).and_then(|s| s.procedure.as_deref()) {
            sheet.fields.insert(format!("known:{procedure}"), FieldValue::Bool(true));
        }
    }
    sheet
}

/// Study one season from a summa: the sheet's experience and score rise,
/// capped by the summa's Level. Returns the experience gained.
pub fn study(r: &records::Records, sheet: &mut SheetData, subject: &str, level: u32, quality: u32) -> Result<u32, String> {
    let art = r.art(subject).is_some();
    if !art && r.ability(subject).is_none() {
        return Err(format!("{subject} is not an Art or Ability of this slice"));
    }
    let cost = if art { rules::art_xp as fn(u32) -> u32 } else { rules::ability_xp };
    let key = field(&format!("xp:{subject}"));
    let xp = u32::try_from(sheet.int(&key).unwrap_or(0)).map_err(|_| "negative experience")?;
    let affinity = sheet.fields.get(&field(&format!("affinity:{subject}"))) == Some(&FieldValue::Bool(true));
    let gain = rules::summa_gain(xp, level, quality, affinity, cost)
        .ok_or_else(|| format!("this summa's Level {level} teaches nothing past the current score"))?;
    sheet.set_int(key, i64::from(xp + gain));
    sheet.set_int(field(subject), i64::from(rules::score(xp + gain, cost)));
    Ok(gain)
}

#[cfg(test)]
mod tests;
