// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use super::journey_tests::{ACTOR, OWN, add, fixture, paired};
use crate::{population::Population, probe::BodyFounding, rules::*};

#[test]
fn generic_paid_children_and_soil_keep_remaining_journeys_in_both_forms() {
    for bodies in [false,true] {
        let mut w=fixture(2,bodies); w.ticks=4;
        w.genesis.rules.processes.remove("test:next");
        let provision=if bodies { 1 } else { 8 };
        add(&mut w,"test:birth",vec![Effect::Birth {
            provision:Ledger::from([(OWN.into(),provision)]) }],2,40);
        add(&mut w,"test:upkeep",vec![Effect::Transform { who:Binding::Actor,
            take:Ledger::from([(OWN.into(),1)]),give:Ledger::from([("world:soil".into(),1)]),conversion:None },
            Effect::Transfer { from:Binding::Actor,to:Binding::Place,account:"world:soil".into(),
                amount:Amount::Fixed(1) }],1,20);
        for collect in [false,true] {
            paired(&w,collect,|tick,_,c| if tick==2 {
                let children:Vec<_>=c.bins.keys().filter(|e|e.born==2).collect();
                assert!(!children.is_empty(),"paid native births occur while travelling");
                assert!(children.iter().all(|e|e.journey.is_some_and(|j|j.arrives==4)),
                    "kernel newborns retain the remaining journey");
                assert!(c.bins.keys().filter(|e|e.born==0).all(|e|
                    e.accounts.get("world:soil").copied().unwrap_or(0)==2),
                    "kernel upkeep carries soil without depositing at origin");
            });
        }
    }
}

#[test]
fn spend_upkeep_and_age_continue_then_corpse_arrives_conserved() {
    let mut w=fixture(3,false);w.ticks=5;
    w.genesis.rules.processes.remove("test:next");
    add(&mut w,"test:spend",vec![Effect::Spend { from:vec![OWN.into()],to:Binding::Place,
        amount:Amount::Fixed(1),into:Some("world:soil".into()) }],1,20);
    add(&mut w,"test:age",vec![Effect::Death],1,30);
    w.genesis.rules.processes.get_mut("test:age").unwrap().requires.push(Query::Age { at_least:2 });
    for collect in [false,true] {
        paired(&w,collect,|tick,_,c| {
            let e=c.bins.keys().next().unwrap();
            if tick>=2 {
                assert!(!e.alive,"age continues for travelling reactive members");
                assert_eq!(e.accounts.get("world:soil").copied().unwrap_or(0),2,
                    "authored spend carries produced soil until a later deposit");
            }
            if tick==3 { assert_eq!(e.resident_site(),None); }
            if tick>=4 { assert_eq!(e.resident_site(),Some(1),"kernel corpse finishes the remaining journey"); }
        });
    }
}

fn hunters(first: bool) -> ProbeWorld {
    let mut w=fixture(1,false); w.ticks=4;
    w.genesis.rules.processes.remove("test:next");
    let prey=w.genesis.population.groups.values().next().unwrap().entity.clone();
    let mut hunter=prey.clone();hunter.traits.remove(ACTOR);hunter.traits.insert("test:hunter".into());
    hunter.lineage="lineage:1".into();hunter.accounts.clear();
    let mut destination=hunter.clone();destination.place=1;
    let members=if first { vec![hunter,prey,destination] } else { vec![destination,prey,hunter] };
    let mut population=Population::default();
    for e in members { population.insert(e,1).unwrap(); }
    w.genesis.population=population;
    w.genesis.rules.traits.insert("test:hunter".into());
    let mut p=crate::generate::process("test:meal",Causation::Choice,vec![Effect::Eat {
        from:Binding::Target,amount:Amount::Fixed(1),into:"matter:1-0".into(),
        of:vec![OWN.into()],whole:false }]);
    p.requires.push(Query::Trait { who:Binding::Actor,key:"test:hunter".into() });
    p.target=Some(Target { same_place:true,alive:None,lineage:Some(prey_lineage(&w)),
        among:Default::default(),weighted:true });
    p.period=Some(1);p.priority=10;p.note=false;
    w.genesis.rules.processes.insert(p.id.clone(),p);
    add(&mut w,"test:age",vec![Effect::Death],1,30);
    w.genesis.rules.processes.get_mut("test:age").unwrap().requires.push(Query::Age { at_least:2 });
    w
}

fn prey_lineage(w: &ProbeWorld) -> Key {
    w.genesis.population.groups.values().find(|g|g.entity.traits.contains(ACTOR)).unwrap().entity.lineage.clone()
}

#[test]
fn first_and_last_hunters_see_arrival_but_never_origin_or_transit_prey() {
    for first in [false,true] {
        let w=hunters(first);
        for collect in [false,true] {
            paired(&w,collect,|tick,_,c| {
                let eaten=|site|c.bins.iter().filter(|(e,_)|e.resident_site()==Some(site)
                    && e.traits.contains("test:hunter")).map(|(e,n)|
                        e.accounts.get("matter:1-0").copied().unwrap_or(0)*n).sum::<u64>();
                assert_eq!(eaten(0),0,"454 hunters cannot eat at the historical origin");
                assert_eq!(eaten(1),u64::from(tick==4),"arrival precedes destination feeding pass");
            });
        }
    }
}

/// The existing fixed-recipe nursing scene, now with an authored route.
/// No production generator or A1 profile/rate is modified by this fixture.
fn nursing_journey() -> ProbeWorld {
    let mut w=BodyFounding { seed:1,ticks:4,sites:[1,1],variance:[0,0],absence:[0,0],
        riff:[0,0],vary:[0,0],semelparous:0,..Default::default() }.generate().unwrap();
    let g=&mut w.genesis;
    g.rules.processes.retain(|id,_|["body:bear-1","body:nurse-1","body:wean-1"].contains(&id.as_str()));
    let parent=g.population.groups.values().find(|g|g.entity.lineage=="lineage:1").unwrap().entity.clone();
    let mut parent=parent;
    parent.traits.insert(ACTOR.into());parent.method=Method::Reactive;
    for key in ["tissue:1","reserve:1","provision:1"] {
        let room=crate::anatomy::room(&parent,&g.rules,key);
        crate::anatomy::give(&mut parent,&g.rules,key,room).unwrap().unwrap();
    }
    let origin=parent.place;
    let mut site=g.sites[&origin].clone();site.routes.clear();g.sites.insert(99,site);
    g.sites.get_mut(&origin).unwrap().routes=vec![Route { to:99,travel:3,
        transmission:1_000_000,border:None,authored:None }];
    let mut population=Population::default();population.insert(parent,2).unwrap();g.population=population;
    g.rules.traits.insert(ACTOR.into());
    add(&mut w,"test:depart",vec![Effect::Move { destination:99 }],1,-100);
    w
}

#[test]
fn paid_bodied_broods_and_bonded_kin_arrive_by_the_native_clock() {
    let w=nursing_journey();
    for collect in [false,true] {
        paired(&w,collect,|tick,_,c| {
            if tick==1 {
                assert!(!c.kin.is_empty(),"bodied births actually retain bonded kin in transit");
                assert!(c.kin.values().all(|e|e.journey.is_some_and(|j|j.arrives==4)));
            }
            if tick==4 {
                assert!(c.bins.keys().chain(c.kin.values()).all(|e|e.resident_site()==Some(99)),
                    "kin and broods arrive before destination nursing and weaning");
            }
        });
    }
}

#[test]
fn target_harm_excludes_transit_until_the_native_arrival_pass() {
    // Exercise the actual predicate/candidate pass with one native world actor.
    let mut w=nursing_journey();w.ticks=4;
    w.genesis.rules.processes.retain(|id,_|id=="test:depart");
    let mut ground=BodyFounding { seed:1,sites:[1,1],..Default::default() }.generate().unwrap()
        .genesis.population.groups.values().find(|g|g.entity.kingdom=="kingdom:world").unwrap().entity.clone();
    ground.place=99;ground.accounts.clear();
    ground.traits.insert("test:hazard".into());
    w.genesis.rules.traits.insert("test:hazard".into());
    w.genesis.population.insert(ground,1).unwrap();
    let mut p=crate::generate::process("test:hazard",Causation::Agentless,vec![Effect::Wound {
        who:Binding::Target,cells:Amount::Fixed(0),slot:0 }]);
    p.requires.push(Query::Trait { who:Binding::Actor,key:"test:hazard".into() });
    p.target=Some(Target { same_place:true,alive:Some(true),lineage:Some("lineage:1".into()),
        among:Default::default(),weighted:true });
    p.period=Some(1);p.priority=20;p.note=false;
    w.genesis.rules.processes.insert(p.id.clone(),p);
    paired(&w,true,|tick,_,c| {
        assert_eq!(c.meals.get("test:hazard").map(|m|m.count).unwrap_or(0),u64::from(tick==4),
            "native target candidates require resident destination bodies");
        assert_eq!(c.total_matter(),c.matter);
    });
}

#[test]
fn competition_rounds_wait_for_residence_without_consuming_transit_food() {
    let mut w=crate::probe::ProbeFounding { seed:3,ticks:3,lineages:1,sites:[2,2],
        members:[4,4],cohort:4,regrowth:[0,0],..Default::default() }.generate().unwrap();
    for p in w.genesis.rules.processes.values_mut() { p.period=None; }
    for site in w.genesis.sites.values_mut() { site.accounts.insert("world:food".into(),1000); }
    for g in w.genesis.population.groups.values_mut() {
        if g.entity.kingdom!="kingdom:world" {
            g.entity.place=0;g.entity.arrived=0;g.entity.patch=None;
            g.entity.journey=Some(Journey { from:0,to:1,departed:0,arrives:3 });
        }
    }
    let mut c=Crowd::new(&w,7,Variant::Histogram).unwrap();
    let matter=c.total_matter();
    let mut ticks=0;
    let exact=crate::probe::run_exact_traced(&w,7,true,&mut |n| {
        c.step().unwrap();ticks+=1;
        let native=super::journey_tests::states(n.state().population.groups.values().map(|g|(&g.entity,g.count)));
        let histogram=super::journey_tests::states(c.bins.iter().map(|(e,&n)|(e,n)));
        assert_eq!(histogram,native,"per-tick native/histogram competition states after journey clock");
        assert_eq!(c.sites,n.state().sites);
        assert_eq!(c.total_matter(),matter);
        assert_eq!(n.matter(),matter);
        assert_eq!(c.sites[&0].accounts["world:food"],1000,
            "competition rounds cannot use the historical origin of transit members");
        if ticks<3 { assert_eq!(c.sites[&1].accounts["world:food"],1000); }
    }).unwrap();
    assert_eq!(ticks,3);
    assert_eq!(c.work.arrivals,exact.work.arrivals);
    assert!(c.sites[&1].accounts["world:food"]<1000,"destination competition actually lands after arrival");
}
