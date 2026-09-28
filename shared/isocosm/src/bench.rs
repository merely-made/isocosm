// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Execution, Founding, Result, Session,
    aggregate::{self, Comparison},
    history::Command,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draws {
    pub version: u32,
    pub master_seed: u64,
    pub domain: String,
    pub comparisons: Vec<Comparison>,
    pub saved_replays: u64,
    pub refusal_checks: u64,
}

/// Receipts vary laws and topology, as well as initial populations. The domain
/// is explicit; this does not certify unimplemented social/spatial reductions.
pub fn draws(master_seed: u64, count: u64, ticks: u64) -> Result<Draws> {
    if count == 0 || count > 1024 || ticks > 1000 {
        return Err("bench draw budget exceeded".into());
    }
    let mut report = Draws { version: crate::VERSION, master_seed,
        domain: "alternating unary reservoir networks and shared-resource ecological compartments; 1..8 sites, 1..6 lineages, 16..128 initial members, 2..4 accounts/lineage; changing observed identities; no spatial-contact claim".into(),
        comparisons: vec![], saved_replays: 0, refusal_checks: 0 };
    for i in 0..count {
        let seed = crate::draw(master_seed, "bench-seed", &[i]);
        let founding = Founding {
            seed,
            sites: (1 + seed % 8) as u32,
            lineages: (1 + seed.rotate_left(7) % 6) as u32,
            population: 16 + seed.rotate_left(13) % 113,
            cohort_size: 4 + seed.rotate_left(23) % 29,
            ecology: i % 2 == 1,
            ..Founding::default()
        };
        report.comparisons.push(
            aggregate::compare(&founding, ticks)
                .map_err(|why| format!("draw {i}, seed {seed}: {why}"))?,
        );
        let mut session = Session::new(founding.generate()?, Execution::Grouped)?;
        session.advance(ticks)?;
        let actor = u64::from(founding.sites);
        session.command(Command::Act {
            actor,
            target: None,
            process: "sim:remember".into(),
            cause: None,
        })?;
        let encoded = serde_json::to_vec(&session.save()).map_err(|e| e.to_string())?;
        let saved = serde_json::from_slice(&encoded).map_err(|e| e.to_string())?;
        let replay = Session::load(saved, Execution::Individuals)?;
        if replay.sim.state_hash() != session.sim.state_hash() {
            return Err("saved replay mismatch".into());
        }
        report.saved_replays += 1;
        let before = session.sim.state_hash();
        let refusal = session.sim.execute(actor, None, "unknown:process", None);
        if !matches!(refusal.outcome, crate::simulation::Outcome::Refused(_))
            || before != session.sim.state_hash()
        {
            return Err("refusal was not inert".into());
        }
        report.refusal_checks += 1;
    }
    Ok(report)
}

/// One drawn world map and what SP1 checked on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDraw {
    pub seed: u64,
    pub grid: crate::map::Grid,
    pub borders: u64,
    pub corners: u64,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDraws {
    pub version: u32,
    pub master_seed: u64,
    pub domain: String,
    pub draws: Vec<MapDraw>,
}

/// SP1's receipt (the spine plan, §A.5): drawn world maps whose borders read
/// the same from both sides, whose corners agree, and which found the same
/// world twice.
pub fn map_draws(master_seed: u64, count: u64) -> Result<MapDraws> {
    use crate::terrain::{View, check};
    if count == 0 || count > 1024 {
        return Err("bench draw budget exceeded".into());
    }
    let mut report = MapDraws {
        version: crate::VERSION,
        master_seed,
        domain: "square sites on planes, rings and tori; 2..16 by 2..16 sites; sides 256..2048 base units; elevation within one side, relief within an eighth".into(),
        draws: vec![],
    };
    for i in 0..count {
        let seed = crate::draw(master_seed, "map-seed", &[i]);
        let grid = crate::map::Grid::drawn(seed);
        let founding = Founding {
            seed,
            sites: grid.width * grid.height,
            map: Some(crate::map::Layout::Grid(grid.clone())),
            ..Founding::default()
        };
        let genesis = founding
            .generate()
            .map_err(|why| format!("draw {i}, seed {seed}: {why}"))?;
        let view = View::of(&genesis)?;
        let borders = check::profiles(&view, |v, s, k| v.edge_profile(s, k))
            .map_err(|why| format!("draw {i}, seed {seed}: {why}"))?;
        let corners = check::corners(&view, |v, s, c| v.corner_height(s, c))
            .map_err(|why| format!("draw {i}, seed {seed}: {why}"))?;
        let digest = crate::digest(&genesis);
        if crate::digest(&founding.generate()?) != digest {
            return Err(format!("draw {i}, seed {seed}: one founding laid two maps"));
        }
        report.draws.push(MapDraw {
            seed,
            grid,
            borders,
            corners,
            digest,
        });
    }
    Ok(report)
}

/// The world a map seed founds, as both map receipts draw it.
fn mapped(seed: u64) -> Result<crate::simulation::Genesis> {
    let grid = crate::map::Grid::drawn(seed);
    Founding {
        seed,
        sites: grid.width * grid.height,
        map: Some(crate::map::Layout::Grid(grid)),
        ..Founding::default()
    }
    .generate()
}

/// The chunks a lift receipt compares: the first and last chunk of `site`
/// at the base grain, and every chunk three levels up.
fn sample(view: &crate::terrain::View<'_>, site: u64) -> Result<Vec<crate::terrain::Chunk>> {
    let last = view.chunks(0) - 1;
    let mut chunks = vec![
        view.lift(site, 0, [0, 0])?,
        view.lift(site, 0, [last, last])?,
    ];
    let n = view.chunks(3);
    for z in 0..n {
        for x in 0..n {
            chunks.push(view.lift(site, 3, [x, z])?);
        }
    }
    Ok(chunks)
}

/// The digest of the sampled chunks of a map seed's first `sites` sites:
/// what a second process must repeat (the spine plan's SP2).
pub fn lift_digest(seed: u64, sites: u64) -> Result<String> {
    let genesis = mapped(seed)?;
    let view = crate::terrain::View::of(&genesis)?;
    let mut chunks = Vec::new();
    for &site in view.sites.keys().take(sites as usize) {
        chunks.extend(sample(&view, site)?);
    }
    Ok(crate::digest(&chunks))
}

/// One drawn world's lift and what SP2 checked on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiftDraw {
    pub seed: u64,
    pub grid: crate::map::Grid,
    pub borders: u64,
    pub means: u64,
    pub reliefs: u64,
    pub chunks: u64,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiftDraws {
    pub version: u32,
    pub master_seed: u64,
    pub domain: String,
    pub draws: Vec<LiftDraw>,
}

/// SP2's receipt: drawn worlds whose neighbours meet exactly at every
/// base-grain border point, whose first and last sites' means are their
/// elevations exactly, whose detail stays within relief, and whose sampled
/// chunks lift the same bytes twice.
pub fn lift_draws(master_seed: u64, count: u64) -> Result<LiftDraws> {
    use crate::terrain::{View, check};
    if count == 0 || count > 1024 {
        return Err("bench draw budget exceeded".into());
    }
    let mut report = LiftDraws {
        version: crate::VERSION,
        master_seed,
        domain: "the map draws' space; every border at the base grain, the mean of the first and last sites by brute force, detail against relief at every site, sampled chunks at levels 0 and 3".into(),
        draws: vec![],
    };
    for i in 0..count {
        let seed = crate::draw(master_seed, "lift-seed", &[i]);
        let genesis = mapped(seed).map_err(|why| format!("draw {i}, seed {seed}: {why}"))?;
        let view = View::of(&genesis)?;
        let fail = |why: String| format!("draw {i}, seed {seed}: {why}");
        let borders = check::borders(&view, |v, s| v.lattice(s)).map_err(fail)?;
        let ends = [
            *view.sites.keys().next().unwrap(),
            *view.sites.keys().last().unwrap(),
        ];
        let means = check::means(&view, &ends, |v, s| v.lattice(s)).map_err(fail)?;
        let reliefs = check::reliefs(&view, |v, s| v.lattice(s)).map_err(fail)?;
        let chunks = sample(&view, ends[0])?;
        let digest = crate::digest(&chunks);
        if crate::digest(&sample(&view, ends[0])?) != digest {
            return Err(fail("one site lifted two ways".into()));
        }
        report.draws.push(LiftDraw {
            seed,
            grid: match genesis.founding.and_then(|f| f.map) {
                Some(crate::map::Layout::Grid(grid)) => grid,
                None => return Err(fail("a map draw without its map".into())),
            },
            borders,
            means,
            reliefs,
            chunks: chunks.len() as u64,
            digest,
        });
    }
    Ok(report)
}
