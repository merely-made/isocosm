// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

impl SectionProducer {
    pub(super) fn accepted(&self) -> Option<(&Accepted, Rc<super::super::Snapshot>)> {
        let source = self.inner.as_ref()?.source();
        let frame = source.accepted.as_ref()?;
        if !self.handle.is_lifetime(frame.lifetime) {
            return None;
        }
        let input = if source.transactional {
            frame.input.clone()
        } else {
            let current = self.handle.snapshot()?;
            if current.fingerprint != frame.input.fingerprint {
                return None;
            }
            current
        };
        Some((frame, input))
    }
    pub(crate) fn body_stats(&mut self) -> Option<section::BodyFrameStats> {
        Some(self.accepted()?.0.body.clone())
    }
    pub(crate) fn terrain_stats(&mut self) -> Option<isometer::ResidencyStats> {
        Some(self.accepted()?.0.terrain)
    }
    pub(crate) fn select_part(
        &mut self,
        subject: u64,
        current: Option<section::BodySelection>,
        backwards: bool,
    ) -> Option<section::BodySelection> {
        let (frame, input) = self.accepted()?;
        input.scene.body(subject)?;
        frame
            .queries
            .as_ref()?
            .select_part(
                isometer::SubjectKey(subject),
                current.map(section::BodySelection::address),
                backwards,
            )
            .map(section::BodySelection::from_address)
    }
    pub(crate) fn validate_selection(&mut self, selected: section::BodySelection) -> bool {
        self.accepted().is_some_and(|(frame, input)| {
            input.scene.body(selected.organism).is_some()
                && frame
                    .queries
                    .as_ref()
                    .is_some_and(|q| q.validates_address(selected.address()))
        })
    }
    pub(super) fn has_current_frame(&self) -> bool {
        self.accepted().is_some()
    }
    pub(crate) fn pick(
        &mut self,
        local: (f64, f64),
        size: (f64, f64),
    ) -> Result<(Option<isometer::Pick>, u64), String> {
        self.pick_picture(local, size)
            .map(|(pick, revision, _site)| (pick, revision))
    }
    /// Geometry, action revision and site all come from one accepted picture.
    pub(crate) fn pick_picture(
        &mut self,
        local: (f64, f64),
        size: (f64, f64),
    ) -> Result<(Option<isometer::Pick>, u64, u64), String> {
        let (frame, input) = self
            .accepted()
            .ok_or("the pointed picture changed before presentation")?;
        let revision = input.revision.ok_or("viewport has no world revision")?;
        if ![local.0, local.1, size.0, size.1]
            .iter()
            .all(|v| v.is_finite())
            || size.0 <= 0.0
            || size.1 <= 0.0
            || local.0 < 0.0
            || local.1 < 0.0
            || local.0 >= size.0
            || local.1 >= size.1
        {
            return Err("pointer is outside the viewport".into());
        }
        let queries = frame
            .queries
            .as_ref()
            .ok_or("viewport has no exact query frame")?;
        let target = queries.extent();
        let pixel = [
            (local.0 / size.0 * target[0] as f64).floor() as u32,
            (local.1 / size.1 * target[1] as f64).floor() as u32,
        ];
        let pick = queries
            .pick_at_pixel(pixel)
            .map_err(|why| format!("{why:?}"))?;
        if let Some(isometer::Pick::Body(body)) = pick {
            if input.scene.body(body.address.subject.0).is_none() || !queries.validate_pick(body) {
                return Err("pointed body geometry is stale".into());
            }
        }
        Ok((pick, revision, input.site()))
    }
}
