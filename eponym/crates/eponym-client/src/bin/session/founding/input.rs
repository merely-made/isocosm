// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Separate strict startup grammars preserve the original material input.

use super::*;

#[derive(Deserialize)]
struct Version { version:u32 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpreadInput {
    version:u32,
    seed:u64,
    config:WorldConfig,
    profile:eponym_play::SpreadMaterialProfile,
    initial_intents:Vec<GameIntent>,
    #[serde(default)]
    stock_ownership:Vec<cosmograph::projects::StockLot>,
    played:SubjectId,
    target:SubjectId,
    target_item:ItemId,
    combat_rules:CombatRules,
    network:NetworkSnapshot,
    timed_rules:TimedActionRules,
    motion_rules:MotionRules,
    glyphs:Option<GlyphRules>,
}

pub(super) struct Prepared {
    pub world:World,
    pub initial_intents:Vec<GameIntent>,
    pub stock_ownership:Vec<cosmograph::projects::StockLot>,
    pub played:SubjectId,
    pub target:SubjectId,
    pub target_item:ItemId,
    pub combat_rules:CombatRules,
    pub network:NetworkSnapshot,
    pub timed_rules:TimedActionRules,
    pub motion_rules:MotionRules,
    pub glyphs:Option<GlyphRules>,
}

pub(super) fn decode(bytes:&[u8]) -> Result<Prepared,String> {
    let version:Version=serde_json::from_slice(bytes).map_err(|error|error.to_string())?;
    match version.version {
        1=> {
            let input:Input=serde_json::from_slice(bytes).map_err(|error|error.to_string())?;
            let world=match input.owned_stock {
                Some(stock)=>World::generate_owned_materials(input.seed,input.config,input.materials,stock),
                None=>World::generate_materials(input.seed,input.config,input.materials),
            }.map_err(|error|format!("material founding: {error:?}"))?;
            Ok(Prepared {world,initial_intents:input.initial_intents,stock_ownership:input.stock_ownership,
                played:input.played,target:input.target,target_item:input.target_item,combat_rules:input.combat_rules,
                network:input.network,timed_rules:input.timed_rules,motion_rules:input.motion_rules,glyphs:input.glyphs})
        },
        2=> {
            let input:SpreadInput=serde_json::from_slice(bytes).map_err(|error|error.to_string())?;
            let world=World::generate_spread_materials(input.seed,input.config,input.profile)
                .map_err(|error|format!("spread material founding: {error:?}"))?;
            Ok(Prepared {world,initial_intents:input.initial_intents,stock_ownership:input.stock_ownership,
                played:input.played,target:input.target,target_item:input.target_item,combat_rules:input.combat_rules,
                network:input.network,timed_rules:input.timed_rules,motion_rules:input.motion_rules,glyphs:input.glyphs})
        },
        _=>Err("unsupported Eponym material-world startup version".into()),
    }
}
