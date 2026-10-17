// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Only a bound native admission can enter the actual offered runtime.

use super::{Setup,Assessment,Seal,Runtime};
use isocosm::{Session,ecology::Admission,directing::interim::Interim};

pub(super) fn accepted(setup:&Setup,assessment:Assessment,admission:Admission,
    ticks_per_second:u32)->Result<Runtime,String> {
    admission.validate(&assessment.witness)?;
    if ticks_per_second==0 {return Err("offered runtime requires a positive clock rate".into());}
    // The runner has ended. Restore its exact authenticated state as the one
    // live owner, retaining all matter, chosen acts and physical history.
    let world=isocosm_host::founding::from_admitted(setup.execution,
        isocosm_host::founding::Offered { witness:&assessment.witness, admission:&admission })?;
    let (session,sections)=world.into_parts()?;
    let mut game=assessment.spatial;
    game.validate_replay(&session)?;
    let configured=setup.spatial.is_some();
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
        product.spatial.ecology=Some(Seal {version:1,witness:assessment.witness,admission});
        product.sync_loop(world);
        product.spatial.validate_replay(&super::super::session(world))
    })?;
    Ok(runtime)
}
