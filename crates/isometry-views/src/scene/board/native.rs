// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A derived physical snapshot. It owns no Session, clocks or admission door.

use crate::{UiState,NativeSceneInput};
use isocosm::schema::{BodyDocument,Id};
use isocosm_overlay::Seat;
use isometer::{Pose,core::Yaw,space::volume::Volume};
use isometry_core::{TokenId,TileCoord};
use std::{collections::{BTreeMap,BTreeSet},rc::Rc};

mod source;

#[derive(Default)]
pub(super) struct Cache {
    pub(super) terrain:Option<String>,pub(super) presented:Option<Rc<Picture>>,pub(super) error:Option<String>,
    blank:Option<(wgpu::Texture,wgpu::TextureView,[u32;2])>,
}

pub(in crate::scene) struct Body {pub entity:Id,pub document:BodyDocument,pub revision:u64,pub pose:Pose,pub played:bool}
pub(in crate::scene) struct Cell {pub at:[i64;3],pub tile:TileCoord,pub elevation:i32}
pub(in crate::scene) struct Picture {
    pub volume:Volume,pub input:NativeSceneInput,pub bodies:Vec<Body>,
    pub tokens:BTreeMap<Id,TokenId>,pub cells:Vec<Cell>,pub origin:[i64;3],pub fingerprint:String,
}

pub(in crate::scene) fn read(ui:&UiState)->Result<Rc<Picture>,String> {
    let input=ui.native.scene.input.as_ref().ok_or("native scene has no supplied picture input")?;
    input.validate()?;
    let sender=ui.native.scene.sender.ok_or("native scene has no current authenticated local actor")?;
    let world=ui.native.world.as_ref().ok_or("native scene has no accepted native world")?;
    let table=ui.native.table.as_ref().ok_or("native scene has no explicit table bindings")?;
    let campaign=world.load(isocosm::Execution::Grouped)?;
    let session=campaign.table_session();
    table.validate_history(session)?;
    let participant=table.participant(sender)?;
    let played=session.sim.plays(participant);
    let seat=if matches!(sender,isonetry::native_table::TableSender::Host) {Seat::Dm} else {Seat::Player};
    let mode=if seat==Seat::Player {isocosm::directing::readings::Mode::Survival}
        else {isocosm::directing::readings::Mode::Creative};
    let visible=isocosm::directing::readings::view(&session.sim,played.unwrap_or(u64::MAX),mode);
    if !visible.sites.contains(&input.site) {return Err("native scene site is outside actual native awareness".into());}
    let game=isometry_campaign::native::GameBindings::decode(&table.game)?;
    game.validate(session)?;
    let active=ui.active_map.as_ref().ok_or("native scene needs an explicitly active table map")?;
    let mut matching=table.boards.iter().filter(|board|&board.map==active);
    let board=matching.next().ok_or("native scene map has no authored bindings")?;
    if matching.next().is_some() || board.ground_digest!=isonetry::native_table::ground_digest(&ui.map) {
        return Err("native scene table mapping is ambiguous or stale".into());
    }
    let volume=session.sim.volume(input.site,input.window.min,input.window.max,input.window.depth)?;
    let half=(volume.max[0]-volume.min[0]).max(volume.max[1]-volume.min[1])/2;
    let origin=[volume.min[0].checked_add(half).ok_or("native frame x overflows")?,volume.datum,
        volume.min[1].checked_add(half).ok_or("native frame z overflows")?];
    let mut roots=BTreeMap::new();
    for (key,character) in game.characters() {
        if !visible.shows(character.entity) || !session.sim.state().population.get(character.entity)
            .is_some_and(|body|body.resident_site()==Some(input.site)) {continue;}
        let binding=game.positions().get(key).ok_or("visible native character has no admitted physical position")?;
        // PositionBinding has no orientation grammar: its saved body is unrotated.
        roots.insert(character.entity,(binding.current(session)?,Yaw::Zero));
    }
    let mut items=BTreeSet::new();
    for binding in game.item_records().iter().rev().filter_map(|record|record.binding.as_ref()) {
        if !items.insert(binding.entity) || !visible.shows(binding.entity) || !session.sim.state().population
            .get(binding.entity).is_some_and(|body|body.resident_site()==Some(input.site)) {continue;}
        let pose=isometry_campaign::native::items::current_item(session,&game,binding.entity)?;
        if roots.get(&binding.entity).is_some_and(|at|*at!=(pose.at,pose.yaw)) {
            return Err("native item and character placements disagree on the same entity".into());
        }
        roots.insert(binding.entity,(pose.at,pose.yaw));
    }
    let mut bodies=Vec::new();
    for (entity,(at,yaw)) in roots {
        if !volume.holds(at[0],at[2]) || at[1]<volume.datum || at[1]>=volume.ceiling() {continue;}
        let body=session.sim.state().population.get(entity).ok_or("native scene body is absent")?;
        let document=body.body.clone().ok_or("native scene has no actual body geometry")?;
        let m=volume.materials;
        if !volume.material(at).is_some_and(|material|material==m.air || material==m.water) {
            return Err("native scene root is not in the current clear volume".into());
        }
        bodies.push(Body {entity,revision:body.body_revision,document,pose:root_pose(at,yaw,origin)?,played:played==Some(entity)});
    }
    if bodies.is_empty() || seat==Seat::Player && played.is_none_or(|entity|!bodies.iter().any(|body|body.entity==entity)) {
        return Err("native scene lacks its current admitted bodies or actual played subject".into());
    }
    let shown:BTreeSet<_>=bodies.iter().map(|body|body.entity).collect();
    let mut tokens=BTreeMap::new();let mut token_ids=BTreeSet::new();
    for binding in &board.tokens {
        if !token_ids.insert(binding.token) {return Err("native scene token mapping repeats".into());}
        let character=game.characters().get(&binding.character).ok_or("native scene token has no admitted character")?;
        if !shown.contains(&character.entity) {continue;}
        let token=ui.map.token(binding.token).ok_or("native scene binding names no current token")?;
        let position=game.positions().get(&binding.character).ok_or("native scene token has no current native pose")?;
        let cell=board.cells.iter().find(|cell|cell.tile==token.at).ok_or("native scene token tile has no authored native cell")?;
        if cell.to.site.0!=input.site || cell.to.at.0.map(i64::from)!=position.current(session)?
            || tokens.insert(character.entity,binding.token).is_some() {
            return Err("native scene token mapping differs from its actual current body or is ambiguous".into());
        }
    }
    let mut cells=Vec::new();let mut tiles=BTreeSet::new();let mut coordinates=BTreeSet::new();
    for cell in &board.cells {
        if cell.to.site.0!=input.site {continue;}
        let at=cell.to.at.0.map(i64::from);
        if !tiles.insert(cell.tile) || !coordinates.insert(at) || !ui.map.ground.in_bounds(cell.tile.0,cell.tile.1) {
            return Err("native scene tile/cell mapping is ambiguous or outside the active board".into());
        }
        if !volume.holds(at[0],at[2]) || at[1]<volume.datum || at[1]>=volume.ceiling() {continue;}
        let elevation=*ui.map.elevation.get(cell.tile.0 as u32,cell.tile.1 as u32).ok_or("mapped tile elevation absent")?;
        cells.push(Cell {at,tile:cell.tile,elevation:i32::from(elevation)});
    }
    let fingerprint=isocosm::digest(&(world.cursor()?,table,&ui.map,input,participant,seat,&visible));
    Ok(Rc::new(Picture {volume,input:input.clone(),bodies,tokens,cells,origin,fingerprint}))
}

fn root_pose(at:[i64;3],yaw:Yaw,origin:[i64;3])->Result<Pose,String> {
    let mut position=[0.0;3];
    for axis in 0..3 {position[axis]=at[axis].checked_sub(origin[axis]).ok_or("native root frame overflows")? as f32;}
    let yaw_radians=match yaw {Yaw::Zero=>0.0,Yaw::Quarter=>std::f32::consts::FRAC_PI_2,
        Yaw::Half=>std::f32::consts::PI,Yaw::ThreeQuarter=>-std::f32::consts::FRAC_PI_2};
    Ok(Pose {position,yaw_radians})
}
