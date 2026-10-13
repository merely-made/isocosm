// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Deliberate single-record learning and sharing at the actual player's cut.

use cambium::{el,text};
use isocosm_overlay::{EntityHandle,Seat,vtt::{PartyAction,PartyIntent,VttIntent}};
use super::{model::{App,Defaults},view::{Child,button}};
use crate::products::Mode;

pub(super) fn panel(app:&App,key:u64)->Child {
    let mut rows:Vec<Child>=vec![Box::new(el("h3",text("Held map knowledge")))];
    let result=(|| {
        let local=app.windows.get(&key).ok_or("window is closed")?;
        let access=local.native.projection()?.access;
        if access.seat!=Seat::Player {return Err("The player's body learns and discloses these records.".into());}
        let defaults:Defaults=serde_json::from_slice(&local.native.binding().view_defaults)
            .map_err(|error|error.to_string())?;
        let party=defaults.party.ok_or("Choose an authored party to share map knowledge.")?;
        app.world.read_game(Mode::Vtt,|session,game| {
            let reading=isometry_campaign::native::party::read_knowledge(session,game,access)?
                .ok_or("No currently played body holds these records.")?;
            Ok((party,reading))
        })
    })();
    match result {
        Err(reason)=>rows.push(Box::new(el("p",text(reason)))),
        Ok((party,reading))=>{
            for (index,(source,statement,revision)) in reading.sources.iter().enumerate() {
                rows.push(Box::new(el("p",text(format!("{statement:?} · source {} · revision {}",
                    revision.note.core.object,revision.revision)))));
                let intent=PartyIntent {party:party.clone(),member:EntityHandle(reading.holder),
                    action:PartyAction::ObserveStatement {source_note:source.clone()}};
                rows.push(button(format!("learn-map-{index}"),"Learn this statement".into(),key,
                    move |app|app.vtt_send(key,VttIntent::Party(intent.clone()))));
            }
            match isometry_campaign::native::party::detail_keys(&reading.held) {
                Err(reason)=>rows.push(Box::new(el("p",text(reason)))),
                Ok(keys)=>for (index,held) in keys.into_iter().enumerate() {
                    rows.push(Box::new(el("p",text(format!("Held record: {held}")))));
                    let intent=PartyIntent {party:party.clone(),member:EntityHandle(reading.holder),
                        action:PartyAction::DiscloseDetails {sites:Vec::new(),events:Vec::new(),held:vec![held]}};
                    rows.push(button(format!("share-map-{index}"),"Disclose this record to the party".into(),key,
                        move |app|app.vtt_send(key,VttIntent::Party(intent.clone()))));
                },
            }
        },
    }
    Box::new(el("section",rows).attr("class","panel"))
}
