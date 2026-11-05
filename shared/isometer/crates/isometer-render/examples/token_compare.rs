// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Same recipe, cached live meshes versus experimental colour+depth sprites.
//! Run in release. Measures this renderer probe, not the full application.
#[path = "token_compare/gpu.rs"]
mod gpu;
#[path = "token_compare/sprite.rs"]
mod sprite;

use glam::{Mat4, Vec3, Vec4};
use gpu::{Gpu, Target, Timer};
use isometer_mesh::{
    BodyMesh,
    bake::{Appearance, Sheet, demo},
    live::LiveBodyProjector,
    token::{TokenBody, material_colours},
};
use isometer_render::{Camera, LiveBody, LiveBodyRenderer, MaterialPalette};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::Instant};

const NATIVE: u32 = 64;
const SAMPLES: u32 = 96;
const ROUNDS: usize = 3;
const WARM: u32 = 24;

fn camera() -> Camera {
    Camera {
        target: [0.0; 3],
        extent: 18.0,
        yaw: -std::f32::consts::FRAC_PI_4,
        pitch: std::f32::consts::PI / 6.0,
        aspect: 1.0,
    }
}

struct Inputs<'a> {
    bodies: Vec<LiveBody<'a>>,
    sprites: Vec<sprite::Instance>,
    matrix: [[f32; 4]; 4],
}

fn inputs<'a>(
    mesh: &'a BodyMesh,
    colours: &'a [[u8; 3]],
    count: usize,
    frame: u32,
    moving: bool,
    zoom: f32,
    width: u32,
    height: u32,
    facing: usize,
) -> Inputs<'a> {
    let cam = camera();
    let base = cam.view_proj();
    let matrix = Mat4::from_scale(Vec3::new(
        NATIVE as f32 * zoom / width as f32,
        NATIVE as f32 * zoom / height as f32,
        1.0,
    )) * base;
    let toward = Vec3::new(
        cam.yaw.cos() * cam.pitch.cos(),
        cam.pitch.sin(),
        cam.yaw.sin() * cam.pitch.cos(),
    );
    let right = Vec3::new(cam.yaw.sin(), 0.0, -cam.yaw.cos());
    let up = toward.cross(right);
    let columns = if count == 1 {
        1
    } else {
        (count as f32 * width as f32 / height as f32).sqrt().ceil() as usize
    };
    let rows = count.div_ceil(columns);
    let spacing = (width as f32 / (columns + 1) as f32)
        .min(height as f32 / (rows + 1) as f32)
        .floor();
    let units_per_pixel = 2.0 * cam.extent / (NATIVE as f32 * zoom);
    let base_depth = (base * Vec4::W).z;
    let mut bodies = Vec::with_capacity(count);
    let mut sprites = Vec::with_capacity(count);
    for i in 0..count {
        let (column, row) = (i % columns, i / columns);
        let dx = ((column as f32 - (columns - 1) as f32 / 2.0) * spacing).round()
            + if moving {
                (frame % 9) as f32 - 4.0
            } else {
                0.0
            };
        let dy = ((row as f32 - (rows - 1) as f32 / 2.0) * spacing).round();
        let origin = right * (dx * units_per_pixel) - up * (dy * units_per_pixel)
            + toward * (row as f32 * 0.5);
        let layer = (i + facing + if moving { (frame / 16) as usize } else { 0 }) % 4;
        let mut body = LiveBody::new(mesh, origin.to_array());
        body.yaw_radians = layer as f32 * std::f32::consts::FRAC_PI_2;
        body.palette = Some(MaterialPalette::new(colours));
        bodies.push(body);
        let clip = matrix * origin.extend(1.0);
        sprites.push(sprite::Instance {
            rect: [
                clip.x,
                clip.y,
                NATIVE as f32 * zoom / width as f32,
                NATIVE as f32 * zoom / height as f32,
            ],
            layer_depth: [layer as f32, clip.z - base_depth, 0.0, 0.0],
        });
    }
    Inputs {
        bodies,
        sprites,
        matrix: matrix.to_cols_array_2d(),
    }
}

fn draw(
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    views: &(wgpu::TextureView, wgpu::TextureView),
    live: &mut LiveBodyRenderer,
    cached: &mut sprite::Sprites,
    input: &Inputs<'_>,
    sprite: bool,
) -> Value {
    gpu::clear(encoder, &views.0, &views.1);
    if sprite {
        let upload = cached.draw(gpu, encoder, &views.0, &views.1, &input.sprites);
        json!({"instance_upload_bytes":upload,"static_upload_bytes":0,"draws":1,"vertices_per_instance":6})
    } else {
        let stats = live
            .draw(
                &gpu.device,
                &gpu.queue,
                encoder,
                &views.0,
                &views.1,
                input.matrix,
                None,
                &input.bodies,
            )
            .expect("live body draw");
        json!({"instance_upload_bytes":stats.instance_upload_bytes,"static_upload_bytes":stats.mesh_upload_bytes,
            "palette_upload_bytes":stats.palette_upload_bytes,"frame_upload_bytes":stats.frame_upload_bytes,
            "draws":stats.draws,"cached_meshes":stats.cached_meshes})
    }
}

fn summary(mut samples: Vec<f64>) -> Value {
    samples.sort_by(f64::total_cmp);
    let at = |q: f64| samples[((samples.len() - 1) as f64 * q).round() as usize];
    json!({"samples":samples.len(),"p50_us":at(0.5),"p95_us":at(0.95),"min_us":samples[0],"max_us":samples[samples.len()-1]})
}

fn capture(
    gpu: &Gpu,
    target: &Target,
    live: &mut LiveBodyRenderer,
    cached: &mut sprite::Sprites,
    input: &Inputs<'_>,
    sprite: bool,
) -> Sheet {
    let views = target.views(0);
    let mut encoder = gpu.encoder();
    draw(gpu, &mut encoder, &views, live, cached, input, sprite);
    gpu.queue.submit([encoder.finish()]);
    gpu.read(&target.colour, 0)
}

fn differences(a: &Sheet, b: &Sheet) -> Value {
    assert_eq!((a.w, a.h), (b.w, b.h));
    let mut union = 0;
    let mut intersection = 0;
    let mut different = 0;
    for (x, y) in a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)) {
        let (ax, bx) = (x[3] > 0, y[3] > 0);
        union += usize::from(ax || bx);
        intersection += usize::from(ax && bx);
        different += usize::from(x != y);
    }
    json!({"different_pixels":different,"silhouette_iou":intersection as f64 / union.max(1) as f64,
        "union_pixels":union,"live_opaque_pixels":a.opaque_pixels(),"sprite_opaque_pixels":b.opaque_pixels()})
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).expect("output directory argument"));
    assert!(
        !out.exists(),
        "keep earlier receipts: choose a fresh output directory"
    );
    fs::create_dir_all(&out).unwrap();
    let mut report = json!({"format":1,"method":"GPU renderer probe; no simulation, window, readback or PNG encoding in frame timings",
        "sprite_method":"four frozen 64x64 colour+depth captures of the same live renderer; nearest texel fetch; 1 mip; fixed pose/light/camera",
        "resolution":[1024,768],"warmup_frames_per_round":WARM,"rounds":ROUNDS,"frames_per_round":SAMPLES,
        "order":"alternating live-first and sprite-first between rounds", "counts":[1,64,256,1024]});
    let (voxels, palette) = demo::hero();
    let appearance = Appearance {
        layers: vec!["demo:hero".into()],
        palette: palette.clone(),
        clips: vec![],
    };
    fs::write(
        out.join("recipe.json"),
        serde_json::to_vec_pretty(&appearance).unwrap(),
    )
    .unwrap();
    let start = Instant::now();
    let token =
        TokenBody::from_appearance(&appearance, |name| (name == "demo:hero").then_some(&voxels))
            .unwrap();
    let (mesh, _) = LiveBodyProjector::new()
        .project_body(&token.document, &token.volumes)
        .unwrap();
    report["mesh_build_cpu_ms"] = json!(start.elapsed().as_secs_f64() * 1000.0);
    report["recipe"] = json!({"name":"existing demo hero (pipeline art)","volume":format!("{:?}",token.volume),
        "size_voxels":token.size(),"parts":mesh.placement_count(),"quads":mesh.drawn_quads(),"vertices":mesh.drawn_quads()*6});
    let colours = material_colours(&palette);
    let start = Instant::now();
    let gpu = Gpu::new();
    report["device_init_ms"] = json!(start.elapsed().as_secs_f64() * 1000.0);
    report["adapter"] = json!({"name":gpu.info.name,"backend":format!("{:?}",gpu.info.backend),
        "driver":gpu.info.driver,"driver_info":gpu.info.driver_info,"device_type":format!("{:?}",gpu.info.device_type)});
    eprintln!("adapter: {} {:?}", gpu.info.name, gpu.info.backend);
    let start = Instant::now();
    let mut live = LiveBodyRenderer::new(&gpu.device, gpu::FORMAT, 16);
    report["live_pipeline_create_cpu_ms"] = json!(start.elapsed().as_secs_f64() * 1000.0);
    let start = Instant::now();
    let atlas = Target::new(&gpu, NATIVE, NATIVE, 4);
    let bake_timer = Timer::new(&gpu, 4);
    let mut bake_stats = Vec::new();
    for layer in 0..4 {
        let input = inputs(
            &mesh,
            &colours,
            1,
            0,
            false,
            1.0,
            NATIVE,
            NATIVE,
            layer as usize,
        );
        let views = atlas.views(layer);
        let mut encoder = gpu.encoder();
        encoder.write_timestamp(&bake_timer.queries, layer * 2);
        gpu::clear(&mut encoder, &views.0, &views.1);
        let stats = live
            .draw(
                &gpu.device,
                &gpu.queue,
                &mut encoder,
                &views.0,
                &views.1,
                input.matrix,
                None,
                &input.bodies,
            )
            .unwrap();
        bake_stats.push(json!({"mesh_upload_bytes":stats.mesh_upload_bytes,"instance_upload_bytes":stats.instance_upload_bytes,
            "palette_upload_bytes":stats.palette_upload_bytes}));
        encoder.write_timestamp(&bake_timer.queries, layer * 2 + 1);
        gpu.queue.submit([encoder.finish()]);
    }
    gpu.wait();
    report["cache_bake_wall_ms"] = json!(start.elapsed().as_secs_f64() * 1000.0);
    report["cache_bake_gpu_us_by_facing"] = json!(bake_timer.finish(&gpu));
    report["cache_first_uploads"] = json!(bake_stats);
    report["sprite_cache_bytes"] = json!({"colour":NATIVE*NATIVE*4*4,"depth":NATIVE*NATIVE*4*4,
        "total":NATIVE*NATIVE*4*4*2,"facings":4,"pixels_per_facing":[NATIVE,NATIVE]});
    for layer in 0..4 {
        fs::write(
            out.join(format!("baked-{layer}.png")),
            gpu.read(&atlas.colour, layer).to_png(),
        )
        .unwrap();
    }
    let start = Instant::now();
    let mut cached = sprite::Sprites::new(&gpu, &atlas);
    report["sprite_pipeline_create_cpu_ms"] = json!(start.elapsed().as_secs_f64() * 1000.0);
    let single = Target::new(&gpu, NATIVE, NATIVE, 1);
    let zoomed = Target::new(&gpu, 256, 256, 1);
    let board = Target::new(&gpu, 1024, 768, 1);
    let mut visuals = Vec::new();
    for facing in 0..4 {
        let input = inputs(&mesh, &colours, 1, 0, false, 1.0, NATIVE, NATIVE, facing);
        let a = capture(&gpu, &single, &mut live, &mut cached, &input, false);
        let b = capture(&gpu, &single, &mut live, &mut cached, &input, true);
        let diff = differences(&a, &b);
        fs::write(out.join(format!("live-native-{facing}.png")), a.to_png()).unwrap();
        fs::write(out.join(format!("sprite-native-{facing}.png")), b.to_png()).unwrap();
        assert_eq!(
            a.rgba, b.rgba,
            "native-size facing {facing} must match byte for byte"
        );
        visuals.push(json!({"case":"native","facing":facing,"difference":diff}));
        for (sprite, name) in [(false, "live"), (true, "sprite")] {
            let input = inputs(&mesh, &colours, 1, 0, false, 4.0, 256, 256, facing);
            let image = capture(&gpu, &zoomed, &mut live, &mut cached, &input, sprite);
            fs::write(
                out.join(format!("{name}-zoom-{facing}.png")),
                image.to_png(),
            )
            .unwrap();
        }
    }
    for count in [64, 256, 1024] {
        let input = inputs(
            &mesh,
            &colours,
            count,
            0,
            false,
            1.0,
            board.width,
            board.height,
            0,
        );
        let a = capture(&gpu, &board, &mut live, &mut cached, &input, false);
        let b = capture(&gpu, &board, &mut live, &mut cached, &input, true);
        visuals.push(
            json!({"case":"overlapping board","count":count,"difference":differences(&a,&b)}),
        );
        fs::write(out.join(format!("live-board-{count}.png")), a.to_png()).unwrap();
        fs::write(out.join(format!("sprite-board-{count}.png")), b.to_png()).unwrap();
    }
    report["visual_checks"] = json!(visuals);
    let views = board.views(0);
    let mut timings = Vec::new();
    for moving in [false, true] {
        for count in [1, 64, 256, 1024] {
            let mut cpu = [Vec::new(), Vec::new()];
            let mut gpu_us = [Vec::new(), Vec::new()];
            let mut uploads = [Value::Null, Value::Null];
            for round in 0..ROUNDS {
                for arm in if round % 2 == 0 { [0, 1] } else { [1, 0] } {
                    for frame in 0..WARM {
                        let input = inputs(
                            &mesh,
                            &colours,
                            count,
                            frame,
                            moving,
                            1.0,
                            board.width,
                            board.height,
                            0,
                        );
                        let mut encoder = gpu.encoder();
                        draw(
                            &gpu,
                            &mut encoder,
                            &views,
                            &mut live,
                            &mut cached,
                            &input,
                            arm == 1,
                        );
                        gpu.queue.submit([encoder.finish()]);
                    }
                    gpu.wait();
                    let timer = Timer::new(&gpu, SAMPLES);
                    for frame in 0..SAMPLES {
                        let start = Instant::now();
                        let input = inputs(
                            &mesh,
                            &colours,
                            count,
                            frame + WARM,
                            moving,
                            1.0,
                            board.width,
                            board.height,
                            0,
                        );
                        let mut encoder = gpu.encoder();
                        encoder.write_timestamp(&timer.queries, frame * 2);
                        uploads[arm] = draw(
                            &gpu,
                            &mut encoder,
                            &views,
                            &mut live,
                            &mut cached,
                            &input,
                            arm == 1,
                        );
                        encoder.write_timestamp(&timer.queries, frame * 2 + 1);
                        gpu.queue.submit([encoder.finish()]);
                        cpu[arm].push(start.elapsed().as_secs_f64() * 1_000_000.0);
                    }
                    gpu_us[arm].extend(timer.finish(&gpu));
                }
            }
            for arm in 0..2 {
                timings.push(json!({"tokens":count,"moving":moving,"arm":if arm==0 {"live"} else {"sprite"},
                    "cpu_prepare_encode_submit":summary(std::mem::take(&mut cpu[arm])),
                    "gpu_clear_and_draw":summary(std::mem::take(&mut gpu_us[arm])),"last_frame_uploads":uploads[arm]}));
            }
            eprintln!("measured count={count} moving={moving}");
        }
    }
    report["timings"] = json!(timings);
    fs::write(
        out.join("comparison.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("receipt: {}", out.display());
}
