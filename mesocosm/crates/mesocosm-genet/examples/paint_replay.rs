// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Replay a captured real document without laying out or shaping it again.
//! Usage: paint_replay PACKET.paintlist CAPTURE.png REPLAY.png
//! Prints a JSON receipt. An empty/invalid font palette, missing glyph face,
//! changed positioned glyphs, or external GPU reference is an error.

use paint_list_api::{PaintCmd, PaintEnvelope};
use std::{collections::HashMap, error::Error, fs::File, io::BufReader, path::Path};

fn validate(envelope: &PaintEnvelope) -> Result<(usize, usize), Box<dyn Error>> {
    let mut faces = HashMap::new();
    for font in &envelope.fonts {
        let face = ttf_parser::Face::parse(&font.data, font.index)?;
        if faces.insert(font.key, face).is_some() {
            return Err("duplicate font key".into());
        }
    }
    let mut runs = 0;
    let mut glyphs = 0;
    for command in &envelope.commands {
        if let PaintCmd::DrawText(run) = command {
            let face = faces
                .get(&run.font_instance)
                .ok_or("missing font resource")?;
            if !run.font_size.is_finite() || run.font_size <= 0.0 {
                return Err("invalid font size".into());
            }
            for glyph in &run.glyphs {
                if glyph.index >= u32::from(face.number_of_glyphs())
                    || !glyph.point.x.is_finite()
                    || !glyph.point.y.is_finite()
                {
                    return Err("invalid positioned glyph".into());
                }
            }
            runs += 1;
            glyphs += run.glyphs.len();
        }
    }
    if runs == 0 || glyphs == 0 || faces.is_empty() {
        return Err("capture must contain real text and fonts".into());
    }
    Ok((runs, glyphs))
}

fn read_png(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn Error>> {
    let mut reader = png::Decoder::new(BufReader::new(File::open(path)?)).read_info()?;
    let mut pixels = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut pixels)?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err("reference must be RGBA8".into());
    }
    pixels.truncate(info.buffer_size());
    Ok((info.width, info.height, pixels))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: paint_replay PACKET.paintlist CAPTURE.png REPLAY.png".into());
    }
    let bytes = std::fs::read(&args[0])?;
    let envelope: PaintEnvelope = postcard::from_bytes(&bytes)?;
    if postcard::to_allocvec(&envelope)? != bytes {
        return Err("paint packet failed lossless postcard round trip".into());
    }
    let (text_runs, glyph_count) = validate(&envelope)?;
    // Positive control: a palette-free packet must be rejected before booting
    // a GPU, even though all of its positioned glyph commands remain intact.
    let mut broken = envelope.clone();
    broken.fonts.clear();
    if validate(&broken).is_ok() {
        return Err("missing-font control unexpectedly passed".into());
    }
    let translated = paint_list_render::translate_envelope_with_external_textures(&envelope);
    if !translated.external_textures.is_empty() {
        return Err("external GPU texture references need their producer capture".into());
    }
    let scene = &translated.scene;
    let source_runs: Vec<_> = envelope
        .commands
        .iter()
        .filter_map(|cmd| {
            if let PaintCmd::DrawText(run) = cmd {
                Some(run)
            } else {
                None
            }
        })
        .collect();
    let replay_runs: Vec<_> = scene.iter_glyph_runs().collect();
    if source_runs.len() != replay_runs.len() {
        return Err("text run count changed".into());
    }
    for (source, replay) in source_runs.iter().zip(&replay_runs) {
        if source.font_size != replay.font_size || source.glyphs.len() != replay.glyphs.len() {
            return Err("shaped run changed during translation".into());
        }
        let original = envelope
            .fonts
            .iter()
            .find(|font| font.key == source.font_instance)
            .ok_or("missing source font")?;
        let replay_font = scene
            .fonts
            .get(replay.font_id as usize)
            .ok_or("missing translated font")?;
        if original.index != replay_font.index
            || original.data.as_slice() != replay_font.data.as_ref()
        {
            return Err("font bytes or collection index changed".into());
        }
        for (source, replay) in source.glyphs.iter().zip(&replay.glyphs) {
            if source.index != replay.id || source.point.x != replay.x || source.point.y != replay.y
            {
                return Err("positioned glyph changed during translation".into());
            }
        }
    }
    let (width, height, reference) = read_png(Path::new(&args[1]))?;
    if scene.viewport_width == 0 || scene.viewport_height == 0 {
        return Err("empty viewport".into());
    }
    let scale = width as f32 / scene.viewport_width as f32;
    if (height as f32 - scene.viewport_height as f32 * scale).abs() > scale {
        return Err("reference dimensions disagree with captured viewport".into());
    }
    let handles = netrender::boot()?;
    let adapter = handles.adapter.get_info();
    let renderer = netrender::create_netrender_instance(
        handles.clone(),
        netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(256),
            ..Default::default()
        },
    )
    .map_err(|error| format!("renderer initialization: {error:?}"))?;
    for mask in &translated.box_shadow_masks {
        renderer.build_box_shadow_mask(
            mask.key,
            mask.dim,
            mask.bounds,
            mask.corner_radius,
            mask.blur_radius_px,
            mask.invert,
        );
    }
    let texture = handles.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("real panel paint replay"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    renderer.render_vello_scaled(
        scene,
        &texture.create_view(&Default::default()),
        netrender::ColorLoad::Clear(wgpu::Color::BLACK),
        scale,
    );
    let readback = netrender::WgpuDevice::with_external(handles)
        .map_err(|error| format!("readback device features: {error:?}"))?;
    let pixels = readback.read_rgba8_texture(&texture, width, height);
    if pixels.len() != reference.len() {
        return Err("readback size mismatch".into());
    }
    let changed_pixels = pixels
        .chunks_exact(4)
        .zip(reference.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    let max_channel_delta = pixels
        .iter()
        .zip(&reference)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    let output = File::options()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    let mut encoder = png::Encoder::new(output, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "kind": "real-panel-paint-replay", "packet": args[0].to_string_lossy(),
            "adapter": { "name": adapter.name, "backend": format!("{:?}", adapter.backend),
                "device_type": format!("{:?}", adapter.device_type), "driver": adapter.driver,
                "driver_info": adapter.driver_info },
            "packet_bytes": bytes.len(), "commands": envelope.commands.len(),
            "fonts": envelope.fonts.len(), "font_bytes": envelope.fonts.iter().map(|f| f.data.len()).sum::<usize>(),
            "text_runs": text_runs, "glyphs": glyph_count, "images": envelope.images.len(),
            "external_textures": translated.external_textures.len(), "box_shadow_masks": translated.box_shadow_masks.len(),
            "viewport": [scene.viewport_width, scene.viewport_height], "pixels": [width, height], "scale": scale,
            "lossless_round_trip": true, "font_and_glyph_identity": true, "missing_font_control_rejected": true,
            "changed_pixels": changed_pixels, "max_channel_delta": max_channel_delta
        }))?
    );
    if max_channel_delta > 1 {
        return Err("replay differs from paired PNG by more than one channel value".into());
    }
    Ok(())
}
