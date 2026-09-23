// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{
    Execution, Founding, Session,
    history::{Command, Saved},
};

fn main() {
    if let Err(why) = run() {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut founding = Founding {
        seed: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos() as u64,
        ..Default::default()
    };
    let mut ticks = 32;
    let mut draws = None;
    let mut output = None;
    let mut load = None;
    let mut world = None;
    let mut mode = Execution::Grouped;
    while let Some(arg) = args.next() {
        if arg == "--individuals" {
            mode = Execution::Individuals;
            continue;
        }
        if arg == "--ecology" {
            founding.ecology = true;
            continue;
        }
        if arg == "--help" {
            println!(
                "isocosm-bench [--seed N] [--ticks N] [--population N] [--sites N] [--lineages N] [--cohort-size N] [--ecology] [--draws N] [--load SAVE | --world GENESIS] [--output FILE] [--individuals]"
            );
            return Ok(());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {arg}"))?;
        let number = || {
            value
                .parse::<u64>()
                .map_err(|_| format!("invalid integer for {arg}"))
        };
        match arg.as_str() {
            "--seed" => founding.seed = number()?,
            "--ticks" => ticks = number()?,
            "--population" => founding.population = number()?,
            "--cohort-size" => founding.cohort_size = number()?,
            "--sites" => founding.sites = u32::try_from(number()?).map_err(|e| e.to_string())?,
            "--lineages" => {
                founding.lineages = u32::try_from(number()?).map_err(|e| e.to_string())?
            },
            "--draws" => draws = Some(number()?),
            "--output" => output = Some(value),
            "--load" => load = Some(value),
            "--world" => world = Some(value),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    let json = if let Some(count) = draws {
        eprintln!("Generated-draw receipt: master seed {}, draws {count}, ticks {ticks}",founding.seed);
        let report = isocosm::bench::draws(founding.seed,count,ticks)?;
        eprintln!("{} generated comparisons, {} saved replays, master seed {}",report.comparisons.len(),report.saved_replays,founding.seed);
        serde_json::to_string_pretty(&report)
    } else {
        let mut session = if let Some(path) = load {
            let bytes = std::fs::read(path).map_err(|e|e.to_string())?;
            let saved: Saved = serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
            Session::load(saved,mode)?
        } else if let Some(path) = world {
            let bytes=std::fs::read(path).map_err(|e|e.to_string())?;
            let genesis=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
            Session::new(genesis,mode)?
        } else { Session::new(founding.generate()?,mode)? };
        let work = session.advance(ticks)?;
        // Observation is an explicit log entry, never a side effect of drawing UI.
        let actor = session.sim.state().population.groups.iter()
            .find(|(_,g)|g.entity.method != isocosm::schema::Method::Inert).map(|(id,_)|*id).ok_or("no critter")?;
        session.command(Command::Inspect(actor))?;
        eprintln!("seed {} · tick {} · {} entities · {} groups · {} evaluations for {} represented acts · {}",
            session.sim.genesis().seed, session.sim.state().tick, session.sim.state().population.count(),
            session.sim.state().population.groups.len(),work.evaluations,work.represented,session.sim.state_hash());
        serde_json::to_string_pretty(&session.save())
    }.map_err(|e|e.to_string())?;
    if let Some(path) = output {
        std::fs::write(path, json).map_err(|e| e.to_string())?;
    } else {
        println!("{json}");
    }
    Ok(())
}
