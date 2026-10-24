// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The kept readers: a v1 or v2 save is verified by the hashes it carries,
//! final and at every checkpoint, and saves again as the current version
//! (rulings 608, 641 and 651).

use super::*;

mod wire;
mod preimage;
pub use wire::{LegacyCohort, LegacyEntity, LegacyGenesis, LegacyPart, LegacyPopulation};

/// A version-1 save, whose hashes are SHA-256 over JSON (608).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedV1 {
    pub version: u32,
    pub genesis: LegacyGenesis,
    pub genesis_digest: Key,
    pub branch: Key,
    #[serde(deserialize_with = "wire::retained")]
    pub entries: Vec<Entry>,
    pub tick: Tick,
    pub state_hash: Key,
    pub checkpoints: Vec<CheckpointV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointV1 {
    pub tick: Tick,
    pub state_hash: Key,
}

/// A version-2 save, whose hashes are FNV-1a over the whole state's
/// postcard bytes (634 and 636).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedV2 {
    pub version: u32,
    pub genesis: LegacyGenesis,
    pub genesis_digest: Key,
    pub branch: Key,
    #[serde(deserialize_with = "wire::retained")]
    pub entries: Vec<Entry>,
    pub tick: Tick,
    pub state_hash: u64,
    pub checkpoints: Vec<CheckpointV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
        verify(saved.version == 1, old, mode, preimage::sha)
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
        verify(saved.version == 2, old, mode, preimage::fnv)
    }
}

struct Old<H> {
    genesis: LegacyGenesis,
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
    hash: fn(&Simulation, &preimage::Family) -> Result<H>,
) -> Result<Session> {
    if !version || crate::digest(&old.genesis) != old.genesis_digest {
        return Err("save version or genesis digest mismatch".into());
    }
    // Only authenticated old facts may supply retired geometry metadata.
    // This first family has virtual parts and reactive/inert actors; wider
    // body histories need their own lossless admission, not a default body.
    let family = preimage::Family::new(&old.genesis)?;
    if old.entries.iter().any(|e| !matches!(e.command,
        Command::Act { .. } | Command::Inspect(_) | Command::Release(_)
        | Command::Collect | Command::Learn { .. } | Command::PlaceMatter { .. }))
    {
        return Err("unsupported legacy command".into());
    }
    let genesis = old.genesis.native()?;
    let stops: Vec<Tick> = old.checkpoints.iter().map(|c| c.0).collect();
    let check = |i: usize, sim: &Simulation| {
        if hash(sim, &family)? == old.checkpoints[i].1 {
            Ok(())
        } else {
            Err("checkpoint history mismatch".to_string())
        }
    };
    let session = Session::replay(
        genesis,
        old.branch,
        old.entries,
        old.tick,
        mode,
        Movement::ImmediateV3,
        None,
        &stops,
        check,
    )?;
    if hash(&session.sim, &family)? != old.state_hash {
        return Err("save state hash mismatch".into());
    }
    if !session.checkpoints.iter().map(|c| c.tick).eq(stops) {
        return Err("checkpoint history mismatch".into());
    }
    // Nothing is exposed on a failed old hash or outcome. Also require the
    // authenticated bridge to be reproducible as an ordinary Saved4 prefix.
    VerifiedLegacy(session).migrate(mode)
}

struct VerifiedLegacy(Session);

impl VerifiedLegacy {
    fn migrate(self, mode: Execution) -> Result<Session> {
        let saved = self.0.save();
        let replay = Session::load(saved.clone(), mode)?;
        if replay.save() != saved {
            return Err("legacy migration is not reproducible as a current save".into());
        }
        Ok(self.0)
    }
}
