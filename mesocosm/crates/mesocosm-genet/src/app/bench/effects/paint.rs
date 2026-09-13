// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::effect_experiment::{Experiment, Mark, Receiver};
use sprigging::{ColorF, Leaf, PaintCx, Path, Size, SizeHint, round_stroke};

#[derive(Default)]
pub(super) struct EffectLeaf {
    marks: Vec<Mark>,
    receiver: Option<Receiver>,
    identity: String,
    dirty: bool,
}
impl EffectLeaf {
    pub fn set(&mut self, experiment: &Experiment, tick: u16) {
        let identity = format!(
            "{}:{tick}",
            serde_json::to_string(&experiment.request).unwrap()
        );
        if self.identity != identity {
            self.identity = identity;
            self.marks = experiment.sample(tick);
            self.receiver = Some(experiment.request.receiver);
            self.dirty = true;
        }
    }
}
fn color(r: f32, g: f32, b: f32, a: f32) -> ColorF {
    ColorF { r, g, b, a }
}
impl Leaf for EffectLeaf {
    fn measure(&mut self, _: SizeHint, _: SizeHint) -> Size {
        Size {
            width: 800.,
            height: 280.,
        }
    }
    fn paint_dirty(&self) -> bool {
        self.dirty
    }
    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let size = cx.size();
        cx.push_clip_rect(0., 0., size.width, size.height);
        cx.fill_rect(
            0.,
            0.,
            size.width,
            size.height,
            color(0.055, 0.09, 0.10, 1.),
        );
        let sx = size.width / 1000.;
        let sy = size.height / 1000.;
        let place = |x: f32, y: f32| (x * sx, y * sy);
        for n in 1..10 {
            let x = n as f32 * 100. * sx;
            cx.stroke_path(
                Path::new().move_to(x, 0.).line_to(x, size.height).build(),
                round_stroke(color(0.17, 0.24, 0.24, 0.5), 1.),
            );
        }
        let (x, y) = place(700., 500.);
        let receiver_color = match self.receiver {
            Some(Receiver::Stone) => color(0.50, 0.52, 0.55, 1.),
            Some(Receiver::Metal) => color(0.65, 0.72, 0.79, 1.),
            _ => color(0.32, 0.55, 0.31, 1.),
        };
        // Same normalized receiver bounds as core's contact fixture. The
        // ellipse on a wide viewport is the plane's affine projection.
        let mut ring = Path::new();
        for i in 0..=48 {
            let angle = i as f32 * std::f32::consts::TAU / 48.;
            let p = place(700. + 100. * angle.cos(), 500. + 100. * angle.sin());
            ring = if i == 0 {
                ring.move_to(p.0, p.1)
            } else {
                ring.line_to(p.0, p.1)
            };
        }
        cx.fill_path(
            ring.clone().close().build(),
            color(receiver_color.r, receiver_color.g, receiver_color.b, 0.25),
        );
        cx.stroke_path(ring.build(), round_stroke(receiver_color, 2.));
        cx.fill_rect(x - 2., y - 2., 4., 4., receiver_color);
        // Only core-admitted group relationships connect marks. Shared
        // visual proximity alone must never invent a semantic string.
        let mut previous = std::collections::BTreeMap::new();
        for mark in &self.marks {
            let point = place(mark.x as f32, mark.y as f32);
            if let Some(group) = mark.group {
                if let Some((px, py)) = previous.insert(group, point) {
                    cx.stroke_path(
                        Path::new()
                            .move_to(px, py)
                            .line_to(point.0, point.1)
                            .build(),
                        round_stroke(color(0.48, 0.81, 0.73, 0.35), 1.5),
                    );
                }
            }
        }
        for mark in &self.marks {
            let (x, y) = place(mark.x as f32, mark.y as f32);
            let a = mark.angle as f32 * std::f32::consts::PI / 180.;
            let scale = mark.scale as f32 / 1000. * 12.;
            let transform = |dx: f32, dy: f32| {
                (
                    x + scale * (dx * a.cos() - dy * a.sin()),
                    y + scale * (dx * a.sin() + dy * a.cos()),
                )
            };
            let strokes: &[&[(f32, f32)]] = match mark.glyph {
                '/' => &[&[(-0.45, 0.8), (0.45, -0.8)]],
                '`' => &[&[(-0.25, -0.7), (0.25, -0.2)]],
                '\'' => &[&[(-0.1, -0.75), (0.05, -0.35), (-0.2, -0.05)]],
                _ => &[
                    &[(-0.4, -0.75), (-0.25, -0.35), (-0.5, -0.05)],
                    &[(0.35, -0.75), (0.5, -0.35), (0.25, -0.05)],
                ],
            };
            for stroke in strokes {
                let mut path = Path::new();
                for (i, &(dx, dy)) in stroke.iter().enumerate() {
                    let (px, py) = transform(dx, dy);
                    path = if i == 0 {
                        path.move_to(px, py)
                    } else {
                        path.line_to(px, py)
                    };
                }
                cx.stroke_path(
                    path.build(),
                    round_stroke(color(0.87, 0.98, 0.77, mark.opacity as f32 / 255.), 2.5),
                );
            }
        }
        cx.pop_clip();
        self.dirty = false;
    }
}
