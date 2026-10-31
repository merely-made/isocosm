// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{*, tests::*};
use crate::{Session, Execution, Command, rules::*, simulation::{Genesis, Receipt}};

fn neighbour(g: &mut Genesis, actor: Id, place: Id) -> Id {
    let mut e = g.population.get(actor).unwrap().clone();
    e.place = place;
    e.method = Method::Inert;
    e.traits.remove(ACTOR);
    g.population.insert(e, 1).unwrap()
}

fn meal(g: &mut Genesis, period: Option<Tick>, same_place: bool) {
    let mut p = crate::generate::process("test:meal", Causation::Transition,
        vec![Effect::Eat { from:Binding::Target, amount:Amount::Fixed(1),
            into:OWN.into(), of:vec![OWN.into()], whole:false }]);
    g.rules.traits.insert("test:hunter".into());
    p.requires.push(Query::Trait { who:Binding::Actor, key:"test:hunter".into() });
    p.target = Some(Target { same_place, alive:None, lineage:Some("lineage:0".into()),
        among:Default::default(), weighted:false });
    p.period = period;
    g.rules.processes.insert(p.id.clone(), p);
}

#[test]
fn arrival_precedes_the_due_feeding_pass_in_both_actor_orders() {
    for order in [false,true] {
        let (mut g, old) = fixture(1);
        let second = neighbour(&mut g, old, 1);
        let (prey, hunter) = if order { (second,old) } else { (old,second) };
        let p = g.population.lift(prey).unwrap();
        p.place = 0; p.traits.insert(ACTOR.into()); p.method = Method::Reactive;
        let h = g.population.lift(hunter).unwrap();
        h.place = 1; h.traits.remove(ACTOR); h.traits.insert("test:hunter".into());
        h.method = Method::Reactive;
        meal(&mut g, Some(1), true);
        for mode in [Execution::Individuals,Execution::Grouped] {
            let mut s = Session::new(g.clone(), mode).unwrap();
            s.sim.watch("test:meal");
            let matter = s.sim.matter();
            assert!(act(&mut s, prey, None, "test:first").accepted());
            for _ in 0..2 {
                s.advance_tick_with_flows().unwrap();
                assert!(s.sim.take_watched().is_empty(), "no resident prey before arrival");
                assert_eq!(s.sim.matter(), matter);
            }
            let final_tick = s.advance_tick_with_flows().unwrap();
            assert_eq!(final_tick.result.arrivals, 1);
            let watch = s.sim.take_watched();
            assert_eq!(watch.len(), 1, "destination pass observes actual arrival");
            assert_eq!((watch[0].tick, watch[0].actor, watch[0].target), (3,hunter,Some(prey)));
            assert!(final_tick.flows.iter().any(|f| f.from.0.body()==Some(prey)
                && f.to.0.body()==Some(hunter)));
            assert_eq!(s.sim.matter(), matter);
            replay(&s);
        }
    }
}

#[test]
fn travelling_living_and_dead_prey_are_unavailable_even_to_global_meals() {
    let (mut g, prey) = fixture(1);
    let hunter = neighbour(&mut g, prey, 1);
    g.population.lift(hunter).unwrap().traits.insert("test:hunter".into());
    meal(&mut g, None, false);
    for mode in [Execution::Individuals,Execution::Grouped] {
        for dead in [false,true] {
            let mut s = Session::new(g.clone(), mode).unwrap();
            assert!(act(&mut s, prey, None, "test:first").accepted());
            if dead { s.command(Command::Kill { entity:prey }).unwrap(); }
            let before = s.sim.state_hash();
            let refused = s.command_with_flows(Command::Act { actor:hunter, target:Some(prey),
                process:"test:meal".into(), cause:None }).unwrap();
            let receipt: Receipt = serde_json::from_str(&refused.result).unwrap();
            assert!(!receipt.accepted(), "off-site prey cannot be taken by global meal");
            assert!(refused.flows.is_empty());
            assert_eq!(s.sim.state_hash(), before);
            s.advance(3).unwrap();
            assert!(act(&mut s, hunter, Some(prey), "test:meal").accepted());
            replay(&s);
        }
    }
}

#[test]
fn framed_departure_preserves_old_body_and_candidate_scope() {
    let (g, actor) = fixture(1);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    s.sim.targets = Some(crate::targets::Targets::new(&s.sim.state.population));
    s.sim.frame = Some(crate::schedule::Frame::default());
    assert!(s.sim.apply(actor, None, "test:first", None, 1).accepted());
    assert_eq!(s.sim.state().population.get(actor).unwrap().resident_site(), None);
    assert_eq!(s.sim.body_at_start(actor).unwrap().resident_site(), Some(0),
        "454 departure cannot change another actor's pass-start reading");
    assert_eq!(s.sim.body_at_start(actor).unwrap().journey, None);
    let target = Target { same_place:true, alive:None, lineage:None,
        among:Default::default(), weighted:false };
    let old = s.sim.frame.as_ref().unwrap().targets.lists(&target,0).flatten().copied().collect::<Vec<_>>();
    assert_eq!(old, [actor], "old residence remains in the lazy scope overlay");
    assert!(s.sim.targets.as_ref().unwrap().lists(&target,0).flatten().next().is_none());
}

#[test]
fn a_selected_journey_process_stays_committed_with_its_explicit_nudge() {
    let (mut g,id) = fixture(1);
    g.population.lift(id).unwrap().method = Method::Deliberative;
    g.rules.traits.insert("test:fallback".into());
    g.rules.processes.get_mut("test:first").unwrap().period = Some(1);
    g.rules.processes.get_mut("test:first").unwrap().priority = -20;
    add(&mut g,"test:mark",Causation::Choice,vec![Effect::Trait {
        who:Binding::Actor,key:"test:fallback".into(),present:true,
    }],Some(1),0);
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let joined = s.command(Command::Join).unwrap();
        let participant = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        s.command(Command::Take { participant,critter:id }).unwrap();
        s.command(Command::Nudge { participant,critter:id,aim:crate::directing::Aim::Act,
            toward:crate::directing::Toward::Site(1),act:Some("test:first".into()) }).unwrap();
        s.sim.watch("test:first"); s.sim.watch("test:mark");
        s.advance(1).unwrap();
        let acts = s.sim.take_watched();
        assert_eq!(acts.len(),1,"813 Some remains selected across later passes");
        assert_eq!((acts[0].actor,acts[0].process.as_str()),(id,"test:first"));
        assert_eq!(s.sim.state().population.get(id).unwrap().resident_site(),None);
        assert!(!s.sim.state().population.get(id).unwrap().traits.contains("test:fallback"));
        assert_eq!(s.sim.state().nudges[0].act.as_deref(),Some("test:first"));
        // The chosen process, not a fallback, is what can serve the bound nudge.
        let answer = s.sim.state().nudges[0].answer.as_ref().unwrap();
        assert_eq!(answer.site,crate::directing::PLACELESS,
            "a departure answer cannot claim the historical origin as residence");
        assert_eq!(answer.resident_site(),None);
        let orders = crate::directing::readings::orders(&s.sim,participant,id);
        assert!(!orders.range.contains_key(&0),"orders cannot invent thriving at origin");
        assert!(!orders.range.contains_key(&crate::directing::PLACELESS));
        replay(&s);
    }
}

#[test]
fn an_unavailable_nudged_prey_does_not_credit_an_autonomous_meal() {
    let (mut g,prey) = fixture(1);
    let hunter = neighbour(&mut g,prey,0);
    let spare = neighbour(&mut g,prey,0);
    g.population.lift(hunter).unwrap().method = Method::Deliberative;
    g.population.lift(hunter).unwrap().traits.insert("test:hunter".into());
    meal(&mut g,Some(1),true);
    g.rules.processes.get_mut("test:meal").unwrap().causation = Causation::Choice;
    for mode in [Execution::Individuals,Execution::Grouped] {
        let mut s = Session::new(g.clone(),mode).unwrap();
        let joined = s.command(Command::Join).unwrap();
        let participant = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        s.command(Command::Take { participant,critter:hunter }).unwrap();
        s.command(Command::Nudge { participant,critter:hunter,aim:crate::directing::Aim::Act,
            toward:crate::directing::Toward::Thing(prey),act:Some("test:meal".into()) }).unwrap();
        assert!(act(&mut s,prey,None,"test:first").accepted());
        let bond = s.sim.bond(participant,hunter);
        s.sim.watch("test:meal");
        s.advance(1).unwrap();
        // The prey left before this tick's choice. The nudge cannot bind an
        // infeasible target, and does not prohibit the critter's own meal.
        let meals = s.sim.take_watched();
        assert_eq!(meals.len(),1,"the critter chooses its available autonomous meal");
        assert_eq!((meals[0].actor,meals[0].target),(hunter,Some(spare)));
        assert_eq!(s.sim.state().population.get(spare).unwrap().accounts[OWN],31);
        assert!(s.sim.state().nudges[0].answer.is_none(),"another prey cannot answer the named Thing");
        assert_eq!(s.sim.bond(participant,hunter),bond,"no credit from a different prey");
        replay(&s);
    }
}

#[test]
fn journey_events_have_no_site_reach_and_a_later_native_telling_is_explicit() {
    let (mut g, id) = fixture(1);
    let listener = neighbour(&mut g, id, 1);
    add(&mut g, "test:memory", Causation::Transition, vec![], None, 0);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    assert!(act(&mut s, id, None, "test:first").accepted());
    let remembered = act(&mut s, id, None, "test:memory");
    assert!(remembered.accepted());
    let event = s.sim.state().events[&remembered.id].clone();
    assert!(matches!(event.place, EventPlace::Journey(_)), "822 native typed journey location");
    assert!(!s.sim.state().reach.arrivals.contains_key(&event.id),
        "822 journey event has no site seed");
    for legend in [false,true] {
        let mut record = event.clone(); record.legend = legend;
        assert_eq!(s.sim.state().reach.strength(&record,1,3,&s.sim.genesis().rules.field),0);
    }
    assert!(s.sim.knows(id,&event.id).unwrap(), "actor remembers actual transit act");
    assert!(!s.sim.knows(listener,&event.id).unwrap());
    // Add the telling definition to a fresh Genesis BEFORE replaying the same actions.
    let mut g = s.sim.genesis().clone();
    add(&mut g, "test:tell", Causation::Choice,
        vec![Effect::Tell { event:event.id.clone() }], None, 0);
    g.rules.processes.get_mut("test:tell").unwrap().target = Some(Target {
        same_place:true, alive:Some(true), lineage:None, among:Default::default(), weighted:false,
    });
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    assert!(act(&mut s,id,None,"test:first").accepted());
    let remembered = act(&mut s,id,None,"test:memory");
    assert_eq!(remembered.id,event.id);
    s.advance(3).unwrap();
    assert!(!s.sim.state().reach.arrivals.contains_key(&event.id), "arrival cannot seed old transit event");
    let outcome = s.command(Command::Act { actor:id, target:Some(listener),
        process:"test:tell".into(), cause:Some(event.id.clone()) }).unwrap();
    let told: Receipt = serde_json::from_str(&outcome).unwrap();
    assert!(told.accepted());
    assert!(s.sim.knows(listener,&event.id).unwrap());
    let telling = &s.sim.state().events[&told.id];
    assert_eq!(telling.place, EventPlace::Site(1));
    assert_eq!(telling.cause.as_deref(),Some(event.id.as_str()));
    assert!(s.sim.state().reach.arrivals.contains_key(&telling.id));
    assert!(!s.sim.state().reach.arrivals.contains_key(&event.id));
    replay(&s);
}

#[test]
fn typed_event_locations_roundtrip_json_and_binary_with_kept_site_preimages() {
    let (g, actor) = fixture(1);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let site = act(&mut s,actor,None,"test:first").id;
    let journey = Journey { from:0,to:1,departed:0,arrives:3 };
    for place in [EventPlace::Site(0), EventPlace::Site(crate::directing::PLACELESS),
                  EventPlace::Journey(journey)] {
        let wire = postcard::to_allocvec(&place).unwrap();
        assert_eq!(postcard::from_bytes::<EventPlace>(&wire).unwrap(),place);
        let json = serde_json::to_vec(&place).unwrap();
        assert_eq!(serde_json::from_slice::<EventPlace>(&json).unwrap(),place);
        let mut event = s.sim.state().events[&site].clone(); event.place = place;
        for populated in [false,true] {
            if populated {
                event.object=Some(actor+1); event.cause=Some("event:prior".into());
                event.agreement=Some(17); event.authored=Some(crate::asserted::HistoryLine::default());
            }
            let binary = postcard::to_allocvec(&event).unwrap();
            assert_eq!(postcard::from_bytes::<Event>(&binary).unwrap(),event);
            assert_eq!(serde_json::from_slice::<Event>(&serde_json::to_vec(&event).unwrap()).unwrap(),event);
            let mut wrong = binary; assert_eq!(wrong[0],4); wrong[0]=3;
            assert!(postcard::from_bytes::<Event>(&wrong).is_err(),"event wire rejects the old version tag");
        }
        if let EventPlace::Site(id) = place {
            assert_eq!(json,id.to_string().as_bytes());
            let preimage = crate::simulation::preimage::event::Event::from(&event);
            assert_eq!(serde_json::to_vec(&preimage).unwrap(),serde_json::to_vec(&event).unwrap());
        }
    }
    // The existing placeless numeric identity remains a Site value on old JSON.
}

#[test]
fn aggregate_reactive_routes_are_supported_while_identity_choice_remains_native() {
    let (mut g,id) = fixture(1);
    let p = g.rules.processes.get_mut("test:first").unwrap();
    p.period=Some(1); p.causation=Causation::Transition; p.note=false;
    g.rules.processes.remove("test:next");
    let mut world = crate::probe::ProbeWorld { genesis:g, ticks:4 };
    let exact = crate::probe::run_exact(&world,7,true).unwrap();
    let crowd = crate::probe::Crowd::new(&world,7,crate::probe::Variant::Histogram).unwrap().run().unwrap();
    assert_eq!(exact.work.arrivals,1);
    assert_eq!(crowd.work.arrivals,1,"reactive migration is actual aggregate support");
    assert_eq!(exact.sim.state().population.get(id).unwrap().resident_site(),Some(1));
    assert!(crowd.bins.keys().all(|e| e.resident_site()==Some(1)));
    world.genesis.population.groups.get_mut(&id).unwrap().entity_mut().method=Method::Deliberative;
    assert_eq!(crate::probe::Crowd::new(&world,7,crate::probe::Variant::Histogram).err().unwrap(),
        "moving aggregate worlds require the native deliberative chooser");
    world.genesis.rules.processes.clear();
    assert!(world.genesis.sites.values().any(|s| !s.routes.is_empty()));
    assert!(crate::probe::Crowd::new(&world,7,crate::probe::Variant::Histogram).is_ok());
}

#[test]
fn bodily_wounds_and_rot_cannot_spill_a_traveller_at_the_actor_site() {
    let (mut g,prey) = bodied_fixture();
    let hunter = neighbour(&mut g,prey,1);
    for (name,effect) in [("test:wound",Effect::Wound { who:Binding::Target,
        cells:Amount::Fixed(1),slot:0 }), ("test:rot",Effect::Rot { who:Binding::Target,
        amount:Amount::Fixed(1) })] {
        let mut p = crate::generate::process(name,Causation::Transition,vec![effect]);
        p.target = Some(Target { same_place:false,alive:None,lineage:None,
            among:Default::default(),weighted:false });
        g.rules.processes.insert(name.into(),p);
    }
    for mode in [Execution::Individuals,Execution::Grouped] {
        for dead in [false,true] {
            let mut s = Session::new(g.clone(),mode).unwrap();
            let matter = s.sim.matter();
            assert!(act(&mut s,prey,None,"test:first").accepted());
            if dead { s.command(Command::Kill { entity:prey }).unwrap(); }
            let process = if dead { "test:rot" } else { "test:wound" };
            let before = s.sim.state_hash();
            let refused = s.command_with_flows(Command::Act { actor:hunter,target:Some(prey),
                process:process.into(),cause:None }).unwrap();
            let receipt: Receipt = serde_json::from_str(&refused.result).unwrap();
            assert!(!receipt.accepted(),"off-site harm cannot use the resident actor's site");
            assert!(refused.flows.is_empty());
            assert_eq!(s.sim.state_hash(),before);
            if !dead {
                assert!(s.command(Command::Wound { entity:prey,part:None,cells:1 }).is_err());
                assert_eq!(s.sim.state_hash(),before,"direct wound cannot spill at historical origin");
            }
            s.advance(3).unwrap();
            assert!(act(&mut s,hunter,Some(prey),process).accepted(),"same harm can land after arrival");
            assert_eq!(s.sim.matter(),matter);
            replay(&s);
        }
    }
}

#[test]
#[ignore = "requires immutable genuine v3 archive and SHA256 supplied by root"]
fn authentic_v3_movement_archive_is_authenticated_before_kept_replay() {
    use sha2::{Digest,Sha256};
    let path = std::env::var("ISOCOSM_JOURNEY_V3_ARCHIVE").unwrap();
    let expected = std::env::var("ISOCOSM_JOURNEY_V3_ARCHIVE_SHA256").unwrap();
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(format!("{:x}",Sha256::digest(&bytes)),expected);
    let saved: crate::history::Saved = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved.version,3);
    assert_eq!(saved.movement,None);
    assert_eq!(crate::digest(&saved.genesis),saved.genesis_digest);
    for mode in [Execution::Individuals,Execution::Grouped] {
        let restored = Session::load(saved.clone(),mode).unwrap();
        assert_eq!(restored.sim.state().movement,Movement::ImmediateV3);
        assert_eq!(restored.sim.state_hash(),saved.state_hash);
        let again = restored.save();
        assert_eq!(again.version,4);
        assert_eq!(again.movement,Some(Movement::ImmediateV3));
        assert_eq!(again.genesis_digest,saved.genesis_digest);
        assert_eq!(again.entries,saved.entries);
        assert_eq!(again.checkpoints,saved.checkpoints);
        replay(&restored);
    }
}
