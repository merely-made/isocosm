// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BiomeReading { pub key: Key, pub name: String, pub color: Color, pub source: Key }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellReading {
    pub at: [u32; 2], pub site: Id, pub source: Key, pub label: String, pub tags: BTreeSet<Key>,
    pub color: Color, pub biomes: Vec<BiomeReading>, pub thumbnail: Option<Color>,
    pub anchor: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteReading { pub from: Id, pub to: Id, pub travel: u64 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OvermapReading {
    pub map: isocosm::party::SharedMap,
    pub disclosure: Option<Key>, pub available_disclosures: Vec<Key>,
    pub bounds: Option<[u32; 2]>, pub cells: Vec<CellReading>, pub routes: Vec<RouteReading>,
    pub settings: Option<ViewSettings>, pub route_color: Option<Color>, pub route_width_milli: Option<u32>,
    pub limitations: Vec<String>,
}

/// A shared disclosure view does not grant each member these native facts.
pub fn read_overmap(session: &Session, game: &[u8], access: OrdinaryAccess, party: &str,
    settings: Option<ViewSettings>) -> Result<OvermapReading, String> {
    let map = super::super::party::read_party(session, access, party)?;
    let bindings = super::super::GameBindings::decode(game)?;
    let settings = settings.or_else(|| bindings.overmap().map(|presentation| presentation.settings.clone()));
    let disclosure = settings.as_ref().and_then(|settings| settings.disclosure.clone());
    let available_disclosures = map.records.clone();
    let mut reading = OvermapReading { map, bounds:None, cells:Vec::new(), routes:Vec::new(),
        disclosure, available_disclosures,settings:settings.clone(), route_color:None, route_width_milli:None, limitations:vec![
            "Terrain details have not been disclosed to this map.".into(),
            "Faction claims have not been disclosed to this map.".into(),
        ] };
    if let Some(key) = &reading.disclosure {
        let index = reading.map.records.iter().position(|record| record == key)
            .ok_or("historical overmap cut is not an exact retained disclosure for this party")?;
        let records = reading.map.records[..=index].to_vec();
        reading.map.places.clear();
        reading.map.routes.clear();
        reading.map.events.clear();
        for source in &records {
            let record = types::disclosure(session,source)?;
            for place in record.places { reading.map.places.insert(place.site,place); }
            reading.map.routes.extend(record.routes);
            reading.map.events.extend(record.events);
        }
        reading.map.records = records;
    }
    let Some(presentation) = bindings.overmap() else {
        reading.limitations.push("No authored overmap display-cell presentation is saved.".into());
        return Ok(reading);
    };
    validate_presentation(session, presentation)?;
    let mut settings = settings.unwrap_or_else(|| presentation.settings.clone());
    settings.thumbnails = Some(settings.thumbnails.or(presentation.settings.thumbnails)
        .unwrap_or(presentation.thumbnail.is_some()));
    for cell in &presentation.cells {
        if !reading.map.places.contains_key(&cell.site) || !reading.map.records.contains(&cell.disclosure) { continue; }
        let source = types::disclosure(session, &cell.disclosure)?;
        if source.party != party { continue; }
        let place = source.places.iter().find(|place| place.site == cell.site)
            .ok_or("display cell has no exact disclosed place")?;
        let tags = place.authored.as_ref().map(|place| place.tags.clone()).unwrap_or_default();
        let biomes = presentation.biomes.iter().filter(|biome| biome.tags.is_subset(&tags))
            .map(|biome| BiomeReading { key:biome.key.clone(), name:biome.name.clone(), color:biome.color,
                source:cell.disclosure.clone() }).collect();
        let thumbnail = presentation.thumbnail.as_ref().map(|thumbnail|
            thumbnail.pixels[(u64::from(cell.at[1]) * u64::from(thumbnail.size[0]) + u64::from(cell.at[0])) as usize]);
        reading.cells.push(CellReading { at:cell.at, site:cell.site, source:cell.disclosure.clone(),
            label:place.authored.as_ref().map(|place| place.name.clone()).unwrap_or_else(|| format!("Site {}", cell.site)),
            tags, color:presentation.cell_color, biomes, thumbnail,
            anchor:presentation.anchors.get(&cell.site) == Some(&cell.at) });
    }
    let shown: BTreeSet<_> = reading.cells.iter().filter(|cell| cell.anchor).map(|cell| cell.site).collect();
    reading.routes = reading.map.routes.iter().filter(|route| shown.contains(&route.from) && shown.contains(&route.to))
        .map(|route| RouteReading { from:route.from, to:route.to, travel:route.travel }).collect();
    reading.bounds = Some(presentation.bounds);
    reading.route_color = Some(presentation.route_color);
    reading.route_width_milli = Some(presentation.route_width_milli);
    reading.settings = Some(settings);
    Ok(reading)
}
