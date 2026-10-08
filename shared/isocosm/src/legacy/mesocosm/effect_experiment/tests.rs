use super::*;

#[test]
fn authored_rules_are_guaranteed_across_seed_and_appearance_changes() {
    for (receiver, response) in [
        (Receiver::Stone, Response::Reflect),
        (Receiver::Metal, Response::Split),
        (Receiver::Moss, Response::Bind),
    ] {
        for seed in [0, 1, 7, u64::MAX] {
            for &glyph in Glyph::ALL {
                let exp = Request {
                    receiver,
                    world_seed: seed,
                    appearance_seed: seed,
                    glyph,
                    ..Default::default()
                }
                .prepare()
                .unwrap();
                assert_eq!(exp.report.response, response);
                assert_eq!(exp.report.origin, RuleOrigin::Guaranteed);
            }
        }
    }
}

#[test]
fn generated_material_laws_are_stable_and_independent_of_presentation() {
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 0..64 {
        for &receiver in Receiver::ALL {
            let req = Request {
                world_seed: seed,
                receiver,
                profile: Profile::Generated,
                ..Default::default()
            };
            let baseline = req.prepare().unwrap();
            outcomes.insert(baseline.report.response.label());
            for &glyph in Glyph::ALL {
                for &behavior in Behavior::ALL {
                    let changed = Request {
                        appearance_seed: seed + 1,
                        glyph,
                        behavior,
                        count: 100,
                        ..req.clone()
                    }
                    .prepare()
                    .unwrap();
                    assert_eq!(changed.report.response, baseline.report.response);
                }
            }
        }
    }
    assert_eq!(outcomes.len(), 4);
}

#[test]
fn saved_request_and_out_of_order_sampling_replay_exactly() {
    let req = Request {
        profile: Profile::Generated,
        ..Default::default()
    };
    let exp = req.prepare().unwrap();
    let restored: Request = serde_json::from_str(&serde_json::to_string(&req).unwrap()).unwrap();
    let replay = restored.prepare().unwrap();
    assert_eq!(exp, replay);
    let a = exp.sample(73);
    exp.sample(15);
    assert_eq!(a, exp.sample(73));
    assert_eq!(a, replay.sample(73));
    assert!(exp.sample(req.duration_ticks).is_empty());
}

#[test]
fn grouping_requires_glyph_and_behavior_compatibility() {
    for &glyph in Glyph::ALL {
        for &behavior in Behavior::ALL {
            let exp = Request {
                glyph,
                behavior,
                receiver: Receiver::Moss,
                ..Default::default()
            }
            .prepare()
            .unwrap();
            let expected = match (glyph, behavior) {
                (Glyph::Quotes, Behavior::Enclose) => Connection::Pairs,
                (Glyph::Backticks, Behavior::Inscribe) => Connection::String,
                _ => Connection::Separate,
            };
            assert_eq!(exp.report.connection, expected);
            assert!(
                exp.sample(80)
                    .iter()
                    .all(|m| m.group.is_some() == (expected != Connection::Separate))
            );
        }
    }
}

#[test]
fn receiver_response_changes_post_contact_sample_with_bounded_work() {
    let mut samples = Vec::new();
    for &receiver in Receiver::ALL {
        let exp = Request {
            receiver,
            behavior: Behavior::Stream,
            ..Default::default()
        }
        .prepare()
        .unwrap();
        assert_eq!(exp.sample(30).len(), 24);
        let marks = exp.sample(100);
        assert!(marks.len() <= 48);
        if receiver == Receiver::Metal {
            assert_eq!(marks.len(), 48);
        }
        samples.push(marks);
    }
    assert_ne!(samples[0], samples[1]);
    assert_ne!(samples[0], samples[2]);
}

#[test]
fn stream_emits_a_spatial_sequence_and_reacts_per_mark_arrival() {
    let exp = Request {
        receiver: Receiver::Metal,
        behavior: Behavior::Stream,
        ..Default::default()
    }
    .prepare()
    .unwrap();
    assert_eq!(exp.sample(0).len(), 1);
    let midway = exp.sample(60);
    assert_eq!(midway.len(), 24);
    let xs: std::collections::BTreeSet<_> = midway.iter().map(|m| m.x).collect();
    assert_eq!(xs.len(), 24);
    assert!(midway.iter().all(|m| m.x < 600));
    let arriving = exp.sample(90);
    assert!(arriving.len() > 24 && arriving.len() < 48);
    let last = arriving.iter().find(|m| m.id == 23).unwrap();
    assert!(last.x < 600);
    assert!(!arriving.iter().any(|m| m.id == 47));
    assert_eq!(exp.sample(100).len(), 48);
    assert!(exp.sample(120).is_empty());
}

#[test]
fn invalid_saved_requests_are_refused_before_sampling() {
    for req in [
        Request {
            version: 2,
            ..Default::default()
        },
        Request {
            count: 0,
            ..Default::default()
        },
        Request {
            count: 129,
            ..Default::default()
        },
        Request {
            duration_ticks: 0,
            ..Default::default()
        },
        Request {
            duration_ticks: 3601,
            ..Default::default()
        },
    ] {
        assert!(req.prepare().is_err());
    }
    assert!(serde_json::from_str::<Request>(r#"{"invented_field":1}"#).is_err());
}
