// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One admitted bite plus only its real previous ingesta is digested.
//! Processing may exceed storage capacity; the final residue may not.

use super::*;

impl Staged<'_> {
    pub(super) fn partition_meal(
        &mut self,
        meal: &MealPartition,
        into: &Key,
        amount: u64,
    ) -> Result<()> {
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let intake = self
            .stage
            .intake
            .ok_or("partitioned meal has no intake part")?;
        let actor = self.stage.actor;
        let source = match rules.accounts.get(&meal.ingesta) {
            Some(AccountKind::Matter {
                lineage,
                reserve: false,
                provision: false,
            }) => lineage,
            _ => return Err("ingesta is not foreign unreserved matter".into()),
        };
        let retained = match rules.accounts.get(into) {
            Some(AccountKind::Matter {
                lineage,
                reserve: false,
                provision: false,
            }) => lineage,
            _ => return Err("retained output is not tissue".into()),
        };
        if source == &self.stage.bodies[&actor].lineage || source == retained {
            return Err("ingesta cannot borrow the body's own or inherited tissue".into());
        }
        let previous = self.stage.bodies[&actor]
            .parts
            .get(&intake)
            .ok_or("intake part missing")?
            .matter
            .get(&meal.ingesta)
            .copied()
            .unwrap_or(0);
        if previous >= meal.units {
            return Err("intake residue exceeds its partition bound".into());
        }
        let total = previous.checked_add(amount).ok_or("ingesta overflow")?;
        let groups = total / meal.units;
        let residue = total % meal.units;
        let kept = groups
            .checked_mul(meal.kept)
            .ok_or("retained meal overflow")?;
        let returned = groups
            .checked_mul(meal.units - meal.kept)
            .ok_or("meal waste overflow")?;
        self.guard_owned_debit(Binding::Actor, &meal.ingesta, previous)?;
        if previous > 0 {
            debit(
                &mut self
                    .actor()
                    .part_mut(&intake)
                    .ok_or("intake part missing")?
                    .matter,
                &meal.ingesta,
                previous,
            )?;
        }
        if kept > 0 {
            self.give(Binding::Actor, into, kept)?;
        }
        if returned > 0 {
            self.give(Binding::Place, &meal.waste, returned)?;
        }
        let body = &self.stage.bodies[&actor];
        let part = body.parts.get(&intake).ok_or("intake part missing")?;
        if crate::anatomy::space(body.extent(intake), part, rules, &meal.ingesta) < residue {
            return Err("final ingesta residue does not fit the actual intake part".into());
        }
        if residue > 0 {
            credit(
                &mut self.actor().part_mut(&intake).unwrap().matter,
                &meal.ingesta,
                residue,
            )?;
        }
        if self.sim.flowing() {
            let from = (Holder::Part(actor, intake), meal.ingesta.clone());
            if kept > 0 {
                self.stage.legs.push(Leg {
                    from: from.clone(),
                    to: (Holder::Entity(actor), into.clone()),
                    amount: kept,
                });
            }
            if returned > 0 {
                self.stage.legs.push(Leg {
                    from,
                    to: (Holder::Site(self.stage.place), meal.waste.clone()),
                    amount: returned,
                });
            }
        }
        self.sim.diagnostic_partition(
            previous,
            residue,
            &Ledger::from([(into.clone(), kept), (meal.waste.clone(), returned)]),
        );
        Ok(())
    }
}
