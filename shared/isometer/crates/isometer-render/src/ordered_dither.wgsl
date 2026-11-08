// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

fn selects_sprite(screen: vec2<f32>, coverage: u32) -> bool {
    if coverage == 0u { return false; }
    if coverage == 16u { return true; }
    let at = vec2<u32>(screen) % vec2<u32>(4u);
    let bayer = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    return bayer[at.y * 4u + at.x] < coverage;
}
