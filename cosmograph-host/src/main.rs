// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph_host::{save, world};

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 2 && args[0] == "--validate-save" {
        let bytes = std::fs::read(&args[1]).map_err(|why| why.to_string())?;
        let world = world::World::resume(save::WorldSave::decode(&bytes)?, cosmograph::Execution::Grouped)?;
        let cut = world.cut()?;
        println!("tick={} entries={} hash={}", cut.tick, cut.entries, cut.state_hash);
        return Ok(());
    }
    #[cfg(feature = "endpoint")]
    if args.first().is_some_and(|a| a == "--endpoint") {
        return endpoint(&args[1..]);
    }
    Err(concat!("usage: --validate-save SAVE | --endpoint (--bench SEED POPULATION SITES LINEAGES | SAVE) ",
        "[--save PATH]; the endpoint needs the `endpoint` feature").into())
}

/// Serve the inspection endpoint over stdio (ruling 901): `--endpoint --bench
/// SEED POPULATION SITES LINEAGES` founds a bench world (897), `--endpoint
/// SAVE` resumes one; `--save PATH` names where the save intent writes.
#[cfg(feature = "endpoint")]
fn endpoint(args: &[std::ffi::OsString]) -> Result<(), String> {
    use cosmograph_host::{endpoint::SimEndpoint, founding::{Purpose, found}};
    let mut args: Vec<String> = args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let save_to = match args.iter().position(|a| a == "--save") {
        Some(at) if at + 1 < args.len() => { let path = args.remove(at + 1); args.remove(at); Some(path.into()) },
        Some(_) => return Err("--save wants a path".into()),
        None => None,
    };
    let world = match args.as_slice() {
        [flag, seed, population, sites, lineages] if flag == "--bench" => {
            let n = |s: &String| s.parse::<u64>().map_err(|e| format!("{s}: {e}"));
            let genesis = cosmograph::Founding { seed: n(seed)?, population: n(population)?,
                sites: n(sites)? as u32, lineages: n(lineages)? as u32, ..Default::default() }.generate()?;
            found(genesis, cosmograph::Execution::Grouped, Purpose::Bench, None)?
        },
        [path] => world::World::resume(save::WorldSave::decode(&std::fs::read(path).map_err(|e| e.to_string())?)?,
            cosmograph::Execution::Grouped)?,
        _ => return Err("usage: --endpoint (--bench SEED POPULATION SITES LINEAGES | SAVE) [--save PATH]".into()),
    };
    let mut endpoint = SimEndpoint::new(world, save_to);
    graphshell_endpoint::stdio::serve_basic(&mut endpoint, std::io::stdin().lock(), std::io::stdout().lock())
        .map_err(|e| format!("{e:?}"))
}

