// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Founding, Session, Command, Execution, rules::*, simulation::{Genesis, Receipt}};

pub(super) const OWN: &str = "matter:0-0";
pub(super) const ACTOR: &str = "test:traveller";

pub(super) fn fixture(count: u64) -> (Genesis, Id) {
    let mut g = Founding { seed: 7, sites: 3, population: 12, lineages: 1,
        cohort_size: 3, ecology: true, ..Default::default() }.generate().unwrap();
    let id = *g.population.groups.keys().next().unwrap();
    g.population.groups.retain(|first, _| *first == id);
    let group = g.population.groups.get_mut(&id).unwrap();
    group.count = count;
    group.entity.place = 0;
    group.entity.method = Method::Reactive;
    group.entity.accounts = Ledger::from([(OWN.into(), 32)]);
    group.entity.traits.insert(ACTOR.into());
    g.rules.traits.insert(ACTOR.into());
    g.rules.processes.clear();
    g.rules.epoch_ticks = 2;
    for site in g.sites.values_mut() { site.routes.clear(); }
    for (from, to, travel) in [(0,1,3), (1,2,5), (0,2,7)] {
        g.sites.get_mut(&from).unwrap().routes.push(Route {
            to, travel, transmission: 1_000_000, border: None, authored: None,
        });
    }
    for (key, destination) in [("test:first",1), ("test:next",2)] {
        add(&mut g, key, Causation::Choice, vec![Effect::Move { destination }], None, 0);
    }
    (g, id)
}

pub(super) fn add(g: &mut Genesis, key: &str, kind: Causation, effects: Vec<Effect>,
                  period: Option<Tick>, priority: i32) {
    let mut p = crate::generate::process(key, kind, effects);
    p.requires.push(Query::Trait { who: Binding::Actor, key: ACTOR.into() });
    p.period = period;
    p.priority = priority;
    p.note = true;
    g.rules.processes.insert(key.into(), p);
}

pub(super) fn act(s: &mut Session, actor: Id, target: Option<Id>, process: &str) -> Receipt {
    serde_json::from_str(&s.command(Command::Act {
        actor, target, process: process.into(), cause: None,
    }).unwrap()).unwrap()
}

pub(super) fn replay(s: &Session) {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load(s.save(), mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash(), "journey replay witness");
        assert_eq!(loaded.entries, s.entries);
        assert_eq!(loaded.checkpoints, s.checkpoints);
        assert_eq!(loaded.sim.matter(), s.sim.matter());
    }
}

#[test]
fn route_duration_and_next_route_use_the_traversed_route() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, id) = fixture(1);
        let mut s = Session::new(g, mode).unwrap();
        let matter = s.sim.matter();
        s.advance(1).unwrap();
        assert!(act(&mut s, id, None, "test:first").accepted());
        let e = s.sim.state().population.get(id).unwrap();
        assert_eq!(e.resident_site(), None, "departure cannot arrive immediately");
        assert_eq!(e.journey, Some(Journey { from:0, to:1, departed:1, arrives:4 }),
            "817 arrival uses the traversed route's authored duration");
        assert_eq!(e.visits, vec![Visit { place:0, from:0, until:1 }]);
        assert!(!act(&mut s, id, None, "test:next").accepted());
        assert_eq!(s.advance(2).unwrap().arrivals, 0);
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(), None);
        assert_eq!(s.advance(1).unwrap().arrivals, 1);
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(), Some(1));
        assert!(act(&mut s, id, None, "test:next").accepted());
        assert_eq!(s.sim.state().population.get(id).unwrap().journey.unwrap().arrives,
            9, "next route must use its own five-tick travel");
        s.advance(4).unwrap();
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(), None);
        s.advance(1).unwrap();
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(), Some(2));
        assert_eq!(s.sim.matter(), matter);
        replay(&s);
    }
}

#[test]
fn invalid_route_times_and_arrival_overflow_never_change_authority() {
    let (mut invalid,id) = fixture(1);
    invalid.sites.get_mut(&0).unwrap().routes[0].travel=0;
    for mode in [Execution::Individuals,Execution::Grouped] {
        assert!(Session::new(invalid.clone(),mode).is_err(),
            "a zero-time authored route remains an admission refusal");
    }
    let mut actor = invalid.population.get(id).unwrap().clone();
    let before = actor.clone();
    assert_eq!(depart(&mut actor,&invalid.sites[&0],1,1).unwrap_err(),
        "route travel must be positive");
    assert_eq!(actor,before,"an invalid route cannot partially depart");
    let (mut g,id) = fixture(1);
    g.sites.get_mut(&0).unwrap().routes[0].travel=Tick::MAX;
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        s.advance(1).unwrap();
        let hash = s.sim.state_hash(); let matter=s.sim.matter();
        let population=s.sim.state().population.clone();
        let refused=s.command_with_flows(Command::Act { actor:id,target:None,
            process:"test:first".into(),cause:None }).unwrap();
        let receipt: Receipt=serde_json::from_str(&refused.result).unwrap();
        assert!(matches!(receipt.outcome,crate::simulation::Outcome::Blocked(ref why)
            if why.contains("journey arrival tick overflows")),
            "route arrival overflow must refuse, never saturate");
        assert!(refused.flows.is_empty());
        assert_eq!(s.sim.state_hash(),hash);
        assert_eq!(s.sim.state().population,population);
        assert_eq!(s.sim.matter(),matter);
        replay(&s);
    }
}

#[test]
fn cohorts_split_and_arrive_without_a_due_process() {
    let (g, id) = fixture(3);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        s.advance(1).unwrap();
        assert!(act(&mut s, id+1, None, "test:first").accepted());
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(), Some(0));
        assert_eq!(s.sim.state().population.get(id+2).unwrap().resident_site(), Some(0));
        assert_eq!(s.sim.state().population.get(id+1).unwrap().resident_site(), None);
        s.command(Command::Collect).unwrap();
        replay(&s);
        let before = s.sim.matter();
        let work = s.advance(3).unwrap();
        assert_eq!((work.evaluations, work.represented, work.arrivals), (0,0,1));
        assert_eq!(s.sim.state().population.get(id+1).unwrap().resident_site(), Some(1));
        assert_eq!(s.sim.matter(), before);
        replay(&s);
    }
}

#[test]
fn wide_and_tick_advances_land_identical_cohorts_and_checkpoints() {
    let (mut g, id) = fixture(3);
    let p = g.rules.processes.get_mut("test:first").unwrap();
    p.period = Some(1);
    p.causation = Causation::Transition;
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut wide = Session::new(g.clone(), mode).unwrap();
        let mut ticks = Session::new(g.clone(), mode).unwrap();
        let matter = wide.sim.matter();
        let work = wide.advance(4).unwrap();
        let mut arrivals = 0;
        for _ in 0..4 {
            arrivals += ticks.advance_tick_with_flows().unwrap().result.arrivals;
            assert_eq!(ticks.sim.matter(), matter);
        }
        assert_eq!(work.arrivals, 3);
        assert_eq!(arrivals, 3);
        assert_eq!(wide.sim.state_hash(), ticks.sim.state_hash());
        assert_eq!(wide.checkpoints, ticks.checkpoints);
        // The already-landed members cannot repeat a route absent at site 1.
        for member in id..id+3 {
            assert_eq!(wide.sim.state().population.get(member).unwrap().resident_site(), Some(1));
        }
        replay(&wide);
    }
}

#[test]
fn upkeep_carries_soil_and_a_paid_child_inherits_remaining_journey() {
    let (mut g, id) = fixture(1);
    add(&mut g, "test:upkeep", Causation::Transition, vec![
        Effect::Transform { who:Binding::Actor, take:Ledger::from([(OWN.into(),1)]),
            give:Ledger::from([("world:soil".into(),1)]), conversion:None },
        Effect::Transfer { from:Binding::Actor, to:Binding::Place,
            account:"world:soil".into(), amount:Amount::Fixed(1) },
    ], Some(1), 20);
    add(&mut g, "test:birth", Causation::Transition,
        vec![Effect::Birth { provision:Ledger::from([(OWN.into(),8)]) }], None, 40);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        let matter = s.sim.matter();
        s.advance(1).unwrap();
        assert!(act(&mut s, id, None, "test:first").accepted());
        let child = s.sim.state().population.next_id;
        let paid = s.command_with_flows(Command::Act { actor:id, target:None,
            process:"test:birth".into(), cause:None }).unwrap();
        let receipt: Receipt = serde_json::from_str(&paid.result).unwrap();
        assert!(receipt.accepted());
        assert_eq!(s.sim.state().population.get(child).unwrap().journey,
            s.sim.state().population.get(id).unwrap().journey, "paid child inherits remaining journey");
        assert_eq!(s.sim.state().population.get(child).unwrap().accounts[OWN], 8);
        assert!(paid.flows.iter().any(|f| f.from.0.body()==Some(id)
            && f.to.0.body()==Some(child) && f.amount==8));
        let origin = s.sim.state().sites[&0].accounts["world:soil"];
        let dest = s.sim.state().sites[&1].accounts["world:soil"];
        for tick in 2..=3 {
            let advanced = s.advance_tick_with_flows().unwrap();
            assert_eq!(advanced.tick, tick);
            assert!(!advanced.flows.iter().any(|f| matches!(f.to.0, crate::flows::Holder::Site(_))));
            for member in [id, child] {
                let e = s.sim.state().population.get(member).unwrap();
                assert_eq!(e.accounts.get("world:soil").copied().unwrap_or(0), tick-1,
                    "produced soil stays with traveller");
                assert_eq!(e.resident_site(), None);
            }
            assert_eq!(s.sim.state().sites[&0].accounts["world:soil"], origin);
            assert_eq!(s.sim.state().sites[&1].accounts["world:soil"], dest);
            assert_eq!(s.sim.matter(), matter);
        }
        let advanced = s.advance_tick_with_flows().unwrap();
        assert_eq!(advanced.result.arrivals, 2);
        for member in [id, child] {
            let e = s.sim.state().population.get(member).unwrap();
            assert_eq!(e.resident_site(), Some(1));
            assert_eq!(e.accounts["world:soil"], 2, "arrival has no invented bulk deposit");
        }
        assert_eq!(s.sim.state().sites[&1].accounts["world:soil"], dest+2);
        assert_eq!(s.sim.matter(), matter);
        replay(&s);
    }
}

pub(super) fn bodied_fixture() -> (Genesis,Id) {
    let mut g = Founding { seed:7, sites:3, population:12, lineages:1,
        cohort_size:3, ecology:true, bodies:Some(Default::default()),
        ..Default::default() }.generate().unwrap();
    let id = *g.population.groups.keys().next().unwrap();
    g.population.groups.retain(|first,_| *first==id);
    let group = g.population.groups.get_mut(&id).unwrap();
    group.count = 1; group.entity.place = 0; group.entity.method = Method::Reactive;
    group.entity.traits.insert(ACTOR.into());
    assert!(group.entity.body.is_some());
    assert!(!group.entity.parts.is_empty());
    assert!(crate::anatomy::held(&group.entity,&g.rules,OWN)>=1);
    g.rules.traits.insert(ACTOR.into()); g.rules.processes.clear();
    g.sites.get_mut(&0).unwrap().routes = vec![Route { to:1,travel:3,
        transmission:1_000_000,border:None,authored:None }];
    add(&mut g,"test:first",Causation::Choice,vec![Effect::Move { destination:1 }],None,0);
    (g,id)
}

#[test]
fn a_bodied_paid_birth_in_transit_uses_native_part_matter() {
    let (mut g,id) = bodied_fixture();
    add(&mut g,"test:birth",Causation::Transition,
        vec![Effect::Birth { provision:Ledger::from([(OWN.into(),1)]) }],None,0);
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let matter = s.sim.matter();
        assert!(act(&mut s,id,None,"test:first").accepted());
        let child = s.sim.state().population.next_id;
        let paid = s.command_with_flows(Command::Act { actor:id,target:None,
            process:"test:birth".into(),cause:None }).unwrap();
        assert!(serde_json::from_str::<Receipt>(&paid.result).unwrap().accepted());
        let young = s.sim.state().population.get(child).unwrap();
        assert_eq!(young.journey,s.sim.state().population.get(id).unwrap().journey);
        assert_eq!(crate::anatomy::held(young,&s.sim.genesis().rules,OWN),1);
        assert!(paid.flows.iter().any(|f| f.from.0.body()==Some(id)
            && matches!(f.to.0,crate::flows::Holder::Part(c,_) if c==child)));
        assert_eq!(s.advance(3).unwrap().arrivals,2);
        assert_eq!(s.sim.state().population.get(child).unwrap().resident_site(),Some(1));
        assert_eq!(s.sim.matter(),matter);
        replay(&s);
    }
}

#[test]
fn authored_spend_upkeep_mineralizes_into_carried_soil() {
    let (mut g,id) = fixture(1);
    add(&mut g,"test:spend",Causation::Transition,vec![Effect::Spend {
        from:vec![OWN.into()],to:Binding::Place,amount:Amount::Fixed(1),
        into:Some("world:soil".into()),
    }],Some(1),20);
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let matter = s.sim.matter();
        assert!(act(&mut s,id,None,"test:first").accepted());
        let destination = s.sim.state().sites[&1].accounts["world:soil"];
        for tick in 1..=2 {
            let paid = s.advance_tick_with_flows().unwrap();
            let e = s.sim.state().population.get(id).unwrap();
            assert_eq!(e.accounts[OWN],32-tick);
            assert_eq!(e.accounts["world:soil"],tick,"820 mineralization stays conserved on body");
            assert!(paid.flows.iter().all(|f| f.from.0.body()==Some(id) && f.to.0.body()==Some(id)));
            assert_eq!(s.sim.state().sites[&1].accounts["world:soil"],destination);
            assert_eq!(s.sim.matter(),matter);
        }
        let landed = s.advance_tick_with_flows().unwrap();
        assert_eq!(landed.result.arrivals,1);
        assert_eq!(s.sim.state().sites[&1].accounts["world:soil"],destination+1);
        assert_eq!(s.sim.state().population.get(id).unwrap().accounts["world:soil"],2);
        replay(&s);
    }
}

#[test]
fn age_death_carries_a_conserved_corpse_until_destination() {
    let (mut g, id) = fixture(1);
    add(&mut g, "test:age", Causation::Transition, vec![Effect::Death], Some(1), 30);
    g.rules.processes.get_mut("test:age").unwrap().requires.push(Query::Age { at_least:2 });
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        let matter = s.sim.matter();
        s.advance(1).unwrap();
        assert!(act(&mut s, id, None, "test:first").accepted());
        s.advance(1).unwrap();
        let e = s.sim.state().population.get(id).unwrap();
        assert!(!e.alive, "native age continues while travelling");
        assert_eq!(e.resident_site(), None);
        assert_eq!(e.accounts[OWN], 32);
        s.command(Command::Collect).unwrap();
        s.advance(1).unwrap();
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(), None);
        s.advance(1).unwrap();
        let e = s.sim.state().population.get(id).unwrap();
        assert!(!e.alive);
        assert_eq!(e.resident_site(), Some(1), "corpse finishes remaining journey");
        assert_eq!(e.accounts[OWN], 32);
        assert_eq!(s.sim.matter(), matter);
        replay(&s);
    }
}

#[test]
fn arrivals_do_not_spend_the_authored_evaluation_budget() {
    let (mut g, id) = fixture(3);
    g.rules.limits.events_per_advance = 2;
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    for member in id..id+3 { assert!(act(&mut s, member, None, "test:first").accepted()); }
    s.command(Command::Collect).unwrap();
    let work = s.advance(3).unwrap();
    assert_eq!((work.represented,work.arrivals),(0,3),"259 budgets only executed evaluations");
    replay(&s);
}

#[test]
fn a_due_budget_failure_restores_already_landed_cohorts() {
    let (mut g, id) = fixture(3);
    g.rules.limits.events_per_advance = 2;
    add(&mut g,"test:after-arrival",Causation::Transition,vec![],Some(3),0);
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    for member in id..id+3 { assert!(act(&mut s, member, None, "test:first").accepted()); }
    s.command(Command::Collect).unwrap();
    let before = s.save();
    assert!(s.advance(3).unwrap_err().contains("operation budget"));
    assert_eq!(s.save(), before, "authored budget rollback restores landed journey authority");
    assert!(s.sim.frame.is_none());
    assert!(s.sim.journeys.is_none());
    replay(&s);
}

#[test]
fn v4_witnesses_movement_and_keeps_explicit_v3_semantics() {
    let (g, id) = fixture(1);
    for movement in [Movement::Journey, Movement::ImmediateV3] {
        let mut s = Session::new(g.clone(), Execution::Individuals).unwrap();
        s.sim.state.movement = movement;
        assert!(act(&mut s, id, None, "test:first").accepted());
        let saved = s.save();
        assert_eq!(saved.version, 4);
        assert_eq!(saved.movement, Some(movement));
        replay(&s);
        let mut missing = saved.clone(); missing.movement = None;
        assert!(Session::load(missing, Execution::Individuals).is_err());
        let mut tampered = saved; tampered.movement = Some(if movement.immediate() {
            Movement::Journey } else { Movement::ImmediateV3 });
        assert!(Session::load(tampered, Execution::Individuals).is_err());
        let fork = s.fork_at(s.sim.state().tick, "branch:journey-test".into()).unwrap();
        assert_eq!(fork.sim.state().movement, movement);
        assert_eq!(fork.sim.state_hash(), s.sim.state_hash());
        let mut other = s.clone(); other.sim.state.movement = if movement.immediate() {
            Movement::Journey } else { Movement::ImmediateV3 };
        assert!(s.merge(&other).is_err());
    }
}
