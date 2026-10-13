// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl World {
    pub fn restore_record(save: WorldSave) -> Result<Self, WorldError> {
        if ![2, 3, GENERATOR_VERSION, MATERIAL_GENERATOR_VERSION, OWNED_GENERATOR_VERSION,
            SPREAD_GENERATOR_VERSION, crate::founding::fixture::GENERATOR_VERSION].contains(&save.generator_version) {
            return Err(WorldError::GeneratorDiverged { saved: save.generator_version, current: GENERATOR_VERSION });
        }
        let material = [MATERIAL_GENERATOR_VERSION, OWNED_GENERATOR_VERSION, SPREAD_GENERATOR_VERSION]
            .contains(&save.generator_version);
        if save.intents.iter().filter(|i| matches!(i, WorldIntent::MaterialProfile { .. }
            | WorldIntent::OwnedMaterialProfile { .. } | WorldIntent::SpreadMaterialProfile { .. })).count()
            != usize::from(material) { return Err(WorldError::InvalidMaterialProfile); }
        let mut world = if material {
            match save.intents.first() {
                Some(WorldIntent::MaterialProfile { profile }) if save.generator_version == MATERIAL_GENERATOR_VERSION =>
                    Self::generate_materials(save.seed, save.config, (**profile).clone())?,
                Some(WorldIntent::OwnedMaterialProfile { profile, stock }) if save.generator_version == OWNED_GENERATOR_VERSION =>
                    Self::generate_owned_materials(save.seed, save.config, (**profile).clone(), stock.clone())?,
                Some(WorldIntent::SpreadMaterialProfile { profile }) if save.generator_version == SPREAD_GENERATOR_VERSION =>
                    Self::generate_spread_materials(save.seed, save.config, (**profile).clone())?,
                _ => return Err(WorldError::InvalidMaterialProfile),
            }
        } else { Self::generate_version(save.seed, save.config, save.generator_version)? };
        if save.base_hash != world.base_hash {
            return Err(WorldError::BaseDiverged { saved: save.base_hash, regrown: world.base_hash });
        }
        for intent in save.intents.into_iter().skip(usize::from(material)) { world.apply(intent)?; }
        Ok(world)
    }
}
