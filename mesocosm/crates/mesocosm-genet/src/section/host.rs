// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{BodyFrameStats, BodyMode, CritterPose, SiteScene, bodies, tenant};

/// The subjects Mesocosm examines besides the played critter (Mark,
/// 2026-11-08): the followed camera target and the dev inspection focus, its
/// selected part's body included. Cosmograph's examined region joins once
/// M4.8's host side exists.
pub(super) fn examined_subjects(
    followed: Option<u64>,
    focus: Option<u64>,
    selected: Option<u64>,
) -> Vec<u64> {
    let mut subjects: Vec<u64> = [followed, focus, selected].into_iter().flatten().collect();
    subjects.sort_unstable();
    subjects.dedup();
    subjects
}

/// Any one of the played critter or an examined subject keeps a body live.
fn examined(played: Option<u64>, examined: &[u64], subject: u64) -> bool {
    played == Some(subject) || examined.contains(&subject)
}

/// The host policy one frame calls back into: Mesocosm's capsule stand-in for
/// a body that would not project, and the counters only the scene can supply.
pub(super) struct SectionHost<'a> {
    pub(super) bodies: &'a mut bodies::HostBodies,
    pub(super) scene: &'a SiteScene,
    pub(super) tenant: &'a mut tenant::BodyTenant,
    pub(super) mode: BodyMode,
}

impl isometer::SceneHost for SectionHost<'_> {
    fn uses_body_tenant(&self) -> bool {
        self.mode != BodyMode::Hybrid
    }

    fn hybrid_settings(&self) -> Option<isometer::hybrid::HybridSettings> {
        self.mode
            .hybrid_settings()
            .map(|settings| settings.with_light_step(self.bodies.light_step))
    }

    /// Natively posed bodies hold live detail; cohort stand-ins and other
    /// bodies without a native pose hold only a sprite (case 1171). Until
    /// M4.8, the played body in a stand-in view therefore stays a sprite.
    fn hybrid_held(&self, subject: isometer::SubjectKey) -> isometer::hybrid::BodyDetail {
        match self.scene.body(subject.0) {
            Some(body) if body.physical.is_none() => isometer::hybrid::BodyDetail::Sprite,
            _ => isometer::hybrid::BodyDetail::Live,
        }
    }

    /// The tenant's sun, for the shared pass hybrid draws with (case 1170).
    fn body_light(&self) -> Option<isometer::BodyLight> {
        Some(self.tenant.block().body(self.tenant.light()))
    }

    fn hybrid_examined(&self, subject: isometer::SubjectKey) -> bool {
        examined(self.scene.played, &self.bodies.examined, subject.0)
    }

    fn encode_body_tenant(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        bodies: &mut isometer::BodyLayer,
        camera: isometer::SlabCamera,
    ) -> Result<(), String> {
        if self.uses_body_tenant() {
            self.tenant.encode(
                encoder,
                bodies,
                camera,
                (self.mode == BodyMode::Smooth).then_some(self.scene),
            )?;
        }
        Ok(())
    }

    fn begin(&mut self) {
        self.bodies.fallback.clear();
        self.bodies.played_fallback = None;
    }

    fn fallback(&mut self, body: &isometer::SceneBody<'_>, stats: &mut BodyFrameStats) {
        self.bodies.add_fallback(body, stats);
    }

    fn prepared(&mut self, layer: &mut isometer::BodyLayer) {
        layer.stats.body_scale = self.bodies.scale;
        let scene = self.scene;
        layer.stats.carcasses = layer
            .drawn()
            .filter(|subject| scene.body(subject.0).is_some_and(|b| !b.alive))
            .count();
    }

    fn roster(&self) -> &[CritterPose] {
        &self.bodies.fallback
    }

    fn played(&self) -> Option<&CritterPose> {
        self.bodies.played_fallback.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn played_followed_and_dev_focus_each_keep_a_subject_examined() {
        let set = examined_subjects(Some(4), Some(4), Some(9));
        assert_eq!(set, [4, 9]);
        assert!(examined(Some(1), &set, 1), "played");
        assert!(examined(Some(1), &set, 4), "followed or dev focus");
        assert!(examined(Some(1), &set, 9), "selected part's body");
        assert!(!examined(Some(1), &set, 2), "an ordinary subject");
        assert!(!examined(None, &[], 1), "nothing examined");
        // Followed without dev: no focus or selection, still examined.
        assert!(examined(
            Some(1),
            &examined_subjects(Some(7), None, None),
            7
        ));
    }
}
