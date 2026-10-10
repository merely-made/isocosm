// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use serde::Serialize;

/// The exact Part field order before ruling 810's loss records. This is
/// a hashing control, never a bypass of a stored save's verification.
#[derive(Serialize)]
struct PreviousPart<'a> {
    traits: &'a BTreeSet<Key>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    functions: &'a BTreeSet<Key>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    cells: &'a BTreeMap<Key, u32>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    matter: &'a Ledger,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tracts: &'a Vec<isocosm::mosaic::Tract>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    lost: &'a Vec<isocosm::mosaic::CellId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: &'a Option<isocosm::process::IntakePort>,
}

fn previous(p: &Part) -> PreviousPart<'_> {
    PreviousPart {
        traits: &p.traits,
        functions: &p.functions,
        cells: &p.cells,
        matter: &p.matter,
        tracts: &p.tracts,
        lost: &p.lost,
        port: &p.port,
    }
}

#[test]
fn absent_loss_records_preserve_the_previous_json_and_postcard_preimages() {
    let mut g = fixture(false);
    let mut s = Session::new(g.clone(), Execution::Individuals).unwrap();
    s.command(Command::Wound {
        entity: 1,
        part: Some(PartId(1)),
        cells: 1,
    })
    .unwrap();
    g.population = s.sim.state().population.clone();
    for mut p in g
        .population
        .get(1)
        .unwrap()
        .parts
        .values()
        .cloned()
        .chain([Part::default()])
    {
        p.losses.clear();
        let old = previous(&p);
        assert_eq!(
            serde_json::to_vec(&old).unwrap(),
            serde_json::to_vec(&p).unwrap()
        );
        assert_eq!(
            postcard::to_allocvec(&old).unwrap(),
            postcard::to_allocvec(&p).unwrap()
        );
        assert_eq!(
            state_witness::hash_value(&old).unwrap(),
            state_witness::hash_value(&p).unwrap()
        );
        // JSON is the native stored Part format; postcard is its witness
        // preimage, not a native Part archive decoder.
        let loaded: Part = serde_json::from_slice(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert_eq!(loaded, p);
    }
}

#[test]
fn recorded_free_and_function_losses_change_both_hashes_and_roundtrip_in_json() {
    let mut s = Session::new(fixture(false), Execution::Individuals).unwrap();
    s.command(Command::Wound {
        entity: 1,
        part: Some(PartId(1)),
        cells: 1,
    })
    .unwrap();
    let mut p = s.sim.state().population.get(1).unwrap().parts[&PartId(1)].clone();
    for allocation in [None, Some("function:contract".into())] {
        p.losses.insert(p.lost[0], allocation);
        let old = previous(&p);
        assert_ne!(isocosm::digest(&old), isocosm::digest(&p));
        assert_ne!(
            state_witness::hash_value(&old).unwrap(),
            state_witness::hash_value(&p).unwrap()
        );
        let loaded: Part = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        assert_eq!(loaded, p);
    }
}
