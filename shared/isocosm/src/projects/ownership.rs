// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::{AccountKind, Binding};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StockSource { Entity(Id), Site(Id) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Owner { Entity(Id), Polity(Id) }

/// The claim names actual quantities; it holds no second physical ledger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockLot {
    pub key: Key,
    pub source: StockSource,
    pub account: Key,
    pub amount: u64,
    pub owners: BTreeSet<Owner>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub collectives: BTreeMap<Id, DecisionPolicy>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LotAt { Source, Project(ProjectId), Output(Id), Unresolved, Closed }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedLot { pub lot: StockLot, pub at: LotAt }

impl OwnedLot {
    pub fn current_source(&self) -> Option<StockSource> {
        match self.at {
            LotAt::Source => Some(self.lot.source),
            LotAt::Output(id) => Some(StockSource::Entity(id)),
            LotAt::Project(_) | LotAt::Unresolved | LotAt::Closed => None,
        }
    }
}

impl Simulation {
    pub fn source_lots(&self) -> impl Iterator<Item = &StockLot> {
        self.state.projects.ownership.values().filter(move |held|
            held.at == LotAt::Source && held.lot.amount > 0 && self.source_claims_sound(held.lot.source, &held.lot.account)
                && held.lot.owners.iter().all(|owner| self.owner_exists(*owner).is_ok()))
            .map(|held| &held.lot)
    }
    /// Completed output retains the original claim, at its current native holder.
    pub fn available_lots(&self) -> impl Iterator<Item = (StockSource, &StockLot)> {
        self.state.projects.ownership.values().filter_map(move |held| {
            let source = held.current_source()?;
            (held.lot.amount > 0 && self.source_claims_sound(source,&held.lot.account)
                && held.lot.owners.iter().all(|owner| self.owner_exists(*owner).is_ok())).then_some((source,&held.lot))
        })
    }
    fn source_claims_sound(&self, source: StockSource, account: &str) -> bool {
        let claimed = self.state.projects.ownership.values()
            .filter(|held| held.current_source() == Some(source) && held.lot.account == account)
            .try_fold(0u64, |sum, held| sum.checked_add(held.lot.amount));
        claimed.zip(self.source_amount(source, account).ok()).is_some_and(|(claim, actual)| claim <= actual)
    }
    pub(crate) fn stock_lot_at_start(&self, key: &str) -> Option<&OwnedLot> {
        self.frame.as_ref().and_then(|f| f.stock_lot(key))
            .map_or_else(|| self.state.projects.ownership.get(key), |was| was.as_ref())
    }
    pub(crate) fn stock_source(&self, actor: Id, target: Option<Id>, from: Binding) -> Result<StockSource> {
        match from {
            Binding::Place => self.body_at_start(actor).and_then(Entity::resident_site)
                .map(StockSource::Site).ok_or_else(|| "stock source has no resident site".into()),
            Binding::Actor | Binding::Target => self.bound(actor, target, from).map(StockSource::Entity),
            Binding::Part => Err("stock lots require an addressed body or site ledger".into()),
        }
    }
    fn source_amount(&self, source: StockSource, account: &str) -> Result<u64> {
        let ledger = match source {
            StockSource::Entity(id) => crate::anatomy::books(self.body_at_start(id).ok_or("stock body is absent")?),
            StockSource::Site(id) => self.site_at_start(id).ok_or("stock site is absent")?.accounts.clone(),
        };
        Ok(ledger.get(account).copied().unwrap_or(0))
    }
    pub(crate) fn assert_stock_ownership(&mut self, lot: &StockLot) -> Result<String> {
        if let Some(was) = self.state.projects.ownership.get(&lot.key) {
            return if &was.lot == lot { Ok(format!("stock-ownership:{}", lot.key)) }
                else { Err("stock lot is already asserted otherwise".into()) };
        }
        crate::validation::key(&lot.key)?;
        crate::validation::key(&lot.account)?;
        if lot.amount == 0 || lot.owners.is_empty()
            || !matches!(self.genesis.rules.accounts.get(&lot.account), Some(AccountKind::Matter { .. })) {
            return Err("stock ownership needs positive declared matter and native owners".into());
        }
        for owner in &lot.owners {
            self.owner_exists(*owner)?;
            if let Owner::Polity(id) = owner {
                let policy = lot.collectives.get(id).ok_or("collective owner needs an explicit decision policy")?;
                policy.validate(&self.genesis.rules)?;
                if let DecisionPolicy::Representatives { actors, .. } = policy {
                    for actor in actors { self.owner_exists(Owner::Entity(*actor))?; }
                }
            }
        }
        if lot.collectives.keys().any(|id| !lot.owners.contains(&Owner::Polity(*id))) {
            return Err("stock decision policy names no collective owner".into());
        }
        let claimed = self.state.projects.ownership.values()
            .filter(|held| held.current_source() == Some(lot.source) && held.lot.account == lot.account)
            .try_fold(lot.amount, |sum, held| sum.checked_add(held.lot.amount).ok_or("stock ownership overflow"))?;
        if claimed > self.source_amount(lot.source, &lot.account)? {
            return Err("stock ownership exceeds the actual native source ledger".into());
        }
        if let StockSource::Entity(id) = lot.source { self.state.population.lift(id)?; }
        self.state.projects.ownership.insert(lot.key.clone(), OwnedLot { lot: lot.clone(), at: LotAt::Source });
        Ok(format!("stock-ownership:{}", lot.key))
    }
    pub(crate) fn selected_project_lots(&self, actor: Id, authored: &[Key]) -> Result<Vec<Key>> {
        let supplied = self.project_inputs.as_ref().filter(|(id, _)| *id == actor).map(|(_, lots)| lots.as_slice());
        if !authored.is_empty() && supplied.is_some_and(|lots| !lots.is_empty() && lots != authored) {
            return Err("selected stock lots differ from the authored reservation".into());
        }
        let selected = supplied.filter(|lots| !lots.is_empty()).unwrap_or(authored);
        if selected.is_empty() { return Err("project reservation needs explicitly selected native stock lots".into()); }
        Ok(selected.to_vec())
    }
    pub(crate) fn check_project_lots(&self, source: StockSource, selected: &[Key], stocks: &Ledger)
        -> Result<Vec<OwnedLot>> {
        let mut seen = BTreeSet::new();
        let mut sum = Ledger::new();
        let mut out = Vec::new();
        let mut collectives = BTreeMap::new();
        for key in selected {
            if !seen.insert(key) { return Err("selected stock lot is duplicated".into()); }
            let held = self.state.projects.debit_capture.with(|context| Ok(context.shadow.get(key).cloned()))?
                .flatten().or_else(|| self.stock_lot_at_start(key).cloned())
                .ok_or("selected native stock lot is absent")?;
            if held.current_source() != Some(source) {
                return Err("selected stock lot is not at this native source".into());
            }
            for owner in &held.lot.owners { self.owner_exists(*owner)?; }
            if held.lot.owners.is_empty() { return Err("selected stock lot has no native owners".into()); }
            for (owner,policy) in &held.lot.collectives {
                if collectives.get(owner).is_some_and(|was| was != policy) {
                    return Err("selected native stock lots have ambiguous collective decision policy".into());
                }
                collectives.insert(*owner,policy.clone());
            }
            crate::meaning::credit(&mut sum, &held.lot.account, held.lot.amount)?;
            out.push(held.clone());
        }
        if &sum != stocks { return Err("whole selected stock lots differ from the exact recipe stocks".into()); }
        for (account, amount) in &sum {
            if !self.source_claims_sound(source, account) {
                return Err("native source ownership needs an explicit stock disposition".into());
            }
            if self.source_amount(source, account)? < *amount { return Err("selected stock lot lacks native matter".into()); }
        }
        Ok(out)
    }
}
