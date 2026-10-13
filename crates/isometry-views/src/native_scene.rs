// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit local presentation over admitted native geometry (243, 799).

use cambium::{TextInput,caret_text_field,clickable,el,lens,text};
use crate::{UiChild,UiState};
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSceneInput {
    pub version:u32,pub site:u64,pub window:isocosm::operations::Window,
    /// Camera coordinates are in this explicitly supplied native Ground frame.
    pub centre:[f32;3],pub forward:[f32;3],pub half_height:f32,pub depth:f32,
    pub scale:f32,pub material:u8,pub played_tint:[f32;3],pub other_tint:[f32;3],
}

impl NativeSceneInput {
    pub fn validate(&self)->Result<(),String> {
        if self.version!=1 || self.window.depth<0 || self.window.min.iter().zip(self.window.max)
            .any(|(min,max)|*min>=max) || !self.scale.is_finite() || self.scale<=0.0
            || self.played_tint.iter().chain(&self.other_tint).any(|n|!n.is_finite() || !(0.0..=1.0).contains(n))
            || isometer::SlabCamera::new(self.centre,self.forward,self.half_height,1.0,self.depth).is_none() {
            return Err("native scene needs a complete finite authored window and presentation".into());
        }
        Ok(())
    }
    pub(crate) fn camera(&self,aspect:f32)->Result<isometer::SlabCamera,String> {
        isometer::SlabCamera::new(self.centre,self.forward,self.half_height,aspect,self.depth)
            .ok_or("native scene camera cannot frame this pane".into())
    }
}

#[derive(Clone,Copy,Debug)]
pub enum SceneFileAction { Load,Clear }

#[derive(Clone,Debug,Default)]
pub struct NativeSceneUi {
    pub path:TextInput,pub request:Option<SceneFileAction>,pub input:Option<NativeSceneInput>,
    /// Supplied only by the real local/session actor, never table names.
    pub sender:Option<isonetry::native_table::TableSender>,
    /// This local source was enabled by native mode, rather than the scene flag.
    pub automatic:bool,
}

pub(crate) fn controls(ui:&UiState)->Vec<UiChild> {
    if ui.native.world.is_none() {return Vec::new();}
    let mut rows:Vec<UiChild>=vec![Box::new(el("h4",text("Native physical picture")))];
    rows.push(Box::new(el("div",(text("Authored picture JSON file "),lens(
        |field:&mut TextInput|caret_text_field(field,&[]),|ui:&mut UiState|&mut ui.native.scene.path
    ))).attr("class","sheet-row")));
    rows.push(Box::new(el("div",(
        clickable(el("span",text("Load picture input")).attr("class","btn"),
            |ui:&mut UiState,_|ui.native.scene.request=Some(SceneFileAction::Load)),
        clickable(el("span",text("Withdraw picture")).attr("class","btn"),
            |ui:&mut UiState,_|ui.native.scene.request=Some(SceneFileAction::Clear)),
    )).attr("class","btn-row")));
    rows.push(Box::new(el("div",text("Bodies, terrain and picks use current native bindings. Missing inputs withdraw the physical picture."))
        .attr("class","side-hint")));
    rows
}
