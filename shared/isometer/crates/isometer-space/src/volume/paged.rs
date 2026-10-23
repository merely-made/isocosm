// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The same column and edit rules without materializing the whole window.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Paged {
    rule:SiteRule, edits:Vec<Edit>, pub high:i64,
    columns:Option<([i64;2],usize,Vec<i64>)>,
}
impl Paged {
    fn top(&self,x:i64,z:i64) -> i64 {
        self.columns.as_ref().map_or_else(|| self.rule.top(x,z),|(min,width,tops)|
            tops[(z-min[1]) as usize*width+(x-min[0]) as usize])
    }
    pub(super) fn material(&self, [x,y,z]:[i64;3]) -> u8 {
        for edit in self.edits.iter().rev() {
            if runs(&edit.shape,x,z).iter().any(|r| (r[0]..r[1]).contains(&y)) {
                return edit.op.material(self.rule.m);
            }
        }
        self.rule.rule(self.top(x,z),y)
    }
    fn surface(&self,x:i64,z:i64) -> i64 {
        let mut high = self.top(x,z).max(self.rule.water);
        for edit in &self.edits {
            if edit.op != crate::edit::Op::Carve {
                high = high.max(runs(&edit.shape,x,z).iter().map(|r| r[1]-1).max().unwrap_or(high));
            }
        }
        while self.material([x,high,z]) == self.rule.m.air { high-=1; }
        high
    }
}

impl Volume {
    pub fn is_paged(&self) -> bool { self.columns.is_some() }
    /// An exact read-only volume backed by the existing column rules and edits.
    /// `ground()` is an empty projection; consumers page its materials on demand.
    pub fn paged<A: Atlas + ?Sized>(a:&A, site:SiteId, min:[i64;2],max:[i64;2],depth:i64,
        edits:&[(SiteId,Edit)]) -> Result<Self> {
        let side = a.footprint().side as i64;
        if (0..2).any(|k| min[k]<0 || min[k]>=max[k] || max[k]>side) {
            return Err("a window outside its site".into());
        }
        let rule = SiteRule::of(a,site)?;
        let materials = rule.m;
        if materials.air != isometer_core::ground::AIR || [materials.air,materials.water,materials.soil,materials.rock]
            .iter().any(|m|*m>isometer_core::ground::MAX_MATERIAL) {
            return Err("paged materials exceed Ground's material contract".into());
        }
        let [mut low,mut high] = rule.heights();
        let width=(max[0]-min[0]) as usize;
        let count=width*(max[1]-min[1]) as usize;
        // Fine physical windows keep one height per column, never all the
        // water/rock voxels beneath it. Wider display windows remain procedural.
        let columns = if count<=256*256 {
            let mut tops=Vec::with_capacity(count);
            for z in min[1]..max[1] {for x in min[0]..max[0] {tops.push(rule.top(x,z));}}
            low=*tops.iter().min().unwrap();
            high=(*tops.iter().max().unwrap()).max(rule.water);
            Some((min,width,tops))
        } else {None};
        let edits:Vec<_> = reaching(a,site,edits).into_iter().filter(|e| {
            let [lo,hi]=e.shape.extent();
            lo[0]<max[0] && hi[0]>min[0] && lo[2]<max[1] && hi[2]>min[1]
        }).collect();
        for e in &edits { let [lo,hi]=e.shape.extent(); low=low.min(lo[1]); high=high.max(hi[1]); }
        let datum = low-rule.soil-depth.max(0);
        let extent = ((max[0]-min[0]).max(max[1]-min[1])/2) as i32;
        let ground = Ground::empty(extent,(rule.water-datum) as i32);
        Ok(Self {site,min,max,datum,materials,extent,ground,
            columns:Some(Arc::new(Paged {rule,edits,high,columns})) })
    }

    /// Highest occupied cell, including water and edits, in site coordinates.
    pub fn surface(&self,x:i64,z:i64) -> Option<i64> {
        if !self.holds(x,z) { return None; }
        if let Some(p) = &self.columns { return Some(p.surface(x,z)); }
        let [gx,_,gz] = self.to_ground([x,self.datum,z]);
        self.ground.surface(gx,gz).map(|y| self.datum+i64::from(y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixture::Grid,edit::{Op,Shape}};
    #[test]
    fn paged_columns_match_realized_voxels_with_water_and_border_edits() {
        let a=Grid::drawn(17,2,2,64,false);
        let edits=vec![(0,Edit {op:Op::Carve,shape:Shape::Box {min:[24,10,24],max:[31,35,31]}}),
            (0,Edit {op:Op::Fill(3),shape:Shape::Sphere {centre:[30,28,30],radius:3}}),
            (1,Edit {op:Op::Fill(2),shape:Shape::Box {min:[-2,30,24],max:[2,34,29]}})];
        for site in [0,1] {
            let v=Volume::lift(&a,site,[24,24],[36,36],8,&edits).unwrap();
            let p=Volume::paged(&a,site,[24,24],[36,36],8,&edits).unwrap();
            assert_eq!(p.ground().brick_count(),0);
            for x in 24..36 { for z in 24..36 {
                assert_eq!(p.surface(x,z),v.surface(x,z));
                for y in v.datum..v.ceiling()+3 {
                    assert_eq!(p.material([x,y,z]),v.material([x,y,z]),"{site} {x} {y} {z}");
                }
            } }
        }
    }
}
