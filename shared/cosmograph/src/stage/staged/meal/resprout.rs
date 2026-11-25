// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Staged<'_> {
    /// Refill a partially grazed fixing organ from storage on the same living
    /// body. The store received its share through ordinary tissue allocation.
    /// Use only the amount, provenance and space actually lost in this bite.
    pub(super) fn resprout(
        &mut self,
        from: Binding,
        id: Id,
        leaf: PartId,
        taken: &Ledger,
    ) -> Result<()> {
        let sim = self.sim;
        let rules = &sim.genesis.rules;
        let body = self.body(from)?;
        if taken.is_empty()
            || !body.alive
            || !body.lives(leaf)
            || body.parts[&leaf]
                .cells
                .get("function:fix")
                .copied()
                .unwrap_or(0)
                == 0
        {
            return Ok(());
        }
        let stores: Vec<_> = body
            .living()
            .filter(|(part, p)| {
                *part != leaf
                    && p.port.is_some_and(|port| port.resprouts())
                    && p.cells.get(crate::anatomy::STORE).copied().unwrap_or(0) > 0
            })
            .map(|(part, _)| part)
            .collect();
        if stores.is_empty() {
            return Ok(());
        }

        let mut legs = Vec::new();
        for (key, &lost) in taken {
            if !matches!(
                rules.accounts.get(key),
                Some(AccountKind::Matter {
                    lineage,
                    reserve: false,
                    provision: false,
                }) if sim.state.lineages.get(lineage).is_some_and(|l| l.kingdom != "kingdom:world")
            ) {
                continue;
            }
            let mut remaining = lost.min(crate::anatomy::space(
                body.extent(leaf),
                &body.parts[&leaf],
                rules,
                key,
            ));
            for &store in &stores {
                if remaining == 0 {
                    break;
                }
                let held = body.parts[&store].matter.get(key).copied().unwrap_or(0);
                let amount = held.min(remaining);
                if amount == 0 {
                    continue;
                }
                debit(&mut body.part_mut(&store).unwrap().matter, key, amount)?;
                credit(&mut body.part_mut(&leaf).unwrap().matter, key, amount)?;
                if sim.flowing() {
                    legs.push(Leg {
                        from: (Holder::Part(id, store), key.clone()),
                        to: (Holder::Part(id, leaf), key.clone()),
                        amount,
                    });
                }
                remaining -= amount;
            }
        }
        self.stage.legs.extend(legs);
        Ok(())
    }
}
