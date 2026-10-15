// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm_host::{save, world};
#[cfg(feature = "desktop")]
use isocosm_host::{access, binding, file_save, native_attention, spatial, stream, window};
#[cfg(feature = "desktop")]
mod mode_sections;
#[cfg(any(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
mod viewport;
#[cfg(feature = "desktop")]
mod cli;
#[cfg(feature = "desktop")]
mod desktop;
#[cfg(feature = "desktop")]
mod products;
#[cfg(feature = "desktop")]
mod ui;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 2 && args[0] == "--validate-save" {
        let bytes = std::fs::read(&args[1]).map_err(|why| why.to_string())?;
        let world = world::World::resume(save::WorldSave::decode(&bytes)?, isocosm::Execution::Grouped)?;
        let cut = world.cut()?;
        println!("tick={} entries={} hash={}", cut.tick, cut.entries, cut.state_hash);
        return Ok(());
    }
    #[cfg(feature = "desktop")]
    { desktop::run(cli::open(&args)?) }
    #[cfg(not(feature = "desktop"))]
    { Err("usage: --validate-save <save>; windowed play requires a product feature".into()) }
}

#[cfg(all(test, feature = "mesocosm", feature = "vtt", feature = "eponym"))]
mod shell_tests;
