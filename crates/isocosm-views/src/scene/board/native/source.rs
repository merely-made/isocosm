// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use crate::scene::board::{BoardSource,BoardPick,PlainHost,BODY_BUDGET,board_grade};
use isometer::{BodySignature,Cutaway,DeclaredExtentVolumes,FrameRequest,GroundTerrain,Pick,SceneBody,
    SceneFrame,SceneSignature,SceneVolumes,SubjectKey,TerrainSource};

impl BoardSource {
    pub fn native_error(&self)->Option<&str> {self.native.error.as_deref()}

    pub(in crate::scene) fn native_inputs(&self,r:&FrameRequest<'_>)->Result<SceneSignature,String> {
        let view=self.view.borrow();
        let input=view.native.as_ref().ok_or("not a native picture")?;
        match input {
            Err(why)=>Ok(SceneSignature {size:r.size,render_scale:r.render_scale,
                host:fingerprint(&cosmograph::digest(why))?,..Default::default()}),
            Ok(picture)=>{
                let p=&picture.input;
                let bodies=picture.bodies.iter().map(|body|BodySignature {
                    subject:SubjectKey(body.entity),revision:body.revision,pose:body.pose,
                    scale:p.scale,grounded:false,tint:if body.played {p.played_tint}else{p.other_tint},
                    always_visible:body.played,
                }).collect();
                Ok(SceneSignature {size:r.size,render_scale:r.render_scale,camera:Some(camera(picture,r.aspect)?),
                    terrain_revision:Some(u64::from_str_radix(&picture.fingerprint[..16],16).map_err(|e|e.to_string())?),
                    bodies,host:fingerprint(&picture.fingerprint)?})
            },
        }
    }

    pub(in crate::scene) fn native_frame(&mut self,r:&FrameRequest<'_>)->Result<Option<wgpu::TextureView>,String> {
        self.native.presented=None;
        let input=self.view.borrow().native.as_ref().ok_or("not a native picture")?.clone();
        let result=input.and_then(|picture|self.draw_native(r,&picture).map(|view|(picture,view)));
        match result {
            Ok((picture,view))=>{self.native.error=None;self.native.presented=Some(picture);Ok(Some(view))},
            Err(why)=>{self.native.error=Some(why);Ok(Some(self.withdraw_native(r)))},
        }
    }

    fn draw_native(&mut self,r:&FrameRequest<'_>,picture:&Picture)->Result<wgpu::TextureView,String> {
        self.ensure_scene(r)?;
        let scene=self.scene.as_mut().ok_or("native scene is absent")?;
        if self.native.terrain.as_ref()!=Some(&picture.fingerprint) {
            let map=isometer::lens::BrickMap::from_ground(picture.volume.ground()).map_err(|e|e.to_string())?;
            scene.set_terrain_palette(None);
            scene.set_terrain_map(map);
            self.native.terrain=Some(picture.fingerprint.clone());
        }
        // A prior sim-off token's numeric key confers neither a native palette
        // nor geometry. This same Scene now consumes actual native documents.
        self.ground=None;self.palette_revision=None;self.palettes.clear();self.world=None;self.placeholders=0;
        let p=&picture.input;
        let volumes=DeclaredExtentVolumes::from_documents(picture.bodies.iter().map(|body|&body.document),p.material);
        let bodies:Vec<_>=picture.bodies.iter().map(|body|SceneBody {
            subject:SubjectKey(body.entity),document:&body.document,pose:body.pose,scale:p.scale,grounded:false,
            tint:if body.played {p.played_tint}else{p.other_tint},materials:&[],always_visible:body.played,
        }).collect();
        for body in &bodies {scene.bodies_mut().set_palette(body.subject,None);}
        let ground=GroundTerrain(picture.volume.ground());
        let mut encoder=r.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label:Some("native VTT physical scene")});
        scene.render(&mut encoder,SceneFrame {camera:camera(picture,r.aspect)?,bodies:&bodies,
            volumes:SceneVolumes::DeclaredSolid(&volumes),terrain:Some(&ground as &dyn TerrainSource),dirty:&[],
            grade:board_grade(),terrain_appearance:None,body_budget:BODY_BUDGET,capsules:None},&mut PlainHost)?;
        r.queue.submit(Some(encoder.finish()));
        Ok(scene.encoded_view().clone())
    }

    fn withdraw_native(&mut self,r:&FrameRequest<'_>)->wgpu::TextureView {
        let size=[r.size[0].max(1),r.size[1].max(1)];
        if self.native.blank.as_ref().is_none_or(|blank|blank.2!=size) {
            let texture=r.device.create_texture(&wgpu::TextureDescriptor {label:Some("withdrawn native VTT picture"),
                size:wgpu::Extent3d {width:size[0],height:size[1],depth_or_array_layers:1},mip_level_count:1,sample_count:1,
                dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,
                usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING,view_formats:&[]});
            let view=texture.create_view(&Default::default());self.native.blank=Some((texture,view,size));
        }
        let mut encoder=r.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label:Some("withdraw native VTT picture")});
        {
            let _pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor {label:Some("withdraw native VTT picture"),
                color_attachments:&[Some(wgpu::RenderPassColorAttachment {view:&self.native.blank.as_ref().unwrap().1,
                    depth_slice:None,resolve_target:None,ops:wgpu::Operations {load:wgpu::LoadOp::Clear(wgpu::Color::BLACK),store:wgpu::StoreOp::Store}})],
                depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None,multiview_mask:None});
        }
        r.queue.submit(Some(encoder.finish()));self.native.blank.as_ref().unwrap().1.clone()
    }

    pub(in crate::scene) fn native_pick(&self,ndc:[f32;2])->Option<BoardPick> {
        let picture=self.native.presented.as_ref()?;
        let view=self.view.borrow();
        let current=view.native.as_ref()?.as_ref().ok()?;
        if current.fingerprint!=picture.fingerprint {return None;}
        match self.scene.as_ref()?.pick(ndc).ok().flatten()? {
            Pick::Body(body)=>picture.tokens.get(&body.address.subject.0).copied().map(BoardPick::Token),
            Pick::Terrain(hit)=>{
                if hit.normal[1]<=0.5 {return None;}
                let mut at=[0;3];
                for axis in 0..3 {at[axis]=picture.origin[axis].checked_add(i64::from(hit.voxel[axis]))?;}
                at[1]=at[1].checked_add(1)?;
                // Only the exact authored clear cell above the picked surface
                // maps to a table tile. No nearest tile or numeric-ID fallback.
                picture.cells.iter().find(|cell|cell.at==at).map(|cell|BoardPick::Tile {
                    at:cell.tile,elevation:cell.elevation,top:true,
                })
            },
        }
    }

    pub(in crate::scene) fn native_camera(&self)->Option<isometer::SlabCamera> {
        self.native.presented.as_ref()?;
        self.scene.as_ref()?.presented_camera()
    }
}

fn camera(picture:&Picture,aspect:f32)->Result<isometer::SlabCamera,String> {
    let v=&picture.volume;let origin=picture.origin;
    Ok(picture.input.camera(aspect)?.with_cutaway(Some(Cutaway::Bounds {
        min:[(v.min[0]-origin[0]) as f32,0.0,(v.min[1]-origin[2]) as f32],
        max:[(v.max[0]-origin[0]) as f32,(v.ceiling()-v.datum) as f32,(v.max[1]-origin[2]) as f32],
    })))
}

fn fingerprint(digest:&str)->Result<Vec<u64>,String> {
    (0..4).map(|i|u64::from_str_radix(&digest[i*16..(i+1)*16],16).map_err(|e|e.to_string())).collect()
}
