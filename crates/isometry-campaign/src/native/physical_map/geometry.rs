// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::{projects::{DebitPreview,DebitSelection},rules::AccountKind};
use isometer_space::Atlas;

pub(super) fn check(input:&PhysicalMapInput) -> Result<(),String> {
    if input.metadata.key.trim().is_empty() || input.metadata.cells.is_empty()
        || input.metadata.cells.len() != input.geometry.cells.len()
        || input.geometry.cells.len() != input.steps.len() || input.geometry.window.depth < 0 {
        return Err("physical map edit needs one complete ordered geometry and quantity binding per metadata cell".into());
    }
    let mut cells=BTreeSet::new();
    for (metadata,geometry) in input.metadata.cells.iter().zip(&input.geometry.cells) {
        if metadata != &geometry.table || !cells.insert(metadata.at.0) {
            return Err("physical geometry differs from or repeats its exact authored metadata cell".into());
        }
    }
    Ok(())
}

fn material(session:&Session,id:u8) -> Result<MaterialBinding,String> {
    let material=session.sim.genesis().world.materials.get(usize::from(id))
        .ok_or("physical map material ID is absent")?;
    let density=material.density.ok_or("physical map material has no declared density")?;
    let account=material.account.as_ref().ok_or("physical map material has no native stock account")?;
    if !matches!(session.sim.genesis().rules.accounts.get(account),Some(AccountKind::Matter { lineage,.. })
        if lineage == &material.lineage) {
        return Err("physical map material account does not preserve its declared native lineage".into());
    }
    Ok(MaterialBinding { id,key:material.key.clone(),lineage:material.lineage.clone(),
        account:account.clone(),density })
}

fn extent(shape:&Shape) -> Result<[[i64;3];2],String> {
    let around=|point:[i64;3],radius:u64| -> Result<[[i64;3];2],String> {
        let radius=i64::try_from(radius).map_err(|_|"physical map shape radius exceeds native coordinates")?;
        if radius == 0 { return Err("physical map shape has no affected cell region".into()); }
        let mut low=[0;3];
        let mut high=[0;3];
        for axis in 0..3 {
            low[axis]=point[axis].checked_sub(radius).ok_or("physical map shape extent underflow")?;
            high[axis]=point[axis].checked_add(radius).ok_or("physical map shape extent overflow")?;
        }
        Ok([low,high])
    };
    match shape {
        Shape::Box { min,max } => {
            if (0..3).any(|axis|min[axis]>=max[axis]) { return Err("physical map box has no ordered cell volume".into()); }
            Ok([*min,*max])
        },
        Shape::Sphere { centre,radius } => around(*centre,*radius),
        Shape::Route { points,radius } => {
            if points.is_empty() || points.len()>isometer_space::edit::ROUTE_POINTS {
                return Err("physical map route has no bounded authored points".into());
            }
            let mut bounds=[[i64::MAX;3],[i64::MIN;3]];
            for point in points {
                let [lo,hi]=around(*point,*radius)?;
                for axis in 0..3 { bounds[0][axis]=bounds[0][axis].min(lo[axis]); bounds[1][axis]=bounds[1][axis].max(hi[axis]); }
            }
            Ok(bounds)
        },
    }
}

pub(super) fn command(session:&Session,input:&PhysicalMapInput,index:usize) -> Result<Command,String> {
    let geometry=input.geometry.cells.get(index).ok_or("physical cell geometry is absent")?;
    let op=match &geometry.operation {
        Operation::Carve=>Op::Carve,
        Operation::Fill(binding)=> {
            if &material(session,binding.id)? != binding { return Err("physical fill material binding is stale or incomplete".into()); }
            Op::Fill(binding.id)
        },
    };
    let [lo,hi]=extent(&geometry.shape)?;
    let window=&input.geometry.window;
    let volume=session.sim.volume(input.geometry.site,window.min,window.max,window.depth)?;
    if lo[0]<volume.min[0] || hi[0]>volume.max[0] || lo[2]<volume.min[1] || hi[2]>volume.max[1]
        || lo[1]<volume.datum || hi[1]>volume.ceiling() {
        return Err("physical edit shape exceeds its explicit realized native window".into());
    }
    let edit=Edit { op,shape:geometry.shape.clone() };
    edit.check(session.sim.atlas()?.footprint())?;
    Ok(Command::Edit { site:input.geometry.site,edit,by:Some(input.actor) })
}

pub(super) fn movements(session:&Session,command:&Command) -> Result<Vec<MaterialQuantity>,String> {
    let Command::Edit { site,edit,.. }=command else { return Err("physical map operation is not a native edit".into()); };
    session.sim.moved(*site,edit)?.into_iter().map(|(id,cells)| {
        let material=material(session,id)?;
        let quantity=cells.unsigned_abs().checked_mul(material.density).ok_or("physical material quantity overflow")?;
        Ok(MaterialQuantity { material,cells,quantity })
    }).collect()
}

pub(super) fn selections(preview:&DebitPreview,selections:&[DebitSelection]) -> bool {
    selections.len()==preview.requests.len() && preview.requests.iter().all(|request|
        selections.iter().filter(|selection| &selection.request==request).count()==1)
}
