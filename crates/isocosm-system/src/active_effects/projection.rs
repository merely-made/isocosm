use super::*;
use isocosm_campaign::{EquipmentSlot, ItemModifierKind};
use isocosm_core::{HeightSightRules, TileCoord};

fn potency(effect: &Active) -> u8 {
    match (effect.definition.edition, effect.definition.kind) {
        (Edition::Dnd5, Kind::Weapon) => {
            if effect.rank >= 6 {
                3
            } else if effect.rank >= 4 {
                2
            } else {
                1
            }
        },
        (Edition::Pathfinder2, Kind::Weapon) => {
            if effect.rank >= 9 {
                3
            } else if effect.rank >= 6 {
                2
            } else {
                1
            }
        },
        (Edition::Pathfinder2, Kind::Size) => {
            if effect.rank >= 4 {
                2
            } else {
                1
            }
        },
        _ => 1,
    }
}

pub(super) fn selected<'a>(records: &'a Records, target: &Target) -> Vec<&'a Active> {
    let mut groups: BTreeMap<&str, &Active> = BTreeMap::new();
    for effect in records
        .effects
        .values()
        .filter(|e| &e.target == target && e.applies(records.tick))
    {
        let old = groups.get(effect.definition.id.as_str());
        if old.is_none_or(|old| {
            (potency(effect), effect.started, effect.id) > (potency(old), old.started, old.id)
        }) {
            groups.insert(&effect.definition.id, effect);
        }
    }
    groups.into_values().collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponChange {
    pub attack_bonus: i64,
    pub damage_bonus: i64,
    pub damage_dice: i64,
    pub magical: bool,
    pub resisted_by_magic: bool,
    pub penetration: Option<i64>,
}

pub fn weapon(map: &MapDocument, item: &ItemId) -> Result<WeaponChange, String> {
    let Some(records) = Records::read(map)? else {
        return Ok(WeaponChange::default());
    };
    let mut change = WeaponChange::default();
    for effect in selected(&records, &Target::Item(item.clone())) {
        change.magical = true;
        match effect.definition.edition {
            Edition::Dnd5 => {
                let bonus = i64::from(potency(effect));
                change.attack_bonus = change.attack_bonus.max(bonus);
                change.damage_bonus = change.damage_bonus.max(bonus);
            },
            Edition::Pathfinder2 => {
                let bonus = i64::from(potency(effect));
                change.attack_bonus = change.attack_bonus.max(bonus);
                change.damage_dice = change.damage_dice.max(bonus + 1);
            },
            Edition::Ars5 => {
                change.damage_bonus = change.damage_bonus.max(2);
                change.resisted_by_magic = true;
                change.penetration = Some(
                    change
                        .penetration
                        .unwrap_or(i64::MIN)
                        .max(effect.penetration.unwrap_or(0)),
                );
            },
        }
    }
    Ok(change)
}

/// Recompute against today's base sheet. Base edits and wounds survive expiry;
/// no rollback snapshot can erase another source's changes.
pub fn project_sheet(
    map: &MapDocument,
    token: TokenId,
    inventory: Option<&Inventory>,
    base: &SheetData,
) -> Result<SheetData, String> {
    let Some(records) = Records::read(map)? else {
        return Ok(base.clone());
    };
    let mut result = base.clone();
    for effect in selected(&records, &Target::Creature(token)) {
        if base.system != effect.definition.edition.system() {
            return Err("effect requires an explicit cross-edition projection".into());
        }
        match (effect.definition.edition, effect.definition.kind) {
            (Edition::Dnd5, Kind::Sight) => result.set_int(
                "darkvision_ft",
                result.int("darkvision_ft").unwrap_or(0).max(60),
            ),
            (Edition::Pathfinder2, Kind::Sight) => {
                result
                    .fields
                    .insert("darkvision".into(), FieldValue::Bool(true));
            },
            (Edition::Ars5, Kind::Sight) => {
                result
                    .fields
                    .insert("cat_eyes".into(), FieldValue::Bool(true));
            },
            (Edition::Dnd5, Kind::Size) => {
                let change = i64::from(effect.size_change);
                add(&mut result, "size_category", change)?;
                result.set_int("strength_advantage", change);
                if let Some(weight) = base.int("weight_mg") {
                    result.set_int(
                        "weight_mg",
                        if change > 0 {
                            weight.checked_mul(8).ok_or("weight overflow")?
                        } else {
                            weight / 8
                        },
                    );
                }
                if inventory.is_some_and(|inv| inv.equipped.contains_key(&EquipmentSlot::MainHand))
                {
                    result.set_text("effect_weapon_action", "attack");
                    result.set_int("effect_weapon_d4", change);
                }
            },
            (Edition::Pathfinder2, Kind::Size) => {
                result.set_int("size_category", if effect.rank >= 4 { 4 } else { 3 });
                result.set_int("clumsy", result.int("clumsy").unwrap_or(0).max(1));
                let reach = base.int("reach_ft").ok_or("base reach is undeclared")?;
                let tiny = base.int("size_category") == Some(0);
                result.set_int(
                    "reach_ft",
                    reach
                        .checked_add(if effect.rank >= 4 {
                            if tiny { 15 } else { 10 }
                        } else if tiny {
                            10
                        } else {
                            5
                        })
                        .ok_or("reach overflow")?,
                );
                // The PF2e pack's tactical board uses five-foot squares.
                result.set_int("effect_reach_tiles", result.int("reach_ft").unwrap() / 5);
                result.set_int(
                    "melee_damage_status_bonus",
                    result
                        .int("melee_damage_status_bonus")
                        .unwrap_or(0)
                        .max(if effect.rank >= 4 { 4 } else { 2 }),
                );
                result.set_text("effect_weapon_action", "strike");
            },
            (Edition::Ars5, Kind::Size) => {
                add(&mut result, "ars_size", i64::from(effect.size_change))?;
                // Corpus's Size is authoritative. The book does not specify
                // exact voxel dimensions; project_body requires authored scale.
            },
            (_, Kind::Weapon) => return Err("weapon effect on a creature".into()),
        }
    }
    if let Some(inventory) = inventory {
        if let Some(item_id) = inventory.equipped.get(&EquipmentSlot::MainHand) {
            if let Some(item) = inventory.items.get(item_id) {
                let change = weapon(map, item_id)?;
                if change.magical {
                    for effect in selected(&records, &Target::Item(item_id.clone())) {
                        if effect.definition.edition.system() != base.system {
                            return Err("weapon needs a cross-edition projection".into());
                        }
                    }
                    // PF2e potency is an item bonus, not an additive second
                    // rune. D&D's nonmagical target and Ars's edge add their
                    // own modifier rather than consuming mundane quality.
                    let old_attack = if base.system == "pf2e-srd" {
                        item.modifiers
                            .iter()
                            .filter(|m| m.kind == ItemModifierKind::Enchantment)
                            .filter_map(|m| m.stats.get("attack_bonus"))
                            .copied()
                            .max()
                            .unwrap_or(0)
                            .max(0)
                    } else {
                        0
                    };
                    add(
                        &mut result,
                        "attack_bonus",
                        (change.attack_bonus - old_attack).max(0),
                    )?;
                    result.set_int("effect_weapon_damage", change.damage_bonus);
                    result.set_int("effect_weapon_dice", change.damage_dice);
                    result
                        .fields
                        .insert("magical_weapon".into(), FieldValue::Bool(true));
                    result.set_text(
                        "effect_weapon_action",
                        if base.system == "pf2e-srd" {
                            "strike"
                        } else {
                            "attack"
                        },
                    );
                }
            }
        }
    }
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lighting {
    Bright,
    Dim,
    NearDark,
    AbsoluteDark,
    MagicalDark { rank: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perceived {
    Colour,
    Grey,
    Hidden,
}

pub fn perceive(sheet: &SheetData, lighting: Lighting, distance_ft: u64) -> Perceived {
    let flag = |key: &str| sheet.fields.get(key) == Some(&FieldValue::Bool(true));
    if sheet.int("blinded").is_some_and(|v| v > 0) || flag("blinded") {
        return Perceived::Hidden;
    }
    if lighting == Lighting::Bright {
        return Perceived::Colour;
    }
    let dnd = sheet.system == "5e-srd";
    let darkvision = if dnd {
        sheet
            .int("darkvision_ft")
            .is_some_and(|range| range > 0 && distance_ft <= range as u64)
    } else {
        flag("darkvision")
    };
    if let Lighting::MagicalDark { rank } = lighting {
        return if !dnd && rank < 4 && darkvision {
            Perceived::Grey
        } else {
            Perceived::Hidden
        };
    }
    match lighting {
        Lighting::Dim => {
            if darkvision {
                Perceived::Colour
            } else {
                Perceived::Grey
            }
        },
        Lighting::NearDark => {
            if darkvision || flag("cat_eyes") {
                Perceived::Grey
            } else {
                Perceived::Hidden
            }
        },
        Lighting::AbsoluteDark => {
            if darkvision {
                Perceived::Grey
            } else {
                Perceived::Hidden
            }
        },
        _ => Perceived::Hidden,
    }
}

/// Combine actual observer senses with the existing height/terrain line tracer.
pub fn visible_tiles(
    map: &MapDocument,
    token: TokenId,
    inventory: Option<&Inventory>,
    sight: &HeightSightRules<'_>,
    feet_per_tile: u32,
    light: &dyn Fn(TileCoord) -> Lighting,
) -> Result<BTreeSet<TileCoord>, String> {
    if feet_per_tile == 0 {
        return Err("board distance calibration is undeclared".into());
    }
    let observer = map.token(token).ok_or("observer is absent")?;
    let base = map.sheet(token).ok_or("observer has no sheet")?;
    let sheet = project_sheet(map, token, inventory, base)?;
    let sheet =
        crate::sheet_with_conditions(&sheet, map.conditions.get(&token).into_iter().flatten());
    Ok(isocosm_core::visible_from_height(map, observer.at, sight)
        .into_iter()
        .filter(|at| {
            perceive(
                &sheet,
                light(*at),
                u64::from(isocosm_core::distance(observer.at, *at)) * u64::from(feet_per_tile),
            ) != Perceived::Hidden
        })
        .collect())
}
