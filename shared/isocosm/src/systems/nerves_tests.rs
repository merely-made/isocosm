// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::tests::{grazer, part, rules};
use super::*;
use crate::rules::Role;

const INTAKE: (&str, Role) = ("function:intake", Role::Source);
const CONTRACT: (&str, Role) = ("function:contract", Role::Effect);

#[test]
fn a_grazer_with_an_eye_joins_its_limbs_to_its_intake() {
    let e = grazer();
    assert!(
        e.systems.contains_key("system:nervous"),
        "a sense realizes it"
    );
    let (reached, total) = joined(&e, &rules(10), INTAKE, CONTRACT, None);
    assert!(total > 0);
    assert_eq!(reached, total);
}

#[test]
fn without_nerves_or_a_sense_nothing_joins() {
    let mut numb = grazer();
    numb.systems.remove("system:nervous");
    let mut blind = grazer();
    blind.parts.remove(&PartId(3));
    for e in [numb, blind] {
        let (reached, total) = joined(&e, &rules(10), INTAKE, CONTRACT, None);
        assert_eq!(reached, 0);
        assert!(total > 0, "the limbs are still there");
    }
}

#[test]
fn the_joint_share_is_the_contracting_cells_the_senses_reach() {
    // The second limb hangs from the lump through a joint that cannot
    // carry, so the senses reach one limb of two.
    let mut e = grazer();
    let (frame, joint) = part(Some(0), [1, 1, 1], &[]);
    let joint = e.add_part(&frame, joint).unwrap();
    let doc = e.body.as_mut().unwrap();
    doc.parts[2].attachment.as_mut().unwrap().parent = joint;
    let r = rules(10);
    assert_eq!(capacity(e.extent(joint), &e.parts[&joint], &r), 0);
    let (reached, total) = joined(&e, &r, INTAKE, CONTRACT, None);
    assert_eq!((reached, total), (2, 4));
    // With the joint able to carry, both are reached.
    e.parts
        .get_mut(&joint)
        .unwrap()
        .cells
        .insert("function:gate".into(), 1);
    assert_eq!(joined(&e, &r, INTAKE, CONTRACT, None), (4, 4));
}
