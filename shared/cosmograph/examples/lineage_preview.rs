// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A read-only observer for the optional-gland founding editor. The registered
//! native ecology and development rules do all the work in a grouped session.

use cosmograph::{
    Execution, Founding, Session,
    ecology::Ward,
    flows::Holder,
    rules::{Anchor, Binding, Effect, Facing, Query, Rule},
    schema::Entity,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io;

const PLANT: &str = "lineage:0";
const GRAZE: &str = "ecology:feed-1";
const RENT: &str = "guard:rent-0";
const HORIZON: u64 = 35;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Edit {
    whole: Option<[i32; 3]>,
    piece: Option<[i32; 3]>,
    grazers: Option<bool>,
    seed: Option<u64>,
    absence: Option<[u32; 2]>,
    repeat: Option<u8>,
    stem_variance: Option<u8>,
    mapped_sites: Option<bool>,
    direct_grazers: Option<bool>,
    cohort_size: Option<u64>,
    anchor: Option<Anchor>,
    side: Option<Facing>,
    frond_socket: Option<Facing>,
    organ: Option<String>,
    rent_cells: Option<u32>,
}

#[derive(Default, Serialize)]
struct Totals {
    births: [u64; 2],
    grazed: [u64; 2],
    rent: u64,
    soil_fixed: [u64; 2],
    upkeep: [u64; 2],
    defence_upkeep: [u64; 2],
    wounds: u64,
    spine_wounds: u64,
}

#[derive(Clone, Copy)]
enum Forms {
    Gland(u8),
    Partition(usize),
}

fn form(entity: &Entity, forms: Forms) -> usize {
    match forms {
        Forms::Gland(run) => usize::from(!entity.absent.contains(&(run, 0))),
        Forms::Partition(run) => usize::from(entity.soma[run] > 1),
    }
}

impl Totals {
    fn observe_tick(&mut self, session: &mut Session, gland_run: Forms) -> Result<(), String> {
        let first_new_id = session.sim.state().population.next_id;
        let tick = session.advance_tick_with_flows()?;
        let state = session.sim.state();
        for flow in tick.flows {
            let amount = flow.count * flow.amount;
            if let Some(plant) = flow.from.0.body().and_then(|id| state.population.get(id))
                && plant.lineage == PLANT
            {
                let index = form(plant, gland_run);
                if flow.by_process("ecology:produce-0")
                    && flow.internal()
                    && flow.from.1 == "world:soil"
                    && flow.to.1 == "matter:0-0"
                {
                    self.soil_fixed[index] += amount;
                } else if matches!(flow.to.0, Holder::Site(_)) && flow.to.1 == "world:soil" {
                    if flow.by_process("ecology:upkeep-0") {
                        self.upkeep[index] += amount;
                    } else if flow.by_process(RENT) {
                        self.defence_upkeep[index] += amount;
                        self.rent += amount;
                    }
                }
            }
            if flow.by_process(GRAZE)
                && let Some(prey) = flow.from.0.body().and_then(|id| state.population.get(id))
                && prey.lineage == PLANT
            {
                self.grazed[form(prey, gland_run)] += amount;
            }
        }
        self.wounds += session
            .sim
            .harms()
            .iter()
            .filter(|h| h.made_by == GRAZE && h.ward == Some(Ward::Toxin))
            .map(|h| u64::from(h.cells))
            .sum::<u64>();
        self.spine_wounds += session
            .sim
            .harms()
            .iter()
            .filter(|h| h.made_by == GRAZE && h.ward == Some(Ward::Spines))
            .map(|h| u64::from(h.cells))
            .sum::<u64>();
        for relation in &state.relations {
            if relation.kind == "sim:parent"
                && relation.subject >= first_new_id
                && let Some(child) = state.population.get(relation.subject)
                && child.lineage == PLANT
            {
                self.births[form(child, gland_run)] += 1;
            }
        }
        Ok(())
    }

    fn frame(&self, session: &Session, gland_run: Forms) -> Result<Value, String> {
        let living = living_forms(session, gland_run);
        // Preview entering at this tick. The continuing timeline stays unobserved.
        let mut entry = session.clone();
        cosmograph::bench::inspect_first_actor(&mut entry)?;
        Ok(
            json!({"tick":session.sim.state().tick, "living":living, "totals":self,
            "hash":format!("{:016x}",entry.sim.state_hash())}),
        )
    }
}

fn living_forms(session: &Session, gland_run: Forms) -> [u64; 2] {
    let mut living = [0; 2];
    for group in session.sim.state().population.groups.values() {
        if group.entity.lineage == PLANT && group.entity.alive {
            living[form(&group.entity, gland_run)] += group.count;
        }
    }
    living
}

/// Sample the same edited template in three independently founded habitats.
/// These sessions never join or advance the player's chosen world.
fn reference_habitats(founding: &Founding, gland_run: Forms) -> Result<Vec<Value>, String> {
    (1..=3).map(|offset| {
        let mut reference = founding.clone();
        reference.seed = founding.seed.wrapping_add(offset);
        let mut run = Session::new(reference.generate()?, Execution::Grouped)?;
        let initial = living_forms(&run, gland_run);
        let mut totals = Totals::default();
        for _ in 0..HORIZON {
            totals.observe_tick(&mut run, gland_run)?;
        }
        Ok(json!({"seed": reference.seed.to_string(), "initial": initial, "living": living_forms(&run, gland_run),
            "births": totals.births, "soil_fixed": totals.soil_fixed, "ticks": HORIZON}))
    }).collect()
}

fn growth_inputs(session: &Session) -> Result<Value, String> {
    let grow = session
        .sim
        .genesis()
        .rules
        .processes
        .get("ecology:produce-0")
        .ok_or("missing producer process")?;
    let site_input = |key: &str| {
        grow.effects
            .iter()
            .find_map(|effect| match effect {
                Effect::Transfer {
                    from: Binding::Place,
                    to: Binding::Actor,
                    account,
                    amount,
                } if account == key => Some(amount.resolved()),
                _ => None,
            })
            .ok_or_else(|| format!("missing producer input {key}"))?
    };
    Ok(json!({"light": site_input("sim:energy")?, "soil": site_input("world:soil")?,
        "needs_fixing_canopy": grow.requires.contains(&Query::FixingCanopy)}))
}

#[derive(Serialize)]
struct Choice {
    key: &'static str,
    shape: &'static str,
    traits: &'static [String],
    spine: u8,
    toxin: u8,
    cells: u32,
}

/// Only native, optional defensive tissues this plant can grow under the
/// current world rules fit this slot. Show the strengths the sim will read.
fn trait_bank(founding: &Founding) -> Vec<Choice> {
    cosmograph::catalog::parts()
        .iter()
        .filter_map(|part| {
            if part.kingdom != "kingdom:flora" {
                return None;
            }
            let tissue = cosmograph::repertoire::lowering_full(
                part,
                founding.categories.is_some(),
                founding.guard.is_some(),
                founding.mutualism.is_some(),
                (founding.detritus.is_some(), founding.nutrients.is_some()),
            )
            .ok()?
            .template;
            let cells = tissue
                .cells
                .get("function:support")
                .or_else(|| tissue.cells.get("function:secrete"))?;
            let strength = tissue.port?.strength();
            (strength.spine > 0 || strength.toxin > 0).then_some(Choice {
                key: &part.key,
                shape: &part.shape,
                traits: &part.traits,
                spine: strength.spine,
                toxin: strength.toxin,
                cells: *cells,
            })
        })
        .collect()
}

fn founders(session: &Session, forms: Forms) -> Result<Vec<Value>, String> {
    let mut founders = Vec::new();
    for (id, group) in &session.sim.state().population.groups {
        if group.entity.lineage != PLANT {
            continue;
        }
        let body = group.entity.body.as_ref().ok_or("plant has no body")?;
        let canopy = cosmograph::trophic::canopy_parts(body);
        let site = group.entity.resident_site().and_then(|id| {
            session.sim.state().sites.get(&id).map(|s| {
                json!({"id":id,"light":s.accounts.get("sim:energy").copied().unwrap_or(0),
            "soil":s.accounts.get("world:soil").copied().unwrap_or(0),
            "habitable":s.conditions.get("world:habitable").copied().unwrap_or(0) >= 1})
            })
        });
        let mut parts = Vec::new();
        for (part, tissue) in group.entity.living() {
            let geo = body.part(part).ok_or("missing body frame")?;
            let center = body.world_pivot(part).ok_or("missing part position")?;
            parts.push(json!({"id":part.0, "situs":geo.situs, "center":center,
                "parent":geo.attachment.map(|a| a.parent.0),"canopy":canopy.contains(&part),
                "half":geo.half_extent, "cells":tissue.cells}));
        }
        let mut founder = json!({"id":id, "count":group.count, "parts":parts,
            "site":site,"trophic":cosmograph::trophic::of(&group.entity)});
        let index = form(&group.entity, forms);
        match forms {
            Forms::Gland(_) => founder["gland"] = json!(index == 1),
            Forms::Partition(_) => founder["form"] = json!(index),
        }
        founders.push(founder);
    }
    Ok(founders)
}

fn frond_preview(mut founding: Founding, edit: Edit, run: usize) -> Result<Value, String> {
    if let Some(seed) = edit.seed { founding.seed = seed; }
    if edit.grazers == Some(false) { founding.unstocked_lineages.insert(1); }
    let grammar = &mut founding.body_maps.get_mut(PLANT).ok_or("missing body map")?.grammar;
    if let Some(socket) = edit.frond_socket { grammar.seed[run].facing = socket; }
    for rule in &mut grammar.rules {
        if let Rule::Partition(plan) = rule && usize::from(plan.tagma) == run {
            if let Some(whole) = edit.whole { plan.whole = whole; }
            if let Some(piece) = edit.piece { plan.piece = piece; }
        }
    }
    let source = toml::to_string_pretty(&founding).map_err(|e| e.to_string())?;
    let mut session = Session::new(founding.generate()?, Execution::Grouped)?;
    let forms = Forms::Partition(run);
    let founders = founders(&session, forms)?;
    let mut totals = Totals::default();
    let mut frames = Vec::new();
    for tick in 0..=HORIZON {
        if tick > 0 { totals.observe_tick(&mut session, forms)?; }
        let mut frame = totals.frame(&session, forms)?;
        let mut exposed = [0; 2];
        let mut active = [0; 2];
        let mut grazers = 0;
        for group in session.sim.state().population.groups.values().filter(|g| g.entity.alive) {
            let plant = &group.entity;
            if plant.lineage == PLANT {
                let (lit, funded) = cosmograph::trophic::fixing_tissue(plant, "matter:0-0");
                let index = form(plant, forms);
                exposed[index] += lit * group.count;
                active[index] += funded * group.count;
            } else if plant.lineage == "lineage:1" { grazers += group.count; }
        }
        frame["exposed"] = json!(exposed);
        frame["active"] = json!(active);
        frame["grazers"] = json!(grazers);
        frames.push(frame);
    }
    Ok(json!({"seed":founding.seed.to_string(), "map":founding.body_maps[PLANT],
        "founders":founders, "frames":frames,"source":source}))
}

fn preview(input: &str, edit: Edit) -> Result<Value, String> {
    let original: Founding = toml::from_str(input).map_err(|e| e.to_string())?;
    if let Some(run) = original.body_maps.get(PLANT).and_then(|m|
        m.grammar.rules.iter().find_map(|r| if let Rule::Partition(plan) = r { Some(usize::from(plan.tagma)) } else { None })) {
        return frond_preview(original, edit, run);
    }
    let bank = trait_bank(&original);
    let mut founding = original.clone();
    if edit.mapped_sites == Some(true) && founding.map.is_none() {
        founding.sites = 4;
        founding.map = Some(cosmograph::map::Layout::Grid(cosmograph::map::Grid {
            shape: "shape:plane".into(),
            width: 2,
            height: 2,
            side: 128,
            elevation: [0, 0],
            relief: [0, 6],
            sea_per_mille: 0,
        }));
    }
    if edit.direct_grazers == Some(true) {
        founding.played = Some(cosmograph::directing::Played {
            lineage: 1,
            region_sites: 1,
        });
    }
    if let Some(n) = edit.cohort_size {
        if n != 1 && n != original.cohort_size {
            return Err("choose single founders or the founding cohort size".into());
        }
        founding.cohort_size = n;
    }
    let map = founding
        .body_maps
        .get_mut(PLANT)
        .ok_or("missing plant body map")?;
    let [Rule::Repeat { tagma: 0, by }] = map.grammar.rules.as_mut_slice() else {
        return Err("this editor expects the optional-gland repeat rule".into());
    };
    if let Some(n) = edit.repeat {
        if n > 3 {
            return Err("repeat must add zero to three segments".into());
        }
        *by = n;
    }
    if let Some(v) = edit.stem_variance {
        if v > 1 {
            return Err("stem variance outside 0..1".into());
        }
        map.grammar.seed[0].variance = (v != 0).then_some(v);
    }
    if let Some(socket) = edit.frond_socket {
        if !matches!(socket, Facing::Above | Facing::Below) {
            return Err("select an upper or lower frond socket".into());
        }
        map.grammar.seed[0].socket = socket;
    }
    if let Some(organ) = edit.organ {
        if !bank.iter().any(|part| part.key == organ) {
            return Err("organ outside the defensive trait bank".into());
        }
        let repertoire = founding.repertoire.as_mut().ok_or("missing repertoire")?;
        let previous = map
            .regions
            .insert("region:gland".into(), organ.clone())
            .ok_or("missing defence binding")?;
        if !repertoire.only.is_empty() {
            repertoire.only.remove(&previous);
            repertoire.only.insert(organ);
        }
    }
    if let Some(odds) = edit.absence {
        map.absence = odds;
    }
    let gland_index = map
        .grammar
        .seed
        .iter()
        .position(|run| run.bears.as_deref() == Some("region:gland"))
        .ok_or("missing gland run")?;
    let side = &mut map.grammar.seed[gland_index];
    if let Some(anchor) = edit.anchor {
        side.anchor = anchor;
    }
    if let Some(facing) = edit.side {
        if !matches!(facing, Facing::Left | Facing::Right) {
            return Err("select the left or right side of the stem".into());
        }
        side.facing = facing;
    }
    let gland_run = Forms::Gland(u8::try_from(gland_index).map_err(|e| e.to_string())?);
    let guard = founding.guard.as_mut().ok_or("missing gland upkeep")?;
    if let Some(n) = edit.rent_cells {
        if !(1..=64).contains(&n) {
            return Err("upkeep interval outside 1..64 cells".into());
        }
        guard.rent_cells = n;
    }
    let rent_cells = guard.rent_cells;
    let source = if founding == original {
        input.to_owned()
    } else {
        toml::to_string_pretty(&founding).map_err(|e| e.to_string())?
    };
    let mut session = Session::new(founding.generate()?, Execution::Grouped)?;
    let inputs = growth_inputs(&session)?;
    let founders = founders(&session, gland_run)?;
    let mut totals = Totals::default();
    let mut frames = vec![totals.frame(&session, gland_run)?];
    for _ in 0..HORIZON {
        totals.observe_tick(&mut session, gland_run)?;
        frames.push(totals.frame(&session, gland_run)?);
    }
    Ok(
        json!({"seed":founding.seed.to_string(),"lineage":PLANT,"map":founding.body_maps[PLANT],
        "rent_cells":rent_cells,"founders":founders,
        "bank":bank,"cohort_size":founding.cohort_size,"mapped_sites":founding.map.is_some(),
        "direct_grazers":founding.played.is_some(),
        "frames":frames,"inputs":inputs,"reference":reference_habitats(&founding, gland_run)?,"source":source}),
    )
}

fn run() -> Result<(), String> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("supply the optional-gland founding TOML")?;
    let input = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let edit = serde_json::from_reader(io::stdin().lock()).map_err(|e| e.to_string())?;
    serde_json::to_writer(io::stdout().lock(), &preview(&input, edit)?).map_err(|e| e.to_string())
}

fn main() {
    if let Err(why) = run() {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
