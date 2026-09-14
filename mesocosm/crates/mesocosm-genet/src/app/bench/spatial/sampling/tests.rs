// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use mesocosm_core::{PartId, effect_experiment::Glyph};
use mesocosm_mesh::BodyDependencyRevision;
use isometer::{PartAddress, SubjectKey};

const BOUNDS: ([f32; 3], [f32; 3]) = ([-10., -4., -8.], [12., 16., 10.]);
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn difference(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] - b[i])
}
fn anchors() -> [GlyphAnchor; 2] {
    let a = 0.6_f32;
    [
        GlyphAnchor {
            selection: PartAddress {
                subject: SubjectKey(1),
                part: PartId(2),
                revision: BodyDependencyRevision(1),
            },
            part: PartId(2),
            centre: [-4., 2., -3.],
            right: [a.cos(), 0., a.sin()],
            up: [0., 1., 0.],
            normal: [-a.sin(), 0., a.cos()],
            extent: [3., 8.],
        },
        GlyphAnchor {
            selection: PartAddress {
                subject: SubjectKey(1),
                part: PartId(9),
                revision: BodyDependencyRevision(1),
            },
            part: PartId(9),
            centre: [6., 5., 7.],
            right: [1., 0., 0.],
            up: [0., 0., -1.],
            normal: [0., 1., 0.],
            extent: [7., 2.],
        },
    ]
}
fn model(form: Form, glyph: Glyph, count: u16) -> Spatial {
    Spatial {
        enabled: true,
        form,
        glyph,
        count,
        ..Spatial::default()
    }
}

#[test]
fn every_form_and_glyph_is_bounded_finite_and_replays_out_of_order() {
    let anchors = anchors();
    for form in Form::ALL {
        for &glyph in Glyph::ALL {
            for count in [2, 18, 128] {
                let mut spatial = model(form, glyph, count);
                let original = spatial.marks(BOUNDS, &anchors);
                for tick in [359, 31, 120, 0, 240, 31] {
                    spatial.tick = tick;
                    let first = spatial.marks(BOUNDS, &anchors);
                    assert_eq!(
                        first.len(),
                        usize::from(count),
                        "{form:?}/{glyph:?}/{count}"
                    );
                    assert!(first.iter().all(|m| {
                        m.centre.iter().chain(m.color.iter()).all(|v| v.is_finite())
                            && m.size.is_finite()
                            && m.size > 0.
                            && m.angle.is_finite()
                            && m.color[3] == 1.
                    }));
                    assert!(first.iter().all(|m| m.glyph == glyph));
                    spatial.tick = 359 - tick;
                    let _ = spatial.marks(BOUNDS, &anchors);
                    spatial.tick = tick;
                    assert_eq!(
                        spatial.marks(BOUNDS, &anchors),
                        first,
                        "sampling order changes replay"
                    );
                    assert_eq!(spatial.seed, 7, "sampling must not consume seed state");
                }
                spatial.tick = 0;
                assert_eq!(spatial.marks(BOUNDS, &anchors), original);
            }
        }
    }
}

#[test]
fn appearance_seeds_are_independent_and_reversible_for_every_form() {
    let anchors = anchors();
    for form in Form::ALL {
        let mut spatial = model(form, Glyph::Quotes, 128);
        spatial.tick = 47;
        let initial = spatial.marks(BOUNDS, &anchors);
        spatial.seed = 8;
        assert_ne!(
            initial,
            spatial.marks(BOUNDS, &anchors),
            "{form:?}: seed must change expression"
        );
        spatial.seed = 7;
        assert_eq!(initial, spatial.marks(BOUNDS, &anchors));
        assert_eq!(spatial.tick, 47);
    }
}

#[test]
fn surface_stroke_envelopes_fit_offset_actual_face_planes() {
    let anchors = anchors();
    for &glyph in Glyph::ALL {
        for count in [2, 18, 128] {
            for seed in [0, 7, 42, u64::MAX] {
                let mut spatial = model(Form::Surface, glyph, count);
                spatial.seed = seed;
                for mark in spatial.marks(BOUNDS, &anchors) {
                    let GlyphOrientation::WorldPlane { right, up } = mark.orientation else {
                        panic!("surface requires a world plane")
                    };
                    // Independent conservative local envelope: every punctuation
                    // stroke, including thickness, lies inside +/-0.56 glyph units.
                    // Test all four corners after the host-selected in-plane rotation,
                    // rather than only centres or reconstructed slot indices.
                    let anchor = anchors
                        .iter()
                        .find(|a| a.right == right && a.up == up)
                        .unwrap();
                    for x in [-0.56_f32, 0.56] {
                        for y in [-0.56_f32, 0.56] {
                            let dx = mark.size * (x * mark.angle.cos() - y * mark.angle.sin());
                            let dy = mark.size * (x * mark.angle.sin() + y * mark.angle.cos());
                            let point =
                                [0, 1, 2].map(|i| mark.centre[i] + right[i] * dx + up[i] * dy);
                            let relative = difference(point, anchor.centre);
                            assert!(
                                (dot(relative, anchor.normal) - 0.01).abs() < 1e-4,
                                "glyph leaves offset face plane"
                            );
                            assert!(
                                dot(relative, right).abs() <= anchor.extent[0] * 0.5 + 1e-4,
                                "stroke overhangs face width"
                            );
                            assert!(
                                dot(relative, up).abs() <= anchor.extent[1] * 0.5 + 1e-4,
                                "stroke overhangs face height"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn attached_forms_refuse_missing_faces_and_tether_requires_two() {
    let anchors = anchors();
    for form in [Form::Surface, Form::Emission, Form::Tether] {
        let mut spatial = model(form, Glyph::Slashes, 18);
        assert!(spatial.marks(BOUNDS, &[]).is_empty());
        if form == Form::Tether {
            assert!(spatial.marks(BOUNDS, &anchors[..1]).is_empty());
        }
        spatial.enabled = false;
        assert!(spatial.marks(BOUNDS, &anchors).is_empty());
    }
    assert_eq!(
        model(Form::Orbit, Glyph::Slashes, 18)
            .marks(BOUNDS, &[])
            .len(),
        18
    );
}

#[test]
fn emission_stays_outside_source_faces_and_tethers_span_their_endpoints() {
    let anchors = anchors();
    let mut emission = model(Form::Emission, Glyph::Backticks, 128);
    for tick in [0, 31, 119, 240, 359] {
        emission.tick = tick;
        for mark in emission.marks(BOUNDS, &anchors) {
            assert!(
                anchors.iter().any(|a| {
                    let d = difference(mark.centre, a.centre);
                    dot(d, a.normal) >= 0.0199 && dot(d, a.up).abs() < 1e-4
                }),
                "emission must leave an actual source face"
            );
        }
    }
    let tether = model(Form::Tether, Glyph::Quotes, 128).marks(BOUNDS, &anchors);
    for pair in tether.windows(2) {
        assert!(pair[0].centre[0] < pair[1].centre[0] && pair[0].centre[2] < pair[1].centre[2]);
    }
    for mark in tether {
        assert!(mark.centre[0] > -4. && mark.centre[0] < 6.);
        assert!(mark.centre[2] > -3. && mark.centre[2] < 7.);
        assert!(mark.centre[1] > 2.);
    }
}
