// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! D54's proof: the cache answers exactly what a fresh search answers, for
//! every sequence of other-body movements, near and far.
use super::*;
use crate::spatial::embodiment::{Box3, tests::terrain};

const SEEDS: u64 = 6;
const STEPS: usize = 14;
/// The searching body stands here; its window is control_input's.
const CENTRE: i64 = 384;
const NEAR: i64 = 10;
const WINDOW: [i64; 2] = [350, 418];
const HEIGHT: i64 = 40;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % (hi - lo) as u64) as i64
    }
}
/// A body-sized box with its corner in `[lo, hi)` on the ground plane.
fn body(rng: &mut Rng, lo: i64, hi: i64) -> Box3 {
    let at = [rng.range(lo, hi), rng.range(0, HEIGHT / 2), rng.range(lo, hi)];
    let size = [rng.range(1, 7), rng.range(2, 14), rng.range(1, 7)];
    [at, [0, 1, 2].map(|k| at[k] + size[k])]
}
fn near(rng: &mut Rng) -> Box3 {
    body(rng, CENTRE - NEAR, CENTRE + NEAR)
}
fn anywhere(rng: &mut Rng) -> Box3 {
    body(rng, WINDOW[0] - 4, WINDOW[1] + 4)
}

#[test]
fn the_cache_answers_what_a_fresh_search_answers_over_moving_bodies() {
    let sim = terrain();
    let base = search::turn_tests::control_input(&sim);
    let (mut hits, mut misses, mut routes, mut none) = (0, 0, 0, 0);
    for seed in 0..SEEDS {
        let mut rng = Rng(seed);
        let cache = Cache::default();
        let mut input = base.clone();
        for _ in 0..rng.range(0, 4) {
            input.blocked.push(anywhere(&mut rng));
        }
        for step in 0..STEPS {
            let len = cache.0.lock().unwrap().len();
            let got = cache.find(input.clone()).unwrap();
            let fresh = search::find(&{
                let mut windowed = input.clone();
                windowed.blocked.retain(|[lo, hi]| {
                    (0..2).all(|k| {
                        lo[2 * k] < input.terrain.volume.max[k]
                            && hi[2 * k] > input.terrain.volume.min[k]
                    })
                });
                windowed
            })
            .unwrap();
            assert_eq!(got, fresh, "seed {seed} step {step}");
            if cache.0.lock().unwrap().len() == len {
                hits += 1
            } else {
                misses += 1
            }
            if got.route.is_some() {
                routes += 1
            } else {
                none += 1
            }
            // Bodies arrive, leave, and move a little or a lot.
            let n = input.blocked.len();
            match (rng.next() % 5, n) {
                (0, _) | (_, 0) => input.blocked.push(near(&mut rng)),
                (1, _) => input.blocked.push(anywhere(&mut rng)),
                (2, _) => {
                    input.blocked.remove(rng.range(0, n as i64) as usize);
                }
                (3, _) => {
                    let i = rng.range(0, n as i64) as usize;
                    let d = [rng.range(-2, 3), 0, rng.range(-2, 3)];
                    let [lo, hi] = &mut input.blocked[i];
                    for k in 0..3 {
                        lo[k] += d[k];
                        hi[k] += d[k];
                    }
                }
                _ => {
                    let i = rng.range(0, n as i64) as usize;
                    input.blocked[i] = anywhere(&mut rng);
                }
            }
        }
    }
    assert!(hits > 0 && misses > 0, "hits {hits} misses {misses}");
    assert!(routes > 0 && none > 0, "routes {routes} none {none}");
}

#[test]
fn a_body_leaving_a_consulted_region_or_entering_an_empty_one_ends_the_entry() {
    let sim = terrain();
    let clear = search::turn_tests::control_input(&sim);
    let mut blocked = clear.clone();
    blocked.blocked.push([[360, 0, 360], [408, 64, 408]]);
    let cache = Cache::default();
    // The blocker sits across the route: no route, and the region is recorded.
    let held = cache.find(blocked.clone()).unwrap();
    assert!(held.route.is_none());
    // Leaving that region changes the answer, so the entry must not serve it.
    let freed = cache.find(clear.clone()).unwrap();
    assert!(freed.route.is_some());
    assert_eq!(freed, search::find(&clear).unwrap());
    assert_eq!(cache.0.lock().unwrap().len(), 2);
    // A body entering a region the search read and found empty.
    let (_, asked) = search::find_watched(&clear).unwrap();
    let cell = asked.a_cell().expect("a search records cells");
    let mut entered = clear.clone();
    entered.blocked.push(cell);
    assert!(!cache.0.lock().unwrap()[1].asked.holds_for(&entered.blocked));
    assert_eq!(
        cache.find(entered.clone()).unwrap(),
        search::find(&entered).unwrap()
    );
    assert_eq!(cache.0.lock().unwrap().len(), 3);
    // A body wholly outside every consulted region keeps the entry.
    let mut far = clear.clone();
    far.blocked.push([[WINDOW[1] - 3, 0, WINDOW[0]], [WINDOW[1] - 2, 8, WINDOW[0] + 1]]);
    assert!(cache.0.lock().unwrap()[1].asked.holds_for(&far.blocked));
}
