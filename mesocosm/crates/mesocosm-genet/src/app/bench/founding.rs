// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The native founding bench edits a Cosmograph recipe. Only an accepted
//! founding becomes a projection. The selected site is a read-only lens.
use super::{Bench, Child};
use cambium::TextInput;
use cosmograph::{Execution, Founding, Session, bodied::BodyMap, preset::Preset, rules::Rule};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) mod diagram;
pub(super) mod fields;
#[cfg(test)]
mod tests;
mod view;
pub(super) use view::view;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    World,
    Body,
    Watch,
}

pub(super) struct Panel {
    draft: Founding,
    seed: TextInput,
    population: TextInput,
    cohort: TextInput,
    stock_min: TextInput,
    stock_max: TextInput,
    path: TextInput,
    page: Page,
    lineage: String,
    run: usize,
    bank_page: usize,
    site: u64,
    session: Session,
    flows: Vec<cosmograph::flows::Flow>,
    // Actual movements of matter during the displayed tick, by site and process.
    specimen_id: Option<u64>,
    part_id: Option<usize>,
    transfers: BTreeMap<u64, BTreeMap<String, u128>>,
    notice: String,
    next_tick: Option<std::time::Instant>,
}

impl Panel {
    fn starter(seed: u64) -> Founding {
        let mut draft: Founding = toml::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../shared/cosmograph/examples/optional-gland.toml"
        )))
        .expect("shipped starting lineage");
        draft.seed = seed;
        draft.sites = 4;
        draft.map = Some(cosmograph::map::Layout::Grid(cosmograph::map::Grid {
            shape: "shape:plane".into(),
            width: 2,
            height: 2,
            side: 128,
            elevation: [0, 24],
            relief: [0, 6],
            sea_per_mille: 0,
        }));
        draft.played = Some(cosmograph::directing::Played {
            lineage: 1,
            region_sites: 1,
        });
        draft.repertoire = Some(Default::default());
        draft
    }

    fn new(draft: Founding) -> Result<Self, String> {
        let session = Session::new(draft.generate()?, Execution::Grouped)?;
        Self::from_session(draft, session)
    }

    fn from_session(draft: Founding, session: Session) -> Result<Self, String> {
        let lineage = draft
            .body_maps
            .keys()
            .next()
            .cloned()
            .or_else(|| {
                session
                    .sim
                    .genesis()
                    .lineages
                    .keys()
                    .find(|k| k.starts_with("lineage:"))
                    .cloned()
            })
            .ok_or("no founding lineage")?;
        let site = *session
            .sim
            .state()
            .sites
            .keys()
            .next()
            .ok_or("no founding sites")?;
        Ok(Self {
            seed: field(draft.seed),
            population: field(draft.population),
            cohort: field(draft.cohort_size),
            stock_min: field(draft.stock_min),
            stock_max: field(draft.stock_max),
            path: TextInput::new(""),
            draft,
            page: Page::World,
            lineage,
            run: 0,
            bank_page: 0,
            site,
            session,
            specimen_id: None,
            part_id: None,
            flows: Vec::new(),
            transfers: BTreeMap::new(),
            notice: String::new(),
            next_tick: None,
        })
    }

    fn recipe(&self) -> Result<Founding, String> {
        let mut draft = self.draft.clone();
        let number = |name, value: &TextInput| {
            value
                .text()
                .trim()
                .parse::<u64>()
                .map_err(|_| format!("{name} must be a whole nonnegative number."))
        };
        draft.seed = number("Seed", &self.seed)?;
        draft.population = number("Population", &self.population)?;
        draft.cohort_size = number("Cohort size", &self.cohort)?;
        draft.stock_min = number("Minimum stock", &self.stock_min)?;
        draft.stock_max = number("Maximum stock", &self.stock_max)?;
        Ok(draft)
    }

    fn project(&mut self) -> Result<(), String> {
        let draft = self.recipe()?;
        let session = Session::new(draft.generate()?, Execution::Grouped)?;
        if !session.sim.state().sites.contains_key(&self.site) {
            self.site = *session
                .sim
                .state()
                .sites
                .keys()
                .next()
                .ok_or("no founding sites")?;
        }
        self.draft = draft;
        self.session = session;
        self.transfers.clear();
        self.flows.clear();
        self.specimen_id = None;
        self.part_id = None;
        self.page = Page::Watch;
        self.next_tick = None;
        Ok(())
    }

    pub(super) fn playing(&self) -> bool {
        self.page == Page::Watch && self.next_tick.is_some()
    }

    pub(super) fn advance_clock(&mut self) {
        let now = std::time::Instant::now();
        if self.playing() && self.next_tick.is_some_and(|t| t <= now) {
            self.next_tick = Some(now + std::time::Duration::from_millis(350));
            if let Err(e) = self.seek(self.session.sim.state().tick.saturating_add(1)) {
                self.notice = e;
                self.next_tick = None;
            }
        }
    }

    pub(super) fn reading(&self) -> (&Session, u64) {
        (&self.session, self.site)
    }

    fn seek(&mut self, tick: u64) -> Result<(), String> {
        let current = self.session.sim.state().tick;
        if current == tick {
            return Ok(());
        }
        let before = tick.saturating_sub(1);
        if current > before {
            self.session = self.session.prefix(0, before)?;
        } else {
            self.session.advance(before - current)?;
        }
        self.transfers.clear();
        self.flows.clear();
        if tick == 0 {
            return Ok(());
        }
        let observed = self.session.advance_tick_with_flows()?;
        let state = self.session.sim.state();
        self.flows = observed.flows;
        for flow in &self.flows {
            use cosmograph::flows::{Holder, MadeBy};
            let MadeBy::Process(process) = &flow.made_by else {
                continue;
            };
            let site = |holder: Holder| match holder {
                Holder::Site(id) => Some(id),
                other => other
                    .body()
                    .and_then(|id| state.population.get(id))?
                    .resident_site(),
            };
            let from = site(flow.from.0);
            let to = site(flow.to.0);
            for id in from
                .into_iter()
                .chain(to.filter(|dest| Some(*dest) != from))
            {
                *self
                    .transfers
                    .entry(id)
                    .or_default()
                    .entry(process.clone())
                    .or_default() += u128::from(flow.count) * u128::from(flow.amount);
            }
        }
        Ok(())
    }

    fn add_map(&mut self) {
        self.draft
            .body_maps
            .entry(self.lineage.clone())
            .or_insert_with(BodyMap::single_run);
        self.run = 0;
    }

    fn map(&self) -> Option<&BodyMap> {
        self.draft.body_maps.get(&self.lineage)
    }

    fn map_mut(&mut self) -> &mut BodyMap {
        self.draft
            .body_maps
            .get_mut(&self.lineage)
            .expect("selected mapped lineage")
    }

    fn bind(&mut self, tissue: String) {
        let idx = self.run;
        let map = self.map_mut();
        let run = &mut map.grammar.seed[idx];
        if let Some(region) = run.bears.as_ref().filter(|key| key.starts_with("region:")) {
            map.regions.insert(region.clone(), tissue);
        } else {
            run.bears = Some(tissue);
        }
        run.per_segment = run.per_segment.max(1);
    }

    fn repeat(&mut self, more: bool) {
        let tagma = self.run as u8;
        let rules = &mut self.map_mut().grammar.rules;
        let pos = rules
            .iter()
            .position(|rule| matches!(rule, Rule::Repeat { tagma: r, .. } if *r == tagma));
        if let Some(i) = pos {
            if let Rule::Repeat { by, .. } = &mut rules[i] {
                if more {
                    *by = by.saturating_add(1);
                } else if *by > 1 {
                    *by -= 1;
                } else {
                    rules.remove(i);
                }
            }
        } else if more {
            rules.push(Rule::Repeat { tagma, by: 1 });
        }
    }

    fn change_outline(&mut self) {
        let Some(cosmograph::map::Layout::Grid(grid)) = &mut self.draft.map else {
            return;
        };
        if grid.shape == "shape:sphere" {
            grid.shape = "shape:plane".into();
        } else {
            let side = (grid.width / 3).clamp(2, 6);
            grid.width = 3 * side;
            grid.height = 2 * side;
            grid.shape = "shape:sphere".into();
        }
        self.draft.sites = grid.width * grid.height;
    }

    fn next_preset(&mut self) {
        self.draft.preset = match self.draft.preset {
            None => Some(Preset::TidalShelf),
            Some(Preset::TidalShelf) => Some(Preset::HeavyDeep),
            Some(Preset::HeavyDeep) => Some(Preset::LongYear),
            Some(Preset::LongYear) => None,
        };
    }

    fn load(&mut self) -> Result<(), String> {
        let path = self.path.text().trim();
        let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let mut loaded = Self::new(toml::from_str(&source).map_err(|e| format!("{e}"))?)?;
        loaded.path = self.path.clone();
        *self = loaded;
        Ok(())
    }

    fn load_map(&mut self) -> Result<(), String> {
        let source = std::fs::read_to_string(self.path.text().trim()).map_err(|e| e.to_string())?;
        let map: BodyMap = source.parse().map_err(|e| format!("{e}"))?;
        let mut draft = self.recipe()?;
        draft.body_maps.insert(self.lineage.clone(), map);
        let validated = Self::new(draft)?;
        self.draft = validated.draft;
        self.session = validated.session;
        self.transfers.clear();
        self.flows.clear();
        self.run = 0;
        self.bank_page = 0;
        self.page = Page::Body;
        Ok(())
    }

    fn reopen(&mut self) -> Result<(), String> {
        let bytes = std::fs::read(self.path.text().trim()).map_err(|e| e.to_string())?;
        let session = Session::load_json(&bytes, Execution::Grouped)?;
        if !session.entries.is_empty() {
            return Err(
                "This save has player commands. Resume it in Mesocosm with --watch.".into(),
            );
        }
        let recipe = session
            .sim
            .genesis()
            .founding
            .clone()
            .ok_or("Save has no founding template")?;
        let tick = session.sim.state().tick;
        let hash = session.sim.state_hash();
        let mut loaded = Self::from_session(recipe, session)?;
        if tick > 0 {
            loaded.seek(tick - 1)?;
            loaded.seek(tick)?;
        }
        if loaded.session.sim.state_hash() != hash {
            return Err("Reopened projection differs from its saved state".into());
        }
        loaded.path = self.path.clone();
        loaded.page = Page::Watch;
        *self = loaded;
        Ok(())
    }

    /// Give each visit its own save directory, including repeated entries at
    /// the same seed and tick. Keep the template and exact unplayed checkpoint.
    fn save(&self, base: &Path) -> Result<PathBuf, String> {
        let g = self.session.sim.genesis();
        let root = g.founding.as_ref().ok_or("world has no founding recipe")?;
        let source = toml::to_string_pretty(root).map_err(|e| e.to_string())?;
        let tick = self.session.sim.state().tick;
        std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
        for n in 1u64.. {
            let dir = base.join(format!("seed-{}-tick-{tick}-{n}", root.seed));
            match std::fs::create_dir(&dir) {
                Ok(()) => {
                    let result = (|| {
                        std::fs::write(dir.join("founding.toml"), &source)
                            .map_err(|e| e.to_string())?;
                        for (key, map) in &root.body_maps {
                            if let Some(id) = key
                                .strip_prefix("lineage:")
                                .and_then(|s| s.parse::<u32>().ok())
                            {
                                let data =
                                    toml::to_string_pretty(map).map_err(|e| e.to_string())?;
                                std::fs::write(dir.join(format!("lineage-{id}.toml")), data)
                                    .map_err(|e| e.to_string())?;
                            }
                        }
                        crate::played::write_json(&dir.join("entry.json"), &self.session.save())
                    })();
                    if let Err(why) = result {
                        return Err(format!("Could not finish saving {}: {why}", dir.display()));
                    }
                    return Ok(dir);
                },
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        unreachable!()
    }
}

fn field(v: u64) -> TextInput {
    TextInput::new(v.to_string())
}

impl Bench {
    pub(super) fn open_founding(&mut self) {
        self.sim.playing = false;
        let seed = self.sim.seed.text().parse().unwrap_or(1);
        match Panel::new(Panel::starter(seed)) {
            Ok(panel) => self.founding = Some(panel),
            Err(why) => self.sim.notice = why,
        }
    }

    fn store_projection(&mut self, enter: bool) -> Result<(), String> {
        let folder = self
            .world_directory
            .as_ref()
            .ok_or("No home directory for saved worlds")?;
        let panel = self
            .founding
            .as_mut()
            .ok_or("Open the founding editor first")?;
        let dir = panel.save(folder)?;
        panel.notice = format!("Saved {}", dir.display());
        if enter {
            std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                .arg("--enter-world")
                .arg(dir.join("entry.json"))
                .arg("--watch-site")
                .arg(panel.site.to_string())
                .arg("--trace")
                .arg(dir.join("world.json"))
                .arg("--capture")
                .arg(dir.join("scene.png"))
                .arg("--receipt")
                .arg(dir.join("receipt.json"))
                .spawn()
                .map_err(|e| format!("Saved {}. Could not enter: {e}", dir.display()))?;
        }
        Ok(())
    }
}
