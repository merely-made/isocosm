// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Export a spherical founding and stream successive observations from its save.
//! The viewer reads the same session for orbit, valley and individual views.
#[path = "planet_atlas/observation.rs"]
mod observation;
use cosmograph::{Execution, Founding, Session};
use observation::{Result, frame, terrain};
use serde_json::Value;
use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
};

fn next(s: &mut Session) -> Result<Value> {
    let result = s.advance_tick_with_flows()?;
    frame(s, &result.flows)
}

fn write_json(path: &Path, v: &impl serde::Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(v)?)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, config, directory] if mode == "found" => {
            let d = Path::new(directory);
            fs::create_dir_all(d)?;
            let f: Founding = toml::from_str(&fs::read_to_string(config)?)?;
            let mut s = Session::new(f.generate()?, Execution::Grouped)?;
            write_json(&d.join("terrain.json"), &terrain(&s)?)?;
            write_json(&d.join("founding.json"), &s.save())?;
            let mut frames = vec![frame(&s, &[])?];
            for _ in 0..24 {
                frames.push(next(&mut s)?);
            }
            write_json(&d.join("frames.json"), &frames)?;
            write_json(&d.join("resume.json"), &s.save())
        },
        [mode, save] if mode == "watch" => {
            let mut s = Session::load_json(&fs::read(save)?, Execution::Grouped)?;
            let stdout = io::stdout();
            let mut out = io::BufWriter::new(stdout.lock());
            for line in io::stdin().lock().lines() {
                let f = match line?.as_str() {
                    "next" => next(&mut s)?,
                    "now" => frame(&s, &[])?,
                    _ => return Err("expected next or now".into()),
                };
                serde_json::to_writer(&mut out, &f)?;
                writeln!(out)?;
                out.flush()?;
            }
            Ok(())
        },
        _ => Err("usage: planet_atlas found TEMPLATE DIRECTORY | watch SAVE".into()),
    }
}
