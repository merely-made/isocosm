//! The probe's GPU arm: the scene's own picture of the headed board.

use super::*;

/// The scene's own picture of the board at `headroom`, drawn on the GPU.
pub(super) fn render(ui: &UiState, camera: SlabCamera, headroom: u32) -> Option<Vec<u8>> {
    let (device, queue) = device()?;
    let mut scene = Scene::new(device.clone(), queue.clone(), TEXTURE[0], TEXTURE[1]).ok()?;
    scene.set_terrain_palette(Some(terrain_palette(&ui.map)));
    let settings = ResidencySettings { headroom };
    let limits = scene.atlas_limits();
    let ground = BoardGround::new(&ui.map, &Overlays::of(ui), 1, settings, limits);
    let bricks = ground.bricks();
    let framed = ground.framed(camera, &bricks);
    let terrain = ground.terrain(&bricks, &framed);
    let volumes = VolumeMap::new();
    let mut encoder = device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera,
                bodies: &[],
                volumes: SceneVolumes::Voxels(&volumes),
                terrain: Some(&terrain as &dyn TerrainSource),
                dirty: &[],
                grade: board_grade(),
                terrain_appearance: Some(board_appearance()),
                body_budget: 16,
                capsules: None,
            },
            &mut PlainHost,
        )
        .ok()?;
    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    scene.capture(|_, _, _| {}).map(|(_, _, pixels)| pixels)
}

pub(super) fn pixel(image: &[u8], px: u32, py: u32) -> [u8; 4] {
    let at = ((py * TEXTURE[0] + px) * 4) as usize;
    [image[at], image[at + 1], image[at + 2], image[at + 3]]
}
