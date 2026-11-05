@group(0) @binding(0) var live_colour: texture_2d<f32>;
@group(0) @binding(1) var live_depth: texture_depth_2d;
@group(0) @binding(2) var sprite_colour: texture_2d<f32>;
@group(0) @binding(3) var sprite_depth: texture_depth_2d;
@group(0) @binding(4) var<uniform> weight: vec4<u32>;
@vertex fn vs(@builtin(vertex_index) i:u32)->@builtin(position) vec4<f32> {
    let p=array<vec2<f32>,3>(vec2(-1.0,-1.0),vec2(3.0,-1.0),vec2(-1.0,3.0));
    return vec4(p[i],0.0,1.0);
}
struct Out { @location(0) colour:vec4<f32>, @builtin(frag_depth) depth:f32 }
@fragment fn fs(@builtin(position) p:vec4<f32>)->Out {
    let at=vec2<i32>(p.xy);
    let bayer=array<u32,16>(0,8,2,10,12,4,14,6,3,11,1,9,15,7,13,5);
    let cached=bayer[u32(at.y%4)*4+u32(at.x%4)]<weight.x;
    var out:Out;
    if cached {out.colour=textureLoad(sprite_colour,at,0);out.depth=textureLoad(sprite_depth,at,0);}
    else {out.colour=textureLoad(live_colour,at,0);out.depth=textureLoad(live_depth,at,0);}
    return out;
}
