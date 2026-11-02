// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Necks (ruling 738) over the stance graph, as `places::neck` reads them:
//! a stance is wide when some `w`-square holding it is covered by stances
//! it reaches by climbable steps within the square; wide and narrow
//! stances walk apart.

use super::Profile;
use super::derive::{Scratch, climbable, walk};
use super::stances::Stances;

pub(super) fn split(
    p: &Profile<'_>,
    s: &Stances,
    scratch: &mut Scratch,
    component: &[usize],
    w: i64,
) -> Vec<Vec<usize>> {
    for &i in component {
        scratch.inside[i] = true;
    }
    for &i in component {
        let [x, _, z] = s.at[i];
        scratch.wide[i] = (0..w * w).any(|k| covers(p, s, &scratch.inside, i, [x - k % w, z - k / w], w));
    }
    let mut pieces = Vec::new();
    for &i in component {
        if !scratch.stood[i] {
            let class = scratch.wide[i];
            let (inside, wide) = (&scratch.inside, &scratch.wide);
            pieces.push(walk(p, s, i, &mut scratch.stood, |b| inside[b] && wide[b] == class));
        }
    }
    for &i in component {
        scratch.inside[i] = false;
        scratch.wide[i] = false;
        scratch.stood[i] = false;
    }
    pieces
}

/// Whether stances reached from `from` within the `w`-square at `corner`
/// stand in every one of its columns.
fn covers(p: &Profile<'_>, s: &Stances, inside: &[bool], from: usize, corner: [i64; 2], w: i64) -> bool {
    let holds = |a: [i64; 3]| (corner[0]..corner[0] + w).contains(&a[0]) && (corner[1]..corner[1] + w).contains(&a[2]);
    let mut seen = vec![from];
    let mut columns = vec![[s.at[from][0], s.at[from][2]]];
    let mut next = 0;
    while next < seen.len() {
        let a = seen[next];
        next += 1;
        for &(b, rise) in s.steps(a) {
            if inside[b] && climbable(p, rise) && holds(s.at[b]) && !seen.contains(&b) {
                seen.push(b);
                let column = [s.at[b][0], s.at[b][2]];
                if !columns.contains(&column) {
                    columns.push(column);
                }
            }
        }
    }
    columns.len() as i64 == w * w
}
