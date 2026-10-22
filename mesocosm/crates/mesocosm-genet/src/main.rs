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
//! critter to attend to its site or the picked body, Q chooses a current
//! suggestion, S speciates its line. Click attends; right click chooses a hint.
//! At a checkpoint the keys narrow to Enter
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
    let mut curve: Option<u64> = None;
    let (mut replay, mut trace, mut receipt, mut capture) = (None, None, None, None);
    let mut no_trace = false;
    let mut bench = false;
    let mut bench_epoch_explicit = false;
    let mut sun_explicit = false;
    let mut size_explicit = false;
    let mut founding_explicit = false;
    let mut seed_explicit = false;
    let mut unchosen_explicit = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bench" => bench = true,
            "--bench-world" => config.bench_world = true,
            "--bench-epoch" => { bench_epoch_explicit=true; config.bench_epoch = args.next().and_then(|v|v.parse().ok())
                .filter(|v|*v>0).unwrap_or_else(||fail("--bench-epoch wants positive ticks")); },
            "--frames" => config.frames = args.next().and_then(|v| v.parse().ok()),
            "--render-scale" => config.render_scale = args.next().and_then(|v|v.parse().ok())
                .filter(|scale: &u32| (1..=8).contains(scale))
                .unwrap_or_else(|| fail("--render-scale wants 1 to 8")),
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
                    .unwrap_or_else(|| fail("--bodies wants smooth, voxels, boxes or capsules"));
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
            "--sun" => {
                sun_explicit = true;
                config.sun = args.next().as_deref().and_then(mesocosm_genet::section::SunSource::parse)
                    .unwrap_or_else(|| fail("--sun wants fixed, sim, or a tick number"));
            },
            "--terrain-style" => {
                config.terrain_style = TerrainStyle::parse(&args.next().unwrap_or_default())
                    .unwrap_or_else(|| fail("--terrain-style wants auto, classic or habitat"));
            },
            "--camera" => {
                config.camera = CameraMode::parse(&args.next().unwrap_or_default())
                    .unwrap_or_else(|| {
                        fail("--camera wants dimetric, dimetric-south/west/north, isometric, isometric-south/west/north, oblique, side, across or terrarium-east/south/west/north")
                    });
            },
            "--view-depth" => {
                let depth: f32 = args.next().unwrap_or_default().parse()
                    .unwrap_or_else(|_| fail("--view-depth wants 1 to 1024 voxels"));
                if !depth.is_finite() || !(1.0..=1024.0).contains(&depth) {
                    fail("--view-depth wants 1 to 1024 voxels");
                }
                config.view_depth = Some(depth);
            },
            "--slab" => {
                if let Some(half) = args.next().and_then(|v| v.parse::<f32>().ok()) {
                    config.slab_half_height = half;
                }
            },
            "--seed" => {
                founding_explicit = true;
                seed_explicit = true;
                config.seed = args.next().and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| fail("--seed wants an explicit unsigned world seed"));
            },
            "--unchosen-seed" => unchosen_explicit = true,
            "--candidate-attempts" => {
                config.candidate_attempts = args.next().and_then(|v| v.parse::<u32>().ok())
                    .filter(|v| (1..=128).contains(v))
                    .unwrap_or_else(|| fail("--candidate-attempts wants an integer from 1 to 128"));
            },
            "--start" => {
                let value = args.next().unwrap_or_default();
                let (within, epochs) = value.split_once(':')
                    .and_then(|(within, epochs)| Some((within.parse().ok()?, epochs.parse().ok()?)))
                    .unwrap_or_else(|| fail("--start wants WITHIN:EPOCHS, both unsigned epoch counts"));
                config.start = Some(isocosm::directing::interim::Start { within, epochs });
            },
            "--mode" => config.mode = Some(match args.next().as_deref() {
                Some("survival") => isocosm::directing::readings::Mode::Survival,
                Some("creative") => isocosm::directing::readings::Mode::Creative,
                _ => fail("--mode wants survival or creative"),
            }),
            "--population" => {
                founding_explicit = true;
                config.population = args
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|v| *v > 0)
                    .unwrap_or_else(|| fail("--population wants a positive integer"));
            },
            "--ecology-rates" => {
                config.ecology_rates = Some(mesocosm_genet::HostConfig::parse_ecology_rates(
                    &args.next().unwrap_or_default())
                    .unwrap_or_else(|| fail("--ecology-rates wants stock or a1")));
            },
            "--capture" => capture = args.next().map(PathBuf::from),
            "--trace" => {
                let Some(path) = args.next().filter(|path| !path.starts_with("--")) else {
                    fail("--trace wants a path");
                };
                trace = Some(PathBuf::from(path));
            },
            "--no-trace" => no_trace = true,
            "--receipt" => receipt = args.next().map(PathBuf::from),
            "--replay" => replay = args.next().map(PathBuf::from),
            "--watch" => match args.next().map(PathBuf::from) {
                Some(path) => config.watch = Some(path),
                None => fail("--watch wants a recorded save"),
            },
            "--spatial" => match args.next().map(PathBuf::from) {
                Some(path) => config.spatial = Some(path),
                None => fail("--spatial wants authored physical input for admitted native bodies"),
            },
            "--spread" => match args.next().map(PathBuf::from) {
                Some(path) => config.spread = Some(path),
                None => fail("--spread wants explicit authored founding and spread steps"),
            },
            "--ecology-witness" => match args.next().map(PathBuf::from) {
                Some(path) => config.ecology_witness = Some(path),
                None => fail("--ecology-witness wants complete authored joint native witness inputs"),
            },
            "--ecology-candidate" => match args.next().map(PathBuf::from) {
                Some(path) => config.ecology_candidate = Some(path),
                None => fail("--ecology-candidate wants a complete explicit generated candidate and joint witness"),
            },
            "--graft" => match args.next().map(PathBuf::from) {
                Some(path) => config.graft = Some(path),
                None => fail("--graft wants a complete native Carry input for the played normative body"),
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
            "--play" => config.play = true,
            "--body-roster" => config.body_roster = true,
            "--body-founding" => config.body_founding = Some(args.next()
                .unwrap_or_else(|| fail("--body-founding wants a name or Drawn"))),
            "--curve" => curve = args.next().and_then(|v| v.parse().ok()),
            "--hour" => config.play_hour = args.next().and_then(|v| v.parse().ok())
                .filter(|hour: &u32| *hour < 24).unwrap_or_else(|| fail("--hour wants 0 to 23")),
            "--soil" => config.play_soil = args.next().and_then(|v| v.parse().ok())
                .filter(|percent: &u64| *percent > 0).unwrap_or_else(|| fail("--soil wants a percent above zero")),
            "--epoch" => config.play_epoch = args.next().and_then(|v| v.parse().ok())
                .filter(|ticks: &u64| *ticks > 0).unwrap_or_else(|| fail("--epoch wants a tick count above zero")),
            "--help" | "-h" => {
                println!("{HELP}");
                return;
            },
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }

    if seed_explicit && unchosen_explicit { fail("choose --seed or --unchosen-seed"); }
    if (config.body_roster || config.body_founding.is_some()) && (!config.play || config.watch.is_some() || config.spread.is_some()
        || config.bench_world || config.ecology_candidate.is_some() || config.ecology_witness.is_some()
        || config.spatial.is_some() || config.graft.is_some() || replay.is_some() || bench) {
        fail("body founding options require a new --play world without another startup input");
    }
    if config.play && config.watch.is_none() && (config.spread.is_some() || config.bench_world
        || config.ecology_candidate.is_some() || config.ecology_witness.is_some()
        || config.spatial.is_some() || config.graft.is_some() || replay.is_some() || bench) {
        fail("--play uses generated physical placement and cannot combine with another startup input");
    }
    if no_trace && trace.is_some() { fail("--trace and --no-trace cannot be combined"); }
    if config.ecology_rates.is_some()
        && (config.bench_world || config.spread.is_some() || config.watch.is_some()
            || config.ecology_candidate.is_some()
            || config.spatial.is_some() || config.graft.is_some() || replay.is_some() || bench) {
        fail("--ecology-rates requires an ordinary generated world without a bench, candidate, resume, spread, replay, spatial or graft input");
    }
    if unchosen_explicit && (config.watch.is_some() || config.spread.is_some() || replay.is_some() || bench) {
        fail("--unchosen-seed requires an ordinary new generated world");
    }
    if config.ecology_candidate.is_some() && (config.watch.is_some() || config.spread.is_some()
        || config.ecology_witness.is_some() || config.spatial.is_some() || replay.is_some() || bench
        || founding_explicit || unchosen_explicit || config.start.is_some() || config.mode.is_some()) {
        fail("--ecology-candidate supplies its full founding, lifecycle and physical provider; competing startup inputs refuse");
    }
    if config.start.is_some() && (config.watch.is_some() || config.spread.is_some() || replay.is_some() || bench) {
        fail("--start requires an ordinary new generated world; authored spread declares its own start");
    }
    if config.mode.is_some() && (config.spread.is_some() || replay.is_some() || bench) {
        fail("--mode applies to ordinary new or watched worlds; authored spread declares its own mode");
    }
    config.unchosen_seed = unchosen_explicit || (!seed_explicit && config.watch.is_none()
        && config.spread.is_none() && config.ecology_candidate.is_none() && replay.is_none() && !bench && !config.bench_world);
    if config.spread.is_some()
        && (config.watch.is_some() || replay.is_some() || bench || founding_explicit) {
        fail("--spread supplies its founding and cannot combine with --watch, --replay, --bench, --seed or --population");
    }
    if config.ecology_witness.is_some() && (config.watch.is_some() || config.spread.is_some()
        || config.spatial.is_some() || replay.is_some() || bench) {
        fail("--ecology-witness requires a new generated world and contains its own complete spatial provider");
    }
    if let Some(ticks) = curve {
        std::process::exit(match mesocosm_genet::app::curve(&config, ticks, 20) {
            Ok(()) => 0,
            Err(why) => { eprintln!("curve: {why}"); 1 },
        });
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

    if (bench || config.bench_world) && !sun_explicit { config.sun = mesocosm_genet::section::SunSource::Fixed; }
    if config.bench_world && (bench || config.spread.is_some() || config.ecology_candidate.is_some()
        || config.ecology_witness.is_some() || unchosen_explicit || replay.is_some()) {
        fail("--bench-world is an explicit dev world or watched save; it cannot combine with another founding path");
    }
    if bench_epoch_explicit && !config.bench_world { fail("--bench-epoch requires --bench-world"); }
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
    if !bench && !no_trace {
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
  --render-scale N  draw the viewport at 1/N resolution, nearest scaled (1..8, default 1)
  --size WxH      initial window size in logical pixels (default 960x540)
  --seed N        world seed
  --play          a playable world: founds at once, no ecology witness
  --body-roster   use the historical authored roster for a new play world
  --body-founding NAME  new-play body roster (default SpacedRoster; Drawn uses random recipes)
  --epoch N       play epoch ticks (default 60)
  --hour H        play world's starting hour, 0 to 23 (default 7)
  --soil P        play world's founding soil, percent (default 100)
  --candidate-attempts N  cap unchosen A1 structural candidates (default 128, max 128)
  --unchosen-seed use the native clock-sourced new-world seed (ordinary default)
  --start W:E     wait at most W epochs for habitability, then add E epochs
  --mode MODE     survival or creative, using the native view
  --population N  founding population (default: the founding's own)
  --ecology-rates RATE  a1 (ordinary default) or stock (requires an authored witness)
  --bodies MODE   smooth (default, contact gait), voxels, boxes or capsules
  --body-budget N maximum detailed bodies in the section
  --body-light N  body light scale, sun and ambient (default 1 matches terrain, zero is dark)
  --sun S         sky light: fixed (bench default), sim (play default) or a tick number
  --terrain-style MODE  auto (classic), classic or habitat
  --camera MODE   dimetric (default), dimetric-south/west/north, isometric, isometric-south/west/north, oblique, side, across, terrarium-east/south/west/north
  --slab H        view half-height in voxels (default 28)
  --view-depth D  visible depth, 1..1024 voxels (default: surface frame or 16 for sections)
  --capture PATH  write the final frame as a PNG
  --trace PATH    write the session's save as the trace
  --no-trace      omit the session save (default behavior remains enabled)
  --receipt PATH  write the run's receipt
  --replay PATH   replay a recorded save headlessly and check its hash
  --watch PATH    take a recorded save up and play on in the window
  --spatial PATH  load explicit authored poses and operation geometry
  --graft PATH    apply an explicit native Carry input after spatial admission
  --spread PATH   found from explicit native stocks/rules and logged author spread steps
  --ecology-witness PATH  require one authored joint native witness before a new-world offer
  --ecology-candidate PATH  supply complete authored generated rules, lifecycle and joint witness
  --scenario PATH drive the run from a text scenario and exit 1 if it fails
  --bench         open the bench over a native session (1280x900)
  --bench-world   play an explicit dev world through the core and Assembly
  --bench-epoch N dev-world epoch ticks (default 60); requires --bench-world
  --dev           enable the dev lane and its keys (DT1, DT2, DT3)
  --follow ID     start the camera on this body (presentation only)

--capture, --trace and --receipt default to scratch names under
<Code>/testing/mesocosm/, never the golden ps1_played.* fixture.
--no-trace omits the save output; it cannot be combined with --trace.

controls: click or E/Space attend, right click or Q choose a current hint,
  S speciate, arrows pan, Z/V rotate, C surface/section, Esc quit
in the hint panel: Up/Down select, Enter or click the panel sends the hint,
  Escape closes it; each hint shows its actual body/site topic
at a checkpoint the keys narrow:
  Enter  carry on (keep the parent, take the first heir, close the boundary)
  T      take the body on offer
at the boundary the trait board comes up:
  Tab    move among the offers
  R      commit the selected offer
when the lineage or authored global world ends, or while watching:
  Up/Down or Tab select a native-visible other body, Watch, or New World
  Enter or click the pane applies the selected choice; W watches, N requests
  a new world. Esc saves the configured trace and quits; a new run starts
  separately through the startup flags, and --watch PATH resumes this world.
  At a pending new-world request, W cancels the request and keeps watching.
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
