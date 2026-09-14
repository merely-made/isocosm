// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent arithmetic for a product's authored viewport and the decoded
//! PNG checks over it. This deliberately does not call Genet's
//! transform/content-box implementation: a check that reuses the implementation
//! it is checking proves nothing.

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use cambium_rootstock::Frame;
use serde::Serialize;

use crate::lane::Capture;

/// The CSS transform a product authored on its viewport element, restated here
/// so the mask can be inverted without asking the engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportTransform {
    /// Rotation in degrees, as authored.
    pub rotate_deg: f32,
    /// Uniform scale, as authored.
    pub scale: f32,
    /// Transform origin as a fraction of the border box, `[x, y]`.
    pub origin: [f32; 2],
}

/// A product's rendered viewport as a pixel mask: where it sits, how device
/// pixels map onto it, an optional overlay rectangle to exclude, and the
/// transform (if any) to invert.
///
/// Serializes as `border`, `transformed`, `pixel_scale` and `overlay` — the
/// transform detail and the inset are mask arithmetic, not receipt facts.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Viewport {
    border: [f32; 4],
    transformed: bool,
    pixel_scale: f32,
    overlay: Option<[f32; 4]>,
    #[serde(skip)]
    transform: Option<ViewportTransform>,
    #[serde(skip)]
    inset: f32,
}

impl Viewport {
    /// A mask over the logical `border` box `[x, y, w, h]`, at `pixel_scale`
    /// device pixels per logical unit, discarding `inset` logical units at
    /// every edge (a product's own border, padding and antialiased rim).
    pub fn new(border: [f32; 4], pixel_scale: f32, inset: f32) -> Self {
        Self {
            border,
            transformed: false,
            pixel_scale,
            overlay: None,
            transform: None,
            inset,
        }
    }

    /// Exclude a rectangle painted over the viewport (a floating control).
    #[must_use]
    pub fn with_overlay(mut self, overlay: Option<[f32; 4]>) -> Self {
        self.overlay = overlay;
        self
    }

    /// Declare the CSS transform on the viewport element, so mask tests and
    /// click targeting can map through it.
    #[must_use]
    pub fn with_transform(mut self, transform: Option<ViewportTransform>) -> Self {
        self.transformed = transform.is_some();
        self.transform = transform;
        self
    }

    /// Map a logical point through the declared transform, or its inverse.
    pub fn map(&self, point: (f32, f32), inverse: bool) -> (f32, f32) {
        let Some(transform) = self.transform else {
            return point;
        };
        let [x, y, width, height] = self.border;
        let origin = (
            x + width * transform.origin[0],
            y + height * transform.origin[1],
        );
        let angle = transform.rotate_deg.to_radians() * if inverse { -1.0 } else { 1.0 };
        let scale = if inverse {
            1.0 / transform.scale
        } else {
            transform.scale
        };
        let (sin, cos) = angle.sin_cos();
        let (dx, dy) = (point.0 - origin.0, point.1 - origin.1);
        (
            origin.0 + scale * (cos * dx - sin * dy),
            origin.1 + scale * (sin * dx + cos * dy),
        )
    }

    /// Whether a device pixel is inside the masked viewport interior.
    pub fn contains(&self, x: u32, y: u32) -> bool {
        let point = (
            (x as f32 + 0.5) / self.pixel_scale,
            (y as f32 + 0.5) / self.pixel_scale,
        );
        if self.overlay.is_some_and(|[x, y, w, h]| {
            point.0 >= x - 1.0
                && point.0 <= x + w + 1.0
                && point.1 >= y - 1.0
                && point.1 <= y + h + 1.0
        }) {
            return false;
        }
        let point = self.map(point, true);
        let [x, y, width, height] = self.border;
        let inset = self.inset;
        point.0 >= x + inset
            && point.0 < x + width - inset
            && point.1 >= y + inset
            && point.1 < y + height - inset
    }
}

/// One recorded pixel comparison between two completed captures.
#[derive(Debug, Serialize)]
pub struct PixelCheck {
    operation: String,
    first: String,
    second: String,
    sampled_pixels: usize,
    changed_pixels: usize,
    background_rgb: [u8; 3],
    background_pixels: usize,
    changed_background_pixels: usize,
}

/// Compare two captures inside the viewport mask they share.
///
/// `body-pixels-change` demands a substantial change that leaves the modal
/// background colour untouched (the subject moved, the scene did not);
/// `viewport-pixels-change` demands any substantial change;
/// `viewport-pixels-same` demands none.
pub fn compare(
    captures: &[Capture],
    operation: &str,
    first: &str,
    second: &str,
) -> Result<PixelCheck, String> {
    let capture = |name: &str| {
        captures
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| format!("capture {name} has not completed"))
    };
    let (a, b) = (capture(first)?, capture(second)?);
    if a.width != b.width || a.height != b.height || a.viewport != b.viewport {
        return Err(format!(
            "{operation}: {first}/{second} do not share viewport geometry"
        ));
    }
    let viewport = a
        .viewport
        .ok_or("pixel check requires a visible specimen viewport")?;
    let (a, b) = (read_png(&a.path)?, read_png(&b.path)?);
    if (a.width, a.height) != (b.width, b.height) {
        return Err("decoded captures have different dimensions".into());
    }
    let mut colors = BTreeMap::<[u8; 3], usize>::new();
    let mut sampled = Vec::new();
    for y in 0..a.height {
        for x in 0..a.width {
            if !viewport.contains(x, y) {
                continue;
            }
            let i = (y as usize * a.width as usize + x as usize) * 4;
            let color = [a.rgba[i], a.rgba[i + 1], a.rgba[i + 2]];
            *colors.entry(color).or_default() += 1;
            sampled.push((i, color));
        }
    }
    let (&background, &background_pixels) = colors
        .iter()
        .max_by_key(|(_, count)| **count)
        .ok_or("viewport pixel mask contains no presented pixels")?;
    let mut check = PixelCheck {
        operation: operation.into(),
        first: first.into(),
        second: second.into(),
        sampled_pixels: sampled.len(),
        changed_pixels: 0,
        background_rgb: background,
        background_pixels,
        changed_background_pixels: 0,
    };
    for (i, color) in sampled {
        if (0..3).any(|channel| a.rgba[i + channel].abs_diff(b.rgba[i + channel]) > 2) {
            check.changed_pixels += 1;
            if color == background {
                check.changed_background_pixels += 1;
            }
        }
    }
    let valid = match operation {
        "body-pixels-change" => {
            check.changed_pixels >= 32
                && background_pixels >= check.sampled_pixels / 4
                && check.changed_background_pixels == 0
        },
        "viewport-pixels-change" => check.changed_pixels >= 32,
        "viewport-pixels-same" => check.changed_pixels == 0,
        _ => return Err(format!("unknown pixel operation {operation}")),
    };
    if !valid {
        return Err(format!(
            "{operation} {first}/{second}: {} of {} pixels changed, {} of {} background pixels changed",
            check.changed_pixels,
            check.sampled_pixels,
            check.changed_background_pixels,
            background_pixels
        ));
    }
    Ok(check)
}

/// Create a path's parent directory chain, if it has one.
pub fn create_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Write a read-back frame as RGBA8 PNG, creating its parent directory.
pub fn write_png(path: &Path, frame: &Frame) -> Result<(), String> {
    create_parent(path)?;
    let file = BufWriter::new(File::create(path).map_err(|e| e.to_string())?);
    let mut encoder = png::Encoder::new(file, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer
        .write_image_data(&frame.rgba)
        .map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

fn read_png(path: &Path) -> Result<Frame, String> {
    let file = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
    let mut reader = png::Decoder::new(file)
        .read_info()
        .map_err(|e| e.to_string())?;
    let mut rgba = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut rgba).map_err(|e| e.to_string())?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err("capture is not RGBA8".into());
    }
    rgba.truncate(info.buffer_size());
    Ok(Frame {
        width: info.width,
        height: info.height,
        rgba,
    })
}
