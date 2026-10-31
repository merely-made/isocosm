//! Explicit exporter for the H7 headed UI control. This is test-only Bench
//! founding, never a shipping campaign preset or head-admission claim.

use super::*;
use cosmograph::{
    Command, Execution, Founding, Session,
    schema::{Id, Method},
};
use cosmograph_host::world::World;
use cosmograph_overlay::{ParticipantHandle, Seat};
use isocosm_native::{
    VttEndpoint,
    founding::{Campaign, NativeWorld},
    native::GameBindings,
};
use isocosm_core::{MapDocument, TurnList};
use isonetry::{GameSnapshot, NativeTable};
use std::{collections::BTreeMap, fs::OpenOptions, io::Write, path::PathBuf};

#[path = "../../../isocosm-native/tests/native_founding/fixture.rs"]
mod campaign_content;

const BAND: &str = "party:band";
const MEMBER: &str = "relation:member-of-band";
const VTT_SECTION: &str = "vtt";

fn native_table_fixture() -> (
    World,
    Id,
    Id,
    isocosm_native::founding::Content,
    Option<isocosm_native::founding::GenerationOrigin>,
) {
    // Keep the seed, population, played lineage, relation and body-taking
    // sequence aligned with the official shared VTT endpoint fixture.
    let mut genesis = Founding {
        seed: 11,
        sites: 3,
        population: 24,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies: Some(Default::default()),
        played: Some(cosmograph::directing::Played {
            lineage: 1,
            region_sites: 1,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    genesis.rules.relations.insert(MEMBER.into());
    genesis
        .rules
        .party_relations
        .insert(BAND.into(), MEMBER.into());
    genesis.validate().unwrap();

    // The official VTT campaign fixture supplies structurally validated
    // content. Founding still uses the table fixture's actual seed-11 genesis.
    let prepared =
        Campaign::from_draft_bench(genesis, &campaign_content::draft(), Execution::Individuals)
            .unwrap();
    let saved = prepared.campaign.save();
    let content = prepared.campaign.content().clone();
    let origin = saved.origin.clone();
    let session = Session::load(saved.native, Execution::Individuals).unwrap();
    let world = World::new(session);
    let bodies: Vec<Id> = world
        .read(|session| {
            session
                .sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(_, group)| group.entity.method == Method::Deliberative)
                .map(|(first, _)| *first)
                .take(2)
                .collect()
        })
        .unwrap();
    assert_eq!(
        bodies.len(),
        2,
        "the seed-11 table fixture supplies two bodies"
    );

    let mut participants = Vec::new();
    for body in bodies {
        let joined = world.command(Command::Join).unwrap();
        let participant: Id = joined
            .strip_prefix("participant:")
            .unwrap()
            .parse()
            .unwrap();
        world
            .command(Command::Take {
                participant,
                critter: body,
            })
            .unwrap();
        participants.push(participant);
    }
    (world, participants[0], participants[1], content, origin)
}

fn game_snapshot(native: NativeWorld, table: NativeTable) -> GameSnapshot {
    GameSnapshot {
        map: MapDocument::new("H7 Bench Table", 4, 3),
        turns: TurnList::new(),
        roll_log: Vec::new(),
        journal: Vec::new(),
        inventories: BTreeMap::new(),
        generations: Vec::new(),
        maps: BTreeMap::new(),
        active_map: None,
        world: Default::default(),
        native: Some(native),
        clocks: BTreeMap::new(),
        party_cap: isonetry::default_party_cap(),
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
        native_table: Some(table),
    }
}

/// Run with `ISOMETRY_H7_RECEIPT_DIR` set to the existing H7 receipt owner.
/// The fixed stem is create-only: a prior fixture is never replaced.
#[test]
#[ignore = "explicit Bench-only H7 headed fixture exporter"]
fn export_seed11_bench_table_for_headed_gate() {
    let receipt_root = PathBuf::from(
        std::env::var_os("ISOMETRY_H7_RECEIPT_DIR")
            .expect("set ISOMETRY_H7_RECEIPT_DIR to the existing H7 receipt folder"),
    );
    assert!(receipt_root.is_dir(), "receipt owner must already exist");
    let campaign_dir = receipt_root.join("campaigns");
    let campaign_path = campaign_dir.join("h7_vtt_bench.redb");
    let manifest_path = receipt_root.join("h7_vtt_bench.fixture.json");
    assert!(
        !campaign_path.exists(),
        "refusing to overwrite an existing campaign fixture"
    );
    assert!(
        !manifest_path.exists(),
        "refusing to overwrite an existing fixture manifest"
    );

    let (world, host, player, content, origin) = native_table_fixture();
    let game = GameBindings::decode(&[]).unwrap().encode().unwrap();
    let table = NativeTable {
        version: 1,
        game: game.clone(),
        host_participant: host,
        peers: vec![isonetry::native_table::PeerParticipant {
            endpoint: [0x11; 32],
            participant: player,
        }],
        boards: Vec::new(),
    };
    let (session, mut sections) = world.into_parts().unwrap();
    table.validate_history(&session).unwrap();
    let native = NativeWorld::from_session(&session, content.clone(), origin).unwrap();
    sections.insert(VTT_SECTION.into(), game);
    let world = World::with_sections(session, sections);
    let _endpoint = VttEndpoint::open(&world, ParticipantHandle(host), Seat::Dm).unwrap();

    let public = game_snapshot(native, table);
    let checkpoint = CampaignCheckpoint::new(
        public.clone(),
        CampaignStore::new(),
        Journal::new(),
        Some(public),
    );
    std::fs::create_dir_all(&campaign_dir).unwrap();
    CampaignRepository::open(&campaign_path)
        .unwrap()
        .save_checkpoint(&checkpoint)
        .unwrap();
    let restored = CampaignRepository::open(&campaign_path)
        .unwrap()
        .load_checkpoint()
        .unwrap()
        .expect("saved campaign fixture reopens");
    assert_eq!(restored, checkpoint);
    let restored_native = restored
        .public
        .native
        .as_ref()
        .unwrap()
        .load(Execution::Grouped)
        .unwrap();
    assert_eq!(restored_native.content(), &content);

    let manifest = serde_json::json!({
        "purpose": "Bench-only headed UI fixture; not a product-head founding claim",
        "campaign": "h7-vtt-bench",
        "seed": 11,
        "source": "official isocosm-native native_founding draft content and seed-11 table fixture semantics",
        "native": "actual accepted Session serialized with NativeWorld::from_session",
        "table": "validated NativeTable/GameBindings with explicit host and peer participant handles",
        "headed_command": "isocosm-genet.exe --campaign h7-vtt-bench",
        "action_sequence": ["TABLE: click the advertised party selection action", "TABLE: click an advertised body join action"]
    });
    let mut manifest_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest_path)
        .expect("fixture manifest stem is fresh");
    manifest_file
        .write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
        .unwrap();
    println!(
        "H7_BENCH_FIXTURE={} campaign={} native_cursor={:?} projections=6",
        campaign_path.display(),
        "h7-vtt-bench",
        restored.public.native.as_ref().unwrap().cursor().unwrap(),
    );
}
