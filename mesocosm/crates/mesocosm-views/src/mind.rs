// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A lineage's mind in words: its drive toward each need the world's sheet
//! names, against the written score's, and what it leans toward doing when
//! nothing presses. Every figure is read off the policy by the sim.

use cosmograph::directing::policy::Leaning;

/// The shapes of act a policy reads, in its order.
const SHAPES: [&str; 4] = ["moving", "eating another body", "working the ground", "acting on another body"];

fn percent(mille: i32) -> i32 {
    (mille + mille.signum() * 5) / 10
}

fn need(key: &str) -> &str {
    key.trim_start_matches("need:")
}

/// The mind as it stands: each drive, then each lean that rounds to something.
pub fn mind_words(leaning: &Leaning) -> String {
    let mut out: Vec<String> =
        leaning.drives.iter().map(|(key, mille)| format!("{} {}%", need(key), percent(*mille))).collect();
    for (shape, mille) in SHAPES.iter().zip(leaning.shapes) {
        match percent(mille) {
            0 => {},
            p if p > 0 => out.push(format!("drawn to {shape} {p}%")),
            p => out.push(format!("shy of {shape} {}%", -p)),
        }
    }
    out.join(" · ")
}

/// What `offered` would change of `had`, signed; empty where nothing would
/// show.
pub fn mind_change_words(had: &Leaning, offered: &Leaning) -> String {
    let signed = |name: &str, from: i32, to: i32| {
        let d = percent(to) - percent(from);
        (d != 0).then(|| format!("{name} {d:+}%"))
    };
    let drives = had.drives.iter().zip(&offered.drives).filter_map(|((key, a), (_, b))| signed(need(key), *a, *b));
    let shapes = SHAPES.iter().zip(had.shapes.iter().zip(offered.shapes)).filter_map(|(s, (a, b))| signed(s, *a, b));
    drives.chain(shapes).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mind_reads_as_drives_then_leans_and_a_change_as_signed_differences() {
        let had = Leaning { drives: vec![("need:hunger".into(), 1000), ("need:wound".into(), 875)], shapes: [0, 0, 0, 0] };
        let offered = Leaning { drives: vec![("need:hunger".into(), 1125), ("need:wound".into(), 875)], shapes: [60, 0, -40, 2] };
        assert_eq!(mind_words(&had), "hunger 100% · wound 88%");
        assert_eq!(mind_words(&offered), "hunger 113% · wound 88% · drawn to moving 6% · shy of working the ground 4%");
        assert_eq!(mind_change_words(&had, &offered), "hunger +13%, moving +6%, working the ground -4%");
        assert_eq!(mind_change_words(&had, &had), "");
    }
}
