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

mod comparison;
pub(super) mod diagram;
pub(super) mod fields;
#[cfg(test)]
mod catalogue_tests;
#[cfg(test)]
mod tests;
mod view;
pub(super) use view::view;

enum Version {
    Original,
    Edited,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    World,
    Body,
    Watch,
}

pub(super) struct Panel {
    draft: Founding,
    generated_maps: BTreeMap<String, BodyMap>,
    habitats: BTreeMap<String, cosmograph::generate::WorldBody>,
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
    lineage_page: usize,
    world_body_slot: usize,
    show_atlas: bool,
    kingdom_min: TextInput,
    kingdom_max: TextInput,
    richness_min: TextInput,
    richness_max: TextInput,
    site: u64,
    session: Session,
    original: Option<Session>,
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
        let mut draft = Founding {
            seed,
            web: cosmograph::ecology::Web::Diets,
            population: 90,
            cohort_size: 2,
            stock_max: 16,
            ecology: true,
            ecology_rates: cosmograph::ecology::Rates::A1,
            bodies: Some(cosmograph::bodied::Bodies {
                canopy_growth: true,
                fixing_budget: 16,
                ..Default::default()
            }),
            inherit: true,
            guard: Some(Default::default()),
            categories: Some(Default::default()),
            nutrients: Some(Default::default()),
            mutualism: Some(Default::default()),
            detritus: Some(Default::default()),
            repertoire: Some(cosmograph::repertoire::Repertoire {
                founders: true,
                kingdom_draw: Some(Default::default()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let grid=Self::world_grid(seed);
        draft.sites=grid.width*grid.height;
        draft.map=Some(cosmograph::map::Layout::Grid(grid));
        draft.lineages=draft.repertoire.as_ref().unwrap().kingdom_draw.as_ref().unwrap()
            .draw(seed).expect("valid default draw").iter().map(|(_,n)|n).sum();
        draft.place_memory = Some(Default::default());
        draft.played = Some(cosmograph::directing::Played {
            lineage: 1,
            region_sites: 1,
        });
        draft
    }

    fn world_grid(seed: u64) -> cosmograph::map::Grid {
        let mut grid=cosmograph::map::Grid::drawn(seed);
        if cosmograph::draw(seed,"world-outline",&[]) % 4 == 0 {
            let face=1+(cosmograph::draw(seed,"world-face",&[])%6) as u32;
            grid.width=3*face;
            grid.height=2*face;
            grid.shape="shape:sphere".into();
        }
        grid
    }

    fn new(draft: Founding) -> Result<Self, String> {
        let session = Session::new(draft.generate()?, Execution::Grouped)?;
        Self::from_session(draft, session)
    }

    fn from_session(draft: Founding, session: Session) -> Result<Self, String> {
        let generated_maps = draft.catalogue_body_maps()?;
        let lineage = draft
            .body_maps
            .keys()
            .next()
            .cloned()
            .or_else(|| generated_maps.keys().next().cloned())
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
        let distribution=draft.repertoire.as_ref().and_then(|r|r.kingdom_draw.as_ref()).cloned().unwrap_or_default();
        Ok(Self {
            lineage_page: 0,
            world_body_slot: 0,
            show_atlas: false,
            kingdom_min: field(distribution.kingdoms[0].into()),
            kingdom_max: field(distribution.kingdoms[1].into()),
            richness_min: field(distribution.per_kingdom[0].into()),
            richness_max: field(distribution.per_kingdom[1].into()),
            habitats: draft.world_body_recipes()?,
            generated_maps,
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
            original: None,
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
        if let Some(dist)=draft.repertoire.as_mut().and_then(|r|r.kingdom_draw.as_mut()) {
            let bounds=|name,field| u32::try_from(number(name,field)?).map_err(|_|format!("{name} exceeds the founding range"));
            dist.kingdoms=[bounds("Minimum kingdoms",&self.kingdom_min)?,bounds("Maximum kingdoms",&self.kingdom_max)?];
            dist.per_kingdom=[bounds("Minimum lineages",&self.richness_min)?,bounds("Maximum lineages",&self.richness_max)?];
            draft.lineages=dist.draw(draft.seed)?.iter().map(|(_,n)|n).sum();
            if draft.seed!=self.draft.seed && draft.map==Some(cosmograph::map::Layout::Grid(Self::world_grid(self.draft.seed))) {
                let grid=Self::world_grid(draft.seed);
                draft.sites=grid.width*grid.height;
                draft.map=Some(cosmograph::map::Layout::Grid(grid));
            }
        }
        if let Some(previous)=self.session.sim.genesis().founding.as_ref() {
            draft.carry_body_edits(previous)?;
        }
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
        self.habitats = draft.world_body_recipes()?;
        self.generated_maps = draft.catalogue_body_maps()?;
        self.draft = draft;
        if !session.sim.genesis().lineages.contains_key(&self.lineage) { self.lineage="lineage:0".into(); }
        self.lineage_page=0;
        self.run = self.map().map_or(0, |m| self.run.min(m.grammar.seed.len().saturating_sub(1)));
        self.session = session;
        self.original = None;
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
        let mut tick = tick.max(self.session.sim.genesis().started);
        if let Some(original) = &self.original {
            tick = tick.max(original.sim.genesis().started);
        }
        let current = self.session.sim.state().tick;
        if current == tick {
            return Ok(());
        }
        self.flows = if let Some(original) = &self.original {
            // Publish the pair only when both simulations reached the requested tick.
            let mut original = original.clone();
            let mut edited = self.session.clone();
            if tick < current {
                original = original.prefix(0, tick)?;
            } else {
                original.advance(tick - current)?;
            }
            let flows = Self::flows_at(&mut edited, tick)?;
            self.original = Some(original);
            self.session = edited;
            flows
        } else {
            Self::flows_at(&mut self.session, tick)?
        };
        self.transfers.clear();
        let state = self.session.sim.state();
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

    fn flows_at(session: &mut Session, tick: u64) -> Result<Vec<cosmograph::flows::Flow>, String> {
        let current = session.sim.state().tick;
        let start = session.sim.genesis().started;
        let before = tick.saturating_sub(1).max(start);
        if current > before {
            *session = session.prefix(0, before)?;
        } else {
            session.advance(before - current)?;
        }
        if tick == start {
            return Ok(Vec::new());
        }
        Ok(session.advance_tick_with_flows()?.flows)
    }

    fn add_map(&mut self) {
        self.draft
            .body_maps
            .entry(self.lineage.clone())
            .or_insert_with(BodyMap::single_run);
        self.run = 0;
    }

    fn map(&self) -> Option<&BodyMap> {
        self.draft
            .body_maps
            .get(&self.lineage)
            .or_else(|| self.generated_maps.get(&self.lineage))
    }

    fn map_mut(&mut self) -> &mut BodyMap {
        self.draft
            .body_maps
            .entry(self.lineage.clone())
            .or_insert_with(|| self.generated_maps[&self.lineage].clone())
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
            grid.width=grid.width.clamp(2,16); grid.height=grid.height.clamp(2,16);
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
        self.draft = Self::new(draft)?.draft;
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
    fn save(session: &Session, base: &Path) -> Result<PathBuf, String> {
        let g = session.sim.genesis();
        let root = g.founding.as_ref().ok_or("world has no founding recipe")?;
        let source = toml::to_string_pretty(root).map_err(|e| e.to_string())?;
        let tick = session.sim.state().tick;
        std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
        for n in 1u64.. {
            let dir = base.join(format!("seed-{}-tick-{tick}-{n}", root.seed));
            match std::fs::create_dir(&dir) {
                Ok(()) => {
                    let result = (|| {
                        std::fs::write(dir.join("founding.toml"), &source)
                            .map_err(|e| e.to_string())?;
                        let mut maps = root.catalogue_body_maps()?;
                        maps.extend(root.body_maps.clone());
                        for (key, map) in &maps {
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
                        crate::played::write_json(&dir.join("entry.json"), &session.save())
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

/// Abilities supported by this specimen's living tissue now.
fn organ_reading(port: cosmograph::process::IntakePort) -> Vec<String> {
    use cosmograph::process::{NisKind, Trade};
    let mut words = Vec::new();
    for (kind, food) in [
        (NisKind::Producer, "plant tissue"),
        (NisKind::Consumer, "animal tissue"),
        (NisKind::Decomposer, "decomposer tissue"),
    ] {
        if port.admits_live(kind) {
            words.push(format!("Can eat {food}"));
        }
    }
    for (enabled, label) in [
        (port.admits_deadstock(), "Can eat dead tissue"),
        (
            port.trades(Trade::Draws),
            "Draws the scarcest soil nutrient",
        ),
        (port.trades(Trade::Gives), "Can supply a trading partner"),
        (
            port.trades(Trade::Takes),
            "Can receive from a trading partner",
        ),
    ] {
        if enabled {
            words.push(label.into());
        }
    }
    let strength = port.strength();
    for (n, label) in [
        (strength.toxin, "Toxin"),
        (strength.spine, "Contact harm"),
        (strength.pierce, "Piercing strength"),
        (strength.crush, "Crushing strength"),
    ] {
        if n > 0 {
            words.push(format!("{label} {n}"));
        }
    }
    words
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

    fn store_projection(&mut self, version: Version, enter: bool) -> Result<(), String> {
        let folder = self
            .world_directory
            .as_ref()
            .ok_or("No home directory for saved worlds")?;
        let panel = self
            .founding
            .as_mut()
            .ok_or("Open the founding editor first")?;
        let session = match version {
            Version::Original => panel.original.as_ref().ok_or("No original projection")?,
            Version::Edited => &panel.session,
        };
        if enter && !session.sim.state().sites.contains_key(&panel.site) {
            return Err(format!("Site {} does not exist in this world.", panel.site));
        }
        let dir = Panel::save(session, folder)?;
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
