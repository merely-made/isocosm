// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Doing<'_> {
    fn admitted(&self, holdings: &Ledger, alive: bool) -> Result<Ledger> {
        let Some(feeding) = &self.port_feeding else { return Ok(holdings.clone()) };
        let intake = feeding.intake(&self.member, self.intake.ok_or("no intake is bound")?)?;
        feeding.offered(intake, holdings, alive, self.rules,
            self.lineages.ok_or("a port meal requires native source lineages")?)
    }

    pub(super) fn meal(&mut self, into: &Key, of: &[Key], whole: bool) -> Result<bool> {
        let Some(portion) = self.portion.take() else {
            return Err("the crowd eats only in a hunt".into());
        };
        let prey = self.prey.as_ref().ok_or("the crowd eats only in a hunt")?;
        let holdings = match self.bitten.and_then(|id| prey.parts.get(&id)) {
            Some(part) => &part.matter,
            None => &prey.accounts,
        };
        let offered = match self.admitted(&edible(holdings, of), prey.alive) {
            Ok(offered) => offered,
            Err(_) => return Ok(false),
        };
        if self.port_feeding.is_some() && offered.is_empty() { return Ok(false); }
        let whole_admitted = if whole && self.port_feeding.is_some() {
            let all = match self.admitted(holdings, prey.alive) {
                Ok(all) => all,
                Err(_) => return Ok(false),
            };
            holdings.iter().filter(|(_, n)| **n > 0).all(|(k, n)| all.get(k) == Some(n))
        } else { true };
        if whole && whole_admitted && self.whole(of, &portion)? { return Ok(true); }
        let prey = self.prey.as_mut().ok_or("the crowd eats only in a hunt")?;
        let held = match self.bitten.and_then(|id| prey.parts.get_mut(&id)) {
            Some(part) => &mut part.matter,
            None => &mut prey.accounts,
        };
        let taken: Ledger = portion.into_iter().map(|(k, v)| {
            let v = v.min(offered.get(&k).copied().unwrap_or(0));
            (k, v)
        }).filter(|(_, v)| *v > 0).collect();
        let mut total = 0u64;
        for (key, value) in &taken {
            debit(held, key, *value)?;
            total = total.checked_add(*value).ok_or("meal overflow")?;
        }
        credit(&mut self.member.accounts, into, total)?;
        self.took = Some(Took::Bite(taken));
        Ok(true)
    }
}
