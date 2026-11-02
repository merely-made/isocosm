// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{
    rules::{Binding, Query},
    spatial::embodiment as body,
};

pub fn ordinary(seed: u64) -> Founded {
    let mut founded = Founded::generated(seed, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = cosmograph::ecology::Rates::A1;
    founded.genesis = domain.generate().unwrap();
    // Every lineage decides each tick: these tests ask how a forager walks,
    // not how often it decides (D19 paces played worlds).
    for l in founded.genesis.lineages.values_mut() {
        l.decides = None;
    }
    founded.genesis.rules.epoch_ticks = 60;
    let sun = founded.genesis.rules.sun.clone().unwrap_or_default();
    founded.genesis.rules.sun = Some(cosmograph::light::SunRules { start_hour: 7, ..sun });
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
    build(hungry, nodes, edible, 4, false, false)
}

/// The hungry control in a world whose foragers travel out of view (1084).
pub fn travelling() -> Fixture {
    build(true, 256, true, 4, false, true)
}

pub fn at_food() -> Fixture {
    build(true, 256, true, 0, true, false)
}

fn build(hungry: bool, nodes: usize, edible: bool, gap: u64, migration: bool, travel: bool) -> Fixture {
    let mut founded = ordinary(16_003_851_315_348_448_882);
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.bodies.as_mut().unwrap().hazard = [0; 2];
    domain.bodies.as_mut().unwrap().wound = [0; 2];
    founded.genesis = domain.generate().unwrap();
    // Every lineage decides each tick: these tests ask how a forager walks,
    // not how often it decides (D19 paces played worlds).
    for l in founded.genesis.lineages.values_mut() {
        l.decides = None;
    }
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
    // These controls walk by the game's steps, as a round's bodies do.
    if !travel {
        g.rules.speed = None;
    }
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
    // The first consumer, by site and identity, with a producer that can
    // stand `gap` cells beyond its right side on the flat ground.
    let (site, actor, food, pose) = {
        let t = g.rules.spatial_templates.as_ref().unwrap();
        let has = |id: &u64, key: &str| g.population.get(*id).unwrap().traits.contains(key);
        let mut found = None;
        'sites: for (&site, stands) in &t.stands {
            let ids: Vec<u64> = stands.iter().filter_map(|s| s.actor).collect();
            for &actor in ids.iter().filter(|id| has(id, "life:consumer")) {
                for &food in ids.iter().filter(|id| has(id, "life:producer")) {
                    if let Some(pose) = short_food(g, site, actor, food, gap) {
                        found = Some((site, actor, food, pose));
                        break 'sites;
                    }
                }
            }
        }
        found.expect("fixture admits a short food route")
    };
    // This authored navigation control needs an actual sense. The random
    // recipe used here can omit eyes entirely. Allocate one existing cell;
    // function is independent of shape and the physical route stays the same.
    let e = std::sync::Arc::make_mut(&mut g.population.groups.get_mut(&actor).unwrap().entity);
    if !e.living().any(|(_, p)| p.cells.get("function:sense").is_some_and(|n| *n > 0)) {
        let id = e.living().find(|(_, p)| p.cells.values().any(|n| *n > 1)).unwrap().0;
        let extent = e.extent(id);
        let p = e.part_mut(&id).unwrap();
        let source = p.cells.iter().find(|(_, n)| **n > 1).unwrap().0.clone();
        *p.cells.get_mut(&source).unwrap() -= 1;
        p.cells.insert("function:sense".into(), 1);
        p.functions.insert("function:sense".into());
        cosmograph::mosaic::sync(p, extent);
        e.systems = cosmograph::systems::founded(e, &g.rules);
    }
    g.population.groups.get_mut(&food).unwrap().count = 2;
    let t = g.rules.spatial_templates.as_ref().unwrap();
    let (walk, f) = t.foraging.iter().next().unwrap();
    let (walk, feed) = (walk.clone(), f.feed.clone());
    // Author only the fixture's appetite level. Choice, competing processes,
    // native food intake, population cohorts and the spatial clock stay active.
    let wants = cosmograph::needs::wants(&g.rules, g.population.get(actor).unwrap());
    let want = wants.first().expect("native account hunger");
    let account = want.account.clone();
    if hungry {
        let amount = want.held.min(3);
        cosmograph::anatomy::take(std::sync::Arc::make_mut(&mut g.population.groups.get_mut(&actor).unwrap().entity),
            &g.rules, &account, amount).unwrap().unwrap();
        *g.sites.get_mut(&site).unwrap().accounts.entry("world:soil".into()).or_default() += amount;
    }
    let held = cosmograph::anatomy::held(g.population.get(actor).unwrap(), &g.rules, &account);
    for n in &mut g.rules.mind.as_mut().unwrap().needs {
        if cosmograph::needs::account(&n.query) == Some(&account) {
            n.query = Query::Below { who: Binding::Actor, key: account.clone(),
                amount: if hungry { held + 1 } else { 0 } };
        }
    }
    let t = g.rules.spatial_templates.as_mut().unwrap();
    t.foraging.get_mut(&walk).unwrap().nodes = nodes;
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
            .sources = cosmograph::rules::FeedingSources::Only(Default::default());
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

/// A producer's stance `gap` cells beyond `actor`'s right side, on the flat
/// ground of `site`'s window.
fn short_food(g: &cosmograph::simulation::Genesis, site: u64, actor: u64, food: u64, gap: u64) -> Option<native::Pose> {
    let t = g.rules.spatial_templates.as_ref()?;
    let at = t.stands.get(&site)?.iter().find(|s| s.actor == Some(actor))?;
    let before = native::Pose { at: at.at, yaw: at.yaw, feet: at.feet.clone() };
    let actor_boxes = body::boxes(g.population.get(actor)?.body.as_ref()?, &before).ok()?;
    let view = cosmograph::terrain::View::of(g).ok()?;
    let window = &t.window;
    let volume = isometer_space::volume::Volume::lift(&view, site, window.min, window.max, window.depth, &[]).ok()?;
    let document = g.population.get(food)?.body.as_ref()?;
    let mut grounding = body::Grounding::new(&volume);
    let actor_right = actor_boxes.iter().map(|b| b[1][0]).max()?;
    // A long plant may need a quarter turn to lie beside the actor, and
    let yaws = (0..4).scan(before.yaw, |y, _| { let at = *y; *y = y.compose(isometer_core::Yaw::Quarter); Some(at) });
    // its root may slide along z for its parts to face the actor squarely.
    let shifts = |yaw| (0i64..96).flat_map(move |dx| (-8i64..=8).map(move |dz| (yaw, dx, dz)));
    yaws.flat_map(shifts).find_map(|(yaw, offset, dz)| {
        let p = grounding.at(document, before.at[0] + offset, before.at[2] + dz, yaw).ok()?;
        let boxes = body::boxes(document, &p).ok()?;
        let d = native::foraging::gap(&actor_boxes, &boxes);
        let left = boxes.iter().map(|b| b[0][0]).min().unwrap();
        (!body::intersects(&actor_boxes, &boxes) && d == gap && left >= actor_right + gap as i64).then_some(p)
    })
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
