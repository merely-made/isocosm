// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Runs Mesocosm in a window over a native Isocosm session.
//!
//! ```text
//! cargo run -p mesocosm-genet
//! cargo run -p mesocosm-genet -- --replay <trace>
//! cargo run -p mesocosm-genet -- --watch <trace>
//! cargo run -p mesocosm-genet -- --scenario <scenario>
//! ```
//!
//! The player directs and never drives (wing rulings 671, 679): E nudges the
//! critter to attend to its site or the picked body, Q nudges it to act
//! there, S speciates its line. At a checkpoint the keys narrow to Enter
//! (carry on) and T (take the body on offer); at the boundary the board's Tab
//! and R pick and commit an offer. See `--help` for the rest.
//!
//! `--replay` is headless: it loads a recorded save, replays it through the
//! runtime, and exits 1 when the hash it lands on differs from the recorded
//! one. `--watch` takes the save up where it stands and plays on in the
//! window (786). With nothing named, the trace, receipt and capture go to scratch
//! names under `<Code>/testing/mesocosm/`, never the golden fixture.

use std::path::PathBuf;

use mesocosm_genet::section::{BodyMode, CameraMode, TerrainStyle};
use mesocosm_genet::{Host, HostConfig, played};

fn main() {
    let mut config = HostConfig::default();
    let mut args = std::env::args().skip(1);
    let (mut replay, mut trace, mut receipt, mut capture) = (None, None, None, None);
    let mut bench = false;
    let mut size_explicit = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bench" => bench = true,
            "--frames" => config.frames = args.next().and_then(|v| v.parse().ok()),
            "--size" => {
                let size = args.next().unwrap_or_default();
                let parsed = size
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)));
                let Some((width, height)) = parsed.filter(|(w, h)| *w > 0 && *h > 0) else {
                    fail("--size wants positive WIDTHxHEIGHT in logical pixels");
                };
                config.width = width;
                config.height = height;
                size_explicit = true;
            },
            "--bodies" => {
                config.body_mode = BodyMode::parse(&args.next().unwrap_or_default())
                    .unwrap_or_else(|| fail("--bodies wants voxels or capsules"));
            },
            "--body-budget" => {
                config.body_budget = args
                    .next()
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|v| *v > 0)
                    .unwrap_or_else(|| fail("--body-budget wants a positive integer"));
            },
            "--body-light" => {
                config.body_light = args
                    .next()
                    .and_then(|v| v.parse::<f32>().ok())
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .unwrap_or_else(|| fail("--body-light wants a finite nonnegative intensity"));
            },
            "--terrain-style" => {
                config.terrain_style = TerrainStyle::parse(&args.next().unwrap_or_default())
                    .unwrap_or_else(|| fail("--terrain-style wants auto, classic or habitat"));
            },
            "--camera" => {
                config.camera = CameraMode::parse(&args.next().unwrap_or_default())
                    .unwrap_or_else(|| {
                        fail("--camera wants oblique, side, across or terrarium-east/south/west/north")
                    });
            },
            "--slab" => {
                if let Some(half) = args.next().and_then(|v| v.parse::<f32>().ok()) {
                    config.slab_half_height = half;
                }
            },
            "--seed" => {
                if let Some(seed) = args.next().and_then(|v| v.parse().ok()) {
                    config.seed = seed;
                }
            },
            "--population" => {
                config.population = args
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|v| *v > 0)
                    .unwrap_or_else(|| fail("--population wants a positive integer"));
            },
            "--capture" => capture = args.next().map(PathBuf::from),
            "--trace" => trace = args.next().map(PathBuf::from),
            "--receipt" => receipt = args.next().map(PathBuf::from),
            "--replay" => replay = args.next().map(PathBuf::from),
            "--watch" => match args.next().map(PathBuf::from) {
                Some(path) => config.watch = Some(path),
                None => fail("--watch wants a recorded save"),
            },
            "--scenario" => {
                let path = args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| fail("--scenario wants a path"));
                match std::fs::read_to_string(&path) {
                    Ok(text) => config.scenario = Some(text),
                    Err(error) => fail(&format!("scenario: {}: {error}", path.display())),
                }
            },
            "--dev" => config.dev = true,
            "--follow" => config.follow = args.next().and_then(|v| v.parse().ok()),
            "--help" | "-h" => {
                println!("{HELP}");
                return;
            },
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }

    if let Some(path) = replay {
        std::process::exit(match played::replay(&path) {
            Ok((recorded, replayed)) if recorded == replayed => {
                println!("replay: hash {replayed:016x} (matches the recorded hash)");
                0
            },
            Ok((recorded, replayed)) => {
                println!(
                    "replay: hash {replayed:016x} (MISMATCH: the trace recorded {recorded:016x})"
                );
                1
            },
            Err(error) => {
                eprintln!("replay: {error}");
                1
            },
        });
    }

    let stem = |default: PathBuf, bench_name: &str| {
        if bench {
            default.with_file_name(bench_name)
        } else {
            default
        }
    };
    config.capture =
        Some(capture.unwrap_or_else(|| stem(played::default_capture_path(), "scratch_bench.png")));
    config.receipt =
        Some(receipt.unwrap_or_else(|| stem(played::default_receipt_path(), "scratch_bench.json")));
    if !bench {
        config.trace = Some(trace.unwrap_or_else(played::default_trace_path));
    }
    if bench && !size_explicit {
        config.width = 1280;
        config.height = 900;
    }
    let result = if bench {
        mesocosm_genet::app::bench::run(config)
    } else {
        Host::run(config)
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => fail(&format!("host failed: {error}")),
    }
}

fn fail(why: &str) -> ! {
    eprintln!("{why}");
    std::process::exit(1);
}

const HELP: &str = "\
mesocosm-genet: run Mesocosm in a window over a native Isocosm session

  --frames N      run N frames and exit
  --size WxH      initial window size in logical pixels (default 960x540)
  --seed N        world seed
  --population N  founding population (default: the founding's own)
  --bodies MODE   voxels (default) or capsules (comparison), presentation only
  --body-budget N maximum detailed bodies in the section
  --body-light N  ambient body light intensity (default 1, zero is dark)
  --terrain-style MODE  auto (classic), classic or habitat
  --camera MODE   oblique (default), side, across, terrarium-east/south/west/north
  --slab H        section slab half-height in voxels (presentation only, default 28)
  --capture PATH  write the final frame as a PNG
  --trace PATH    write the session's save as the trace
  --receipt PATH  write the run's receipt
  --replay PATH   replay a recorded save headlessly and check its hash
  --watch PATH    take a recorded save up and play on in the window
  --scenario PATH drive the run from a text scenario and exit 1 if it fails
  --bench         open the bench over a native session (1280x900)
  --dev           enable the dev lane and its keys (DT1, DT2, DT3)
  --follow ID     start the camera on this body (presentation only)

--capture, --trace and --receipt default to scratch names under
<Code>/testing/mesocosm/, never the golden ps1_played.* fixture.

controls: E/Space attend, Q act (toward the followed body, else the site),
  S speciate, arrows pan, Z/V turn a terrarium camera, Esc quit
at a checkpoint the keys narrow:
  Enter  carry on (keep the parent, take the first heir, close the boundary)
  T      take the body on offer
at the boundary the trait board comes up:
  Tab    move among the offers
  R      commit the selected offer
with --dev, host-only keys (none reaches the session's log):
  P pause, . step, , step ten, [ ] speed, N/B follow next/previous, M follow
  the played body, I inspect parts, J/L walk them, U clear
and four dev intents, which do enter the log and mark the run assisted:
  X end the epoch, F force a birth, K kill, G place matter
  (the runtime refuses F and K until native has a command for them)

a scenario is one verb a line (blank lines and # comments skipped):
  act NAME        a key letter above, or follow ID, follow-nearest, demo STEPS
  settle N        pump N frames
  wait [CAP]      hold until the host reports quiet
  assert text S   S is on a chrome lane that is on screen
  assert snap F OP V   hash, mode, camera, tick, steps, frames, dev,
                  dev-intents, assisted, queued, controlled, follow, site,
                  living, checkpoint, boundary, fault, paused, inspecting
  assert event S  S is in what the runtime answered
  capture NAME    a PNG, at NAME if it is a path or beside the fixtures if not
  log WORDS       into the run's log";
