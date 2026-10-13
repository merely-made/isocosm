// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual dev ForceBirth uses the same authored newborn geometry as the clock.

use super::*;
use isocosm::{Command, spatial::{BirthPreparation, BirthResolution}};

impl Game {
    pub(crate) fn force_birth(&mut self, session: &mut Session, parent: Id) -> Result<String, String> {
        self.version()?;
        let mut native = session.clone();
        let mut source = self.clone();
        source.refresh(&mut native)?;
        let mut inputs: Vec<BirthResolution> = Vec::new();
        loop {
            match native.prepare_birth_command(Command::ForceBirth { parent }, &inputs)? {
                BirthPreparation::Birth(pending) => {
                    if inputs.len() as u128 >= u128::from(native.sim.genesis().rules.limits.entities)
                        || inputs.iter().any(|r| r.request.child == pending.request.child)
                    { return Err("forced birth geometry made no finite progress".into()); }
                    let mut game = source.clone();
                    game.apply_publications(&pending.sim, &[], &inputs,
                        pending.sim.spatial_births(), pending.sim.spatial_order())?;
                    let geometry = game.resolve_birth(&pending);
                    inputs.push(BirthResolution { request: pending.request, geometry });
                },
                BirthPreparation::Ready(prepared) => {
                    let reports = prepared.births.clone();
                    let order = prepared.order.clone();
                    let out = prepared.publish(&mut native)?;
                    let mut game = source;
                    let changes = game.apply_publications(&native.sim, &[], &inputs, &reports, &order)?;
                    if let Some(driver) = &mut game.driver {
                        let entry = native.entries.last().ok_or("forced birth native entry absent")?.id.clone();
                        driver.records.push(profile::Record { entry, changes, steps: BTreeSet::new(),
                            arrivals: Vec::new(), births: reports, order, command_outcome: Some(out.result.clone()) });
                    }
                    game.version = game.current_version();
                    game.refresh(&mut native)?;
                    *session = native;
                    *self = game;
                    return Ok(out.result);
                },
            }
        }
    }
}
