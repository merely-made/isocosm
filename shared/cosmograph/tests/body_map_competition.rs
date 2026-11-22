// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Execution, Founding, Session, ecology::Ward, schema::Entity};

const PLANT: &str = "lineage:0";
const GRAZER: &str = "lineage:1";
const FEED: &str = "ecology:feed-1";
const RENT: &str = "guard:rent-0";
const HORIZON: u64 = 35;

fn founding() -> Founding {
    toml::from_str(include_str!("../examples/optional-gland.toml")).unwrap()
}

// The saved developmental draw identifies the form even after damage.
fn form(plant: &Entity) -> usize {
    usize::from(!plant.absent.contains(&(1, 0)))
}

fn verify_body(plant: &Entity) {
    let with = |function| {
        plant
            .parts
            .values()
            .filter(|p| p.cells.contains_key(function))
            .count()
    };
    assert_eq!(with("function:fix"), 2);
    assert_eq!(with("function:secrete"), form(plant));
}

fn living(session: &Session, lineage: &str) -> u64 {
    session
        .sim
        .state()
        .population
        .groups
        .values()
        .filter(|g| g.entity.alive && g.entity.lineage == lineage)
        .map(|g| g.count)
        .sum()
}

fn plants(session: &Session) -> [u64; 2] {
    let mut counts = [0; 2];
    for group in session.sim.state().population.groups.values() {
        let plant = &group.entity;
        if plant.alive && plant.lineage == PLANT {
            counts[form(plant)] += group.count;
        }
    }
    counts
}

#[test]
fn founder_draws_obey_the_map_and_invalid_odds_fail_before_development() {
    let mut found = founding();
    for (odds, expected) in [([0, 1], 1), ([1, 1], 0)] {
        found.body_maps.get_mut(PLANT).unwrap().absence = odds;
        let g = found.generate().unwrap();
        for group in g
            .population
            .groups
            .values()
            .filter(|g| g.entity.lineage == PLANT)
        {
            assert_eq!(form(&group.entity), expected);
            verify_body(&group.entity);
        }
    }
    found.body_maps.get_mut(PLANT).unwrap().absence = [1, 0];
    assert!(
        found
            .generate()
            .unwrap_err()
            .contains("invalid absence odds")
    );
}

#[test]
fn two_forms_grow_and_compete_in_the_same_world() {
    println!(
        "seed,grazing_period,gland_rent,plain_start,gland_start,plain_births,gland_births,plain_alive,gland_alive,plain_biomass_eaten,gland_biomass_eaten,gland_rent_paid,toxin_cells_logged,grazers_alive"
    );
    let mut births_across_runs = [0; 2];
    for seed in [9, 17, 31, 47] {
        for period in [0, 3, 1] {
            for rent in [1, 2, 3] {
                let mut found = Founding { seed, ..founding() };
                // One gland has nine cells. These are native upkeep rates of
                // one, two and three matter per payment.
                found.guard.as_mut().unwrap().rent_cells = match rent {
                    1 => 9,
                    2 => 8,
                    _ => 3,
                };
                let mut g = found.generate().unwrap();
                if period == 0 {
                    g.rules.processes.remove(FEED).unwrap();
                } else {
                    g.rules.processes.get_mut(FEED).unwrap().period = Some(period);
                }
                let mode = Execution::Grouped;
                let mut session = Session::new(g, mode).unwrap();
                let [plain_start, gland_start] = plants(&session);
                assert!(plain_start > 0 && gland_start > 0, "seed {seed}");
                for g in session
                    .sim
                    .state()
                    .population
                    .groups
                    .values()
                    .filter(|g| g.entity.lineage == PLANT)
                {
                    verify_body(&g.entity);
                }
                let matter = session.sim.matter();
                let (mut births, mut grazed) = ([0; 2], [0; 2]);
                let (mut paid, mut wounds) = (0, 0);
                for tick in 1..=HORIZON {
                    let before = session.sim.state().population.next_id;
                    let result = session.advance_tick_with_flows().unwrap();
                    let state = session.sim.state();
                    for flow in result.flows {
                        if flow.by_process(FEED)
                            && let Some(prey) =
                                flow.from.0.body().and_then(|id| state.population.get(id))
                            && prey.lineage == PLANT
                        {
                            grazed[form(prey)] += flow.amount * flow.count;
                        }
                        if flow.by_process(RENT) && !flow.internal() {
                            paid += flow.amount * flow.count;
                        }
                    }
                    wounds += session
                        .sim
                        .harms()
                        .iter()
                        .filter(|h| h.made_by == FEED && h.ward == Some(Ward::Toxin))
                        .map(|h| u64::from(h.cells))
                        .sum::<u64>();
                    for r in state
                        .relations
                        .iter()
                        .filter(|r| r.kind == "sim:parent" && r.subject >= before)
                    {
                        let child = state.population.get(r.subject).unwrap();
                        if child.lineage == PLANT {
                            let parent = state.population.get(r.object).unwrap();
                            assert_eq!(parent.absent, child.absent);
                            verify_body(child);
                            births[form(child)] += 1;
                        }
                    }
                    if (seed, period, rent, tick) == (9, 3, 2, 17) {
                        let save = session.save();
                        let reopened =
                            Session::load_json(&serde_json::to_vec(&save).unwrap(), mode).unwrap();
                        assert_eq!(reopened.save(), save);
                        let mut resumed = reopened.clone();
                        let mut continuous = session.clone();
                        continuous.advance(1).unwrap();
                        resumed.advance(1).unwrap();
                        assert_eq!(resumed.save(), continuous.save());
                        session = reopened;
                    }
                    assert_eq!(session.sim.matter(), matter);
                }
                if period == 0 {
                    assert_eq!((grazed, wounds), ([0, 0], 0));
                }
                let [plain_alive, gland_alive] = plants(&session);
                let [plain_births, gland_births] = births;
                let [plain_grazed, gland_grazed] = grazed;
                births_across_runs[0] += plain_births;
                births_across_runs[1] += gland_births;
                let grazers = living(&session, GRAZER);
                println!(
                    "{seed},{period},{rent},{plain_start},{gland_start},{plain_births},{gland_births},{plain_alive},{gland_alive},{plain_grazed},{gland_grazed},{paid},{wounds},{grazers}"
                );
            }
        }
    }
    assert!(births_across_runs.iter().all(|&count| count > 0));
}

#[test]
fn fixing_depends_on_each_cohorts_own_canopy() {
    use cosmograph::{flows::Holder, rules::Facing};
    use std::collections::BTreeSet;

    // Develop both placements, then give alternate founder cohorts the lower
    // sockets in one site. Both forms keep the same lineage, cells and stocks.
    let mut shaded = founding();
    shaded.body_maps.get_mut(PLANT).unwrap().grammar.seed[0].socket = Facing::Below;
    let lower = shaded.generate().unwrap();
    let mut mixed = founding().generate().unwrap();
    let mut with_canopy = BTreeSet::new();
    let mut without_canopy = BTreeSet::new();
    for (i, (&first, group)) in mixed.population.groups.iter_mut()
        .filter(|(_, g)| g.entity.lineage == PLANT).enumerate()
    {
        let members = first..first + group.count;
        if i % 2 == 0 {
            with_canopy.extend(members);
        } else {
            group.entity_mut().body = lower.population.get(first).unwrap().body.clone();
            without_canopy.extend(members);
        }
    }
    // Isolate one producer process so every funded body gets a turn. Its two
    // site inputs and every founder account are still the ordinary ones.
    const GROW: &str = "ecology:produce-0";
    mixed.rules.processes.retain(|key, _| key == GROW);
    fn growers(run: &mut Session) -> BTreeSet<u64> {
        let mut actors = BTreeSet::new();
        for _ in 0..2 {
            for flow in run.advance_tick_with_flows().unwrap().flows {
                if flow.by_process(GROW) && matches!(flow.from.0, Holder::Site(_)) && flow.from.1 == "world:soil" {
                    let first = flow.to.0.body().unwrap();
                    actors.extend(first..first + flow.count);
                }
            }
        }
        actors
    }

    assert!(!without_canopy.is_empty() && !with_canopy.is_empty());
    for mode in [Execution::Grouped, Execution::Individuals] {
        let mut run = Session::new(mixed.clone(), mode).unwrap();
        let matter = run.sim.matter();
        assert_eq!(growers(&mut run), with_canopy);

        // One member loses its two fixing plates a turn apart. Its cohort
        // mates can still grow on each pass, including after reopening.
        let actor = *with_canopy.first().unwrap();
        let plate = |s: &Session| cosmograph::trophic::fixing_plate(s.sim.state().population.get(actor).unwrap());
        let remove = |s: &mut Session| {
            let part = plate(s).unwrap();
            s.command(cosmograph::Command::Wound { entity: actor, part: Some(part), cells: 9 }).unwrap();
        };
        remove(&mut run);
        assert_eq!(growers(&mut run), with_canopy);
        remove(&mut run);
        assert!(plate(&run).is_none());
        let bytes = serde_json::to_vec(&run.save()).unwrap();
        let mut reopened = Session::load_json(&bytes, mode).unwrap();
        let mut able = with_canopy.clone();
        able.remove(&actor);
        assert_eq!(growers(&mut run), able);
        assert_eq!(growers(&mut reopened), able);
        assert_eq!(run.save(), reopened.save());
        assert_eq!(run.sim.matter(), matter);
    }
}

#[test]
fn the_canopy_requirement_follows_light_driven_growth() {
    use cosmograph::{bodied::Bodies, ecology::{Mutualism, Rates}, rules::Query};

    let found = Founding {
        ecology: true, ecology_rates: Rates::A1,
        bodies: Some(Bodies { canopy_growth: true, ..Bodies::default() }),
        mutualism: Some(Mutualism::default()),
        ..Founding::default()
    };
    let partnered = found.generate().unwrap();
    let needs_canopy = |world: &cosmograph::simulation::Genesis, key: &str| {
        world.rules.processes[key].requires.contains(&Query::FixingCanopy)
    };
    for key in ["ecology:produce-0", "mutualism:grow-0"] {
        assert!(needs_canopy(&partnered, key), "{key}");
    }

    let seasonal = Founding { ecology_rates: Rates::WorldTime, mutualism: None, ..found };
    let stored = seasonal.generate().unwrap();
    assert!(needs_canopy(&stored, "ecology:produce-0-reserve"));
    assert!(!needs_canopy(&stored, "ecology:regrow-0"), "regrowth spends stored matter");
}
