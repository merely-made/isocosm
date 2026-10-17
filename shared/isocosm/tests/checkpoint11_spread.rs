// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! C11's native site-grain contract, including the ruled capacity/route path.
//! Values here are explicit test-world inputs, not production defaults.

use isocosm::{Command, Execution, Founding, Session, rules::{AccountKind, Binding, Causation, Effect, Expr, Process, Query, Reading},
    schema::{Id, Ledger, Method, Route}, simulation::{Genesis, Receipt}, spread::{self, CapacityLayer,
        CapacityModifier, CapacityRules, CapacitySource, Identity, PlaceLedger, Placement,
        Stacking, Substrate, Supply}};
use isometer_space::places::PlaceId;
use std::collections::{BTreeMap, BTreeSet};

const SUPPLY: &str = "c11:supply";
const GENE: &str = "c11:capacity";
const BASE: &str = "capacity:base";
const LAYER: &str = "capacity:modifier";
const INPUT: &str = "capacity:value";

struct World {
    genesis: Genesis,
    host: Id,
    from: Id,
    to: [Id; 3],
    territory: Identity,
    surface: Identity,
    place: PlaceId,
}

fn kept(key: &str) -> Expr { Expr::Read(Reading::Kept { name: key.into() }) }

fn world(seed: u64) -> World {
    let mut genesis = Founding { seed, sites: 4, population: 24, lineages: 1,
        cohort_size: 8, ..Default::default() }.generate().unwrap();
    genesis.rules.processes.clear();
    genesis.rules.epoch_ticks = 2;
    genesis.rules.shapes = isocosm::rules::default_shapes();
    genesis.rules.functions = isocosm::rules::default_functions();
    for cohort in genesis.population.groups.values_mut() {
        assert!(cohort.entity.parts.values().all(|part| part.matter.is_empty() && part.cells.is_empty()));
        cohort.entity_mut().parts.clear();
    }
    let (&host, cohort) = genesis.population.groups.iter()
        .find(|(_, group)| group.entity.kingdom != "kingdom:world").unwrap();
    assert!(cohort.count > 1, "this world must exercise actual crowd storage");
    let from = cohort.entity.resident_site().unwrap();
    let lineage = cohort.entity.lineage.clone();
    let to: Vec<_> = genesis.sites.keys().copied().filter(|id| *id != from).collect();
    let to = [to[0], to[1], to[2]];
    genesis.rules.traits.extend([GENE.into(), "function:fix".into()]);
    genesis.lineages.get_mut(&lineage).unwrap().traits.extend([GENE.into(), "function:fix".into()]);
    genesis.rules.accounts.insert(SUPPLY.into(), AccountKind::Matter {
        lineage: lineage.clone(), reserve: false, provision: false });
    let effect = genesis.world.canon.glyphs.first().unwrap().effect.clone();
    genesis.rules.spread_capacity = Some(CapacityRules {
        key: BASE.into(), trait_inputs: BTreeMap::from([("capacity:gene".into(), GENE.into())]),
        mechanic: Expr::Mul(vec![kept("capacity:gene"), Expr::Const(20)]),
        layers: vec![CapacityLayer { key: LAYER.into(), effect, stacking: Stacking::Sum,
            input: INPUT.into(), reading: Expr::Add(vec![kept(BASE), kept(INPUT)]) }],
    });
    for site in genesis.sites.values_mut() { site.routes.clear(); }
    genesis.sites.get_mut(&from).unwrap().routes = vec![
        Route { to: to[0], travel: 2, transmission: 500_000, border: None, authored: None },
        Route { to: to[1], travel: 3, transmission: 1_000_000, border: None, authored: None },
        Route { to: to[2], travel: 1, transmission: 0, border: None, authored: None },
    ];
    genesis.sites.get_mut(&from).unwrap().accounts.insert(SUPPLY.into(), 500);
    let territory = Identity::Territory(host);
    let surface = Identity::Surface(lineage);
    genesis.spreads.identities = BTreeSet::from([territory.clone(), surface.clone()]);
    let place = PlaceId { site: from, cell: [1, -2, 3] };
    genesis.spreads.places.push(PlaceLedger { place, accounts: Ledger::new() });
    genesis.validate().unwrap();
    World { genesis, host, from, to, territory, surface, place }
}

fn place(session: &mut Session, world: &World, identity: &Identity, at: Substrate, amount: u64) {
    session.command(Command::Spread(spread::Command::Place(Placement {
        source: Supply::Matter { from: Substrate::Site(world.from), account: SUPPLY.into() },
        identity: identity.clone(), at, amount,
    }))).unwrap();
}

fn modifier(key: &str, source: CapacitySource, identity: Option<Identity>, milligrams: u64)
    -> CapacityModifier {
    CapacityModifier { key: key.into(), source, identity, layer: LAYER.into(), milligrams }
}

fn direction_world(method: Method) -> World {
    let mut w = world(17);
    w.genesis.rules.traits.insert("c11:director".into());
    let body = w.genesis.population.lift(w.host).unwrap();
    body.method = method;
    body.traits.insert("c11:director".into());
    w.genesis.sites.get_mut(&w.from).unwrap().accounts.insert(w.territory.key(), 61);
    for (name, destination) in [("c11:route-a", w.to[0]), ("c11:route-b", w.to[1])] {
        let process = Process { id: name.into(), causation: Causation::Choice,
            requires: vec![Query::Alive(Binding::Actor), Query::Trait { who: Binding::Actor, key: "c11:director".into() }],
            commitments: vec![], effects: vec![Effect::DirectOverflow { from: w.from,
                destinations: BTreeSet::from([destination]) }], risk: None, target: None,
            period: Some(1), priority: 0, need_account: None, need_below: 0,
            glyphs: BTreeSet::new(), invariants: BTreeSet::from(["sim:matter".into(), "sim:nonnegative".into()]),
            note: true, port_feeding: None, goal_directives: Vec::new() };
        w.genesis.rules.processes.insert(name.into(), process);
    }
    w.genesis.validate().unwrap();
    w
}

fn join_take(session: &mut Session, body: Id) -> Id {
    let receipt = session.command(Command::Join).unwrap();
    let participant = receipt.strip_prefix("participant:").unwrap().parse().unwrap();
    session.command(Command::Take { participant, critter: body }).unwrap();
    participant
}

fn exact(session: &Session, mass: u128) {
    assert_eq!(session.sim.matter(), mass, "include every site, host, fine place and in-flight mg");
    let json = serde_json::to_vec(&session.save()).unwrap();
    let state_json = serde_json::to_vec(session.sim.state()).unwrap();
    let decoded: isocosm::simulation::State = serde_json::from_slice(&state_json).unwrap();
    assert_eq!(decoded, *session.sim.state());
    for execution in [Execution::Individuals, Execution::Grouped] {
        let restored = Session::load_json(&json, execution).unwrap();
        assert_eq!(restored.sim.state_hash(), session.sim.state_hash());
        assert_eq!(restored.sim.entity_witness(), session.sim.entity_witness());
        assert_eq!(restored.entries, session.entries);
        assert_eq!(restored.checkpoints, session.checkpoints);
        assert_eq!(restored.sim.matter(), mass);
    }
}

#[test]
fn declared_capacity_and_modifiers_read_local_lineage_without_parts() {
    let w = world(17);
    let mut s = Session::new(w.genesis.clone(), Execution::Grouped).unwrap();
    let mass = s.sim.matter();
    let source = Substrate::Site(w.from);
    assert_eq!(spread::capacity(s.sim.state(), &w.genesis.rules, source, &w.territory).unwrap(), 20);
    for value in [modifier("modifier:world", CapacitySource::World, None, 3),
        modifier("modifier:local", CapacitySource::Substrate(source), Some(w.territory.clone()), 7)] {
        s.command(Command::Spread(spread::Command::SetModifier(value))).unwrap();
    }
    assert_eq!(spread::capacity(s.sim.state(), &w.genesis.rules, source, &w.territory).unwrap(), 30);
    assert_eq!(spread::capacity(s.sim.state(), &w.genesis.rules, source, &w.surface).unwrap(), 23);
    assert_eq!(spread::capacity(s.sim.state(), &w.genesis.rules,
        Substrate::Place(w.place), &w.territory).unwrap(), 23);
    place(&mut s, &w, &w.territory, source, 10);
    assert!(s.sim.state().population.get(w.host).unwrap().parts.is_empty());
    assert_eq!(spread::functions(s.sim.state(), &w.genesis.rules, source, &w.territory)
        .unwrap()["function:fix"], 10);
    exact(&s, mass);
}

#[test]
fn automatic_split_excludes_zero_routes_and_keeps_division_and_transmission_remainders() {
    let w = world(17);
    let mut final_states = Vec::new();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(w.genesis.clone(), execution).unwrap();
        let mass = s.sim.matter();
        place(&mut s, &w, &w.territory, Substrate::Site(w.from), 101);
        assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.from), &w.territory).unwrap(), 41);
        let queue = &s.sim.state().spreads.transit;
        assert_eq!(queue.len(), 2);
        assert_eq!((queue[0].to, queue[0].amount, queue[0].arrives), (w.to[0], 20, 2));
        assert_eq!((queue[1].to, queue[1].amount, queue[1].arrives), (w.to[1], 40, 3));
        exact(&s, mass);
        for tick in 1..=3 {
            s.advance(1).unwrap();
            assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.to[0]), &w.territory).unwrap(),
                if tick >= 2 { 20 } else { 0 });
            assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.to[1]), &w.territory).unwrap(),
                if tick >= 3 { 40 } else { 0 });
            assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.to[2]), &w.territory).unwrap(), 0);
            exact(&s, mass);
        }
        final_states.push((s.sim.state_hash(), s.sim.entity_witness()));
    }
    assert_eq!(final_states[0], final_states[1]);
}

#[test]
fn site_host_and_fine_place_debit_only_the_actual_substrate_in_both_engines() {
    let w = world(23);
    for (identity, at) in [(&w.territory, Substrate::Site(w.from)),
        (&w.surface, Substrate::Site(w.from)), (&w.surface, Substrate::Host(w.host)),
        (&w.surface, Substrate::Place(w.place))] {
        let mut witnesses = Vec::new();
        for execution in [Execution::Individuals, Execution::Grouped] {
            let mut s = Session::new(w.genesis.clone(), execution).unwrap();
            let mass = s.sim.matter();
            place(&mut s, &w, identity, at, 61);
            assert_eq!(spread::held(s.sim.state(), at, identity).unwrap(), 31);
            assert_eq!(s.sim.state().spreads.transit.iter().map(|t| t.amount).sum::<u64>(), 30);
            if at == Substrate::Host(w.host) {
                assert_eq!(spread::held(s.sim.state(), Substrate::Host(w.host + 1), identity).unwrap(), 0,
                    "one lifted cohort member must not credit its neighbours");
            }
            exact(&s, mass);
            s.advance(4).unwrap();
            exact(&s, mass);
            witnesses.push((s.sim.state_hash(), s.sim.entity_witness()));
        }
        assert_eq!(witnesses[0], witnesses[1]);
    }
}

#[test]
fn capacity_reduction_spills_once_and_clock_partitioning_does_not_resend_remainders() {
    let w = world(17);
    let mut s = Session::new(w.genesis.clone(), Execution::Grouped).unwrap();
    let mass = s.sim.matter();
    place(&mut s, &w, &w.surface, Substrate::Site(w.from), 101);
    s.command(Command::Spread(spread::Command::SetModifier(
        modifier("modifier:raise", CapacitySource::World, None, 10)))).unwrap();
    assert_eq!(s.sim.state().spreads.transit.len(), 2);
    s.command(Command::Spread(spread::Command::RemoveModifier { key: "modifier:raise".into() })).unwrap();
    assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.from), &w.surface).unwrap(), 26);
    assert_eq!(s.sim.state().spreads.transit.iter().map(|t| t.amount).sum::<u64>(), 75);
    exact(&s, mass);
    let mut whole = s.clone();
    whole.advance(6).unwrap();
    for _ in 0..6 { s.advance(1).unwrap(); }
    assert!(s.sim.state().spreads.transit.is_empty());
    assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.from), &w.surface).unwrap(), 26);
    assert_eq!(whole.sim.state_hash(), s.sim.state_hash());
    assert_eq!(whole.checkpoints, s.checkpoints);
    exact(&s, mass);
}

#[test]
fn arrival_overflow_starts_its_next_authored_travel_after_actual_delivery() {
    let mut w = world(17);
    w.genesis.sites.get_mut(&w.from).unwrap().routes[0].transmission = 1_000_000;
    w.genesis.sites.get_mut(&w.to[0]).unwrap().routes.push(Route {
        to: w.to[2], travel: 2, transmission: 1_000_000, border: None, authored: None });
    let mut results = Vec::new();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(w.genesis.clone(), execution).unwrap();
        let mass = s.sim.matter();
        place(&mut s, &w, &w.surface, Substrate::Site(w.from), 101);
        s.advance(2).unwrap();
        assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.to[0]), &w.surface).unwrap(), 20);
        let onward = s.sim.state().spreads.transit.iter().find(|t| t.from == w.to[0]).unwrap();
        assert_eq!((onward.departed, onward.arrives, onward.amount), (2, 4, 20));
        exact(&s, mass);
        s.advance(1).unwrap();
        assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.to[2]), &w.surface).unwrap(), 0);
        s.advance(1).unwrap();
        assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.to[2]), &w.surface).unwrap(), 20);
        exact(&s, mass);
        results.push((s.sim.state_hash(), s.sim.entity_witness()));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn strains_retain_selected_local_functions_and_deterministic_seed_identity() {
    let mut seeded = BTreeSet::new();
    for seed in [17, 23, 41] {
        let w = world(seed);
        let parent = w.surface.key();
        let traits = BTreeSet::from([GENE.into(), "function:fix".into()]);
        let expected = spread::strain_key(w.genesis.dynamics_seed(), &parent, Substrate::Host(w.host), &traits);
        let mut results = Vec::new();
        for execution in [Execution::Individuals, Execution::Grouped] {
            let mut s = Session::new(w.genesis.clone(), execution).unwrap();
            let mass = s.sim.matter();
            place(&mut s, &w, &w.surface, Substrate::Host(w.host), 61);
            let child = s.command(Command::Spread(spread::Command::ForkStrain {
                parent: parent.clone(), at: Substrate::Host(w.host), amount: 7, traits: traits.clone(),
            })).unwrap();
            assert_eq!(child, expected);
            let identity = Identity::Surface(child.clone());
            assert_eq!(spread::held(s.sim.state(), Substrate::Host(w.host), &w.surface).unwrap(), 24);
            assert_eq!(spread::held(s.sim.state(), Substrate::Host(w.host), &identity).unwrap(), 7);
            assert_eq!(s.sim.state().lineages[&child].parent.as_ref(), Some(&parent));
            assert_eq!(spread::functions(s.sim.state(), &w.genesis.rules, Substrate::Host(w.host), &identity)
                .unwrap()["function:fix"], 7);
            assert_ne!(child, spread::strain_key(w.genesis.dynamics_seed(), &parent,
                Substrate::Place(w.place), &traits));
            s.advance(4).unwrap();
            exact(&s, mass);
            results.push((s.sim.state_hash(), s.sim.entity_witness()));
        }
        assert_eq!(results[0], results[1]);
        assert!(seeded.insert(expected));
    }
}

#[test]
fn refused_modifier_and_nonlocal_stock_placement_preserve_entire_saved_native_state() {
    let w = world(17);
    for execution in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(w.genesis.clone(), execution).unwrap();
        let before = s.save();
        let mut invalid = modifier("modifier:invalid", CapacitySource::World, None, 1);
        invalid.layer = "capacity:absent".into();
        assert!(s.command(Command::Spread(spread::Command::SetModifier(invalid))).is_err());
        assert_eq!(s.save(), before);
        assert!(s.command(Command::Spread(spread::Command::Place(Placement {
            source: Supply::Matter { from: Substrate::Site(w.from), account: SUPPLY.into() },
            identity: w.surface.clone(), at: Substrate::Site(w.to[0]), amount: 1,
        }))).is_err());
        assert_eq!(s.save(), before);
        exact(&s, s.sim.matter());
    }
}

#[test]
fn a_genuine_nudge_changes_the_native_chosen_route_without_moving_the_territory_body() {
    use isocosm::directing::{Aim, Toward};
    let w = direction_world(Method::Deliberative);
    let mut results = Vec::new();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(w.genesis.clone(), execution).unwrap();
        let mass = s.sim.matter();
        let participant = join_take(&mut s, w.host);
        s.command(Command::Nudge { participant, critter: w.host, aim: Aim::Act,
            toward: Toward::Site(w.to[1]), act: Some("c11:route-b".into()) }).unwrap();
        s.advance(1).unwrap();
        let event = s.sim.state().events.values().find(|event| event.subject == w.host).unwrap();
        assert_eq!(event.process, "c11:route-b");
        assert_eq!(s.sim.state().spreads.transit.len(), 1);
        let transfer = &s.sim.state().spreads.transit[0];
        assert_eq!((transfer.from, transfer.to, transfer.amount), (w.from, w.to[1], 41));
        assert_eq!((transfer.departed, transfer.arrives), (event.tick, event.tick + 3));
        let body = s.sim.state().population.get(w.host).unwrap();
        assert_eq!(body.resident_site(), Some(w.from));
        assert!(body.journey.is_none());
        assert_eq!(spread::held(s.sim.state(), Substrate::Site(w.from), &w.territory).unwrap(), 20);
        exact(&s, mass);
        results.push((s.sim.state_hash(), s.sim.entity_witness()));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn direct_author_input_cannot_force_a_deliberative_territory_or_duplicate_its_effect() {
    let w = direction_world(Method::Deliberative);
    let mut s = Session::new(w.genesis.clone(), Execution::Grouped).unwrap();
    let hash = s.sim.state_hash();
    let outcome = s.command(Command::Act { actor: w.host, target: None,
        process: "c11:route-b".into(), cause: None }).unwrap();
    let receipt: Receipt = serde_json::from_str(&outcome).unwrap();
    assert!(!receipt.accepted());
    assert_eq!(s.sim.state_hash(), hash);
    assert!(s.sim.state().spreads.transit.is_empty());
    let mut invalid = w.genesis.clone();
    let process = invalid.rules.processes.get_mut("c11:route-b").unwrap();
    process.effects.push(process.effects[0].clone());
    assert!(invalid.validate().is_err(), "one source must not spill twice in one native act");
}

#[cfg(feature = "society")]
#[test]
fn normative_direction_requires_exact_owned_quantities_and_retains_claims_through_delivery() {
    use isocosm::{asserted::Assertion, projects::{DebitSelection, Disposition, DispositionRule,
        LotQuantity, OperationRule, OwnedActed, Owner, RetainedOutput, SpreadDebit,
        SpreadOutput, StockLot, StockRules, StockSource}};
    let mut w = direction_world(Method::Normative);
    w.genesis.rules.owned_stock = Some(StockRules { selections: Vec::new(),
        dispositions: vec![DispositionRule { operation: OperationRule::Spread {
            operation: SpreadDebit::Overflow, identity: w.territory.clone(), at: Substrate::Site(w.from) },
            disposition: Disposition::Outputs { outputs: vec![RetainedOutput {
                input: "lot:territory".into(), amount: 41, output: SpreadOutput::Route(1) }], closed: Vec::new() } }] });
    for execution in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(w.genesis.clone(), execution).unwrap();
        let mass = s.sim.matter();
        let participant = join_take(&mut s, w.host);
        s.command(Command::Assert(Assertion::SpreadOwnership { identity: w.territory.clone(),
            lot: StockLot { key: "lot:territory".into(), source: StockSource::Site(w.from),
                account: w.territory.key(), amount: 41, owners: BTreeSet::from([Owner::Entity(w.host)]),
                collectives: BTreeMap::new() } })).unwrap();
        let preview = s.sim.prepare_owned_act(w.host, None, "c11:route-b", None).unwrap();
        assert!(preview.refusal.is_none(), "{:?}", preview.refusal);
        let [request] = preview.requests.as_slice() else { panic!("one actual full-quantity territory debit is required"); };
        assert_eq!(request.amount, 41);
        let before = s.sim.state_hash();
        let invoke = |selection| Command::ActOwned { participant, actor: w.host, target: None,
            process: "c11:route-b".into(), cause: None, selections: vec![selection] };
        let bad = s.command(invoke(DebitSelection { request: request.clone(), lots: Vec::new(), unowned: 41 })).unwrap();
        let bad: OwnedActed = serde_json::from_str(&bad).unwrap();
        assert!(!bad.receipt.accepted());
        assert_eq!(s.sim.state_hash(), before);
        assert!(s.sim.state().spreads.transit.is_empty());
        // Refused entry identity is not a cost revision; quote the actual current state again.
        let preview = s.sim.prepare_owned_act(w.host, None, "c11:route-b", None).unwrap();
        let request = preview.requests[0].clone();
        let good = s.command(invoke(DebitSelection { request, lots: vec![LotQuantity {
            key: "lot:territory".into(), amount: 41 }], unowned: 0 })).unwrap();
        let good: OwnedActed = serde_json::from_str(&good).unwrap();
        assert!(good.receipt.accepted(), "{:?}", good.receipt.outcome);
        assert_eq!(s.sim.state().tick, 0, "admitting an act adds no hidden clock");
        assert_eq!(s.sim.state().spreads.transit_marks.len(), 1);
        let transit = s.sim.state().spreads.transit_marks[0].id;
        assert!(s.sim.available_lots().any(|(source, lot)| source == StockSource::Transit(transit)
            && lot.amount == 41 && lot.owners == BTreeSet::from([Owner::Entity(w.host)])));
        exact(&s, mass);
        s.advance(3).unwrap();
        assert!(s.sim.available_lots().any(|(source, lot)| source == StockSource::Site(w.to[1])
            && lot.amount == 41 && lot.owners == BTreeSet::from([Owner::Entity(w.host)])));
        exact(&s, mass);
    }
}
