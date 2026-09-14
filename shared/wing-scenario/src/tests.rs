// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! GPU-free receipts for the parts of the lane that are pure arithmetic: where
//! a named capture lands, what the receipt JSON carries, the viewport mask, and
//! the deferred-close half of the frame-limit logic.

use std::{
    cell::Cell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use genet_probe::Scenario;

use super::*;

/// A product that needs no host: only the three required hooks, and they are
/// never called by the tests below (which stay off `after_frame`, the one
/// entry point that needs a live `AppCtx`).
struct Headless;

impl Product for Headless {
    type State = ();
    type Logic = fn(&()) -> Self::View;
    type View = Box<dyn cambium::AnyView<(), (), cambium::GenetCtx, cambium::GenetElement>>;

    const KIND: &'static str = "test-lane";
    const SURFACE: &'static str = "test";
    const LOG_PREFIX: &'static str = "test";

    fn sheet(&self) -> &'static str {
        ""
    }

    fn snapshot(&self, _: &Ctx<'_, Self>, _: usize, _: f32) -> genet_probe::ProbeSnapshot {
        genet_probe::ProbeSnapshot::default()
    }

    fn drain_events(&mut self, _: &mut Ctx<'_, Self>) -> Vec<String> {
        Vec::new()
    }

    fn default_capture_path(&self) -> PathBuf {
        PathBuf::from("out").join("specimen-bench.png")
    }
}

fn lane(scenario: Option<Scenario>, receipt: Option<PathBuf>) -> Lane<Headless> {
    Lane::new(Headless, scenario, receipt, None, Rc::new(Cell::new(0)))
}

#[test]
fn a_named_capture_hangs_off_the_run_s_own_artifact_stem() {
    let default = Path::new("out/specimen-bench.png");
    let receipt = Path::new("/runs/p3a/acceptance.json");
    // The final capture wins over the receipt, and the receipt over the default.
    assert_eq!(
        capture_path("opened", Some(Path::new("/runs/p3a/shot.png")), Some(receipt), default),
        PathBuf::from("/runs/p3a/shot-opened.png")
    );
    assert_eq!(
        capture_path("opened", None, Some(receipt), default),
        PathBuf::from("/runs/p3a/acceptance-opened.png")
    );
    assert_eq!(
        capture_path("opened", None, None, default),
        PathBuf::from("out/specimen-bench-opened.png")
    );
}

#[test]
fn a_capture_name_with_a_separator_is_taken_literally_and_others_are_sanitized() {
    let default = Path::new("out/specimen-bench.png");
    assert_eq!(
        capture_path("sub/dir/shot.png", None, None, default),
        PathBuf::from("sub/dir/shot.png")
    );
    // Everything outside [A-Za-z0-9-] lowers to `_`, so a name can never
    // escape the artifact directory or collide with a shell metacharacter.
    assert_eq!(
        capture_path("after volley!", None, None, default),
        PathBuf::from("out/specimen-bench-after_volley_.png")
    );
}

#[test]
fn a_close_before_the_scenario_completes_fails_the_run_and_a_second_close_is_inert() {
    let mut lane = lane(Some(Scenario::parse("log one\nlog two\n").unwrap()), None);
    lane.request_close();
    assert_eq!(
        lane.errors,
        ["window closed before the scenario completed"],
        "an unfinished scenario must be an error, not a silent pass"
    );
    lane.request_close();
    assert_eq!(lane.errors.len(), 1, "the outcome is decided once");
}

#[test]
fn a_close_with_no_scenario_is_an_ordinary_successful_end() {
    let mut lane = lane(None, None);
    lane.request_close();
    assert!(lane.errors.is_empty());
    assert!(lane.receipt_value(true, &BTreeMap::new())["ok"].as_bool().unwrap());
}

#[test]
fn the_receipt_carries_every_named_section_and_its_kind() {
    let mut lane = lane(Some(Scenario::parse("log hello\n").unwrap()), None);
    lane.request_close();
    let value = lane.receipt_value(false, &BTreeMap::new());
    let keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    assert_eq!(
        keys,
        [
            "captures",
            "checkpoints",
            "cost",
            "errors",
            "final",
            "frames",
            "kind",
            "ok",
            "pixel_checks",
            "scenario",
            "scenario_log"
        ]
    );
    assert_eq!(value["kind"], "test-lane");
    assert_eq!(value["scenario"], true);
    assert_eq!(value["ok"], false);
    assert_eq!(value["errors"][0], "window closed before the scenario completed");
    // The cost block always declares its schema, even with no phases.
    assert_eq!(value["cost"]["schema"], 1);
}

#[test]
fn the_frame_limit_decides_only_once_and_only_after_the_limit() {
    let lane = lane(None, None).with_frame_limit(Some(3));
    assert!(!lane.frame_limit_reached(2));
    assert!(lane.frame_limit_reached(3));
    assert!(lane.frame_limit_reached(4));
    let unbounded = lane_with_no_limit();
    assert!(!unbounded.frame_limit_reached(1_000_000));
}

fn lane_with_no_limit() -> Lane<Headless> {
    lane(None, None)
}

#[test]
fn the_viewport_mask_insets_its_border_and_drops_an_overlay() {
    // A 100x100 box at the origin, 1 device pixel per logical unit, 10 inset.
    let viewport = Viewport::new([0.0, 0.0, 100.0, 100.0], 1.0, 10.0);
    assert!(!viewport.contains(5, 50), "inside the inset is masked out");
    assert!(viewport.contains(50, 50));
    assert!(!viewport.contains(95, 50));
    let masked = viewport.with_overlay(Some([40.0, 40.0, 20.0, 20.0]));
    assert!(!masked.contains(50, 50), "the overlay rectangle is excluded");
    assert!(masked.contains(20, 20));
}

#[test]
fn a_declared_transform_round_trips_a_point_and_shows_in_the_receipt() {
    let plain = Viewport::new([0.0, 0.0, 100.0, 100.0], 2.0, 13.0);
    assert_eq!(plain.map((10.0, 20.0), false), (10.0, 20.0));
    let turned = plain.with_transform(Some(ViewportTransform {
        rotate_deg: 7.0,
        scale: 0.9,
        origin: [0.25, 0.75],
    }));
    let there = turned.map((10.0, 20.0), false);
    let back = turned.map(there, true);
    assert!((back.0 - 10.0).abs() < 1e-3 && (back.1 - 20.0).abs() < 1e-3);
    let json = serde_json::to_value(turned).unwrap();
    assert_eq!(json["transformed"], true);
    assert_eq!(json["pixel_scale"], 2.0);
    assert_eq!(
        json.as_object().unwrap().keys().cloned().collect::<Vec<_>>(),
        ["border", "overlay", "pixel_scale", "transformed"],
        "the receipt shape must not gain the mask's private arithmetic"
    );
    assert_eq!(serde_json::to_value(plain).unwrap()["transformed"], false);
}
