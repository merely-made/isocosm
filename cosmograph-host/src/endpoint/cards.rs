// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world's cards and links, read from native state.

use chirograph::{CardValueV1, IntentEffect};
use cosmograph::{Session, schema::Ledger};

use super::disclosure::{Item, NULL, action, card, value};
use super::{Curation, EXAMINE, INSPECT, SAVE, STEP};

fn ledger(values: &mut Vec<CardValueV1>, ledger: &Ledger) {
    values.extend(ledger.iter().filter(|(_, v)| **v != 0).map(|(k, v)| value(k.as_str(), v)));
}

/// The world item, a card per site, and the examined site's stored groups.
pub(super) fn items(session: &Session, curation: Curation) -> Vec<Item> {
    let state = session.sim.state();
    let groups = &state.population.groups;
    let mut items = vec![Item {
        id: "world".into(),
        card: card("World", vec![
            value("Tick", state.tick),
            value("Entities", groups.values().map(|g| g.count).sum::<u64>()),
            value("Stored groups", groups.len()),
            value("Sites", state.sites.len()),
            value("Matter", session.sim.matter()),
            value("State hash", format!("{:016x}", session.sim.state_hash())),
        ], &["world"]),
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
        let badges: &[&str] = if curation.site == Some(site) { &["site", "examined"] } else { &["site"] };
        items.push(Item {
            id: format!("site:{site}"),
            card: card(format!("Site {site}"), values, badges),
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
        let inspected = curation.group == Some(first);
        if inspected {
            for (id, part) in &e.parts {
                let matter: u64 = part.matter.values().sum();
                let cells: u64 = part.cells.values().map(|c| u64::from(*c)).sum();
                values.push(value(format!("Part {}", id.0), format!("matter {matter} · cells {cells}")));
            }
        }
        let badges: &[&str] = if inspected { &["group", "inspected"] } else { &["group"] };
        items.push(Item {
            id: format!("group:{first}"),
            card: card(format!("{} ×{}", e.lineage, group.count), values, badges),
            actions: vec![action(INSPECT, "Inspect", "Show this group's parts.", NULL,
                IntentEffect::Curation)],
        });
    }
    items
}

/// Routes link sites; residence links an examined site's groups to it.
pub(super) fn links(session: &Session, curation: Curation) -> Vec<(String, String, &'static str)> {
    let state = session.sim.state();
    let mut links: Vec<_> = state.sites.iter().flat_map(|(site, at)| at.routes.iter()
        .map(move |route| (format!("site:{site}"), format!("site:{}", route.to), "route"))).collect();
    if let Some(site) = curation.site {
        links.extend(state.population.groups.iter().filter(|(_, g)| g.entity.place == site)
            .map(|(first, _)| (format!("group:{first}"), format!("site:{site}"), "resides")));
    }
    links
}
