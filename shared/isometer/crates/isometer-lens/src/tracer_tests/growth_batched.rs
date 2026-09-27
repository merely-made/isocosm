//! Ruling 373: pack patch boxes, then encode the retained copy and patches
//! in one submission. The original experiment remains unchanged.

use super::*;
use wgpu::util::DeviceExt;

struct PatchBox {
    origin: [u32; 3],
    size: [u32; 3],
    offset: u64,
    pitch: u32,
}

/// Same consecutive-slot boxes as write_atlas_slot_boxes, explicitly packed
/// for buffer copies. D3 rows are 256-byte aligned, including their padding.
fn pack(map: &BrickMap, slots: &[u32]) -> (Vec<u8>, Vec<PatchBox>) {
    let [sx, _, sz] = map.slots();
    let [width, height, _] = map.atlas_extent();
    let mut sorted = slots.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut total = 0usize;
    let mut boxes = Vec::new();
    let mut push = |origin: [u32; 3], size: [u32; 3]| {
        let origin = origin.map(|axis| axis * 8);
        let size = size.map(|axis| axis * 8);
        let pitch = size[0].div_ceil(256) * 256;
        let offset = total;
        total += (pitch * size[1] * size[2]) as usize;
        boxes.push(PatchBox {
            origin,
            size,
            offset: offset as u64,
            pitch,
        });
    };
    let mut runs = sorted.iter().peekable();
    while let Some(&start) = runs.next() {
        let mut end = start;
        while runs.peek().is_some_and(|&&next| next == end + 1) {
            end = *runs.next().unwrap();
        }
        let mut at = start - 1;
        let last = end - 1;
        while at <= last {
            let x = at % sx;
            let z = (at / sx) % sz;
            let y = at / (sx * sz);
            let remaining = last - at + 1;
            if x == 0 && remaining >= sx {
                let rows = (remaining / sx).min(sz - z);
                push([0, y, z], [sx, 1, rows]);
                at += rows * sx;
            } else {
                let run = (sx - x).min(remaining);
                push([x, y, z], [run, 1, 1]);
                at += run;
            }
        }
    }
    let mut bytes = vec![0; total];
    for part in &boxes {
        for z in 0..part.size[2] {
            for y in 0..part.size[1] {
                let from = (((part.origin[2] + z) * height + part.origin[1] + y) * width
                    + part.origin[0]) as usize;
                let to = part.offset as usize + ((z * part.size[1] + y) * part.pitch) as usize;
                bytes[to..to + part.size[0] as usize]
                    .copy_from_slice(&map.atlas()[from..from + part.size[0] as usize]);
            }
        }
    }
    (bytes, boxes)
}

struct Batched {
    sample: Sample,
    pack_us: f64,
    staging_us: f64,
    padded_bytes: usize,
}

fn batched(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    old_texture: &wgpu::Texture,
    old: &BrickMap,
    new: &BrickMap,
    patches: &[u32],
    copy_old: bool,
    patch: bool,
    swap: Option<[u32; 2]>,
) -> Batched {
    let start = Instant::now();
    let target = texture(device, new.atlas_extent());
    let create_us = start.elapsed().as_secs_f64() * 1e6;
    let packing = Instant::now();
    let (bytes, boxes) = pack(new, patches);
    let pack_us = packing.elapsed().as_secs_f64() * 1e6;
    let staging = Instant::now();
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("packed atlas patch staging"),
        contents: &bytes,
        usage: wgpu::BufferUsages::COPY_SRC,
    });
    let staging_us = staging.elapsed().as_secs_f64() * 1e6;
    let mut encoder = device.create_command_encoder(&Default::default());
    if copy_old {
        encoder.copy_texture_to_texture(
            info(old_texture, [0; 3]),
            info(&target, [0; 3]),
            extent(old.atlas_extent()),
        );
    }
    if let Some([first, second]) = swap {
        let [sx, _, sz] = old.slots();
        let origin = |slot: u32| {
            let i = slot - 1;
            [i % sx * 8, i / (sx * sz) * 8, (i / sx) % sz * 8]
        };
        for (source, destination) in [(first, second), (second, first)] {
            encoder.copy_texture_to_texture(
                info(old_texture, origin(source)),
                info(&target, origin(destination)),
                extent([8; 3]),
            );
        }
    }
    if patch {
        for part in &boxes {
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: part.offset,
                        bytes_per_row: Some(part.pitch),
                        rows_per_image: Some(part.size[1]),
                    },
                },
                info(&target, part.origin),
                extent(part.size),
            );
        }
    }
    queue.submit([encoder.finish()]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    Batched {
        sample: Sample {
            texture: target,
            create_us,
            total_us: start.elapsed().as_secs_f64() * 1e6,
            copied: if copy_old {
                old.atlas().len() as u64
            } else {
                0
            },
            uploaded: if patch { patches.len() as u64 * 512 } else { 0 },
            copy_calls: u32::from(copy_old),
            write_calls: if patch { boxes.len() as u32 } else { 0 },
        },
        pack_us,
        staging_us,
        padded_bytes: bytes.len(),
    }
}

/// Distinct per-voxel/per-key data catches wrong boxes or slot permutations
/// that uniform soil could hide. Map construction stays outside all timers.
fn patterned_fixture(
    old_count: usize,
    new_count: usize,
    rows: [usize; 2],
    limits: AtlasLimits,
) -> (BrickMap, BrickMap, Vec<u32>, Vec<u32>) {
    let (mut old, mut new, retained, patches) = fixture(old_count, new_count, rows, limits);
    let keys: Vec<_> = (0..new_count)
        .map(|i| [(i / 80) as i16, 0, (i % 80) as i16])
        .collect();
    let values: Vec<[u8; 512]> = (0..new_count)
        .map(|i| {
            std::array::from_fn(|v| {
                let mut hash =
                    (i as u32 + 1).wrapping_mul(0x9e37_79b9) ^ (v as u32).wrapping_mul(0x85eb_ca6b);
                hash ^= hash >> 16;
                hash = hash.wrapping_mul(0x7feb_352d);
                hash ^= hash >> 15;
                ((hash % 255) + 1) as u8
            })
        })
        .collect();
    assert_eq!(
        values
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        new_count,
        "every brick has distinct content"
    );
    let index = |key: [i16; 3]| key[0] as usize * 80 + key[2] as usize;
    old.refresh(keys[..old_count].iter().copied(), |key| {
        Some(&values[index(key)][..])
    })
    .unwrap();
    new.refresh(keys.iter().copied(), |key| Some(&values[index(key)][..]))
        .unwrap();
    let changed = values[0].map(|value| ((u32::from(value) + 10) % 255 + 1) as u8);
    new.refresh([keys[0]], |_| Some(&changed[..])).unwrap();
    (old, new, retained, patches)
}

#[test]
#[ignore = "manual batched replacement benchmark requires a real GPU"]
fn atlas_replacement_batched_experiment() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect("a real adapter");
    assert_ne!(
        adapter.get_info().device_type,
        wgpu::DeviceType::Cpu,
        "hardware GPU required"
    );
    eprintln!("[atlas-batched] adapter {:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let limits = AtlasLimits {
        max_texture_dimension_3d: device.limits().max_texture_dimension_3d,
        max_atlas_bytes: 8 * 1024 * 1024,
    };
    eprintln!(
        "[atlas-batched] max3D={} budget={} packing/new-buffer/new-texture/encoding/completion included; CPU map/pointer/bind-group/render/readback excluded",
        limits.max_texture_dimension_3d, limits.max_atlas_bytes
    );
    for (old_count, new_count, rows) in [(260, 700, [2, 4]), (2504, 5000, [10, 24])] {
        let (old, new, retained, patches) = patterned_fixture(old_count, new_count, rows, limits);
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
        let atlas_peak = old.atlas().len() + new.atlas().len();
        eprintln!(
            "[atlas-batched] case={old_count}-{new_count} extents={:?}->{:?} peak_atlas_payload={atlas_peak} pointer_bytes={}",
            old.atlas_extent(),
            new.atlas_extent(),
            new.pointers().len() * 4
        );
        for (name, copy, patch, swap) in [
            ("missing-copy", false, true, None),
            ("missing-patch", true, false, None),
            (
                "swapped-retained-bricks",
                true,
                true,
                Some([retained[0], retained[1]]),
            ),
        ] {
            let actual = batched(
                &device,
                &queue,
                &old_texture,
                &old,
                &new,
                &patches,
                copy,
                patch,
                swap,
            );
            let diff = differences(
                &read(&device, &queue, &actual.sample.texture, new.atlas_extent()),
                new.atlas(),
            );
            assert!(diff > 0, "{name} must detect broken content");
            eprintln!(
                "[atlas-batched] control case={old_count}-{new_count} {name} differing_texels={diff}"
            );
        }
        let mut times = [Vec::new(), Vec::new(), Vec::new()];
        let names = ["Full", "BulkTwoSubmits", "BatchedOneSubmit"];
        for round in 0..33 {
            for offset in 0..3 {
                let variant = (round + offset) % 3;
                let (sample, pack_us, staging_us, padded) = if variant == 2 {
                    let measured = batched(
                        &device,
                        &queue,
                        &old_texture,
                        &old,
                        &new,
                        &patches,
                        true,
                        true,
                        None,
                    );
                    (
                        measured.sample,
                        measured.pack_us,
                        measured.staging_us,
                        measured.padded_bytes,
                    )
                } else {
                    (
                        replace(
                            &device,
                            &queue,
                            &old_texture,
                            &old,
                            &new,
                            &retained,
                            &patches,
                            if variant == 0 { Path::Full } else { Path::Bulk },
                        ),
                        0.0,
                        0.0,
                        0,
                    )
                };
                assert_eq!(
                    differences(
                        &read(&device, &queue, &sample.texture, new.atlas_extent()),
                        new.atlas()
                    ),
                    0,
                    "{}",
                    names[variant]
                );
                if variant == 2 {
                    assert_eq!(sample.uploaded, patches.len() as u64 * 512);
                }
                eprintln!(
                    "[atlas-batched] sample case={old_count}-{new_count} round={round} path={} create_us={:.3} pack_us={pack_us:.3} staging_us={staging_us:.3} total_us={:.3} copy_bytes={} texel_upload_bytes={} known_padded_staging_bytes={padded} copy_calls={} patch_calls={} submits={} known_gpu_payload_with_staging={} known_host_pack_bytes={padded}",
                    names[variant],
                    sample.create_us,
                    sample.total_us,
                    sample.copied,
                    sample.uploaded,
                    sample.copy_calls,
                    sample.write_calls,
                    if variant == 1 { 2 } else { 1 },
                    atlas_peak + padded
                );
                if round >= 3 {
                    times[variant].push(sample.total_us);
                }
            }
        }
        for (name, mut samples) in names.into_iter().zip(times) {
            samples.sort_by(f64::total_cmp);
            eprintln!(
                "[atlas-batched] summary case={old_count}-{new_count} path={name} n={} median_us={:.3} p95_us={:.3} max_us={:.3}",
                samples.len(),
                (samples[14] + samples[15]) / 2.0,
                samples[28],
                samples[29]
            );
        }
    }
}
