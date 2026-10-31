use super::*;

pub(super) fn expiry(def: &Definition, request: &Cast, clock: Clock) -> Result<u64, String> {
    if def.edition == Edition::Pathfinder2 && def.kind == Kind::Sight && request.rank >= 5 {
        return clock
            .next_daily_preparation
            .filter(|tick| *tick > clock.tick)
            .ok_or("next daily preparation must be declared for this heightened spell".into());
    }
    match def.duration {
        Duration::Seconds(seconds) => clock.seconds(seconds),
        Duration::Sun => clock
            .next_sun
            .filter(|tick| *tick > clock.tick)
            .ok_or("Sun duration needs the next accepted sunrise or sunset".into()),
    }
}

pub(super) fn charge(
    map: &mut MapDocument,
    def: &Definition,
    request: &Cast,
    combat: bool,
    rng: &mut Rng,
    receipt: &mut Receipt,
) -> Result<bool, String> {
    let sheet = map
        .sheets
        .get_mut(&request.caster)
        .ok_or("caster has no sheet")?;
    if sheet.system != def.edition.system() {
        return Err("caster ruleset does not match procedure".into());
    }
    if sheet.fields.get(&format!("known:{}", def.id)) != Some(&FieldValue::Bool(true)) {
        return Err("spell is not declared known".into());
    }
    if sheet.int("hp_current").is_some_and(|hp| hp <= 0)
        || ["incapacitated", "unconscious", "stunned", "paralyzed"]
            .iter()
            .any(|key| {
                map.conditions
                    .get(&request.caster)
                    .is_some_and(|c| c.get(*key).is_some_and(|n| *n > 0))
            })
    {
        return Err("caster cannot act".into());
    }
    let require = |sheet: &SheetData, key: &str| {
        if sheet.fields.get(key) == Some(&FieldValue::Bool(true)) {
            Ok(())
        } else {
            Err(format!("casting requirement unavailable: {key}"))
        }
    };
    if combat {
        let counters = map.turn_counters.entry(request.caster).or_default();
        let used = |key: &str| counters.get(key).copied().unwrap_or(0);
        if def.edition == Edition::Dnd5
            && ((def.bonus_action && used("effect_levelled_actions") > 0)
                || (!def.bonus_action && used("effect_bonus_spell") > 0))
        {
            return Err("bonus-action spell prevents another levelled spell this turn".into());
        }
        let key = if def.bonus_action {
            "bonus_actions_spent"
        } else {
            "actions_spent"
        };
        let amount = if def.bonus_action {
            1
        } else {
            i64::from(def.actions)
        };
        let limit = if def.bonus_action {
            1
        } else {
            sheet
                .int("actions_per_turn")
                .unwrap_or(if def.edition == Edition::Pathfinder2 {
                    3
                } else {
                    1
                })
        };
        let next = used(key)
            .checked_add(amount)
            .ok_or("action counter overflow")?;
        if next > limit {
            return Err("not enough actions this turn".into());
        }
        counters.insert(key.into(), next);
        receipt.costs.insert(format!("turn:{key}"), amount);
        if def.edition == Edition::Dnd5 {
            counters.insert(
                if def.bonus_action {
                    "effect_bonus_spell"
                } else {
                    "effect_levelled_actions"
                }
                .into(),
                1,
            );
        }
    }
    match def.edition {
        Edition::Dnd5 | Edition::Pathfinder2 => {
            require(sheet, "can_manipulate")?;
            if def.edition == Edition::Dnd5 {
                require(sheet, "can_speak")?;
                if map
                    .conditions
                    .get(&request.caster)
                    .is_some_and(|c| c.contains_key("silenced"))
                {
                    return Err("verbal component blocked by silence".into());
                }
                if def.kind != Kind::Weapon {
                    require(sheet, "material_or_focus")?;
                }
            } else if map
                .conditions
                .get(&request.caster)
                .is_some_and(|c| c.contains_key("restrained") || c.contains_key("grabbed"))
                && request.ruling.as_ref().is_none_or(|r| r.trim().is_empty())
            {
                return Err(
                    "restricted manipulation needs an adjudicated flat check or exception".into(),
                );
            }
            let key = if def.edition == Edition::Dnd5 {
                format!("slot:{}", request.rank)
            } else {
                format!("prepared:{}:{}", def.id, request.rank)
            };
            let left = sheet
                .int(&key)
                .ok_or_else(|| format!("casting resource undeclared: {key}"))?;
            if left <= 0 {
                return Err("no prepared cast or spell slot remains".into());
            }
            sheet.set_int(&key, left - 1);
            receipt.costs.insert(key, -1);
            Ok(true)
        },
        Edition::Ars5 => {
            if sheet.fields.get("stressful") == Some(&FieldValue::Bool(true))
                || sheet.int("spell_mastery").unwrap_or(0) > 0
            {
                return Err("this formulaic procedure requires an adjudicated stress die; calm unmastered casting is executable".into());
            }
            let fatigue = sheet.int("fatigue_lost").unwrap_or(0);
            let penalty = match fatigue {
                0 | 1 => 0,
                2 => 1,
                3 => 3,
                4 => 5,
                _ => return Err("caster is unconscious from fatigue".into()),
            };
            let stat = |key: &str| {
                sheet
                    .int(key)
                    .ok_or_else(|| format!("casting statistic undeclared: {key}"))
            };
            let mut form = stat(def.form.as_deref().ok_or("formulaic Form missing")?)?;
            if let Some(requisite) = &def.requisite {
                form = form.min(stat(requisite)?);
            }
            let score = stat("stamina")?
                .checked_add(stat("muto")?)
                .and_then(|n| n.checked_add(form))
                .and_then(|n| n.checked_add(sheet.int("casting_modifier").unwrap_or(0)))
                .and_then(|n| n.checked_add(sheet.int("aura_bonus").unwrap_or(0)))
                .and_then(|n| n.checked_sub(penalty))
                .ok_or("casting total overflow")?;
            let die = rng.die(10) as u16;
            receipt.dice.push(die);
            let total = score
                .checked_add(i64::from(die))
                .ok_or("casting total overflow")?;
            receipt.casting_total = Some(total);
            if total < i64::from(def.level) {
                sheet.set_int("fatigue_lost", fatigue + 1);
                receipt.costs.insert("fatigue_lost".into(), 1);
            }
            Ok(total >= i64::from(def.level) - 10)
        },
    }
}
