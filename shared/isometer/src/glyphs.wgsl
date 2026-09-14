// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
struct Clip { plane:vec4<f32>, interval:vec4<f32>, minimum:vec4<f32>, maximum:vec4<f32> }
@group(0) @binding(0) var<uniform> clip:Clip;
struct Vertex { @builtin(position) position:vec4<f32>, @location(0) color:vec4<f32>, @location(1) world:vec3<f32> }
@vertex fn vs_main(@location(0) position:vec4<f32>, @location(1) color:vec4<f32>, @location(2) world:vec3<f32>)->Vertex {
    var output:Vertex; output.position=position; output.color=color; output.world=world; return output;
}
@fragment fn fs_main(input:Vertex)->@location(0) vec4<f32> {
    let depth=dot(clip.plane.xyz,input.world);
    if depth<clip.interval.x || depth>clip.interval.y { discard; }
    if clip.interval.z>0.5 && (any(input.world<clip.minimum.xyz) || any(input.world>clip.maximum.xyz)) { discard; }
    return input.color;
}
