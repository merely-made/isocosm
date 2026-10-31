// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn part(cells: &[(&str, u32)]) -> Part {
    let cells: std::collections::BTreeMap<Key, u32> = cells
        .iter()
        .map(|(f, n)| (format!("function:{f}"), *n))
        .collect();
    Part {
        functions: cells.keys().cloned().collect(),
        cells,
        ..Default::default()
    }
}

#[test]
fn the_path_steps_only_to_neighbours_and_covers_the_lattice() {
    let d = dims([4, 3, 2]);
    assert_eq!(d, [3, 2, 2]);
    let p = path(d);
    assert_eq!(p.len(), 12);
    assert!(p.windows(2).all(|w| neighbours(d, w[0]).contains(&w[1])));
}

#[test]
fn sync_lays_out_the_counts_and_keeps_identity_as_they_move() {
    let half = [4, 1, 1];
    let mut p = part(&[("contract", 2), ("store", 1)]);
    sync(&mut p, half);
    assert!(agrees(&p, half).is_ok());
    let before = p.tracts.clone();
    *p.cells.get_mut("function:contract").unwrap() -= 1;
    *p.cells.get_mut("function:store").unwrap() += 1;
    sync(&mut p, half);
    assert!(agrees(&p, half).is_ok());
    let kept = |f: &str| {
        p.tracts
            .iter()
            .find(|t| t.function == f)
            .unwrap()
            .cells
            .clone()
    };
    let had = |f: &str| {
        before
            .iter()
            .find(|t| t.function == f)
            .unwrap()
            .cells
            .clone()
    };
    assert!(
        had("function:store")
            .iter()
            .all(|c| kept("function:store").contains(c))
    );
}

#[test]
fn a_proposal_places_cells_and_refuses_a_broken_tract() {
    let half = [4, 1, 1];
    let mut p = part(&[("contract", 3)]);
    sync(&mut p, half);
    let cost = propose(
        &mut p,
        half,
        &[
            ("function:store".into(), vec![CellId(0)]),
            ("function:contract".into(), vec![CellId(1), CellId(2)]),
        ],
    )
    .unwrap();
    assert_eq!(cost, 1, "one cell changed what it does");
    assert_eq!(p.cells["function:store"], 1);
    let split = vec![CellId(0), CellId(2)];
    assert!(propose(&mut p, half, &[("function:store".into(), split)]).is_err());
}
