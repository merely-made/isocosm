// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Cambium's graph canvas supplies painting, hit targets and pointer capture.
//! The draft is an attachment diagram. Specimen nodes use Isometer's real
//! joint positions and the same dimetric projection as the game.
use super::{Bench, Child, Page, Panel};
use cambium::{
    GraphCanvasEdge, GraphCanvasEvent, GraphCanvasNode, GraphCanvasSubgraph, GraphCanvasSwatch,
    PointerPhase, graph_canvas,
};
use cosmograph::{rules::Facing, schema::Entity};

pub(in crate::app::bench) const LEAF: u64 = 0x4d435f424f4459;
pub(in crate::app::bench) type Diagram = GraphCanvasSwatch<usize, bool>;

struct Node {
    id: usize,
    parent: Option<usize>,
    at: [f32; 3],
    label: String,
    tissue: bool,
}

fn axes(f: Facing) -> [f32; 3] {
    let (i, sign) = f.axis();
    let mut axis = [0.; 3];
    axis[i] = sign as f32;
    axis
}

fn flat(at: [f32; 3]) -> [f32; 2] {
    let camera = isometer::SlabCamera::dimetric_2_1([0.; 3], 1., 1., 128.).unwrap();
    let [x, y] = camera.ndc_of(at).unwrap();
    [x, -y]
}

impl Panel {
    pub(super) fn specimen(&self) -> Option<(u64, &Entity)> {
        let population = &self.session.sim.state().population;
        if let Some(id) = self.specimen_id {
            return population.get(id).map(|e| (id, e));
        }
        population
            .groups
            .iter()
            .find(|(_, g)| {
                g.entity.alive
                    && g.entity.body.is_some()
                    && g.entity.resident_site() == Some(self.site)
            })
            .map(|(&id, g)| (id, g.entity.as_ref()))
    }

    pub(super) fn next_specimen(&mut self) {
        let current = self.specimen().map(|(id, _)| id);
        let ids = || {
            self.session
                .sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(_, g)| {
                    g.entity.alive
                        && g.entity.body.is_some()
                        && g.entity.resident_site() == Some(self.site)
                })
                .map(|(&id, _)| id)
        };
        self.specimen_id = ids()
            .find(|id| Some(*id) > current)
            .or_else(|| ids().next());
        self.part_id = None;
    }

    pub(in crate::app::bench) fn diagram(&self) -> Option<Diagram> {
        let (title, selected, points) = match self.page {
            Page::World => return None,
            Page::Body => {
                let map = self.map()?;
                let mut points: Vec<Node> = Vec::new();
                for (i, run) in map.grammar.seed.iter().enumerate() {
                    let parent = run.parent.map(usize::from).or_else(|| i.checked_sub(1));
                    let mut at = [0.; 3];
                    if let Some(p) = parent {
                        at = points.get(p)?.at;
                        let direction = axes(run.facing);
                        for axis in 0..3 {
                            at[axis] += direction[axis] * f32::from(run.segments.max(1));
                        }
                    }
                    points.push(Node {
                        id: i,
                        parent,
                        at,
                        label: format!("Run {}", i + 1),
                        tissue: run.bears.is_some(),
                    });
                }
                ("Growth attachments", Some(self.run), points)
            },
            Page::Watch => {
                let (_, e) = self.specimen()?;
                let body = e.body.as_ref()?;
                let points = body
                    .parts
                    .iter()
                    .filter_map(|p| {
                        Some(Node {
                            id: p.id.0 as usize,
                            parent: p.attachment.map(|a| a.parent.0 as usize),
                            at: body.world_pivot(p.id)?.map(|n| n as f32),
                            label: format!("Part {} · {}", p.id.0, p.shape),
                            tissue: e.parts.contains_key(&p.id),
                        })
                    })
                    .collect();
                ("Developed specimen", self.part_id, points)
            },
        };
        if points.is_empty() {
            return None;
        }
        let coords: Vec<_> = points.iter().map(|n| flat(n.at)).collect();
        let bounds = coords.iter().fold(
            ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]),
            |(mut lo, mut hi), xy| {
                for k in 0..2 {
                    lo[k] = lo[k].min(xy[k]);
                    hi[k] = hi[k].max(xy[k]);
                }
                (lo, hi)
            },
        );
        let (lo, hi) = bounds;
        let scale = (350. / (hi[0] - lo[0]).max(1.)).min(95. / (hi[1] - lo[1]).max(1.));
        let mut graph = GraphCanvasSubgraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        for (n, xy) in points.into_iter().zip(coords) {
            let position = (
                0.5 + (xy[0] - (lo[0] + hi[0]) * 0.5) * scale / 420.,
                0.5 + (xy[1] - (lo[1] + hi[1]) * 0.5) * scale / 150.,
            );
            graph.nodes.push(GraphCanvasNode {
                id: n.id,
                kind: n.tissue,
                position,
                label: n.label,
                key: Some(format!("joint-{}", n.id)),
            });
            if let Some(from) = n.parent {
                graph.edges.push(GraphCanvasEdge { from, to: n.id });
            }
        }
        let mut swatch = Diagram::new(LEAF, graph).with_size(420, 150);
        swatch.label = title.into();
        swatch.selected = selected;
        swatch.show_labels = self.page == Page::Body;
        swatch.show_expand = false;
        swatch.hit_size = 24.;
        Some(swatch)
    }
}

pub(super) fn view(p: &Panel) -> Option<Child> {
    let diagram = p.diagram()?;
    Some(Box::new(graph_canvas(&diagram, |b: &mut Bench, event| {
        let Some(p) = &mut b.founding else { return };
        match event {
            GraphCanvasEvent::Activate(id) if p.page == Page::Body => {
                p.run = id;
                p.bank_page = 0;
            },
            GraphCanvasEvent::Activate(id) => p.part_id = Some(id),
            GraphCanvasEvent::Drag(drag)
                if p.page == Page::Body && drag.id > 0 && drag.phase == PointerPhase::Up =>
            {
                let Some(scene) = p.diagram() else { return };
                let Some(edge) = scene.graph.edges.iter().find(|e| e.to == drag.id) else {
                    return;
                };
                let here = scene.graph.nodes[drag.id].position;
                if (drag.position.0 - here.0).hypot(drag.position.1 - here.1) < 0.035 {
                    return;
                }
                let origin = scene.graph.nodes[edge.from].position;
                let offset = (
                    (drag.position.0 - origin.0) * 420.,
                    (drag.position.1 - origin.1) * 150.,
                );
                if offset.0.hypot(offset.1) < 5. {
                    return;
                }
                let alignment = |f| {
                    let [x, y] = flat(axes(f));
                    (offset.0 * x + offset.1 * y) / x.hypot(y)
                };
                if let Some(f) = Facing::ALL
                    .into_iter()
                    .max_by(|a, b| alignment(*a).total_cmp(&alignment(*b)))
                {
                    p.map_mut().grammar.seed[drag.id].facing = f;
                    p.run = drag.id;
                }
            },
            _ => {},
        }
    })))
}
