// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Probe settings, not adopted product defaults or the complete L10 selector.
use serde_json::{Value, json};
use std::num::NonZeroU64;

use isometer_render::hybrid::{BodyChoice, BodyDetail, HybridPolicy, RefreshClass};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub enter: f32,
    pub exit: f32,
    pub medium_enter: f32,
    pub medium_exit: f32,
    pub far_enter: f32,
    pub far_exit: f32,
    pub periods: [usize; 3],
    pub transition: usize,
}
impl Config {
    fn policy(self) -> HybridPolicy {
        HybridPolicy::new(
            [
                [self.far_exit, self.far_enter],
                [self.medium_exit, self.medium_enter],
                [self.exit, self.enter],
            ],
            [2, 1, 0].map(|i| NonZeroU64::new(self.periods[i] as u64).unwrap()),
        )
        .expect("ordered hybrid thresholds and periods")
    }

    pub fn load(value: Option<Value>) -> Self {
        let v = value.unwrap_or(json!({}));
        let number = |key: &str, default: f32| v[key].as_f64().map_or(default, |x| x as f32);
        let c = Self {
            enter: number("live_enter_px", 56.0),
            exit: number("live_exit_px", 44.0),
            medium_enter: number("medium_enter_px", 30.0),
            medium_exit: number("medium_exit_px", 26.0),
            far_enter: number("far_enter_px", 16.0),
            far_exit: number("far_exit_px", 12.0),
            periods: v
                .get("periods")
                .map_or([2, 4, 12], |x| serde_json::from_value(x.clone()).unwrap()),
            transition: v["transition_frames"].as_u64().unwrap_or(8) as usize,
        };
        assert!(
            c.enter > c.exit
                && c.exit > c.medium_enter
                && c.medium_enter > c.medium_exit
                && c.medium_exit > c.far_enter
                && c.far_enter > c.far_exit
        );
        assert!(
            c.periods.iter().all(|p| *p > 0 && 24 % p == 0)
                && c.transition > 0
                && c.transition <= 16
        );
        c
    }
    pub fn json(self) -> Value {
        json!({"live_enter_px":self.enter,"live_exit_px":self.exit,"medium_enter_px":self.medium_enter,
        "medium_exit_px":self.medium_exit,"far_enter_px":self.far_enter,"far_exit_px":self.far_exit,"periods":self.periods,"transition_frames":self.transition})
    }

    pub fn period(self, pixels: u32) -> usize {
        let policy = self.policy();
        let choice = policy
            .select(pixels as f32, None, false, BodyDetail::Live)
            .choice;
        policy.refresh_period(choice.refresh).get() as usize
    }
}
pub fn checks(c: Config) -> Value {
    use BodyDetail::{Live, Sprite};

    let initial = BodyChoice {
        detail: Live,
        refresh: RefreshClass::Fast,
    };
    let middle = (c.enter + c.exit) * 0.5;
    let delta = (c.enter - c.exit) * 0.2;
    let wave = (0..64)
        .map(|i| middle + if i % 2 == 0 { delta } else { -delta })
        .collect::<Vec<_>>();
    let count = |cfg: Config| {
        let policy = cfg.policy();
        let mut old = initial;
        let mut changes = 0;
        for &p in &wave {
            let next = policy.select(p, Some(old), false, Live).choice;
            changes += usize::from(next.detail != old.detail);
            old = next;
        }
        changes
    };
    assert_eq!(count(c), 0);
    let mut zero = c;
    zero.enter = middle;
    zero.exit = middle;
    assert!(count(zero) > 0);
    let policy = c.policy();
    let held = policy
        .select(96.0, Some(initial), true, Sprite)
        .choice
        .detail;
    let unbounded = policy.select(96.0, Some(initial), true, Live).choice.detail;
    assert!(held == Sprite && unbounded == Live);
    let mut old = BodyChoice {
        detail: Sprite,
        refresh: RefreshClass::Fast,
    };
    for i in 0..64 {
        let next = policy
            .select(
                if i % 2 == 0 {
                    c.medium_exit + 0.5
                } else {
                    c.medium_enter - 0.5
                },
                Some(old),
                false,
                Live,
            )
            .choice;
        assert_eq!(next.refresh, old.refresh);
        old = next;
    }
    json!({"rung_band_changes":count(c),"zero_band_control_changes":count(zero),"holding_bound_pass":held == Sprite,"bound_removed_control_live":unbounded == Live,"cadence_band_pass":true})
}
