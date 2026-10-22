// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use glam::{Mat4, Vec3};
use isometer_core::{Attachment, BodyDocument, VolumeRef, Yaw};
use isometer_mesh::{Volume, VolumeMap, mesh_body};

fn clip() -> ClipSlab {
    ClipSlab::new([0., 0., 1.], -100., 100.)
}
fn block(centre: Vec3, part: u32) -> Primitive {
    Primitive::rounded_box(
        PartId(part),
        centre,
        Vec3::ONE,
        Quat::IDENTITY,
        0.3,
        [0.7, 0.3, 0.1],
    )
}
#[test]
fn field_picks_rounded_silhouette_and_blended_join_with_part_identity() {
    let surface = Surface {
        parts: vec![block(Vec3::ZERO, 2), block(Vec3::X * 2., 7)],
        blend: 0.5,
    };
    let ray = surface
        .pick([0., 0., -5.], [0., 0., 1.], 20., clip())
        .unwrap()
        .unwrap();
    assert_eq!(ray.part, PartId(2));
    assert!((ray.distance - 4.).abs() < 0.025);
    assert!(
        surface
            .pick([-0.95, 0.95, -5.], [0., 0., 1.], 20., clip())
            .unwrap()
            .is_none()
    );
    assert!(surface.sample([1., 0., 0.]).0 < 0.);
    assert!(surface.pick([0., 0., -5.], [0.; 3], 20., clip()).is_err());
    let bounds = surface.bounds().unwrap();
    assert!(bounds.0[0] < -1. && bounds.1[0] > 3.);
    let slab = ClipSlab::new([0., 0., 1.], -10., -2.);
    assert!(
        surface
            .pick([0., 0., -5.], [0., 0., 1.], 20., slab)
            .unwrap()
            .is_none()
    );
}
#[test]
fn inverse_kinematics_preserves_lengths_and_clamps_unreachable_contacts() {
    for target in [Vec3::new(0., -3., 0.), Vec3::new(0., -30., 0.), Vec3::ZERO] {
        let (knee, foot) = two_bone(Vec3::ZERO, target, Vec3::X, 2., 2.);
        assert!((knee.length() - 2.).abs() < 0.001);
        assert!(((foot - knee).length() - 2.).abs() < 0.001);
        assert!(foot.length() <= 4. && knee.is_finite() && foot.is_finite());
    }
}
#[test]
fn anatomy_uses_actual_attachment_frames_ground_contacts_and_surviving_parts() {
    let root = VolumeRef::from_tag(1);
    let leg = VolumeRef::from_tag(2);
    let mut document = BodyDocument::new(root, [2, 2, 2]);
    let id = document
        .attach(
            leg,
            [1, 4, 1],
            Attachment {
                parent: document.root,
                offset: [2, -5, 0],
                yaw: Yaw::Quarter,
            },
            None,
        )
        .unwrap();
    let mut source = VolumeMap::new();
    source.insert(root, Volume::solid([4, 4, 4], 1));
    source.insert(leg, Volume::solid([2, 8, 2], 2));
    let original = document.clone();
    let mesh = mesh_body(&document, &source).unwrap();
    let mut live = crate::LiveBody::new(&mesh, [20., 10., 30.]);
    live.yaw_radians = 0.7;
    let (surface, stats) = from_anatomy(&document, live, |_| Some(2.)).unwrap();
    assert_eq!(stats.joints, 1);
    assert_eq!(stats.contacts, 1);
    assert_eq!(document, original);
    let leg_parts: Vec<_> = surface
        .parts
        .iter()
        .filter(|p| p.meta[1] == id.0 && p.meta[0] == 1)
        .collect();
    assert_eq!(leg_parts.len(), 3); // two links and its declared attachment neck
    // The actual links keep their soles above the supplied solid surface.
    // Conservative selection bounds also include the union's padding.
    for link in &leg_parts[..2] {
        assert!(link.a[1].min(link.b[1]) - link.a[3] >= 1.999);
    }
    document.parts[id.0 as usize].severed = true;
    let mesh = mesh_body(&document, &source).unwrap();
    let (surface, stats) = from_anatomy(&document, crate::LiveBody::new(&mesh, [0.; 3]), |_| {
        Some(0.)
    })
    .unwrap();
    assert_eq!(stats.joints, 0);
    assert!(surface.parts.iter().all(|p| p.meta[1] != id.0));
}

#[test]
fn generated_cube_support_uses_its_graph_edge_while_the_upper_cube_stays_rigid() {
    let root = VolumeRef::from_tag(1);
    let mut document = BodyDocument::new(root, [2; 3]);
    document
        .attach(
            root,
            [2; 3],
            Attachment {
                parent: document.root,
                offset: [0, -4, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
    document
        .attach(
            root,
            [2; 3],
            Attachment {
                parent: document.root,
                offset: [0, 4, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
    let mut source = VolumeMap::new();
    source.insert(root, Volume::solid([4; 3], 1));
    let mesh = mesh_body(&document, &source).unwrap();
    let (surface, stats) =
        from_anatomy(&document, crate::LiveBody::new(&mesh, [0., 6., 0.]), |_| {
            Some(0.)
        })
        .unwrap();
    assert_eq!((stats.joints, stats.contacts), (1, 1));
    assert!(
        surface
            .parts
            .iter()
            .any(|p| p.meta[1] == 2 && p.meta[0] == 0)
    );
    let (hanging, stats) =
        from_anatomy(&document, crate::LiveBody::new(&mesh, [0., 6., 0.]), |_| {
            Some(-10.)
        })
        .unwrap();
    assert_eq!(stats.contacts, 0);
    for p in hanging
        .parts
        .iter()
        .filter(|p| p.meta[1] == 1 && p.meta[0] == 1)
    {
        assert!(
            p.a[1].min(p.b[1]) - p.a[3] >= -0.001,
            "a short limb must not stretch toward an unreachable floor"
        );
    }
}

#[test]
fn rounded_skin_preserves_the_coarse_box_corner_that_actually_supports_it() {
    let root = VolumeRef::from_tag(1);
    let document = BodyDocument::new(root, [2; 3]);
    let mut source = VolumeMap::new();
    source.insert(root, Volume::solid([4; 3], 1));
    let mesh = mesh_body(&document, &source).unwrap();
    let before = Surface {
        parts: vec![Primitive::rounded_box(
            PartId(0),
            Vec3::new(0., 2., 0.),
            Vec3::splat(2.),
            Quat::IDENTITY,
            1.6,
            [1.; 3],
        )],
        blend: 0.,
    };
    assert!(before.sample([-1.5, 0., -1.5]).0 > 0.1);
    let (surface, stats) =
        from_anatomy(&document, crate::LiveBody::new(&mesh, [0., 2., 0.]), |p| {
            Some(if p[0] < -1. && p[2] < -1. { 0. } else { -5. })
        })
        .unwrap();
    assert_eq!(stats.supports, 1);
    assert!(surface.sample([-1.5, 0., -1.5]).0 <= 0.015);
    assert!(surface.parts.iter().all(|p| p.meta[1] == 0));
}

#[test]
fn gpu_surface_agrees_with_picker_and_respects_terrain_depth() {
    let gpu =
        crate::Renderer::headless(48, 48).expect("implicit GPU qualification requires an adapter");
    let device = gpu.device();
    let queue = gpu.queue();
    let size = wgpu::Extent3d {
        width: 48,
        height: 48,
        depth_or_array_layers: 1,
    };
    let make = |format, usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("implicit test"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let colour = make(
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let terrain = make(
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let renderer = Renderer::new(device, wgpu::TextureFormat::Rgba8UnormSrgb, [48, 48]);
    let surface = Surface {
        parts: vec![block(Vec3::ZERO, 4)],
        blend: 0.2,
    };
    let matrix = Mat4::from_cols_array_2d(&[
        [0.25, 0., 0., 0.],
        [0., 0.25, 0., 0.],
        [0., 0., 0.1, 0.],
        [0., 0., 0.5, 1.],
    ]);
    for (depth, visible) in [(1., true), (0.2, false)] {
        let mut encoder = device.create_command_encoder(&Default::default());
        let view = terrain.create_view(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terrain depth fixture"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(depth),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        renderer
            .encode(
                device,
                queue,
                &mut encoder,
                &colour,
                &terrain,
                Frame {
                    clip_from_world: matrix.to_cols_array_2d(),
                    clip: clip(),
                    toward_sun: [0., 1., 0.],
                    ambient: 1.,
                    sun: [0.; 3],
                },
                &[&surface],
            )
            .unwrap();
        let image = gpu.read_back(encoder, &colour).unwrap();
        assert_eq!(image.pixel(24, 24)[3] > 0, visible);
        if visible {
            // Centre pixel's ray is exactly the orthographic camera ray.
            let p = matrix
                .inverse()
                .project_point3(Vec3::new(1. / 48., -1. / 48., 0.));
            assert_eq!(
                surface
                    .pick(p.to_array(), [0., 0., 1.], 10., clip())
                    .unwrap()
                    .unwrap()
                    .part,
                PartId(4)
            );
            assert_eq!(image.pixel(0, 0)[3], 0);
        }
    }
}
