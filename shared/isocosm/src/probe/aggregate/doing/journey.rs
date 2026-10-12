// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Own upkeep/birth continue off-site; site operations do not (820–823).

use super::*;

impl Doing<'_> {
    pub(super) fn site_reach(&self) -> Result<()> {
        if self.member.resident_site() == Some(self.site_id) { Ok(()) }
        else { Err("actor has no resident site".into()) }
    }

    /// Generic paid births retain exactly the native parent's form and
    /// remaining journey, clearing all but provisioned matter (TD6, 821).
    /// A completed prototype is hatched per represented member, without draws.
    pub(super) fn provisioned_birth(&mut self, provision: &Ledger) -> Result<bool> {
        let mut child = self.member.clone();
        child.accounts.clear();
        for part in child.parts.values_mut() { part.matter.clear(); }
        child.born = self.tick;
        child.arrived = self.tick;
        child.visits.clear();
        child.skills.clear();
        child.provenance = Provenance::Born(child.lineage.clone());
        child.authored = None;
        for (key, value) in provision {
            if self.take(Binding::Actor, key, *value).is_err() { return Ok(false); }
            match crate::anatomy::give(&mut child, self.rules, key, *value) {
                Some(Ok(_)) => {},
                Some(Err(_)) => return Ok(false),
                None => credit(&mut child.accounts, key, *value)?,
            }
        }
        self.born.push(Pending::Fragment { child });
        Ok(true)
    }

    /// Keep the same authored deposit amount and timing after arrival.
    /// This is the native staged rule, not an invented bulk deposit.
    pub(super) fn carried(&self, e: Effect) -> Option<Effect> {
        if self.member.journey.is_none() { return Some(e); }
        match e {
            Effect::Transfer { from: Binding::Actor, to: Binding::Place, ref account, .. }
                if account == "world:soil" => None,
            Effect::Spend { from, to: Binding::Place, amount, into: Some(ref key) }
                if key == "world:soil" => Some(Effect::Spend {
                    from, to: Binding::Actor, amount, into: Some(key.clone()),
                }),
            e => Some(e),
        }
    }
}
