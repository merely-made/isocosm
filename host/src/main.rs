// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

mod access;
mod binding;
mod file_save;
mod native_attention;
mod save;
mod stream;
mod spatial;
#[cfg(feature = "desktop")]
mod mode_sections;
mod window;
mod world;
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

#[cfg(test)]
mod tests;
