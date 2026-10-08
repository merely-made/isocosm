// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The kept readers: a v1 or v2 save is verified by the hashes it carries,
//! final and at every checkpoint, and saves again as the current version
//! (rulings 608, 641 and 651).

use super::*;

/// A version-1 save, whose hashes are SHA-256 over JSON (608).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedV1 {
    pub version: u32,
    pub genesis: Genesis,
    pub genesis_digest: Key,
    pub branch: Key,
    pub entries: Vec<Entry>,
    pub tick: Tick,
    pub state_hash: Key,
    pub checkpoints: Vec<CheckpointV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointV1 {
    pub tick: Tick,
    pub state_hash: Key,
}

/// A version-2 save, whose hashes are FNV-1a over the whole state's
/// postcard bytes (634 and 636).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedV2 {
    pub version: u32,
    pub genesis: Genesis,
    pub genesis_digest: Key,
    pub branch: Key,
    pub entries: Vec<Entry>,
    pub tick: Tick,
    pub state_hash: u64,
    pub checkpoints: Vec<CheckpointV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointV2 {
    pub tick: Tick,
    pub state_hash: u64,
}

impl Session {
    pub fn load_v1(saved: SavedV1, mode: Execution) -> Result<Self> {
        let checkpoints = saved
            .checkpoints
            .into_iter()
            .map(|c| (c.tick, c.state_hash));
        let old = Old {
            genesis: saved.genesis,
            genesis_digest: saved.genesis_digest,
            branch: saved.branch,
            entries: saved.entries,
            tick: saved.tick,
            state_hash: saved.state_hash,
            checkpoints: checkpoints.collect(),
        };
        verify(saved.version == 1, old, mode, Simulation::state_hash_v1)
    }
    pub fn load_v2(saved: SavedV2, mode: Execution) -> Result<Self> {
        let checkpoints = saved
            .checkpoints
            .into_iter()
            .map(|c| (c.tick, c.state_hash));
        let old = Old {
            genesis: saved.genesis,
            genesis_digest: saved.genesis_digest,
            branch: saved.branch,
            entries: saved.entries,
            tick: saved.tick,
            state_hash: saved.state_hash,
            checkpoints: checkpoints.collect(),
        };
        verify(saved.version == 2, old, mode, Simulation::state_hash_v2)
    }
}

struct Old<H> {
    genesis: Genesis,
    genesis_digest: Key,
    branch: Key,
    entries: Vec<Entry>,
    tick: Tick,
    state_hash: H,
    checkpoints: Vec<(Tick, H)>,
}

/// Replays, stopping at each recorded checkpoint to compare its hash.
fn verify<H: PartialEq>(
    version: bool,
    old: Old<H>,
    mode: Execution,
    hash: fn(&Simulation) -> H,
) -> Result<Session> {
    if !version || crate::digest(&old.genesis) != old.genesis_digest {
        return Err("save version or genesis digest mismatch".into());
    }
    let stops: Vec<Tick> = old.checkpoints.iter().map(|c| c.0).collect();
    let check = |i: usize, sim: &Simulation| {
        if hash(sim) == old.checkpoints[i].1 {
            Ok(())
        } else {
            Err("checkpoint history mismatch".to_string())
        }
    };
    let session = Session::replay(
        old.genesis,
        old.branch,
        old.entries,
        old.tick,
        mode,
        &stops,
        check,
    )?;
    if hash(&session.sim) != old.state_hash {
        return Err("save state hash mismatch".into());
    }
    if !session.checkpoints.iter().map(|c| c.tick).eq(stops) {
        return Err("checkpoint history mismatch".into());
    }
    Ok(session)
}
