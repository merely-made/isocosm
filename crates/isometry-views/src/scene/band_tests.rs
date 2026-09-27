//! Where one spare layer of headroom moves pixels: a probe of the headed 256
//! by 256 frame (2026-09-26), run by hand with `--ignored --nocapture`, its
//! raw output kept out of tree. `testing/scene-board-paging/bands.md` has
//! what it found.
//!
//! It rebuilds the headed session's exact frame (the host's pixel grid, pane,
//! pan and 890 by 752 scene texture), builds the brick map at headroom 0 and
//! at 1, and asks four things of the texels the two disagree on: which texels
//! (on the CPU through the maps' own `trace_ray`, the shader's mirror, and on
//! the GPU); how far each column's rays sit from running along voxel edges;
//! which part of the pointer box moves them, through a replica of the
//! traversal whose box can be moved alone; and what an f64 traversal of the
//! same ray, started from no box at all, says the pixel shows.

use std::cell::RefCell;
use std::collections::HashMap;

use isometer::lens::{BrickMap, TraceCamera};
use isometer::{
    BRICK_BYTES, BrickSource, ResidencySettings, Scene, SceneFrame, SceneVolumes, SlabCamera,
    TerrainSource, VolumeMap,
};
use serde_json::json;

use super::board::{PlainHost, board_appearance, board_grade};
use super::columns::{BoardBricks, TileColumns};
use super::ground::BoardGround;
use super::harness::device;
use super::overlay::{Overlays, terrain_palette};
use super::world::{BoardWorld, tallest};
use crate::demo::synth_map;
use crate::state::UiState;

/// The headed session: a 2200 by 1504 window at device scale 2 and zoom
/// 0.9170732, the 228 px panel beside the board, the host's opening pan, and
/// the scene texture the producer drew at render scale 2.
const DEVICE: f32 = 2.0;
const ZOOM: f32 = 0.917_073_2;
const PAN: (f32, f32) = (420.0, 140.0);
const TEXTURE: [u32; 2] = [890, 752];

mod gpu;
mod traversal;

use gpu::{pixel, render};
use traversal::{lockstep, ray64, replica, truth};

type Hit = Option<([i32; 3], u8)>;

fn line(value: serde_json::Value) {
    println!("[band-probe] {value}");
}

/// The headed board's state, at the host's pixel grid, pane and pan.
fn headed() -> UiState {
    let mut ui = UiState::new(synth_map(256, 256));
    ui.set_pixel_grid((DEVICE, ZOOM));
    ui.viewport = ((2200.0 / DEVICE) / ZOOM - 228.0, (1504.0 / DEVICE) / ZOOM);
    ui.camera = PAN;
    ui
}

fn camera(ui: &UiState) -> SlabCamera {
    BoardWorld::with_tallest(&ui.map, tallest(&ui.map))
        .camera(&ui.geo, ui.camera, ui.viewport, None)
        .expect("the headed pane frames the board")
}

/// The board's brick map at `headroom`, and its pointer box in world units.
fn built(
    ui: &UiState,
    camera: SlabCamera,
    headroom: u32,
) -> (BrickMap, [f32; 3], [f32; 3], Vec<[i16; 3]>) {
    let settings = ResidencySettings {
        headroom,
        ..ResidencySettings::default()
    };
    let ground = BoardGround::new(&ui.map, &Overlays::of(ui), 1, settings);
    let bricks = ground.bricks();
    let framed = ground.framed(camera, &bricks);
    let map = ground
        .terrain(&bricks, &framed)
        .brick_map()
        .expect("a paged map");
    let low = map.origin().map(|key| f32::from(key) * 8.0);
    let extent = map.pointer_extent();
    let high = [0, 1, 2].map(|axis| low[axis] + extent[axis] as f32 * 8.0);
    (map, low, high, framed.keys)
}

fn ndc(px: u32, py: u32) -> [f32; 2] {
    [
        2.0 * (px as f32 + 0.5) / TEXTURE[0] as f32 - 1.0,
        1.0 - 2.0 * (py as f32 + 0.5) / TEXTURE[1] as f32,
    ]
}

/// The ground's own materials, made from the tile columns on demand.
struct Ground<'a> {
    bricks: BoardBricks<'a>,
    cache: RefCell<HashMap<[i16; 3], Option<Vec<u8>>>>,
}

impl Ground<'_> {
    fn material(&self, voxel: [i32; 3]) -> u8 {
        let key = voxel.map(|v| v.div_euclid(8) as i16);
        let mut cache = self.cache.borrow_mut();
        let brick = cache.entry(key).or_insert_with(|| {
            self.bricks
                .layers([key[0], key[2]])
                .contains(&key[1])
                .then(|| {
                    let mut bytes = vec![0; BRICK_BYTES];
                    self.bricks.fill(key, &mut bytes);
                    bytes
                })
        });
        let local = voxel.map(|v| v.rem_euclid(8));
        brick.as_ref().map_or(0, |bytes| {
            bytes[((local[1] * 8 + local[2]) * 8 + local[0]) as usize]
        })
    }
}

#[test]
#[ignore = "a probe run by hand; its output is kept out of tree"]
fn headroom_bands() {
    let ui = headed();
    let camera = camera(&ui);
    let trace: TraceCamera = camera.trace().expect("a trace camera");
    let fields = serde_json::to_value(trace).expect("the camera serializes");
    let (map0, low0, high0, keys) = built(&ui, camera, 0);
    let (map1, low1, high1, _) = built(&ui, camera, 1);
    line(
        json!({"kind": "setup", "pane": ui.viewport, "geo": [ui.geo.tile_w, ui.geo.tile_h],
        "camera": format!("{camera:?}"), "trace": fields,
        "box0": [low0, high0], "box1": [low1, high1]}),
    );

    let columns = TileColumns::of(&ui.map, &Overlays::of(&ui));
    let ground = Ground {
        bricks: BoardBricks::new(&columns, None),
        cache: RefCell::new(HashMap::new()),
    };
    let rays: Vec<(u32, u32, [f32; 3], [f32; 3])> = (0..TEXTURE[1])
        .flat_map(|py| (0..TEXTURE[0]).map(move |px| (px, py)))
        .map(|(px, py)| {
            let (o, d) = trace.ray_at(ndc(px, py)).expect("a ray");
            (px, py, o, d)
        })
        .collect();
    let far = trace.far();
    let through = |map: &BrickMap, o, d| -> Hit {
        map.trace_ray(o, d, far)
            .ok()
            .flatten()
            .map(|hit| (hit.voxel, hit.material))
    };

    // 1. Which texels the two maps disagree on, through `trace_ray`.
    let mut moved = Vec::new();
    for (px, py, o, d) in &rays {
        let (a, b) = (through(&map0, *o, *d), through(&map1, *o, *d));
        if a != b {
            moved.push((*px, *py, *o, *d, a, b));
        }
    }
    let mut by_column: HashMap<u32, usize> = HashMap::new();
    for (px, ..) in &moved {
        *by_column.entry(*px).or_default() += 1;
    }
    let (mut material, mut face) = (0, 0);
    for (_, _, o, d, ..) in &moved {
        let (a, b) = (map0.trace_ray(*o, *d, far), map1.trace_ray(*o, *d, far));
        if let (Ok(Some(a)), Ok(Some(b))) = (a, b) {
            material += usize::from(a.material != b.material);
            face += usize::from(a.normal != b.normal);
        }
    }
    line(
        json!({"kind": "cpu-diff", "texels": moved.len(), "columns": by_column,
        "material_changed": material, "face_changed": face}),
    );

    // 2. How far each column's rays run from voxel edges: x - z of the f64
    // ray, the same for every row, against the nearest integer.
    let mut alignment: Vec<(f64, u32, f64)> = (0..TEXTURE[0])
        .map(|px| {
            let (o, _) = ray64(&fields, px, 0);
            let diagonal = o[0] - o[2];
            ((diagonal - diagonal.round()).abs(), px, diagonal)
        })
        .collect();
    alignment.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (distance, px, diagonal) in alignment.iter().take(8) {
        line(json!({"kind": "column", "px": px, "x_minus_z": diagonal,
            "off_integer": distance, "moved": by_column.get(px).copied().unwrap_or(0)}));
    }

    // 3. The replica, first against the maps it mirrors, then with the box
    // moved one part at a time.
    let box0 = [low0, high0];
    let variants: Vec<(&str, [[f32; 3]; 2])> = vec![
        ("headroom 0", box0),
        ("headroom 1", [low1, high1]),
        ("up only", [low0, [high0[0], high1[1], high0[2]]]),
        (
            "across only",
            [[low1[0], low0[1], low1[2]], [high1[0], high0[1], high1[2]]],
        ),
        ("grown down", [[low0[0], low0[1] - 8.0, low0[2]], high0]),
        (
            "far faces out a brick",
            [[low0[0] - 8.0, low0[1], low0[2] - 8.0], high0],
        ),
        (
            "shifted out a brick",
            [
                [low0[0] - 8.0, low0[1], low0[2] - 8.0],
                [high0[0] - 8.0, high0[1], high0[2] - 8.0],
            ],
        ),
        (
            "top up a quarter voxel",
            [low0, [high0[0], high0[1] + 0.25, high0[2]]],
        ),
        (
            "top up one voxel",
            [low0, [high0[0], high0[1] + 1.0, high0[2]]],
        ),
        (
            "top up two layers",
            [low0, [high0[0], high0[1] + 16.0, high0[2]]],
        ),
    ];
    let base: Vec<Hit> = rays
        .iter()
        .map(|(_, _, o, d)| through(&map0, *o, *d))
        .collect();
    for (name, bounds) in &variants {
        let (mut unlike_base, mut unlike_self) = (0usize, 0usize);
        for ((_, _, o, d), before) in rays.iter().zip(&base) {
            let hit = replica(*o, *d, far, *bounds, &ground);
            unlike_base += usize::from(hit != *before);
            if *name == "headroom 1" {
                unlike_self += usize::from(hit != through(&map1, *o, *d));
            }
        }
        let lost = keys
            .iter()
            .filter(|key| {
                (0..3).any(|axis| {
                    let low = f32::from(key[axis]) * 8.0;
                    low < bounds[0][axis] || low + 8.0 > bounds[1][axis]
                })
            })
            .count();
        line(
            json!({"kind": "variant", "name": name, "box": bounds, "framed_bricks_outside": lost,
            "texels_unlike_headroom_0_map": unlike_base,
            "texels_unlike_headroom_1_map": if *name == "headroom 1" { json!(unlike_self) } else { json!(null) }}),
        );
    }

    // 4. The f64 truth for every moved texel.
    let top = f64::from(high1[1]) + 8.0;
    let (mut with0, mut with1, mut edge) = (0, 0, 0);
    for (px, py, o, _, a, b) in &moved {
        let (origin, direction) = ray64(&fields, *px, *py);
        let (hit, gap) = truth(origin, direction, top, &ground);
        with0 += usize::from(hit == *a);
        with1 += usize::from(hit == *b);
        edge += usize::from(gap < 1e-9);
        line(json!({"kind": "texel", "px": px, "py": py, "origin32": o,
            "x_minus_z": origin[0] - origin[2], "headroom0": a, "headroom1": b,
            "f64": hit, "f64_gap": gap}));
    }
    line(
        json!({"kind": "truth", "moved": moved.len(), "f64_agrees_with_headroom_0": with0,
        "f64_agrees_with_headroom_1": with1, "on_an_edge_within_1e-9": edge}),
    );

    let (mut wrong0, mut wrong1) = (0usize, 0usize);
    for (px, py, o, d) in &rays {
        let (origin, direction) = ray64(&fields, *px, *py);
        let (hit, _) = truth(origin, direction, top, &ground);
        wrong0 += usize::from(hit != through(&map0, *o, *d));
        wrong1 += usize::from(hit != through(&map1, *o, *d));
    }
    line(json!({"kind": "frame-truth", "texels": rays.len(),
        "headroom_0_unlike_f64": wrong0, "headroom_1_unlike_f64": wrong1}));

    let mut first: HashMap<u32, (u32, [f32; 3], [f32; 3])> = HashMap::new();
    for (px, py, o, d, ..) in &moved {
        first.entry(*px).or_insert((*py, *o, *d));
    }
    let mut firsts: Vec<_> = first.into_iter().collect();
    firsts.sort_by_key(|entry| entry.0);
    for (px, (py, o, d)) in firsts {
        for (name, bounds) in [("headroom 0", box0), ("headroom 1", [low1, high1])] {
            line(lockstep(px, py, o, d, far, bounds, name, &ground));
        }
    }

    // 5. The GPU's own two pictures.
    match (render(&ui, camera, 0), render(&ui, camera, 1)) {
        (Some(zero), Some(one)) => {
            let mut gpu = Vec::new();
            for py in 0..TEXTURE[1] {
                for px in 0..TEXTURE[0] {
                    if pixel(&zero, px, py) != pixel(&one, px, py) {
                        gpu.push((px, py));
                    }
                }
            }
            let cpu: std::collections::BTreeSet<_> = moved.iter().map(|m| (m.0, m.1)).collect();
            let shared = gpu.iter().filter(|at| cpu.contains(at)).count();
            let mut gpu_columns: HashMap<u32, usize> = HashMap::new();
            for (px, _) in &gpu {
                *gpu_columns.entry(*px).or_default() += 1;
            }
            line(
                json!({"kind": "gpu-diff", "texels": gpu.len(), "columns": gpu_columns,
                "also_moved_on_the_cpu": shared}),
            );
        },
        _ => eprintln!("SKIPPED: no wgpu adapter, so the GPU arm drew nothing."),
    }
}
