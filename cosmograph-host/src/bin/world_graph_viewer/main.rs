// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world-graph viewer (sim host plan H9, cases 1038 and 1048): a small
//! native Graphshell client of the core's inspection endpoint that draws a
//! world as one graph and moves it through time.
//!
//! ```text
//! world-graph-viewer [--seed N] [--sites N] [--population N] [--lineages N]
//! world-graph-viewer --save PATH
//! ```
//!
//! Without `--seed` the seed is read from the clock, so nobody chose it; it
//! is printed so the world can be founded again. A generated world is
//! founded under the bench purpose, the viewer being an instrument (1048).
//! `WORLD_GRAPH_VIEWER_SCENARIO`, `_CAPTURE_DIR` and `_RECEIPT` self-drive it.

mod board;
mod client;
mod lane;
mod view;

use std::cell::RefCell;
use std::rc::Rc;

use cambium_genet_winit_host::{AppCtx, CloseDisposition, HostHooks, HostOptions, Init, Runner, run};
use cosmograph::{Execution, Founding};
use cosmograph_host::endpoint::SimEndpoint;
use cosmograph_host::founding::{Purpose, found};
use cosmograph_host::{save::WorldSave, world::World};

use board::{Board, LEAF, producer};
use client::Client;
use view::{Child, Logic, SHEET, Viewer, root};

fn world(args: &[String]) -> Result<World, String> {
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|at| args.get(at + 1));
    if let Some(path) = flag("--save") {
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        return World::resume(WorldSave::decode(&bytes)?, Execution::Grouped);
    }
    let number = |name: &str, default: u64| flag(name).map_or(Ok(default), |v| v.parse::<u64>()
        .map_err(|e| format!("{name} {v}: {e}")));
    let seed = match flag("--seed") {
        Some(_) => number("--seed", 0)?,
        None => std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?.as_nanos() as u64,
    };
    let founding = Founding { seed, sites: number("--sites", 12)? as u32, population: number("--population", 200)?,
        lineages: number("--lineages", 6)? as u32, ecology: true, ..Default::default() };
    eprintln!("[world-graph-viewer] seed={seed} sites={} population={} lineages={}",
        founding.sites, founding.population, founding.lineages);
    found(founding.generate()?, Execution::Grouped, Purpose::Bench, None)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let client = Client::new(SimEndpoint::new(world(&args)?, None))?;
    let board = Rc::new(RefCell::new(Board::default()));
    board.borrow_mut().sync(&client);

    let lane = mesquite::LaneConfig::from_env("WORLD_GRAPH_VIEWER").map(|config| {
        mesquite::Lane::from_config(config, lane::Lane, cambium_genet_winit_host::read_file)
            .unwrap_or_else(|why| panic!("{why}"))
    });
    let lane = Rc::new(RefCell::new(lane));
    let (registered, after) = (board.clone(), lane.clone());
    let hooks: HostHooks<Viewer, Logic, Child> = HostHooks {
        frame: Box::new(move |ctx: &mut AppCtx<'_, Viewer, Logic, Child>| {
            if !ctx.producers.contains(LEAF)
                && let Err(why) = ctx.producers.register(LEAF, producer(registered.clone()), &[]) {
                eprintln!("[world-graph-viewer] the board's leaf: {why:?}");
            }
            ctx.runner.state().board.borrow_mut().step()
        }),
        after_dispatch: Box::new(|_| {}),
        after_frame: Box::new(move |ctx: &mut AppCtx<'_, Viewer, Logic, Child>| {
            if let Some(lane) = after.borrow_mut().as_mut() { lane.after_frame(ctx); }
        }),
        after_wake: Box::new(|_| {}),
        close_request: Box::new(|_, _| CloseDisposition::Exit),
        focused_text: Box::new(|_: &Runner<Viewer, Logic, Child>| None),
        key_intercept: Box::new(|_, _| false),
    };
    let options = HostOptions {
        title: "World graph".into(),
        initial_logical_size: (1280.0, 800.0),
        ..Default::default()
    };
    run(options, move |_, _, _| Init {
        state: Viewer { client, board, events: Vec::new(), graph_size: (960, 780), drag: None },
        logic: root as Logic,
        sheet: SHEET.to_string(),
        fonts: Vec::new(),
        images: Vec::new(),
    }, hooks).map_err(|e| e.to_string())
}
