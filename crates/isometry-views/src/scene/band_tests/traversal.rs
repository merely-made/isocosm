//! The probe's instruments: the traversal in f32 with a movable box, the
//! same ray in f64 from no box, and the lockstep of the two.

use super::*;

/// `BrickMap::trace_ray`, step for step in f32, with the pointer box an
/// argument and the ground's own materials: the same traversal with a box
/// that can be moved on its own.
pub(super) fn replica(
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
    [low, high]: [[f32; 3]; 2],
    ground: &Ground<'_>,
) -> Hit {
    let length = direction
        .iter()
        .map(|&v| f64::from(v).powi(2))
        .sum::<f64>()
        .sqrt();
    let direction = direction.map(|v| (f64::from(v) / length) as f32);
    let (mut enter, mut exit) = (0.0f32, far);
    for axis in 0..3 {
        let d = direction[axis];
        if d.abs() < 1e-6 {
            if origin[axis] < low[axis] || origin[axis] >= high[axis] {
                return None;
            }
            continue;
        }
        let a = (low[axis] - origin[axis]) / d;
        let b = (high[axis] - origin[axis]) / d;
        enter = enter.max(a.min(b));
        exit = exit.min(a.max(b));
    }
    let start_t = enter.max(0.0) + 0.0001;
    if enter > exit || start_t > exit {
        return None;
    }
    let start = [0, 1, 2].map(|i| origin[i] + direction[i] * start_t);
    let mut voxel = start.map(|v| v.floor() as i32);
    let step = direction.map(|v| if v >= 0.0 { 1 } else { -1 });
    let mut crossing = [0, 1, 2].map(|i| {
        if direction[i] > 1e-6 {
            start_t + ((voxel[i] + 1) as f32 - start[i]) / direction[i]
        } else if direction[i] < -1e-6 {
            start_t + (voxel[i] as f32 - start[i]) / direction[i]
        } else {
            1e30
        }
    });
    let delta = direction.map(|v| if v.abs() > 1e-6 { 1.0 / v.abs() } else { 1e30 });
    for _ in 0..1024 {
        let inside = (0..3).all(|i| voxel[i] as f32 >= low[i] && (voxel[i] as f32) < high[i]);
        let material = if inside { ground.material(voxel) } else { 0 };
        if material != 0 {
            return Some((voxel, material));
        }
        let axis = if crossing[0] <= crossing[1] && crossing[0] <= crossing[2] {
            0
        } else if crossing[1] <= crossing[2] {
            1
        } else {
            2
        };
        let distance = crossing[axis];
        crossing[axis] += delta[axis];
        voxel[axis] += step[axis];
        if distance > exit || distance > far {
            return None;
        }
    }
    None
}

/// The ray of texel `(px, py)` in f64, from the trace camera's own f32
/// fields, by the shader's own formula.
pub(super) fn ray64(camera: &serde_json::Value, px: u32, py: u32) -> ([f64; 3], [f64; 3]) {
    let field = |name: &str| -> Vec<f64> {
        camera[name]
            .as_array()
            .expect("a camera field")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect()
    };
    let (o, f, r, u, w) = (
        field("origin"),
        field("forward"),
        field("right"),
        field("up"),
        field("wall"),
    );
    let nx = 2.0 * (f64::from(px) + 0.5) / f64::from(TEXTURE[0]) - 1.0;
    let ny = 1.0 - 2.0 * (f64::from(py) + 0.5) / f64::from(TEXTURE[1]);
    let advance = w[0] * nx + w[1] * ny + w[2];
    let origin = [0, 1, 2].map(|i| o[i] + r[i] * nx + u[i] * ny + f[i] * advance);
    (origin, [f[0], f[1], f[2]])
}

/// The first solid voxel along an f64 ray, started where it comes down
/// through `top`, above every voxel, so no pointer box enters into it. Each
/// crossing is recomputed from the ray's origin rather than accumulated. Also
/// the smallest gap met between the chosen crossing and the next axis's,
/// relative to its distance: near zero, the ray runs along a voxel edge and
/// either neighbour is the surface there.
pub(super) fn truth(
    origin: [f64; 3],
    direction: [f64; 3],
    top: f64,
    ground: &Ground<'_>,
) -> (Hit, f64) {
    let t0 = (top - origin[1]) / direction[1];
    let mut voxel = [0, 1, 2].map(|i| (origin[i] + direction[i] * t0).floor() as i32);
    let step = direction.map(|v| if v >= 0.0 { 1 } else { -1 });
    let mut gap = f64::INFINITY;
    for _ in 0..4096 {
        let material = ground.material(voxel);
        if material != 0 {
            return (Some((voxel, material)), gap);
        }
        if voxel[1] < 0 {
            return (None, gap);
        }
        let t = [0, 1, 2].map(|axis| {
            let boundary = voxel[axis] + i32::from(step[axis] > 0);
            (f64::from(boundary) - origin[axis]) / direction[axis]
        });
        let axis = if t[0] <= t[1] && t[0] <= t[2] {
            0
        } else if t[1] <= t[2] {
            1
        } else {
            2
        };
        let next = (0..3)
            .filter(|other| *other != axis)
            .map(|other| t[other])
            .fold(f64::INFINITY, f64::min);
        gap = gap.min((next - t[axis]).abs() / t[axis].abs());
        voxel[axis] += step[axis];
    }
    (None, gap)
}

/// One step of a traversal: the voxel it stood in, the axis it left by, and
/// the three crossing times it chose between.
pub(super) struct Step {
    pub(super) voxel: [i32; 3],
    pub(super) axis: usize,
    pub(super) crossings: [f64; 3],
}

/// The replica's steps, as `replica` takes them in f32.
pub(super) fn walk32(
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
    [low, high]: [[f32; 3]; 2],
    ground: &Ground<'_>,
) -> Vec<Step> {
    let length = direction
        .iter()
        .map(|&v| f64::from(v).powi(2))
        .sum::<f64>()
        .sqrt();
    let direction = direction.map(|v| (f64::from(v) / length) as f32);
    let (mut enter, mut exit) = (0.0f32, far);
    for axis in 0..3 {
        let a = (low[axis] - origin[axis]) / direction[axis];
        let b = (high[axis] - origin[axis]) / direction[axis];
        enter = enter.max(a.min(b));
        exit = exit.min(a.max(b));
    }
    let start_t = enter.max(0.0) + 0.0001;
    let start = [0, 1, 2].map(|i| origin[i] + direction[i] * start_t);
    let mut voxel = start.map(|v| v.floor() as i32);
    let step = direction.map(|v| if v >= 0.0 { 1 } else { -1 });
    let mut crossing = [0, 1, 2].map(|i| {
        let boundary = if direction[i] > 0.0 {
            voxel[i] + 1
        } else {
            voxel[i]
        };
        start_t + (boundary as f32 - start[i]) / direction[i]
    });
    let delta = direction.map(|v| 1.0 / v.abs());
    let mut steps = Vec::new();
    for _ in 0..1024 {
        if ground.material(voxel) != 0 || steps.len() > 400 {
            break;
        }
        let axis = if crossing[0] <= crossing[1] && crossing[0] <= crossing[2] {
            0
        } else if crossing[1] <= crossing[2] {
            1
        } else {
            2
        };
        steps.push(Step {
            voxel,
            axis,
            crossings: crossing.map(f64::from),
        });
        crossing[axis] += delta[axis];
        voxel[axis] += step[axis];
    }
    steps
}

/// The same f32 ray traversed exactly: every crossing recomputed in f64 from
/// the ray's origin, from the same starting voxel as `walk32`.
pub(super) fn walk64(
    origin: [f32; 3],
    direction: [f32; 3],
    from: [i32; 3],
    ground: &Ground<'_>,
) -> Vec<Step> {
    let length = direction
        .iter()
        .map(|&v| f64::from(v).powi(2))
        .sum::<f64>()
        .sqrt();
    let o = origin.map(f64::from);
    let d = direction.map(|v| f64::from(v) / length);
    let step = d.map(|v| if v >= 0.0 { 1 } else { -1 });
    let mut voxel = from;
    let mut steps = Vec::new();
    for _ in 0..1024 {
        if ground.material(voxel) != 0 || steps.len() > 400 {
            break;
        }
        let crossings = [0, 1, 2].map(|axis| {
            let boundary = voxel[axis] + i32::from(step[axis] > 0);
            (f64::from(boundary) - o[axis]) / d[axis]
        });
        let axis = if crossings[0] <= crossings[1] && crossings[0] <= crossings[2] {
            0
        } else if crossings[1] <= crossings[2] {
            1
        } else {
            2
        };
        steps.push(Step {
            voxel,
            axis,
            crossings,
        });
        voxel[axis] += step[axis];
    }
    steps
}

/// Where the f32 traversal of one texel first steps differently from the
/// exact one, and by how far each f32 crossing has drifted by then.
#[allow(clippy::too_many_arguments)]
pub(super) fn lockstep(
    px: u32,
    py: u32,
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
    bounds: [[f32; 3]; 2],
    name: &str,
    ground: &Ground<'_>,
) -> serde_json::Value {
    let f32_steps = walk32(origin, direction, far, bounds, ground);
    let Some(start) = f32_steps.first().map(|step| step.voxel) else {
        return json!({"kind": "lockstep", "px": px, "py": py, "box": name, "steps": 0});
    };
    let exact = walk64(origin, direction, start, ground);
    let parted = f32_steps
        .iter()
        .zip(&exact)
        .position(|(a, b)| a.voxel != b.voxel || a.axis != b.axis);
    let drift = |at: usize| -> Vec<f64> {
        (0..3)
            .map(|axis| f32_steps[at].crossings[axis] - exact[at].crossings[axis])
            .collect()
    };
    match parted {
        Some(at) => json!({"kind": "lockstep", "px": px, "py": py, "box": name,
            "start_voxel": start, "steps_to_part": at, "voxel": f32_steps[at].voxel,
            "f32_axis": f32_steps[at].axis, "exact_axis": exact[at].axis,
            "f32_crossings": f32_steps[at].crossings, "exact_crossings": exact[at].crossings,
            "drift": drift(at), "steps_taken": f32_steps.len()}),
        None => {
            let last = f32_steps.len().min(exact.len()).saturating_sub(1);
            json!({"kind": "lockstep", "px": px, "py": py, "box": name,
                "start_voxel": start, "steps_to_part": null, "steps_taken": f32_steps.len(),
                "drift_at_last_step": drift(last)})
        },
    }
}
