struct Frame {
    clip_from_world: mat4x4<f32>,
    slab_normal_min: vec4<f32>,
    slab_max_enabled: vec4<f32>,
    bounds_min: vec4<f32>,
    bounds_max: vec4<f32>,
    // Toward the sun and ambient; sun weight and 1 when lit (case 1170).
    light_toward_ambient: vec4<f32>,
    light_sun_enabled: vec4<f32>,
    // x: 1 for the body tenant's Neutral-then-2.2 resolve (case 1176).
    display: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

// One body's colour table. Entry i is material i, and its w marks an entry the
// table actually names; an all-zero block is "no table" and every material
// then keeps the vertex's own hashed colour. Bound per draw by a dynamic
// offset, so bodies sharing a palette still batch.
struct Palette {
    entries: array<vec4<f32>, 256>,
};

@group(1) @binding(0)
var<uniform> palette: Palette;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) model_x: vec4<f32>,
    @location(3) model_y: vec4<f32>,
    @location(4) model_z: vec4<f32>,
    @location(5) model_w: vec4<f32>,
    @location(6) tint: vec4<f32>,
    @location(7) expression: vec4<f32>,
    @location(8) expression_tail: vec4<f32>,
    @location(9) material_shade: vec2<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) local_position: vec3<f32>,
    @location(3) expression: vec4<f32>,
    @location(4) expression_tail: vec4<f32>,
    @location(5) inspection_accent: f32,
    @location(6) @interpolate(flat) sprite_coverage: u32,
    // World face normal, then the face shade baked into `color`.
    @location(7) @interpolate(flat) normal_shade: vec4<f32>,
};

// Each face shade names one axis direction (`geometry::face_shade`).
fn face_normal(shade: f32) -> vec3<f32> {
    if (shade > 0.93) { return vec3<f32>(0.0, 1.0, 0.0); }
    if (shade > 0.82) { return vec3<f32>(0.0, 0.0, 1.0); }
    if (shade > 0.72) { return vec3<f32>(1.0, 0.0, 0.0); }
    if (shade > 0.62) { return vec3<f32>(-1.0, 0.0, 0.0); }
    if (shade > 0.515) { return vec3<f32>(0.0, 0.0, -1.0); }
    return vec3<f32>(0.0, -1.0, 0.0);
}

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    let model = mat4x4<f32>(input.model_x, input.model_y, input.model_z, input.model_w);
    let world = model * vec4<f32>(input.position, 1.0);
    // The vertex already carries material_colour(material) * face_shade(..).
    // A table entry replaces only the material half of that product, so the
    // face shading and the tint multiply exactly as they always have.
    var base = input.color;
    let entry = palette.entries[u32(input.material_shade.x)];
    if (entry.w > 0.5) {
        base = entry.xyz * input.material_shade.y;
    }
    return VertexOut(
        frame.clip_from_world * world,
        base * input.tint.xyz,
        world.xyz,
        input.position,
        input.expression,
        input.expression_tail,
        input.tint.w,
        u32(input.expression_tail.y),
        vec4<f32>(
            normalize((model * vec4<f32>(face_normal(input.material_shade.y), 0.0)).xyz),
            input.material_shade.y,
        ),
    );
}

fn channel_colour(index: u32) -> vec3<f32> {
    // One mark per tissue channel, in the producer's own channel order. These
    // marks mean an allocated material, not a kingdom or a guessed surface.
    switch index {
        case 0u: { return vec3<f32>(0.82, 0.30, 0.25); }
        case 1u: { return vec3<f32>(0.82, 0.55, 0.20); }
        case 2u: { return vec3<f32>(0.32, 0.72, 0.92); }
        case 3u: { return vec3<f32>(0.34, 0.76, 0.30); }
        default: { return vec3<f32>(0.78, 0.25, 0.72); }
    }
}

fn tissue_mark(local_position: vec3<f32>, expression: vec4<f32>, tail: f32) -> vec3<f32> {
    // A regular voxel-aligned screen. The mosaic has no render-voxel
    // coordinates, so this is a density reading of caller-supplied fractions,
    // never a fabricated cell-to-voxel assignment. The five fractions arrive
    // in the producer's channel order and are read positionally.
    let cell = floor(local_position + vec3<f32>(0.001));
    let phase = fract((cell.x + 3.0 * cell.y + 5.0 * cell.z) / 16.0);
    let end_0 = expression.x;
    let end_1 = end_0 + expression.y;
    let end_2 = end_1 + expression.z;
    let end_3 = end_2 + expression.w;
    let end_4 = end_3 + tail;
    if (phase < end_0) { return channel_colour(0u); }
    if (phase < end_1) { return channel_colour(1u); }
    if (phase < end_2) { return channel_colour(2u); }
    if (phase < end_3) { return channel_colour(3u); }
    if (phase < end_4) { return channel_colour(4u); }
    return vec3<f32>(-1.0);
}

// Khronos PBR Neutral, as kiss3d's resolve applies it.
fn pbr_neutral(colour_in: vec3<f32>) -> vec3<f32> {
    let start = 0.8 - 0.04;
    let x = min(colour_in.r, min(colour_in.g, colour_in.b));
    let colour = colour_in - vec3<f32>(select(0.04, x - 6.25 * x * x, x < 0.08));
    let peak = max(colour.r, max(colour.g, colour.b));
    if (peak < start) {
        return colour;
    }
    let d = 1.0 - start;
    let new_peak = 1.0 - d * d / (peak + d - start);
    let g = 1.0 - 1.0 / (0.15 * (peak - new_peak) + 1.0);
    return mix(colour * (new_peak / peak), vec3<f32>(new_peak), g);
}

fn srgb(linear: vec3<f32>) -> vec3<f32> {
    let low = linear * 12.92;
    let high = 1.055 * pow(max(linear, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, linear <= vec3<f32>(0.0031308));
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    if (selects_sprite(input.clip_position.xy, input.sprite_coverage)) {
        discard;
    }
    if (frame.slab_max_enabled.w > 0.5) {
        let slab = dot(frame.slab_normal_min.xyz, input.world_position);
        if (slab < frame.slab_normal_min.w || slab > frame.slab_max_enabled.x) {
            discard;
        }
    }
    if (frame.bounds_min.w > 0.5 &&
        (any(input.world_position < frame.bounds_min.xyz) ||
         any(input.world_position > frame.bounds_max.xyz))) {
        discard;
    }
    let mark = tissue_mark(input.local_position, input.expression, input.expression_tail.x);
    var colour = input.color;
    if (all(mark >= vec3<f32>(0.0))) {
        // Keep the face's structural light/dark contrast under the process
        // mark, so expression does not flatten a readable voxel body.
        let shade = max(max(input.color.x, input.color.y), input.color.z);
        // Retain the caller's body tint, including its deadened carrion tone.
        colour = mix(input.color, mark * shade, 0.72);
    }
    if (frame.light_sun_enabled.w > 0.5) {
        // Replace the fixed face shade with the scene sun's Lambert term.
        let lambert = frame.light_toward_ambient.w + frame.light_sun_enabled.xyz
            * max(dot(input.normal_shade.xyz, frame.light_toward_ambient.xyz), 0.0);
        colour = colour / max(input.normal_shade.w, 0.001) * lambert;
    }
    // Selection remains an accent. Replacing the material colour with amber
    // would conceal the tissue reading the inspection asked for.
    colour = mix(colour, vec3<f32>(1.0, 0.55, 0.10), input.inspection_accent * 0.28);
    // Lens FRAME_FORMAT is Rgba8Unorm: it stores display values directly.
    if (frame.display.x > 0.5) {
        return vec4<f32>(pow(max(pbr_neutral(colour), vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2)), 1.0);
    }
    return vec4<f32>(srgb(colour), 1.0);
}
