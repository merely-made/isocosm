//! Clicking painted endpoint controls the way a person would: find the
//! rendered node, scroll it into view, then press at its painted centre.
//! Shared by the self-tests that drive endpoint cards.
use super::*;
use cambium_genet_winit_host::ScrollAlign;

#[derive(Default)]
pub(super) struct Painted {
    scrolled: Option<(genet_scripted_dom::NodeId, Option<(u64, u64)>)>,
    missed: Option<genet_scripted_dom::NodeId>,
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
        self.click_text(ctx, "vtt-tab", &label);
        false
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
        self.click_node(ctx, node)
    }

    pub(super) fn click_node(
        &mut self,
        ctx: &mut Ctx<'_>,
        node: genet_scripted_dom::NodeId,
    ) -> bool {
        let presentation = ctx.presentation.map(|frame| (frame.host, frame.sequence));
        if self.scrolled.is_none_or(|(previous, _)| previous != node) {
            ctx.scroll_into_view(node, ScrollAlign::Nearest);
            self.scrolled = Some((node, presentation));
            return false;
        }
        // Native pointer routing reads the accepted picture. The retained
        // layout can already know where a control will appear on a later redraw.
        if ctx.window.is_some()
            && (presentation.is_none() || self.scrolled == Some((node, presentation)))
        {
            return false;
        }
        let rect = ctx.visible_rect(node);
        let dom = ctx.runner.dom();
        let dom = dom.borrow();
        let label = dom
            .attribute(node, &Namespace::from(""), &LocalName::from("aria-label"))
            .map(str::to_owned)
            .unwrap_or_else(|| dom.dom_children(node).filter_map(|n| dom.text(n)).collect());
        let point = rect.and_then(|(x, y, w, h)| {
            [(0.5, 0.5), (0.25, 0.5), (0.75, 0.5)]
                .into_iter()
                .find_map(|(u, v)| {
                    let point = (x + w * u, y + h * v);
                    let hit = ctx.painted_hit(point.0, point.1)?;
                    std::iter::successors(Some(hit), |n| dom.parent(*n))
                        .any(|n| n == node)
                        .then_some(point)
                })
        });
        drop(dom);
        let Some((x, y)) = point else {
            if self.missed != Some(node) {
                eprintln!(
                    "[painted] waiting for {label:?} to receive pointer, visible={rect:?}, status={}",
                    ctx.runner.state().status
                );
                self.missed = Some(node);
            }
            ctx.scroll_into_view(node, ScrollAlign::Start);
            return false;
        };
        self.missed = None;
        eprintln!(
            "[painted] click {label:?} at ({x:.1}, {y:.1}), presentation={presentation:?}, status={}",
            ctx.runner.state().status
        );
        ctx.pointer.push(HostPointer::Press(x, y));
        ctx.pointer.push(HostPointer::Release(x, y));
        self.scrolled = None;
        true
    }
}
