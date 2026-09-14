// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Headless device, pixel readback and the region/coverage queries the
//! receipts assert with. No byte-exact frame is ever compared.

use std::path::PathBuf;

use isometer::{FrameRequest, SlabCamera};

use super::{Appearance, CameraPolicy, SceneHandle, SceneProducer};

/// Square and a multiple of 64 pixels wide, so a readback row is already
/// 256-byte aligned.
pub const SIZE: [u32; 2] = [256, 256];

/// Deliberately extreme tints: identity is read back out of the pixels, and a
/// face shade only scales a colour, so a dominant channel survives every face.
///
/// Green and blue, never red: the retro terrain grade puts dark reds on screen
/// (`(64, 0, 0)` among them), and a red body would be indistinguishable from
/// rock. Found by a keeper that read as 4 396 body pixels in a frame that drew
/// 356 of them.
pub const PLAYED: [f32; 3] = [0.02, 1.0, 0.02];
pub const OTHER: [f32; 3] = [0.02, 0.02, 1.0];

pub fn appearance() -> Appearance {
    Appearance {
        played: PLAYED,
        other: OTHER,
        ..Appearance::default()
    }
}

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

/// `None` with a printed reason when this machine has no adapter, so the suite
/// skips cleanly rather than failing for the absence of hardware.
pub fn gpu(what: &str) -> Option<Gpu> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping {what}");
        return None;
    };
    let Ok((device, queue)) = pollster::block_on(adapter.request_device(&Default::default()))
    else {
        eprintln!("no device; skipping {what}");
        return None;
    };
    Some(Gpu { device, queue })
}

/// Which body a pixel belongs to. Anything else — terrain, the cleared
/// background — is [`Ink::Scene`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Played,
    Other,
    Scene,
}

pub struct Frame {
    pub pixels: Vec<u8>,
    pub size: [u32; 2],
}

impl Frame {
    pub fn ink(&self, x: u32, y: u32) -> Ink {
        let index = ((y * self.size[0] + x) * 4) as usize;
        let [r, g, b] = [
            self.pixels[index] as u32,
            self.pixels[index + 1] as u32,
            self.pixels[index + 2] as u32,
        ];
        if g >= 100 && g > 3 * r && g > 3 * b {
            Ink::Played
        } else if b >= 100 && b > 3 * r && b > 3 * g {
            Ink::Other
        } else {
            Ink::Scene
        }
    }

    pub fn count(&self, ink: Ink) -> usize {
        self.region_count(ink, [0, 0, self.size[0], self.size[1]])
    }

    /// Pixels of `ink` inside `[x0, y0, x1, y1]`, clamped to the frame.
    pub fn region_count(&self, ink: Ink, region: [u32; 4]) -> usize {
        let mut count = 0;
        for y in region[1]..region[3].min(self.size[1]) {
            for x in region[0]..region[2].min(self.size[0]) {
                count += usize::from(self.ink(x, y) == ink);
            }
        }
        count
    }

    pub fn centroid(&self, ink: Ink) -> Option<[f64; 2]> {
        let (mut sx, mut sy, mut n) = (0.0, 0.0, 0u64);
        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                if self.ink(x, y) == ink {
                    sx += x as f64;
                    sy += y as f64;
                    n += 1;
                }
            }
        }
        (n > 0).then(|| [sx / n as f64, sy / n as f64])
    }

    pub fn write_png(&self, name: &str) -> PathBuf {
        let dir = std::env::var_os("PAREDROS_P1_CAPTURES")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(r"C:\Users\mark_\Code\.tmp\paredros-genet-host-20260913")
            });
        std::fs::create_dir_all(&dir).expect("capture directory");
        let path = dir.join(format!("p1-{name}.png"));
        let file = std::fs::File::create(&path).expect("capture file");
        let mut encoder = png::Encoder::new(file, self.size[0], self.size[1]);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut writer| writer.write_image_data(&self.pixels))
            .expect("capture written");
        eprintln!("capture: {}", path.display());
        path
    }
}

/// One producer invocation at the receipts' own size.
pub fn request<'a>(gpu: &'a Gpu, needs_frame: bool) -> FrameRequest<'a> {
    FrameRequest {
        device: &gpu.device,
        queue: &gpu.queue,
        size: SIZE,
        aspect: SIZE[0] as f32 / SIZE[1] as f32,
        color: None,
        needs_frame,
    }
}

/// Renders one frame and reads it back. Panics when the producer skipped, so
/// a caller that expects pixels says so.
pub fn frame(gpu: &Gpu, producer: &mut SceneProducer, needs_frame: bool) -> Frame {
    producer
        .render_scene(&request(gpu, needs_frame))
        .expect("scene renders")
        .expect("scene produced a frame");
    let (width, height, pixels) = producer.capture().expect("a completed frame to read back");
    Frame {
        pixels,
        size: [width, height],
    }
}

/// Aims the scene, leaving everything else the policy's default.
pub fn aim(handle: &SceneHandle, forward: [f32; 3], half_height: f32, terrain: bool) {
    let mut model = handle.borrow_mut();
    model.camera = CameraPolicy {
        forward,
        half_height,
        ..CameraPolicy::default()
    };
    model.appearance = appearance();
    model.terrain = terrain;
}

/// Candidate directions the receipts search over: twenty-four yaws at two
/// pitches. Enough to find a framing without assuming what seed 7 grew.
pub fn candidates() -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for step in 0..24 {
        let yaw = step as f32 / 24.0 * std::f32::consts::TAU;
        for pitch in [0.0f32, -0.35] {
            let level = pitch.cos();
            out.push([yaw.cos() * level, pitch.sin(), yaw.sin() * level]);
        }
    }
    out
}

/// The screen box a world AABB projects into, clamped to the frame.
pub fn screen_box(
    camera: SlabCamera,
    bounds: ([f32; 3], [f32; 3]),
    size: [u32; 2],
) -> Option<[u32; 4]> {
    let (min, max) = bounds;
    let mut low = [f32::MAX; 2];
    let mut high = [f32::MIN; 2];
    for mask in 0..8 {
        let corner = [0, 1, 2].map(|axis| {
            if mask & (1 << axis) == 0 {
                min[axis]
            } else {
                max[axis]
            }
        });
        let ndc = camera.ndc_of(corner)?;
        let pixel = [
            (ndc[0] * 0.5 + 0.5) * size[0] as f32,
            (0.5 - ndc[1] * 0.5) * size[1] as f32,
        ];
        for axis in 0..2 {
            low[axis] = low[axis].min(pixel[axis]);
            high[axis] = high[axis].max(pixel[axis]);
        }
    }
    let clamp = |v: f32, limit: u32| v.max(0.0).min(limit as f32) as u32;
    let region = [
        clamp(low[0].floor(), size[0]),
        clamp(low[1].floor(), size[1]),
        clamp(high[0].ceil(), size[0]),
        clamp(high[1].ceil(), size[1]),
    ];
    (region[0] < region[2] && region[1] < region[3]).then_some(region)
}

pub fn overlap(a: [u32; 4], b: [u32; 4]) -> Option<[u32; 4]> {
    let region = [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ];
    (region[0] < region[2] && region[1] < region[3]).then_some(region)
}
