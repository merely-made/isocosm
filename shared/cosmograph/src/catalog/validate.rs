// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The catalog's consistency rules, each a problem string when broken.

use std::collections::{BTreeMap, BTreeSet};

use super::refs::{self, FLOWS, KINGDOMS, is_proposed, process_name};
use super::{Catalog, Model, Status};

fn plain_key(key: &str, prefix: &str) -> bool {
    key.strip_prefix(prefix).is_some_and(|rest| {
        !rest.is_empty()
            && rest.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    })
}

fn duplicates<'a>(keys: impl Iterator<Item = &'a String>) -> Vec<&'a String> {
    let mut seen = BTreeSet::new();
    keys.filter(|k| !seen.insert(*k)).collect()
}

pub(super) fn problems(c: &Catalog) -> Vec<String> {
    let mut out = Vec::new();
    let mechanics: BTreeSet<&str> = c.mechanics.iter().map(|m| m.key.as_str()).collect();
    let proposed: BTreeSet<&str> = c.proposed.iter().map(|p| p.key.as_str()).collect();

    for k in duplicates(c.traits.iter().map(|t| &t.key)) {
        out.push(format!("duplicate trait key {k}"));
    }
    for k in duplicates(c.effects.iter().map(|e| &e.key)) {
        out.push(format!("duplicate effect key {k}"));
    }
    for k in duplicates(c.parts.iter().map(|p| &p.key)) {
        out.push(format!("duplicate part key {k}"));
    }
    for k in duplicates(c.mechanics.iter().map(|m| &m.key)) {
        out.push(format!("duplicate mechanic key {k}"));
    }

    let process_ok = |process: &str, shape: &str, who: &str, out: &mut Vec<String>| {
        if is_proposed(process) {
            if !proposed.contains(process_name(process)) {
                out.push(format!("{who}: {process} is not a listed proposed process"));
            }
        } else if process.split_once(':').map(|(ns, _)| ns) != Some(refs::NATIVE_NAMESPACE)
            || !refs::native_process(process_name(process))
        {
            out.push(format!("{who}: process {process} is not in the native registry"));
        }
        if !refs::shape_exists(shape) {
            out.push(format!("{who}: unknown shape {shape}"));
        } else if !refs::fits(process, shape) {
            out.push(format!("{who}: {process} does not fit {shape}"));
        }
    };
    let flows_ok = |flows: &[String], who: &str, out: &mut Vec<String>| {
        for f in flows.iter().filter(|f| !FLOWS.contains(&f.as_str())) {
            out.push(format!("{who}: unknown flow {f}"));
        }
    };
    let kingdoms_ok = |kingdoms: &[String], who: &str, out: &mut Vec<String>| {
        for k in kingdoms.iter().filter(|k| !KINGDOMS.contains(&k.as_str())) {
            out.push(format!("{who}: {k} is not a kingdom key"));
        }
    };
    let status_ok = |via: &[String], needs: &Option<String>, who: &str, out: &mut Vec<String>| {
        for r in via.iter().filter(|r| !refs::resolves(r)) {
            out.push(format!("{who}: via {r} names no existing rule"));
        }
        if let Some(m) = needs {
            if !mechanics.contains(m.as_str()) {
                out.push(format!("{who}: needs {m}, which is not a listed mechanic"));
            }
        }
    };

    let by_trait_keys: BTreeSet<&str> = c.traits.iter().map(|t| t.key.as_str()).collect();
    for t in &c.traits {
        let who = format!("trait {}", t.key);
        if !plain_key(&t.key, "trait:") {
            out.push(format!("{who}: key is not trait:<plain-words>"));
        }
        process_ok(&t.process, &t.shape, &who, &mut out);
        kingdoms_ok(&t.kingdoms, &who, &mut out);
        flows_ok(&t.admits, &who, &mut out);
        flows_ok(&t.emits, &who, &mut out);
        status_ok(&t.via, &t.needs, &who, &mut out);
        if t.kingdoms.is_empty() || t.models.is_empty() || t.sites.is_empty() || t.cost_mg == 0 {
            out.push(format!("{who}: needs kingdoms, models, sites and a cost"));
        }
        flows_ok(&t.unlocks, &who, &mut out);
        if t.strengths.iter().any(|s| s.amount == 0 || s.amount > 10) {
            out.push(format!("{who}: strengths run 1 to 10"));
        }
        if t.reads.is_empty() || t.analogue.is_empty() {
            out.push(format!("{who}: needs readers and an analogue"));
        }
    }
    for e in &c.effects {
        let who = format!("effect {}", e.key);
        if !plain_key(&e.key, "effect:") {
            out.push(format!("{who}: key is not effect:<plain-words>"));
        }
        kingdoms_ok(&e.kingdoms, &who, &mut out);
        flows_ok(&e.admits, &who, &mut out);
        flows_ok(&e.emits, &who, &mut out);
        status_ok(&e.via, &e.needs, &who, &mut out);
        for p in &e.processes {
            let native = refs::native_process(p);
            if !native && !proposed.contains(p.as_str()) {
                out.push(format!("{who}: process {p} is neither native nor proposed"));
            }
        }
        for gate in &e.gates {
            if !by_trait_keys.contains(gate.requires.as_str()) && !FLOWS.contains(&gate.requires.as_str()) {
                out.push(format!("{who}: gate requires {}, neither a trait nor a food category", gate.requires));
            }
        }
        if let Some(t) = e.timing {
            if t.intensity == 0 || t.duration == 0 {
                out.push(format!("{who}: timing needs an intensity and a duration"));
            }
        }
        for t in e.only.iter().filter(|t| !by_trait_keys.contains(t.as_str())) {
            out.push(format!("{who}: only names {t}, which is not a trait"));
        }
        if e.models.is_empty() || e.accepts.is_empty() || e.cost_mg == 0 || e.analogue.is_empty() {
            out.push(format!("{who}: needs models, accepts, a cost and an analogue"));
        }
    }
    for m in &c.mechanics {
        if m.summary.is_empty() || m.key.is_empty() {
            out.push(format!("mechanic {}: needs a summary", m.key));
        }
    }
    for p in &c.proposed {
        if p.key.contains(':') || p.summary.is_empty() {
            out.push(format!("proposed process {}: plain name and a summary", p.key));
        }
    }

    let mut meets: BTreeMap<super::Kind, [bool; 2]> = BTreeMap::new();
    for s in c.traits.iter().flat_map(|t| &t.strengths) {
        meets.entry(s.kind).or_default()[(s.role == super::Role::Defend) as usize] = true;
    }
    for (kind, [attack, defend]) in meets {
        if !(attack && defend) {
            out.push(format!("kind {kind:?} has an attack or a defence but not both"));
        }
    }
    let by_trait: BTreeMap<&str, _> = c.traits.iter().map(|t| (t.key.as_str(), t)).collect();
    let by_effect: BTreeMap<&str, _> = c.effects.iter().map(|e| (e.key.as_str(), e)).collect();
    let mut prints: BTreeMap<String, &str> = BTreeMap::new();
    for p in &c.parts {
        let who = format!("part {}", p.key);
        if !plain_key(&p.key, "part:") {
            out.push(format!("{who}: key is not part:<plain-words>"));
        }
        if !KINGDOMS.contains(&p.kingdom.as_str()) {
            out.push(format!("{who}: {} is not a kingdom key", p.kingdom));
        }
        process_ok(&p.process, &p.shape, &who, &mut out);
        flows_ok(&p.admits, &who, &mut out);
        flows_ok(&p.emits, &who, &mut out);
        if p.traits.is_empty() || p.cost_mg == 0 || p.analogue.is_empty() || p.models.is_empty() {
            out.push(format!("{who}: needs traits, a cost, an analogue and models"));
        }
        if p.effect_keys().next().is_none() {
            out.push(format!("{who}: has no effect at any site (memo section 2, test 3)"));
        }
        let mut served: BTreeSet<Model> = BTreeSet::new();
        for key in &p.traits {
            let Some(t) = by_trait.get(key.as_str()) else {
                out.push(format!("{who}: trait {key} does not exist"));
                continue;
            };
            served.extend(t.models.iter().copied());
            if !t.kingdoms.contains(&p.kingdom) {
                out.push(format!("{who}: trait {key} is not for {}", p.kingdom));
            }
            if !is_proposed(&t.process) && !refs::fits(&t.process, &p.shape) {
                out.push(format!("{who}: trait {key} ({}) does not fit {}", t.process, p.shape));
            }
        }
        for (site, keys) in &p.effects {
            for key in keys {
                let Some(e) = by_effect.get(key.as_str()) else {
                    out.push(format!("{who}: effect {key} does not exist"));
                    continue;
                };
                served.extend(e.models.iter().copied());
                if e.site != *site {
                    out.push(format!("{who}: effect {key} fires at {:?}, not {site:?}", e.site));
                }
                if !e.kingdoms.is_empty() && !e.kingdoms.contains(&p.kingdom) {
                    out.push(format!("{who}: effect {key} is not for {}", p.kingdom));
                }
            }
        }
        for m in &p.models {
            if !served.contains(m) {
                out.push(format!("{who}: serves {m:?}, which none of its traits or effects do"));
            }
        }
        match &p.status {
            Status::Expressible { via } => {
                if via.is_empty() || via.iter().any(|r| !refs::resolves(r)) {
                    out.push(format!("{who}: expressible but names no real rule"));
                }
                if !p.waits_on.is_empty() || is_proposed(&p.process) {
                    out.push(format!("{who}: expressible yet waits"));
                }
            }
            Status::NeedsRule { mechanic } => {
                if !mechanics.contains(mechanic.as_str()) || !p.waits_on.contains(mechanic) {
                    out.push(format!("{who}: needs {mechanic}, not a listed mechanic it waits on"));
                }
            }
            Status::Proposed { process } => {
                if process != &p.process || !is_proposed(process) {
                    out.push(format!("{who}: proposed status without a proposed process"));
                }
            }
        }
        if let Some(other) = prints.insert(p.fingerprint(), &p.key) {
            out.push(format!("{who}: same fingerprint as {other} (two names for one thing)"));
        }
    }
    out
}
