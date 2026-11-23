// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The same column and edit rules without materializing the whole window.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Paged {
    rule:SiteRule, edits:Vec<Edit>, pub ceiling:i64,
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
    /// The column's runs over `[lo, hi)`: the rule's rock, soil, water and
    /// air, each edit laid over them in order.
    pub(super) fn column(&self, x:i64, z:i64, [lo,hi]:[i64;2]) -> Vec<([i64;2],u8)> {
        let (top,r) = (self.top(x,z),&self.rule);
        let mut out = vec![([i64::MIN,top-r.soil+1],r.m.rock), ([top-r.soil+1,top+1],r.m.soil),
            ([top+1,r.water+1],r.m.water), ([top.max(r.water)+1,i64::MAX],r.m.air)];
        out.retain(|(s,_)| s[0]<s[1]);
        for edit in &self.edits {
            let m = edit.op.material(r.m);
            for span in runs(&edit.shape,x,z) { lay(&mut out,span,m); }
        }
        let mut clipped: Vec<([i64;2],u8)> = Vec::new();
        for (s,m) in out {
            let s = [s[0].max(lo),s[1].min(hi)];
            if s[0]>=s[1] { continue; }
            match clipped.last_mut() {
                Some((run,last)) if *last==m && run[1]==s[0] => run[1]=s[1],
                _ => clipped.push((s,m)),
            }
        }
        clipped
    }
    fn surface(&self, x: i64, z: i64, datum: i64) -> Option<i64> {
        if self.edits.is_empty() { return Some(self.top(x, z).max(self.rule.water)); }
        self.column(x, z, [datum, self.ceiling]).into_iter().rev()
            .find(|(_, m)| *m != self.rule.m.air).map(|(span, _)| span[1] - 1)
    }

    /// Dense replay starts from the raised terrain and applies the final
    /// material of every touched cell in one write. Above that terrain only
    /// surviving non-air edits allocate bricks. A write below the datum fails.
    fn replay_ceiling(&self, min: [i64; 2], max: [i64; 2], datum: i64, mut high: i64) -> Result<i64> {
        for edit in &self.edits {
            let [lo, hi] = edit.shape.extent();
            let raises = edit.op.material(self.rule.m) != self.rule.m.air;
            if lo[1] >= datum && (!raises || hi[1] <= high + 1) { continue; }
            for z in lo[2].max(min[1])..hi[2].min(max[1]) {
                for x in lo[0].max(min[0])..hi[0].min(max[0]) {
                    for [bottom, end] in runs(&edit.shape, x, z) {
                        if bottom < datum { return Err("an edit reaching below the window's datum".into()); }
                        if raises && end > high + 1
                            && let Some((span, _)) = self.column(x, z, [bottom.max(high + 1), end])
                                .into_iter().rev().find(|(_, m)| *m != self.rule.m.air) {
                            high = span[1] - 1;
                        }
                    }
                }
            }
        }
        let brick = i64::from(BRICK);
        Ok(brick_ceiling(datum, (high - datum).div_euclid(brick)))
    }
}

/// Lays `m` over `[a, b)` in sorted, disjoint runs.
fn lay(out:&mut Vec<([i64;2],u8)>, [a,b]:[i64;2], m:u8) {
    if a>=b { return; }
    let mut next = Vec::with_capacity(out.len()+2);
    for &(s,was) in out.iter() {
        if s[0]<a.min(s[1]) { next.push(([s[0],a.min(s[1])],was)); }
        if b.max(s[0])<s[1] { next.push(([b.max(s[0]),s[1]],was)); }
    }
    next.push(([a,b],m));
    next.sort_unstable_by_key(|(s,_)| s[0]);
    *out = next;
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
        let width=(max[0]-min[0]) as usize;
        let count = width.checked_mul((max[1] - min[1]) as usize);
        // Fine physical windows keep one height per column, never all the
        // water/rock voxels beneath it. Wider display windows remain procedural.
        let (columns, [low, high]) = if let Some(count) = count.filter(|n| *n <= 256 * 256) {
            let mut tops = Vec::with_capacity(count);
            for z in min[1]..max[1] { for x in min[0]..max[0] { tops.push(rule.top(x,z)); } }
            let bounds = [*tops.iter().min().unwrap(), *tops.iter().max().unwrap()];
            (Some((min, width, tops)), bounds)
        } else { (None, rule.heights(min, max)) };
        let edits:Vec<_> = reaching(a,site,edits).into_iter().filter(|e| {
            let [lo,hi]=e.shape.extent();
            lo[0]<max[0] && hi[0]>min[0] && lo[2]<max[1] && hi[2]>min[1]
        }).collect();
        let datum = low-rule.soil-depth.max(0);
        let extent = ((max[0]-min[0]).max(max[1]-min[1])/2) as i32;
        let ground = Ground::empty(extent,(rule.water-datum) as i32);
        let high = high.max(rule.water);
        let mut paged = Paged { rule, edits, ceiling: 0, columns };
        paged.ceiling = paged.replay_ceiling(min, max, datum, high)?;
        Ok(Self {site,min,max,datum,materials,extent,ground, columns:Some(Arc::new(paged)) })
    }

    /// Highest occupied cell, including water and edits, in site coordinates.
    pub fn surface(&self,x:i64,z:i64) -> Option<i64> {
        if !self.holds(x,z) { return None; }
        if let Some(p) = &self.columns { return p.surface(x,z,self.datum); }
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

#[cfg(test)]
#[path = "paged_tests.rs"]
mod admission_tests;
