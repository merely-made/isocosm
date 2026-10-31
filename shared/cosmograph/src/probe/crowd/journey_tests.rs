// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Command, Execution, Founding, Session, rules::*};

pub(super) const OWN: &str = "matter:0-0";
pub(super) const ACTOR: &str = "test:traveller";

pub(super) fn fixture(count: u64, bodies: bool) -> ProbeWorld {
    let mut g = Founding { seed:7, sites:3, population:12, lineages:2,
        cohort_size:3, ecology:true, bodies:bodies.then(Default::default),
        ..Default::default() }.generate().unwrap();
    let first = *g.population.groups.iter().find(|(_,group)|group.entity.lineage=="lineage:0").unwrap().0;
    g.population.groups.retain(|id,_| *id==first);
    let group = g.population.groups.get_mut(&first).unwrap();
    group.count=count; group.entity_mut().place=0; group.entity_mut().method=Method::Reactive;
    group.entity_mut().traits.insert(ACTOR.into());
    if !bodies { group.entity_mut().accounts=Ledger::from([(OWN.into(),32)]); }
    else {
        assert!(group.entity.body.is_some(), "the selected lineage has a generated native body");
        group.entity_mut().accounts.remove("world:soil");
        let room=crate::anatomy::room(&group.entity,&g.rules,OWN);
        crate::anatomy::give(group.entity_mut(),&g.rules,OWN,room).unwrap().unwrap();
        assert!(crate::anatomy::held(&group.entity,&g.rules,OWN)>2,
            "the bodily unit fixture actually provisions upkeep and a paid child");
    }
    g.rules.traits.insert(ACTOR.into()); g.rules.processes.clear();
    g.rules.epoch_ticks=2;
    for site in g.sites.values_mut() { site.routes.clear(); }
    for (from,to,travel) in [(0,1,3),(1,2,5),(0,2,7)] {
        g.sites.get_mut(&from).unwrap().routes.push(Route { to,travel,
            transmission:1_000_000,border:None,authored:None });
    }
    let mut w = ProbeWorld { genesis:g,ticks:10 };
    add(&mut w,"test:first",vec![Effect::Move { destination:1 }],1,0);
    add(&mut w,"test:next",vec![Effect::Move { destination:2 }],1,1);
    w
}

pub(super) fn add(w: &mut ProbeWorld, key: &str, effects: Vec<Effect>, period: Tick, priority: i32) {
    let mut p = crate::generate::process(key,Causation::Transition,effects);
    p.requires.push(Query::Trait { who:Binding::Actor,key:ACTOR.into() });
    p.period=Some(period); p.priority=priority; p.note=false;
    w.genesis.rules.processes.insert(key.into(),p);
}

pub(super) fn states<'a>(members: impl IntoIterator<Item=(&'a Entity,u64)>) -> BTreeMap<Entity,u64> {
    let mut out = BTreeMap::new();
    for (e,n) in members { *out.entry(normalize(e.clone())).or_default() += n; }
    out
}

/// Exact per-tick checks, not distributional certification. The native
/// saved tape is replayed in both execution modes at each final boundary.
pub(super) fn paired(w: &ProbeWorld, collect: bool,
    each: impl Fn(Tick,&Session,&Crowd)) {
    let mut g=w.genesis.clone(); g.dynamics=Some(7);
    let mut native=Session::new(g,Execution::Individuals).unwrap();
    let mut crowd=Crowd::new(w,7,Variant::Histogram).unwrap();
    let matter=native.sim.matter();
    let mut native_arrivals=0;
    for tick in 1..=w.ticks {
        native_arrivals+=native.advance_tick_with_flows().unwrap().result.arrivals;
        if collect { native.command(Command::Collect).unwrap(); }
        crowd.step().unwrap();
        let n=states(native.sim.state().population.groups.values().map(|g|(g.entity.as_ref(),g.count)));
        let h=states(crowd.bins.iter().map(|(e,&n)|(e,n)).chain(crowd.kin.values().map(|e|(e,1))));
        assert_eq!(h,n,"kernel journey agrees with native at every actual tick");
        assert_eq!(crowd.sites,native.sim.state().sites,"pass-start site ledgers at tick {tick}");
        assert_eq!(crowd.total_matter(),matter,"histogram conservation every actual tick");
        assert_eq!(native.sim.matter(),matter,"native conservation every actual tick");
        assert_eq!(crowd.work.arrivals,native_arrivals,"arrivals are counted outside evaluations");
        each(tick,&native,&crowd);
    }
    for mode in [Execution::Individuals,Execution::Grouped] {
        let replay=Session::load(native.save(),mode).unwrap();
        assert_eq!(replay.sim.state_hash(),native.sim.state_hash());
        assert_eq!(replay.checkpoints,native.checkpoints);
        assert_eq!(replay.entries,native.entries);
        assert_eq!(replay.sim.matter(),matter);
    }
}

#[test]
fn reactive_routes_land_at_authored_ticks_before_the_next_route() {
    let w=fixture(4,false);
    for collect in [false,true] {
        paired(&w,collect,|tick,_,c| {
            let e=c.bins.keys().next().unwrap();
            match tick {
                1..=3 => assert_eq!(e.journey.map(|j|j.arrives),Some(4),"kernel preserves first route duration"),
                4..=8 => assert_eq!(e.journey.map(|j|j.arrives),Some(9),"kernel preserves subsequent route duration"),
                _ => assert_eq!(e.resident_site(),Some(2)),
            }
        });
    }
}

#[test]
fn split_bins_and_kin_land_without_authored_evaluations() {
    let mut w=fixture(4,false); w.genesis.rules.processes.clear();
    let mut c=Crowd::new(&w,7,Variant::Histogram).unwrap();
    let resident=c.bins.keys().next().unwrap().clone();
    let mut traveller=resident.clone();
    crate::journey::depart(&mut traveller,&c.sites[&0],1,0).unwrap();
    c.moved(&resident,traveller,4);
    let before=c.bins.keys().next().unwrap().clone();
    let mut varied=before.clone(); varied.accounts.insert(OWN.into(),31);
    varied.accounts.insert("world:soil".into(),1);
    c.moved(&before,varied.clone(),2);
    let kin=c.lift(&varied);
    let mut dead=c.kin[&kin].clone(); dead.alive=false; c.insert_kin(kin,dead);
    let matter=c.total_matter();
    c.arrive_until(2).unwrap(); assert_eq!(c.work.arrivals,0);
    c.arrive_until(3).unwrap();
    assert_eq!(c.work.arrivals,4,"clock lands split bins and dead kin by represented count");
    assert_eq!((c.work.evaluations,c.work.represented),(0,0),"clock arrivals spend no authored evaluations");
    assert!(c.bins.keys().chain(c.kin.values()).all(|e|e.resident_site()==Some(1)));
    assert!(!c.kin[&kin].alive);
    assert_eq!(c.total_matter(),matter);
    let mut scheduled=fixture(4,false);
    scheduled.genesis.rules.processes.remove("test:next"); scheduled.ticks=4;
    paired(&scheduled,true,|tick,_,c| if tick==4 { assert_eq!(c.work.arrivals,4); });
}

#[test]
fn invalid_and_overflowing_departures_leave_bin_authority_unchanged() {
    let mut w=fixture(2,false);
    w.genesis.sites.get_mut(&0).unwrap().routes[0].travel=0;
    assert!(Crowd::new(&w,7,Variant::Histogram).is_err(),"zero route time is never clamped");
    w.genesis.sites.get_mut(&0).unwrap().routes[0].travel=Tick::MAX;
    w.genesis.rules.processes.remove("test:next");
    w.ticks=1;
    paired(&w,true,|_,n,c| {
        assert!(c.bins.keys().all(|e|e.journey.is_none()),"overflow cannot partially depart a bin");
        assert_eq!(n.sim.matter(),c.total_matter());
    });
    let valid=fixture(1,false);
    let mut c=Crowd::new(&valid,7,Variant::Histogram).unwrap();
    c.tick=Tick::MAX;
    let old=c.bins.clone();
    assert_eq!(c.step().unwrap_err(),"crowd tick overflows");
    assert_eq!(c.bins,old);
}

#[test]
fn transit_site_reads_cannot_take_origin_resources_or_conditions() {
    let mut w=fixture(3,false); w.genesis.rules.processes.remove("test:next");
    add(&mut w,"test:take-origin",vec![Effect::Transfer { from:Binding::Place,to:Binding::Actor,
        account:"world:soil".into(),amount:Amount::Fixed(1) }],1,20);
    add(&mut w,"test:condition",vec![Effect::Condition { key:"condition:wet".into(),delta:1 }],1,30);
    w.genesis.rules.conditions.insert("condition:wet".into());
    w.ticks=4;
    for collect in [false,true] {
        paired(&w,collect,|tick,_,c| {
            if tick<=3 {
                assert!(c.bins.keys().all(|e|e.accounts.get("world:soil").copied().unwrap_or(0)==0),
                    "transit cannot read or take the historical origin site");
            }
        });
    }
}

#[test]
fn averaging_never_pools_travellers_with_origin_residents() {
    let mut w=fixture(2,false); w.genesis.rules.processes.clear();
    let mut c=Crowd::new(&w,7,Variant::Averaged).unwrap();
    let e=c.bins.keys().next().unwrap().clone();
    let mut traveller=e.clone(); traveller.accounts.insert(OWN.into(),12);
    traveller.accounts.insert("world:soil".into(),20);
    traveller.journey=Some(Journey { from:0,to:1,departed:0,arrives:3 });
    c.moved(&e,traveller.clone(),1);
    let old=c.bins.clone(); c.average(); c.index_journeys();
    assert_eq!(c.bins,old,"averaging classes require actual site residence");
    c.arrive_until(3).unwrap();
    assert_eq!(c.work.arrivals,1);
}

#[test]
fn moving_deliberative_and_identity_worlds_keep_a_named_native_boundary() {
    let mut w=fixture(1,false);
    w.genesis.population.groups.values_mut().next().unwrap().entity_mut().method=Method::Deliberative;
    assert_eq!(Crowd::new(&w,7,Variant::Histogram).err().unwrap(),
        "moving aggregate worlds require the native deliberative chooser");
    w.genesis.population.groups.values_mut().next().unwrap().entity_mut().method=Method::Reactive;
    w.genesis.rules.processes.get_mut("test:first").unwrap().note=true;
    assert_eq!(Crowd::new(&w,7,Variant::Histogram).err().unwrap(),"identity-bound migration runs individually");
    w.genesis.rules.processes.clear();
    assert!(Crowd::new(&w,7,Variant::Histogram).is_ok(),"routes alone do not imply unsupported identity choice");
}
