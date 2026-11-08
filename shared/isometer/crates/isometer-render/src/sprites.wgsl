// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

@group(0) @binding(0) var colours: texture_2d_array<f32>;
@group(0) @binding(1) var depths: texture_depth_2d_array;

struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) layer: u32,
    @location(2) @interpolate(flat) clip_z: vec4<f32>,
    @location(3) @interpolate(flat) clip_w: vec4<f32>,
    @location(4) @interpolate(flat) coverage: u32,
};

fn image_xy(uv: vec2<f32>) -> vec2<f32> {
    return vec2(2.0, -2.0) * uv + vec2(-1.0, 1.0);
}

@vertex
fn vs(
    @builtin(vertex_index) i: u32,
    @location(0) a: vec4<f32>,
    @location(1) b: vec4<f32>,
    @location(2) c: vec4<f32>,
    @location(3) d: vec4<f32>,
    @location(4) layer: u32,
    @location(5) coverage: u32,
) -> Vertex {
    let corners = array(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
    );
    let uv = corners[i];
    let clip = mat4x4(a, b, c, d) * vec4(image_xy(uv), 0.5, 1.0);
    // Rasterize the image rectangle at its mid-plane. Real depth clipping and
    // occlusion use the baked surface in the fragment pass.
    return Vertex(
        vec4(clip.xy, 0.5 * clip.w, clip.w), uv, layer,
        vec4(a.z, b.z, c.z, d.z), vec4(a.w, b.w, c.w, d.w),
        coverage,
    );
}

struct Fragment {
    @location(0) colour: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs(input: Vertex) -> Fragment {
    if !selects_sprite(input.position.xy, input.coverage) { discard; }
    let size = vec2<i32>(textureDimensions(colours));
    let texel = clamp(vec2<i32>(input.uv * vec2<f32>(size)), vec2(0), size - vec2(1));
    let colour = textureLoad(colours, texel, input.layer, 0);
    if colour.a < 0.5 { discard; }
    let local = vec4(image_xy(input.uv), textureLoad(depths, texel, input.layer, 0), 1.0);
    let w = dot(input.clip_w, local);
    let z = dot(input.clip_z, local);
    if w <= 0.0 || z < 0.0 || z > w { discard; }
    return Fragment(colour, z / w);
}
