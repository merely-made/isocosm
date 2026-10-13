// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{products::Mode,ui::model::{App,Defaults}};
use eponym_play::contract::mode::{BorrowedReading,ModeState};
use isocosm_overlay::Seat;

pub(super) fn refresh(app:&mut App) {
    for local in app.windows.values_mut() {
        if local.native.binding().mode != Mode::Eponym.key() {
            local.eponym_scene.clear();continue;
        }
        let result=(|| {
            let site=local.native.binding().attention.examined.ok_or("Eponym picture requires this window's admitted close-up")?.0;
            let cut=local.native.projection()?;
            let turn=serde_json::from_slice::<Defaults>(&local.native.binding().view_defaults)
                .map_err(|e|e.to_string())?.quarter_turn;
            app.world.read_game(Mode::Eponym,|session,bytes| {
                crate::spatial::region(session,&cut,local.native.handle()?,site)?;
                let state=ModeState::decode(bytes)?.ok_or("Eponym picture has no supplied product pose bindings")?;
                let reader=BorrowedReading::new(session,&cut.visible,&state)?;
                let mut windows=Vec::new();
                for binding in state.bindings.iter().filter(|b|b.site==site && cut.visible.shows(b.entity)) {
                    binding.current(session)?;
                    if !windows.contains(&binding.window) {windows.push(binding.window.clone());}
                }
                let frame=match local.input.as_ref().filter(|input|input.mode==Mode::Eponym)
                    .and_then(|input|input.region.as_ref()) {
                    Some(region) if region.site==site=>isocosm::operations::Window {min:region.min,max:region.max,depth:region.depth},
                    Some(_)=>return Err("Eponym picture frame belongs to another site".into()),
                    None if windows.len()==1=>windows[0].clone(),
                    None=>return Err("Eponym bindings require a single window or explicit site-region".into()),
                };
                if cut.access.seat==Seat::Player {reader.played(cut.access.participant.0)?;}
                let picture=reader.picture(site,&frame,cut.access.participant.0,cut.access.seat==Seat::Player)?;
                local.eponym_scene.update(picture,turn)
            })
        })();
        if let Err(why)=result {local.eponym_scene.clear();local.error=Some(why);}
    }
}
