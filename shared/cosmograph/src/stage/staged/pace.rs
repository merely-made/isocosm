// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Staged<'_> {
    pub(super) fn paced_move(&mut self, to: Id, name: &str, cause: &str) -> Result<()> {
        let sim = self.sim;
        if sim.state.movement.immediate() { return Err("paced travel requires native journeys".into()); }
        let actor = &self.stage.bodies[&self.stage.actor];
        if actor.resident_site() != Some(self.stage.place) {
            return Err("movement actor has no resident site".into());
        }
        let site = sim.site_at_start(self.stage.place).ok_or("departure site absent")?;
        let route = site.routes.iter().find(|route| route.to == to).ok_or("no route to destination")?;
        let profile = sim.genesis.rules.pace_profiles.get(name).ok_or("unknown pace profile")?;
        let journey = crate::journey::departure(actor, site, to, sim.state.tick, profile.duration(route.travel)?)?;
        for cost in &profile.costs {
            // Spend is ordinarily an up-to take. A pace owes its full authored
            // cost, checked against the stage after every preceding cost.
            let mut held = 0u128;
            for key in &cost.from {
                held = held.checked_add(u128::from(Parties::held(self, Binding::Actor, key)?))
                    .ok_or("pace source balances overflow")?;
            }
            if held < u128::from(cost.amount) { return Err("insufficient stocks for authored pace".into()); }
            self.effect(&cost.effect(), cause)?;
        }
        // All debits, anatomy routing, title admission and departure share the
        // disposable native stage; any refusal publishes none of them.
        crate::journey::leave(self.actor(), journey);
        Ok(())
    }
}
