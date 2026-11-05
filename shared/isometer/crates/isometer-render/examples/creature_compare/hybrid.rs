// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Bounded R0/R1 presentation experiment, not a product LOD implementation.
use super::*;
#[path = "hybrid/composite.rs"]
mod composite;
#[path = "hybrid/policy.rs"]
mod policy;
use composite::Composite;
use policy::{Choice, Config, choose};
const HEIGHTS: [u32; 6] = [10, 24, 40, 48, 64, 96];
const WARM: usize = 24;

struct Scene {
    target: Target,
    terrain: Target,
    renderer: implicit::Renderer,
}
impl Scene {
    fn new(gpu: &Gpu, w: u32, h: u32) -> Self {
        let mut target = Target::new(gpu, w, h, 1);
        let terrain = Target::new(gpu, w, h, 1);
        let renderer = implicit::Renderer::new(&gpu.device, gpu::FORMAT, [w, h]);
        target.depth = renderer.depth_texture().clone();
        let mut e = gpu.encoder();
        let v = terrain.views(0);
        gpu::clear(&mut e, &v.0, &v.1);
        gpu.queue.submit([e.finish()]);
        Self {
            target,
            terrain,
            renderer,
        }
    }
    fn draw(
        &self,
        gpu: &Gpu,
        e: &mut wgpu::CommandEncoder,
        cam: Camera,
        zoom: f32,
        s: &[Surface],
    ) -> u64 {
        self.renderer
            .encode(
                &gpu.device,
                &gpu.queue,
                e,
                &self.target.colour,
                &self.terrain.depth,
                frame(cam, self.target.width, self.target.height, zoom),
                &s.iter().collect::<Vec<_>>(),
            )
            .unwrap()
    }
}
struct World {
    cam: Camera,
    origins: Vec<Vec3>,
    bounds: [Vec3; 2],
    height: f32,
}
impl World {
    fn new(r: &Recipe, n: usize) -> Self {
        let cam = camera(r);
        let mut points = Vec::new();
        for y in -5i32..=5 {
            for x in -5i32..=5 {
                points.push((x, y));
            }
        }
        points.sort_by_key(|&(x, y)| (x * x + y * y, y, x));
        let origins = points
            .into_iter()
            .take(n)
            .map(|(x, y)| offset(cam, x as f32 * 76.0, y as f32 * 76.0, 1.0))
            .collect();
        let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        for f in 0..24 {
            let (a, b) = r.pose(f, -Vec3::from_array(r.roots[f])).bounds().unwrap();
            lo = lo.min(a.into());
            hi = hi.max(b.into());
        }
        let bounds = [lo, hi];
        let projected = project(cam.view_proj(), bounds, Vec3::ZERO);
        Self {
            cam,
            origins,
            bounds,
            height: (projected[1].y - projected[0].y) * 32.0,
        }
    }
    fn zoom(&self, pixels: u32) -> f32 {
        pixels as f32 / self.height
    }
    fn visible(&self, w: u32, h: u32, pixels: u32) -> Vec<usize> {
        let matrix = Mat4::from_cols_array_2d(&matrix(self.cam, w, h, self.zoom(pixels)));
        self.origins
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                let [a, b] = project(matrix, self.bounds, *o);
                (!(b.x < -1.0 || a.x > 1.0 || b.y < -1.0 || a.y > 1.0)).then_some(i)
            })
            .collect()
    }
    fn surfaces(&self, r: &Recipe, visible: &[usize], f: usize, focus_only: bool) -> Vec<Surface> {
        visible
            .iter()
            .filter(|&&i| !focus_only || i == 0)
            .map(|&i| r.pose((f + i) % 24, self.origins[i]))
            .collect()
    }
}
fn project(m: Mat4, b: [Vec3; 2], origin: Vec3) -> [Vec3; 2] {
    let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for k in 0..8 {
        let p = Vec3::new(
            b[(k & 1) as usize].x,
            b[((k >> 1) & 1) as usize].y,
            b[((k >> 2) & 1) as usize].z,
        ) + origin;
        let v = m.transform_point3(p);
        lo = lo.min(v);
        hi = hi.max(v);
    }
    [lo, hi]
}
struct Probe {
    live: Scene,
    mixed: Scene,
    output: Target,
    cache: Cache,
    composite: Composite,
}
impl Probe {
    fn new(g: &Gpu, n: usize, w: u32, h: u32) -> Self {
        let live = Scene::new(g, w, h);
        let mixed = Scene::new(g, w, h);
        let output = Target::new(g, w, h, 1);
        let cache = Cache::new(g, n - 1);
        let composite = Composite::new(g, &live.target, &mixed.target);
        Self {
            live,
            mixed,
            output,
            cache,
            composite,
        }
    }
    fn live(
        &self,
        g: &Gpu,
        e: &mut wgpu::CommandEncoder,
        r: &Recipe,
        world: &World,
        visible: &[usize],
        f: usize,
        pixels: u32,
    ) -> u64 {
        self.live.draw(
            g,
            e,
            world.cam,
            world.zoom(pixels),
            &world.surfaces(r, visible, f, false),
        )
    }
    fn bake(
        &self,
        g: &Gpu,
        e: &mut wgpu::CommandEncoder,
        r: &Recipe,
        world: &World,
        visible: &[usize],
        f: usize,
    ) -> u64 {
        let c = &self.cache;
        let positions = layout(c.slots, c.target.width, c.target.height, true);
        let s = visible
            .iter()
            .filter(|&&i| i != 0)
            .map(|&i| {
                let phase = (f + i) % 24;
                let p = positions[i - 1];
                // Cache only the local pose. World/root position is applied every display frame.
                r.pose(
                    phase,
                    offset(world.cam, p[0], p[1], 1.0) - Vec3::from_array(r.roots[phase]),
                )
            })
            .collect::<Vec<_>>();
        c.renderer
            .encode(
                &g.device,
                &g.queue,
                e,
                &c.target.colour,
                &c.terrain,
                frame(world.cam, c.target.width, c.target.height, 1.0),
                &s.iter().collect::<Vec<_>>(),
            )
            .unwrap()
    }
    fn mixed(
        &mut self,
        g: &Gpu,
        e: &mut wgpu::CommandEncoder,
        r: &Recipe,
        world: &World,
        visible: &[usize],
        f: usize,
        pixels: u32,
        refresh: bool,
    ) -> u64 {
        let mut upload = if refresh {
            self.bake(g, e, r, world, visible, f)
        } else {
            0
        };
        let zoom = world.zoom(pixels);
        upload += self
            .mixed
            .draw(g, e, world.cam, zoom, &world.surfaces(r, visible, f, true));
        let t = &self.mixed.target;
        let m = Mat4::from_cols_array_2d(&matrix(world.cam, t.width, t.height, zoom));
        let instances = visible
            .iter()
            .filter(|&&i| i != 0)
            .map(|&i| {
                let shift = world.origins[i] + Vec3::from_array(r.roots[(f + i) % 24]);
                let clip = m * shift.extend(0.0);
                sprite::Instance {
                    rect: [
                        clip.x,
                        clip.y,
                        64.0 * zoom / t.width as f32,
                        64.0 * zoom / t.height as f32,
                    ],
                    layer_depth: [(i - 1) as f32, clip.z, 0.0, 0.0],
                }
            })
            .collect::<Vec<_>>();
        let v = t.views(0);
        upload += self.cache.sprites.draw(g, e, &v.0, &v.1, &instances) as u64;
        upload
    }
}
fn benchmark(
    g: &Gpu,
    p: &mut Probe,
    r: &Recipe,
    world: &World,
    c: Config,
    pixels: u32,
    mixed: bool,
) -> Value {
    let visible = world.visible(p.live.target.width, p.live.target.height, pixels);
    let period = choose(
        Choice {
            live: false,
            period: 0,
        },
        pixels as f32,
        true,
        false,
        c,
    )
    .period;
    let (mut cpu, mut times, mut refresh_gpu, mut held_gpu, mut uploads) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for _ in 0..3 {
        for f in 0..WARM {
            let mut e = g.encoder();
            if mixed {
                p.mixed(g, &mut e, r, world, &visible, f, pixels, f % period == 0);
            } else {
                p.live(g, &mut e, r, world, &visible, f, pixels);
            }
            g.queue.submit([e.finish()]);
        }
        g.wait();
        let timer = Timer::new(g, 72);
        for f in 0..72 {
            let start = Instant::now();
            let mut e = g.encoder();
            e.write_timestamp(&timer.queries, f * 2);
            let upload = if mixed {
                p.mixed(
                    g,
                    &mut e,
                    r,
                    world,
                    &visible,
                    f as usize,
                    pixels,
                    f as usize % period == 0,
                )
            } else {
                p.live(g, &mut e, r, world, &visible, f as usize, pixels)
            };
            e.write_timestamp(&timer.queries, f * 2 + 1);
            g.queue.submit([e.finish()]);
            cpu.push(start.elapsed().as_secs_f64() * 1e6);
            uploads.push(upload);
        }
        let gpu_times = timer.finish(g);
        for (f, &t) in gpu_times.iter().enumerate() {
            if mixed && f % period == 0 {
                refresh_gpu.push(t)
            } else {
                held_gpu.push(t)
            }
        }
        times.extend(gpu_times);
    }
    json!({"height":pixels,"mixed":mixed,"visible":visible.len(),"live":if mixed {usize::from(visible.contains(&0))}else{visible.len()},
        "sprites":if mixed {visible.len()-usize::from(visible.contains(&0))}else{0},"period":period,"cpu":stats(cpu),"gpu":stats(times),
        "refresh_gpu":if refresh_gpu.is_empty(){Value::Null}else{stats(refresh_gpu)},"held_gpu":if held_gpu.is_empty(){Value::Null}else{stats(held_gpu)},
        "mean_upload_bytes":uploads.iter().sum::<u64>() as f64/uploads.len() as f64})
}
fn save(g: &Gpu, t: &Target, out: &std::path::Path, name: &str) {
    fs::write(out.join(name), g.read(&t.colour, 0).to_png()).unwrap();
}
fn captures(g: &Gpu, p: &mut Probe, r: &Recipe, world: &World, c: Config, out: &std::path::Path) {
    for height in HEIGHTS {
        let visible = world.visible(p.live.target.width, p.live.target.height, height);
        let period = choose(
            Choice {
                live: false,
                period: 0,
            },
            height as f32,
            true,
            false,
            c,
        )
        .period;
        for f in 0..24 {
            let mut e = g.encoder();
            p.live(g, &mut e, r, world, &visible, f, height);
            p.mixed(g, &mut e, r, world, &visible, f, height, f % period == 0);
            g.queue.submit([e.finish()]);
            save(g, &p.live.target, out, &format!("live-{height}-{f}.png"));
            save(g, &p.mixed.target, out, &format!("mixed-{height}-{f}.png"));
        }
    }
}
fn transition(
    g: &Gpu,
    p: &mut Probe,
    r: &Recipe,
    world: &World,
    c: Config,
    out: &std::path::Path,
    capture: bool,
) -> Value {
    let pixels = 48;
    let visible = world.visible(p.live.target.width, p.live.target.height, pixels);
    let mut rows = Vec::new();
    let count = (c.transition * 2) as u32;
    for round in 0..3 {
        let timer = Timer::new(g, count);
        let mut cpu = Vec::new();
        for f in 0..count {
            let start = Instant::now();
            let mut e = g.encoder();
            e.write_timestamp(&timer.queries, f * 2);
            p.live(g, &mut e, r, world, &visible, f as usize, pixels);
            p.mixed(
                g,
                &mut e,
                r,
                world,
                &visible,
                f as usize,
                pixels,
                f as usize % c.periods[0] == 0,
            );
            let step = if (f as usize) < c.transition {
                f as usize + 1
            } else {
                count as usize - f as usize - 1
            };
            p.composite
                .draw(g, &mut e, &p.output, (step * 16 / c.transition) as u32);
            e.write_timestamp(&timer.queries, f * 2 + 1);
            g.queue.submit([e.finish()]);
            cpu.push(start.elapsed().as_secs_f64() * 1e6);
        }
        for (f, t) in timer.finish(g).into_iter().enumerate() {
            rows.push(json!({"round":round,"frame":f,"entry_refresh":f==0,"refresh":f%c.periods[0]==0,"cpu_us":cpu[f],"gpu_us":t}));
        }
    }
    let mut verification = json!(null);
    if capture {
        let mut e = g.encoder();
        p.live(g, &mut e, r, world, &visible, 12, pixels);
        p.mixed(g, &mut e, r, world, &visible, 12, pixels, true);
        g.queue.submit([e.finish()]);
        let a = g.read(&p.live.target.colour, 0);
        let b = g.read(&p.mixed.target.colour, 0);
        let mut checked = 0;
        let mut novel = 0;
        for (x, y) in a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)) {
            let mean = [0, 1, 2, 3].map(|k| ((x[k] as u16 + y[k] as u16) / 2) as u8);
            novel += usize::from(mean != x && mean != y);
        }
        assert!(
            novel > 0,
            "alpha control must create pixels absent from both inputs"
        );
        save(g, &p.live.target, out, "switch-live.png");
        save(g, &p.mixed.target, out, "switch-mixed.png");
        for weight in (0..=16).step_by(2) {
            let mut e = g.encoder();
            p.composite.draw(g, &mut e, &p.output, weight);
            g.queue.submit([e.finish()]);
            let image = g.read(&p.output.colour, 0);
            for ((x, y), z) in a
                .rgba
                .chunks_exact(4)
                .zip(b.rgba.chunks_exact(4))
                .zip(image.rgba.chunks_exact(4))
            {
                assert!(z == x || z == y);
                checked += 1;
            }
            fs::write(out.join(format!("switch-{weight}.png")), image.to_png()).unwrap();
        }
        verification = json!({"source_pixel_checks":checked,"new_dither_pixels":0,"alpha_control_novel_pixels":novel});
    }
    json!({"height":pixels,"rows":rows,"verification":verification,"note":"two pictures + depth-preserving ordered selection; first frame forces entry refresh; retained cache allocations already exist"})
}
pub fn run() {
    let args = std::env::args().collect::<Vec<_>>();
    let out = PathBuf::from(args.get(1).expect("fresh output directory"));
    assert!(!out.join("hybrid.json").exists(), "preserve old receipts");
    fs::create_dir_all(&out).unwrap();
    let data: Value =
        serde_json::from_slice(&fs::read(args.get(2).expect("motion JSON")).unwrap()).unwrap();
    let config = Config::load(
        args.get(3)
            .map(|path| serde_json::from_slice(&fs::read(path).unwrap()).unwrap()),
    );
    let checks = policy::checks(config);
    let recipe = Recipe::load(&data);
    let g = Gpu::new();
    let reverse = std::env::var_os("CREATURE_COMPARE_REVERSE").is_some();
    let setup = Instant::now();
    let mut p = Probe::new(&g, 64, 1024, 768);
    g.wait();
    let setup_us = setup.elapsed().as_secs_f64() * 1e6;
    let world = World::new(&recipe, 64);
    let mut cases = Vec::new();
    let mut heights = HEIGHTS.to_vec();
    if reverse {
        heights.reverse();
    }
    for height in heights {
        for mixed in if reverse {
            [true, false]
        } else {
            [false, true]
        } {
            let v = benchmark(&g, &mut p, &recipe, &world, config, height, mixed);
            println!(
                "height={height} mixed={mixed} visible={} CPU={} GPU={}",
                v["visible"], v["cpu"]["mean_us"], v["gpu"]["mean_us"]
            );
            cases.push(v);
        }
    }
    let switch = transition(&g, &mut p, &recipe, &world, config, &out, false);
    let mut visual_verification = json!(null);
    if !reverse {
        let mut visual = Probe::new(&g, 16, 256, 192);
        let world = World::new(&recipe, 16);
        captures(&g, &mut visual, &recipe, &world, config, &out);
        visual_verification =
            transition(&g, &mut visual, &recipe, &world, config, &out, true)["verification"]
                .clone();
    }
    let report = json!({"adapter":format!("{:?}",g.info),"config":config.json(),"checks":checks,"cases":cases,"transition":switch,
        "visual_verification":visual_verification,"source_seed":data["seed"],"source_id":data["id"],"parts":recipe.document.parts.len(),
        "cache_capacity_bytes":p.cache.target.width*p.cache.target.height*8,"cache_slots":p.cache.slots,"setup_wall_us":setup_us,
        "root_transform":"every displayed frame; baked root removed; native local foot pose alone is held",
        "method":"64 generated clones, one examined, independent staggered poses. 1024x768; 3x72 samples, 24 warm frames/round. 256x192/16-clone captures. Projected union bounds choose zoom, fixed world placements, visibility culling. Cache 64px colour+depth, one facing/light. GPU includes offscreen refresh, current-root sprites and live draw. Native physics/planning, readback/PNG and resource creation excluded from steady frame times; setup recorded separately. Reverse-order confirmation."});
    fs::write(
        out.join("hybrid.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
