// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact disclosed display cells, with all paint sharply clipped to them.

use cambium::{GraphCanvasAtlas, GraphCanvasAtlasField, GraphCanvasAtlasPaint};
use isocosm_vtt::native::overmap::{CellReading, OvermapReading, ViewSettings, Color};
use sceno::{Footprint, Rect, Size2, Vec2};
use sprigging::ColorF;
use std::sync::Arc;

pub fn native_overmap_cell_id(cell: &CellReading) -> String {
    if cell.anchor { format!("site:{}", cell.site) }
    else { format!("native-cell:{}:{}:{}", cell.site, cell.at[0], cell.at[1]) }
}

pub fn native_overmap_swatch(read:&OvermapReading)
    -> Option<cambium::GraphCanvasSwatch<String,crate::OvermapNodeKind>> {
    let atlas = native_overmap_atlas(read,None)?;
    let nodes = read.cells.iter().map(|cell| cambium::GraphCanvasNode {
        id:native_overmap_cell_id(cell),kind:crate::OvermapNodeKind::Elsewhere,
        position:atlas.normalize_world(Vec2::new(cell.at[0] as f32+0.5,cell.at[1] as f32+0.5)),
        label:if cell.anchor { cell.label.clone() } else { String::new() },
        key:Some(native_overmap_cell_id(cell)),
    }).collect();
    Some(cambium::GraphCanvasSwatch::new(crate::OVERMAP_LEAF_KEY,
        cambium::GraphCanvasSubgraph { nodes,edges:Vec::new() })
        .with_size(crate::OVERMAP_CANVAS.0,crate::OVERMAP_CANVAS.1)
        .with_label("Disclosed native map").with_expand(false).with_node_labels(true).with_atlas(atlas))
}

pub fn native_overmap_atlas(read: &OvermapReading, settings: Option<&ViewSettings>)
    -> Option<GraphCanvasAtlas<String>> {
    let [width, height] = read.bounds?;
    if read.cells.is_empty() { return None; }
    let settings = settings.or(read.settings.as_ref())?;
    let mut paint = Vec::new();
    let mut fields = Vec::new();
    for cell in &read.cells {
        let anchor = Vec2::new(cell.at[0] as f32 + 0.5, cell.at[1] as f32 + 0.5);
        rectangle(&mut paint, anchor, Size2::new(1.0, 1.0), cell.color);
        if settings.thumbnails.or(read.settings.as_ref().and_then(|settings| settings.thumbnails)) == Some(true) {
            if let Some(color) = cell.thumbnail {
                rectangle(&mut paint, anchor, Size2::new(1.0, 1.0), color);
            }
        }
        if settings.biomes && !cell.biomes.is_empty() {
            let stripe = 1.0 / cell.biomes.len() as f32;
            for (index, biome) in cell.biomes.iter().enumerate() {
                rectangle(&mut paint, Vec2::new(cell.at[0] as f32 + stripe * (index as f32 + 0.5),
                    cell.at[1] as f32 + 0.25), Size2::new(stripe, 0.5), biome.color);
            }
        }
        if settings.factions {
            let claims: Vec<_> = cell.claims.iter().filter(|claim| claim.present && claim.color.is_some()).collect();
            if !claims.is_empty() {
                let stripe = 1.0 / claims.len() as f32;
                for (index, claim) in claims.iter().enumerate() {
                    if let Some(color) = claim.color {
                        rectangle(&mut paint, Vec2::new(cell.at[0] as f32 + stripe * (index as f32 + 0.5),
                            cell.at[1] as f32 + 0.75), Size2::new(stripe, 0.5), color);
                    }
                }
            }
        }
        fields.push(GraphCanvasAtlasField { id:native_overmap_cell_id(cell), anchor,
            footprint:Footprint::Rect { size:Size2::new(1.0,1.0) }, priority:0,
            label:if cell.anchor { cell.label.clone() } else { String::new() } });
    }
    if let (Some(color), Some(width)) = (read.route_color, read.route_width_milli) {
        for route in &read.routes {
            let from = read.cells.iter().find(|cell| cell.site == route.from && cell.anchor)?;
            let to = read.cells.iter().find(|cell| cell.site == route.to && cell.anchor)?;
            let a = Vec2::new(from.at[0] as f32 + 0.5, from.at[1] as f32 + 0.5);
            let b = Vec2::new(to.at[0] as f32 + 0.5, to.at[1] as f32 + 0.5);
            let delta = Vec2::new(b.x-a.x,b.y-a.y);
            let length = (delta.x*delta.x+delta.y*delta.y).sqrt();
            if length == 0.0 { continue; }
            let half = width as f32 / 2000.0;
            let offset = Vec2::new(-delta.y/length*half,delta.x/length*half);
            let polygon = vec![Vec2::new(a.x+offset.x,a.y+offset.y),Vec2::new(b.x+offset.x,b.y+offset.y),
                Vec2::new(b.x-offset.x,b.y-offset.y),Vec2::new(a.x-offset.x,a.y-offset.y)];
            for cell in &read.cells {
                let points = clip_cell(polygon.clone(),cell.at);
                if points.len() >= 3 {
                    paint.push(GraphCanvasAtlasPaint { anchor:Vec2::ZERO, footprint:Footprint::Polygon { points },
                        fill:rgba(color),stroke:None,stroke_width:0.0 });
                }
            }
        }
    }
    Some(GraphCanvasAtlas::new(Rect::new(Vec2::ZERO,Size2::new(width as f32,height as f32)))
        .with_paint(Arc::<[GraphCanvasAtlasPaint]>::from(paint)).with_fields(fields))
}

fn rectangle(paint: &mut Vec<GraphCanvasAtlasPaint>, anchor:Vec2, size:Size2, color:Color) {
    paint.push(GraphCanvasAtlasPaint { anchor,footprint:Footprint::Rect { size },
        fill:rgba(color),stroke:None,stroke_width:0.0 });
}

fn rgba([r,g,b,a]:Color) -> ColorF {
    ColorF { r:f32::from(r)/255.0,g:f32::from(g)/255.0,b:f32::from(b)/255.0,a:f32::from(a)/255.0 }
}

fn clip_cell(mut polygon:Vec<Vec2>, at:[u32;2]) -> Vec<Vec2> {
    for (axis,bound,lower) in [(0,at[0] as f32,true),(0,at[0] as f32+1.0,false),
        (1,at[1] as f32,true),(1,at[1] as f32+1.0,false)] {
        let coordinate = |p:Vec2| if axis == 0 { p.x } else { p.y };
        let inside = |p:Vec2| if lower { coordinate(p)>=bound } else { coordinate(p)<=bound };
        let Some(mut before) = polygon.last().copied() else { return polygon; };
        let mut next = Vec::new();
        for point in polygon {
            if inside(point) != inside(before) {
                let t = (bound-coordinate(before))/(coordinate(point)-coordinate(before));
                next.push(Vec2::new(before.x+(point.x-before.x)*t,before.y+(point.y-before.y)*t));
            }
            if inside(point) { next.push(point); }
            before = point;
        }
        polygon = next;
    }
    polygon
}

pub(crate) fn panel(ui:&crate::UiState, read:&crate::PartyView) -> Vec<crate::UiChild> {
    use cambium::{TextInput,caret_text_field,clickable,el,lens,text};
    let mut body:Vec<crate::UiChild> = Vec::new();
    if read.dm {
        body.push(Box::new(el("label",(text("Explicit saved map presentation JSON "),lens(
            |field:&mut TextInput| caret_text_field(field,&[]),
            |ui:&mut crate::UiState| &mut ui.native.party.presentation)))));
        body.push(Box::new(clickable(el("button",text("Save world map presentation")).attr("class","btn"),
            |ui:&mut crate::UiState,_| match serde_json::from_str(ui.native.party.presentation.text()) {
                Ok(input)=>ui.request_party(crate::PartyRequest::Presentation(input)),
                Err(error)=>ui.status=format!("map presentation input: {error}"),
            })));
    }
    let Some(map) = &read.overmap else { return body; };
    if let Some(settings) = &map.settings {
        for (label,field,value) in [("Biome layer",0,settings.biomes),("Faction layer",1,settings.factions),
            ("Thumbnails",2,settings.thumbnails==Some(true))] {
            let mut selected=settings.clone();
            match field {0=>selected.biomes=!value,1=>selected.factions=!value,_=>selected.thumbnails=Some(!value)};
            let party=read.map.party.clone();
            body.push(Box::new(clickable(el("button",text(format!("{label}: {}",if value {"on"} else {"off"})))
                .attr("class","btn btn-mini"),move |ui:&mut crate::UiState,_|
                ui.request_party(crate::PartyRequest::ReadAt {party:party.clone(),settings:selected.clone()}))));
        }
        body.push(Box::new(el("div",text(match &map.disclosure {
            Some(key)=>format!("Disclosures through {key}; access and membership are current."),
            None=>"All currently accepted disclosures; access and membership are current.".into(),
        })).attr("class","side-line")));
        for record in &map.available_disclosures {
            let mut selected=settings.clone();
            selected.disclosure=Some(record.clone());
            let party=read.map.party.clone();
            body.push(Box::new(clickable(el("button",text(format!("Read through {record}"))).attr("class","btn btn-mini"),
                move |ui:&mut crate::UiState,_| ui.request_party(crate::PartyRequest::ReadAt {
                    party:party.clone(),settings:selected.clone() }))));
        }
        let mut selected=settings.clone();
        selected.disclosure=None;
        let party=read.map.party.clone();
        body.push(Box::new(clickable(el("button",text("Return to all accepted disclosures")).attr("class","btn btn-mini"),
            move |ui:&mut crate::UiState,_| ui.request_party(crate::PartyRequest::ReadAt {
                party:party.clone(),settings:selected.clone() }))));
    }
    for limitation in &map.limitations {
        body.push(Box::new(el("div",text(limitation.clone())).attr("class","side-hint")));
    }
    if let Some(settings) = &map.settings {
        body.push(Box::new(el("div",text(format!("Biome layer {} · faction layer {} · thumbnails {} · sharp disclosed boundary",
            if settings.biomes { "on" } else { "off" },if settings.factions { "on" } else { "off" },
            if settings.thumbnails == Some(true) { "on" } else { "off" }))).attr("class","side-line")));
    }
    if let Some(cell) = map.cells.iter().find(|cell| ui.overmap_hover.as_deref() == Some(native_overmap_cell_id(cell).as_str())) {
        body.push(Box::new(el("div",text(format!("{} · native site {} · disclosure {} · display cell {},{}",
            cell.label,cell.site,cell.source,cell.at[0],cell.at[1]))).attr("class","side-line")));
        body.push(Box::new(el("div",text(format!("Retained place tags: {}",cell.tags.iter().cloned().collect::<Vec<_>>().join(", "))))
            .attr("class","side-line")));
        for biome in &cell.biomes {
            body.push(Box::new(el("div",text(format!("Biome tag reading {} ({}) · source {}",biome.name,biome.key,biome.source)))
                .attr("class","side-line")));
        }
        for held in &cell.conditions {
            body.push(Box::new(el("div",text(format!("Held condition statement {:?} · {:?} · source {} · revision {}",
                held.statement,held.meaning,held.source.hash,held.source.revision))).attr("class","side-line")));
        }
        for claim in &cell.claims {
            body.push(Box::new(el("div",text(format!("Held claim statement polity {}: {} · {:?} · source {} · revision {}",
                claim.polity,claim.present,claim.held.meaning,claim.held.source.hash,claim.held.source.revision)))
                .attr("class","side-line")));
        }
        for held in &cell.terrain {
            let source=&held.source;
            let named=&source.observation.named;
            body.push(Box::new(el("div",text(format!("Terrain {} · {:?}..{:?} · {} observed cells; other cells unknown · {:?} · tick {} · revision {} · hash {}",
                named.name,named.min,named.max,source.cells.len(),held.meaning,source.tick,
                source.observation.eye.terrain_revision,held.hash))).attr("class","side-line")));
        }
    }
    for (id,location) in &map.locations {
        body.push(Box::new(el("div",text(format!("Known Location {id} · {} identity sources · {} membership statements · {} extent statements",
            location.identity.len(),location.members.len(),location.extent.len()))).attr("class","side-line")));
        for held in location.members.iter().chain(&location.extent) {
            body.push(Box::new(el("div",text(format!("Retained {:?} · {:?} · source {}",
                held.statement,held.meaning,held.source.hash))).attr("class","side-hint")));
        }
    }
    body
}
