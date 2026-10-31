use super::*;
use isocosm_campaign::{EquipmentSlot, ItemInstance, ItemModifierKind};

fn item<'a>(
    inventories: &'a BTreeMap<TokenId, Inventory>,
    id: &ItemId,
) -> Result<(TokenId, &'a ItemInstance), String> {
    let matches: Vec<_> = inventories
        .iter()
        .filter_map(|(owner, inventory)| inventory.items.get(id).map(|item| (*owner, item)))
        .collect();
    if matches.len() != 1 {
        return Err("weapon identity is absent or owned more than once".into());
    }
    Ok(matches[0])
}

pub fn cast(
    map: &mut MapDocument,
    inventories: &BTreeMap<TokenId, Inventory>,
    definitions: &Definitions,
    request: &Cast,
    context: &CastContext<'_>,
    rng: &mut Rng,
) -> Result<Receipt, String> {
    let mut records = Records::at(map, context.clock)?;
    if let Some(prior) = records
        .casts
        .iter()
        .find(|r| r.request.request_id == request.request_id)
    {
        return if &prior.request == request {
            Ok(prior.clone())
        } else {
            Err("cast request identity conflicts with accepted history".into())
        };
    }
    if request.request_id.trim().is_empty()
        || request.request_id.len() > 256
        || records.casts.len() >= 4096
    {
        return Err("invalid cast request identity or effect history limit".into());
    }
    let def = definitions.get(&request.definition)?.clone();
    if request.rank < def.level
        || (def.edition != Edition::Ars5
            && request.rank > if def.edition == Edition::Dnd5 { 9 } else { 10 })
        || (def.edition == Edition::Ars5 && request.rank != def.level)
    {
        return Err("invalid casting rank or formulaic level".into());
    }
    if def.edition == Edition::Pathfinder2 && def.kind == Kind::Size && request.rank >= 6 {
        return Err("multi-target heightening needs a single group transaction; use the supported single-target ranks".into());
    }
    let starts = map
        .token(request.caster)
        .ok_or("caster token is absent")?
        .at;
    let target_id = match &request.target {
        Target::Creature(id) => *id,
        Target::Item(id) => item(inventories, id)?.0,
    };
    let target = map
        .token(target_id)
        .ok_or("target's physical position is absent")?;
    let target_sheet = map.sheets.get(&target_id).ok_or("target has no sheet")?;
    if target_sheet.system != def.edition.system() {
        return Err("cross-edition targeting requires a named hybrid procedure".into());
    }
    if (def.kind == Kind::Weapon) != matches!(request.target, Target::Item(_)) {
        return Err("effect targets the wrong bearer kind".into());
    }
    let personal = def.personal && !(def.edition == Edition::Pathfinder2 && request.rank >= 3);
    if personal && target_id != request.caster {
        return Err("base Darkvision is personal in this edition".into());
    }
    let range = if def.personal && !personal {
        5
    } else {
        def.range_ft
    };
    if context.feet_per_tile == 0 {
        return Err("board distance calibration is undeclared".into());
    }
    let distance =
        u64::from(isocosm_core::distance(starts, target.at)) * u64::from(context.feet_per_tile);
    if distance > u64::from(range) || !(context.line_of_effect)(starts, target.at) {
        return Err("effect target is out of range or line of effect".into());
    }
    let ruled = request
        .ruling
        .as_ref()
        .is_some_and(|r| !r.trim().is_empty());
    if request.resisted && !ruled {
        return Err("resistance outcome requires its adjudication".into());
    }
    if !request.willing && !(def.edition == Edition::Dnd5 && def.kind == Kind::Size) && !ruled {
        return Err("target must be willing or resistance must be adjudicated".into());
    }
    if def.kind == Kind::Size {
        let valid = match def.edition {
            Edition::Dnd5 => matches!(request.size_change, -1 | 1),
            Edition::Ars5 => matches!(request.size_change, -2 | -1 | 1),
            Edition::Pathfinder2 => request.size_change == 1,
        };
        if !valid {
            return Err("unsupported size-change choice".into());
        }
    }
    if def.edition == Edition::Ars5
        && def.kind != Kind::Weapon
        && (target_sheet.text("body_form") != Some("corpus")
            || target_sheet.int("ars_size").unwrap_or(0) > 1)
    {
        return Err("Corpus Individual requires a human body within its declared Size".into());
    }
    if let Target::Item(id) = &request.target {
        let (owner, weapon) = item(inventories, id)?;
        let inventory = &inventories[&owner];
        if !weapon.tags.iter().any(|tag| tag == "weapon")
            || ![EquipmentSlot::MainHand, EquipmentSlot::OffHand]
                .iter()
                .any(|slot| inventory.equipped.get(slot) == Some(id))
        {
            return Err("this procedure requires a positioned, wielded weapon".into());
        }
        if def.edition == Edition::Dnd5
            && (weapon.tags.iter().any(|tag| tag == "magical")
                || weapon
                    .modifiers
                    .iter()
                    .any(|m| m.kind == ItemModifierKind::Enchantment)
                || records.effects.values().any(|e| {
                    e.target == request.target
                        && e.definition.kind == Kind::Weapon
                        && e.applies(context.clock.tick)
                }))
        {
            return Err("Magic Weapon requires a nonmagical weapon".into());
        }
        if def.edition == Edition::Ars5
            && !(weapon.tags.iter().any(|t| t == "metal")
                && weapon.tags.iter().any(|t| t == "edged" || t == "pointed"))
        {
            return Err("Edge of the Razor requires a metal edge or point".into());
        }
    }
    let expires = procedures::expiry(&def, request, context.clock)?;
    let mut candidate = map.clone();
    let mut proposed_rng = rng.clone();
    lifecycle::settle(&mut records, &candidate, context.clock);
    if def.edition == Edition::Pathfinder2 && def.kind == Kind::Size {
        let prior: Vec<_> = records
            .effects
            .values()
            .filter(|e| {
                e.target == request.target
                    && e.running(context.clock.tick)
                    && e.definition.edition == Edition::Pathfinder2
                    && e.definition.kind == Kind::Size
            })
            .map(|e| e.id)
            .collect();
        if !prior.is_empty() && !ruled {
            return Err("another polymorph needs its counteract adjudication".into());
        }
        if ruled && !request.resisted {
            for id in prior {
                records.effects.get_mut(&id).unwrap().ended = Some((
                    context.clock.tick,
                    End::Replaced {
                        ruling: request.ruling.clone().unwrap(),
                    },
                ));
            }
        }
    }
    let mut receipt = Receipt {
        request: request.clone(),
        tick: context.clock.tick,
        effect: None,
        costs: BTreeMap::new(),
        dice: Vec::new(),
        casting_total: None,
        outcome: String::new(),
        definition_revision: def.revision(),
        experimental: true,
    };
    let mut success = procedures::charge(
        &mut candidate,
        &def,
        request,
        context.in_combat,
        &mut proposed_rng,
        &mut receipt,
    )?;
    if def.concentration {
        for effect in records.effects.values_mut().filter(|e| {
            e.caster == request.caster && e.concentration && e.running(context.clock.tick)
        }) {
            effect.ended = Some((context.clock.tick, End::Concentration));
        }
    }
    if !request.willing && def.edition == Edition::Dnd5 && def.kind == Kind::Size && !ruled {
        let dc = candidate.sheets[&request.caster]
            .int("spell_save_dc")
            .ok_or("spell save DC is undeclared")?;
        let bonus = target_sheet
            .int("con_save_bonus")
            .ok_or("target Constitution save bonus is undeclared")?;
        let die = proposed_rng.die(20) as u16;
        receipt.dice.push(die);
        success &= i64::from(die)
            .checked_add(bonus)
            .ok_or("saving throw overflow")?
            < dc;
    }
    success &= !request.resisted;
    if def.edition == Edition::Pathfinder2 && def.kind == Kind::Size {
        let limit = if request.rank >= 4 { 4 } else { 3 }; // Tiny=0, Small=1, Medium=2, Large=3, Huge=4.
        success &= target_sheet
            .int("size_category")
            .ok_or("target size category is undeclared")?
            < limit;
    }
    if success {
        let id = records.next_id;
        records.next_id = id.checked_add(1).ok_or("effect identity overflow")?;
        let penetration = if def.edition == Edition::Ars5 {
            Some(
                receipt.casting_total.unwrap() - i64::from(def.level)
                    + candidate.sheets[&request.caster]
                        .int("penetration_bonus")
                        .unwrap_or(0),
            )
        } else {
            None
        };
        records.effects.insert(
            id,
            Active {
                id,
                definition: def.clone(),
                caster: request.caster,
                target: request.target.clone(),
                rank: request.rank,
                size_change: request.size_change,
                penetration,
                started: context.clock.tick,
                expires,
                concentration: def.concentration,
                suppressed_by: BTreeSet::new(),
                ended: None,
            },
        );
        receipt.effect = Some(id);
        receipt.outcome = "effect activated".into();
    } else {
        receipt.outcome = "cast resolved without an effect".into();
    }
    records.casts.push(receipt.clone());
    records.store(&mut candidate)?;
    *map = candidate;
    *rng = proposed_rng;
    Ok(receipt)
}
