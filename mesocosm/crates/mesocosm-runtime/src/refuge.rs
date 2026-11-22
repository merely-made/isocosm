// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A matched meadow study over two native histories. The opening is the native
//! experiment's saved world; the treatment uses Mesocosm's ordinary place nudge.
use cosmograph::{Command, Execution, Session, anatomy, schema::Id};
use cosmograph_host::world::World;
use cosmograph_overlay::{
    Intent, ParticipantHandle, Tick,
    mesocosm::{MesocosmIntentEnvelope, NudgeMeaning},
};
use serde::Serialize;
use std::collections::BTreeMap;

const OPENING: &[u8] = include_bytes!("../../../packs/mesocosm/fixtures/refuge-opening.json");
pub const DAY: u64 = 1440;
const LEAN: u64 = 6 * DAY;
const REFUGE: Id = 1;
const GRAZE: &str = "ecology:feed-1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeadowChange {
    #[default]
    Reopens,
    TurnsLean,
}

fn set_weather(session: &mut Session, ground: Id, process: &str) -> Result<(), String> {
    session.command(Command::Act {
        actor: ground,
        process: process.into(),
        target: None,
        cause: None,
    })?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Patch {
    pub grazed: u64,
    pub eaten_by_grazer: u64,
    pub emerged: u64,
    pub shoots: u64,
    pub roots: u64,
    pub seeds: u64,
    pub grazers: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub hash: String,
    pub meadow_open: bool,
    pub alive: bool,
    pub site: Option<Id>,
    pub nudge: &'static str,
    pub minutes: [u64; 2],
    pub blocked_route_minutes: BTreeMap<String, u64>,
    pub patches: [Patch; 2],
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Pair {
    pub tick: u64,
    pub grazer: Id,
    pub alone: Observation,
    pub trial: Observation,
}

struct Run {
    session: Session,
    grazer: Id,
    ground: Id,
    totals: [Patch; 2],
    expires: u64,
    minutes: [u64; 2],
    blocked: BTreeMap<String, u64>,
}

impl Run {
    fn new(mut session: Session, grazer: Id, ground: Id) -> Self {
        for process in ["ecology:wake-0", "ecology:wake-3"] {
            session.sim.watch(process);
        }
        Self {
            expires: session.sim.state().tick + session.sim.genesis().rules.directing().span,
            minutes: [0; 2],
            blocked: BTreeMap::new(),
            session,
            grazer,
            ground,
            totals: Default::default(),
        }
    }

    fn advance_to(&mut self, end: u64) -> Result<(), String> {
        let s = &mut self.session;
        let founded = s.sim.genesis().started;
        while s.sim.state().tick - founded < end {
            let animal = s
                .sim
                .state()
                .population
                .get(self.grazer)
                .ok_or("missing grazer")?;
            if let Some(site) = animal.resident_site().filter(|_| animal.alive) {
                *self
                    .minutes
                    .get_mut(site as usize)
                    .ok_or("unknown grazer patch")? += 1;
            }
            if s.sim.state().tick < self.expires
                && let Some(route) = cosmograph::directing::readings::unable(&s.sim, self.grazer)
                    .into_iter()
                    .find(|r| r.process == "move:0-1")
            {
                *self.blocked.entry(route.why).or_default() += 1;
            }
            let advance = s.advance_tick_with_flows()?;
            let pop = &s.sim.state().population;
            for flow in advance.flows.iter().filter(|f| f.by_process(GRAZE)) {
                let Some(plant) = flow
                    .from
                    .0
                    .body()
                    .and_then(|id| pop.get(id))
                    .filter(|e| e.kingdom == "kingdom:flora")
                else {
                    continue;
                };
                let site = plant.resident_site().ok_or("grazed plant has no patch")?;
                let patch = self
                    .totals
                    .get_mut(site as usize)
                    .ok_or("unknown feeding patch")?;
                let quantity = flow.amount * flow.count;
                patch.grazed += quantity;
                if flow.to.0.body() == Some(self.grazer) {
                    patch.eaten_by_grazer += quantity;
                }
            }
            for act in s.sim.take_watched() {
                let plant = s
                    .sim
                    .state()
                    .population
                    .get(act.actor)
                    .ok_or("missing waking plant")?;
                if plant.alive && !s.sim.genesis().rules.dormant(plant) {
                    self.totals
                        .get_mut(plant.place as usize)
                        .ok_or("unknown waking patch")?
                        .emerged += act.count;
                }
            }
            let weather = match s.sim.state().tick - founded {
                LEAN => "experiment:begin-lean",
                t if t == 8 * DAY => "experiment:end-lean",
                _ => continue,
            };
            set_weather(s, self.ground, weather)?;
        }
        Ok(())
    }

    fn observe(&self) -> Result<Observation, String> {
        let sim = &self.session.sim;
        let state = sim.state();
        let rules = &sim.genesis().rules;
        let mut patches = self.totals;
        for group in state.population.groups.values() {
            let e = &group.entity;
            let Some(site) = e.resident_site().filter(|_| e.alive) else {
                continue;
            };
            let patch = patches
                .get_mut(site as usize)
                .ok_or("unknown resident patch")?;
            if e.lineage == "lineage:1" {
                patch.grazers += group.count;
            }
            let tissue = match e.lineage.as_str() {
                "lineage:0" => "matter:0-0",
                "lineage:3" => "matter:3-0",
                _ => continue,
            };
            if rules.dormant(e) {
                patch.seeds += group.count;
                continue;
            }
            let root = e.body.as_ref().ok_or("plant has no root")?.root;
            for (&id, part) in &e.parts {
                let matter = part.matter.get(tissue).copied().unwrap_or(0) * group.count;
                if id == root {
                    patch.roots += matter
                } else if anatomy::root_store::exposed(e, rules, id) {
                    patch.shoots += matter
                }
            }
        }
        let animal = state.population.get(self.grazer).ok_or("missing grazer")?;
        let nudge = match state.nudges.last() {
            None => "none",
            Some(n) if n.answer.is_some() => "answered",
            Some(n) if state.tick < n.tick + rules.directing().span => "pending",
            Some(_) => "expired",
        };
        Ok(Observation {
            meadow_open: state.sites[&0].conditions["experiment:growth-open"] > 0,
            hash: format!("{:016x}", sim.state_hash()),
            alive: animal.alive,
            site: animal.resident_site().filter(|_| animal.alive),
            nudge,
            minutes: self.minutes,
            blocked_route_minutes: self.blocked.clone(),
            patches,
        })
    }
}

/// Emits the opening before the click, its expiry time, and each following
/// day through day 12. Dropping the receiver stops the run at the next sample.
pub fn compare(nudge: bool, receive: impl FnMut(Pair) -> bool) -> Result<(), String> {
    compare_at(MeadowChange::Reopens, nudge, receive)
}

/// Follow the common history through the chosen weather change, then fork.
/// Residence and grazing are counted only from that fork.
pub fn compare_at(
    when: MeadowChange,
    nudge: bool,
    mut receive: impl FnMut(Pair) -> bool,
) -> Result<(), String> {
    let saved = serde_json::from_slice(OPENING).map_err(|e| e.to_string())?;
    let (mut session, _) = World::resume_native(saved, Execution::Individuals)?.into_parts()?;
    // Observation minutes start at founding. Envelopes use the world's clock.
    let opening = session.sim.state().tick - session.sim.genesis().started;
    if opening != 2 * DAY {
        return Err("expected the meadow's day-2 reopening".into());
    }
    let span = session.sim.genesis().rules.directing().span;
    let (participant, grazer) = session
        .entries
        .iter()
        .rev()
        .find_map(|entry| match entry.command {
            Command::Take {
                participant,
                critter,
            } => Some((participant, critter)),
            _ => None,
        })
        .ok_or("the meadow save has no bonded grazer")?;
    let ground = session
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.traits.contains("experiment:weather-keeper"))
        .map(|(&id, _)| id)
        .ok_or("the meadow save has no weather keeper")?;
    if when == MeadowChange::TurnsLean {
        session.advance(LEAN - opening)?;
        set_weather(&mut session, ground, "experiment:begin-lean")?;
    }
    let start = session.sim.state().tick - session.sim.genesis().started;
    let mut alone = Run::new(session.clone(), grazer, ground);
    let mut trial = Run::new(session, grazer, ground);
    for tick in [start, start + span]
        .into_iter()
        .chain((start / DAY + 1..=12).map(|day| day * DAY))
    {
        alone.advance_to(tick)?;
        trial.advance_to(tick)?;
        if !receive(Pair {
            tick,
            grazer,
            alone: alone.observe()?,
            trial: trial.observe()?,
        }) {
            break;
        }
        if nudge && tick == start {
            let envelope = MesocosmIntentEnvelope {
                participant: ParticipantHandle(participant),
                tick: Tick(trial.session.sim.state().tick),
                intent: Intent::Game(crate::runtime::keymap::nudge(
                    grazer,
                    REFUGE,
                    None,
                    NudgeMeaning::Attend,
                )),
            };
            crate::borrowed::submit(&mut trial.session, participant, 0, &envelope)
                .map_err(|e| format!("refuge nudge refused: {e:?}"))?;
        }
    }
    Ok(())
}
