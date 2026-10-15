// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world's scene and its cards, read from native state.

use std::collections::{BTreeMap, HashMap};

use chirograph::{
    AdvertisedAction, BoundsRelationship, CardValueV1, ContentHash, IntentEffect, IntentReference,
    PortableCardV1, PresentationBinding, PresentationCapability, PresentationCodec, PresentationKey,
    PresentationManifest, PresentationOffer, PresentationSemantics, SceneSnapshot, SemanticRole,
};
use isocosm::{Session, schema::Ledger};
use sceno::{
    Arrangement, Footprint, InstanceId, Placement, Representation, RoutedRelation, Scene, Score,
    ScoreItem, SourceRef, Spiral,
};

use super::{Curation, EXAMINE, INSPECT, SAVE, STEP};

const ADAPTER: &str = "isocosm.world";

pub(super) fn arrangement() -> Arrangement { Arrangement::Spiral(Spiral::default()) }

pub(super) struct Item {
    id: String,
    card: PortableCardV1,
    actions: Vec<AdvertisedAction>,
}

fn value(label: impl Into<String>, value: impl ToString) -> CardValueV1 {
    CardValueV1 { label: label.into(), value: value.to_string() }
}

fn ledger(values: &mut Vec<CardValueV1>, ledger: &Ledger) {
    values.extend(ledger.iter().filter(|(_, v)| **v != 0).map(|(k, v)| value(k.as_str(), v)));
}

fn action(intent: &str, label: &str, explanation: &str, schema: &str, effect: IntentEffect) -> AdvertisedAction {
    AdvertisedAction {
        intent: IntentReference(intent.into()),
        label: label.into(),
        explanation: explanation.into(),
        payload_schema: schema.into(),
        input_form: None,
        effect,
    }
}

const NULL: &str = r#"{"type":"null"}"#;

/// The world item, a card per site, and the examined site's stored groups.
fn items(session: &Session, curation: Curation) -> Vec<Item> {
    let state = session.sim.state();
    let groups = &state.population.groups;
    let mut items = vec![Item {
        id: "world".into(),
        card: PortableCardV1 {
            title: "World".into(),
            values: vec![
                value("Tick", state.tick),
                value("Entities", groups.values().map(|g| g.count).sum::<u64>()),
                value("Stored groups", groups.len()),
                value("Sites", state.sites.len()),
                value("Matter", session.sim.matter()),
                value("State hash", format!("{:016x}", session.sim.state_hash())),
            ],
            badges: vec!["world".into()],
            media: Vec::new(),
        },
        actions: vec![
            action(STEP, "Step", "Advance the native world by the requested ticks.",
                r#"{"type":"object","properties":{"ticks":{"type":"integer","minimum":1}}}"#,
                IntentEffect::DomainTruth),
            action(SAVE, "Save", "Write the world to this endpoint's save path.", NULL,
                IntentEffect::ExternalEffect),
        ],
    }];
    for (&site, at) in &state.sites {
        let resident = groups.values().filter(|g| g.entity.place == site);
        let (alive, dead) = resident.fold((0u64, 0u64), |(a, d), g|
            if g.entity.alive { (a + g.count, d) } else { (a, d + g.count) });
        let routes: Vec<String> = at.routes.iter().map(|r| r.to.to_string()).collect();
        let mut values = vec![value("Alive", alive), value("Dead", dead), value("Routes", routes.join(", "))];
        ledger(&mut values, &at.accounts);
        values.extend(at.conditions.iter().map(|(k, v)| value(k.as_str(), v)));
        let mut badges = vec!["site".to_string()];
        if curation.site == Some(site) { badges.push("examined".into()); }
        items.push(Item {
            id: format!("site:{site}"),
            card: PortableCardV1 { title: format!("Site {site}"), values, badges, media: Vec::new() },
            actions: vec![action(EXAMINE, "Examine", "Show this site's stored groups.", NULL,
                IntentEffect::Curation)],
        });
    }
    let Some(site) = curation.site else { return items };
    for (&first, group) in groups.iter().filter(|(_, g)| g.entity.place == site) {
        let e = &group.entity;
        let mut values = vec![
            value("Kingdom", &e.kingdom),
            value("Alive", e.alive),
            value("Born", e.born),
            value("Members", group.count),
            value("Parts", e.parts.len()),
        ];
        if !e.traits.is_empty() {
            values.push(value("Traits", e.traits.iter().map(String::as_str).collect::<Vec<_>>().join(", ")));
        }
        ledger(&mut values, &e.accounts);
        let mut badges = vec!["group".to_string()];
        if curation.group == Some(first) {
            badges.push("inspected".into());
            for (id, part) in &e.parts {
                let matter: u64 = part.matter.values().sum();
                let cells: u64 = part.cells.values().map(|c| u64::from(*c)).sum();
                values.push(value(format!("Part {}", id.0), format!("matter {matter} · cells {cells}")));
            }
        }
        items.push(Item {
            id: format!("group:{first}"),
            card: PortableCardV1 { title: format!("{} ×{}", e.lineage, group.count), values, badges,
                media: Vec::new() },
            actions: vec![action(INSPECT, "Inspect", "Show this group's parts.", NULL,
                IntentEffect::Curation)],
        });
    }
    items
}

/// The items and their solved scene: routes link sites, residence links groups to their site.
pub(super) fn world(session: &Session, curation: Curation) -> Result<(Vec<Item>, Scene), String> {
    let items = items(session, curation);
    let mut score = Score::new(arrangement());
    score.items = items.iter().enumerate().map(|(ordinal, item)| ScoreItem {
        source: SourceRef::new(ADAPTER, &item.id),
        ordinal: ordinal as u32,
        footprint: Footprint::Circle { radius: 6.0 },
        representation: Representation::Card,
        placement: Placement::Ordinal,
        layer: 0,
        visible: true,
        axis: None,
        embedding: None,
        weight: None,
    }).collect();
    let mut scene = scenomise::solve(&score);
    let instances: HashMap<String, InstanceId> = scene.items.iter().enumerate()
        .filter_map(|(index, item)| scene.sources.get(item.source.0 as usize)
            .map(|source| (source.id.clone(), InstanceId(index as u32))))
        .collect();
    let link = |scene: &mut Scene, from: &str, to: &str, kind: &str| {
        let (Some(&from), Some(&to)) = (instances.get(from), instances.get(to)) else { return };
        let points = vec![scene.items[from.0 as usize].transform.translate,
            scene.items[to.0 as usize].transform.translate];
        scene.relations.push(RoutedRelation { from, to, space: Scene::WORLD, points,
            kind: Some(kind.into()), weight: None });
    };
    let state = session.sim.state();
    for (site, at) in &state.sites {
        for route in &at.routes {
            link(&mut scene, &format!("site:{site}"), &format!("site:{}", route.to), "route");
        }
    }
    if let Some(site) = curation.site {
        for (first, _) in state.population.groups.iter().filter(|(_, g)| g.entity.place == site) {
            link(&mut scene, &format!("group:{first}"), &format!("site:{site}"), "resides");
        }
    }
    Ok((items, scene))
}

/// A portable card per scene item, with its actions, disclosed as resources.
pub(super) fn presentations(items: &[Item], scene: &SceneSnapshot,
    resources: &mut BTreeMap<ContentHash, Vec<u8>>) -> Result<PresentationManifest, String> {
    let by_id: HashMap<&str, &Item> = items.iter().map(|item| (item.id.as_str(), item)).collect();
    let mut manifest = PresentationManifest::default();
    for (instance, item) in scene.active_items_in_order() {
        let source = scene.tables.sources.get(item.source.0 as usize).and_then(Option::as_ref)
            .ok_or_else(|| format!("scene item {} has no source", instance.0))?;
        let item = by_id.get(source.id.as_str())
            .ok_or_else(|| format!("scene names an undisclosed item {}", source.id))?;
        let bytes = serde_json::to_vec(&item.card).map_err(|e| format!("could not encode a card: {e}"))?;
        let hash = ContentHash::of(&bytes);
        let byte_size = bytes.len() as u64;
        resources.insert(hash, bytes);
        let key = PresentationKey(source.id.clone());
        manifest.bindings.push(PresentationBinding { instance, key: key.clone() });
        manifest.offers.insert(key, vec![PresentationOffer {
            codec: PresentationCodec::PortableCardV1,
            resource: hash,
            byte_size,
            requires: PresentationCapability::PortableCard,
            semantics: PresentationSemantics {
                label: item.card.title.clone(),
                role: SemanticRole::Article,
                bounds: BoundsRelationship::FillFootprint,
                actions: item.actions.clone(),
            },
        }]);
    }
    Ok(manifest)
}
