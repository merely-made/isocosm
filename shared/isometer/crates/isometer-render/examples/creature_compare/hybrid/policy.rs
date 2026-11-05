// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Probe settings, not adopted product defaults or the complete L10 selector.
use serde_json::{Value, json};
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
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Choice {
    pub live: bool,
    pub period: usize,
}
pub fn choose(old: Choice, pixels: f32, held_live: bool, examined: bool, c: Config) -> Choice {
    let live = held_live
        && (examined
            || if old.live {
                pixels > c.exit
            } else {
                pixels >= c.enter
            });
    let period = if old.period == c.periods[0] && pixels >= c.medium_exit {
        c.periods[0]
    } else if old.period == c.periods[2] && pixels <= c.far_enter {
        c.periods[2]
    } else if pixels >= c.medium_enter {
        c.periods[0]
    } else if pixels <= c.far_exit {
        c.periods[2]
    } else {
        c.periods[1]
    };
    Choice { live, period }
}
pub fn checks(c: Config) -> Value {
    let initial = Choice {
        live: true,
        period: c.periods[0],
    };
    let middle = (c.enter + c.exit) * 0.5;
    let delta = (c.enter - c.exit) * 0.2;
    let wave = (0..64)
        .map(|i| middle + if i % 2 == 0 { delta } else { -delta })
        .collect::<Vec<_>>();
    let count = |cfg| {
        let mut old = initial;
        let mut changes = 0;
        for &p in &wave {
            let next = choose(old, p, true, false, cfg);
            changes += usize::from(next.live != old.live);
            old = next;
        }
        changes
    };
    assert_eq!(count(c), 0);
    let mut zero = c;
    zero.enter = middle;
    zero.exit = middle;
    assert!(count(zero) > 0);
    let held = choose(initial, 96.0, false, true, c);
    let unbounded = choose(initial, 96.0, true, true, c);
    assert!(!held.live && unbounded.live);
    let mut old = Choice {
        live: false,
        period: c.periods[0],
    };
    for i in 0..64 {
        let next = choose(
            old,
            if i % 2 == 0 {
                c.medium_exit + 0.5
            } else {
                c.medium_enter - 0.5
            },
            true,
            false,
            c,
        );
        assert_eq!(next.period, old.period);
        old = next;
    }
    json!({"rung_band_changes":count(c),"zero_band_control_changes":count(zero),"holding_bound_pass":!held.live,"bound_removed_control_live":unbounded.live,"cadence_band_pass":true})
}
