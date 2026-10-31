//! `ISOMETRY_LOOK_SELFTEST=preview|board`: a headed look-editor run.
//!
//! Equips the first token with a sword, opens its look editor (the menu row's
//! own call), then picks a class and the hood (the core pack's .vox layer)
//! through the painted buttons, a shirt colour, and one quarter turn.
//! `preview` stops there, editor open; `board` saves, faces the token the way
//! the preview was turned, and closes, so the two runs' captures are the
//! creator's preview and the same token on the board.
use super::*;
use cambium_genet_winit_host::ScrollAlign;
use isometry_campaign::EquipmentSlot;

pub(crate) struct LookSelftest {
    save: bool,
    step: usize,
    scrolled: bool,
    turned: usize,
    started: Instant,
}

impl LookSelftest {
    pub(crate) fn from_env() -> Option<Self> {
        let mode = std::env::var("ISOMETRY_LOOK_SELFTEST").ok()?;
        assert!(["preview", "board"].contains(&mode.as_str()), "look selftest mode is preview or board");
        Some(Self {
            save: mode == "board",
            step: 0,
            scrolled: false,
            turned: 0,
            started: Instant::now(),
        })
    }

    pub(crate) fn done(&self) -> bool {
        self.step >= 7
    }

    pub(crate) fn drive(&mut self, ctx: &mut Ctx<'_>) {
        if self.done() || self.started.elapsed() < Duration::from_secs(2) {
            return;
        }
        assert!(self.started.elapsed() < Duration::from_secs(60), "look selftest stalled at {}", self.step);
        let advanced = match self.step {
            0 => {
                ctx.runner.update(|ui| {
                    let id = ui.map.tokens[0].id;
                    let inventory = ui.inventories.entry(id).or_default();
                    let item = ItemId::new("look-selftest.sword");
                    let _ = inventory.insert(ItemInstance {
                        id: item.clone(),
                        template: "srd5e:longsword".to_owned(),
                        name: "Longsword".to_owned(),
                        quantity: 1,
                        tags: vec!["weapon".to_owned()],
                        modifiers: Vec::new(),
                        appearance_layers: Vec::new(),
                    });
                    let _ = inventory.equip(EquipmentSlot::MainHand, item);
                    ui.open_look(id);
                });
                ctx.runner.state().look_open
            },
            1 => self.click(ctx, "traveler"),
            2 => self.click(ctx, "hood"),
            3 => {
                ctx.runner.update(|ui| ui.set_look_colour(1, [0xd8, 0xb0, 0x4a]));
                true
            },
            4 => {
                // `ISOMETRY_LOOK_TURNS` quarter turns (one by default).
                let turns: usize = std::env::var("ISOMETRY_LOOK_TURNS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1);
                if self.turned < turns {
                    if self.click(ctx, "turn") {
                        self.turned += 1;
                    }
                    false
                } else {
                    true
                }
            },
            5 => {
                if self.save && self.click(ctx, "save") {
                    // Face the token the way the preview was turned, as a
                    // player would after saving, so the pair compares.
                    ctx.runner.update(|ui| {
                        let facing = ui.look.facing;
                        let id = ui.map.tokens[0].id;
                        let _ = apply(&mut ui.map, &SessionEvent::TokenFaced { id, facing });
                    });
                    true
                } else {
                    !self.save
                }
            },
            _ => {
                let ui = ctx.runner.state();
                let token = &ui.map.tokens[0];
                eprintln!(
                    "[isometry] look selftest: {} as {} with {:?}, editor open {}, draft {:?}",
                    token.id.0,
                    token.sprite,
                    ui.map.appearances.get(&token.id),
                    ui.look_open,
                    ui.look
                );
                true
            },
        };
        if advanced {
            self.step += 1;
        }
        // The board parks when still; ask for the frame the next step (or the
        // capture, once done) needs.
        if let Some(window) = ctx.window {
            window.request_redraw();
        }
    }

    fn click(&mut self, ctx: &mut Ctx<'_>, label: &str) -> bool {
        let node = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), "btn").into_iter().find(|node| {
                dom.dom_children(*node).filter_map(|child| dom.text(child)).collect::<String>() == label
            })
        };
        let Some(node) = node else {
            return false;
        };
        if !self.scrolled {
            ctx.scroll_into_view(node, ScrollAlign::Nearest);
            self.scrolled = true;
            return false;
        }
        let Some((x, y, w, h)) = ctx.painted_rect(node) else {
            return false;
        };
        let (x, y) = (x + w / 2.0, y + h / 2.0);
        ctx.pointer.push(HostPointer::Moved(x, y));
        ctx.pointer.push(HostPointer::Press(x, y));
        ctx.pointer.push(HostPointer::Release(x, y));
        self.scrolled = false;
        true
    }
}
