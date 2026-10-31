// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A player opts their own played body in and out of a party (853, 857).
use super::*;
use cosmograph::party::{self, Membership};

#[test]
fn only_a_players_currently_played_body_opts_in() {
    let w = party_world(101);
    let [a, b, c] = w.bodies;
    let (mut s, p) = table(&w);
    refused(&mut s, join(p[0], b));
    refused(&mut s, join(a, a));
    refused(&mut s, party(party::Change::Join { participant: p[0], party: "party:none".into(), member: a }));
    assert!(matches!(outcome(s.command(join(p[0], a))), party::Outcome::Joined(Membership { participant, member })
        if (participant, member) == (p[0], a)));
    outcome(s.command(join(p[1], b)));
    let members = party::members(&s.sim, BAND).unwrap();
    assert_eq!(members, [Membership { participant: p[0], member: a }, Membership { participant: p[1], member: b }]);
    // Membership grants neither control nor Take: the third player still plays only their own body.
    assert_eq!(s.sim.plays(p[2]), Some(c));
    assert_eq!(s.sim.plays(p[0]), Some(a));
    exact(&s, s.sim.matter());
}

#[test]
fn a_player_withdraws_an_earlier_membership_after_changing_bodies() {
    let w = party_world(102);
    let [a, b, _] = w.bodies;
    let (mut s, p) = table(&w);
    outcome(s.command(join(p[0], a)));
    // The player releases a and takes b: the old membership persists until they withdraw it.
    s.command(Command::Release(a)).unwrap();
    s.command(Command::Release(b)).unwrap();
    s.command(Command::Take { participant: p[0], critter: b }).unwrap();
    assert_eq!(s.sim.plays(p[0]), Some(b));
    assert_eq!(party::members(&s.sim, BAND).unwrap(), [Membership { participant: p[0], member: a }]);
    refused(&mut s, join(p[0], a));
    refused(&mut s, disclose(p[0], a, &[w.home]));
    refused(&mut s, leave(p[1], a));
    refused(&mut s, leave(p[0], b));
    assert!(matches!(outcome(s.command(leave(p[0], a))), party::Outcome::Left(_)));
    assert!(party::members(&s.sim, BAND).unwrap().is_empty());
    refused(&mut s, leave(p[0], a));
    exact(&s, s.sim.matter());
}
