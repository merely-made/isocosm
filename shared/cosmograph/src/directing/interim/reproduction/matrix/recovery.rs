// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Compact replay recipes for the separately declared bodied-stock recovery.

use super::*;
use crate::{Session, history::Saved};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Recipe {
    schema: String,
    founding: crate::Founding,
    form: Form,
    original: bool,
    epoch_ticks: Tick,
    profile_adjustment: Option<String>,
    genesis_digest: String,
    native_saved_without_genesis: Value,
}

fn recipe(
    founding: &crate::Founding,
    form: Form,
    original: bool,
    adjustment: Option<&str>,
    digest: &str,
    saved: &impl Serialize,
) -> Recipe {
    let mut native_saved_without_genesis = serde_json::to_value(saved).unwrap();
    let object = native_saved_without_genesis
        .as_object_mut()
        .expect("native Saved object");
    assert!(
        object.remove("genesis").is_some(),
        "omit duplicate native Genesis"
    );
    Recipe {
        schema: "a1-seeded-958-recovery-recipe-v1".into(),
        founding: founding.clone(),
        form,
        original,
        epoch_ticks: EPOCH,
        profile_adjustment: adjustment.map(str::to_owned),
        genesis_digest: digest.to_owned(),
        native_saved_without_genesis,
    }
}

fn reconstruct(recipe: &Recipe) -> std::result::Result<(Genesis, Saved), String> {
    if recipe.schema != "a1-seeded-958-recovery-recipe-v1" || recipe.epoch_ticks != EPOCH {
        return Err("unsupported recovery recipe profile".into());
    }
    if recipe.form != Form::Bodied || !recipe.original {
        return Err("recovery recipe is outside the bodied-stock declaration".into());
    }
    if recipe
        .profile_adjustment
        .as_deref()
        .is_some_and(|adjustment| !adjustment.is_empty())
    {
        return Err("nonempty profile adjustment is outside the bodied-stock declaration".into());
    }
    let rates = if recipe.original {
        crate::ecology::Rates::Stock
    } else {
        crate::ecology::Rates::A1
    };
    let canonical_founding = recipe.form.founding(recipe.founding.seed, rates);
    if recipe.founding != canonical_founding {
        return Err("authored Founding differs from declared native generator input".into());
    }
    let genesis = profile::world_from_founding(
        canonical_founding,
        recipe.original,
        recipe.profile_adjustment.as_deref(),
    );
    if crate::digest(&genesis) != recipe.genesis_digest {
        return Err("authored founding/profile does not match declared Genesis digest".into());
    }
    let mut saved = recipe.native_saved_without_genesis.clone();
    let object = saved.as_object_mut().ok_or("native Saved object")?;
    object.insert(
        "genesis".into(),
        serde_json::to_value(&genesis).map_err(|e| e.to_string())?,
    );
    let saved: Saved = serde_json::from_value(saved).map_err(|e| e.to_string())?;
    if saved.genesis_digest != recipe.genesis_digest {
        return Err("native Saved Genesis digest differs from recipe".into());
    }
    Ok((genesis, saved))
}

pub(super) fn validated_recipe_bytes(
    founding: &crate::Founding,
    form: Form,
    original: bool,
    adjustment: Option<&str>,
    digest: &str,
    saved: &impl Serialize,
) -> Vec<u8> {
    let value = recipe(founding, form, original, adjustment, digest, saved);
    let (genesis, saved) =
        reconstruct(&value).expect("recovery recipe rehydrates exact native save");
    assert_eq!(crate::digest(&genesis), digest);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let replay = Session::load(saved.clone(), mode).expect("recovery native replay loads");
        assert_eq!(
            replay.sim.state_hash(),
            saved.state_hash,
            "{mode:?} recovery replay hash"
        );
    }
    serde_json::to_vec(&value).unwrap()
}

pub(super) fn run_missing_bodied_original() {
    assert!(
        std::env::var("COSMOGRAPH_REPRO_TWEAK")
            .unwrap_or_default()
            .is_empty(),
        "bodied-stock recovery must use the unchanged stock profile"
    );
    let dir = receipt_directory();
    let mut missing = Vec::new();
    for seed in 0..20 {
        let matrix_stem = format!("a1-seeded-958-matrix-bodied-original-seed{seed}-");
        let recovery_stem = format!("a1-seeded-958-recovery-bodied-original-seed{seed}-");
        let complete = std::fs::read_dir(&dir).unwrap().any(|entry| {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            (name.starts_with(&matrix_stem) || name.starts_with(&recovery_stem))
                && name.ends_with("-evidence.json")
        });
        if !complete {
            missing.push(seed);
        }
    }
    eprintln!("A1 recovery missing bodied-stock draws: {missing:?}");
    for seed in missing {
        let g = profile::world(seed, Form::Bodied, true);
        let draw = Draw {
            seed,
            form: Form::Bodied,
            original: true,
            recovery: true,
        };
        let out = played_recorded(g, None, Some(draw));
        assert!(
            !out.qualifies(),
            "bodied-stock recovery seed {seed} unexpectedly qualifies"
        );
        assert_eq!(
            out.living, 0,
            "bodied-stock recovery extinction seed {seed}"
        );
    }
}

#[test]
fn recovery_recipe_rehydrates_native_save_and_rejects_authored_tampering() {
    let founding = Form::Bodied.founding(12, crate::ecology::Rates::Stock);
    let genesis = profile::world_from_founding(founding.clone(), true, None);
    let session = Session::new(genesis.clone(), Execution::Individuals).unwrap();
    let mut session = session;
    session.command(crate::Command::Collect).unwrap();
    session.advance(1).unwrap();
    let saved = session.save();
    assert_eq!(saved.entries.len(), 1);
    assert_eq!(saved.tick, 1);
    let good = recipe(
        &founding,
        Form::Bodied,
        true,
        None,
        &crate::digest(&genesis),
        &saved,
    );
    let wire = serde_json::to_vec(&good).unwrap();
    let good: Recipe = serde_json::from_slice(&wire).unwrap();
    let (rebuilt, loaded) = reconstruct(&good).expect("valid recipe");
    assert_eq!(crate::digest(&rebuilt), crate::digest(&genesis));
    assert_eq!(loaded, saved);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let replay = Session::load(loaded.clone(), mode).unwrap();
        assert_eq!(replay.sim.state_hash(), saved.state_hash);
    }
    let mut tampered = good.clone();
    tampered.profile_adjustment = Some("lifebase=101".into());
    assert!(
        reconstruct(&tampered)
            .unwrap_err()
            .contains("outside the bodied-stock declaration")
    );
    let mut digest_tampered = good;
    digest_tampered.genesis_digest.push_str("-tampered");
    assert!(
        reconstruct(&digest_tampered)
            .unwrap_err()
            .contains("Genesis digest")
    );
}
