// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{ecology::*, rules::*, schema::*, simulation::Genesis, Execution, Founding};
use std::collections::{BTreeMap, BTreeSet};

pub struct Candidate {
    pub basis: GeneratedBasis,
    pub genesis: Genesis,
    pub spec: JointSpec,
    pub process: Key,
}

fn address(process: &str) -> EffectAddress {
    EffectAddress { process: process.into(), phase: EffectPhase::Outcome,
        path: vec![EffectStep::Item(0)] }
}

fn declaration(process: &Process, name: &str, source: ScalarSource,
    dimension: Dimension, conservation: Conservation) -> ScalarDeclaration {
    ScalarDeclaration { name: format!("test:{name}"), source, dimension,
        consumer: process.id.clone(), cadence: process.period.unwrap(), conservation }
}

// One authored candidate, not a claim that the ordinary generator supplies capture.
// Two actual residents supply conserved soil and a real same-lineage counter target.
pub fn candidate(process_name: &str, extra_quantity: u64, account_alias: bool) -> Candidate {
    build(process_name, extra_quantity, account_alias, false)
}

pub fn candidate_with_played(process_name: &str, extra_quantity: u64) -> Candidate {
    build(process_name, extra_quantity, false, true)
}

fn build(process_name: &str, extra_quantity: u64, account_alias: bool, played: bool) -> Candidate {
    let founding = Founding { seed: 19, sites: 1, population: 2, lineages: 1,
        cohort_size: 1, stock_min: 12, stock_max: 12, period_max: 1,
        played: played.then_some(isocosm::directing::Played { lineage: 0, region_sites: 1 }),
        ..Default::default() };
    let basis = GeneratedBasis::new(&founding).unwrap();
    let mut genesis = founding.generate().unwrap();
    let original_id = if played {
        // Native choice feeds its actual authored need, without a foreground Act.
        genesis.rules.processes.keys().filter(|key| key.starts_with("process:cycle-0-")).next_back().unwrap().clone()
    } else { "process:cycle-0-0".into() };
    let original = genesis.rules.processes[&original_id].clone();
    let mut process = original.clone();
    process.id = process_name.into();
    process.priority = if played { 100 } else { -100 };
    process.target = Some(Target { same_place: true, alive: Some(true),
        lineage: Some("lineage:0".into()), among: BTreeSet::new(), weighted: false });
    let Effect::Transform { take, give, .. } = &original.effects[0] else { panic!("native cycle changed"); };
    let source = take.keys().next().unwrap().clone();
    let destination = give.keys().next().unwrap().clone();
    let input = if account_alias { "test:renamed-stock".into() } else { source.clone() };
    let amount = take[&source] + extra_quantity;
    if account_alias {
        genesis.rules.accounts.insert(input.clone(), genesis.rules.accounts[&source].clone());
        for group in genesis.population.groups.values_mut().filter(|g| g.entity.lineage == "lineage:0") {
            let quantity = group.entity.accounts.remove(&source).unwrap();
            group.entity.accounts.insert(input.clone(), quantity);
        }
        let Query::Account { key, .. } = &mut process.requires[2] else { panic!("native gate changed"); };
        *key = input.clone();
    }
    process.effects = vec![
        Effect::Transform { who: Binding::Actor, take: BTreeMap::from([(input.clone(), amount)]),
            give: BTreeMap::from([(destination.clone(), amount)]), conversion: None },
        Effect::Transfer { from: Binding::Place, to: Binding::Actor,
            account: "sim:energy".into(), amount: 1.into() },
        Effect::Transfer { from: Binding::Actor, to: Binding::Place,
            account: "world:soil".into(), amount: 1.into() },
        Effect::Birth { provision: BTreeMap::from([("world:soil".into(), 1)]) },
        Effect::Transfer { from: Binding::Target, to: Binding::Place,
            account: "world:soil".into(), amount: 1.into() },
    ];
    genesis.sites.get_mut(&0).unwrap().accounts.insert("sim:energy".into(), 2);
    let selection = Selection { process: process.id.clone(), actor_lineage: "lineage:0".into(),
        target_lineage: Some("lineage:0".into()) };
    let pairs = vec![
        (NativeReference::Process(process.id.clone()), NativeReference::Process(original.id.clone())),
        (NativeReference::Account(input.clone()), NativeReference::Account(source)),
        (NativeReference::Account(destination.clone()), NativeReference::Account(destination)),
        (NativeReference::Lineage("lineage:0".into()), NativeReference::Lineage("lineage:0".into())),
        (NativeReference::Trait("ability:cycle-0".into()), NativeReference::Trait("ability:cycle-0".into())),
        (NativeReference::Condition("world:habitable".into()), NativeReference::Condition("world:habitable".into())),
    ].into_iter().map(|(candidate, compared)| ReferencePair { candidate, compared }).collect();
    let extra = ExtraNoun { noun: NativeReference::Process(process.id.clone()), consumer: selection.clone(),
        effect: address(&process.id), compare: address(&original.id), equivalences: pairs,
        axis: MechanicalAxis::TypedFlow };
    let mut extras = vec![extra.clone()];
    if account_alias {
        extras.push(ExtraNoun { noun: NativeReference::Account(input.clone()), ..extra });
    }
    let effect = |index| ScalarEffect { process: process.id.clone(), commitment: false, index };
    let stock = |account: &str| Dimension::Stock(genesis.rules.accounts[account].clone());
    let scalars = vec![
        declaration(&process, "period", ScalarSource::Period(process.id.clone()), Dimension::Ticks, Conservation::NotStock),
        declaration(&process, "priority", ScalarSource::Priority(process.id.clone()), Dimension::Priority, Conservation::NotStock),
        declaration(&process, "source-gate", ScalarSource::Requirement { process: process.id.clone(), index: 2 },
            stock(&input), Conservation::NativeMatter),
        declaration(&process, "habitable-gate", ScalarSource::Requirement { process: process.id.clone(), index: 3 },
            Dimension::ConditionLevel, Conservation::NotStock),
        declaration(&process, "cycle-input", ScalarSource::Input { effect: effect(0), account: input },
            stock(take.keys().next().unwrap()), Conservation::NativeMatter),
        declaration(&process, "cycle-output", ScalarSource::Output { effect: effect(0), account: give.keys().next().unwrap().clone() },
            stock(give.keys().next().unwrap()), Conservation::NativeMatter),
        declaration(&process, "capture", ScalarSource::Amount(effect(1)), stock("sim:energy"), Conservation::OtherStock),
        declaration(&process, "return", ScalarSource::Amount(effect(2)), stock("world:soil"), Conservation::NativeMatter),
        declaration(&process, "child", ScalarSource::Input { effect: effect(3), account: "world:soil".into() },
            stock("world:soil"), Conservation::NativeMatter),
        declaration(&process, "counter", ScalarSource::Amount(effect(4)), stock("world:soil"), Conservation::NativeMatter),
    ];
    let spec = JointSpec { version: 1, key: "test:joint-879".into(), horizon: 1,
        capture: CaptureRequirement { selection: selection.clone(), stock: CaptureStock::EnergyAccount("sim:energy".into()), at_least: 1 },
        returned: ReturnRequirement { selection: selection.clone(), account: "world:soil".into(), at_least: 1 },
        reproduction: BirthRequirement { selection: selection.clone(), at_least: 1 },
        counterplay: CounterRequirement { selection, proof: CounterProof::TargetDebited { account: "world:soil".into(), at_least: 1 } },
        independence: Some(IndependenceSpec { version: 1, extras, scalars }) };
    genesis.rules.processes.insert(process.id.clone(), process);
    genesis.rules.ecology_witness = Some(spec.clone());
    genesis.validate().unwrap();
    Candidate { basis, genesis, spec, process: process_name.into() }
}

impl Candidate {
    pub fn sync(&mut self) { self.genesis.rules.ecology_witness = Some(self.spec.clone()); }
    pub fn witness(&self) -> JointWitness {
        run_joint(&self.genesis, &self.spec, Execution::Individuals,
            &mut |session| session.advance_tick_with_flows()).unwrap()
    }
    pub fn admission(&self, witness: &JointWitness) -> Result<Admission, String> {
        admit(witness, self.spec.independence.as_ref().unwrap(), &self.basis)
    }
}
