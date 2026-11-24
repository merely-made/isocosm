// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Observe a physical sibling of a planet founding through the ordinary Mesocosm spatial clock.
#[path = "planet_ground/departure.rs"]
mod departure;
#[path = "planet_ground/field.rs"]
mod field;
#[path = "../../../../../shared/cosmograph/examples/planet_atlas/observation.rs"]
mod observation;
#[path = "planet_ground/soil.rs"]
mod soil;
#[path = "planet_ground/recruitment.rs"]
mod recruitment;
use cosmograph::{
    Command, Execution, Session,
    attention::{Region, Target, WindowAttention},
    spatial::embodiment,
};
use cosmograph_host::drive::{STEPS_PER_MINUTE, world_foreground_walks};
use isometer_space::Atlas;
use mesocosm_runtime::spatial::Game;
use observation::Result;
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
};

struct Branch {
    session: Session,
    game: Game,
    actor: u64,
    field_site: Option<u64>,
    observer: u64,
}

fn boxes(
    entity: &cosmograph::schema::Entity,
    pose: &cosmograph::spatial::Pose,
) -> Result<Vec<Value>> {
    let doc = entity.body.as_ref().ok_or("ground resident has no body")?;
    let bounds = embodiment::part_boxes(doc, pose)?;
    Ok(entity
        .living()
        .filter_map(|(id, t)| {
            let [lo, hi] = bounds.get(&id)?;
            let role = if t.port.is_some() && t.cells.contains_key("function:intake") {
                "mouth"
            } else if t.cells.contains_key("function:sense") {
                "sense"
            } else if t.cells.contains_key("function:move") {
                "limb"
            } else {
                "tissue"
            };
            Some(json!([id.0, lo, hi, role]))
        })
        .collect())
}

impl Branch {
    fn found(
        parent: &Path,
        actor: Option<u64>,
        field_site: Option<u64>,
        litter: Option<soil::Survey>,
    ) -> Result<Self> {
        let parent = Session::load_json(&fs::read(parent)?, Execution::Grouped)?;
        if parent.sim.state().tick != 0 {
            return Err("a founding save is required".into());
        }
        let mut g = parent.sim.genesis().clone();
        let site = field_site
            .or_else(|| g.population.get(actor?).and_then(|e| e.resident_site()))
            .ok_or("resident missing at founding")?;
        let parent_digest = parent.save().genesis_digest;
        g.world.parent_world = Some(parent_digest.clone());
        embodiment::declare_site_at(&mut g, site)?;
        if let Some(reachable) = litter {
            soil::found(&mut g, site, reachable)?;
        }
        g.rules.foreground_walks = true;
        let side = g.world.footprint.ok_or("missing site extent")?.side as i64;
        let mut session = Session::new(g, Execution::Grouped)?;
        session.branch = match actor {
            Some(id) => format!("ground-{id}-{}", &parent_digest[..12]),
            None if litter.is_some() => format!(
                "soil-{site}-{}-{}",
                litter.expect("soil survey").label(),
                &parent_digest[..12]
            ),
            None => format!("field-{site}-{}", &parent_digest[..12]),
        };
        let mut game = Game::default();
        if !game.found_generated(&mut session)? {
            return Err("physical founding unavailable".into());
        }
        let observer = session
            .command(Command::Join)?
            .strip_prefix("participant:")
            .ok_or("observer absent")?
            .parse()?;
        session.attach_window(
            observer,
            "mesocosm".into(),
            WindowAttention {
                examined: Some(Region::Frame {
                    site,
                    min: [0; 2],
                    max: [side; 2],
                }),
                pinned: actor.into_iter().map(Target::Entity).collect(),
                ..Default::default()
            },
        )?;
        Ok(Self {
            session,
            game,
            observer,
            actor: actor.unwrap_or_default(),
            field_site,
        })
    }

    fn frame(&self, flows: &[cosmograph::flows::Flow]) -> Result<Value> {
        let mut f = match self.field_site {
            Some(site) => field::frame(self, flows, site)?,
            None => observation::frame(&self.session, flows)?,
        };
        let sim = &self.session.sim;
        let mut poses = vec![];
        for (&id, r) in &sim.state().spatial_poses {
            let Some(e) = sim.state().population.get(id) else {
                continue;
            };
            if !e.alive || e.resident_site() != Some(r.site) || e.arrived != r.arrived {
                continue;
            }
            poses.push(
                json!({"id":id,"site":r.site,"pose":r.pose,"travel":r.travel,"kingdom":e.kingdom,
                "boxes":boxes(e,&r.pose)?}),
            );
        }
        if self.field_site.is_some() {
            for (&site, bed) in &sim.genesis().rules.soil_beds {
                f["sites"][site as usize]["soil"] = soil::observe(sim, site, bed, &mut poses)?;
            }
        }
        f["poses"] = json!(poses);
        if self.field_site.is_none() {
            f["observer"] = json!(self.actor);
        }
        f["branch"] = json!(self.session.branch);
        Ok(f)
    }

    fn follow_arrival(&mut self) -> Result<()> {
        let sim = &self.session.sim;
        let Some(e) = sim.state().population.get(self.actor).filter(|e| e.alive) else {
            return Ok(());
        };
        let destination = e
            .journey
            .filter(|j| j.arrives <= sim.state().tick + 1)
            .map(|j| j.to)
            .or_else(|| e.resident_site());
        let Some(site) = destination else {
            return Ok(());
        };
        self.examine(site)
    }

    fn examine(&mut self, site: u64) -> Result<()> {
        let sim = &self.session.sim;
        if !sim.state().sites.contains_key(&site) {
            return Err("unknown site".into());
        }
        let window = sim
            .window_for(self.observer, "mesocosm")
            .ok_or("observation window missing")?;
        let (_, profile, mut attention) = sim.attention_window(self.observer, window.id())?;
        if matches!(attention.examined,Some(Region::Frame{site:s,..}) if s==site) {
            return Ok(());
        }
        let side = sim
            .genesis()
            .world
            .footprint
            .ok_or("missing site extent")?
            .side as i64;
        attention.examined = Some(Region::Frame {
            site,
            min: [0; 2],
            max: [side; 2],
        });
        self.session
            .replace_attention(self.observer, window, profile, attention)?;
        Ok(())
    }

    fn next(&mut self) -> Result<Value> {
        if self.field_site.is_none() {
            self.follow_arrival()?;
        }
        let walks = world_foreground_walks(&self.session.sim, self.observer);
        let mut handbacks = vec![];
        let mut paths = vec![];
        for w in walks {
            if let Some(h) = w.handback(&self.session.sim, STEPS_PER_MINUTE)? {
                let end = w.cells(STEPS_PER_MINUTE);
                let entity = self
                    .session
                    .sim
                    .state()
                    .population
                    .get(h.actor)
                    .ok_or("walker absent")?;
                let states = w.route().path[..=end]
                    .iter()
                    .map(|p| boxes(entity, p))
                    .collect::<Result<Vec<_>>>()?;
                paths.push((h.actor,json!({"id":h.actor,"site":h.site,"target":w.route().target,
                    "path":w.route().path[..=end],"contacts":h.contacts,"boxes":states,"steps":(0..=STEPS_PER_MINUTE).map(|n|{
                        let st=w.at(n);[st.edge as u64, u64::from(st.fraction)]
                    }).collect::<Vec<_>>() })));
                handbacks.push(h);
            }
        }
        // Stage the animal's autonomous choice. A just-departing observed body
        // can walk toward the chosen exit before the final biological round.
        let mut candidate = self.session.clone();
        let mut game = self.game.clone();
        let mut out = game.advance_clock_round(&mut candidate, &handbacks)?;
        let trip = candidate
            .sim
            .state()
            .population
            .get(self.actor)
            .and_then(|e| e.journey)
            .filter(|j| j.departed == candidate.sim.state().tick);
        if self.field_site.is_none()
            && let Some(trip) = trip
            && !handbacks.iter().any(|h| h.actor == self.actor)
            && let Some((exit, path)) = departure::walk(&self.session.sim, self.actor, trip)?
        {
            let entity = self
                .session
                .sim
                .state()
                .population
                .get(self.actor)
                .ok_or("walker absent")?;
            let states = path
                .iter()
                .map(|p| boxes(entity, p))
                .collect::<Result<Vec<_>>>()?;
            handbacks.push(exit);
            let mut trial = self.session.clone();
            let mut trial_game = self.game.clone();
            let moved = trial_game.advance_clock_round(&mut trial, &handbacks)?;
            eprintln!(
                "exit toward {}: {} strides, native response {:?}",
                trip.to,
                path.len() - 1,
                trial.sim.spatial_handbacks()
            );
            if trial
                .sim
                .spatial_handbacks()
                .iter()
                .any(|h| h.actor == self.actor && h.patch.is_ok())
            {
                paths.push((
                    self.actor,
                    json!({"id":self.actor,"site":trip.from,"exit":trip.to,
                    "path":path,"boxes":states}),
                ));
                candidate = trial;
                game = trial_game;
                out = moved;
            }
        }
        self.session = candidate;
        self.game = game;
        let accepted = self.session.sim.spatial_handbacks();
        let mut f = self.frame(&out.flows)?;
        f["walks"] = json!(
            paths
                .into_iter()
                .filter(|(id, _)| accepted.iter().any(|h| h.actor == *id && h.patch.is_ok()))
                .map(|(_, p)| p)
                .collect::<Vec<_>>()
        );
        // A native single-step choice can also have walked an edge during the minute.
        if let Some(entry) = self
            .session
            .entries
            .iter()
            .rev()
            .find(|e| matches!(e.command, Command::AdvanceBirthSpatial { .. }))
        {
            if let Command::AdvanceBirthSpatial { steps, .. } = &entry.command {
                for step in steps
                    .iter()
                    .filter(|s| self.session.sim.spatial_steps().contains(&s.request.action))
                {
                    let Ok(geom) = &step.geometry else { continue };
                    let Some(entity) = self.session.sim.state().population.get(step.request.actor)
                    else {
                        continue;
                    };
                    let states = geom
                        .path
                        .iter()
                        .map(|p| boxes(entity, p))
                        .collect::<Result<Vec<_>>>()?;
                    f["walks"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"id":step.request.actor,"site":step.request.site,
                        "target":step.request.target,"path":geom.path,"boxes":states}));
                }
            }
        }
        Ok(f)
    }

    fn terrain(&self) -> Result<Value> {
        let mut t = observation::terrain(&self.session)?;
        let sim = &self.session.sim;
        let site = self
            .field_site
            .or_else(|| {
                sim.state()
                    .population
                    .get(self.actor)
                    .and_then(|e| e.resident_site())
            })
            .ok_or("missing focus")?;
        let view = cosmograph::terrain::View::of(sim.genesis())?;
        let side = view.footprint().side;
        let samples = side.min(256);
        for id in std::iter::once(site)
            .chain((0..4).filter_map(|s| view.border(site, s).map(|(other, _)| other)))
        {
            let lattice = view.lattice(id)?;
            let mut heights = Vec::new();
            for z in 0..=samples {
                for x in 0..=samples {
                    heights.push(
                        (16. * lattice.numerator(x * side / samples, z * side / samples) as f64
                            / lattice.denominator() as f64)
                            .round() as i32,
                    );
                }
            }
            t["sites"][id as usize]["ground"] = json!({"samples":samples,"heights":heights});
        }
        Ok(t)
    }

    fn save(&self) -> Value {
        let mut saved = json!({"native":self.session.save(),"game":self.game,"observer":self.observer,"actor":self.actor});
        if let Some(site) = self.field_site {
            saved["fieldSite"] = json!(site);
        }
        saved
    }
    fn reopen(path: &Path) -> Result<Self> {
        let v: Value = serde_json::from_slice(&fs::read(path)?)?;
        let session = Session::load_json(&serde_json::to_vec(&v["native"])?, Execution::Grouped)?;
        let game: Game = serde_json::from_value(v["game"].clone())?;
        game.validate_replay(&session)?;
        Ok(Self {
            session,
            game,
            field_site: v["fieldSite"].as_u64(),
            actor: v["actor"].as_u64().ok_or("missing actor")?,
            observer: v["observer"].as_u64().ok_or("missing observer")?,
        })
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, parent, save] if mode == "seasonal" => {
            recruitment::seasonal_parent(Path::new(parent), Path::new(save))?;
        },
        [mode, parent, site, dir] if mode == "field" => {
            field::found(Path::new(parent), site.parse()?, Path::new(dir), None)?
        },
        [mode, parent, site, treatment, dir] if mode == "soil" || mode == "nursery" => {
            let near = match treatment.as_str() {
                "near" => true,
                "far" => false,
                _ => return Err("choose near or far litter".into()),
            };
            field::found(Path::new(parent), site.parse()?, Path::new(dir), Some(if mode == "nursery" { soil::Survey::Nurse(near) } else { soil::Survey::Litter(near) }))?;
        },
        [mode, parent, actor, dir, n] if mode == "branch" => {
            let dir = Path::new(dir);
            let ticks = n.parse::<u32>()?;
            fs::create_dir_all(dir)?;
            if fs::read_dir(dir)?.next().is_some() {
                return Err("the branch output directory must be empty".into());
            }
            let mut b = Branch::found(Path::new(parent), Some(actor.parse()?), None, None)?;
            let write = |name: &str, v: &Value| -> Result<()> {
                fs::write(dir.join(name), serde_json::to_vec(v)?)?;
                Ok(())
            };
            write("founding.json", &b.save())?;
            write("terrain.json", &b.terrain()?)?;
            let mut frames = vec![b.frame(&[])?];
            for _ in 0..ticks {
                eprintln!(
                    "advancing from tick {} with {} poses",
                    b.session.sim.state().tick,
                    b.session.sim.state().spatial_poses.len()
                );
                let frame = b.next()?;
                let actor = frame["animals"]
                    .as_array()
                    .and_then(|animals| animals.iter().find(|a| a["id"] == b.actor));
                let pose = frame["poses"]
                    .as_array()
                    .and_then(|poses| poses.iter().find(|p| p["id"] == b.actor));
                eprintln!(
                    "tick {}: #{} at {}, food {} from {}, walked {}",
                    frame["tick"],
                    b.actor,
                    pose.map_or(&Value::Null, |p| &p["pose"]["at"]),
                    actor.map_or(&Value::Null, |a| &a["fed"]),
                    actor.map_or(&Value::Null, |a| &a["sources"]),
                    frame["walks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|w| w["id"] == b.actor)
                        .map(|w| w["path"].as_array().unwrap().len() - 1)
                        .sum::<usize>()
                );
                frames.push(frame);
            }
            write("frames.json", &json!(frames))?;
            write("resume.json", &b.save())?;
        },
        [mode, file] if mode == "watch" => {
            let mut b = Branch::reopen(Path::new(file))?;
            let stdout = io::stdout();
            let mut out = io::BufWriter::new(stdout.lock());
            for line in io::stdin().lock().lines() {
                let f = match line?.as_str() {
                    "now" => b.frame(&[])?,
                    cmd if cmd.starts_with("family ") => {
                        recruitment::family(&b.session, cmd[7..].parse()?)?
                    },
                    "next" => b.next()?,
                    visit if b.field_site.is_some() && visit.starts_with("visit ") => {
                        let site = visit[6..].parse()?;
                        b.examine(site)?;
                        b.field_site = Some(site);
                        let frame = b.next()?;
                        json!({"frame":frame,"save":b.save()})
                    },
                    _ => return Err("expected next, now, family ID or visit SITE".into()),
                };
                serde_json::to_writer(&mut out, &f)?;
                writeln!(out)?;
                out.flush()?;
            }
        },
        _ => {
            return Err(
                "usage: planet_ground branch PARENT ACTOR DIRECTORY TICKS | field PARENT SITE DIRECTORY | soil PARENT SITE near|far DIRECTORY | nursery PARENT SITE near|far DIRECTORY | seasonal PARENT NEW_SAVE | watch SAVE".into(),
            );
        },
    };
    Ok(())
}
