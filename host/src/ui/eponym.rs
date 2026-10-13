// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Existing native social successors at the actual paired product cut.

use cambium::{el,text};
use isocosm_overlay::Seat;
use super::{model::App,view::{Child,button}};
use crate::products::{Input,Mode};
use eponym_play::contract::mode::life;

impl App {
    fn eponym_life(&mut self,key:u64,input:life::Input)->Result<(),String> {
        let local=self.windows.get(&key).ok_or("window is closed")?;
        let request=Input {mode:Mode::Eponym,bytes:serde_json::to_vec(&input)
            .map_err(|error|error.to_string())?,round:0,region:None};
        local.native.submit_product(&request)?;
        self.refresh(); Ok(())
    }
}

pub(super) fn panel(app:&App,key:u64)->Child {
    let awaiting=app.windows.get(&key).and_then(|local|local.native.projection().ok())
        .is_some_and(|cut|app.world.read(|session|life::previous_take(session,cut.access.participant.0)
            .and_then(|entry|match &entry.command {isocosm::Command::Take {critter,..}=>
                session.sim.state().population.get(*critter).map(|body|!body.alive),_=>None})
            .unwrap_or(false)).unwrap_or(false));
    if !awaiting {return Box::new(el("div",Vec::<Child>::new()));}
    let mut rows:Vec<Child>=vec![Box::new(el("h2",text("Next life")))];
    let result=(|| {
        let cut=app.windows.get(&key).ok_or("window is closed")?.native.projection()?;
        if cut.access.seat!=Seat::Player {return Err("The player chooses their next life.".into());}
        app.world.read_game(Mode::Eponym,|session,game|
            life::death_choices(session,game,cut.access,&cut.visible))
    })();
    match result {
        Ok(choices)=>{
            if choices.is_empty() {rows.push(Box::new(el("p",text("No eligible next life is visible in this window."))));}
            for input in choices {
                let id=format!("eponym-life-{}",input.subject.0);
                let label=format!("Take subject {}",input.subject.0);
                rows.push(button(id,label,key,move |app|app.eponym_life(key,input.clone())));
            }
        },
        Err(reason)=>rows.push(Box::new(el("p",text(reason)))),
    }
    Box::new(el("section",rows).attr("class","panel"))
}
