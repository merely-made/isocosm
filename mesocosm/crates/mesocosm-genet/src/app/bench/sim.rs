// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A host over the simulation. Rendering and controls never mutate its world directly.
use super::{state::Bench, view::Child};
use cambium::{TextInput, clickable, el, focusable, lens, text, text_field_typed};
use isocosm::{
    Execution, Founding, Session,
    history::{Command, Merge, Saved},
    schema::Method,
};

pub(super) struct Panel {
    pub open: bool,
    pub playing: bool,
    pub ecology: bool,
    pub seed: TextInput,
    pub population: TextInput,
    pub sites: TextInput,
    pub lineages: TextInput,
    pub step: TextInput,
    pub path: TextInput,
    pub individual: TextInput,
    pub session: Option<Session>,
    pub selected: u64,
    pub other: Option<Session>,
    pub proposal: Option<Merge>,
    pub compared: Option<bool>,
    pub branches: u64,
    pub notice: String,
}

impl Panel {
    pub fn new(seed: u64) -> Self {
        Self {
            open: false,
            playing: false,
            ecology: false,
            seed: TextInput::new(seed.to_string()),
            population: TextInput::new("256"),
            sites: TextInput::new("6"),
            lineages: TextInput::new("4"),
            step: TextInput::new("1"),
            path: TextInput::new(""),
            individual: TextInput::new("0"),
            session: None,
            selected: 0,
            other: None,
            proposal: None,
            compared: None,
            branches: 0,
            notice: String::new(),
        }
    }
}

impl Bench {
    pub fn open_sim(&mut self) {
        self.sim.open = true;
        self.effects.playing = false;
        self.model.borrow_mut().pause_trial();
        self.model.borrow_mut().spatial.playing = false;
        if self.sim.session.is_none() {
            self.sim_found();
        }
    }
    pub fn sim_found(&mut self) {
        self.sim.compared = None;
        let result = (|| {
            let founding = Founding {
                seed: parse(&self.sim.seed)?,
                population: parse(&self.sim.population)?,
                sites: u32::try_from(parse(&self.sim.sites)?).map_err(|e| e.to_string())?,
                lineages: u32::try_from(parse(&self.sim.lineages)?).map_err(|e| e.to_string())?,
                ecology: self.sim.ecology,
                ..Default::default()
            };
            Session::new(founding.generate()?, Execution::Grouped)
        })();
        match result {
            Ok(session) => {
                self.sim.selected = first_critter(&session);
                self.sim.individual = TextInput::new(self.sim.selected.to_string());
                self.sim.session = Some(session);
                self.sim.other = None;
                self.sim.proposal = None;
                self.sim.notice =
                    "World founded. Choose a site or an individual to inspect.".into();
            },
            Err(why) => self.sim.notice = why,
        }
        self.sim.playing = false;
    }
    pub fn sim_step(&mut self) {
        self.sim.compared = None;
        let result = parse(&self.sim.step).and_then(|ticks| {
            self.sim
                .session
                .as_mut()
                .ok_or_else(|| "Found a world first.".to_string())?
                .advance(ticks)
        });
        match result {
            Ok(work) => {
                self.sim.notice = format!(
                    "{} process evaluations, {} represented acts, {} accepted.",
                    work.evaluations, work.represented, work.accepted
                )
            },
            Err(why) => {
                self.sim.notice = why;
                self.sim.playing = false;
            },
        }
        self.sim.proposal = None;
    }
    fn sim_command(&mut self, command: Command) {
        self.sim.compared = None;
        if let Some(session) = &mut self.sim.session {
            self.sim.notice = match session.command(command) {
                Ok(s) => serde_json::from_str::<isocosm::simulation::Receipt>(&s)
                    .map(|r| {
                        format!(
                            "{}: {:?}. Matter {} → {}.",
                            r.chosen, r.outcome, r.matter_before, r.matter_after
                        )
                    })
                    .unwrap_or(s),
                Err(e) => e,
            };
        }
        self.sim.proposal = None;
    }
    pub fn sim_select(&mut self, id: u64) {
        if self
            .sim
            .session
            .as_ref()
            .is_none_or(|s| s.sim.state().population.get(id).is_none())
        {
            self.sim.notice = "That individual does not exist.".into();
            return;
        }
        self.sim.selected = id;
        self.sim.individual = TextInput::new(id.to_string());
        self.sim_command(Command::Inspect(id));
    }
    fn sim_act(&mut self, process: &str, target: Option<u64>) {
        self.sim_command(Command::Act {
            actor: self.sim.selected,
            target,
            process: process.into(),
            cause: None,
        });
    }
    fn sim_save(&mut self) {
        let result = (|| {
            let session = self.sim.session.as_ref().ok_or("Found a world first.")?;
            let path = self.export_directory.join(format!(
                "isocosm-{}-{}.json",
                session.sim.genesis().seed,
                session.sim.state().tick
            ));
            let json = serde_json::to_vec_pretty(&session.save()).map_err(|e| e.to_string())?;
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
            Ok::<_, String>(path)
        })();
        match result {
            Ok(path) => {
                self.sim.path = TextInput::new(path.to_string_lossy());
                self.sim.notice = format!("Saved {}", path.display());
            },
            Err(why) => self.sim.notice = why,
        }
    }
    fn sim_load(&mut self) {
        self.sim.compared = None;
        let result = (|| {
            let path = self.sim.path.text().trim();
            if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
                return Err("Save exceeds 64 MiB.".into());
            }
            let saved: Saved =
                serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            Session::load(saved, Execution::Grouped)
        })();
        match result {
            Ok(session) => {
                self.sim.selected = first_critter(&session);
                self.sim.individual = TextInput::new(self.sim.selected.to_string());
                self.sim.session = Some(session);
                self.sim.other = None;
                self.sim.proposal = None;
                self.sim.notice = "Saved history replayed and verified.".into();
            },
            Err(why) => self.sim.notice = why,
        }
        self.sim.playing = false;
    }
    fn sim_compare(&mut self) {
        let Some(session) = &self.sim.session else {
            return;
        };
        let comparison = Session::load(session.save(), Execution::Individuals);
        self.sim.compared = Some(comparison.is_ok());
        self.sim.notice = match comparison {
            Ok(_) => {
                "Individual replay agrees with this grouped world, including its accepted history."
                    .into()
            },
            Err(why) => format!("Comparison failed: {why}"),
        };
    }
    fn sim_fork(&mut self) {
        self.sim.compared = None;
        self.sim.branches = self.sim.branches.saturating_add(1);
        let Some(session) = &self.sim.session else {
            return;
        };
        match session.fork_at(
            session.sim.state().tick,
            format!(
                "branch:{}",
                isocosm::digest(&(
                    &session.branch,
                    session.sim.state().tick,
                    session.entries.len(),
                    self.sim.branches
                ))
            ),
        ) {
            Ok(fork) => {
                self.sim.other = self.sim.session.replace(fork);
                self.sim.proposal = None;
                self.sim.notice =
                    "Playing the new branch. Switch branch returns to the retained world.".into();
            },
            Err(why) => self.sim.notice = why,
        }
        self.sim.playing = false;
    }
    fn sim_merge(&mut self) {
        if let (Some(a), Some(b)) = (&self.sim.session, &self.sim.other) {
            match a.merge(b) {
                Ok(proposal) => {
                    self.sim.notice = format!(
                        "Merge proposal: {} changed outcomes. Review before applying.\n{}",
                        proposal.changes.len(),
                        serde_json::to_string_pretty(&proposal.changes).unwrap_or_default()
                    );
                    self.sim.proposal = Some(proposal);
                },
                Err(why) => self.sim.notice = why,
            }
        } else {
            self.sim.notice = "Make a branch first.".into();
        }
        self.sim.playing = false;
    }
}

fn first_critter(s: &Session) -> u64 {
    s.sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.method != Method::Inert)
        .map_or(0, |(id, _)| *id)
}
fn parse(input: &TextInput) -> Result<u64, String> {
    input
        .text()
        .trim()
        .parse()
        .map_err(|_| "Enter a whole nonnegative number.".into())
}
fn button(label: &'static str, action: fn(&mut Bench)) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |s: &mut Bench, _| action(s),
    )))
}
fn input(label: &'static str, get: fn(&mut Bench) -> &mut TextInput) -> Child {
    Box::new(el(
        "label",
        (
            el("span", text(label)),
            el(
                "div",
                lens(|input: &mut TextInput| text_field_typed(input), get),
            )
            .attr(
                "class",
                format!(
                    "generation-input sim-{}",
                    label.to_lowercase().replace(' ', "-")
                ),
            ),
        ),
    ))
}

pub(super) fn view(state: &Bench) -> Child {
    let mut children: Vec<Child> = vec![
        Box::new(el("h1", text("Specimen bench / Isocosm"))),
        Box::new(el(
            "p",
            text(
                "Found a world, run its processes, and inspect what changes. Grouping is exact for supported independent processes; interactions run individually.",
            ),
        )),
        Box::new(
            el(
                "div",
                vec![
                    input("Seed", |s| &mut s.sim.seed),
                    input("Population", |s| &mut s.sim.population),
                    input("Sites", |s| &mut s.sim.sites),
                    input("Lineages", |s| &mut s.sim.lineages),
                    button("Found world", Bench::sim_found),
                    button("Back to specimen", |s| {
                        s.sim.open = false;
                        s.sim.playing = false;
                    }),
                    button("Reservoir world", |s| {
                        s.sim.ecology = false;
                        s.sim_found();
                    }),
                    button("Ecology world", |s| {
                        s.sim.ecology = true;
                        s.sim_found();
                    }),
                ],
            )
            .attr("class", "toolbar"),
        ),
        Box::new(
            el(
                "div",
                vec![
                    input("Ticks per step", |s| &mut s.sim.step),
                    button("Step sim", Bench::sim_step),
                    button("Play / pause sim", |s| s.sim.playing = !s.sim.playing),
                    button("Compare individual replay", Bench::sim_compare),
                    button("Collect sim", |s| s.sim_command(Command::Collect)),
                    button("Remember individual", |s| s.sim_act("sim:remember", None)),
                    button("Reproduce", |s| s.sim_act("sim:birth", None)),
                    button("Release individual", |s| {
                        s.sim_command(Command::Release(s.sim.selected))
                    }),
                    input("Individual", |s| &mut s.sim.individual),
                    button("Inspect individual", |s| match parse(&s.sim.individual) {
                        Ok(id) => s.sim_select(id),
                        Err(e) => s.sim.notice = e,
                    }),
                    button("Reckon individual", |s| s.sim_act("sim:reckon", None)),
                ],
            )
            .attr("class", "toolbar"),
        ),
    ];
    if let Some(session) = &state.sim.session {
        let world = session.sim.state();
        children.push(Box::new(el("p",text(format!("Tick {} · {} entities in {} stored groups · matter {} · {} notes · {} events · {}",
            world.tick,world.population.count(),world.population.groups.len(),session.sim.matter(),world.notes.len(),world.events.len(),session.branch.chars().take(22).collect::<String>()))).attr("id","sim-status")));
        let sites: Vec<Child> = world
            .sites
            .iter()
            .map(|(&id, site)| {
                let groups: Vec<_> = world
                    .population
                    .groups
                    .iter()
                    .filter(|(_, g)| g.entity.place == id && g.entity.method != Method::Inert)
                    .collect();
                let count: u64 = groups.iter().map(|(_, g)| g.count).sum();
                let alive: u64 = groups
                    .iter()
                    .filter(|(_, g)| g.entity.alive)
                    .map(|(_, g)| g.count)
                    .sum();
                let selected = groups.first().map(|(id, _)| **id);
                let mut site_children: Vec<Child> = vec![
                    Box::new(el("h3", text(format!("Site {id}")))),
                    Box::new(el(
                        "p",
                        text(format!(
                            "{alive} alive, {} dead · routes {}",
                            count - alive,
                            site.routes
                                .iter()
                                .map(|r| r.to.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    )),
                    Box::new(el(
                        "p",
                        text(
                            site.conditions
                                .iter()
                                .map(|(k, v)| {
                                    format!("{} {v}", k.strip_prefix("world:").unwrap_or(k))
                                })
                                .collect::<Vec<_>>()
                                .join(" · "),
                        ),
                    )),
                    Box::new(el(
                        "p",
                        text(format!(
                            "Soil {}",
                            site.accounts.get("world:soil").copied().unwrap_or(0)
                        )),
                    )),
                ];
                if let Some(id) = selected {
                    site_children.push(Box::new(focusable(clickable(
                        el("button", text("Inspect inhabitant")),
                        move |s: &mut Bench, _| s.sim_select(id),
                    ))));
                }
                Box::new(el("section", site_children).attr("class", "sim-site")) as Child
            })
            .collect();
        let detail = world
            .population
            .get(state.sim.selected)
            .map(|e| {
                format!(
                    "{} · {}\nSite {} · born {} · alive {}\nTraits: {}\n\n{}",
                    e.lineage,
                    e.kingdom,
                    e.place,
                    e.born,
                    e.alive,
                    e.traits.iter().cloned().collect::<Vec<_>>().join(", "),
                    e.accounts
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .unwrap_or_default();
        children.push(Box::new(
            el(
                "div",
                (
                    el("div", sites).attr("class", "sim-sites"),
                    el(
                        "section",
                        (
                            el("h2", text(format!("Individual {}", state.sim.selected))),
                            el("pre", text(detail)),
                        ),
                    )
                    .attr("class", "sim-detail"),
                ),
            )
            .attr("class", "sim-columns"),
        ));
    }
    children.push(Box::new(
        el(
            "div",
            vec![
                button("Branch world", Bench::sim_fork),
                button("Switch branch", |s| {
                    if s.sim.other.is_some() {
                        std::mem::swap(&mut s.sim.session, &mut s.sim.other);
                    }
                    s.sim.proposal = None;
                    s.sim.playing = false;
                }),
                button("Preview merge", Bench::sim_merge),
                button("Apply reviewed merge", |s| {
                    if let Some(proposal) = s.sim.proposal.take() {
                        s.sim.session = Some(proposal.proposed);
                        s.sim.notice = "Merged world accepted.".into();
                    }
                }),
                button("Save sim", Bench::sim_save),
                input("Saved world path", |s| &mut s.sim.path),
                button("Load sim", Bench::sim_load),
            ],
        )
        .attr("class", "toolbar"),
    ));
    children.push(Box::new(
        el("pre", text(state.sim.notice.clone())).attr("id", "sim-notice"),
    ));
    Box::new(el("div", children).attr("class", "bench sim"))
}
