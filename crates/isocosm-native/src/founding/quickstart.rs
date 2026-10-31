// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A quick-start world founded as a campaign in one step (case 1023; Mark: "Picker + packs from disk"):
//! the pack laid over its drawn world, its founders entered by the founding
//! door at the first tick, and the smallest table content beside it.

use super::*;
use isocosm_campaign::{CampaignWorld, DraftMap, LocalMapProposal, MapScale, StoryletProposal,
    StoryletRequirements};

/// One map and one storylet named for the pack, and no world facts: the
/// world's facts are the pack's, entered natively.
pub fn quickstart_draft(pack: &cosmograph::quickstart::Pack) -> CampaignDraft {
    let mut world = CampaignWorld::default();
    world.storylets.insert("final".into(), StoryletProposal { key: "final".into(),
        entry: format!("{} goes on.", pack.name), tags: vec![],
        requirements: StoryletRequirements { faction_tags: vec![], hidden_facts: vec![], world_laws: vec![] },
        roles: vec![], effects: vec![] });
    let map = format!("{}-map", pack.key);
    CampaignDraft { id: pack.key.clone(), name: pack.name.clone(), world,
        maps: vec![DraftMap { scale: MapScale::Local, map: LocalMapProposal {
            id: map.clone(), name: pack.name.clone(), width: 4, height: 3,
            default_ground: "grass".into(), cells: vec![], spawn_zones: vec![], transitions: vec![],
            encounter_anchors: vec![] }, inhabitants: vec![] }],
        secrets: vec![], rewards: vec![], starting_map: map, final_storylet: "final".into() }
}

impl Campaign {
    /// Found the quick-start pack `text` over a world drawn under `seed`.
    pub fn from_quickstart_for(text: &str, seed: u64, mode: Execution, purpose: Purpose,
        offered: Option<Offered<'_>>) -> Result<Prepared, FoundingError> {
        let native = |reason| FoundingError::Native(ReplayError { phase: crate::native::Phase::Founding, reason });
        let pack = cosmograph::quickstart::parse(text).map_err(native)?;
        let (genesis, sites) = pack.genesis(seed).map_err(native)?;
        let world = cosmograph_host::founding::found(table_genesis(genesis), mode, purpose, offered)
            .map_err(native)?;
        pack.found(&mut |command| world.command(command), &sites).map_err(native)?;
        let replay = Replay::from_world_commands(world, []).map_err(FoundingError::Native)?;
        let content = Content::from_draft(&quickstart_draft(&pack))?;
        Ok(Prepared { campaign: Self { replay, content, origin: None }, private: CampaignStore::new() })
    }
}
