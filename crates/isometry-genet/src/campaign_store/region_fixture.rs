//! Explicit exporter for the region lane's headed read: the `mill-ford`
//! quick-start pack (case 1023) founded as a native table whose host holds
//! the GM seat. Test-only, never a shipping campaign preset.

use super::*;
use isocosm::{Command, Execution, Session, schema::Id};
use isocosm_host::world::World;
use isocosm_vtt::{founding::{Campaign, NativeWorld}, native::GameBindings};
use isometry_campaign::*;
use isometry_core::{MapDocument, TurnList};
use isonetry::NativeTable;
use std::{collections::BTreeMap, path::PathBuf};

// The H7 fixture's party handle and relation, so no new keys are made.
const PARTY: &str = "party:band";
const MEMBER: &str = "relation:member-of-band";

/// The smallest valid table content: one map and one storylet, no world facts.
fn draft() -> CampaignDraft {
    let mut world = CampaignWorld::default();
    world.storylets.insert("final".into(), StoryletProposal { key: "final".into(),
        entry: "Mill Ford goes on.".into(), tags: vec![],
        requirements: StoryletRequirements { faction_tags: vec![], hidden_facts: vec![], world_laws: vec![] },
        roles: vec![], effects: vec![] });
    CampaignDraft { id: "mill-ford".into(), name: "Mill Ford".into(), world,
        maps: vec![DraftMap { scale: MapScale::Local, map: LocalMapProposal {
            id: "ford-map".into(), name: "Ford Map".into(), width: 4, height: 3,
            default_ground: "grass".into(), cells: vec![], spawn_zones: vec![], transitions: vec![],
            encounter_anchors: vec![] }, inhabitants: vec![] }],
        secrets: vec![], rewards: vec![], starting_map: "ford-map".into(), final_storylet: "final".into() }
}

fn founded(seed: u64) -> (World, Id, isocosm_vtt::founding::Content,
    Option<isocosm_vtt::founding::GenerationOrigin>) {
    let pack = isocosm::quickstart::pack("mill-ford").unwrap();
    let mut genesis = pack.founding(seed).generate().unwrap();
    pack.lay(&mut genesis).unwrap();
    genesis.rules.relations.insert(MEMBER.into());
    genesis.rules.party_relations.insert(PARTY.into(), MEMBER.into());
    genesis.validate().unwrap();
    let sites: Vec<Id> = genesis.sites.keys().copied().collect();
    let prepared = Campaign::from_draft_bench(genesis, &draft(), Execution::Individuals).unwrap();
    let saved = prepared.campaign.save();
    let content = prepared.campaign.content().clone();
    let origin = saved.origin.clone();
    let world = World::new(Session::load(saved.native, Execution::Individuals).unwrap());
    pack.found(&mut |c| world.command(c), &sites).unwrap();
    let joined = world.command(Command::Join).unwrap();
    let host: Id = joined.strip_prefix("participant:").unwrap().parse().unwrap();
    (world, host, content, origin)
}

/// Run with `ISOMETRY_REGION_FIXTURE_DIR` naming an existing folder; it
/// writes `campaigns/region_quickstart.redb` there, launched with
/// `--campaign region_quickstart` from that folder.
#[test]
#[ignore = "explicit region-lane headed fixture exporter"]
fn export_region_quickstart_for_headed_read() {
    let root = PathBuf::from(std::env::var_os("ISOMETRY_REGION_FIXTURE_DIR")
        .expect("set ISOMETRY_REGION_FIXTURE_DIR to the receipt folder"));
    assert!(root.is_dir(), "the receipt folder must already exist");
    let path = root.join("campaigns").join("region_quickstart.redb");
    assert!(!path.exists(), "refusing to overwrite an existing campaign fixture");
    // A seed nobody chose (Method 1), printed so the run can be found again.
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
    let (world, host, content, origin) = founded(seed);
    let game = GameBindings::decode(&[]).unwrap().encode().unwrap();
    let table = NativeTable { version: 1, game: game.clone(), host_participant: host, peers: vec![], boards: Vec::new() };
    let (session, mut sections) = world.into_parts().unwrap();
    table.validate_history(&session).unwrap();
    let places = session.sim.place_readings().unwrap();
    let native = NativeWorld::from_session(&session, content, origin).unwrap();
    sections.insert("vtt".into(), game);
    let public = GameSnapshot { map: MapDocument::new("Mill Ford", 4, 3), turns: TurnList::new(),
        roll_log: Vec::new(), journal: Vec::new(), inventories: BTreeMap::new(), generations: Vec::new(),
        maps: BTreeMap::new(), active_map: None, world: Default::default(), native: Some(native),
        clocks: BTreeMap::new(), party_cap: isonetry::default_party_cap(), last_beats: Vec::new(),
        beat_seq: 0, applied_actions: Default::default(), native_table: Some(table) };
    let checkpoint = CampaignCheckpoint::new(public.clone(), CampaignStore::new(), Journal::new(), Some(public));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    CampaignRepository::open(&path).unwrap().save_checkpoint(&checkpoint).unwrap();
    println!("REGION_FIXTURE={} seed={seed} places={} residents={:?}", path.display(), places.len(),
        places.iter().map(|p| p.residents.len()).collect::<Vec<_>>());
}
