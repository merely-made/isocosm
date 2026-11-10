// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a surface does where it lives (cases 1223, 1225, 1226), on the
//! clock the hosting rule sets. A surface rots a site's dead matter, feeds on
//! a host's tissue, leaks to a host's reserve or enters the bodies at a site,
//! as its lineage's genes allow; whatever its genes, matter in a host that is
//! no longer alive is released to the host's site. Every unit moves between
//! two stocks the world already counts, so matter is conserved, and each move
//! is a flow naming the lineage of each side. Where the rule is unset nothing
//! here runs.

use super::*;
use crate::{
    Result, anatomy, directing,
    ecology::world_time::DETRITUS,
    flows::{Holder, Kind, Kinded, Leg, MadeBy},
    meaning,
    projects::{StockSource, guard_debit},
    rules::{Hosting, Rules},
    schema::{Id, Key, PartId, Tick},
    simulation::{Simulation, State},
};

const SOIL: &str = "world:soil";

/// An account taken from, how much, and the parts it came from.
type Taken = (Key, u64, Vec<(PartId, u64)>);

/// `share` of `held`, at least one where it would round to nought, never
/// more than is held; nothing where the share is nought (cases 1223, 1225).
pub(super) fn share(held: u64, [n, d]: [u32; 2]) -> u64 {
    if n == 0 {
        return 0;
    }
    let part = u128::from(held) * u128::from(n) / u128::from(d.max(1));
    u64::try_from(part).unwrap_or(held).max(1).min(held)
}

fn held_at(state: &State, at: Substrate, key: &str) -> u64 {
    stock::ledger(state, at).ok().and_then(|ledger| ledger.get(key)).copied().unwrap_or(0)
}

/// One turn's moves, kept as flows (case 1226).
struct Turn<'a> {
    rules: &'a Rules,
    hosting: &'a Hosting,
    moves: Vec<Kinded>,
}

impl Turn<'_> {
    fn record(&mut self, by: &str, from: (Holder, &str), to: (Holder, &str), amount: u64, kinds: [Option<Kind>; 2]) {
        let leg = Leg { from: (from.0, from.1.into()), to: (to.0, to.1.into()), amount };
        self.moves.push(Kinded { made_by: MadeBy::Process(format!("hosting:{by}")), leg, kinds });
    }

    /// A surface on a site takes up to `take` of its matter from the site's
    /// detritus, then its soil (case 1225).
    fn rot(&mut self, state: &mut State, key: &str, site: Id, m: u64) -> Result<()> {
        let mut want = share(m, self.hosting.take);
        let surface = kind_of(state, key);
        for dead in [DETRITUS, SOIL] {
            let have = held_at(state, Substrate::Site(site), dead);
            let n = want.min(have);
            if n == 0 || !meaning::matter(self.rules, dead)
                || guard_debit(state, StockSource::Site(site), dead, n).is_err() {
                continue;
            }
            let ledger = stock::ledger_mut(state, Substrate::Site(site))?;
            meaning::debit(ledger, dead, n)?;
            meaning::credit(ledger, key, n)?;
            want -= n;
            self.record("rot", (Holder::Site(site), dead), (Holder::Site(site), key), n, [None, surface.clone()]);
        }
        Ok(())
    }

    /// A surface in a living host takes up to `take` of its matter from the
    /// host's reserve, then its tissue, as upkeep would spend them; a host
    /// emptied dies by the starving process, not here (case 1225).
    fn feed(&mut self, state: &mut State, key: &str, host: Id, m: u64) -> Result<()> {
        let rules = self.rules;
        let Some(body) = state.population.get(host) else { return Ok(()) };
        let own = anatomy::own(rules, &body.lineage);
        let (kinds, mut e, mut want) = (kind_pair(body, state, key), body.clone(), share(m, self.hosting.take));
        let mut taken: Vec<Taken> = Vec::new();
        for account in [own.reserve, own.tissue].into_iter().flatten() {
            let ledger = e.accounts.get(&account).copied().unwrap_or(0);
            let have = match anatomy::anatomical(&e, rules, &account) {
                true => anatomy::held(&e, rules, &account) - ledger,
                false => ledger,
            };
            let n = want.min(have);
            if n == 0 || guard_debit(state, StockSource::Entity(host), &account, n).is_err() {
                continue;
            }
            let mut next = e.clone();
            let parts = match anatomy::take(&mut next, rules, &account, n) {
                Some(Ok(parts)) => parts,
                Some(Err(_)) => continue,
                None => {
                    meaning::debit(&mut next.accounts, &account, n)?;
                    Vec::new()
                },
            };
            meaning::credit(&mut next.accounts, key, n)?;
            (e, want) = (next, want - n);
            taken.push((account, n, parts));
        }
        if taken.is_empty() {
            return Ok(());
        }
        *state.population.lift(host)? = e;
        for (account, n, parts) in taken {
            for (holder, amount) in holders(host, n, &parts) {
                self.record("feed", (holder, &account), (Holder::Entity(host), key), amount, [kinds[0].clone(), kinds[1].clone()]);
            }
        }
        Ok(())
    }

    /// A surface in a living host gives up to `give` of its matter to the
    /// host's reserve, as far as the host has room (case 1225).
    fn leak(&mut self, state: &mut State, key: &str, host: Id, m: u64) -> Result<()> {
        let rules = self.rules;
        let Some(body) = state.population.get(host) else { return Ok(()) };
        let Some(reserve) = anatomy::own(rules, &body.lineage).reserve else { return Ok(()) };
        let (kinds, mut e) = (kind_pair(body, state, key), body.clone());
        let mut n = share(m, self.hosting.give).min(e.accounts.get(key).copied().unwrap_or(0));
        if anatomy::anatomical(&e, rules, &reserve) {
            n = n.min(anatomy::room(&e, rules, &reserve));
        }
        if n == 0 || guard_debit(state, StockSource::Entity(host), key, n).is_err() {
            return Ok(());
        }
        meaning::debit(&mut e.accounts, key, n)?;
        let parts = match anatomy::give(&mut e, rules, &reserve, n) {
            Some(Ok(parts)) => parts,
            Some(Err(_)) => return Ok(()),
            None => {
                meaning::credit(&mut e.accounts, &reserve, n)?;
                Vec::new()
            },
        };
        *state.population.lift(host)? = e;
        for (holder, amount) in holders(host, n, &parts) {
            self.record("leak", (Holder::Entity(host), key), (holder, &reserve), amount, [kinds[1].clone(), kinds[0].clone()]);
        }
        Ok(())
    }

    /// A surface on a site sends up to `send` of what it has left into each
    /// body residing there that is not of its lineage, a participant or the
    /// world's, in entity order; a cohort enters as its first member (case 1225).
    fn enter(&mut self, state: &mut State, key: &str, site: Id) -> Result<()> {
        let hosts: Vec<Id> = state.population.groups.iter().filter(|(_, g)| {
            let e = &g.entity;
            e.alive && e.resident_site() == Some(site) && e.lineage != key
                && !directing::is_participant(e) && e.kingdom != "kingdom:world"
        }).map(|(first, _)| *first).collect();
        let surface = kind_of(state, key);
        for host in hosts {
            let n = share(held_at(state, Substrate::Site(site), key), self.hosting.send);
            if n == 0 || guard_debit(state, StockSource::Site(site), key, n).is_err() {
                continue;
            }
            meaning::debit(stock::ledger_mut(state, Substrate::Site(site))?, key, n)?;
            meaning::credit(&mut state.population.lift(host)?.accounts, key, n)?;
            self.record("enter", (Holder::Site(site), key), (Holder::Entity(host), key), n, [surface.clone(), surface.clone()]);
        }
        Ok(())
    }

    /// A surface in a host no longer alive returns to the site the host
    /// resided at; with no site it stays (case 1226).
    fn release(&mut self, state: &mut State, key: &str, host: Id, m: u64) -> Result<()> {
        let Some(site) = state.population.get(host).and_then(crate::schema::Entity::resident_site)
            .filter(|site| state.sites.contains_key(site)) else { return Ok(()) };
        if guard_debit(state, StockSource::Entity(host), key, m).is_err() {
            return Ok(());
        }
        meaning::debit(&mut state.population.lift(host)?.accounts, key, m)?;
        meaning::credit(stock::ledger_mut(state, Substrate::Site(site))?, key, m)?;
        let surface = kind_of(state, key);
        self.record("release", (Holder::Entity(host), key), (Holder::Site(site), key), m, [surface.clone(), surface]);
        Ok(())
    }
}

/// The lineage and kingdom of surface `key`.
fn kind_of(state: &State, key: &str) -> Option<Kind> {
    let line = state.lineages.get(key)?;
    Some(Kind { lineage: key.into(), kingdom: line.kingdom.clone() })
}

/// The host's kind, then the surface's.
fn kind_pair(body: &crate::schema::Entity, state: &State, key: &str) -> [Option<Kind>; 2] {
    [Some(Kind { lineage: body.lineage.clone(), kingdom: body.kingdom.clone() }), kind_of(state, key)]
}

/// Where a body's matter moved: its parts as routed, or the body itself.
fn holders(host: Id, n: u64, parts: &[(PartId, u64)]) -> Vec<(Holder, u64)> {
    match parts.is_empty() {
        true => vec![(Holder::Entity(host), n)],
        false => parts.iter().map(|(part, amount)| (Holder::Part(host, *part), *amount)).collect(),
    }
}

/// Every surface's turn on `state`, in order of lineage key, then substrate,
/// with the moves made (cases 1223, 1225, 1226).
///
/// # Errors
/// Fails if a stock cannot be debited or credited as the turn found it.
pub(crate) fn turn(state: &mut State, rules: &Rules, hosting: &Hosting) -> Result<Vec<Kinded>> {
    let mut turn = Turn { rules, hosting, moves: Vec::new() };
    let mut stocks: Vec<(Key, Substrate)> = transport::stocks(state).into_iter()
        .filter(|(_, identity)| matches!(identity, Identity::Surface(_)))
        .map(|(at, identity)| (identity.key(), at)).collect();
    stocks.sort();
    for (key, at) in stocks {
        let m = held_at(state, at, &key);
        let traits = state.lineages.get(&key).map(|l| l.traits.clone()).unwrap_or_default();
        if m == 0 {
            continue;
        }
        match at {
            Substrate::Site(site) => {
                if traits.contains(&hosting.rot) {
                    turn.rot(state, &key, site, m)?;
                }
                if traits.contains(&hosting.enter) {
                    turn.enter(state, &key, site)?;
                }
            },
            Substrate::Host(host) => match state.population.get(host).map(|e| e.alive) {
                Some(true) => {
                    if traits.contains(&hosting.feed) {
                        turn.feed(state, &key, host, m)?;
                    }
                    if traits.contains(&hosting.leak) {
                        turn.leak(state, &key, host, m)?;
                    }
                },
                Some(false) => turn.release(state, &key, host, m)?,
                None => {},
            },
            Substrate::Place(_) => {},
        }
    }
    Ok(turn.moves)
}

impl Simulation {
    /// The tick of the next turn: the next multiple of the period, while any
    /// surface holds matter; none where the rule is unset (cases 1223, 1225).
    pub(crate) fn next_turn(&self) -> Option<Tick> {
        let period = self.genesis.rules.hosting.as_ref()?.period;
        transport::stocks(&self.state).iter().any(|(_, identity)| matches!(identity, Identity::Surface(_)))
            .then(|| (self.state.tick / period + 1).checked_mul(period)).flatten()
    }

    /// Takes the turn at `tick`, then the overflow dispatch the spread
    /// commands run; a turn that moves nothing only moves the clock.
    pub(crate) fn take_turn(&mut self, tick: Tick) -> Result<()> {
        let rules = &self.genesis.rules;
        let hosting = rules.hosting.as_ref().ok_or("no hosting rule")?;
        let mut candidate = self.state.clone();
        candidate.tick = tick;
        let moves = turn(&mut candidate, rules, hosting)?;
        if moves.is_empty() {
            self.state.tick = tick;
            return Ok(());
        }
        let before = self.matter();
        native::validate_state(&candidate, rules)?;
        dispatch_changed(&self.state, &mut candidate, rules)?;
        native::validate_state(&candidate, rules)?;
        if crate::simulation::matter(&candidate, rules) != before {
            return Err("a surface's turn does not conserve matter".into());
        }
        let mut accepted = self.clone();
        accepted.state = candidate;
        accepted.flowed_kinds(moves);
        self.publish_candidate(accepted)
    }
}
