// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
pub(super) struct OuterTerrain {
    pub(super) tracer: BrickTracer,
    pub(super) map: Option<BrickMap>,
    pub(super) query: query_terrain::TerrainQueryCache,
    pub(super) cell: f32,
    pub(super) pending: bool,
}
impl Scene {
    pub(super) fn outer_map(&self) -> Option<(&BrickMap, f32)> {
        let outer = self.outer.as_ref().filter(|_| self.outer_drawn)?;
        Some((outer.map.as_ref()?, outer.cell))
    }
    pub(super) fn draw_outer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: &dyn TerrainSource,
        frame: &SceneFrame<'_>,
    ) -> Result<(), String> {
        let band = self.outer.get_or_insert_with(|| OuterTerrain {
            tracer: BrickTracer::with_format(
                self.device.clone(),
                self.queue.clone(),
                self.width,
                self.height,
                FRAME_FORMAT,
            )
            .hits_only(),
            map: None,
            query: Default::default(),
            cell: 1.,
            pending: true,
        });
        band.cell = source.cell_size();
        let camera = coordinates::camera(frame.camera, band.cell);
        let refresh = if let Some(map) = &mut band.map {
            source.refresh(map, &[])?
        } else {
            band.map = Some(source.brick_map()?);
            TerrainRefresh::Full
        };
        let change = match &refresh {
            _ if band.pending => BrickChange::Full,
            TerrainRefresh::Full => BrickChange::Full,
            TerrainRefresh::Current => BrickChange::Slots(&[]),
            TerrainRefresh::Slots(s) => BrickChange::Slots(s),
        };
        let mut input = BrickFrameInput::for_camera(
            band.map.as_ref().expect("prepared band"),
            BrickRevision(source.revision()),
            camera.trace().ok_or("invalid outer terrain camera")?,
            &frame.grade,
        )
        .changed(change)
        .with_clip_from_world(camera.clip_from_world());
        input.terrain_palette = self.terrain_palette.as_ref();
        input.terrain_appearance = coordinates::appearance(frame.terrain_appearance, band.cell);
        input.light = self.light;
        let outer = band
            .tracer
            .encode_with_depth(encoder, &self.traced_view, &self.bodies.depth_view, input)
            .map_err(|e| e.to_string())?;
        // The outer terrain has neither a body roster nor a leased atlas.
        // Add its real uploads and draw to the one presented frame receipt.
        if let Some(total) = &mut self.terrain_diagnostics {
            total.cpu_prepare_us += outer.cpu_prepare_us;
            total.brick_upload_bytes += outer.brick_upload_bytes;
            total.pointer_write_calls += outer.pointer_write_calls;
            total.atlas_write_calls += outer.atlas_write_calls;
            total.uniform_write_calls += outer.uniform_write_calls;
            total.uniform_upload_bytes += outer.uniform_upload_bytes;
            total.changed_slots_declared += outer.changed_slots_declared;
            total.resource_creations += outer.resource_creations;
            total.bind_group_rebuilds += outer.bind_group_rebuilds;
            total.trace_passes += outer.trace_passes;
            total.map_revision_unchanged &= outer.map_revision_unchanged;
            total.full_map_upload |= outer.full_map_upload;
            total.map_recreated |= outer.map_recreated;
            total.projection_replaced |= outer.projection_replaced;
        }
        band.pending = false;
        Ok(())
    }
}
