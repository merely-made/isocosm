// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's import adapter; the core carries the game as opaque bytes.

use super::{Founded, GameSave, Loop, MODE, Runtime};
use isocosm::{
    Execution,
    directing::{
        interim::{Interim, OnCollapse, Pace, Start},
        readings::Mode,
    },
};
use isocosm_host::{
    save::{FORMAT_VERSION, ImportSave, WorldSave},
    world::World,
};

impl ImportSave for GameSave {
    fn into_world_save(self) -> Result<WorldSave, String> {
        self.check()?;
        let bytes = serde_json::to_vec(&self.spatial).map_err(|why| why.to_string())?;
        Ok(WorldSave {
            format: FORMAT_VERSION,
            native: self.native,
            sections: [(MODE.into(), bytes)].into(),
        })
    }
}

impl GameSave {
    /// Read only Mesocosm's section, leaving every other section opaque.
    pub fn from_world(saved: &WorldSave) -> Result<Self, String> {
        if saved.format != FORMAT_VERSION {
            return Err("unknown mode-host save format".into());
        }
        let bytes = saved
            .sections
            .get(MODE)
            .ok_or("world has no Mesocosm section")?;
        let spatial: crate::spatial::Game =
            serde_json::from_slice(bytes).map_err(|why| why.to_string())?;
        let game = Self {
            version: spatial.version,
            native: saved.native.clone(),
            spatial,
        };
        game.check()?;
        Ok(game)
    }
}

impl Runtime {
    /// The core envelope, including sections owned by absent products.
    pub fn save_world(&self) -> Result<WorldSave, String> {
        let mut saved = self.world().save()?;
        // An endpoint may have published since this driver last acted. Its
        // section is then newer than the local product mirror.
        if self.seen == Some(self.world().revision()?) {
            let game = self
                .product()
                .loop_game(self.world(), self.driver.queued().cloned().collect());
            saved.sections.insert(
                MODE.into(),
                serde_json::to_vec(&game).map_err(|why| why.to_string())?,
            );
        } else if self.driver.queued_len() > 0 {
            // Another section's publication must not lose this driver's pending
            // input. Keep the latest game, replacing only the owned queue.
            let mut game = GameSave::from_world(&saved)?.spatial;
            game.continuation
                .as_mut()
                .ok_or("queued input requires a saved loop")?
                .queued = self.driver.queued().cloned().collect();
            saved.sections.insert(
                MODE.into(),
                serde_json::to_vec(&game).map_err(|why| why.to_string())?,
            );
        }
        Ok(saved)
    }

    /// Select a grammar by its discriminator, so a broken world envelope
    /// cannot fall back to a legacy save and silently lose its sections.
    pub fn decode_world_save(bytes: &[u8]) -> Result<WorldSave, String> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|why| why.to_string())?;
        if value.get("format").is_some() || value.get("sections").is_some() {
            let saved = WorldSave::decode(bytes)?;
            GameSave::from_world(&saved)?;
            Ok(saved)
        } else if value.get("spatial").is_some() || value.get("native").is_some() {
            serde_json::from_value::<GameSave>(value)
                .map_err(|why| why.to_string())?
                .into_world_save()
        } else {
            GameSave::from_native(serde_json::from_value(value).map_err(|why| why.to_string())?)
                .into_world_save()
        }
    }

    /// Load native authority in the core, then restore Mesocosm's saved loop.
    pub fn resume_world(
        saved: WorldSave,
        ticks_per_second: u32,
        mode: Option<Mode>,
    ) -> Result<Self, String> {
        let game = GameSave::from_world(&saved)?;
        let execution = game
            .spatial
            .ecology
            .as_ref()
            .map(|seal| seal.witness.execution)
            .unwrap_or(Execution::Grouped);
        let world = World::resume(saved, execution)?;
        let (lp, founded) = world.read(|session| {
            game.spatial.validate_replay(session)?;
            // Interim restores the controller metadata without issuing native
            // commands. Its temporary session is discarded; the core owns play.
            let continuation = game.spatial.continuation.as_ref();
            let settings = continuation.map(|c| c.settings);
            let saved_mode = settings
                .map(|s| s.mode)
                .unwrap_or(mode.unwrap_or(Mode::Creative));
            let pace = settings.map(|s| s.pace).unwrap_or(Pace {
                round: 1,
                scoring: 6,
            });
            let interim = match continuation {
                Some(c) => Interim::resume_as(session.clone(), c.participant, saved_mode, pace)?,
                None => Interim::resume(session.clone(), saved_mode, pace)?,
            };
            let founded = Founded {
                genesis: session.sim.genesis().clone(),
                start: continuation
                    .and_then(|c| c.origin.as_ref())
                    .map(|o| o.start)
                    .unwrap_or(Start::default()),
                mode: saved_mode,
                pace,
                on_collapse: settings.map(|s| s.on_collapse).unwrap_or(OnCollapse::Stay),
            };
            Ok::<_, String>((Loop::split(interim).1, founded))
        })??;
        let mut runtime = Self::over_world(world, lp, founded, ticks_per_second);
        let (queued, fault) = runtime.with_product(|product, world| {
            product.spatial = game.spatial;
            product.restore_loop(world)?;
            if let Some(mode) = mode {
                product.lp.mode = mode;
                product.founded.mode = mode;
                product.sync_loop(world);
            }
            let queued: Vec<_> = product
                .spatial
                .continuation
                .as_ref()
                .map(|c| c.queued.clone())
                .unwrap_or_default();
            Ok::<_, String>((queued, product.fault.clone()))
        })?;
        runtime.driver.restore_queue(queued);
        runtime.driver.restore_fault(fault);
        runtime.publish();
        Ok(runtime)
    }
}
