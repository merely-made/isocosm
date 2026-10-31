// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Coarse instruments and generator-placed play share one native founding.

use super::*;
use cosmograph_host::founding::Purpose;

impl Runtime {
    /// Explicit dev instrument, retaining site-grain placement.
    pub fn bench(founded: Founded, ticks_per_second: u32) -> Result<Self, String> {
        Self::found_in_core(founded, ticks_per_second, Purpose::Bench, false)
    }

    /// Play instrument using the generator's saved stands and spatial clock.
    /// This does not supply the independent ecological admission witness.
    pub fn bench_placed(mut founded: Founded, ticks_per_second: u32) -> Result<Self, String> {
        cosmograph::spatial::embodiment::declare(&mut founded.genesis)?;
        Self::found_in_core(founded, ticks_per_second, Purpose::Bench, true)
    }

    pub fn new(founded: Founded, ticks_per_second: u32) -> Result<Self, String> {
        ecology::fresh_offer(&founded)?;
        Self::found_in_core(founded, ticks_per_second, Purpose::Head, false)
    }

    fn found_in_core(
        founded: Founded,
        ticks_per_second: u32,
        purpose: Purpose,
        placed: bool,
    ) -> Result<Self, String> {
        let world = cosmograph_host::founding::found(
            founded.genesis.clone(),
            Execution::Grouped,
            purpose,
            None,
        )?;
        let (mut session, sections) = world.into_parts()?;
        let mut game = crate::spatial::Game::default();
        if placed && !game.found_generated(&mut session)? {
            return Err("placed play needs generator-declared physical templates".into());
        }
        let mut advance = |session: &mut Session| game.advance_clock(session).map(|_| ());
        let mut interim = if placed {
            Interim::begin_using(
                session,
                founded.start,
                founded.mode,
                founded.pace,
                &mut advance,
            )?
        } else {
            Interim::found_session(session, founded.start, founded.mode, founded.pace)?
        };
        interim.on_collapse = founded.on_collapse;
        let mut runtime = Self::over(interim, founded, ticks_per_second);
        for (mode, bytes) in sections {
            runtime.world().put_section(&mode, bytes)?;
        }
        runtime.with_product(|product, world| {
            if placed {
                product.spatial = game;
                product.sync_loop(world);
            } else {
                product.place_members(world);
            }
            product.retain_origin(world);
        });
        Ok(runtime)
    }
}
