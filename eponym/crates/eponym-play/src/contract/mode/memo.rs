// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A window's realized volume and places, kept by terrain revision (case
//! 1113). The revision digests the world, its sites and every edit, so a
//! kept derivation is the one a fresh derivation would give.

use cosmograph::{Simulation, operations::Window, schema::{Id, Key}};
use isometer_space::{places::{Places, Rules}, volume::Volume};
use std::cell::RefCell;
use std::rc::Rc;

type Frame = (Key, Id, [i64; 2], [i64; 2], i64);

thread_local! {
    static VOLUME: RefCell<Option<(Frame, Rc<Volume>)>> = const { RefCell::new(None) };
    static PLACES: RefCell<Option<((Frame, Rules), Rc<Places>)>> = const { RefCell::new(None) };
}

fn frame(sim: &Simulation, site: Id, w: &Window) -> Frame {
    (sim.terrain_revision(), site, w.min, w.max, w.depth)
}

/// The site's volume over `w`, as `Simulation::volume` realizes it.
pub(crate) fn volume(sim: &Simulation, site: Id, w: &Window) -> Result<Rc<Volume>, String> {
    let key = frame(sim, site, w);
    if let Some(v) = VOLUME.with_borrow(|m| m.as_ref().filter(|(k, _)| *k == key).map(|(_, v)| v.clone())) {
        return Ok(v);
    }
    let v = Rc::new(sim.volume(site, w.min, w.max, w.depth)?);
    VOLUME.set(Some((key, v.clone())));
    Ok(v)
}

/// The places `rules` derive over the site's volume over `w`.
pub(crate) fn places(sim: &Simulation, site: Id, w: &Window, rules: Rules) -> Result<Rc<Places>, String> {
    let key = (frame(sim, site, w), rules);
    if let Some(p) = PLACES.with_borrow(|m| m.as_ref().filter(|(k, _)| *k == key).map(|(_, p)| p.clone())) {
        return Ok(p);
    }
    let p = Rc::new(Places::derive(&*volume(sim, site, w)?, rules)?);
    PLACES.set(Some((key, p.clone())));
    Ok(p)
}
