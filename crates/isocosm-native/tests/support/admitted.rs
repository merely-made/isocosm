// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A table whose two characters came in through native recipe admission,
//! placement and control (case 1042): each has the table position binding a
//! native cell batch moves. Built over the physical table fixture, placed on
//! the clear cells its residents already stand in.

#![allow(dead_code)]

use super::table;
use cosmograph::{
    Command,
    asserted::{Assertion, Character},
    schema::{Id, Method},
};
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use cosmograph_host::world::World;
use isocosm_native::native::{
    AdmissionBatch, CharacterAdmission, CharacterControl, CharacterPlacement, GameBindings,
    ParticipantInput, PlacementWindow, submit_game_access,
};
use std::collections::BTreeMap;

pub const KEYS: [&str; 2] = ["arden", "brisa"];

pub struct Admitted {
    pub world: World,
    pub dm: ParticipantHandle,
    pub player: ParticipantHandle,
    /// The admitted characters in Arden, Brisa order.
    pub characters: [Id; 2],
}

pub fn admitted_table() -> Admitted { admitted_table_controlled([true, false]) }

/// The bench with each admitted character controlled by the player (`true`)
/// or the DM, beside the body each already plays (case 1065).
pub fn admitted_table_controlled(by_player: [bool; 2]) -> Admitted {
    placed_table(by_player, None)
}

/// A table with an explicitly admitted physical eye for each player character.
/// Both use the same native window so their geometry shares one patch reading.
pub fn sighted_table(by_player: [bool; 2], range: u64) -> Admitted { placed_table(by_player, Some(range)) }

fn placed_table(by_player: [bool; 2], sight: Option<u64>) -> Admitted {
    let table::Table { world, dm, player, bodies } = table::table_with_native_geometry();
    let (mut session, mut sections) = world.into_parts().unwrap();
    sections.entry("vtt".into()).or_insert_with(|| GameBindings::decode(&[]).unwrap().encode().unwrap());
    for (key, name) in KEYS.iter().zip(["Arden", "Brisa"]) {
        session.command(Command::Assert(Assertion::Character(Character {
            key: (*key).into(), name: name.into(), tags: Default::default(), faction: None, place: None, table: None,
        }))).unwrap();
    }
    let lineage = session.sim.state().lineages.iter()
        .find(|(_, l)| l.development.is_some()).map(|(k, _)| k.clone()).expect("a lineage with a native recipe");
    let template = session.sim.genesis().rules.spatial_templates.clone().unwrap();
    let stands: Vec<_> = bodies.iter().map(|b| session.sim.state().spatial_poses[b].clone()).collect();
    let site = stands[0].site;
    let access = |participant: ParticipantHandle, session: &cosmograph::Session| OrdinaryAccess {
        participant, seat: Seat::Dm, subject: session.sim.plays(participant.0).map(EntityHandle),
    };
    let submit = |session: &mut cosmograph::Session, sections: &mut BTreeMap<String, Vec<u8>>, batch: &AdmissionBatch| {
        let bytes = serde_json::to_vec(batch).unwrap();
        let game = sections["vtt"].clone();
        let who = access(dm, session);
        let (_, next) = submit_game_access(session, game, &bytes, who).unwrap();
        sections.insert("vtt".into(), next);
    };
    let characters = KEYS.map(|key| CharacterAdmission {
        character: key.into(), lineage: lineage.clone(), seed: 7, site, accounts: Default::default(),
        method: Method::Deliberative, traits: Default::default(), skills: Default::default(), disposition: [0; 5],
    });
    submit(&mut session, &mut sections, &AdmissionBatch { version: 1, assertions: vec![],
        characters: characters.to_vec(), placements: vec![], controls: vec![], project_carry: None, spatial: vec![] });
    let ids = KEYS.map(|key| session.sim.authored_entity(key).unwrap().0);
    let shared_window = stands.iter().map(|stand| cosmograph::spatial::embodiment::site_window(&template, stand.pose.at))
        .reduce(|mut a, b| {
            for k in 0..2 { a.min[k] = a.min[k].min(b.min[k]); a.max[k] = a.max[k].max(b.max[k]); }
            a.depth = a.depth.max(b.depth);
            a
        }).unwrap();
    let placements: Vec<_> = KEYS.iter().zip(&stands).zip(ids).map(|((key, stand), id)| {
        let window = if sight.is_some() { shared_window.clone() } else {
            cosmograph::spatial::embodiment::site_window(&template, stand.pose.at)
        };
        let volume = session.sim.volume(site, window.min, window.max, window.depth).unwrap();
        let places = isometer_space::places::Places::derive(&volume, template.places).unwrap();
        let [x, _, z] = stand.pose.at;
        let at = if sight.is_some() {
            // The marked table starts on the surface rather than at its window ceiling.
            let floor = volume.surface(x, z).expect("grounded stand column");
            let at = [x, floor + 1, z];
            assert!(volume.material([x, floor, z]).is_some_and(|m| m != volume.materials.air && m != volume.materials.water));
            assert_eq!(volume.material(at), Some(volume.materials.air));
            at
        } else {
            // Existing unmarked fixtures and saves may occupy any admitted clear cell.
            (volume.datum..volume.ceiling()).rev().map(|y| [x, y, z])
                .find(|at| volume.material(*at) == Some(volume.materials.air) && places.place_at(*at).is_some())
                .expect("legacy clear cell")
        };
        let patch = places.place_at(at).unwrap();
        CharacterPlacement {
            character: (*key).into(), site, patch,
            body_revision: session.sim.state().population.get(id).unwrap().body_revision,
            terrain_revision: session.sim.terrain_revision(), at,
            window: PlacementWindow { min: window.min, max: window.max, depth: window.depth },
            places: template.places,
        }
    }).collect();
    let controls = KEYS.iter().zip(by_player).map(|(key, mine)| CharacterControl { character: (*key).into(),
        participant: ParticipantInput::Existing(if mine { player.0 } else { dm.0 }) }).collect();
    let (version, placements, spatial) = match sight {
        None => (2, placements, vec![]),
        Some(range) => (4, vec![], placements.into_iter().map(|placement| isocosm_native::native::CharacterSpatialPlacement {
            placement, spatial: isocosm_native::native::SpatialSense {
                yaw: cosmograph::custody::Yaw::Zero, eye_offset: [0, 0, 0], range,
            },
        }).collect()),
    };
    submit(&mut session, &mut sections, &AdmissionBatch { version, assertions: vec![], characters: vec![],
        placements, controls, project_carry: None, spatial });
    Admitted { world: World::with_sections(session, sections), dm, player, characters: ids }
}
