// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Context {
    pub fn finish(&mut self, mut after: Balances) -> Result<()> {
        if !self.active { return Err("native stock operation did not begin".into()); }
        if matches!(self.mode, Mode::Preview) { self.active = false; return Ok(()); }
        after.retain(|(_, key), _| self.matter.contains(key));
        let mut credits = Balances::new();
        let slots: BTreeSet<_> = self.before.keys().chain(after.keys()).chain(self.paid.keys()).cloned().collect();
        for slot in slots {
            let start = self.before.get(&slot).copied().unwrap_or(0);
            let end = after.get(&slot).copied().unwrap_or(0);
            let paid = self.paid.get(&slot).copied().unwrap_or(0);
            let credit = end.checked_add(paid).and_then(|n| n.checked_sub(start))
                .ok_or("native stock loss bypassed its exact debit boundary")?;
            if credit > 0 { credits.insert(slot, credit); }
        }
        if !self.pieces.is_empty() {
            let mut selected: BTreeMap<Key, (StockLot, u64)> = BTreeMap::new();
            for piece in &self.pieces {
                let row = selected.entry(piece.lot.key.clone()).or_insert((piece.lot.clone(), 0));
                row.1 = row.1.checked_add(piece.amount).ok_or("selected claim quantity overflow")?;
            }
            let mut outputs: Vec<_> = self.pieces.iter().filter_map(|piece| piece.preserve.as_ref()
                .map(|(source, account)| OutputClaim { input: piece.lot.key.clone(), amount: piece.amount,
                    destination: Destination::Source(*source), account: account.clone() })).collect();
            let remaining: BTreeMap<Key, u64> = self.pieces.iter().filter(|piece| piece.preserve.is_none())
                .try_fold(BTreeMap::<Key, u64>::new(), |mut rows, piece| -> Result<_> {
                    let amount = rows.entry(piece.lot.key.clone()).or_default();
                    *amount = amount.checked_add(piece.amount).ok_or("consumed claim quantity overflow")?;
                    Ok(rows)
                })?;
            let mut closed = BTreeMap::new();
            if !remaining.is_empty() {
                let operation = self.operation_rule()?;
                let rules: Vec<_> = self.rules.iter().flat_map(|rules| &rules.dispositions)
                    .filter(|rule| rule.operation == operation).collect();
                let [rule] = rules.as_slice() else { return Err("owned debit needs one exact authored operation disposition".into()); };
                if matches!(operation,OperationRule::Spread { operation:SpreadDebit::Overflow, .. })
                    && !matches!(&rule.disposition,Disposition::Outputs { closed, .. } if closed.is_empty()) {
                    return Err("unchanged transported claims need exact authored route outputs".into());
                }
                match &rule.disposition {
                    Disposition::CloseConsumed => closed = remaining,
                    Disposition::Retain { outputs: retained, closed: portions } => {
                        for portion in portions {
                            let available = remaining.get(&portion.key).ok_or("closed portion names no consumed input lot")?;
                            if portion.amount == 0 || closed.insert(portion.key.clone(),portion.amount).is_some()
                                || portion.amount > *available {
                                return Err("closed portion exceeds or repeats its exact consumed input quantity".into());
                            }
                        }
                        outputs.extend(retained.clone());
                    },
                    Disposition::Outputs { outputs: retained,closed:portions } => {
                        for portion in portions {
                            let available = remaining.get(&portion.key).ok_or("closed spread portion names no selected input")?;
                            if portion.amount == 0 || closed.insert(portion.key.clone(),portion.amount).is_some()
                                || portion.amount > *available { return Err("closed spread portion differs from its selected input".into()); }
                        }
                        for retained in retained {
                            let actual = self.outputs.get(&retained.output).ok_or("authored claim names no actual native spread output")?;
                            outputs.push(OutputClaim { input:retained.input.clone(),amount:retained.amount,
                                destination:Destination::Source(actual.source),account:actual.identity.key() });
                        }
                    },
                }
            }
            if !outputs.is_empty() {
                let mut allocated = closed;
                for (index, output) in outputs.iter().enumerate() {
                    let (input, selected_amount) = selected.get(&output.input).ok_or("retained output names no selected input lot")?;
                    if output.amount == 0 { return Err("retained output claim has no quantity".into()); }
                    let to = match output.destination {
                        Destination::Source(source) => source,
                        Destination::Binding(binding) => self.bindings.iter().find_map(|(who, source)|
                            (*who == binding).then_some(*source)).ok_or("retained output has no actual native destination binding")?,
                    };
                    let slot = (to, output.account.clone());
                    let credit = credits.get_mut(&slot).ok_or("retained claim names no actual physical output credit")?;
                    *credit = credit.checked_sub(output.amount).ok_or("retained claims allocate an actual output credit twice")?;
                    let used = allocated.entry(output.input.clone()).or_default();
                    *used = used.checked_add(output.amount).filter(|n| n <= selected_amount)
                        .ok_or("retained output exceeds its explicitly selected input quantity")?;
                    let key = format!("stock:{}", crate::digest(&(&self.action, self.operation, index as u64,
                        &output.input, to, &output.account, output.amount)));
                    if self.shadow.contains_key(&key) { return Err("retained stock identity already exists".into()); }
                    let mut lot = input.clone();
                    lot.key = key.clone();
                    lot.source = to;
                    lot.account = output.account.clone();
                    lot.amount = output.amount;
                    self.shadow.insert(key.clone(), OwnedLot { lot, at: LotAt::Source });
                    self.claims.push(key);
                }
                if selected.iter().any(|(key, (_, amount))| allocated.get(key) != Some(amount)) {
                    return Err("retained disposition leaves an unspecified consumed claim portion".into());
                }
            }
        }
        let mut totals = Balances::new();
        for held in self.shadow.values() {
            if let Some(source) = held.current_source() {
                capture::add(&mut totals, (source, held.lot.account.clone()), held.lot.amount)?;
            }
        }
        for (slot, amount) in totals {
            if amount > after.get(&slot).copied().unwrap_or(0) {
                return Err("retained native claim exceeds its actual destination stock".into());
            }
        }
        self.active = false;
        Ok(())
    }
    pub fn changes(&self) -> BTreeMap<Key, OwnedLot> {
        self.shadow.iter().filter(|(key, held)| self.initial.get(*key) != Some(*held))
            .map(|(key, held)| (key.clone(), held.clone())).collect()
    }
}

pub(crate) fn state_balances(state: &State) -> Balances {
    let mut balances = Balances::new();
    for (id, group) in &state.population.groups {
        for (key, amount) in crate::anatomy::books(&group.entity) {
            balances.insert((StockSource::Entity(*id), key), amount);
        }
    }
    for held in state.projects.ownership.values() {
        if let Some(StockSource::Entity(id)) = held.current_source()
            && let Some(body) = state.population.get(id) {
            for (key,amount) in crate::anatomy::books(body) { balances.insert((StockSource::Entity(id),key),amount); }
        }
    }
    for (id, site) in &state.sites {
        for (key, amount) in &site.accounts { balances.insert((StockSource::Site(*id), key.clone()), *amount); }
    }
    for place in &state.spreads.places {
        for (key,amount) in &place.accounts { balances.insert((StockSource::Place(place.place),key.clone()),*amount); }
    }
    for mark in &state.spreads.transit_marks {
        if let Some(transfer) = state.spreads.transit.get(mark.index) {
            balances.insert((StockSource::Transit(mark.id),transfer.identity.key()),transfer.amount);
        }
    }
    balances
}
