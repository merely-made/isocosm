// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::super::state::Bench;
use super::{Form, Spatial};
use mesocosm_core::effect_experiment::Glyph;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    version: u8,
    form: Form,
    glyph: Glyph,
    seed: u64,
    count: u16,
    tick: u32,
    world: u64,
    isolated: bool,
    yaw: f32,
    camera: String,
    part: Option<(u32, u32, u64)>,
}
impl Saved {
    fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !(2..=128).contains(&self.count)
            || self.tick >= 360
            || !self.yaw.is_finite()
        {
            return Err("Invalid spatial request version, count, tick or pose.".into());
        }
        Ok(())
    }
}
impl Bench {
    pub(super) fn save_spatial(&mut self) {
        let result = (|| -> Result<std::path::PathBuf, String> {
            let m = self.model.borrow();
            if let Some(selection) = m.selected {
                if !self
                    .scene
                    .borrow_mut()
                    .section
                    .as_mut()
                    .is_some_and(|section| {
                        section.validate_selection(selection, m.world(), &m.volumes)
                    })
                {
                    return Err("Selected part is no longer available.".into());
                }
            }
            let s = &m.spatial;
            let saved = Saved {
                version: 1,
                form: s.form,
                glyph: s.glyph,
                seed: s.seed,
                count: s.count,
                tick: s.tick,
                world: mesocosm_core::state_hash(m.world()),
                isolated: m.isolated,
                yaw: m.yaw,
                camera: m.camera.name().into(),
                part: m.selected.map(|s| (s.organism.0, s.part.0, s.revision.0)),
            };
            saved.validate()?;
            let bytes = serde_json::to_vec_pretty(&saved).map_err(|e| e.to_string())?;
            let mut file = tempfile::Builder::new()
                .prefix("spatial-effect-")
                .suffix(".json")
                .tempfile_in(&self.export_directory)
                .map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut file, &bytes).map_err(|e| e.to_string())?;
            file.keep().map(|(_, path)| path).map_err(|e| e.to_string())
        })();
        match result {
            Ok(path) => {
                self.notice = format!("Saved {}", path.display());
                self.model.borrow_mut().spatial.saved = Some(path);
            },
            Err(e) => self.notice = e,
        }
    }
    pub(super) fn reopen_spatial(&mut self) {
        let result = (|| -> Result<Saved, String> {
            let m = self.model.borrow();
            let path = m
                .spatial
                .saved
                .as_ref()
                .ok_or("Save a spatial effect first.")?;
            let saved: Saved =
                serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            saved.validate()?;
            if let Some(selection) = m.selected {
                if !self
                    .scene
                    .borrow_mut()
                    .section
                    .as_mut()
                    .is_some_and(|section| {
                        section.validate_selection(selection, m.world(), &m.volumes)
                    })
                {
                    return Err("Selected part is no longer available.".into());
                }
            }
            if saved.world != mesocosm_core::state_hash(m.world())
                || saved.isolated != m.isolated
                || saved.yaw != m.yaw
                || saved.camera != m.camera.name()
                || saved.part != m.selected.map(|s| (s.organism.0, s.part.0, s.revision.0))
            {
                return Err("Restore the saved specimen, pose, camera and part selection before replaying these settings.".into());
            }
            Ok(saved)
        })();
        match result {
            Ok(s) => {
                let mut m = self.model.borrow_mut();
                let path = m.spatial.saved.clone();
                m.spatial = Spatial {
                    enabled: true,
                    form: s.form,
                    glyph: s.glyph,
                    seed: s.seed,
                    count: s.count,
                    tick: s.tick,
                    saved: path,
                    ..Spatial::default()
                };
                m.changed();
                self.notice = "Spatial settings and tick restored.".into();
            },
            Err(e) => self.notice = e,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> Saved {
        Saved {
            version: 1,
            form: Form::Surface,
            glyph: Glyph::Backticks,
            seed: u64::MAX,
            count: 18,
            tick: 37,
            world: 42,
            isolated: true,
            yaw: 0.5,
            camera: "oblique".into(),
            part: Some((12, 3, 987)),
        }
    }

    #[test]
    fn admitted_boundaries_roundtrip_complete_attachment_identity() {
        for count in [2, 128] {
            for tick in [0, 359] {
                for isolated in [false, true] {
                    let saved = Saved {
                        count,
                        tick,
                        isolated,
                        ..valid()
                    };
                    saved.validate().unwrap();
                    let encoded = serde_json::to_value(&saved).unwrap();
                    let restored: Saved = serde_json::from_value(encoded.clone()).unwrap();
                    restored.validate().unwrap();
                    assert_eq!(serde_json::to_value(&restored).unwrap(), encoded);
                    assert_eq!(restored.part, Some((12, 3, 987)));
                    assert_eq!(restored.isolated, isolated);
                    assert_eq!(restored.seed, u64::MAX);
                }
            }
        }
        let unselected = Saved {
            part: None,
            ..valid()
        };
        let restored: Saved =
            serde_json::from_str(&serde_json::to_string(&unselected).unwrap()).unwrap();
        assert!(restored.part.is_none());
    }

    #[test]
    fn editable_numbers_are_refused_outside_version_count_tick_and_finite_pose() {
        for invalid in [
            Saved {
                version: 0,
                ..valid()
            },
            Saved {
                version: 2,
                ..valid()
            },
            Saved {
                count: 0,
                ..valid()
            },
            Saved {
                count: 1,
                ..valid()
            },
            Saved {
                count: 129,
                ..valid()
            },
            Saved {
                count: u16::MAX,
                ..valid()
            },
            Saved {
                tick: 360,
                ..valid()
            },
            Saved {
                tick: u32::MAX,
                ..valid()
            },
            Saved {
                yaw: f32::NAN,
                ..valid()
            },
            Saved {
                yaw: f32::INFINITY,
                ..valid()
            },
            Saved {
                yaw: f32::NEG_INFINITY,
                ..valid()
            },
        ] {
            assert!(invalid.validate().is_err());
        }
        let encoded = serde_json::to_string(&valid()).unwrap();
        for invalid in ["NaN", "Infinity", "-Infinity", "null"] {
            let json = encoded.replace("\"yaw\":0.5", &format!("\"yaw\":{invalid}"));
            assert_ne!(json, encoded, "fixture must actually replace the pose");
            assert!(serde_json::from_str::<Saved>(&json).is_err());
        }
    }

    #[test]
    fn edited_schema_rejects_unknown_missing_and_malformed_identity_fields() {
        let baseline = serde_json::to_value(valid()).unwrap();
        let mut unknown = baseline.clone();
        unknown["invented_rule"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Saved>(unknown).is_err());
        for field in [
            "version", "isolated", "world", "yaw", "camera", "seed", "count", "tick",
        ] {
            let mut missing = baseline.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(
                serde_json::from_value::<Saved>(missing).is_err(),
                "missing {field}"
            );
        }
        for (field, value) in [
            ("part", serde_json::json!(3)),
            ("part", serde_json::json!([12, 3])),
            ("part", serde_json::json!([12, 3, 987, 1])),
            ("part", serde_json::json!([-1, 3, 987])),
            ("form", serde_json::json!("unknown_form")),
            ("glyph", serde_json::json!("unknown_glyph")),
            ("count", serde_json::json!(-1)),
            ("tick", serde_json::json!(0.5)),
        ] {
            let mut malformed = baseline.clone();
            malformed[field] = value;
            assert!(
                serde_json::from_value::<Saved>(malformed).is_err(),
                "malformed {field}"
            );
        }
    }
}
