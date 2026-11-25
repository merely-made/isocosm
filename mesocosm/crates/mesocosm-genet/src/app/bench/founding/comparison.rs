// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Child, Page, Panel};
use cambium::{el, text};
use cosmograph::Session;
use std::collections::{BTreeMap, BTreeSet};

impl Panel {
    pub(super) fn compare_edit(&mut self) -> Result<(), String> {
        let mut edited = Self::new(self.recipe()?)?;
        if edited.session.sim.state().sites.contains_key(&self.site) {
            edited.site = self.site;
        }
        edited.lineage.clone_from(&self.lineage);
        edited.path = self.path.clone();
        let mut original = self.original.as_ref().unwrap_or(&self.session).clone();
        let at = original
            .sim
            .state()
            .tick
            .max(edited.session.sim.state().tick);
        edited.seek(at)?;
        if original.sim.state().tick < at {
            original.advance(at - original.sim.state().tick)?;
        }
        edited.original = Some(original);
        edited.page = Page::Watch;
        *self = edited;
        Ok(())
    }
}

fn site_readings(session: &Session, site_id: u64) -> Option<BTreeMap<String, i128>> {
    let state = session.sim.state();
    let site = state.sites.get(&site_id)?;
    let mut rows = BTreeMap::new();
    for (account, label) in [
        ("world:soil", "Soil (mg)"),
        ("sim:energy", "Available energy"),
    ] {
        if let Some(&amount) = site.accounts.get(account) {
            rows.insert(label.to_owned(), i128::from(amount));
        }
    }
    if let Some(skeleton) = &session.sim.genesis().rules.skeleton
        && let Some(&level) = site.conditions.get(&skeleton.water)
    {
        rows.insert("Sea level (base units)".into(), i128::from(level));
    }
    if let Some(&water) = site.conditions.get("world:water") {
        rows.insert("Water availability (0–100)".into(), i128::from(water));
    }
    for (key, lineage) in &state.lineages {
        if key.starts_with("lineage:") {
            rows.insert(
                format!(
                    "{} · {key} living",
                    lineage.kingdom.trim_start_matches("kingdom:")
                ),
                0,
            );
        }
    }
    let mut parts = 0i128;
    for group in state.population.groups.values() {
        let body = &group.entity;
        if body.alive
            && body.resident_site() == Some(site_id)
            && body.lineage.starts_with("lineage:")
        {
            let key = format!(
                "{} · {} living",
                body.kingdom.trim_start_matches("kingdom:"),
                body.lineage
            );
            *rows.entry(key).or_default() += i128::from(group.count);
            parts += body.parts.len() as i128 * i128::from(group.count);
        }
    }
    rows.insert("Parts on living residents".into(), parts);
    Some(rows)
}

fn row(label: String, original: String, edited: String, header: bool) -> Child {
    let cell = if header { "columnheader" } else { "cell" };
    Box::new(
        el(
            "div",
            (
                el("span", text(label))
                    .attr("class", "patch-label")
                    .attr("role", if header { cell } else { "rowheader" }),
                el("span", text(original))
                    .attr("class", "patch-value")
                    .attr("role", cell),
                el("span", text(edited))
                    .attr("class", "patch-value")
                    .attr("role", cell),
            ),
        )
        .attr("class", "patch-row")
        .attr("role", "row"),
    )
}

pub(super) fn display(original: &Session, edited: &Session, site: u64, out: &mut Vec<Child>) {
    let a = site_readings(original, site);
    let b = site_readings(edited, site);
    let keys: BTreeSet<_> = a.iter().chain(&b).flat_map(|rows| rows.keys()).collect();
    let mut rows = vec![row(
        format!("Site {site} · tick {}", edited.sim.state().tick),
        "Original".into(),
        "Edited".into(),
        true,
    )];
    for key in keys {
        let value = |data: &Option<BTreeMap<String, i128>>| {
            data.as_ref()
                .and_then(|r| r.get(key))
                .map_or_else(|| "—".into(), i128::to_string)
        };
        rows.push(row(key.clone(), value(&a), value(&b), false));
    }
    out.push(Box::new(el("h2", text("Original and edited"))));
    let first = original.sim.genesis();
    let second = edited.sim.genesis();
    let layout = |s: &Session| {
        s.sim
            .genesis()
            .founding
            .as_ref()
            .and_then(|f| f.map.as_ref())
            .map(|m| match m {
                cosmograph::map::Layout::Grid(g) => (g.shape.clone(), g.width, g.height),
            })
    };
    if first.seed != second.seed || layout(original) != layout(edited) {
        out.push(Box::new(el(
            "p",
            text(format!(
                "Original seed {} · edited seed {}. Sites are matched by number.",
                original.sim.genesis().seed,
                edited.sim.genesis().seed
            )),
        )));
    }
    if a.is_none() {
        out.push(Box::new(el(
            "p",
            text("The original world has no site with this number."),
        )));
    }
    out.push(Box::new(
        el("div", rows)
            .attr("role", "table")
            .attr("aria-label", "Founding comparison"),
    ));
}
