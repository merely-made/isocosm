// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
struct Frame {
    clip: mat4x4<f32>, inverse: mat4x4<f32>, slab: vec4<f32>, limits: vec4<f32>,
    bounds_min: vec4<f32>, bounds_max: vec4<f32>, light: vec4<f32>, sun: vec4<f32>,
}
struct Primitive {
    a:vec4<f32>, b:vec4<f32>, half:vec4<f32>, rotation:vec4<f32>, colour:vec4<f32>, tag:vec4<u32>,
}
struct Draw { min:vec4<f32>, max:vec4<f32>, range:vec4<u32>, rect:vec4<f32> }
@group(0) @binding(0) var<uniform> frame:Frame;
@group(0) @binding(1) var terrain:texture_depth_2d;
@group(0) @binding(2) var<storage,read> parts:array<Primitive>;
@group(0) @binding(3) var<storage,read> draws:array<Draw>;
struct Vertex { @builtin(position) position:vec4<f32>, @location(0) ndc:vec2<f32>, @location(1) @interpolate(flat) body:u32 }
@vertex fn vs_main(@builtin(vertex_index) index:u32,@builtin(instance_index) body:u32)->Vertex {
    let uv=array<vec2<f32>,6>(vec2(0.,0.),vec2(1.,0.),vec2(0.,1.),vec2(0.,1.),vec2(1.,0.),vec2(1.,1.));
    let rect=draws[body].rect;
    let ndc=mix(rect.xy,rect.zw,uv[index]);
    return Vertex(vec4(ndc,0.,1.),ndc,body);
}
fn rotate(q:vec4<f32>,p:vec3<f32>)->vec3<f32> {return p+2.*cross(q.xyz,cross(q.xyz,p)+q.w*p);}
fn distance(p:vec3<f32>,part:Primitive)->f32 {
    if part.tag.x==1u {
        let ab=part.b.xyz-part.a.xyz;
        let t=clamp(dot(p-part.a.xyz,ab)/max(dot(ab,ab),1e-12),0.,1.);
        return length(p-part.a.xyz-ab*t)-part.a.w;
    }
    let local=rotate(vec4(-part.rotation.xyz,part.rotation.w),p-part.a.xyz);
    let q=abs(local)-part.half.xyz+part.half.w;
    return length(max(q,vec3(0.)))+min(max(q.x,max(q.y,q.z)),0.)-part.half.w;
}
// Distance plus nearest contributor. The same order and tie rule is used by
// Surface::sample on the CPU; colour blends locally with the distance field.
struct Sample { d:f32, colour:vec3<f32> }
fn field(p:vec3<f32>,body:Draw)->Sample {
    var d=1e20; var colour=vec3(0.);
    for(var i=0u;i<body.range.y;i++) {
        let part=parts[body.range.x+i]; let next=distance(p,part);
        if i==0u || body.min.w<=0. {
            if next<d {d=next;colour=part.colour.rgb;}
        } else {
            let h=clamp(0.5+0.5*(next-d)/body.min.w,0.,1.);
            d=mix(next,d,h)-body.min.w*h*(1.-h);
            colour=mix(part.colour.rgb,colour,h);
        }
    }
    return Sample(d,colour);
}
fn visible(p:vec3<f32>)->bool {
    let slab=dot(frame.slab.xyz,p);
    return slab>=frame.slab.w && slab<=frame.limits.x &&
        (frame.bounds_min.w<0.5 || (all(p>=frame.bounds_min.xyz) && all(p<=frame.bounds_max.xyz)));
}
struct Fragment { @location(0) colour:vec4<f32>, @builtin(frag_depth) depth:f32 }
@fragment fn fs_main(input:Vertex)->Fragment {
    let body=draws[input.body];
    let origin=(frame.inverse*vec4(input.ndc,0.,1.)).xyz;
    let end=(frame.inverse*vec4(input.ndc,1.,1.)).xyz;
    let direction=normalize(end-origin);
    var start=0.; var finish=length(end-origin);
    let terrain_depth=textureLoad(terrain,vec2<i32>(input.position.xy),0);
    finish=min(finish,terrain_depth*length(end-origin)+0.0001);
    for(var axis=0u;axis<3u;axis++) {
        if abs(direction[axis])<1e-8 {
            if origin[axis]<body.min[axis] || origin[axis]>body.max[axis] {discard;}
        } else {
            let a=(body.min[axis]-origin[axis])/direction[axis];
            let b=(body.max[axis]-origin[axis])/direction[axis];
            start=max(start,min(a,b));finish=min(finish,max(a,b));
        }
    }
    let at=dot(frame.slab.xyz,origin);let slope=dot(frame.slab.xyz,direction);
    if abs(slope)<1e-8 {
        if at<frame.slab.w || at>frame.limits.x {discard;}
    } else {
        let a=(frame.slab.w-at)/slope;let b=(frame.limits.x-at)/slope;
        start=max(start,min(a,b));finish=min(finish,max(a,b));
    }
    if frame.bounds_min.w>0.5 {
        for(var axis=0u;axis<3u;axis++) {
            if abs(direction[axis])<1e-8 {
                if origin[axis]<frame.bounds_min[axis] || origin[axis]>frame.bounds_max[axis] {discard;}
            } else {
                let a=(frame.bounds_min[axis]-origin[axis])/direction[axis];
                let b=(frame.bounds_max[axis]-origin[axis])/direction[axis];
                start=max(start,min(a,b));finish=min(finish,max(a,b));
            }
        }
    }
    if finish<start {discard;}
    var t=start; var hit=false; var p=origin; var sample=Sample(0.,vec3(0.));
    for(var step=0u;step<128u;step++) {
        p=origin+direction*t; sample=field(p,body);
        if abs(sample.d)<0.015 && visible(p) {hit=true;break;}
        t+=max(abs(sample.d),0.008)*0.8;
        if t>finish {break;}
    }
    if !hit {discard;}
    let depth=(frame.clip*vec4(p,1.)).z;
    if depth>terrain_depth+0.000001 || depth<0. || depth>1. {discard;}
    let e=0.025;
    let normal=normalize(vec3(field(p+vec3(e,0.,0.),body).d-field(p-vec3(e,0.,0.),body).d,
        field(p+vec3(0.,e,0.),body).d-field(p-vec3(0.,e,0.),body).d,
        field(p+vec3(0.,0.,e),body).d-field(p-vec3(0.,0.,e),body).d));
    let lighting=vec3(frame.light.w)+frame.sun.rgb*max(dot(normal,normalize(frame.light.xyz)),0.);
    return Fragment(vec4(sample.colour*lighting,1.),depth);
}
