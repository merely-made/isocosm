// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::sync::Mutex;
use crate::rules::{AccountKind, Binding, Rules};

pub(crate) type Balances = BTreeMap<Slot, u64>;

/// Every clone receives a distinct capture. Scratch is never state authority.
#[derive(Debug, Default)]
pub(crate) struct Capture(Mutex<Option<Context>>);
impl Clone for Capture {
    fn clone(&self) -> Self { Self(Mutex::new(self.0.lock().expect("native capture lock").clone())) }
}
impl PartialEq for Capture { fn eq(&self, _: &Self) -> bool { true } }
impl Eq for Capture {}

#[derive(Clone, Debug)]
pub(crate) enum Mode { Preview, Admit(Vec<DebitSelection>), Authored }

#[derive(Clone, Debug)]
pub(crate) struct Piece { pub lot: StockLot, pub amount: u64, pub preserve: Option<Slot> }

#[derive(Clone, Debug)]
pub(crate) struct Context {
    pub mode: Mode,
    pub origin: DebitOrigin,
    pub action: Key,
    pub tick: Tick,
    pub revision: Key,
    pub operation: u64,
    pub next_operation: u64,
    pub active: bool,
    pub failed: bool,
    pub requests: Vec<DebitRequest>,
    pub used: BTreeSet<usize>,
    pub shadow: BTreeMap<Key, OwnedLot>,
    pub staged_titles: BTreeMap<Key, OwnedLot>,
    pub initial: BTreeMap<Key, OwnedLot>,
    pub before: Balances,
    pub remaining: Balances,
    pub paid: Balances,
    pub pieces: Vec<Piece>,
    pub bindings: Vec<(Binding, StockSource)>,
    pub preserve: Option<Slot>,
    pub rules: Option<StockRules>,
    pub matter: BTreeSet<Key>,
    pub claims: Vec<Key>,
    pub outputs: BTreeMap<SpreadOutput,SpreadOutputWitness>,
    pub output_report: Vec<SpreadOutputWitness>,
    pub produced: BTreeMap<Key,ProducedClaim>,
    pub initial_produced: BTreeMap<Key,ProducedClaim>,
    pub identities: BTreeMap<Key,crate::spread::Identity>,
    pub reference_scope: bool,
}

impl Capture {
    pub fn take(&self) -> Option<Context> { self.0.lock().expect("native capture lock").take() }
    pub fn set(&self, context: Option<Context>) { *self.0.lock().expect("native capture lock") = context; }
    pub fn active(&self) -> bool { self.0.lock().expect("native capture lock").is_some() }
    pub fn preview(&self) -> bool {
        self.0.lock().expect("native capture lock").as_ref().is_some_and(|c| matches!(c.mode, Mode::Preview))
    }
    pub fn with<T>(&self, use_context: impl FnOnce(&mut Context) -> Result<T>) -> Result<Option<T>> {
        let mut locked = self.0.lock().map_err(|_| "native debit capture lock poisoned")?;
        locked.as_mut().map(use_context).transpose()
    }
    pub fn record(&self, state: &State, source: StockSource, account: &str, amount: u64, balance: u64) -> Result<bool> {
        self.with(|context| {
            let owned = context.shadow.values().any(|held|
                held.current_source() == Some(source) && held.lot.account == account);
            let result = context.record(state, source, account, amount, balance);
            if result.is_err() && owned { context.failed = true; }
            result
        }).map(|used| used.unwrap_or(false))
    }
}

impl Context {
    pub fn new(state: &State, rules: &Rules, revision: Key, origin: DebitOrigin,
        action: Key, mode: Mode) -> Self {
        Self {
            mode, origin, action, tick: state.tick, revision,
            operation: 0, next_operation: 0, active: false, failed: false,
            requests: Vec::new(), used: BTreeSet::new(), shadow: state.projects.ownership.clone(), staged_titles: BTreeMap::new(),
            initial: state.projects.ownership.clone(), before: Balances::new(), remaining: Balances::new(),
            paid: Balances::new(), pieces: Vec::new(), bindings: Vec::new(), preserve: None,
            rules: rules.owned_stock.clone(),
            matter: rules.accounts.iter().filter_map(|(key, kind)|
                matches!(kind, AccountKind::Matter { .. }).then_some(key.clone()))
                .chain(state.spreads.identities.iter().map(|identity| identity.key())).collect(), claims: Vec::new(),
            outputs: BTreeMap::new(), output_report: Vec::new(),
            produced:state.projects.produced.clone(), initial_produced:state.projects.produced.clone(),
            identities:state.spreads.identities.iter().map(|identity| (identity.key(),identity.clone())).collect(),
            reference_scope:references::declared(rules.owned_stock.as_ref()),
        }
    }
    pub fn begin(&mut self, mut balances: Balances, bindings: Vec<(Binding, StockSource)>, preserve: Option<Slot>) -> Result<()> {
        if self.active { return Err("native debit operation cannot nest".into()); }
        self.operation = self.next_operation;
        self.next_operation = self.next_operation.checked_add(1).ok_or("native operation ordinal overflow")?;
        self.active = true;
        balances.retain(|(_, key), _| self.matter.contains(key));
        self.before = balances.clone();
        self.remaining = balances;
        self.paid.clear();
        self.pieces.clear();
        self.outputs.clear();
        self.bindings = bindings;
        self.preserve = preserve;
        Ok(())
    }
    pub fn complete(&self) -> Result<()> {
        if self.active { return Err("native debit operation did not complete".into()); }
        if let Mode::Admit(selections) = &self.mode {
            if self.used.len() != selections.len() { return Err("owned admission supplied unused or stale debit selections".into()); }
        }
        Ok(())
    }
    fn record(&mut self, _state: &State, source: StockSource, account: &str, amount: u64, balance: u64) -> Result<bool> {
        if amount == 0 { return Ok(true); }
        if !self.matter.contains(account) { return Ok(false); }
        // Unowned kernel spill still uses its original admission; titled spill refuses below.
        if !self.active { return Ok(false); }
        let request = DebitRequest {
            action: self.action.clone(), origin: self.origin.clone(), tick: self.tick, revision: self.revision.clone(),
            operation: self.operation, ordinal: self.requests.len() as u64, source, account: account.into(), amount,
        };
        self.requests.push(request.clone());
        let slot = (source, account.to_owned());
        self.remaining.insert(slot.clone(),balance);
        if balance < amount { return Err("exact native debit lacks actual source stock".into()); }
        let mut selection = None;
        if let Mode::Admit(selections) = &self.mode {
            let matching: Vec<_> = selections.iter().enumerate().filter(|(_, s)| s.request == request).collect();
            if matching.len() > 1 { return Err("native debit request has ambiguous selected lots".into()); }
            if let Some((index, chosen)) = matching.first() {
                if !self.used.insert(*index) { return Err("native debit selection was already used".into()); }
                selection = Some((*chosen).clone());
            }
        }
        if matches!(self.mode, Mode::Preview) {
            self.remaining.insert(slot.clone(), balance - amount);
            add(&mut self.paid, slot, amount)?;
            return Ok(true);
        }
        if selection.is_none() {
            selection = self.authored_selection(&request)?;
        }
        let claimed = self.shadow.values().filter(|held| held.current_source() == Some(source) && held.lot.account == account)
            .try_fold(0u64, |sum, held| sum.checked_add(held.lot.amount).ok_or("native claim quantity overflow"))?;
        if claimed > balance { return Err("native source claim is stale relative to actual matter".into()); }
        let chosen = match selection {
            Some(chosen) => chosen,
            None if claimed == 0 => DebitSelection { request, lots: Vec::new(), unowned: amount },
            None => return Err("owned native debit needs an explicit exact lot selection".into()),
        };
        let total = chosen.lots.iter().try_fold(chosen.unowned, |sum, lot|
            sum.checked_add(lot.amount).ok_or("selected stock quantity overflow"))?;
        if total != amount || chosen.unowned > balance - claimed {
            return Err("selected owned and unowned quantities differ from the actual debit".into());
        }
        let mut seen = BTreeSet::new();
        for quantity in chosen.lots {
            if quantity.amount == 0 || !seen.insert(quantity.key.clone()) { return Err("selected stock lot is zero or duplicated".into()); }
            let held = self.shadow.get_mut(&quantity.key).ok_or("selected native stock lot is absent")?;
            if held.current_source() != Some(source) || held.lot.account != account || held.lot.owners.is_empty() {
                return Err("selected stock lot names a different native holder or account".into());
            }
            held.lot.amount = held.lot.amount.checked_sub(quantity.amount).ok_or("selected quantity exceeds the native lot remainder")?;
            self.pieces.push(Piece { lot: held.lot.clone(), amount: quantity.amount, preserve: self.preserve.clone() });
            if held.lot.amount == 0 { held.at = LotAt::Closed; }
        }
        self.remaining.insert(slot.clone(), balance - amount);
        add(&mut self.paid, slot, amount)?;
        Ok(true)
    }
    pub fn exempt(&mut self, source: StockSource, key: &str, amount: u64, actual: u64) -> Result<()> {
        if amount == 0 || !self.matter.contains(key) { return Ok(()); }
        if !self.active { return Err("exact project transfer lacks an actual operation boundary".into()); }
        let slot = (source, key.to_owned());
        self.remaining.insert(slot.clone(),actual.checked_sub(amount).ok_or("exact project source stock was taken already")?);
        add(&mut self.paid, slot, amount)
    }
}

pub(super) fn actual(state: &State, source: StockSource, account: &str) -> u64 {
    match source {
        StockSource::Entity(id) => state.population.get(id).map(crate::anatomy::books)
            .and_then(|books| books.get(account).copied()).unwrap_or(0),
        StockSource::Site(id) => state.sites.get(&id).and_then(|site| site.accounts.get(account).copied()).unwrap_or(0),
        StockSource::Place(id) => state.spreads.places.iter().find(|place| place.place == id)
            .and_then(|place| place.accounts.get(account).copied()).unwrap_or(0),
        StockSource::Transit(id) => transit_amount(state,id,account).unwrap_or(0),
    }
}

pub(super) fn add(rows: &mut Balances, slot: Slot, amount: u64) -> Result<()> {
    let current = rows.entry(slot).or_default();
    *current = current.checked_add(amount).ok_or("native debit evidence overflow")?;
    Ok(())
}
