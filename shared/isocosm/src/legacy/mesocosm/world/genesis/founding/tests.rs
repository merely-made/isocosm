// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A founding is a name in the datasheet, and worlds founded from the sheet
//! are the worlds the code-built rosters founded.

use super::*;
use crate::legacy::mesocosm::snapshot::state_hash;
use crate::legacy::mesocosm::world::{Intent, World};

const NAMED: [Founding; 8] = [
    Founding::Drawn,
    Founding::BrowsingConsumer,
    Founding::RosterStand,
    Founding::RosterFauna,
    Founding::Roster,
    Founding::BranchingRoster,
    Founding::JointedRoster,
    Founding::SpacedRoster,
];

/// Each founding at two seeds, 12 founders: the state hash at founding and
/// after 40 idle ticks. Recorded on 2026-10-06 at `18945c84`, while the code
/// still built every roster (`Code/testing/isometry-datasheets-p3b-before.txt`).
/// Founding from the sheets has to reproduce every one.
#[rustfmt::skip]
const RECEIPT: [(Founding, u64, u64, u64); 16] = [
    (Founding::Drawn, 7, 0xf479e4083e5b071d, 0xea571fb3dec849af),
    (Founding::Drawn, 4242, 0xe9faf81b79363f3d, 0xf6cf703565313a56),
    (Founding::BrowsingConsumer, 7, 0xe7bcd940eb7a11d0, 0x790b2cb6f98f8673),
    (Founding::BrowsingConsumer, 4242, 0xe79aee755d3c32c0, 0xdf2c89d527d157ac),
    (Founding::RosterStand, 7, 0x997752c3b0b082d9, 0xe01908764a1385f0),
    (Founding::RosterStand, 4242, 0xb2d0db2e32b5ba25, 0x9399a134404684c9),
    (Founding::RosterFauna, 7, 0x324cd33ccaada744, 0x42e45f8241211faf),
    (Founding::RosterFauna, 4242, 0xd0df9cd95fdb80d2, 0x36555ec8b6edd79e),
    (Founding::Roster, 7, 0xe73900114296105f, 0xd5f1075749777c73),
    (Founding::Roster, 4242, 0x3627d9f7e3c1a431, 0xd794e7b884861afc),
    (Founding::BranchingRoster, 7, 0xc2cfedfa63ad42e4, 0xe57aaa679b486075),
    (Founding::BranchingRoster, 4242, 0x57cc4a59a7345f4c, 0xbd93fa4e634777d0),
    (Founding::JointedRoster, 7, 0x0659fe8069291daa, 0x57c10e96508ca6f4),
    (Founding::JointedRoster, 4242, 0xf8ffa66a427fef7b, 0xab44b0a890edb843),
    (Founding::SpacedRoster, 7, 0x75ddbf87bc831f5c, 0xef32da73cb3f6734),
    (Founding::SpacedRoster, 4242, 0x306aa3b53ac11d47, 0xe8f989da5e26fe14),
];

#[test]
fn every_founding_reproduces_its_recorded_world() {
    for (founding, seed, founded, idled) in RECEIPT {
        let mut world = World::founded(seed, 12, founding).expect("founds");
        assert_eq!(
            state_hash(&world),
            founded,
            "{founding:?} at seed {seed}, founded"
        );
        for _ in 0..40 {
            world.apply(Intent::Idle);
        }
        assert_eq!(
            state_hash(&world),
            idled,
            "{founding:?} at seed {seed}, idled"
        );
    }
}

#[test]
fn the_named_foundings_are_the_sheets_and_the_roster_ships() {
    let mut named: Vec<&str> = NAMED.iter().map(|founding| founding.name()).collect();
    named.sort();
    let declared: Vec<&str> = Founding::all().map(Founding::name).collect();
    assert_eq!(named, declared);
    assert_eq!(Founding::default(), Founding::Roster);
}

#[test]
fn a_founding_reads_as_its_bare_name() {
    assert_eq!(format!("{:?}", Founding::SpacedRoster), "SpacedRoster");
    assert_eq!(
        Founding::named("SpacedRoster"),
        Some(Founding::SpacedRoster)
    );
    assert_eq!(Founding::named("Spaced"), None);
}

#[test]
fn only_the_drawn_founding_draws_every_tier() {
    for founding in NAMED {
        let authored: usize = [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer]
            .into_iter()
            .map(|kingdom| founding.tier(kingdom).len())
            .sum();
        assert_eq!(authored == 0, founding == Founding::Drawn, "{founding:?}");
    }
    assert_eq!(Founding::Drawn.palette(), PartPalette::primitive());
}
