//! Character sheets rendered by the same endpoint as table time and party.
use super::{Act, Built, Cx, VttEndpoint};
use crate::ruleset::{self, Action, State};
use chirograph::IntentEffect;
use isocosm_host::endpoint::disclosure::{Item, NULL, action, card, value};
use isocosm_overlay::Seat;
use isometry_campaign::EquipmentSlot;
use isometry_core::FieldValue;
use isometry_system::active_effects::{self as effects, Records};

fn push(
    built: &mut Built,
    id: String,
    title: &str,
    values: Vec<chirograph::CardValueV1>,
    choices: Vec<(String, String, Action)>,
) {
    let actions = choices
        .iter()
        .map(|(id, label, choice)| {
            action(
                id,
                label,
                match choice {
                    Action::Cast { .. } => {
                        "Spend the chosen spell slot and concentrate for up to one hour."
                    },
                    Action::RuleAttack { .. } => {
                        "You rule that the target is within five feet with a clear path."
                    },
                    Action::Damage { .. } => "Apply damage and check concentration when required.",
                    _ => "",
                },
                NULL,
                IntentEffect::DomainTruth,
            )
        })
        .collect();
    built.push(
        Item {
            id,
            card: card(title, values, &["experimental"]),
            actions,
        },
        choices
            .into_iter()
            .map(|(_, _, act)| Act::Ruleset(act))
            .collect(),
    );
}

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let access = cx.cut.access;
    endpoint.world.read_sections(|session,sections| {
        let state=State::read(sections)?.unwrap_or_else(||State::new(session.sim.genesis().seed));
        let map=state.current_map(session)?;
        let pathfinder=super::sheets_pf2e::plays(sections)?;
        let mut built=Built::new();
        let native=session.sim.state();
        let mut bodies:std::collections::BTreeSet<u64>=state.sheets.keys().map(|id|u64::from(id.0)).collect();
        bodies.extend(native.population.groups.iter().filter(|(_,g)|isocosm::directing::is_participant(&g.entity))
            .filter_map(|(id,_)|session.sim.plays(*id)));
        bodies.extend(native.population.groups.iter().filter(|(_,g)|g.entity.authored.is_some() && g.entity.alive).map(|(id,_)|*id));
        bodies.retain(|id|cx.cut.visible.shows(*id));
        let visible=bodies.clone();
        for id in bodies {
            if access.seat!=Seat::Dm && access.subject!=Some(isocosm_overlay::EntityHandle(id)) {continue;}
            let tid=ruleset::token(id)?;
            let title=native.population.get(id).and_then(|b|b.authored.as_ref()).map_or_else(||format!("Character {id}"),|a|a.key.clone());
            let can_control=ruleset::controls(session,access,id);
            let dm=access.seat==Seat::Dm;
            if pathfinder {
                super::sheets_pf2e::body(&mut built,&state,&map,id,tid,&title,can_control)?;
                continue;
            }
            let Some(sheet)=map.sheet(tid) else {
                let choices=if dm && can_control {vec![("vtt.sheet-create".into(),"Create D&D SRD 5.1 sheet".into(),Action::Create(id))]} else {vec![]};
                push(&mut built,format!("sheet:{id}"),&title,vec![value("Native body",id),value("Sheet","not configured")],choices);
                continue;
            };
            let mut choices=Vec::new();
            let mut values=vec![value("Native body",id),value("Ruleset","D&D SRD 5.1; experimental"),
                value("Table HP",sheet.int("hp_current").unwrap_or(0)),value("AC",sheet.int("ac").unwrap_or(0)),
                value("Strength",sheet.int("str").unwrap_or(10)),value("Constitution",sheet.int("con").unwrap_or(10)),
                value("Constitution save bonus",sheet.int("con_save_bonus").unwrap_or(0))];
            let known=sheet.fields.get("known:dnd5:magic-weapon")==Some(&FieldValue::Bool(true));
            values.push(value("Magic Weapon",if known {"known"} else {"not granted"}));
            if dm && can_control {
                if !known {choices.push(("vtt.sheet-learn".into(),"Grant Magic Weapon".into(),Action::Learn(id)));}
                for template in ["longsword","club"] {choices.push((format!("vtt.sheet-give-{template}"),format!("Add {template} to sheet"),Action::Give{actor:id,template:template.into()}));}
                for (field,label) in [("hp_current","HP"),("hp_max","Maximum HP"),("str","Strength"),("con","Constitution"),("ac","Armor class"),("con_save_bonus","Constitution save")] {
                    for delta in [-1,1] {
                        if field=="hp_current" && delta<0 {continue;}
                        let label=if field=="hp_current" {"Restore 1 HP".into()} else {format!("{label} {delta:+}")};
                        choices.push((format!("vtt.sheet-{field}-{delta}"),label,Action::Adjust{actor:id,field:field.into(),delta}));
                    }
                }
                for rank in 3..=9 {if sheet.int(&format!("slot:{rank}")).unwrap_or(0)==0 {choices.push((format!("vtt.slot-{rank}-1"),format!("Add level {rank} spell slot"),Action::Adjust{actor:id,field:format!("slot:{rank}"),delta:1}));}}
                for amount in [1,5,10] {choices.push((format!("vtt.sheet-damage-{amount}"),format!("Apply {amount} external damage"),Action::Damage{target:id,amount}));}
            }
            push(&mut built,format!("sheet:{id}"),&title,values,choices);
            for rank in 2..=9u8 {
                let left=sheet.int(&format!("slot:{rank}")).unwrap_or(0);
                if left==0 && (!dm || rank!=2) {continue;}
                let choices=if dm && can_control {[-1,1].into_iter().filter(|delta|*delta>0||left>0).map(|delta|
                    (format!("vtt.slot-{rank}-{delta}"),format!("Level {rank} slot {delta:+}"),Action::Adjust {actor:id,field:format!("slot:{rank}"),delta})).collect()} else {vec![]};
                push(&mut built,format!("slots:{id}:{rank}"),&format!("{title}: spell slots"),vec![value("Level",rank),value("Remaining",left)],choices);
            }
            if let Some(inventory)=state.inventories.get(&tid) {
                for (item_id,item) in &inventory.items {
                    let equipped=inventory.equipped.get(&EquipmentSlot::MainHand)==Some(item_id);
                    let change=effects::weapon(&map,item_id)?;
                    let mut choices=Vec::new();
                    if can_control && sheet.int("hp_current").unwrap_or(0)>0 {
                        if !equipped {choices.push(("vtt.weapon-equip".into(),"Wield this weapon".into(),Action::Equip{actor:id,item:item_id.clone()}));}
                        else {
                            if known && !change.magical {
                                for rank in 2..=9u8 {if sheet.int(&format!("slot:{rank}")).unwrap_or(0)>0 {
                                    choices.push((format!("vtt.weapon-cast-{rank}"),format!("Cast Magic Weapon: level {rank} slot"),Action::Cast{actor:id,item:item_id.clone(),rank}));
                                }}
                            }
                            for target in visible.iter().copied().filter(|target|*target!=id && map.sheet(ruleset::token(*target).unwrap()).is_some_and(|s|s.int("hp_current").unwrap_or(0)>0)
                                && native.population.get(id).and_then(|b|b.resident_site()).is_some_and(|site|native.population.get(*target).and_then(|b|b.resident_site())==Some(site))) {
                                if state.cells_per_square.is_some() {
                                    if ruleset::measured_reach(&state,session,sections,access,id,target).is_ok() {choices.push((format!("vtt.weapon-attack-{target}"),format!("Attack character {target}"),Action::Attack{actor:id,target}));}
                                }
                                else if dm {choices.push((format!("vtt.weapon-rule-attack-{target}"),format!("Rule within reach and attack {target}"),Action::RuleAttack{actor:id,target}));}
                            }
                        }
                    }
                    push(&mut built,format!("weapon:{id}:{}",item_id.0),&format!("{title}: {}",item.name),vec![value("Source","Table equipment"),
                        value("Wielded",equipped),value("Magical",change.magical),value("Magic attack bonus",change.attack_bonus),value("Magic damage bonus",change.damage_bonus)],choices);
                }
            }
        }
        if let Some(records)=Records::read(&map)? {
            for effect in records.effects.values() {
                let caster=u64::from(effect.caster.0);
                if access.seat!=Seat::Dm && access.subject!=Some(isocosm_overlay::EntityHandle(caster)) {continue;}
                let running=effect.running(records.tick);
                let choices=if running && ruleset::controls(session,access,caster) {vec![("vtt.effect-dismiss".into(),"End concentration".into(),Action::Dismiss{actor:caster,effect:effect.id})]} else {vec![]};
                let target=match &effect.target {effects::Target::Item(item)=>state.inventories.values().find_map(|inv|inv.items.get(item)).map_or_else(||"Weapon no longer present".into(),|item|item.name.clone()),
                    effects::Target::Creature(id)=>format!("Character {}",id.0)};
                let ended=effect.ended.as_ref().map(|(_,why)|match why {
                    effects::End::Expired=>"Duration elapsed".into(),effects::End::Dismissed=>"Caster ended concentration".into(),
                    effects::End::Concentration=>"Concentration lost".into(),effects::End::Incapacitated=>"Caster incapacitated".into(),
                    effects::End::Dispelled{ruling}=>format!("Dispelled: {ruling}"),effects::End::Replaced{ruling}=>format!("Replaced: {ruling}")}).unwrap_or_else(||"Still running".into());
                push(&mut built,format!("effect:{}",effect.id),"Magic Weapon",vec![value("Caster",format!("Character {caster}")),value("Target",target),
                    value("State",if effect.applies(records.tick) {"active"} else if running {"suppressed"} else {"ended"}),
                    value("Seconds remaining",if running {effect.expires.saturating_sub(records.tick).saturating_mul(records.tick_microseconds)/1_000_000} else {0}),
                    value("End cause",ended)],choices);
                if running && access.seat==Seat::Dm {
                    let game=crate::native::GameBindings::decode(sections.get("vtt").map_or(&[],Vec::as_slice))?;
                    if game.shared_players().is_some() {
                        built.push(Item {id:format!("effect-time:{}",effect.id),card:card("Effect and table time",vec![value("Remaining native ticks",effect.expires-records.tick)],&["time"]),
                            actions:vec![action("vtt.effect-propose-expiry","Propose downtime until this effect ends","Propose elapsed time through the table's ordinary consent path.",NULL,IntentEffect::DomainTruth)]},
                            vec![Act::Send(isocosm_overlay::vtt::VttIntent::Time(isocosm_overlay::vtt::TimeIntent::Downtime{ticks:effect.expires-records.tick}))]);
                    }
                }
            }
        }
        for (index,entry) in state.log.iter().enumerate().rev().take(8) {
            if access.seat!=Seat::Dm && !entry_mentions(entry,access.subject.map(|s|s.0)) {continue;}
            let mut values=vec![value("Native tick",entry.tick),value("Result",&entry.note)];
            values.extend(entry.rolls.iter().map(|roll|value(&roll.by,format!("{} = {}; dice {:?}",roll.expr,roll.total,roll.dice))));
            push(&mut built,format!("ruleset-log:{index}"),"Table result",values,vec![]);
        }
        push(&mut built,"ruleset-scale".into(),"Physical distance",vec![value("Native cells per five-foot square",state.cells_per_square.map_or_else(||"not configured".into(),|scale|scale.to_string())),
            value("Distance source",if state.cells_per_square.is_none() {"Your recorded reach rulings. Physical scale can be set in Native operation authoring."}
                else {"Current native positions and terrain. Unplaced, blocked or distant targets are omitted."})],vec![]);
        Ok(built)
    })?
}

fn entry_mentions(entry: &ruleset::Log, subject: Option<u64>) -> bool {
    match (&entry.action, subject) {
        (
            Action::Cast { actor, .. }
            | Action::Equip { actor, .. }
            | Action::Dismiss { actor, .. },
            Some(id),
        ) => *actor == id,
        (Action::Attack { actor, target } | Action::RuleAttack { actor, target }, Some(id)) => {
            *actor == id || *target == id
        },
        (Action::Damage { target, .. }, Some(id)) => *target == id,
        (
            Action::Create(actor)
            | Action::Choose { actor, .. }
            | Action::Undo(actor)
            | Action::Finish(actor)
            | Action::Discard(actor),
            Some(id),
        ) => *actor == id,
        _ => false,
    }
}
