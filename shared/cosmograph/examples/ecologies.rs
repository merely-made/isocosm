// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a run of seeds founds and how each ecology fares: the lineages a
//! seed draws, what each body does for a living, and who is left after a
//! span of unattended ticks. Reads the ordinary starting recipe
//! (`Founding::starter`), so it shows what New World would open on.
//!
//! `cargo run --release --example ecologies -- [first seed] [seeds] [ticks] [diagnose] [lineage]`
//!
//! With a fourth argument it also keeps the food diagnostic: each lineage's
//! meals, refusals, births and deaths, what its neighbours offer it and how
//! many of its mouths still work. With a fifth, a lineage's number, it prints
//! that lineage tick by tick: members, how many are resident, tissue held.
//! A reading for whoever tunes a world; nothing is asserted.

use cosmograph::{
    Execution, Founding, Session,
    coarse::Region,
    flows::MadeBy,
    lineage::sheet,
    rules::Rule,
};
use std::collections::{BTreeMap, BTreeSet};

fn arg(n: usize, default: u64) -> u64 {
    std::env::args().nth(n).and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn operator(rule: &Rule) -> &'static str {
    match rule {
        Rule::Repeat { .. } => "repeat",
        Rule::Branch { .. } => "branch",
        Rule::Differentiate { .. } => "differentiate",
        Rule::Accrete { .. } => "accrete",
        Rule::Radiate { .. } => "radiate",
        Rule::Partner { .. } => "partner",
        Rule::Fuse { .. } => "fuse",
        Rule::Deposit { .. } => "deposit",
        Rule::Symmetry { .. } => "symmetry",
        Rule::Switch { .. } => "switch",
        Rule::Allocate { .. } => "allocate",
        Rule::Partition(_) => "partition",
    }
}

fn living(session: &Session) -> Result<BTreeMap<String, u64>, String> {
    let mut out = BTreeMap::new();
    for region in Region::read(&session.sim)?.values() {
        for (line, n) in &region.living {
            *out.entry(line.clone()).or_default() += n;
        }
        for (line, n) in &region.surfaces {
            *out.entry(line.clone()).or_default() += u64::from(*n > 0);
        }
    }
    Ok(out)
}

/// A requirement's name and the reading it failed on.
fn brief(why: &str) -> String {
    let why = why.trim_start_matches("missing requirement: ");
    let name: String = why.chars().take_while(|c| c.is_alphanumeric()).collect();
    let reading = why.rsplit_once(" = ").map_or("", |(_, reading)| reading);
    format!("{name} {}", reading.chars().take(48).collect::<String>())
}

/// For one living member of each lineage, what stops each way of making a
/// living it has a process for.
fn idle(session: &Session) {
    let sim = &session.sim;
    let mut seen = BTreeSet::new();
    for (id, group) in &sim.state().population.groups {
        let e = &group.entity;
        let Some(index) = e.lineage.strip_prefix("lineage:") else { continue };
        if !e.alive || !seen.insert(e.lineage.clone()) {
            continue;
        }
        let mut stops = Vec::new();
        for family in ["ecology:produce", "ecology:feed", "mutualism:exchange"] {
            let process = format!("{family}-{index}");
            if !sim.genesis().rules.processes.contains_key(&process) {
                continue;
            }
            let why = sim.unmet(*id, &process);
            if family == "ecology:feed" {
                let birth = sim.unmet(*id, &format!("ecology:birth-{index}"));
                let birth: Vec<_> = birth.iter().map(|w| brief(w)).collect();
                let mut beside: BTreeMap<&str, u64> = BTreeMap::new();
                for other in sim.state().population.groups.values() {
                    let o = &other.entity;
                    if o.alive && o.lineage != e.lineage && o.resident_site() == e.resident_site() && e.resident_site().is_some() {
                        *beside.entry(o.kingdom.trim_start_matches("kingdom:")).or_default() += other.count;
                    }
                }
                stops.push(format!("held {} · mood {:?} · beside {beside:?} · journey {} · birth ✗ {}", cosmograph::anatomy::books(e).values().sum::<u64>(), sim.mood(*id), e.journey.is_some(), birth.join(" | ")));
            }
            if !why.is_empty() {
                let short: Vec<_> = why.iter().map(|w| brief(w)).collect();
                stops.push(format!("{} ✗ {}", family.rsplit(':').next().unwrap_or(family), short.join(" | ")));
            }
        }
        if !stops.is_empty() {
            println!("    {} at site {} · {}", e.lineage, e.place, stops.join(" · "));
        }
    }
}

fn found(seed: u64, ticks: u64) -> Result<(usize, usize), String> {
    let founding = Founding::starter(seed);
    let mut session = Session::new(founding.generate()?, Execution::Grouped)?;
    let genesis = session.sim.genesis().clone();
    let start = living(&session)?;
    println!("seed {seed} · {} sites · {} lineages", founding.sites, genesis.lineages.len());
    for (key, line) in &genesis.lineages {
        let Some(d) = &line.development else {
            println!("  {key:<12} {:<14} (no body)", line.kingdom.trim_start_matches("kingdom:"));
            continue;
        };
        let sheet = sheet::of(&genesis.rules, &genesis.lineages, key, d)?;
        let rules: Vec<_> = d.grammar.iter().flat_map(|g| &g.rules).map(operator).collect();
        let functions: Vec<_> = sheet
            .functions
            .iter()
            .map(|(f, n)| format!("{} {n}", f.trim_start_matches("function:")))
            .collect();
        println!(
            "  {key:<12} {:<8} {:?}/{:?} · {} mg · {} · gives {:?} takes {:?} · grows by {}",
            line.kingdom.trim_start_matches("kingdom:"),
            sheet.trophic,
            sheet.feeding,
            sheet.mass,
            functions.join(", "),
            sheet.emits,
            sheet.admits,
            rules.join(" "),
        );
    }
    let diagnose = std::env::args().nth(4).is_some();
    if diagnose {
        session.sim.begin_food_diagnostic(cosmograph::watch::diagnostic::FoodDiagnosticConfig {
            focus_lineages: genesis.lineages.keys().cloned().collect(),
            examples_per_kind: 1,
            bucket_ticks: ticks.max(1),
        })?;
    }
    let mut acts: BTreeMap<String, u64> = BTreeMap::new();
    let mut pairs: BTreeSet<(String, String)> = BTreeSet::new();
    for tick in 1..=ticks {
        for flow in session.advance_tick_with_flows()?.flows {
            let MadeBy::Process(process) = &flow.made_by else { continue };
            let family = process.rsplit_once('-').map_or(process.as_str(), |(head, tail)| {
                if tail.chars().all(|c| c.is_ascii_digit()) { head } else { process.as_str() }
            });
            *acts.entry(family.to_owned()).or_default() += 1;
            if let (Some(from), Some(to)) = (&flow.from_kind, &flow.to_kind)
                && from.lineage != to.lineage
                && (family.starts_with("mutualism:") || family.starts_with("ecology:feed"))
            {
                pairs.insert((format!("{}>{}", family, from.lineage), to.lineage.clone()));
            }
        }
        if let Some(traced) = std::env::args().nth(5) {
            // One lineage, tick by tick: who is where, what they hold, what they did.
            let sim = &session.sim;
            let line = format!("lineage:{traced}");
            let (mut members, mut resident, mut tissue) = (0u64, 0u64, 0u64);
            let mut ids = BTreeSet::new();
            for (first, group) in sim.state().population.groups.iter().filter(|(_, g)| g.entity.alive && g.entity.lineage == line) {
                members += group.count;
                resident += group.count * u64::from(group.entity.resident_site().is_some());
                tissue += group.count * cosmograph::anatomy::books(&group.entity).get(&format!("matter:{traced}-0")).copied().unwrap_or(0);
                ids.extend(*first..*first + group.count);
            }
            let mut did: BTreeMap<String, u64> = BTreeMap::new();
            for event in sim.state().events.values().filter(|e| e.tick == sim.state().tick && ids.contains(&e.subject)) {
                let name = if event.process.starts_with("move:") { "move" } else { event.process.as_str() };
                *did.entry(name.to_owned()).or_default() += 1;
            }
            println!("   t{tick} · {members} members, {resident} resident, tissue {tissue} · {did:?}");
        }
        if diagnose && [8, 96].contains(&tick) {
            println!("   at tick {tick}");
            idle(&session);
        }
        if diagnose && [8, 32, 96].contains(&tick) {
            // Of each lineage founded with a mouth, the members whose mouth still works.
            let mut mouths: BTreeMap<&str, [u64; 3]> = BTreeMap::new();
            for group in session.sim.state().population.groups.values().filter(|g| g.entity.alive) {
                let e = &group.entity;
                let declared = e.parts.values().any(|p| p.port.is_some_and(|port| port.is_mouth()));
                let working = cosmograph::mosaic::ports::ports(e).is_mouth();
                let wounded: u64 = e.parts.values().map(|p| p.lost.len() as u64).sum();
                let row = mouths.entry(e.lineage.as_str()).or_default();
                row[0] += group.count * u64::from(declared);
                row[1] += group.count * u64::from(working);
                row[2] += group.count * wounded;
            }
            // What each neighbour offers one member of each feeding lineage.
            let sim = &session.sim;
            let mut asked = BTreeSet::new();
            for group in sim.state().population.groups.values().filter(|g| g.entity.alive) {
                let e = &group.entity;
                let Some(index) = e.lineage.strip_prefix("lineage:") else { continue };
                let Some(feed) = sim.genesis().rules.processes.get(&format!("ecology:feed-{index}")).filter(|p| p.port_feeding.is_some()) else { continue };
                if e.resident_site().is_none() || !asked.insert(e.lineage.clone()) {
                    continue;
                }
                let mut offers: BTreeMap<String, u64> = BTreeMap::new();
                for other in sim.state().population.groups.values() {
                    let o = &other.entity;
                    if o.lineage == e.lineage || o.resident_site() != e.resident_site() {
                        continue;
                    }
                    let what = match feed.food_offerings(e, o, &sim.genesis().rules, &sim.state().lineages) {
                        Ok(parts) if parts.is_empty() => "nothing".to_owned(),
                        Ok(parts) => format!("food {}", parts.iter().flat_map(|p| p.offered.values()).sum::<u64>().min(9)),
                        Err(why) => why.chars().take(40).collect(),
                    };
                    let life = if o.alive { "live" } else { "dead" };
                    *offers.entry(format!("{} {life}: {what}", o.kingdom.trim_start_matches("kingdom:"))).or_default() += other.count;
                }
                println!("    {} at tick {tick} is offered · {offers:?}", e.lineage);
            }
            let rows: Vec<_> = mouths.iter().filter(|(_, r)| r[0] > 0).map(|(k, r)| format!("{} {}/{} lost {}", k.trim_start_matches("lineage:"), r[1], r[0], r[2])).collect();
            println!("   mouths working at tick {tick} · {}", rows.join(" · "));
        }
        if tick == ticks || tick.is_power_of_two() && tick >= 16 {
            let now = living(&session)?;
            let extant = start.keys().filter(|k| now.get(*k).copied().unwrap_or(0) > 0).count();
            let total: u64 = now.values().sum();
            println!("  tick {tick:>5} · {extant}/{} lineages · {total} living", start.len());
        }
    }
    if let Some(report) = session.sim.end_food_diagnostic() {
        // How each lineage's meals and lives went over the whole span.
        let mut meals: BTreeMap<(String, String), u64> = BTreeMap::new();
        for feed in &report.feeds {
            let how = if feed.accepted { "ate".to_owned() } else { feed.stage.clone() };
            *meals.entry((feed.lineage.clone(), format!("{} {how}", feed.phase))).or_default() += feed.attempts;
        }
        let mut lives: BTreeMap<(String, String), u64> = BTreeMap::new();
        for life in &report.life {
            *lives.entry((life.lineage.clone(), format!("{} {}", life.kind, life.process.split(':').next_back().unwrap_or("")))).or_default() += life.members;
        }
        let mut refused: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for attempt in report.feed_examples.iter().filter(|a| !a.accepted) {
            let prey = attempt.target_lineage.clone().unwrap_or_else(|| "no target".into());
            let why: String = attempt.outcome.chars().take(110).collect();
            refused.entry(attempt.lineage.clone()).or_default().insert(format!("{} of {prey}: {why}", attempt.stage));
        }
        for (line, whys) in &refused {
            for why in whys.iter().take(4) {
                println!("    {line} refused · {why}");
            }
        }
        for line in genesis.lineages.keys() {
            let of = |book: &BTreeMap<(String, String), u64>| {
                book.iter().filter(|((l, _), _)| l == line).map(|((_, how), n)| format!("{how} {n}")).collect::<Vec<_>>().join(", ")
            };
            let (ate, lived) = (of(&meals), of(&lives));
            if !ate.is_empty() || !lived.is_empty() {
                println!("    {line} · meals: {ate} · lives: {lived}");
            }
        }
    }
    let end = living(&session)?;
    let census: Vec<_> = start
        .iter()
        .map(|(k, n)| format!("{} {n}→{}", k.trim_start_matches("lineage:"), end.get(k).copied().unwrap_or(0)))
        .collect();
    println!("  members · {}", census.join(" · "));
    let acts: Vec<_> = acts.iter().map(|(k, n)| format!("{k} {n}")).collect();
    println!("  moves · {}", acts.join(" · "));
    // Of each lineage's choices, the moves taken toward something sensed.
    let minds: Vec<_> = cosmograph::directing::policy::readout(&mut session)
        .iter()
        .filter(|r| r.tally.decisions > 0)
        .map(|r| format!("{} {}/{}", r.lineage.trim_start_matches("lineage:"), r.tally.led, r.tally.decisions))
        .collect();
    println!("  led by sense, of choices · {}", minds.join(" · "));
    let webs: Vec<_> = pairs.iter().map(|(a, b)| format!("{a}>{b}")).collect();
    println!("  between lineages · {}", webs.join(" · "));
    Ok((start.keys().filter(|k| end.get(*k).copied().unwrap_or(0) > 0).count(), start.len()))
}

fn main() {
    let (first, seeds, ticks) = (arg(1, 0), arg(2, 8), arg(3, 256));
    let mut kept = Vec::new();
    for seed in first..first + seeds {
        match found(seed, ticks) {
            Ok((extant, started)) => kept.push(format!("{seed}: {extant}/{started}")),
            Err(why) => {
                println!("seed {seed} · failed · {why}");
                kept.push(format!("{seed}: failed"));
            },
        }
    }
    println!("lineages left after {ticks} ticks · {}", kept.join(" · "));
}
