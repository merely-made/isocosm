// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Terrain residency grain changes neither body scale nor query coordinates.
use super::*;

pub(super) fn camera(c:SlabCamera, cell:f32) -> SlabCamera {
    use crate::camera::Cutaway;
    SlabCamera { centre:c.centre.map(|v| v/cell),half_height:c.half_height/cell,depth:c.depth/cell,
        cutaway:c.cutaway.map(|cut| match cut {
            Cutaway::Bounds {min,max} => Cutaway::Bounds {min:min.map(|v| v/cell),max:max.map(|v| v/cell)},
            Cutaway::Plane {normal,distance} => Cutaway::Plane {normal,distance:distance/cell},
        }),..c }
}
pub(super) fn pose(p:&CritterPose,cell:f32) -> CritterPose {
    let mut out=p.clone();
    out.bounds_centre=out.bounds_centre.map(|v| v/cell); out.bounds_radius/=cell;
    for eye in &mut out.eyes { for v in eye { *v/=cell; } }
    for capsule in &mut out.capsules {
        capsule.a=capsule.a.map(|v| v/cell); capsule.b=capsule.b.map(|v| v/cell);
        capsule.ra/=cell; capsule.rb/=cell;
    }
    out
}
pub(super) fn appearance(a:Option<TerrainAppearance>,cell:f32) -> Option<TerrainAppearance> {
    a.map(|mut a| { a.section_centre=a.section_centre.map(|v| v/cell); a.clearing_y/=cell; a })
}
