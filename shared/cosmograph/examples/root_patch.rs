// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Seed 9 at the boundary between a rooted plant and fungal-accessible litter.
//! Run with an output directory to retain both saves and their measurements.
use cosmograph::{
    Command, Execution, Founding, Result, Session, anatomy,
    process::Trade::{Gives, Takes},
    rules::{AccountKind, Binding, Causation, Effect, Process, Query, Target},
    schema::{Id, Relation},
    simulation::Genesis,
};
use serde::Serialize;
use std::{fs, path::Path};

const FREE: &str = "patch:phosphate";
const BOUND: &str = "patch:litter-phosphorus";
const USED: &str = "patch:incorporated-phosphorus";
const CARBON: &str = "matter:0-0";
const CONTACT: &str = "root:colonized-by";
const ROOT: &str = "root:uptake";
const RECOVER: &str = "litter:mineralize";
const TRADE: &str = "root:exchange";
const END: u64 = 24;
const FORK: u64 = 6;
const TOTAL: u64 = 24;

fn has(who: Binding, key: &str) -> Query {
    Query::Account {
        who,
        key: key.into(),
        at_least: 1,
    }
}
fn transfer(from: Binding, to: Binding, key: &str) -> Effect {
    Effect::Transfer {
        from,
        to,
        account: key.into(),
        amount: 1.into(),
    }
}
fn convert(who: Binding, from: &str, to: &str) -> Effect {
    Effect::Transform {
        who,
        take: [(from.into(), 1)].into(),
        give: [(to.into(), 1)].into(),
        conversion: None,
    }
}
fn turn(
    id: &str,
    lineage: u32,
    period: Option<u64>,
    priority: i32,
    effects: Vec<Effect>,
) -> Process {
    Process {
        id: id.into(),
        causation: if period.is_some() {
            Causation::Transition
        } else {
            Causation::Choice
        },
        requires: vec![
            Query::Alive(Binding::Actor),
            Query::Trait {
                who: Binding::Actor,
                key: format!("ability:cycle-{lineage}"),
            },
        ],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period,
        priority,
        need_account: None,
        need_below: 0,
        glyphs: Default::default(),
        invariants: ["sim:matter".into(), "sim:nonnegative".into()].into(),
        note: false,
        port_feeding: None,
        goal_directives: vec![],
        turn: None,
    }
}

struct Patch {
    genesis: Genesis,
    plant: Id,
    fungus: Id,
    root: Id,
    litter: Id,
}
impl Patch {
    fn new(root_p: u64, start_hour: u32, light_period: u64) -> Result<Self> {
        use Binding::{Actor, Place, Target as Partner};
        let mut f: Founding =
            toml::from_str(include_str!("pocket.toml")).map_err(|e| e.to_string())?;
        f.sites = 2;
        f.pocket = None;
        let mut g = f.generate()?;
        g.rules.sun = Some(cosmograph::light::SunRules {
            start_hour,
            ..Default::default()
        });
        let founder = |line: &str| {
            g.population
                .groups
                .iter()
                .find(|(_, group)| group.entity.lineage == line)
                .map(|(&id, group)| (id, group.entity.place))
                .ok_or("missing founder")
        };
        let (plant, root) = founder("lineage:0")?;
        let (fungus, litter) = founder("lineage:2")?;
        if root == litter || !g.sites[&root].routes.iter().any(|edge| edge.to == litter) {
            return Err("the trial requires founders on adjacent, distinct sites".into());
        }
        // A vegetative window. Keep native photosynthesis, upkeep, mortality and fungal
        // digestion; founding's pocket-wide gifts and reproduction would confound the pair.
        g.rules.processes.retain(|id, _| {
            (!id.starts_with("mutualism:") || id == "mutualism:digest-2-0")
                && !id.starts_with("ecology:birth-")
        });
        for key in [FREE, BOUND, USED] {
            g.rules.accounts.insert(
                key.into(),
                AccountKind::Matter {
                    lineage: "world:ground".into(),
                    reserve: false,
                    provision: false,
                },
            );
        }
        g.rules.relations.insert(CONTACT.into());
        g.sites
            .get_mut(&root)
            .unwrap()
            .accounts
            .insert(FREE.into(), root_p);
        g.sites
            .get_mut(&litter)
            .unwrap()
            .accounts
            .insert(BOUND.into(), TOTAL - root_p);
        for site in g.sites.values_mut() {
            site.accounts.remove("sim:energy");
        }
        // A controlled light supply for the same bodies and phosphorus distribution.
        let sun = g
            .rules
            .processes
            .get_mut("ecology:sunlight")
            .ok_or("no native sunlight")?;
        sun.period = Some(light_period);
        sun.effects = vec![Effect::Sunlight {
            who: Place,
            account: "sim:energy".into(),
            full: 1,
        }];

        let growth = g
            .rules
            .processes
            .get_mut("ecology:produce-0")
            .ok_or("no native plant growth")?;
        growth.requires.push(has(Actor, FREE));
        growth.effects.push(convert(Actor, FREE, USED));

        let mut absorb = turn(ROOT, 0, Some(2), -3, vec![transfer(Place, Actor, FREE)]);
        absorb.requires.extend([
            has(Place, FREE),
            Query::Trades {
                who: Actor,
                what: Takes,
            },
        ]);
        let mut recover = turn(
            RECOVER,
            2,
            Some(2),
            -3,
            vec![transfer(Place, Actor, BOUND), convert(Actor, BOUND, FREE)],
        );
        recover.requires.extend([
            has(Place, BOUND),
            Query::Trades {
                who: Actor,
                what: Gives,
            },
        ]);

        let related = Query::Related {
            kind: CONTACT.into(),
        };
        let link = |present| Effect::Relate {
            kind: CONTACT.into(),
            present,
        };
        let mut attach = turn("root:attach", 0, None, 0, vec![link(true)]);
        let mut detach = turn("root:detach", 0, None, 0, vec![link(false)]);
        detach.requires.push(related.clone());
        let mut trade = turn(
            TRADE,
            0,
            Some(2),
            -1,
            vec![
                transfer(Partner, Actor, FREE),
                transfer(Actor, Partner, CARBON),
            ],
        );
        trade
            .requires
            .extend([related, has(Partner, FREE), has(Actor, CARBON)]);
        for p in [&mut attach, &mut trade] {
            p.requires.extend([
                Query::Trades {
                    who: Actor,
                    what: Takes,
                },
                Query::Trades {
                    who: Partner,
                    what: Gives,
                },
            ]);
        }
        for p in [&mut attach, &mut detach, &mut trade] {
            p.target = Some(Target {
                same_place: false,
                alive: Some(true),
                lineage: None,
                among: ["lineage:2".into()].into(),
                weighted: false,
            });
        }
        for p in [absorb, recover, attach, detach, trade] {
            g.rules.processes.insert(p.id.clone(), p);
        }
        Ok(Self {
            genesis: g,
            plant,
            fungus,
            root,
            litter,
        })
    }

    fn change_contact(&self, s: &mut Session, attach: bool) -> Result<()> {
        let process = if attach { "root:attach" } else { "root:detach" };
        let outcome = s.command(Command::Act {
            actor: self.plant,
            target: Some(self.fungus),
            process: process.into(),
            cause: None,
        })?;
        if s.sim
            .state()
            .relations
            .contains(&Relation::new(self.plant, CONTACT, self.fungus))
            != attach
        {
            return Err(format!("{process}: {outcome}"));
        }
        Ok(())
    }

    fn frame(&self, s: &Session, flows: &Totals) -> Result<serde_json::Value> {
        let state = s.sim.state();
        let body = |id| state.population.get(id).ok_or("missing founder");
        let held = |id, key| Ok::<_, String>(anatomy::held(body(id)?, &s.sim.genesis().rules, key));
        let at = |site: Id, key: &str| state.sites[&site].accounts.get(key).copied().unwrap_or(0);
        let direct = at(self.root, FREE);
        let litter = at(self.litter, BOUND);
        let mobile = held(self.fungus, FREE)?;
        let stored = held(self.plant, FREE)?;
        let used = held(self.plant, USED)?;
        if direct + litter + mobile + stored + used != TOTAL {
            return Err(format!(
                "phosphorus balance at tick {}: {direct}+{litter}+{mobile}+{stored}+{used}",
                state.tick
            ));
        }
        Ok(
            serde_json::json!({"tick":state.tick,"hash":format!("{:016x}",s.sim.state_hash()),
            "plant_tissue":held(self.plant,CARBON)?,"fungus_tissue":held(self.fungus,"matter:2-0")?,
            "plant_alive":body(self.plant)?.alive,"fungus_alive":body(self.fungus)?.alive,
            "attached":state.relations.contains(&Relation::new(self.plant,CONTACT,self.fungus)),
            "root_light":at(self.root,"sim:energy"),"root_zone_p":direct,"litter_bound_p":litter,"fungus_available_p":mobile,
            "plant_available_p":stored,"plant_incorporated_p":used,"flows":flows}),
        )
    }
}

#[derive(Default, Clone, Serialize)]
struct Totals {
    root_uptake: u64,
    recovered: u64,
    received_p: u64,
    paid_sugar: u64,
    produced: u64,
}
impl Totals {
    fn step(&mut self, s: &mut Session) -> Result<()> {
        for f in s.advance_tick_with_flows()?.flows {
            let n = f.amount * f.count;
            if f.by_process(ROOT) && f.to.1 == FREE {
                self.root_uptake += n;
            }
            if f.by_process(RECOVER) && f.internal() && f.to.1 == FREE {
                self.recovered += n;
            }
            if f.by_process(TRADE) && f.to.1 == FREE {
                self.received_p += n;
            }
            if f.by_process(TRADE) && f.to.1 == CARBON {
                self.paid_sugar += n;
            }
            if f.by_process("ecology:produce-0") && f.to.1 == CARBON {
                self.produced += n;
            }
        }
        Ok(())
    }
}
fn write_json(path: &Path, data: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(data).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn trial(root_p: u64, start_hour: u32, light: u64, directory: &Path) -> Result<serde_json::Value> {
    let patch = Patch::new(root_p, start_hour, light)?;
    let mut stay = Session::new(patch.genesis.clone(), Execution::Grouped)?;
    patch.change_contact(&mut stay, true)?;
    let mut totals = Totals::default();
    for _ in 0..FORK {
        totals.step(&mut stay)?;
    }
    let common = patch.frame(&stay, &totals)?;
    let mut leave = stay.fork_at(FORK, "root:detached".into())?;
    if stay.sim.state_hash() != leave.sim.state_hash() {
        return Err("fork differs before detachment".into());
    }
    patch.change_contact(&mut leave, false)?;
    let mut report = serde_json::json!({"seed":9,"root_p":root_p,"litter_p":TOTAL-root_p,
        "start_hour":start_hour,"light_every":light,"fork":common,"plant_id":patch.plant,"fungus_id":patch.fungus});
    for (name, mut branch) in [("attached", stay), ("detached", leave)] {
        let path = directory.join(format!(
            "hour-{start_hour}_light-{light}_root-{root_p}_{name}.json"
        ));
        write_json(&path, &branch.save())?;
        let mut reopened = Session::load_json(
            &fs::read(&path).map_err(|e| e.to_string())?,
            Execution::Grouped,
        )?;
        let mut f = totals.clone();
        let mut frames = vec![patch.frame(&branch, &f)?];
        while branch.sim.state().tick < END {
            f.step(&mut branch)?;
            reopened.advance(1)?;
            if branch.sim.state_hash() != reopened.sim.state_hash() {
                return Err("resumed branch diverged".into());
            }
            frames.push(patch.frame(&branch, &f)?);
        }
        report[name] = serde_json::json!(frames);
    }
    Ok(report)
}
fn run() -> Result<()> {
    let dir = std::env::args_os()
        .nth(1)
        .ok_or("usage: root_patch OUTPUT_DIRECTORY")?;
    let dir = Path::new(&dir);
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut runs = vec![];
    for (start_hour, light) in [(12, 2), (12, 4), (0, 2)] {
        for root_p in 0..=TOTAL {
            runs.push(trial(root_p, start_hour, light, dir)?);
        }
    }
    write_json(&dir.join("trials.json"), &runs)
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
