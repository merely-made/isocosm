// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::SpreadMaterialProfile;

impl World {
    pub fn generate_spread_materials(seed: u64, config: WorldConfig, profile: SpreadMaterialProfile)
        -> Result<Self, WorldError> {
        Self::check_config(config)?;
        let genesis = profile.genesis(seed, u32::from(config.side), config.extent as u64 * 2)
            .map_err(|_| WorldError::InvalidMaterialProfile)?;
        let marker = WorldIntent::SpreadMaterialProfile { profile: Box::new(profile.clone()) };
        let mut world = Self::from_genesis(seed, config, SPREAD_GENERATOR_VERSION, genesis,
            cosmograph::schema::Movement::Journey, vec![marker])?;
        world.arrive_material_items(&profile.materials)?;
        Ok(world)
    }

    pub fn spread_profile(&self) -> Option<&SpreadMaterialProfile> {
        if self.generator_version != SPREAD_GENERATOR_VERSION { return None; }
        match self.intents.first()? { WorldIntent::SpreadMaterialProfile { profile } => Some(profile), _ => None }
    }

    pub(super) fn arrive_material_items(&mut self, profile: &MaterialProfile) -> Result<(), WorldError> {
        for item in &profile.items {
            let [x, _, z] = crate::walking::to_site(&self.volume, item.at);
            if x < self.volume.min[0] || x >= self.volume.max[0]
                || z < self.volume.min[1] || z >= self.volume.max[1] {
                return Err(WorldError::InvalidMaterialProfile);
            }
            let recipe = &self.sim().genesis().rules.material_recipes[&item.recipe];
            let arrival = recipe.arrival(self.site).map_err(|_| WorldError::InvalidMaterialProfile)?;
            self.found(Command::Arrive(arrival))?;
        }
        Ok(())
    }

    pub(crate) fn prepare_spread_command(&self, command: &Command) -> Result<cosmograph::projects::SpreadPreview, WorldError> {
        self.session.prepare_owned_spread(command).map_err(|_| WorldError::Refused)
    }
}
