// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! On-demand bricks sampled from the space crate's exact column/edit rules.
use super::{BrickSource,BRICK_BYTES};
use isometer_space::volume::Volume;
use std::{cell::RefCell,collections::BTreeMap};

pub struct ColumnTerrain { pub volume:Volume, pub cell:i64, layers:RefCell<BTreeMap<[i16;2],i16>> }
impl ColumnTerrain {
    pub fn new(volume:Volume,cell:i64) -> Self {
        Self { volume,cell:cell.max(1),layers:RefCell::default() }
    }
    /// Frame native camera coordinates, coarsening only when its atlas would
    /// otherwise drop visible bricks. The retained source keeps that grain.
    pub fn frame(&mut self,camera:crate::SlabCamera,margin:f32,capacity:usize)
        -> super::FramedBricks {
        loop {
            let cell=self.cell as f32;
            let scaled=crate::SlabCamera {centre:camera.centre.map(|v|v/cell),
                half_height:camera.half_height/cell,depth:camera.depth/cell,..camera};
            let framed=super::framed_bricks(scaled,margin,self,capacity);
            let width=(self.volume.max[0]-self.volume.min[0]).max(self.volume.max[1]-self.volume.min[1]);
            if framed.overflow==0 || self.cell>=width {return framed;}
            self.cell*=2;
            self.layers.get_mut().clear();
        }
    }
    fn site(&self,x:i64,y:i64,z:i64) -> [i64;3] {
        let v=&self.volume;
        let e=i64::from(v.ground().extent());
        [v.min[0]+e+x*self.cell,v.datum+y*self.cell,v.min[1]+e+z*self.cell]
    }
}
impl BrickSource for ColumnTerrain {
    fn cell_size(&self) -> f32 { self.cell as f32 }
    fn bounds(&self) -> Option<[[i16;3];2]> {
        let v=&self.volume;
        let e=i64::from(v.ground().extent());
        let edge=8*self.cell;
        Some([[(-e).div_euclid(edge) as i16,0,(-e).div_euclid(edge) as i16],
            [((v.max[0]-v.min[0]-e-1).div_euclid(edge)) as i16,
             ((v.ceiling()-v.datum).div_euclid(edge)) as i16,
             ((v.max[1]-v.min[1]-e-1).div_euclid(edge)) as i16]])
    }
    fn layers(&self,c:[i16;2]) -> std::ops::Range<i16> {
        if let Some(&top)=self.layers.borrow().get(&c) { return 0..top; }
        let mut high=None;
        for z in 0..8 { for x in 0..8 {
            let p=self.site(i64::from(c[0])*8+x,0,i64::from(c[1])*8+z);
            if let Some(y)=self.volume.surface(p[0],p[2]) { high=Some(high.unwrap_or(y).max(y)); }
        } }
        let end=high.map_or(0,|y| ((y-self.volume.datum).div_euclid(8*self.cell)+1) as i16);
        self.layers.borrow_mut().insert(c,end);
        0..end
    }
    fn fill(&self,key:[i16;3],out:&mut[u8]) {
        assert_eq!(out.len(),BRICK_BYTES);
        for z in 0..8 { for x in 0..8 {
            let base=self.site(i64::from(key[0])*8+x,0,i64::from(key[2])*8+z);
            let top=(self.cell>1).then(||self.volume.surface(base[0],base[2])).flatten();
            for y in 0..8 {
                let mut p=[base[0],self.volume.datum+(i64::from(key[1])*8+y)*self.cell,base[2]];
                // A coarse cell containing the actual surface keeps its material;
                // sampling its bottom alone loses thin soil to the rock beneath.
                if let Some(top)=top.filter(|top|p[1]<=*top && *top<p[1]+self.cell) {p[1]=top;}
                out[(y*64+z*8+x) as usize]=self.volume.material(p).unwrap_or(self.volume.materials.air);
            }
        } } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{GroundTerrain,framed_bricks};
    use crate::SlabCamera;
    #[test]
    fn base_grain_pages_match_the_materialized_ground() {
        let g=flat_fixture();
        let a=isometer_space::volume::Volume::lift(&g,0,[0,0],[64,64],8,&[]).unwrap();
        let b=isometer_space::volume::Volume::paged(&g,0,[0,0],[64,64],8,&[]).unwrap();
        let source=ColumnTerrain::new(b,1);
        // The datums can differ conservatively, so compare in native coordinates.
        for key in framed_bricks(SlabCamera::dimetric_2_1([0.,20.,0.],40.,1.0,128.).unwrap(),
            8.,&source,16383).keys {
            let mut raw=vec![0;BRICK_BYTES]; source.fill(key,&mut raw);
            for y in 0..8 { for z in 0..8 { for x in 0..8 {
                let at=source.site(i64::from(key[0])*8+x,i64::from(key[1])*8+y,i64::from(key[2])*8+z);
                let m=a.material(at).unwrap_or(0);
                assert_eq!(raw[(y*64+z*8+x) as usize],m);
            } } }
        }
        assert!(GroundTerrain(a.ground()).bounds().is_some());
    }

    #[test]
    fn a_coarse_surface_cell_retains_its_thin_native_soil_layer() {
        let v=Volume::paged(&flat_fixture(),0,[0;2],[64;2],8,&[]).unwrap();
        let source=ColumnTerrain::new(v,8);
        let top=source.volume.surface(32,32).unwrap();
        let y=(top-source.volume.datum).div_euclid(8);
        let at=source.site(0,y,0);
        assert_eq!(source.volume.material(at),Some(source.volume.materials.rock));
        let mut raw=vec![0;BRICK_BYTES];source.fill([0,0,0],&mut raw);
        assert_eq!(raw[y as usize*64],source.volume.materials.soil);
        assert_eq!(raw[(y as usize+1)*64],source.volume.materials.air);
    }

    #[test]
    fn a_camera_over_budget_coarsens_without_dropping_framed_terrain() {
        let v=Volume::paged(&flat_fixture(),0,[0;2],[64;2],8,&[]).unwrap();
        let mut source=ColumnTerrain::new(v,1);
        let camera=SlabCamera::dimetric_2_1([0.,20.,0.],40.,1.0,128.).unwrap();
        assert!(framed_bricks(camera,8.,&source,64).overflow>0);
        let framed=source.frame(camera,8.,64);
        assert_eq!(framed.overflow,0);
        assert!(source.cell>1);
        let cell=source.cell;
        assert_eq!(source.frame(camera,8.,64),framed);
        assert_eq!(source.cell,cell);
    }

    fn flat_fixture() -> impl isometer_space::Atlas {
        struct Flat;
        impl isometer_space::Atlas for Flat {
            fn seed(&self)->u64 {7}
            fn footprint(&self)->isometer_space::Footprint {isometer_space::Footprint {sides:4,side:64}}
            fn sites(&self)->Vec<u64> {vec![0]}
            fn terrain_seed(&self,_:u64)->Result<u64,String> {Ok(7)}
            fn elevation(&self,_:u64)->Result<i64,String> {Ok(16)}
            fn relief(&self,_:u64)->Result<i64,String> {Ok(0)}
            fn water(&self,_:u64)->Result<i64,String> {Ok(0)}
            fn border(&self,_:u64,_:u8)->Option<(u64,isometer_space::Border)> {None}
            fn materials(&self)->Result<isometer_space::Materials,String> {
                Ok(isometer_space::Materials {air:0,water:1,soil:2,rock:3})
            }
        }
        Flat
    }
}
