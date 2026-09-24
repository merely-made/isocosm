// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The text-first netrender surface this host draws its receipt on.
//!
//! Moved here from `eponym_client::body_sheet` when the body sheet was
//! retired (M5 of the isomere plan). The body sheet was a second GUI toolkit
//! drawn straight into a netrender scene; all of it went except these two
//! pieces, which the timed-action host is the only remaining consumer of and
//! which are not GUI machinery in the sense that died: a parley label and a
//! list of lines on a panel. Nothing was rewritten — `Text` and `timed_scene`
//! are the code that stood in `body_sheet/text.rs` and `body_sheet.rs`.
//!
//! This host presents through netrender directly rather than through the
//! shared `isomere` host, which is why it keeps a font loader of its own. It
//! is the text receipt until the session host covers everything it supports;
//! when that happens this module retires with it, and the remaining loader in
//! `crossing::draw` is the last one in the crate.

use netrender::Scene;
use netrender_text::parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, Layout, LayoutContext, StyleProperty,
    fontique,
};
use std::sync::Arc;

pub(crate) const LOGICAL_SIZE: [u32; 2] = [1280, 720];

struct Text {
    font: FontContext,
    layouts: LayoutContext<[f32; 4]>,
    family: String,
}

impl Text {
    fn new() -> Result<Self, String> {
        let custom = std::env::var("PAREDROS_FONT").ok();
        let paths = custom.iter().map(String::as_str).chain([
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]);
        for path in paths {
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let mut font = FontContext::new();
            let Some((id, _)) = font
                .collection
                .register_fonts(fontique::Blob::new(Arc::new(bytes)), None)
                .into_iter()
                .next()
            else {
                continue;
            };
            let Some(family) = font.collection.family_name(id).map(str::to_owned) else {
                continue;
            };
            return Ok(Self {
                font,
                layouts: LayoutContext::new(),
                family,
            });
        }
        Err("Timed action needs a readable local font.".into())
    }

    fn label(
        &mut self,
        scene: &mut Scene,
        text: &str,
        xy: [f32; 2],
        size: f32,
        color: [f32; 4],
        width: f32,
    ) {
        let mut builder = self.layouts.ranged_builder(&mut self.font, text, 1., true);
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::Brush(color));
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(&self.family)));
        let mut layout: Layout<[f32; 4]> = builder.build(text);
        layout.break_all_lines(Some(width));
        layout.align(Alignment::Start, AlignmentOptions::default());
        netrender_text::push_layout(scene, &layout, xy);
    }
}

pub(crate) struct Hud {
    text: Text,
}

impl Hud {
    pub(crate) fn new() -> Result<Self, String> {
        Ok(Self { text: Text::new()? })
    }

    /// The whole surface: a title and one line per reading, on one panel.
    pub(crate) fn timed_scene(&mut self, title: &str, lines: &[String]) -> Scene {
        let mut scene = Scene::new(LOGICAL_SIZE[0], LOGICAL_SIZE[1]);
        scene.push_rect(0., 0., 1280., 720., [0.035, 0.05, 0.065, 1.]);
        scene.push_rect(24., 20., 1256., 680., [0.025, 0.04, 0.055, 0.96]);
        let ink = [0.91, 0.95, 0.96, 1.];
        self.text
            .label(&mut scene, title, [48., 48.], 28., ink, 1160.);
        for (index, line) in lines.iter().enumerate() {
            self.text.label(
                &mut scene,
                line,
                [54., 112. + index as f32 * 31.],
                18.,
                if index == 0 {
                    [0.95, 0.78, 0.42, 1.]
                } else {
                    ink
                },
                1140.,
            );
        }
        scene
    }
}
