// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The disclosed overmap: the chosen party's exact retained disclosures,
//! read through this window's map settings. Display cells are cards and
//! routes between anchored cells are relations; an anchored cell inside the
//! seat's cut can be examined, since shared map facts grant no close-up.

use chirograph::IntentEffect;
use cosmograph_host::endpoint::disclosure::{Item, NULL, action, card, value};
use crate::native::overmap::{self, CellReading, ViewSettings};

use super::{Act, Built, Cx, VttEndpoint};

const CURATION: IntentEffect = IntentEffect::Curation;

fn cell_id(cell: &CellReading) -> String { format!("cell:{},{}", cell.at[0], cell.at[1]) }

fn settings_actions(settings: &ViewSettings, available: &[String]) -> (Vec<chirograph::AdvertisedAction>, Vec<Act>) {
    let (mut actions, mut acts) = (Vec::new(), Vec::new());
    for (label, field, on) in [("Biome layer", 0, settings.biomes), ("Faction layer", 1, settings.factions),
        ("Thumbnails", 2, settings.thumbnails == Some(true))] {
        let mut chosen = settings.clone();
        match field { 0 => chosen.biomes = !on, 1 => chosen.factions = !on, _ => chosen.thumbnails = Some(!on) }
        actions.push(action(&format!("vtt.map-layer-{field}"), &format!("{label}: {}", if on { "on" } else { "off" }),
            "Toggle this layer for this window.", NULL, CURATION));
        acts.push(Act::Overmap(Some(chosen)));
    }
    for (index, source) in available.iter().enumerate() {
        let mut chosen = settings.clone();
        chosen.disclosure = Some(source.clone());
        actions.push(action(&format!("vtt.map-history-{index}"), &format!("Read through {source}"),
            "Read the map as of this retained disclosure.", NULL, CURATION));
        acts.push(Act::Overmap(Some(chosen)));
    }
    let mut live = settings.clone();
    live.disclosure = None;
    actions.push(action("vtt.map-history-live", "Read all retained disclosures", "Read every accepted disclosure.",
        NULL, CURATION));
    acts.push(Act::Overmap(Some(live)));
    actions.push(action("vtt.map-inherit", "Use world map settings", "Drop this window's map settings.", NULL, CURATION));
    acts.push(Act::Overmap(None));
    (actions, acts)
}

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let mut built = Built::new();
    let access = cx.cut.access;
    let read = match cx.party {
        None => Err("Choose an authored party".to_string()),
        Some(party) => endpoint.read(|session, bytes| overmap::read_overmap(session, bytes, access, party, cx.overmap.cloned())),
    };
    let read = match read {
        Ok(read) => read,
        Err(reason) => {
            built.push(Item { id: "overmap".into(), card: card("Disclosed overmap", vec![value("Note", reason)], &["note"]),
                actions: Vec::new() }, Vec::new());
            return Ok(built);
        },
    };
    let mut values = vec![value("Disclosures", match &read.disclosure {
        Some(source) => format!("Disclosures through {source}. Access and membership are current."),
        None => "All retained disclosures. Access and membership are current.".into(),
    })];
    values.extend(read.bounds.map(|bounds| value("Bounds", format!("{} by {}", bounds[0], bounds[1]))));
    values.extend(read.limitations.iter().map(|limitation| value("Limitation", limitation)));
    let examined = endpoint.examined();
    values.extend(examined.map(|site| value("Examining", format!("site {site}"))));
    let (mut actions, mut acts) = match &read.settings {
        Some(settings) => settings_actions(settings, &read.available_disclosures),
        None => (Vec::new(), Vec::new()),
    };
    if examined.is_some() {
        actions.push(action("vtt.stop-examining", "Stop examining", "Return this window to its ordinary attention.",
            NULL, IntentEffect::DomainTruth));
        acts.push(Act::Examine(None));
    }
    built.push(Item { id: "overmap".into(), card: card("Disclosed overmap", values, &["overmap"]), actions }, acts);
    let layers = read.settings.clone().unwrap_or(ViewSettings { biomes: false, factions: false, thumbnails: None,
        disclosure: None });
    for cell in &read.cells {
        let mut values = vec![value("Site", cell.site), value("Cell", format!("({}, {})", cell.at[0], cell.at[1])),
            value("Disclosure", &cell.source)];
        if !cell.tags.is_empty() {
            values.push(value("Tags", cell.tags.iter().cloned().collect::<Vec<_>>().join(", ")));
        }
        if layers.biomes {
            values.extend(cell.biomes.iter().map(|biome| value("Biome", &biome.name)));
        }
        if layers.factions {
            values.extend(cell.claims.iter().map(|claim| value("Claim",
                format!("polity {} {}", claim.polity, if claim.present { "present" } else { "absent" }))));
        }
        values.extend(cell.conditions.iter().map(|held| value("Condition", format!("{:?}", held.statement))));
        if !cell.terrain.is_empty() { values.push(value("Terrain evidence", cell.terrain.len())); }
        let badges: &[&str] = if cell.anchor { &["cell", "anchor"] } else { &["cell"] };
        let (actions, acts) = if cell.anchor && cx.cut.visible.sites.contains(&cell.site) {
            (vec![action("vtt.examine", "Examine through this seat", "Attend closely to this site.", NULL,
                IntentEffect::DomainTruth)], vec![Act::Examine(Some(cell.site))])
        } else { (Vec::new(), Vec::new()) };
        built.push(Item { id: cell_id(cell), card: card(&cell.label, values, badges), actions }, acts);
    }
    let anchor = |site| read.cells.iter().find(|cell| cell.anchor && cell.site == site).map(cell_id);
    for route in &read.routes {
        if let (Some(from), Some(to)) = (anchor(route.from), anchor(route.to)) {
            built.links.push((from, to, "route"));
        }
    }
    Ok(built)
}
