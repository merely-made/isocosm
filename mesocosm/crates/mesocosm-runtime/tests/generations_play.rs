// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "../src/bin/mesocosm-active-profile/play.rs"]
mod play;

#[test]
#[ignore = "explicit ordinary saved-world generation run"]
fn saved_ordinary_generations() { play::run(); }
