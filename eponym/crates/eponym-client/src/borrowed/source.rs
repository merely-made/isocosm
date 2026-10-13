// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cambium_rootstock::{ProducedTexture,ProducerContext,TextureProducer};
use isometer::{BodySignature,DeclaredExtentVolumes,FrameRequest,GroundTerrain,Scene,SceneBody,
    SceneFrame,SceneHost,SceneProducer,SceneSignature,SceneSource,SceneVolumes,SubjectKey,TerrainSource};
use isometer::lens::{BrickMap,Grade};
use crate::producer::{BODY_BUDGET,PALETTE};

pub struct Producer {handle:SceneHandle,inner:SceneProducer<Source>}

impl Producer {
    pub fn new(handle:SceneHandle) -> Self {Self {inner:SceneProducer::new(Source::new(handle.clone())),handle}}
}

impl TextureProducer for Producer {
    fn render(&mut self,cx:&ProducerContext<'_>) -> Option<ProducedTexture> {
        let core = &cx.core.renderer().wgpu_device.core;
        if &core.device != cx.device || &core.queue != cx.queue {
            self.handle.clear();
            *self.handle.error.borrow_mut() = Some("Eponym picture GPU differs from the current RenderCore".into());
            return None;
        }
        self.inner.render(cx)
    }
    fn suspend(&mut self) {self.inner.suspend();}
    fn retire(&mut self) {self.inner.retire();}
}

struct PlainHost;
impl SceneHost for PlainHost {}

struct Source {
    handle:SceneHandle,scene:Option<Scene>,size:[u32;2],
    gpu:Option<(wgpu::Device,wgpu::Queue)>,terrain:Option<[u64;4]>,
    blank:Option<(wgpu::Texture,wgpu::TextureView,[u32;2])>,presented:Option<SlabCamera>,
}

impl Source {
    fn new(handle:SceneHandle) -> Self {
        Self {handle,scene:None,size:[0;2],gpu:None,terrain:None,blank:None,presented:None}
    }
    fn ensure(&mut self,r:&FrameRequest<'_>) -> Result<(),String> {
        if let Some((device,queue)) = &self.gpu {
            if device != r.device || queue != r.queue {return Err("Eponym picture cannot migrate to another GPU".into());}
        } else {self.gpu=Some((r.device.clone(),r.queue.clone()));}
        let size=[r.size[0].max(1),r.size[1].max(1)];
        if self.scene.is_none() {
            self.scene=Some(Scene::new(r.device.clone(),r.queue.clone(),size[0],size[1])?);
            self.terrain=None;
        } else if self.size!=size {self.scene.as_mut().unwrap().resize(size[0],size[1]);}
        self.size=size;
        Ok(())
    }
    fn empty(&mut self,r:&FrameRequest<'_>) -> wgpu::TextureView {
        let size=[r.size[0].max(1),r.size[1].max(1)];
        if self.blank.as_ref().is_none_or(|blank|blank.2!=size) {
            let texture=r.device.create_texture(&wgpu::TextureDescriptor {
                label:Some("withdrawn Eponym picture"),size:wgpu::Extent3d {width:size[0],height:size[1],depth_or_array_layers:1},
                mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,
                format:wgpu::TextureFormat::Rgba8Unorm,
                usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats:&[],
            });
            let view=texture.create_view(&Default::default());
            self.blank=Some((texture,view,size));
        }
        let mut encoder=r.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label:Some("clear withdrawn Eponym picture")});
        {
            let _pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label:Some("withdrawn Eponym picture"),
                color_attachments:&[Some(wgpu::RenderPassColorAttachment {
                    view:&self.blank.as_ref().unwrap().1,depth_slice:None,resolve_target:None,
                    ops:wgpu::Operations {load:wgpu::LoadOp::Clear(wgpu::Color::BLACK),store:wgpu::StoreOp::Store},
                })],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None,multiview_mask:None,
            });
        }
        r.queue.submit(Some(encoder.finish()));
        self.presented=None;
        self.blank.as_ref().unwrap().1.clone()
    }
}

impl SceneSource for Source {
    fn inputs(&mut self,r:&FrameRequest<'_>) -> Result<SceneSignature,String> {
        let Some(input)=self.handle.input.borrow().clone() else {
            return Ok(SceneSignature {size:r.size,..Default::default()});
        };
        let p=&input.picture.presentation;
        let bodies=input.picture.bodies.iter().map(|body|BodySignature {
            subject:SubjectKey(body.entity),revision:body.revision,pose:feet_pose(body,p.scale),
            scale:p.scale,grounded:true,
            tint:if Some(body.entity)==input.picture.played {p.played_tint}else{p.other_tint},
            always_visible:Some(body.entity)==input.picture.played,
        }).collect();
        let mut host=input.picture.fingerprint.to_vec();host.push(u64::from(input.turn));
        Ok(SceneSignature {size:r.size,render_scale:r.render_scale,camera:Some(input.camera(r.aspect)?),
            terrain_revision:Some(input.picture.fingerprint[0]),bodies,host})
    }
    fn frame(&mut self,r:&FrameRequest<'_>) -> Result<Option<wgpu::TextureView>,String> {
        self.ensure(r)?;
        let input=self.handle.input.borrow().clone();
        let Some(input)=input else {return Ok(Some(self.empty(r)));};
        let result=(|| {
            let camera=input.camera(r.aspect)?;
            let picture=&input.picture;
            let p=&picture.presentation;
            let scene=self.scene.as_mut().ok_or("Eponym picture scene is absent")?;
            if self.terrain!=Some(picture.fingerprint) {
                scene.set_terrain_map(BrickMap::from_ground(picture.volume.ground()).map_err(|e|format!("Eponym terrain: {e}"))?);
                self.terrain=Some(picture.fingerprint);
            }
            let volumes=DeclaredExtentVolumes::from_documents(picture.bodies.iter().map(|body|&body.document),p.material);
            let bodies:Vec<_>=picture.bodies.iter().map(|body|SceneBody {
                subject:SubjectKey(body.entity),document:&body.document,pose:feet_pose(body,p.scale),
                scale:p.scale,grounded:true,tint:if Some(body.entity)==picture.played {p.played_tint}else{p.other_tint},
                materials:&[],always_visible:Some(body.entity)==picture.played,
            }).collect();
            let ground=GroundTerrain(picture.volume.ground());
            let mut encoder=r.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label:Some("borrowed Eponym scene")});
            scene.render(&mut encoder,SceneFrame {camera,bodies:&bodies,volumes:SceneVolumes::DeclaredSolid(&volumes),
                terrain:Some(&ground as &dyn TerrainSource),dirty:&[],grade:Grade::retro(PALETTE),
                terrain_appearance:None,body_budget:BODY_BUDGET,capsules:None},&mut PlainHost)?;
            r.queue.submit(Some(encoder.finish()));
            self.presented=Some(camera);
            Ok::<_,String>(scene.encoded_view().clone())
        })();
        match result {
            Ok(view)=>Ok(Some(view)),
            Err(why)=>{self.handle.clear();*self.handle.error.borrow_mut()=Some(why);Ok(Some(self.empty(r)))},
        }
    }
    fn presented_camera(&self)->Option<SlabCamera> {self.presented}
    fn cached_bodies(&self)->usize {self.scene.as_ref().map_or(0,Scene::cached_bodies)}
    fn retire(&mut self) {self.scene=None;self.gpu=None;self.terrain=None;self.blank=None;self.presented=None;self.size=[0;2];}
}

/// MotionPose is the accepted feet centre. Rotate the document about its
/// native root, then stand its rotated horizontal centre at those exact quanta.
/// SceneBody's existing grounded lowering supplies the document's floor.
fn feet_pose(body:&eponym_play::contract::mode::PictureBody,scale:f32)->isometer::Pose {
    use isometer::core::Yaw;
    let feet=body.pose.position.map(|n|n as f32/eponym_play::MOTION_SCALE as f32);
    let bounds=body.document.aabb();
    let x=(bounds.min[0] as f32+bounds.max[0] as f32)*0.5;
    let z=(bounds.min[2] as f32+bounds.max[2] as f32)*0.5;
    let (centre,yaw)=match body.yaw {
        Yaw::Zero=>([x,z],0.0),
        Yaw::Quarter=>([z,-x],std::f32::consts::FRAC_PI_2),
        Yaw::Half=>([-x,-z],std::f32::consts::PI),
        Yaw::ThreeQuarter=>([-z,x],-std::f32::consts::FRAC_PI_2),
    };
    isometer::Pose {position:[feet[0]-centre[0]*scale,feet[1],feet[2]-centre[1]*scale],yaw_radians:yaw}
}
