use super::*;
use effects::{Cast, CastContext, Definitions, Target};
use isometry_campaign::{EquipmentSlot, ItemInstance};
use isometry_system::{srd_5e, srd_items};

pub(super) fn apply(
    state: &mut State,
    map: &mut MapDocument,
    session: &Session,
    sections: &BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    action: &Action,
    payload: &[u8],
) -> Result<(String, Vec<RollRecord>), String> {
    use frame::{authorize, dm};
    let mut rolls = Vec::new();
    let note = match action {
        Action::Create(id) => {
            dm(access)?;
            authorize(session, access, *id)?;
            let token = token(*id)?;
            if map.sheets.contains_key(&token) {
                return Err("character already has an edition sheet".into());
            }
            let mut sheet = srd_5e().default_sheet();
            sheet.set_text("name", format!("Character {id}"));
            sheet.set_int("con_save_bonus", 0);
            for rank in 2..=9 {
                sheet.set_int(format!("slot:{rank}"), 0);
            }
            for key in ["can_speak", "can_manipulate"] {
                sheet.fields.insert(key.into(), FieldValue::Bool(true));
            }
            map.set_sheet(token, sheet);
            map.tokens.push(isometry_core::Token {
                id: token,
                at: (0, 0),
                facing: Default::default(),
                sprite: String::new(),
                owner: None,
            });
            format!("D&D SRD 5.1 sheet created for native body {id}; experimental ruleset facts")
        },
        Action::Adjust {
            actor,
            field,
            delta,
        } => {
            dm(access)?;
            let id = authorize(session, access, *actor)?;
            let allowed = srd_5e()
                .fields
                .iter()
                .any(|f| f.key == *field && matches!(f.default, FieldValue::Int(_)))
                || field == "con_save_bonus"
                || (2..=9).any(|r| *field == format!("slot:{r}"));
            if !allowed || ![-1, 1].contains(delta) {
                return Err("invalid sheet adjustment".into());
            }
            if field == "hp_current" && *delta < 0 {
                return Err("use the damage action so concentration is checked".into());
            }
            let sheet = map.sheets.get_mut(&id).ok_or("no edition sheet")?;
            let value = sheet
                .int(field)
                .ok_or("field is not an integer")?
                .checked_add(*delta)
                .ok_or("sheet adjustment overflow")?;
            if value < 0 && (field.starts_with("slot:") || field.starts_with("hp_")) {
                return Err("resource cannot be negative".into());
            }
            sheet.set_int(field, value);
            effects::advance(map, clock(session))?;
            format!("DM adjusted {actor} {field} by {delta:+}")
        },
        Action::Learn(actor) => {
            dm(access)?;
            let id = authorize(session, access, *actor)?;
            map.sheets
                .get_mut(&id)
                .ok_or("no edition sheet")?
                .fields
                .insert("known:dnd5:magic-weapon".into(), FieldValue::Bool(true));
            format!("DM granted Magic Weapon to {actor}")
        },
        Action::Give { actor, template } => {
            dm(access)?;
            let id = authorize(session, access, *actor)?;
            if !["longsword", "club"].contains(&template.as_str()) {
                return Err("this tranche supports one-handed longsword and club attacks".into());
            }
            if map.sheet(id).is_none() {
                return Err("weapon bearer has no sheet".into());
            }
            let item = srd_items()
                .into_iter()
                .find(|i| i.key == *template && i.category == "Weapon")
                .ok_or("SRD weapon is absent")?;
            let damage = item
                .detail
                .split_whitespace()
                .next()
                .ok_or("weapon damage is absent")?
                .to_string();
            if isometry_core::roll(&damage, &mut Rng::new(1)).is_none() {
                return Err("weapon damage is unsupported".into());
            }
            let item_id = ItemId(format!("ruleset:weapon:{}", state.next_item));
            state.next_item = state
                .next_item
                .checked_add(1)
                .ok_or("item identity overflow")?;
            let inventory = state.inventories.entry(id).or_default();
            inventory.items.insert(
                item_id.clone(),
                ItemInstance {
                    id: item_id.clone(),
                    template: format!("srd5e:{template}"),
                    name: item.name,
                    quantity: 1,
                    tags: vec!["weapon".into()],
                    modifiers: vec![],
                    appearance_layers: vec![],
                },
            );
            state.weapon_damage.insert(item_id, damage);
            format!("Character {actor} received a {template} as table equipment")
        },
        Action::Equip { actor, item } => {
            let id = authorize(session, access, *actor)?;
            frame::unmarked(
                sections,
                state.inventories.get(&id).ok_or("no inventory")?,
                item,
            )?;
            let inventory = state.inventories.get_mut(&id).ok_or("no inventory")?;
            if !inventory.items.contains_key(item) {
                return Err("weapon is not in this character's inventory".into());
            }
            let name = inventory.items[item].name.clone();
            inventory
                .equipped
                .insert(EquipmentSlot::MainHand, item.clone());
            format!("Character {actor} wielded {name}")
        },
        Action::Cast { actor, item, rank } => {
            let id = authorize(session, access, *actor)?;
            let inv = state.inventories.get(&id).ok_or("no inventory")?;
            if inv.equipped.get(&EquipmentSlot::MainHand) != Some(item) {
                return Err("select this caster's wielded weapon".into());
            }
            frame::unmarked(sections, inv, item)?;
            let name = inv.items[item].name.clone();
            let request = Cast {
                request_id: format!(
                    "table:{}:{}:{}",
                    session.sim.state().tick,
                    access.participant.0,
                    state.log.len()
                ),
                definition: "dnd5:magic-weapon".into(),
                caster: id,
                target: Target::Item(item.clone()),
                rank: *rank,
                size_change: 0,
                willing: true,
                ruling: None,
                resisted: false,
            };
            effects::cast(
                map,
                &state.inventories,
                &Definitions::builtin()?,
                &request,
                &CastContext {
                    clock: clock(session),
                    feet_per_tile: 5,
                    in_combat: false,
                    line_of_effect: &|_, _| true,
                },
                &mut state.rng,
            )?;
            format!(
                "Character {actor} cast Magic Weapon on {name} using a level {rank} slot; concentration started"
            )
        },
        Action::Attack { actor, target } | Action::RuleAttack { actor, target } => {
            let a = authorize(session, access, *actor)?;
            let b = token(*target)?;
            if map
                .sheet(a)
                .is_none_or(|s| s.int("hp_current").unwrap_or(0) <= 0)
            {
                return Err("attacker cannot act".into());
            }
            let ruled = matches!(action, Action::RuleAttack { .. });
            let ruling_payload = if ruled {
                dm(access)?;
                if state.cells_per_square.is_some() {
                    return Err("calibrated physical reach uses the native placement".into());
                }
                br#"{"squares":1,"reason":"DM ruled the bodies within five feet with a clear path by choosing Rule within reach and attack"}"#.as_slice()
            } else {
                payload
            };
            let (distance, ruling) = frame::reach(
                state,
                session,
                sections,
                access,
                *actor,
                *target,
                ruling_payload,
            )?;
            let inventory = state.inventories.get(&a).ok_or("attacker has no weapon")?;
            let item = inventory
                .equipped
                .get(&EquipmentSlot::MainHand)
                .ok_or("attacker has no wielded weapon")?;
            frame::unmarked(sections, inventory, item)?;
            let damage = state
                .weapon_damage
                .get(item)
                .ok_or("weapon has no accepted damage definition")?;
            let mut system = srd_5e();
            system
                .actions
                .iter_mut()
                .find(|a| a.key == "attack")
                .unwrap()
                .target
                .as_mut()
                .unwrap()
                .damage = damage.clone();
            let actor_sheet = system.effective_map_sheet(
                map,
                a,
                map.sheet(a).ok_or("attacker has no sheet")?,
                Some(inventory),
            )?;
            let target_sheet = system.effective_map_sheet(
                map,
                b,
                map.sheet(b).ok_or("target has no sheet")?,
                state.inventories.get(&b),
            )?;
            let result = system
                .resolve_action(
                    "attack",
                    a,
                    &actor_sheet,
                    (0, 0),
                    b,
                    &target_sheet,
                    (i32::try_from(distance).map_err(|_| "distance overflow")?, 0),
                    &mut state.rng,
                )
                .map_err(|e| format!("attack refused: {e:?}"))?;
            rolls.push(result.attack.clone());
            let mut extra = String::new();
            if let Some(damage) = result.damage {
                extra = frame::hurt(
                    state,
                    map,
                    session,
                    b,
                    u32::try_from(damage.total).map_err(|_| "negative attack damage")?,
                    &mut rolls,
                )?;
                rolls.push(damage);
            }
            format!(
                "{actor} attacked {target}: {}{extra}; {ruling}",
                if result.hit { "hit" } else { "miss" }
            )
        },
        Action::Damage { target, amount } => {
            dm(access)?;
            if *amount == 0 {
                return Err("damage needs a positive amount".into());
            }
            let extra = frame::hurt(state, map, session, token(*target)?, *amount, &mut rolls)?;
            format!("DM adjudicated {amount} external damage to {target}{extra}")
        },
        Action::Dismiss { actor, effect } => {
            let id = authorize(session, access, *actor)?;
            effects::dismiss(map, *effect, id, clock(session))?;
            format!("{actor} ended concentration on effect {effect}")
        },
        Action::Calibrate => {
            dm(access)?;
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Scale {
                cells_per_square: u32,
            }
            let scale: Scale = serde_json::from_slice(payload)
                .map_err(|_| "enter native cells per five-foot square")?;
            if scale.cells_per_square == 0 {
                return Err("physical scale must be positive".into());
            }
            state.cells_per_square = Some(scale.cells_per_square);
            format!(
                "DM calibrated {} native cells per five-foot square",
                scale.cells_per_square
            )
        },
        Action::UseRulings => {
            dm(access)?;
            state.cells_per_square = None;
            "DM selected explicit reach rulings".into()
        },
        Action::Choose { .. } | Action::Undo(_) | Action::Finish(_) | Action::Discard(_) => {
            return Err("creation drafts belong to the Pathfinder 2e edition".into());
        },
    };
    Ok((note, rolls))
}
