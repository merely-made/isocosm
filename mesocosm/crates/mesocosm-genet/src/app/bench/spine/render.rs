// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Landscape;
use isometer::{
    DeclaredExtentVolumes, GroundTerrain, Scene, SceneFrame, SceneHost, SceneVolumes, SlabCamera,
    TerrainSource,
};

pub struct Renderer {
    scene: Scene,
    revision: u64,
    size: (u32, u32),
    pub camera: Option<SlabCamera>,
}
struct Host;
impl SceneHost for Host {}

impl Renderer {
    pub fn diagnostics(&self) -> Option<isometer::lens::BrickDiagnostics> {
        self.scene.terrain_diagnostics()
    }
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
    ) -> Result<Self, String> {
        let mut scene = Scene::new(device.clone(), queue.clone(), size.0, size.1)?;
        scene.set_terrain_palette(Some(isometer::lens::TerrainPalette::new([
            [0.4, 0.0, 0.4],    // unknown, never used for air
            [0.12, 0.42, 0.68], // opaque water
            [0.40, 0.29, 0.14], // soil
            [0.34, 0.37, 0.40], // rock
        ])));
        Ok(Self {
            scene,
            revision: u64::MAX,
            size,
            camera: None,
        })
    }
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        revision: u64,
        land: &Landscape,
    ) -> Result<wgpu::TextureView, String> {
        if self.size != size {
            self.scene.resize(size.0, size.1);
            self.size = size;
        }
        let terrain = GroundTerrain(&land.ground);
        if self.revision != revision {
            self.scene.set_terrain_map(terrain.brick_map()?);
            self.revision = revision;
        }
        let extent = land.window.extent as f32;
        let height = land.window.height as f32;
        let aspect = size.0 as f32 / size.1 as f32;
        let half = (extent * 1.6 / aspect).max(extent * 0.8 + height * 0.5) * 1.1;
        let camera = SlabCamera::dimetric_2_1(
            [0.0, height * 0.5, 0.0],
            half,
            aspect,
            (extent * 4.0 + height * 2.0).max(64.0),
        )
        .ok_or("terrain camera is invalid")?;
        let volumes = DeclaredExtentVolumes::from_documents(std::iter::empty(), 1);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Isocosm terrain bench"),
            ..Default::default()
        });
        self.scene.render(
            &mut encoder,
            SceneFrame {
                camera,
                bodies: &[],
                volumes: SceneVolumes::DeclaredSolid(&volumes),
                terrain: Some(&terrain),
                dirty: &[],
                grade: isometer::lens::Grade {
                    fog_start: 1.0,
                    ..isometer::lens::Grade::clay()
                },
                terrain_appearance: None,
                body_budget: 0,
                capsules: None,
            },
            &mut Host,
        )?;
        queue.submit([encoder.finish()]);
        self.camera = Some(camera);
        Ok(self.scene.encoded_view().clone())
    }
}
