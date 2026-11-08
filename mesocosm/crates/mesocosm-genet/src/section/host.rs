// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{BodyFrameStats, BodyMode, CritterPose, SiteScene, bodies, tenant};

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
        self.mode.hybrid_settings()
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
