// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isometer_space::{Atlas, Border, Footprint, Materials, Result};

struct Valley;
impl Atlas for Valley {
    fn seed(&self) -> u64 {
        9
    }
    fn footprint(&self) -> Footprint {
        Footprint { sides: 4, side: 64 }
    }
    fn sites(&self) -> Vec<u64> {
        vec![0, 1, 2]
    }
    fn terrain_seed(&self, id: u64) -> Result<u64> {
        Ok(9 + id)
    }
    fn elevation(&self, id: u64) -> Result<i64> {
        Ok(id as i64 * 8 + 16)
    }
    fn water(&self, _: u64) -> Result<i64> {
        Ok(0)
    }
    fn relief(&self, _: u64) -> Result<i64> {
        Ok(4)
    }
    fn materials(&self) -> Result<Materials> {
        Ok(Materials {
            air: 0,
            water: 1,
            soil: 2,
            rock: 3,
        })
    }
    fn border(&self, id: u64, side: u8) -> Option<(u64, Border)> {
        let next = match side {
            1 if id < 2 => id + 1,
            3 if id > 0 => id - 1,
            _ => return None,
        };
        Some((
            next,
            Border {
                side,
                enters: (side + 2) % 4,
                flipped: false,
            },
        ))
    }
}

#[test]
fn chart_camera_promotes_the_same_site_cells_across_the_seam_and_back() {
    let volume = Volume::charted(&Valley, 0, 8, &[]).unwrap();
    let datum = volume.datum;
    let mut world = TerrainBands::new(volume);
    let mut original = None;
    for focus in [26, 68, 26] {
        let camera = SlabCamera::dimetric_2_1([focus as f32, 13., 0.], 30., 1.5, 128.).unwrap();
        let (bands, frames, stats) = world.frame(camera, 1, [4096, 4096]).unwrap();
        assert_eq!(stats.centre_site, Some(if focus < 32 { 0 } else { 1 }));
        assert!(stats.sites >= 2);
        assert_eq!(stats.cell, [1, 2]);
        assert!(stats.bricks.iter().all(|n| *n > 0));
        assert!(frames.iter().all(|f| f.overflow == 0));
        // Read actual emitted payloads, in native world coordinates. At the
        // seam each occupied surface belongs to exactly one level.
        for x in 20..52 {
            let z = 1;
            let (vol, [sx, sz]) = bands[0].world.locate(x, z).unwrap();
            let mut hits = 0;
            for (b, f) in bands.iter().zip(&frames) {
                let cell = b.cell;
                let [cx, cz] = [x, z].map(|p| p.div_euclid(cell));
                let (sample, [sx, sz]) = b.world.locate(cx * cell, cz * cell).unwrap();
                let cy = (sample.surface(sx, sz).unwrap() - datum).div_euclid(cell);
                let key = [cx, cy, cz].map(|v| v.div_euclid(8) as i16);
                if f.keys.binary_search(&key).is_ok() {
                    let mut raw = [0; 512];
                    b.fill(key, &mut raw);
                    let [x, y, z] = [cx, cy, cz].map(|v| v.rem_euclid(8) as usize);
                    assert_ne!(raw[(y * 8 + z) * 8 + x], 0);
                    hits += 1;
                    if b.fine {
                        assert_eq!(
                            raw[(y * 8 + z) * 8 + x],
                            vol.material([sx, vol.surface(sx, sz).unwrap(), sz])
                                .unwrap()
                        );
                    }
                }
            }
            assert_eq!(hits, 1, "surface at {x} from focus {focus}");
            assert!(vol.holds(sx, sz));
        }
        if focus == 26 {
            if let Some(before) = original.take() {
                assert_eq!(before, frames);
            } else {
                original = Some(frames);
            }
        }
    }
}
