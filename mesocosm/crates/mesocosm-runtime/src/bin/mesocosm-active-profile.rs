// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reuse the saved-update control without building the library test harness.
use mesocosm_runtime::{Runtime, runtime::MODE};
#[path = "../profile.rs"]
mod profile;
#[path = "../runtime/active_profile.rs"]
mod active_profile;
#[path = "mesocosm-active-profile/play.rs"]
mod play;

fn main() {
    if std::env::var_os("MESOCOSM_PLAY_STEPS").is_some() {
        play::run();
    } else if std::env::var_os("MESOCOSM_PROFILE_SEAL").is_some_and(|v| v == "1") {
        seal();
    } else if std::env::var_os("ISOCOSM_PROFILE_REGISTRY").is_some_and(|v| v == "1") {
        registry();
    } else {
        active_profile::saved_active_update();
    }
}

/// A serialization/COW control over the real saved witness, without replaying
/// or admitting it. Admission is exercised by the ordinary saved/head cases.
fn seal() {
    use mesocosm_runtime::{spatial::{Game,ECOLOGY_VERSION,UNMARKED_VERSION},runtime::ecology::Seal};
    use std::{hint::black_box,time::Instant};
    let bytes=std::fs::read(std::env::var("MESOCOSM_ACTIVE_SAVE").unwrap()).unwrap();
    let saved=Runtime::decode_world_save(&bytes).unwrap();
    let mut game:Game=serde_json::from_slice(&saved.sections[MODE]).unwrap();
    game.continuation=None;
    game.version=ECOLOGY_VERSION;
    let original=game.ecology.as_ref().expect("actual ecology witness");
    let encoded=serde_json::to_vec(original).unwrap();
    let decoded:Seal=serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&decoded,original);
    assert_eq!(serde_json::to_vec(&decoded).unwrap(),encoded);
    let mut unknown:serde_json::Value=serde_json::from_slice(&encoded).unwrap();
    unknown.as_object_mut().unwrap().insert("unknown".into(),true.into());
    assert!(serde_json::from_value::<Seal>(unknown).is_err());
    let reference=isocosm::digest(&game);
    let started=Instant::now();
    assert_eq!(game.physical_digest(),reference);
    let cold_us=started.elapsed().as_micros();
    let started=Instant::now();
    for _ in 0..10 {assert_eq!(black_box(&game).physical_digest(),reference);}
    let warm_us=started.elapsed().as_micros()/10;
    let mut changed=game.clone();
    changed.ecology.as_mut().unwrap().witness.native.branch.push_str(":changed");
    assert_ne!(changed.physical_digest(),reference);
    assert_eq!(changed.physical_digest(),isocosm::digest(&changed));
    assert_eq!(game.physical_digest(),reference,"COW leaves the original seal intact");
    let mut plain=Game::default();
    plain.version=UNMARKED_VERSION;
    assert_eq!(plain.physical_digest(),isocosm::digest(&plain));
    eprintln!("[seal-case] bytes={} cold_us={cold_us} warm_us={warm_us} exact=true cow=true unknown_refused=true",encoded.len());
}

/// Compare the exact public resolution with the same ordered cached digests.
fn registry() {
    use isocosm::process::{Process, Registry};
    use std::{hint::black_box, time::Instant};
    let native = Registry::native();
    let reference = native.of_native(Process::Intake).reference();
    let indexed: Vec<_> = native.all().map(|def| (def.digest(), def)).collect();
    let started = Instant::now();
    for _ in 0..100_000 {
        black_box(native.resolve(black_box(reference)).unwrap());
    }
    let current_us = started.elapsed().as_micros();
    let started = Instant::now();
    for _ in 0..100_000 {
        black_box(indexed.iter().find(|(key, _)| *key == black_box(reference).definition).unwrap().1);
    }
    let cached_us = started.elapsed().as_micros();
    for def in native.all() {
        let reference = def.reference();
        assert_eq!(native.resolve(reference), indexed.iter()
            .find(|(key, _)| *key == reference.definition).map(|(_, def)| *def));
    }
    eprintln!("[registry-case] current_us={current_us} cached_us={cached_us} calls=100000 exact=true");
}
