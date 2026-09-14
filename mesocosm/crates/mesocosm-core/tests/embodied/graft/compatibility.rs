// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn an_over_cap_disfavoured_carry_is_refused_and_regrowth_is_the_route_that_remains() {
    // The third verdict, and the wing contract's rule for it: an incompatible
    // carry is refused or redirected to regrowth, never silently rewritten. So
    // the refusal names the boundary, and the other crossing still lands.
    let mut world = world_with(Verdict::Refused);
    let (frond, _) = donor(&mut world);
    let before = mesocosm_core::state_hash(&world);
    let requested_mg = corpse_of(&world)
        .phenotype
        .harvest(frond)
        .expect("the branch remains whole")
        .mass_mg();
    let recipient = world.controlled().expect("embodied");
    let allowance_mg = recipient.phenotype.cell_mg(recipient.body().root);

    let refused = take(&mut world, frond, Crossing::Carry);
    assert_eq!(
        refused,
        Outcome::Rejected(Rejection::GraftAllowance {
            requested_mg,
            allowance_mg,
        }),
        "the default one-cell allowance refuses this larger carried branch"
    );
    assert!(
        corpse_of(&world).body().is_living(frond),
        "and the corpse still has its branch"
    );

    let outcome = take(&mut world, frond, Crossing::Regrow);
    let Outcome::Grafted { root, verdict, .. } = outcome else {
        panic!("regrowth is supposed to be feasible: {outcome:?}");
    };
    assert_eq!(
        verdict,
        Verdict::Refused,
        "the table did not change its mind"
    );
    let phenotype = world.phenotype().unwrap();
    assert!(
        phenotype.expresses_on(root, fixing()),
        "a regrown plate does what this body's rules make of a plate"
    );
    assert!(
        !phenotype.expresses_on(root, gland()),
        "and not what the donor had arranged on it: regrowing is not carrying"
    );
    // Regrowing preserves identity and provenance and realizes the phenotype
    // under the destination's rules — the wing contract's own words for this
    // route, and the half of it a rebuilt allocation must not quietly drop.
    assert_eq!(
        phenotype
            .body()
            .part(root)
            .map(|part| &part.provenance.origin),
        Some(&Origin::Incorporated {
            from_species: DONOR_LINE,
            from_part: frond,
        })
    );
    assert_ne!(
        mesocosm_core::state_hash(&world),
        before,
        "the refused carry and the landed regrowth are different worlds"
    );
}
