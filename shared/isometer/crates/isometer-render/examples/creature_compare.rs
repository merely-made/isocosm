// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Generated anatomy JSON -> live articulated surface versus refreshed posed atlas.
//! Release-only renderer probe. Simulation admission is exported separately.
#[path = "token_compare/gpu.rs"]
mod gpu;
#[path = "creature_compare/hybrid.rs"]
pub mod hybrid;
#[path = "creature_compare/sprite.rs"]
mod sprite;
use glam::{Mat4, Vec3};
use gpu::{Gpu, Target, Timer};
use isometer_core::{BodyDocument, PartId};
use isometer_mesh::{BodyMesh, Volume, VolumeMap, live::LiveBodyProjector};
use isometer_render::{
    Camera, ClipSlab, LiveBody,
    implicit::{self, Surface},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};
const SIZE: u32 = 64;
const SAMPLES: u32 = 72;
const ROUNDS: usize = 3;

struct Recipe {
    document: BodyDocument,
    mesh: BodyMesh,
    roots: Vec<[f32; 3]>,
    feet: Vec<BTreeMap<PartId, [i64; 3]>>,
    ground: f32,
}
impl Recipe {
    fn load(data: &Value) -> Self {
        let document: BodyDocument = serde_json::from_value(data["document"].clone()).unwrap();
        let mut volumes = VolumeMap::new();
        for p in document.living() {
            volumes.insert(
                p.volume,
                Volume::solid(p.half_extent.map(|v| (v * 2) as u32), 7),
            );
        }
        let (mesh, _) = LiveBodyProjector::new()
            .project_body(&document, &volumes)
            .unwrap();
        let frames = data["frames"].as_array().unwrap();
        let unit = data["unit"].as_i64().unwrap() as f32;
        let base: [i64; 3] = serde_json::from_value(frames[0]["root"].clone()).unwrap();
        let roots = frames
            .iter()
            .map(|f| {
                let r: [i64; 3] = serde_json::from_value(f["root"].clone()).unwrap();
                [0, 1, 2].map(|k| (r[k] - base[k]) as f32 / unit)
            })
            .collect();
        let feet = frames
            .iter()
            .map(|f| serde_json::from_value(f["feet"].clone()).unwrap())
            .collect();
        Self {
            document,
            mesh,
            roots,
            feet,
            ground: data["ground_y"].as_f64().unwrap() as f32 - base[1] as f32 / unit,
        }
    }
    fn pose(&self, phase: usize, offset: Vec3) -> Surface {
        let origin = Vec3::from_array(self.roots[phase % 24]) + offset;
        let mut body = LiveBody::new(&self.mesh, origin.to_array());
        body.tint = [0.70, 0.85, 0.65];
        let mut gait = implicit::Gait::default();
        gait.set_physical_feet(Some(self.feet[phase % 24].clone()));
        implicit::walking_anatomy(
            &self.document,
            body,
            &mut |_| Some(self.ground + offset.y),
            &BTreeMap::new(),
            &mut gait,
            1.0 / 60.0,
        )
        .unwrap()
        .0
    }
}
fn camera(recipe: &Recipe) -> Camera {
    let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for i in 0..24 {
        let (a, b) = recipe.pose(i, Vec3::ZERO).bounds().unwrap();
        lo = lo.min(a.into());
        hi = hi.max(b.into());
    }
    Camera {
        target: ((lo + hi) * 0.5).to_array(),
        extent: (hi - lo).length() * 0.62,
        yaw: -std::f32::consts::FRAC_PI_4,
        pitch: std::f32::consts::PI / 6.0,
        aspect: 1.0,
    }
}
fn matrix(cam: Camera, w: u32, h: u32, zoom: f32) -> [[f32; 4]; 4] {
    (Mat4::from_scale(Vec3::new(
        SIZE as f32 * zoom / w as f32,
        SIZE as f32 * zoom / h as f32,
        1.0,
    )) * cam.view_proj())
    .to_cols_array_2d()
}
fn offset(cam: Camera, x: f32, y: f32, zoom: f32) -> Vec3 {
    let toward = Vec3::new(
        cam.yaw.cos() * cam.pitch.cos(),
        cam.pitch.sin(),
        cam.yaw.sin() * cam.pitch.cos(),
    );
    let right = Vec3::new(cam.yaw.sin(), 0.0, -cam.yaw.cos());
    (right * x - toward.cross(right) * y) * (2.0 * cam.extent / (SIZE as f32 * zoom))
}
fn layout(n: usize, w: u32, h: u32, atlas: bool) -> Vec<[f32; 2]> {
    let cols = if atlas {
        n.min(8)
    } else if n == 1 {
        1
    } else {
        (n as f32 * w as f32 / h as f32).sqrt().ceil() as usize
    };
    let rows = n.div_ceil(cols);
    let spacing = if atlas {
        64.0
    } else {
        (w as f32 / (cols + 1) as f32)
            .min(h as f32 / (rows + 1) as f32)
            .floor()
    };
    (0..n)
        .map(|i| {
            [
                ((i % cols) as f32 * spacing - (cols - 1) as f32 * spacing / 2.0).round(),
                ((i / cols) as f32 * spacing - (rows - 1) as f32 * spacing / 2.0).round(),
            ]
        })
        .collect()
}
fn frame(cam: Camera, w: u32, h: u32, zoom: f32) -> implicit::Frame {
    implicit::Frame {
        clip_from_world: matrix(cam, w, h, zoom),
        clip: ClipSlab::new([0.0, 1.0, 0.0], -1.0e6, 1.0e6),
        toward_sun: [-0.45, 0.8, 0.4],
        ambient: 0.4,
        sun: [0.7; 3],
    }
}
fn surfaces(
    recipe: &Recipe,
    cam: Camera,
    n: usize,
    phase: usize,
    w: u32,
    h: u32,
    zoom: f32,
    atlas: bool,
    private: bool,
) -> Vec<Surface> {
    layout(n, w, h, atlas)
        .iter()
        .enumerate()
        .map(|(i, p)| {
            recipe.pose(
                (phase + if private { i } else { 0 }) % 24,
                offset(cam, p[0], p[1], zoom),
            )
        })
        .collect()
}
struct Cache {
    target: Target,
    terrain: wgpu::Texture,
    renderer: implicit::Renderer,
    sprites: sprite::Sprites,
    slots: usize,
}
impl Cache {
    fn new(gpu: &Gpu, slots: usize) -> Self {
        let (w, h) = (slots.min(8) as u32 * SIZE, slots.div_ceil(8) as u32 * SIZE);
        let mut target = Target::new(gpu, w, h, 1);
        let terrain = Target::new(gpu, w, h, 1);
        let renderer = implicit::Renderer::new(&gpu.device, gpu::FORMAT, [w, h]);
        target.depth = renderer.depth_texture().clone();
        let sprites = sprite::Sprites::new(gpu, &target, renderer.depth_texture());
        let mut e = gpu.encoder();
        let v = terrain.views(0);
        gpu::clear(&mut e, &v.0, &v.1);
        gpu.queue.submit([e.finish()]);
        let terrain = terrain.depth;
        Self {
            target,
            terrain,
            renderer,
            sprites,
            slots,
        }
    }
    fn refresh(
        &self,
        gpu: &Gpu,
        e: &mut wgpu::CommandEncoder,
        r: &Recipe,
        cam: Camera,
        phase: usize,
        private: bool,
    ) -> u64 {
        let s = surfaces(
            r,
            cam,
            self.slots,
            phase,
            self.target.width,
            self.target.height,
            1.0,
            true,
            private,
        );
        self.renderer
            .encode(
                &gpu.device,
                &gpu.queue,
                e,
                &self.target.colour,
                &self.terrain,
                frame(cam, self.target.width, self.target.height, 1.0),
                &s.iter().collect::<Vec<_>>(),
            )
            .unwrap()
    }
    fn draw(
        &mut self,
        gpu: &Gpu,
        e: &mut wgpu::CommandEncoder,
        t: &Target,
        cam: Camera,
        n: usize,
        zoom: f32,
        private: bool,
    ) -> u64 {
        let m = Mat4::from_cols_array_2d(&matrix(cam, t.width, t.height, zoom));
        let v: Vec<_> = layout(n, t.width, t.height, false)
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let clip = m * offset(cam, p[0], p[1], zoom).extend(0.0);
                sprite::Instance {
                    rect: [
                        clip.x,
                        clip.y,
                        SIZE as f32 * zoom / t.width as f32,
                        SIZE as f32 * zoom / t.height as f32,
                    ],
                    layer_depth: [if private { i as f32 } else { 0.0 }, 0.0, 0.0, 0.0],
                }
            })
            .collect();
        let views = t.views(0);
        gpu::clear(e, &views.0, &views.1);
        self.sprites.draw(gpu, e, &views.0, &views.1, &v) as u64
    }
}
fn stats(mut v: Vec<f64>) -> Value {
    v.sort_by(f64::total_cmp);
    let at = |q: f64| v[((v.len() - 1) as f64 * q).round() as usize];
    json!({"samples":v.len(),"p50_us":at(0.5),"p95_us":at(0.95),"mean_us":v.iter().sum::<f64>()/v.len() as f64})
}
fn draw_live(
    gpu: &Gpu,
    e: &mut wgpu::CommandEncoder,
    renderer: &implicit::Renderer,
    t: &Target,
    terrain: &Target,
    r: &Recipe,
    cam: Camera,
    n: usize,
    phase: usize,
    zoom: f32,
    private: bool,
) -> u64 {
    let s = surfaces(r, cam, n, phase, t.width, t.height, zoom, false, private);
    renderer
        .encode(
            &gpu.device,
            &gpu.queue,
            e,
            &t.colour,
            &terrain.depth,
            frame(cam, t.width, t.height, zoom),
            &s.iter().collect::<Vec<_>>(),
        )
        .unwrap()
}
fn main() {
    let out = PathBuf::from(std::env::args().nth(1).expect("fresh output directory"));
    assert!(
        !out.join("comparison.json").exists(),
        "retain old receipts; use a fresh directory"
    );
    fs::create_dir_all(&out).unwrap();
    let source = PathBuf::from(std::env::args().nth(2).expect("exported motion JSON"));
    let data: Value = serde_json::from_slice(&fs::read(&source).unwrap()).unwrap();
    let recipe = Recipe::load(&data);
    let cam = camera(&recipe);
    let gpu = Gpu::new();
    let mut captures = Vec::new();
    let mut equality = Vec::new();
    let reverse = std::env::var_os("CREATURE_COMPARE_REVERSE").is_some();
    for size in if reverse { vec![] } else { vec![64, 256] } {
        let t = Target::new(&gpu, size, size, 1);
        let terrain = Target::new(&gpu, size, size, 1);
        let live = implicit::Renderer::new(&gpu.device, gpu::FORMAT, [size, size]);
        let mut cache = Cache::new(&gpu, 1);
        let views = terrain.views(0);
        let mut e = gpu.encoder();
        gpu::clear(&mut e, &views.0, &views.1);
        gpu.queue.submit([e.finish()]);
        for phase in 0..24 {
            let mut e = gpu.encoder();
            cache.refresh(&gpu, &mut e, &recipe, cam, phase, false);
            gpu.queue.submit([e.finish()]);
            let mut e = gpu.encoder();
            draw_live(
                &gpu,
                &mut e,
                &live,
                &t,
                &terrain,
                &recipe,
                cam,
                1,
                phase,
                size as f32 / 64.0,
                false,
            );
            gpu.queue.submit([e.finish()]);
            let a = gpu.read(&t.colour, 0);
            let name = format!("live-{size}-{phase}.png");
            fs::write(out.join(&name), a.to_png()).unwrap();
            captures.push(name);
            let mut e = gpu.encoder();
            cache.draw(&gpu, &mut e, &t, cam, 1, size as f32 / 64.0, false);
            gpu.queue.submit([e.finish()]);
            let b = gpu.read(&t.colour, 0);
            let name = format!("sprite-{size}-{phase}.png");
            fs::write(out.join(&name), b.to_png()).unwrap();
            captures.push(name);
            equality.push(json!({"size":size,"phase":phase,"different_pixels":a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)).filter(|(a,b)|a!=b).count(),"opaque_pixels":a.opaque_pixels()}));
        }
    }
    let t = Target::new(&gpu, 1024, 768, 1);
    let terrain = Target::new(&gpu, 1024, 768, 1);
    let live = implicit::Renderer::new(&gpu.device, gpu::FORMAT, [1024, 768]);
    let v = terrain.views(0);
    let mut e = gpu.encoder();
    gpu::clear(&mut e, &v.0, &v.1);
    gpu.queue.submit([e.finish()]);
    gpu.wait();
    if !reverse {
        let mut cache = Cache::new(&gpu, 16);
        let mut e = gpu.encoder();
        cache.refresh(&gpu, &mut e, &recipe, cam, 12, true);
        gpu.queue.submit([e.finish()]);
        let mut e = gpu.encoder();
        draw_live(
            &gpu, &mut e, &live, &t, &terrain, &recipe, cam, 16, 12, 1.0, true,
        );
        gpu.queue.submit([e.finish()]);
        let a = gpu.read(&t.colour, 0);
        fs::write(out.join("board-live.png"), a.to_png()).unwrap();
        let mut e = gpu.encoder();
        cache.draw(&gpu, &mut e, &t, cam, 16, 1.0, true);
        gpu.queue.submit([e.finish()]);
        let b = gpu.read(&t.colour, 0);
        fs::write(out.join("board-sprite.png"), b.to_png()).unwrap();
        let different = a
            .rgba
            .chunks_exact(4)
            .zip(b.rgba.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        let silhouette = a
            .rgba
            .chunks_exact(4)
            .zip(b.rgba.chunks_exact(4))
            .filter(|(a, b)| (a[3] > 0) != (b[3] > 0))
            .count();
        equality.push(json!({"size":1024,"count":16,"phase":12,"different_pixels":different,"silhouette_difference_pixels":silhouette}));
    }
    let mut cases = Vec::new();
    for n in if reverse { [64, 16, 1] } else { [1, 16, 64] } {
        for private in if reverse {
            [true, false]
        } else {
            [false, true]
        } {
            if n == 1 && private {
                continue;
            }
            let init = Instant::now();
            let mut cache = Cache::new(&gpu, if private { n } else { 1 });
            let mut e = gpu.encoder();
            cache.refresh(&gpu, &mut e, &recipe, cam, 0, private);
            gpu.queue.submit([e.finish()]);
            gpu.wait();
            let initial_us = init.elapsed().as_secs_f64() * 1e6;
            for cadence in if reverse { [4, 1, 0] } else { [0, 1, 4] } {
                let (
                    mut cpu,
                    mut times,
                    mut refresh_cpu,
                    mut held_cpu,
                    mut refresh_gpu,
                    mut held_gpu,
                    mut uploads,
                ) = (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                );
                for _round in 0..ROUNDS {
                    for j in 0..24 {
                        let mut e = gpu.encoder();
                        if cadence == 0 {
                            draw_live(
                                &gpu, &mut e, &live, &t, &terrain, &recipe, cam, n, j, 1.0, private,
                            );
                        } else {
                            if j % cadence == 0 {
                                cache.refresh(&gpu, &mut e, &recipe, cam, j, private);
                            }
                            cache.draw(&gpu, &mut e, &t, cam, n, 1.0, private);
                        }
                        gpu.queue.submit([e.finish()]);
                    }
                    gpu.wait();
                    let timer = Timer::new(&gpu, SAMPLES);
                    let mut flags = Vec::new();
                    for j in 0..SAMPLES {
                        let start = Instant::now();
                        let mut e = gpu.encoder();
                        e.write_timestamp(&timer.queries, j * 2);
                        let refresh = cadence > 0 && j as usize % cadence == 0;
                        let phase = j as usize % 24;
                        let upload = if cadence == 0 {
                            draw_live(
                                &gpu, &mut e, &live, &t, &terrain, &recipe, cam, n, phase, 1.0,
                                private,
                            )
                        } else {
                            let b = if refresh {
                                cache.refresh(&gpu, &mut e, &recipe, cam, phase, private)
                            } else {
                                0
                            };
                            b + cache.draw(&gpu, &mut e, &t, cam, n, 1.0, private)
                        };
                        e.write_timestamp(&timer.queries, j * 2 + 1);
                        gpu.queue.submit([e.finish()]);
                        let us = start.elapsed().as_secs_f64() * 1e6;
                        cpu.push(us);
                        uploads.push(upload);
                        flags.push(refresh);
                        if refresh {
                            refresh_cpu.push(us)
                        } else {
                            held_cpu.push(us)
                        }
                    }
                    let g = timer.finish(&gpu);
                    for (us, refresh) in g.iter().zip(flags) {
                        if refresh {
                            refresh_gpu.push(*us)
                        } else {
                            held_gpu.push(*us)
                        }
                    }
                    times.extend(g);
                }
                let result = json!({"count":n,"private":private,"cadence":cadence,"cpu":stats(cpu),"gpu":stats(times),
                "refresh_cpu":if refresh_cpu.is_empty(){Value::Null}else{stats(refresh_cpu)},
                "held_cpu":if held_cpu.is_empty(){Value::Null}else{stats(held_cpu)},
                "refresh_gpu":if refresh_gpu.is_empty(){Value::Null}else{stats(refresh_gpu)},
                "held_gpu":if held_gpu.is_empty(){Value::Null}else{stats(held_gpu)},
                "cache_bytes":cache.slots*64*64*8,"initial_cache_wall_us":initial_us,
                "mean_upload_bytes":uploads.iter().sum::<u64>() as f64/uploads.len() as f64});
                println!(
                    "n={n} private={private} cadence={cadence} CPU={} GPU={}",
                    result["cpu"]["p50_us"], result["gpu"]["p50_us"]
                );
                cases.push(result);
            }
        }
    }
    let moving_feet = recipe.feet[0]
        .keys()
        .filter(|id| {
            recipe
                .feet
                .iter()
                .any(|f| f.get(id) != recipe.feet[0].get(id))
        })
        .count();
    assert!(moving_feet > 0, "limbs must move relative to the root");
    let report = json!({"adapter":format!("{:?}",gpu.info),"source":data,"cases":cases,"equality":equality,
        "reversed_case_order":reverse,"moving_feet":moving_feet,
        "cache_renderer_buffer_bytes":8192*96+128*64+224,"sprite_instance_buffer_bytes":1024*32,
        "cache_probe_texture_bytes_per_slot":64*64*12,
        "captures":captures,"primitives":recipe.pose(0,Vec3::ZERO).parts.len(),"primitive_bytes":std::mem::size_of::<implicit::Primitive>(),
        "method":"3 rounds x 72 frames, 24 warm frames per round; CPU includes anatomy/IK, allocations, uploads, encode and submit; GPU includes cache re-render/write and final draw. 1024x768, 64px cache, fixed camera/light; simulation planning/readback/PNG excluded; native stride sampled before benchmark. Sequential case order."});
    fs::write(
        out.join("comparison.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
