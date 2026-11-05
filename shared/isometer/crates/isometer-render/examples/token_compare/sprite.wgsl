@group(0) @binding(0) var colours: texture_2d_array<f32>;
@group(0) @binding(1) var depths: texture_depth_2d_array;

struct VertexIn {
    @location(0) rect: vec4<f32>,
    @location(1) layer_depth: vec4<f32>,
};
struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) layer: i32,
    @location(2) @interpolate(flat) depth_offset: f32,
};
@vertex
fn vs(input: VertexIn, @builtin(vertex_index) index: u32) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0));
    let uv = corners[index];
    var out: VertexOut;
    out.pos = vec4(input.rect.xy + vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0) * input.rect.zw, 0.0, 1.0);
    out.uv = uv;
    out.layer = i32(input.layer_depth.x);
    out.depth_offset = input.layer_depth.y;
    return out;
}
struct FragmentOut {
    @location(0) colour: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};
@fragment
fn fs(input: VertexOut) -> FragmentOut {
    let size = vec2<i32>(textureDimensions(colours));
    let at = clamp(vec2<i32>(input.uv * vec2<f32>(size)), vec2(0), size - vec2(1));
    let colour = textureLoad(colours, at, input.layer, 0);
    if (colour.a < 0.5) { discard; }
    var out: FragmentOut;
    out.colour = colour;
    out.depth = textureLoad(depths, at, input.layer, 0) + input.depth_offset;
    return out;
}
