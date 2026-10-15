// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Supplied native operations on the actual shared VTT world.

use crate::mode_sections::Games as _;
use cambium::{el,text,lens,caret_text_field,TextInput};
use isocosm_overlay::Seat;
use isometry_campaign::native::{institutions,physical_map,items,graft};
use super::{model::App,view::{Child,button}};
use crate::products::Mode;

impl App {
    fn author_input(&self,key:u64)->Result<Vec<u8>,String> {
        Ok(self.windows.get(&key).ok_or("window is closed")?.authored_input.text().as_bytes().to_vec())
    }
    fn quote_author_input(&mut self,key:u64,kind:u8)->Result<(),String> {
        let access=self.windows.get(&key).ok_or("window is closed")?.native.projection()?.access;
        let bytes=self.author_input(key)?;
        let quote=self.world.read_game(Mode::Vtt,|session,game| {
            if kind==1 {
                let input:physical_map::PhysicalMapInput=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                serde_json::to_string_pretty(&physical_map::prepare_physical_map(session,game,access,&input)?)
                    .map_err(|e|e.to_string())
            } else if kind==2 {
                let input:items::ItemInput=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                serde_json::to_string_pretty(&items::prepare_item(session,game,access,&input)?).map_err(|e|e.to_string())
            } else if kind==3 {
                let input:graft::GraftInput=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                serde_json::to_string_pretty(&graft::prepare_graft(session,access,&input)?).map_err(|e|e.to_string())
            } else if kind==4 {
                let input:isometry_campaign::native::party::StatementInput=
                    serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                serde_json::to_string_pretty(&isometry_campaign::native::party::statement_admission(
                    session,game,access,&input)?).map_err(|e|e.to_string())
            } else if kind==5 {
                let input:isometry_campaign::native::party::LocationInput=
                    serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                let mut candidate=session.clone();
                let (outcome,_)=isometry_campaign::native::party::submit_location(
                    &mut candidate,game.to_vec(),access,&input)?;
                Ok(outcome)
            } else {
                let input:institutions::InstitutionInput=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                serde_json::to_string_pretty(&institutions::prepare_institution(session,access,&input)?)
                    .map_err(|e|e.to_string())
            }
        })?;
        self.windows.get_mut(&key).ok_or("window is closed")?.author_quote=Some(quote);
        Ok(())
    }
}

pub(super) fn panel(app:&App,key:u64)->Child {
    let mut rows:Vec<Child>=vec![Box::new(el("h3",text("Native operation authoring")))];
    let reading=(|| {
        let access=app.windows.get(&key).ok_or("window is closed")?.native.projection()?.access;
        if access.seat!=Seat::Dm {return Err("Native authoring context requires the DM seat.".into());}
        app.world.read_game(Mode::Vtt,|session,game| {
            let institutions=session.sim.state().polities.keys().map(|id|
                session.sim.institution_reading(isocosm::institutions::PolityId(*id))).collect::<Result<Vec<_>,_>>()?;
            serde_json::to_string_pretty(&serde_json::json!({
                "institution_source":institutions::source(session),"physical_source":physical_map::physical_source(session),
                "item_source":items::source(session),"graft_source":institutions::source(session),
                "game_source":isocosm::digest(&game),"terrain_revision":session.sim.terrain_revision(),
                "tick":session.sim.state().tick,"participant":access.participant.0,"played_body":access.subject,
                "institutions":institutions,
                "bindings":isometry_campaign::native::GameBindings::decode(game)?
            })).map_err(|e|e.to_string())
        })
    })();
    match reading {
        Ok(context)=>rows.push(Box::new(el("pre",text(context)))),
        Err(reason)=>{rows.push(Box::new(el("p",text(reason))));return Box::new(el("section",rows));}
    }
    rows.push(Box::new(el("label",(text("Supplied operation JSON "),lens(
        |field:&mut TextInput|caret_text_field(field,&[]),move |app:&mut App|
            &mut app.windows.get_mut(&key).expect("the open window owns this field").authored_input)))));
    for (kind,label) in [(0,"Preview institution operation"),(1,"Preview physical map edit"),
        (2,"Preview item or project"),(3,"Preview graft carrying"),(4,"Preview knowledge statement"),
        (5,"Preview Location grouping")] {
        rows.push(button(format!("operation-preview-{kind}"),label.into(),key,
            move |app|app.quote_author_input(key,kind)));
    }
    rows.push(button("operation-apply".into(),"Apply supplied operation".into(),key,move |app| {
        let bytes=app.author_input(key)?; app.vtt_bytes(key,bytes)
    }));
    if let Some(quote)=app.windows.get(&key).and_then(|local|local.author_quote.as_ref()) {
        rows.push(Box::new(el("pre",text(quote.clone()))));
    }
    Box::new(el("section",rows))
}
