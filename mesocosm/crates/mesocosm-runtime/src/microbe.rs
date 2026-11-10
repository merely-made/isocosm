// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A microbe to hold (cases 1216, 1221, 1223, 1226). Mesocosm's generated
//! worlds have no surface lineage, so `--hosting` gives one: the world's
//! hosting rule and its four genes, a microbe lineage, and a small stock of
//! it at the played region's first site.
//!
//! No spread command turns a site's dead matter into a surface stock: a
//! placement takes only an account of the surface's own lineage, and a
//! surface lineage can only be declared in a genesis. So the genesis declares
//! the microbe's own matter account and moves the seed from the site's soil
//! into it, matter for matter; the logged commands then register the surface
//! and place that matter as its stock (Eponym's `Register` and `Place`
//! precedent), so the seeding replays from the session log.

use crate::Runtime;
use cosmograph::{Command, Execution, Session,
    directing::readings, rules::{AccountKind, Hosting}, schema::{Id, Lineage, Method},
    simulation::Genesis, spread::{self, Identity, Placement, Substrate, Supply}};

/// The microbe's lineage and its surface key.
pub const KEY: &str = "lineage:micro";
/// The account its seed lies in until the placement makes it a stock.
pub const ACCOUNT: &str = "matter:micro";
/// Matter taken from the first site's soil and placed as the microbe.
pub const SEED_AMOUNT: u64 = 24;
/// The genes the microbe carries; the hosting rule declares four.
pub const GENES: [&str; 2] = ["gene:rot", "gene:enter"];
const SOIL: &str = "world:soil";

/// The first site of the region the played body lives in.
fn first_site(genesis: &Genesis) -> Result<Id, String> {
    let played = genesis.founding.as_ref().and_then(|f| f.played)
        .ok_or("--hosting needs a world with a played lineage")?;
    let lineage = format!("lineage:{}", played.lineage);
    let session = Session::new(genesis.clone(), Execution::Individuals)?;
    let sim = &session.sim;
    let site = sim.state().population.groups.values().map(|g| &g.entity)
        .find(|e| e.alive && e.method == Method::Deliberative && e.lineage == lineage)
        .and_then(|e| e.resident_site()).ok_or("--hosting found no played body's site")?;
    let regions = readings::regions(sim);
    let region = readings::region_of(&regions, site).ok_or("--hosting found no region for the played site")?;
    region.sites.iter().next().copied().ok_or_else(|| "the played region has no site".into())
}

/// Sets the hosting rule and declares the microbe and its seed in `genesis`,
/// the seed taken from the first site's soil, so the world's matter is as it was.
///
/// # Errors
/// The world has no played lineage, or its first site has too little soil.
pub fn declare(genesis: &mut Genesis) -> Result<(), String> {
    let hosting = Hosting::default();
    let site = first_site(genesis)?;
    let rules = &mut genesis.rules;
    rules.traits.extend([&hosting.rot, &hosting.feed, &hosting.leak, &hosting.enter].map(Clone::clone));
    rules.hosting = Some(hosting);
    rules.accounts.insert(ACCOUNT.into(), AccountKind::Matter {
        lineage: KEY.into(), reserve: false, provision: false });
    genesis.lineages.insert(KEY.into(), Lineage {
        parent: None, revision: 1, traits: GENES.map(String::from).into(), kingdom: "kingdom:micro".into(),
        development: None, decides: None, policy: None, meals: Default::default(), regard: Default::default(),
        ate: Default::default(), held: Default::default() });
    let ledger = &mut genesis.sites.get_mut(&site).ok_or("the first site is absent")?.accounts;
    let soil = ledger.get_mut(SOIL).filter(|soil| **soil >= SEED_AMOUNT)
        .ok_or("--hosting needs soil at the played region's first site")?;
    *soil -= SEED_AMOUNT;
    *ledger.entry(ACCOUNT.into()).or_default() += SEED_AMOUNT;
    Ok(())
}

/// The commands that make the declared seed a registered surface's stock at `site`.
pub fn seeding(site: Id) -> [Command; 2] {
    let identity = Identity::Surface(KEY.into());
    [Command::Spread(spread::Command::Register(identity.clone())),
     Command::Spread(spread::Command::Place(Placement {
         source: Supply::Matter { from: Substrate::Site(site), account: ACCOUNT.into() },
         identity, at: Substrate::Site(site), amount: SEED_AMOUNT }))]
}

impl Runtime {
    /// Seeds a world `declare` prepared, before play: two logged commands.
    ///
    /// # Errors
    /// The world was not prepared, or the sim refuses a command.
    pub fn seed_microbe(&mut self) -> Result<(), String> {
        let site = {
            let sim = self.sim();
            sim.state().sites.iter().find(|(_, site)| site.accounts.get(ACCOUNT).is_some_and(|n| *n >= SEED_AMOUNT))
                .map(|(id, _)| *id).ok_or("no site holds the microbe's seed")?
        };
        self.with_product(|product, world| {
            let mut game = product.spatial.clone();
            world.cut().and_then(|before| world.product(before, |session| {
                for command in seeding(site) { session.command(command)?; }
                game.refresh(session)?;
                crate::continuation::rebind(session, &mut game);
                game.validate(session)
            }))?;
            product.spatial = game;
            product.sync_loop(world);
            Ok(())
        })
    }
}
