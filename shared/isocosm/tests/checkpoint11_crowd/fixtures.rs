// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit C11 world inputs, without changing C10's generator or columns.

use isocosm::{Command, Execution, Founding, Session, probe::{Crowd,ProbeWorld,readings::spreads},
    rules::{AccountKind,Expr,Reading}, schema::{Id,Route},
    spread::{self,CapacityLayer,CapacityModifier,CapacityRules,CapacitySource,Identity,Placement,Stacking,Substrate,Supply}};
use std::collections::{BTreeMap,BTreeSet};

pub const SUPPLY:&str = "c11-crowd:supply";
pub const GENE:&str = "c11-crowd:capacity";
pub const LAYER:&str = "capacity:modifier";

pub struct World {
    pub probe:ProbeWorld,
    pub host:Id,
    pub cohort:u64,
    pub from:Id,
    pub to:[Id;3],
    pub territory:Identity,
    pub surface:Identity,
}

pub fn world(seed:u64) -> World {
    let mut genesis = Founding {seed,sites:4,population:24,lineages:1,cohort_size:8,
        ..Default::default()}.generate().unwrap();
    genesis.rules.processes.clear();
    genesis.rules.competitions.clear();
    genesis.rules.epoch_ticks = 2;
    genesis.rules.shapes = isocosm::rules::default_shapes();
    genesis.rules.functions = isocosm::rules::default_functions();
    for group in genesis.population.groups.values_mut() {
        assert!(group.entity.parts.values().all(|p| p.matter.is_empty() && p.cells.is_empty()));
        group.entity_mut().parts.clear();
    }
    let (&host,group) = genesis.population.groups.iter().find(|(_,g)| g.entity.kingdom != "kingdom:world").unwrap();
    let cohort = group.count;
    assert!(cohort > 1);
    let from = group.entity.resident_site().unwrap();
    let lineage = group.entity.lineage.clone();
    let to:Vec<_> = genesis.sites.keys().copied().filter(|id| *id != from).collect();
    let to = [to[0],to[1],to[2]];
    genesis.rules.traits.extend([GENE.into(),"function:fix".into()]);
    genesis.lineages.get_mut(&lineage).unwrap().traits.extend([GENE.into(),"function:fix".into()]);
    genesis.rules.accounts.insert(SUPPLY.into(),AccountKind::Matter {
        lineage:lineage.clone(),reserve:false,provision:false});
    let effect = genesis.world.canon.glyphs.first().unwrap().effect.clone();
    let kept = |key:&str| Expr::Read(Reading::Kept {name:key.into()});
    genesis.rules.spread_capacity = Some(CapacityRules {key:"capacity:base".into(),
        trait_inputs:BTreeMap::from([("capacity:gene".into(),GENE.into())]),
        mechanic:Expr::Mul(vec![kept("capacity:gene"),Expr::Const(20)]),
        layers:vec![CapacityLayer {key:LAYER.into(),effect,stacking:Stacking::Sum,
            input:"capacity:value".into(),reading:Expr::Add(vec![kept("capacity:base"),kept("capacity:value")])}]});
    for site in genesis.sites.values_mut() {site.routes.clear();}
    genesis.sites.get_mut(&from).unwrap().routes = vec![
        Route {to:to[0],travel:2,transmission:500_000,border:None,authored:None},
        Route {to:to[1],travel:3,transmission:1_000_000,border:None,authored:None},
        Route {to:to[2],travel:1,transmission:0,border:None,authored:None},
    ];
    genesis.sites.get_mut(&from).unwrap().accounts.insert(SUPPLY.into(),500);
    let territory = Identity::Territory(host);
    let surface = Identity::Surface(lineage);
    genesis.spreads.identities = BTreeSet::from([territory.clone(),surface.clone()]);
    genesis.validate().unwrap();
    World {probe:ProbeWorld {genesis,ticks:7},host,cohort,from,to,territory,surface}
}

pub fn modifier(key:&str,source:CapacitySource,identity:Option<Identity>,milligrams:u64) -> CapacityModifier {
    CapacityModifier {key:key.into(),source,identity,layer:LAYER.into(),milligrams}
}

pub struct Pair<'w> {
    pub native:Session,
    pub crowd:Crowd<'w>,
    pub mass:u128,
}

impl<'w> Pair<'w> {
    pub fn new(w:&'w World,named:&BTreeSet<Id>) -> Self {
        let native = Session::new(w.probe.genesis.clone(),Execution::Individuals).unwrap();
        let crowd = Crowd::for_spreads(&w.probe,w.probe.genesis.dynamics_seed(),named).unwrap();
        let mass = native.sim.matter();
        let pair = Self {native,crowd,mass};
        pair.compare();
        pair
    }

    pub fn compare(&self) {
        let names = self.crowd.spread_names().unwrap();
        let exact = spreads::native(self.native.sim.state(),&self.native.sim.genesis().rules,&names).unwrap();
        let histogram = spreads::crowd(&self.crowd).unwrap();
        assert_eq!(histogram,exact,"independent C11 stock/queue/lineage readings");
        assert_eq!(exact.total_matter,self.mass,"all actual site/host/pending mg exactly once");
    }

    pub fn command(&mut self,command:spread::Command) -> String {
        let accepted = self.native.command(Command::Spread(command.clone())).unwrap();
        assert_eq!(self.crowd.spread(&command).unwrap(),accepted);
        self.compare();
        accepted
    }

    pub fn place(&mut self,w:&World,identity:&Identity,at:Substrate,amount:u64) {
        self.command(spread::Command::Place(Placement {source:Supply::Matter {
            from:Substrate::Site(w.from),account:SUPPLY.into()},identity:identity.clone(),at,amount}));
    }

    pub fn advance(&mut self,ticks:u64) {
        self.native.advance(ticks).unwrap();
        self.crowd.advance_spreads(ticks).unwrap();
        self.compare();
    }

    pub fn replay(&self) {
        let bytes = serde_json::to_vec(&self.native.save()).unwrap();
        let names = self.crowd.spread_names().unwrap();
        let before = spreads::crowd(&self.crowd).unwrap();
        for execution in [Execution::Individuals,Execution::Grouped] {
            let restored = Session::load_json(&bytes,execution).unwrap();
            assert_eq!(restored.sim.state_hash(),self.native.sim.state_hash());
            assert_eq!(before,spreads::native(restored.sim.state(),&restored.sim.genesis().rules,&names).unwrap());
        }
    }

    pub fn refused(&mut self,command:spread::Command) {
        let native = self.native.save();
        let crowd = spreads::crowd(&self.crowd).unwrap();
        assert!(self.native.command(Command::Spread(command.clone())).is_err());
        assert!(self.crowd.spread(&command).is_err());
        assert_eq!(self.native.save(),native);
        assert_eq!(spreads::crowd(&self.crowd).unwrap(),crowd);
        self.compare();
    }
}

