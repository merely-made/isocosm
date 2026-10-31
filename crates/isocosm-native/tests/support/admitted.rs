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
    /// The admitted characters' native ids, the player's first.
    pub characters: [Id; 2],
}

pub fn admitted_table() -> Admitted {
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
    let placements = KEYS.iter().zip(&stands).zip(ids).map(|((key, stand), id)| {
        let window = cosmograph::spatial::embodiment::site_window(&template, stand.pose.at);
        let volume = session.sim.volume(site, window.min, window.max, window.depth).unwrap();
        let places = isometer_space::places::Places::derive(&volume, template.places).unwrap();
        // The highest clear cell in the resident's column, inside the window.
        let at = (volume.datum..volume.ceiling()).rev().map(|y| [stand.pose.at[0], y, stand.pose.at[2]])
            .find(|at| volume.material(*at) == Some(volume.materials.air) && places.place_at(*at).is_some())
            .expect("a clear cell in the column");
        let patch = places.place_at(at).unwrap();
        CharacterPlacement {
            character: (*key).into(), site, patch,
            body_revision: session.sim.state().population.get(id).unwrap().body_revision,
            terrain_revision: session.sim.terrain_revision(), at,
            window: PlacementWindow { min: window.min, max: window.max, depth: window.depth },
            places: template.places,
        }
    }).collect();
    let controls = vec![
        CharacterControl { character: KEYS[0].into(), participant: ParticipantInput::Existing(player.0) },
        CharacterControl { character: KEYS[1].into(), participant: ParticipantInput::Existing(dm.0) },
    ];
    submit(&mut session, &mut sections, &AdmissionBatch { version: 2, assertions: vec![], characters: vec![],
        placements, controls, project_carry: None, spatial: vec![] });
    Admitted { world: World::with_sections(session, sections), dm, player, characters: ids }
}
