// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The existing Eponym renderer over a derived picture, never an owning Session.

use std::{cell::RefCell, rc::Rc};
use eponym_play::contract::mode::Picture;
use isometer::SlabCamera;
use crate::producer::CameraPolicy;
mod source;
pub use source::Producer;

pub(super) struct Snapshot { pub picture: Picture, pub turn: u8 }

impl Snapshot {
    fn camera(&self, aspect: f32) -> Result<SlabCamera,String> {
        let p = &self.picture.presentation;
        let rotate = |[x,y,z]| match self.turn {
            0 => [x,y,z], 1 => [-z,y,x], 2 => [-x,y,-z], _ => [z,y,-x],
        };
        CameraPolicy {forward:rotate(p.forward),half_height:p.half_height,depth:p.depth,pan:rotate(p.pan)}
            .camera(p.centre,aspect).ok_or("invalid supplied Eponym camera".into())
    }
}

#[derive(Clone,Default)]
pub struct SceneHandle {
    pub(super) input: Rc<RefCell<Option<Rc<Snapshot>>>>,
    pub(super) error: Rc<RefCell<Option<String>>>,
}

impl SceneHandle {
    pub fn available(&self) -> bool {self.input.borrow().is_some()}
    pub fn last_error(&self) -> Option<String> {self.error.borrow().clone()}
    pub fn clear(&self) {self.input.borrow_mut().take();self.error.borrow_mut().take();}
    pub fn update(&self,picture:Picture,turn:u8) -> Result<(),String> {
        if turn > 3 {self.clear();return Err("quarter turn must be zero through three".into());}
        if let Err(why)=picture.presentation.validate() {self.clear();return Err(why);}
        *self.input.borrow_mut() = Some(Rc::new(Snapshot {picture,turn}));
        self.error.borrow_mut().take();
        Ok(())
    }
}
