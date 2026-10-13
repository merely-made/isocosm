// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isometry_views::{NativeSceneInput,SceneFileAction};
use isonetry::native_table::TableSender;

impl App {
    /// Read supplied view data only. Native authority stays in its existing actor.
    pub(crate) fn pump_native_scene(&mut self,ctx:&mut Ctx<'_>) {
        let sender=match &self.net {
            Some(net)=>net.local_sender(),
            None if ctx.runner.state().net_mode==NetMode::Local=>Some(TableSender::Host),
            None=>None,
        };
        ctx.runner.update(|ui| {
            ui.native.scene.sender=sender;
            match ui.native.scene.request.take() {
                Some(SceneFileAction::Clear)=>{ui.native.scene.input=None;ui.status="withdrew native picture".into();},
                Some(SceneFileAction::Load)=>{
                    let result=(|| {
                        let path=ui.native.scene.path.text().trim();
                        if path.is_empty() {return Err("choose a complete native picture input file".to_string());}
                        let bytes=std::fs::read(path).map_err(|error|format!("native picture file: {error}"))?;
                        let input:NativeSceneInput=serde_json::from_slice(&bytes).map_err(|error|error.to_string())?;
                        input.validate()?;
                        Ok(input)
                    })();
                    // A changed or missing file never keeps an earlier physical input live.
                    ui.native.scene.input=None;
                    match result {
                        Ok(input)=>{ui.native.scene.input=Some(input);ui.status="loaded authored native picture input".into();},
                        Err(why)=>ui.status=why,
                    }
                },
                None=>{},
            }
        });
        if ctx.runner.state().native.world.is_some() {
            if self.scene_board.is_none() {
                let mut next=None;
                ctx.runner.update(|ui| {
                    ui.scene_board=true;ui.native.scene.automatic=true;
                    let board=crate::scene_board::SceneBoard::new(ui);
                    ui.board_pick=Some(board.pick());next=Some(board);
                });
                self.scene_board=next;
            } else {
                ctx.runner.update(|ui|ui.scene_board=true);
            }
        } else if ctx.runner.state().native.scene.automatic {
            // Restore the original sim-off DOM path when native mode alone
            // created this source. An explicit scene flag keeps its own source.
            self.scene_board=None;
            ctx.runner.update(|ui| {
                ui.scene_board=false;ui.board_pick=None;ui.native.scene.automatic=false;
            });
        }
    }
}
