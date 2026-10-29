use super::*;

pub(super) fn settle(records: &mut Records, map: &MapDocument, clock: Clock) -> Vec<u64> {
    records.tick = clock.tick;
    let mut ended = Vec::new();
    for effect in records.effects.values_mut().filter(|e| e.ended.is_none()) {
        if clock.tick >= effect.expires {
            effect.ended = Some((effect.expires, End::Expired));
        } else if effect.concentration
            && map.sheets.get(&effect.caster).is_some_and(|sheet| {
                sheet.int("hp_current").is_some_and(|hp| hp <= 0)
                    || ["incapacitated", "unconscious", "stunned", "paralyzed"]
                        .iter()
                        .any(|key| map.has_condition(effect.caster, key))
            })
        {
            effect.ended = Some((clock.tick, End::Incapacitated));
        }
        if effect.ended.is_some() {
            ended.push(effect.id);
        }
    }
    ended
}

pub fn advance(map: &mut MapDocument, clock: Clock) -> Result<Vec<u64>, String> {
    if Records::read(map)?.is_none() {
        return Ok(Vec::new());
    }
    let mut records = Records::at(map, clock)?;
    let ended = settle(&mut records, map, clock);
    records.store(map)?;
    Ok(ended)
}

/// Voluntary removal is allowed for a concentrating caster. Other durations
/// require their own edition procedure or an explicitly recorded GM ruling.
pub fn dismiss(map: &mut MapDocument, id: u64, by: TokenId, clock: Clock) -> Result<(), String> {
    let mut records = Records::at(map, clock)?;
    let effect = records.effects.get(&id).ok_or("unknown active effect")?;
    if effect.caster != by || !effect.concentration {
        return Err("caster cannot dismiss this effect by concentration".into());
    }
    settle(&mut records, map, clock);
    let effect = records.effects.get_mut(&id).unwrap();
    if effect.ended.is_none() {
        effect.ended = Some((clock.tick, End::Dismissed));
    }
    records.store(map)
}

/// The product authorizes this adjudication. A dispel/counteract is not turned
/// into a universal roll: its edition's completed ruling is retained by cause.
pub fn interrupt(map: &mut MapDocument, id: u64, clock: Clock, reason: End) -> Result<(), String> {
    if !matches!(&reason, End::Dispelled { ruling } | End::Replaced { ruling } if !ruling.trim().is_empty())
        && !matches!(&reason, End::Concentration | End::Incapacitated)
    {
        return Err("interference needs a named adjudication or concentration event".into());
    }
    let mut records = Records::at(map, clock)?;
    if !records.effects.contains_key(&id) {
        return Err("unknown active effect".into());
    }
    settle(&mut records, map, clock);
    let effect = records.effects.get_mut(&id).unwrap();
    if effect.ended.is_none() {
        effect.ended = Some((clock.tick, reason));
    }
    records.store(map)
}

/// Multiple independent suppression sources coexist. Removing one cannot
/// revive an expired/dispelled effect or lift another source's suppression.
pub fn set_suppression(
    map: &mut MapDocument,
    id: u64,
    clock: Clock,
    source: &str,
    suppress: bool,
) -> Result<(), String> {
    if source.trim().is_empty() {
        return Err("suppression needs a source".into());
    }
    let mut records = Records::at(map, clock)?;
    if !records.effects.contains_key(&id) {
        return Err("unknown active effect".into());
    }
    settle(&mut records, map, clock);
    let effect = records.effects.get_mut(&id).unwrap();
    if effect.ended.is_none() {
        if suppress {
            effect.suppressed_by.insert(source.into());
        } else {
            effect.suppressed_by.remove(source);
        }
    }
    records.store(map)
}

/// D&D's concentration check uses the existing dice stream and the supplied
/// edition-native Constitution save total bonus. Failed checks end all of
/// that caster's concentrating effects in one saved change.
pub fn concentration_damage(
    map: &mut MapDocument,
    caster: TokenId,
    damage: u32,
    clock: Clock,
    rng: &mut Rng,
) -> Result<Option<(i64, i64)>, String> {
    let mut records = Records::at(map, clock)?;
    settle(&mut records, map, clock);
    if damage == 0
        || !records
            .effects
            .values()
            .any(|e| e.caster == caster && e.concentration && e.running(clock.tick))
    {
        records.store(map)?;
        return Ok(None);
    }
    let sheet = map.sheets.get(&caster).ok_or("caster has no sheet")?;
    let bonus = sheet
        .int("con_save_bonus")
        .ok_or("Constitution save bonus is undeclared")?;
    let mut proposed = rng.clone();
    let first = i64::from(proposed.die(20));
    let die = if sheet.fields.get("concentration_advantage") == Some(&FieldValue::Bool(true)) {
        first.max(i64::from(proposed.die(20)))
    } else {
        first
    };
    let total = die
        .checked_add(bonus)
        .ok_or("concentration save overflow")?;
    let dc = i64::from(10.max(damage / 2));
    if total < dc {
        for effect in records
            .effects
            .values_mut()
            .filter(|e| e.caster == caster && e.concentration && e.running(clock.tick))
        {
            effect.ended = Some((clock.tick, End::Concentration));
        }
    }
    records.store(map)?;
    *rng = proposed;
    Ok(Some((total, dc)))
}
