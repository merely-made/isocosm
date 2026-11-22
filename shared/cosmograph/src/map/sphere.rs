// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Six consistently oriented square faces. The rectangular atlas stores
//! them in two rows of three. Only the folded edges are simulation routes.

/// For each face, its outward normal, right and down in globe coordinates.
pub const FACES: [[[i8; 3]; 3]; 6] = [
    [[0, 0, 1], [1, 0, 0], [0, -1, 0]],
    [[1, 0, 0], [0, 0, -1], [0, -1, 0]],
    [[0, 0, -1], [-1, 0, 0], [0, -1, 0]],
    [[-1, 0, 0], [0, 0, 1], [0, -1, 0]],
    [[0, 1, 0], [1, 0, 0], [0, 0, 1]],
    [[0, -1, 0], [1, 0, 0], [0, 0, -1]],
];

fn outward(face: usize, side: u8) -> [i8; 3] {
    let [_, right, down] = FACES[face];
    match side {
        0 => down.map(|v| -v),
        1 => right,
        2 => down,
        _ => right.map(|v| -v),
    }
}

/// Atlas column, row and receiving edge after one step.
pub(super) fn across(n: u32, c: u32, r: u32, side: u8) -> (u32, u32, u8) {
    let (x, y) = (c % n, r % n);
    let back = (side + 2) % 4;
    match side {
        0 if y > 0 => return (c, r - 1, back),
        1 if x + 1 < n => return (c + 1, r, back),
        2 if y + 1 < n => return (c, r + 1, back),
        3 if x > 0 => return (c - 1, r, back),
        _ => {},
    }
    let face = (r / n * 3 + c / n) as usize;
    let next = FACES
        .iter()
        .position(|axes| axes[0] == outward(face, side))
        .unwrap();
    let enters = (0..4)
        .find(|&s| outward(next, s) == FACES[face][0])
        .unwrap();
    // Clockwise distance along an oriented edge reverses across a seam.
    let last = n - 1;
    let along = last - [x, y, last - x, last - y][side as usize];
    let (nx, ny) = [
        (along, 0),
        (last, along),
        (last - along, last),
        (0, last - along),
    ][enters as usize];
    (next as u32 % 3 * n + nx, next as u32 / 3 * n + ny, enters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_edge_returns_through_its_peer_and_corners_close_on_a_sphere() {
        for n in 1..=6 {
            let mut vertices = BTreeSet::new();
            for r in 0..2 * n {
                for c in 0..3 * n {
                    for side in 0..4 {
                        let (nc, nr, enter) = across(n, c, r, side);
                        assert_eq!(across(n, nc, nr, enter), (c, r, side));
                        let mut corner = (c, r, side);
                        let mut slots = BTreeSet::new();
                        while slots.insert(corner) {
                            let (x, y, e) = across(n, corner.0, corner.1, corner.2);
                            corner = (x, y, (e + 1) % 4);
                        }
                        assert_eq!(corner, (c, r, side));
                        assert!((3..=4).contains(&slots.len()));
                        vertices.insert(slots);
                    }
                }
            }
            let faces = 6 * n * n;
            assert_eq!(vertices.len() as u32 + faces, 2 * faces + 2);
            assert_eq!(vertices.iter().filter(|c| c.len() == 3).count(), 8);
        }
    }
}
