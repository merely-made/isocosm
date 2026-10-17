// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Authored body-relative eye and orientation; cameras supply neither.

use super::*;
use isocosm::custody::{MountPose,Yaw};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialSense {
    pub yaw:Yaw,
    /// Exact root-local base-cell vector, rotated with the authored body yaw.
    pub eye_offset:[i64;3],
    pub range:u64,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterSpatialPlacement {pub placement:CharacterPlacement,pub spatial:SpatialSense}

pub(crate) fn rotate(yaw:Yaw,value:[i64;3])->Result<[i64;3],String> {
    let [x,y,z]=value;
    let neg=|n:i64|n.checked_neg().ok_or_else(||"spatial rotation overflows base cells".to_string());
    Ok(match yaw {Yaw::Zero=>value,Yaw::Quarter=>[z,y,neg(x)?],
        Yaw::Half=>[neg(x)?,y,neg(z)?],Yaw::ThreeQuarter=>[neg(z)?,y,x]})
}

impl SpatialSense {
    pub(crate) fn validate(&self)->Result<(),String> {
        if self.range>i64::MAX as u64 {return Err("spatial sight range exceeds base-cell bounds".into());}
        rotate(self.yaw,self.eye_offset)?;
        Ok(())
    }
    pub(crate) fn eye(&self,root:[i64;3])->Result<[i64;3],String> {
        self.validate()?;
        let offset=rotate(self.yaw,self.eye_offset)?;
        let mut eye=[0;3];
        for axis in 0..3 {eye[axis]=root[axis].checked_add(offset[axis]).ok_or("spatial eye overflows base cells")?;}
        Ok(eye)
    }
    pub(crate) fn check(&self,session:&Session,placement:&CharacterPlacement)->Result<(),String> {
        let eye=self.eye(placement.at)?;
        let window=&placement.window;
        let volume=session.sim.volume(placement.site,window.min,window.max,window.depth)?;
        let places=Places::derive(&volume,placement.places)?;
        let range=self.range as i64;
        if places.places.values().any(|place|(0..3).any(|axis|
            place.bounds[0][axis].checked_sub(range).is_none()
                || place.bounds[1][axis].checked_add(range).is_none())) {
            return Err("spatial sight candidate bounds overflow".into());
        }
        if !volume.holds(eye[0],eye[2]) || eye[1]<volume.datum || eye[1]>=volume.ceiling()
            || places.place_at(eye)!=Some(placement.patch)
            || !volume.material(eye).is_some_and(|material|material==volume.materials.air || material==volume.materials.water) {
            return Err("authored eye is outside its current realized clear native patch/window".into());
        }
        Ok(())
    }
}

impl PositionBinding {
    /// Older unmarked bindings explicitly retain their original zero yaw.
    pub fn current_pose(&self,session:&Session)->Result<MountPose,String> {
        Ok(MountPose {at:self.current(session)?,yaw:self.spatial.as_ref().map_or(Yaw::Zero,|spatial|spatial.yaw)})
    }
    pub fn current_eye(&self,session:&Session)->Result<Option<([i64;3],u64)>,String> {
        let root=self.current(session)?;
        self.spatial.as_ref().map(|spatial|Ok((spatial.eye(root)?,spatial.range))).transpose()
    }
}

pub(crate) fn place_spatial_character(session:&mut Session,input:&CharacterSpatialPlacement,
    recipe:&RecipeBinding)->Result<PositionBinding,String> {
    input.placement.check(session,recipe.entity,false)?;
    input.spatial.check(session,&input.placement)?;
    let mut binding=place_character(session,&input.placement,recipe)?;
    binding.spatial=Some(input.spatial.clone());
    binding.current(session)?;
    Ok(binding)
}
