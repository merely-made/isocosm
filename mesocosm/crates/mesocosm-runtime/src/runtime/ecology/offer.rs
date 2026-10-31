// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Only a bound native admission can enter the actual offered runtime.

use super::{Setup,Seal,Runtime};
use crate::spatial::Game;
use cosmograph::{Session,ecology::AdmittedJointRun,directing::interim::Interim};

pub(super) fn accepted(setup:&Setup,mut game:Game,admitted:AdmittedJointRun,
    ticks_per_second:u32)->Result<Runtime,String> {
    if ticks_per_second==0 {return Err("offered runtime requires a positive clock rate".into());}
    // Consume the exact Session and proof produced by the one validated run.
    let (world,witness,admission)=cosmograph_host::founding::from_admitted_run(setup.execution,admitted)?;
    let (session,sections)=world.into_parts()?;
    game.validate(&session)?;
    let configured=setup.spatial.is_some() || game.driver.is_some();
    let mut advance=|session:&mut Session| {
        if configured {game.advance_clock(session).map(|_|())}
        else {session.advance(1).map(|_|())}
    };
    let mut interim=Interim::begin_using(session,setup.founded.start,setup.founded.mode,
        setup.founded.pace,&mut advance)?;
    interim.on_collapse=setup.founded.on_collapse;
    let mut runtime=Runtime::over(interim,setup.founded.clone(),ticks_per_second);
    for (mode,bytes) in sections { runtime.world().put_section(&mode,bytes)?; }
    runtime.with_product(|product,world| {
        product.spatial=game;
        // No generated advisory pose overwrites the actual assessed physical source.
        product.retain_origin(world);
        product.spatial.ecology=Some(Seal::new(witness,admission));
        product.sync_loop(world);
        product.spatial.validate(&super::super::session(world))
    })?;
    Ok(runtime)
}
