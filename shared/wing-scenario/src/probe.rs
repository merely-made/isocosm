// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The [`Automatable`]/[`Driveable`] implementation the scenario drives: one
//! retained surface, host-owned selector geometry, pointer delivery through the
//! host's own routing, and the shared verbs (`remember`,
//! `same`/`differs`/`more`/`dropped`, `capture`s comparison, pixel checks,
//! `zoom`, `opacity`, `resize`, `input-text`, `cost-begin`/`cost-end`).

use cambium_rootstock::{HostPointer, WindowCommand};
use genet_probe::{
    Automatable, Driveable, Hit, ProbeSnapshot, ProbeSurface, Selector, SelectorTarget,
};

use crate::{Checkpoints, Ctx, Lane, Product, pixels};

pub(crate) struct Probe<'a, 'c, P: Product> {
    pub(crate) ctx: &'a mut Ctx<'c, P>,
    pub(crate) lane: &'a mut Lane<P>,
}

impl<P: Product> Automatable for Probe<'_, '_, P> {
    fn with_surfaces<R>(&self, f: impl FnOnce(&[ProbeSurface<'_>]) -> R) -> R {
        let dom = self.ctx.runner.dom();
        let dom = dom.borrow();
        let (width, height) = self.ctx.logical_size;
        f(&[ProbeSurface {
            name: P::SURFACE,
            dom: &dom,
            rect: [0.0, 0.0, width, height],
            sheet: self.lane.product.sheet(),
        }])
    }

    fn selector_target(&self, selector: &Selector) -> SelectorTarget {
        let nodes = genet_probe::matching(&self.ctx.runner.dom().borrow(), selector);
        for node in nodes {
            if let Some((x, y, width, height)) = self.ctx.painted_rect(node)
                && width > 0.0
                && height > 0.0
            {
                return SelectorTarget::Hit(Hit {
                    surface: P::SURFACE,
                    point: self
                        .lane
                        .product
                        .target_point(self.ctx, node, [x, y, width, height]),
                });
            }
        }
        self.lane
            .misses
            .borrow_mut()
            .push(format!("no current DOM target for {selector:?}"));
        SelectorTarget::Miss
    }

    fn snapshot(&self) -> ProbeSnapshot {
        self.lane
            .product
            .snapshot(self.ctx, self.lane.captures.len(), self.lane.opacity)
    }

    fn drain_events(&mut self) -> Vec<String> {
        self.lane.product.drain_events(self.ctx)
    }

    fn act(&mut self, label: &str) -> bool {
        self.lane.product.act(self.ctx, label)
    }

    fn press(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Press(x, y));
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Moved(x, y));
    }

    fn release(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Release(x, y));
    }

    fn busy(&mut self) -> Option<bool> {
        let pending = self.lane.capture_pending();
        self.lane.product.busy(self.ctx, pending)
    }
}

impl<P: Product> Driveable for Probe<'_, '_, P> {
    fn capture(&mut self, name: &str) -> bool {
        let path = self.lane.named_capture(name);
        match self.lane.arm_capture(self.ctx, name.to_owned(), path) {
            Ok(()) => true,
            Err(why) => {
                self.lane.errors.push(why);
                false
            },
        }
    }

    fn app_step(&mut self, line: &str) -> Result<(), String> {
        let words: Vec<_> = line.split_whitespace().collect();
        match words.as_slice() {
            ["cost-begin", name] => self.lane.costs.begin(name)?,
            ["cost-end"] => self.lane.costs.end()?,
            ["input-text", value] => {
                let mut select = cambium::KeyEvent::new(cambium::Key::Character("a".into()));
                select.mods.ctrl = true;
                self.ctx.runner.dispatch_key(select);
                self.ctx
                    .runner
                    .dispatch_key(cambium::KeyEvent::new(cambium::Key::Character(
                        (*value).into(),
                    )));
            },
            ["resize", width, height] => {
                let dimension = |value: &str| {
                    value
                        .parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite() && *v >= 1.0 && *v <= 16_384.0)
                        .ok_or_else(|| format!("invalid native size {value}"))
                };
                self.ctx
                    .window_commands
                    .push(WindowCommand::Resize(dimension(width)?, dimension(height)?));
            },
            ["remember", name] => {
                let fields = self.snapshot().fields;
                self.lane.checkpoints.insert((*name).into(), fields);
            },
            ["zoom", value] => {
                let zoom = value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && (0.5..=2.0).contains(v))
                    .ok_or_else(|| format!("invalid receipt zoom {value}"))?;
                *self.ctx.set_ui_zoom = Some(zoom);
            },
            ["opacity", value] => {
                let opacity = value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
                    .ok_or_else(|| format!("invalid receipt opacity {value}"))?;
                self.lane.opacity = opacity;
                if let Some(sheet) = self.lane.product.opacity_sheet(opacity) {
                    *self.ctx.set_sheet = Some(sheet);
                }
            },
            [
                verb @ ("body-pixels-change" | "viewport-pixels-change" | "viewport-pixels-same"),
                first,
                second,
            ] => {
                let check = pixels::compare(&self.lane.captures, verb, first, second)?;
                self.lane.pixel_checks.push(check);
            },
            [
                verb @ ("same" | "differs" | "more" | "dropped"),
                name,
                fields @ ..,
            ] if !fields.is_empty() => {
                let now = self.snapshot();
                Checkpoints(&self.lane.checkpoints).compare(verb, name, fields, &now)?;
            },
            [
                verb @ ("captures-differ" | "capture-size-changed"),
                first,
                second,
            ] => {
                let capture = |name: &str| {
                    self.lane
                        .captures
                        .iter()
                        .find(|c| c.name == name)
                        .ok_or_else(|| format!("capture {name} has not completed"))
                };
                let (a, b) = (capture(first)?, capture(second)?);
                let changed = if *verb == "captures-differ" {
                    a.digest != b.digest
                } else {
                    (a.width, a.height) != (b.width, b.height)
                };
                if !changed {
                    return Err(format!("{verb}: {first} and {second} agree"));
                }
            },
            _ => {
                // Disjoint fields: the product hook is handed the lane's own
                // checkpoints rather than keeping a second set of its own.
                let Lane {
                    product,
                    checkpoints,
                    ..
                } = &mut *self.lane;
                return product.app_step(self.ctx, Checkpoints(checkpoints), line);
            },
        }
        Ok(())
    }
}
