// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Follow two founder IDs through a fork of an ordinary Cosmograph session.
use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    ecology::pocket::{self, CONTACT, LEAVE, TRADE},
    process::Trade::{Gives, Takes},
    schema::{Entity, Id, Relation},
    trophic,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io;
const END: u64 = 36;

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct Edit {
    seed: u64,
    pocket: bool,
    port: bool,
    leave_at: u64,
}
impl Default for Edit {
    fn default() -> Self {
        Self {
            seed: 9,
            pocket: true,
            port: true,
            leave_at: 6,
        }
    }
}

#[derive(Clone)]
struct Pair {
    host: Id,
    resident: Id,
    run: u8,
    host_tissue: String,
    sugar: String,
}
impl Pair {
    fn contact(&self, s: &Session) -> bool {
        s.sim
            .state()
            .relations
            .contains(&Relation::new(self.host, CONTACT, self.resident))
    }
    fn leave(&self, s: &mut Session) -> Result<(), String> {
        let result = s.command(Command::Act {
            actor: self.host,
            target: Some(self.resident),
            process: LEAVE.into(),
            cause: None,
        })?;
        if self.contact(s) {
            return Err(format!("resident could not leave: {result}"));
        }
        Ok(())
    }
}

#[derive(Default, Serialize, Clone)]
struct Flows {
    soil: u64,
    sugar: u64,
    fee: u64,
    digested: u64,
    fixed: u64,
}
impl Flows {
    fn step(&mut self, s: &mut Session, pair: &Pair) -> Result<(), String> {
        for f in s.advance_tick_with_flows()?.flows {
            let endpoints = (f.from.0.body(), f.to.0.body());
            let amount = f.amount * f.count;
            if f.internal()
                && endpoints.0 == Some(pair.host)
                && f.from.1 == pair.sugar
                && f.to.1 == pair.host_tissue
            {
                self.digested += amount;
            }
            if f.internal()
                && endpoints.0 == Some(pair.resident)
                && f.from.1 == "world:soil"
                && f.to.1 == pair.sugar
            {
                self.fixed += amount;
            }
            if !f.by_process(TRADE) {
                continue;
            }
            if endpoints == (Some(pair.host), Some(pair.resident)) && f.to.1 == "world:soil" {
                self.soil += amount;
            }
            if endpoints == (Some(pair.resident), Some(pair.host)) && f.to.1 == pair.sugar {
                self.sugar += amount;
            }
            if endpoints == (Some(pair.host), None)
                && f.from.1 == pair.host_tissue
                && f.to.1 == "world:soil"
            {
                self.fee += amount;
            }
        }
        Ok(())
    }
}
fn ports(e: &Entity) -> Vec<u32> {
    e.living()
        .filter(|(_, p)| {
            cosmograph::mosaic::ports::active(p).is_some_and(|v| v.trades(Gives) && v.trades(Takes))
        })
        .map(|(id, _)| id.0)
        .collect()
}
fn body(s: &Session, id: Id, pair: &Pair, own: &str) -> Result<Value, String> {
    let e = s
        .sim
        .state()
        .population
        .get(id)
        .ok_or("tracked founder missing")?;
    let rules = &s.sim.genesis().rules;
    let parts: Vec<_> = e
        .living()
        .filter_map(|(id, _)| {
            let g = e.geo(id)?;
            Some(
                json!({"id":id.0,"center":e.body.as_ref()?.world_pivot(id)?, "half":g.half_extent}),
            )
        })
        .collect();
    let offspring = s
        .sim
        .state()
        .relations
        .iter()
        .filter(|r| r.kind == "sim:parent" && r.object == id)
        .count();
    Ok(
        json!({"id":id, "alive":e.alive, "offspring":offspring,"age":s.sim.state().tick.saturating_sub(e.born),
        "tissue":anatomy::held(e,rules,own),"soil":anatomy::held(e,rules,"world:soil"),
        "sugar":anatomy::held(e,rules,&pair.sugar),"port":ports(e),"frond":trophic::fixing_plate(e),
        "opening":pocket::opening(e,pair.run),"parts":parts}),
    )
}
fn frame(s: &Session, pair: &Pair, flows: &Flows) -> Result<Value, String> {
    let state = s.sim.state();
    let host = state.population.get(pair.host).ok_or("missing host")?;
    let site = state.sites.get(&host.place).ok_or("missing site")?;
    let stock = |k: &str| site.accounts.get(k).copied().unwrap_or(0);
    Ok(
        json!({"tick":state.tick,"hash":format!("{:016x}",s.sim.state_hash()),"contact":pair.contact(s),
        "host":body(s,pair.host,pair,&pair.host_tissue)?,"resident":body(s,pair.resident,pair,&pair.sugar)?,
        "flows":{"soil":flows.soil,"sugar":flows.sugar,"fee":flows.fee},
        "digested":flows.digested,"fixed":flows.fixed,"site":{"id":host.place,"soil":stock("world:soil"),"light":stock("sim:energy"),
        "habitable":site.conditions.get("world:habitable").copied().unwrap_or(0)>0}}),
    )
}

fn run() -> Result<(), String> {
    let edits: Edit = serde_json::from_reader(io::stdin().lock()).map_err(|e| e.to_string())?;
    if edits.leave_at > END {
        return Err("departure outside the preview".into());
    }
    let mut args = std::env::args().skip(1);
    let selected = match args.next().as_deref() {
        None => None,
        Some("--save") => {
            let branch = args.next().ok_or("missing path")?;
            let tick: u64 = args
                .next()
                .ok_or("missing tick")?
                .parse()
                .map_err(|_| "invalid tick")?;
            let hash = args.next().ok_or("missing hash")?;
            if !matches!(branch.as_str(), "stay" | "leave") || tick > END {
                return Err("invalid selection".into());
            }
            Some((branch, tick, hash))
        },
        _ => return Err("unknown option".into()),
    };
    let mut founding: Founding =
        toml::from_str(include_str!("pocket.toml")).map_err(|e| e.to_string())?;
    founding.seed = edits.seed;
    let cfg = founding.pocket.ok_or("missing pocket founding")?;
    let host_line = format!("lineage:{}", cfg.host);
    let resident_line = format!("lineage:{}", cfg.resident);
    if !edits.pocket {
        let map = founding
            .body_maps
            .get_mut(&host_line)
            .ok_or("missing host map")?;
        let floor = &mut map.grammar.seed[usize::from(cfg.run)];
        floor.bears = None;
        floor.per_segment = 0;
        map.regions.clear();
    }
    if !edits.port {
        let map = founding
            .body_maps
            .get_mut(&resident_line)
            .ok_or("missing resident map")?;
        map.grammar.seed[1].segment = "kind:block".into();
        map.regions.clear();
    }
    let mut stay = Session::new(founding.generate()?, Execution::Grouped)?;
    let founder = |line: &str| {
        stay.sim
            .state()
            .population
            .groups
            .iter()
            .find(|(_, g)| g.entity.lineage == line)
            .map(|(id, _)| *id)
            .ok_or("unstocked founder")
    };
    let pair = Pair {
        host: founder(&host_line)?,
        resident: founder(&resident_line)?,
        run: cfg.run,
        host_tissue: format!("matter:{}-0", cfg.host),
        sugar: format!("matter:{}-0", cfg.resident),
    };
    let mut stay_flows = Flows::default();
    let mut alternative: Option<(Session, Flows)> = None;
    let mut stays = Vec::new();
    let mut leaves = Vec::new();
    for tick in 0..=selected.as_ref().map_or(END, |s| s.1) {
        if tick == edits.leave_at {
            let mut fork = stay.fork_at(tick, format!("pocket-leave-{tick}"))?;
            if pair.contact(&fork)
                && [pair.host, pair.resident].iter().all(|id| {
                    fork.sim
                        .state()
                        .population
                        .get(*id)
                        .is_some_and(|e| e.alive)
                })
            {
                pair.leave(&mut fork)?;
            }
            alternative = Some((fork, stay_flows.clone()));
        }
        if let Some((branch, until, witness)) = &selected {
            if tick == *until {
                let chosen = if branch == "leave" {
                    alternative.as_ref().map_or(&stay, |v| &v.0)
                } else {
                    &stay
                };
                if format!("{:016x}", chosen.sim.state_hash()) != *witness {
                    return Err("selected state changed during replay".into());
                }
                let bytes = serde_json::to_vec(&chosen.save()).map_err(|e| e.to_string())?;
                let reopened = Session::load_json(&bytes, Execution::Grouped)?;
                if reopened.sim.state_hash() != chosen.sim.state_hash() {
                    return Err("reopened state differs".into());
                }
                return std::io::Write::write_all(&mut io::stdout().lock(), &bytes)
                    .map_err(|e| e.to_string());
            }
        } else {
            let current = frame(&stay, &pair, &stay_flows)?;
            leaves.push(match &alternative {
                Some((s, f)) => frame(s, &pair, f)?,
                None => current.clone(),
            });
            stays.push(current);
        }
        if tick < END {
            stay_flows.step(&mut stay, &pair)?;
            if let Some((s, f)) = &mut alternative {
                f.step(s, &pair)?;
            }
        }
    }
    let data = json!({"seed":edits.seed.to_string(),"options":edits,"terms":founding.mutualism,
        "source":toml::to_string(&founding).map_err(|e|e.to_string())?,"frames":stays,"leave":leaves});
    serde_json::to_writer(io::stdout().lock(), &data).map_err(|e| e.to_string())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
