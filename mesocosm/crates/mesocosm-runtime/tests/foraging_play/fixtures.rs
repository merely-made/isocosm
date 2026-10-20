// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::{
    rules::{Binding, Query},
    spatial::embodiment as body,
};

pub fn ordinary(seed: u64) -> Founded {
    let mut founded = Founded::generated(seed, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = isocosm::ecology::Rates::A1;
    founded.genesis = domain.generate().unwrap();
    founded.genesis.rules.epoch_ticks = 60;
    let sun = founded.genesis.rules.sun.clone().unwrap_or_default();
    founded.genesis.rules.sun = Some(isocosm::light::SunRules { start_hour: 7, ..sun });
    founded
}

pub struct Fixture {
    pub session: Session,
    pub game: Game,
    pub actor: u64,
    pub food: u64,
    pub feed: String,
    pub walk: String,
}

pub fn control(hungry: bool, nodes: usize, edible: bool) -> Fixture {
    build(hungry, nodes, edible, 4, false)
}

pub fn at_food() -> Fixture {
    build(true, 256, true, 0, true)
}

fn build(hungry: bool, nodes: usize, edible: bool, gap: u64, migration: bool) -> Fixture {
    let mut founded = ordinary(16_003_851_315_348_448_882);
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.bodies.as_mut().unwrap().hazard = [0; 2];
    domain.bodies.as_mut().unwrap().wound = [0; 2];
    founded.genesis = domain.generate().unwrap();
    let g = &mut founded.genesis;
    for group in g
        .population
        .groups
        .values_mut()
        .filter(|g| g.entity.body.is_some())
    {
        group.count = 1;
    }
    for site in g.sites.values_mut() {
        for (key, value) in [
            ("terrain:elevation", 64),
            ("terrain:relief", 0),
            ("terrain:water", 0),
        ] {
            site.conditions.insert(key.into(), value);
        }
    }
    body::declare(g).unwrap();
    // Isolate contact within a site, retaining feeding, upkeep, healing and
    // reproduction choices. The ordinary acceptance run keeps migration too.
    for process in g
        .rules
        .processes
        .values_mut()
        .filter(|p| !migration && p.id.starts_with("move:"))
    {
        process.period = None;
    }
    let t = g.rules.spatial_templates.as_ref().unwrap();
    let (&site, stands) = t
        .stands
        .iter()
        .find(|(_, stands)| stands.len() >= 2)
        .unwrap();
    let actor = stands
        .iter()
        .filter_map(|s| s.actor)
        .find(|id| {
            g.population
                .get(*id)
                .unwrap()
                .traits
                .contains("life:consumer")
        })
        .unwrap();
    let food = stands
        .iter()
        .filter_map(|s| s.actor)
        .find(|id| {
            g.population
                .get(*id)
                .unwrap()
                .traits
                .contains("life:producer")
        })
        .unwrap();
    g.population.groups.get_mut(&food).unwrap().count = 2;
    let at = stands.iter().find(|s| s.actor == Some(actor)).unwrap();
    let before = native::Pose {
        at: at.at,
        yaw: at.yaw,
    };
    let actor_boxes = body::boxes(
        g.population.get(actor).unwrap().body.as_ref().unwrap(),
        &before,
    )
    .unwrap();
    let (walk, f) = t.foraging.iter().next().unwrap();
    let (walk, feed) = (walk.clone(), f.feed.clone());
    // Author only the fixture's appetite level. Choice, competing processes,
    // native food intake, population cohorts and the spatial clock stay active.
    let wants = isocosm::needs::wants(&g.rules, g.population.get(actor).unwrap());
    let want = wants.first().expect("native account hunger");
    let account = want.account.clone();
    if hungry {
        let amount = want.held.min(3);
        isocosm::anatomy::take(std::sync::Arc::make_mut(&mut g.population.groups.get_mut(&actor).unwrap().entity),
            &g.rules, &account, amount).unwrap().unwrap();
        *g.sites.get_mut(&site).unwrap().accounts.entry("world:soil".into()).or_default() += amount;
    }
    let held = isocosm::anatomy::held(g.population.get(actor).unwrap(), &g.rules, &account);
    for n in &mut g.rules.mind.as_mut().unwrap().needs {
        if isocosm::needs::account(&n.query) == Some(&account) {
            n.query = Query::Below { who: Binding::Actor, key: account.clone(),
                amount: if hungry { held + 1 } else { 0 } };
        }
    }
    let t = g.rules.spatial_templates.as_mut().unwrap();
    t.foraging.get_mut(&walk).unwrap().nodes = nodes;
    let window = t.window.clone();
    // Place real food a few cells beyond contact on flat native terrain.
    let view = isocosm::terrain::View::of(g).unwrap();
    let volume = isometer_space::volume::Volume::lift(
        &view,
        site,
        window.min,
        window.max,
        window.depth,
        &[],
    )
    .unwrap();
    let document = g.population.get(food).unwrap().body.as_ref().unwrap();
    let mut grounding = body::Grounding::new(&volume);
    let actor_right = actor_boxes.iter().map(|b| b[1][0]).max().unwrap();
    let pose = (0i64..96)
        .find_map(|offset| {
            let p = grounding
                .at(document, before.at[0] + offset, before.at[2], before.yaw)
                .ok()?;
            let boxes = body::boxes(document, &p).ok()?;
            let d = native::foraging::gap(&actor_boxes, &boxes);
            let left = boxes.iter().map(|b| b[0][0]).min().unwrap();
            (!body::intersects(&actor_boxes, &boxes)
                && d == gap
                && left >= actor_right + gap as i64)
                .then_some(p)
        })
        .expect("fixture admits a short food route");
    let t = g.rules.spatial_templates.as_mut().unwrap();
    let stand = t
        .stands
        .get_mut(&site)
        .unwrap()
        .iter_mut()
        .find(|s| s.actor == Some(food))
        .unwrap();
    stand.at = pose.at;
    stand.yaw = pose.yaw;
    let replacement = stand.clone();
    for crossing in &mut t.crossings {
        if crossing.stand.actor == Some(food) {
            crossing.stand = replacement.clone();
        }
    }
    if !edible {
        g.rules
            .processes
            .get_mut(&feed)
            .unwrap()
            .port_feeding
            .as_mut()
            .unwrap()
            .sources = isocosm::rules::FeedingSources::Only(Default::default());
    }
    let mut session = Session::new(g.clone(), Execution::Grouped).unwrap();
    let mut game = Game::default();
    game.found_generated(&mut session).unwrap();
    Fixture {
        session,
        game,
        actor,
        food,
        feed,
        walk,
    }
}

impl Fixture {
    pub fn distance(&self) -> u64 {
        let a = self.session.sim.state().population.get(self.actor).unwrap();
        let b = self.session.sim.state().population.get(self.food).unwrap();
        if a.resident_site().is_none() || a.resident_site() != b.resident_site() {
            return u64::MAX;
        }
        let boxes = |id| {
            let doc = self
                .session
                .sim
                .state()
                .population
                .get(id)
                .unwrap()
                .body
                .as_ref()
                .unwrap();
            body::boxes(doc, &self.session.sim.state().spatial_poses[&id].pose).unwrap()
        };
        native::foraging::gap(&boxes(self.actor), &boxes(self.food))
    }

    pub fn steps(&self) -> usize {
        self.game
            .driver
            .as_ref()
            .unwrap()
            .records
            .iter()
            .flat_map(|r| &r.changes)
            .filter(|c| c.actor == self.actor && matches!(c.admission, Admission::Step { .. }))
            .count()
    }

    pub fn tick(&mut self) {
        let _ = self.game.advance_clock(&mut self.session).unwrap();
    }
}
