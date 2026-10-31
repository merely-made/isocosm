//! Clicking painted endpoint controls the way a person would: find the
//! rendered node, scroll it into view, then press at its painted centre.
//! Shared by the self-tests that drive endpoint cards.
use super::*;
use cambium_genet_winit_host::ScrollAlign;

#[derive(Default)]
pub(super) struct Painted {
    scrolled: bool,
}

impl Painted {
    pub(super) fn tab(&mut self, ctx: &mut Ctx<'_>, id: &str) -> bool {
        if ctx.runner.state().vtt_selected_projection == id {
            return true;
        }
        let label = ctx
            .runner
            .state()
            .vtt_projections
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .label
            .clone();
        self.click_text(ctx, "vtt-tab", &label)
    }
    pub(super) fn button(&mut self, ctx: &mut Ctx<'_>, id: &str, body: Option<u64>) -> bool {
        let selected = &ctx.runner.state().vtt_selected_projection;
        let label = ctx
            .runner
            .state()
            .vtt_projections
            .iter()
            .find(|p| p.id == *selected)
            .and_then(|p| {
                p.cards
                    .iter()
                    .filter(|card| {
                        body.is_none_or(|id| {
                            card.values
                                .iter()
                                .any(|v| v.label == "Native body" && v.value == id.to_string())
                        })
                    })
                    .find_map(|card| {
                        card.actions
                            .iter()
                            .find(|a| a.id == id)
                            .map(|a| a.label.clone())
                    })
            });
        let Some(label) = label else {
            return false;
        };
        // A sheet action can have the same label on two bodies. Find its ordinal
        // in the supplied card order, then the corresponding painted action.
        let all = ctx
            .runner
            .state()
            .vtt_projections
            .iter()
            .find(|p| p.id == *selected)
            .unwrap();
        let matching: Vec<_> = all
            .cards
            .iter()
            .flat_map(|card| {
                card.actions
                    .iter()
                    .filter(|a| a.label == label)
                    .map(move |_| {
                        body.is_none_or(|id| {
                            card.values
                                .iter()
                                .any(|v| v.label == "Native body" && v.value == id.to_string())
                        })
                    })
            })
            .collect();
        let ordinal = matching.iter().position(|matches| *matches).unwrap();
        self.click_ordinal(ctx, "vtt-action", &label, ordinal)
    }
    pub(super) fn click_text(&mut self, ctx: &mut Ctx<'_>, class: &str, label: &str) -> bool {
        self.click_ordinal(ctx, class, label, 0)
    }
    pub(super) fn click_ordinal(
        &mut self,
        ctx: &mut Ctx<'_>,
        class: &str,
        label: &str,
        ordinal: usize,
    ) -> bool {
        let node = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), class)
                .into_iter()
                .filter(|node| {
                    dom.dom_children(*node)
                        .filter_map(|child| dom.text(child))
                        .collect::<String>()
                        == label
                })
                .nth(ordinal)
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
