//! Replacement experiment only. Production retains capacity-fixed allocation.

#[path = "growth_batched.rs"]
mod batched;

use super::*;
use crate::{AtlasLimits, BrickProjectionRevision};
use std::time::Instant;

fn extent(size: [u32; 3]) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: size[2],
    }
}

fn texture(device: &wgpu::Device, size: [u32; 3]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("atlas replacement experiment"),
        size: extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn info(texture: &wgpu::Texture, origin: [u32; 3]) -> wgpu::TexelCopyTextureInfo<'_> {
    wgpu::TexelCopyTextureInfo {
        texture,
        mip_level: 0,
        origin: wgpu::Origin3d {
            x: origin[0],
            y: origin[1],
            z: origin[2],
        },
        aspect: wgpu::TextureAspect::All,
    }
}

fn wait(device: &wgpu::Device, queue: &wgpu::Queue) {
    queue.submit([]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
}

fn read(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: [u32; 3],
) -> Vec<u8> {
    let pitch = size[0].div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("replacement readback"),
        size: u64::from(pitch) * u64::from(size[1]) * u64::from(size[2]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        info(texture, [0; 3]),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(size[1]),
            },
        },
        extent(size),
    );
    queue.submit([encoder.finish()]);
    let (send, receive) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            send.send(result).unwrap()
        });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receive.recv().unwrap().unwrap();
    let bytes = buffer.slice(..).get_mapped_range().unwrap();
    let out = bytes
        .chunks_exact(pitch as usize)
        .flat_map(|row| row[..size[0] as usize].iter().copied())
        .collect();
    drop(bytes);
    buffer.unmap();
    out
}

fn slot(map: &BrickMap, key: [i16; 3]) -> u32 {
    let origin = map.origin();
    map.pointer_at([0, 1, 2].map(|axis| (key[axis] - origin[axis]) as u32))
        .unwrap()
}

fn fixture(
    old_count: usize,
    new_count: usize,
    rows: [usize; 2],
    limits: AtlasLimits,
) -> (BrickMap, BrickMap, Vec<u32>, Vec<u32>) {
    let keys: Vec<_> = (0..new_count)
        .map(|i| [(i / 80) as i16, 0, (i % 80) as i16])
        .collect();
    let soil = [3u8; 512];
    let changed = [9u8; 512];
    let mut old = BrickMap::with_limits(
        BrickProjectionRevision(0),
        rows[0] * 256 - 1,
        [80, 1, 80],
        limits,
    )
    .unwrap();
    old.retarget(
        BrickProjectionRevision(1),
        keys[..old_count].iter().copied(),
        |_| Some(&soil[..]),
    )
    .unwrap();
    let mut new = BrickMap::with_limits(
        BrickProjectionRevision(2),
        rows[1] * 256 - 1,
        [80, 1, 80],
        limits,
    )
    .unwrap();
    new.retarget(
        BrickProjectionRevision(3),
        keys[..old_count].iter().copied(),
        |_| Some(&soil[..]),
    )
    .unwrap();
    new.retarget(BrickProjectionRevision(4), keys.iter().copied(), |_| {
        Some(&soil[..])
    })
    .unwrap();
    new.refresh([keys[0]], |_| Some(&changed[..])).unwrap();
    let retained: Vec<_> = keys[1..old_count]
        .iter()
        .map(|&key| {
            assert_eq!(slot(&old, key), slot(&new, key), "stable slot coordinates");
            slot(&new, key)
        })
        .collect();
    let patches = std::iter::once(slot(&new, keys[0]))
        .chain(keys[old_count..].iter().map(|&key| slot(&new, key)))
        .collect();
    (old, new, retained, patches)
}

#[derive(Clone, Copy, Debug)]
enum Path {
    Full,
    Bulk,
    Slots,
    MissingCopy,
    MissingPatch,
}

struct Sample {
    texture: wgpu::Texture,
    create_us: f64,
    total_us: f64,
    copied: u64,
    uploaded: u64,
    copy_calls: u32,
    write_calls: u32,
}

fn replace(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    old_texture: &wgpu::Texture,
    old: &BrickMap,
    new: &BrickMap,
    retained: &[u32],
    patches: &[u32],
    path: Path,
) -> Sample {
    let start = Instant::now();
    let target = texture(device, new.atlas_extent());
    let create_us = start.elapsed().as_secs_f64() * 1e6;
    let (mut copied, mut copy_calls) = (0, 0);
    if matches!(path, Path::Bulk | Path::Slots | Path::MissingPatch) {
        let mut encoder = device.create_command_encoder(&Default::default());
        if matches!(path, Path::Slots) {
            let [sx, _, sz] = old.slots();
            for &slot in retained {
                let index = slot - 1;
                let origin = [index % sx * 8, index / (sx * sz) * 8, (index / sx) % sz * 8];
                encoder.copy_texture_to_texture(
                    info(old_texture, origin),
                    info(&target, origin),
                    extent([8; 3]),
                );
            }
            copied = retained.len() as u64 * 512;
            copy_calls = retained.len() as u32;
        } else {
            encoder.copy_texture_to_texture(
                info(old_texture, [0; 3]),
                info(&target, [0; 3]),
                extent(old.atlas_extent()),
            );
            copied = old.atlas().len() as u64;
            copy_calls = 1;
        }
        // Submit first: queue writes in the next submission must follow the
        // old copy, or it would overwrite the patch to an existing slot.
        queue.submit([encoder.finish()]);
    }
    let (uploaded, write_calls) = if matches!(path, Path::Full) {
        write_texture_3d(queue, &target, [0; 3], new.atlas_extent(), 1, new.atlas());
        (new.atlas().len() as u64, 1)
    } else if matches!(path, Path::MissingPatch) {
        (0, 0)
    } else {
        write_atlas_slot_boxes(queue, &target, new, patches)
    };
    wait(device, queue);
    Sample {
        texture: target,
        create_us,
        total_us: start.elapsed().as_secs_f64() * 1e6,
        copied,
        uploaded,
        copy_calls,
        write_calls,
    }
}

fn differences(actual: &[u8], expected: &[u8]) -> usize {
    assert_eq!(actual.len(), expected.len());
    actual.iter().zip(expected).filter(|(a, b)| a != b).count()
}

#[test]
#[ignore = "manual replacement benchmark requires a real GPU"]
fn atlas_replacement_copy_experiment() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect("a real adapter");
    eprintln!("[atlas-replacement] adapter {:?}", adapter.get_info());
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Default::default())).expect("a real device");
    let limits = AtlasLimits {
        max_texture_dimension_3d: device.limits().max_texture_dimension_3d,
        max_atlas_bytes: 8 * 1024 * 1024,
    };
    eprintln!(
        "[atlas-replacement] device max3D {} budget {}",
        limits.max_texture_dimension_3d, limits.max_atlas_bytes
    );
    for (old_count, new_count, rows) in [(260, 700, [2, 4]), (2504, 5000, [10, 24])] {
        let (old, new, retained, patches) = fixture(old_count, new_count, rows, limits);
        let old_texture = texture(&device, old.atlas_extent());
        write_texture_3d(
            &queue,
            &old_texture,
            [0; 3],
            old.atlas_extent(),
            1,
            old.atlas(),
        );
        wait(&device, &queue);
        assert_eq!(
            differences(
                &read(&device, &queue, &old_texture, old.atlas_extent()),
                old.atlas()
            ),
            0
        );
        eprintln!(
            "[atlas-replacement] case {old_count}->{new_count} extent {:?}->{:?} peak_texture_payload {} pointer_bytes {} unchanged across variants",
            old.atlas_extent(),
            new.atlas_extent(),
            old.atlas().len() + new.atlas().len(),
            new.pointers().len() * 4
        );
        for path in [Path::MissingCopy, Path::MissingPatch] {
            let sample = replace(
                &device,
                &queue,
                &old_texture,
                &old,
                &new,
                &retained,
                &patches,
                path,
            );
            let diff = differences(
                &read(&device, &queue, &sample.texture, new.atlas_extent()),
                new.atlas(),
            );
            assert!(diff > 0, "same-run {path:?} control must fail equality");
            eprintln!("[atlas-replacement] control {path:?} differing_texels {diff}");
        }
        let paths = [Path::Full, Path::Bulk, Path::Slots];
        let mut elapsed = [Vec::new(), Vec::new(), Vec::new()];
        for round in 0..12 {
            for offset in 0..3 {
                let index = (round + offset) % 3;
                let path = paths[index];
                let sample = replace(
                    &device,
                    &queue,
                    &old_texture,
                    &old,
                    &new,
                    &retained,
                    &patches,
                    path,
                );
                assert_eq!(
                    differences(
                        &read(&device, &queue, &sample.texture, new.atlas_extent()),
                        new.atlas()
                    ),
                    0,
                    "{path:?}"
                );
                eprintln!(
                    "[atlas-replacement] sample case={old_count}-{new_count} round={round} path={path:?} create_us={:.3} total_us={:.3} gpu_copy_bytes={} cpu_upload_bytes={} copy_calls={} write_calls={} submits={}",
                    sample.create_us,
                    sample.total_us,
                    sample.copied,
                    sample.uploaded,
                    sample.copy_calls,
                    sample.write_calls,
                    if matches!(path, Path::Full) { 1 } else { 2 }
                );
                if round > 1 {
                    elapsed[index].push(sample.total_us);
                }
            }
        }
        for (path, mut times) in paths.into_iter().zip(elapsed) {
            times.sort_by(f64::total_cmp);
            eprintln!(
                "[atlas-replacement] summary case={old_count}-{new_count} path={path:?} measured_samples={} median_total_us={:.3} p95_total_us={:.3}",
                times.len(),
                (times[4] + times[5]) / 2.0,
                times[9]
            );
        }
    }
}
