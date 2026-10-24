// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graft;

impl Staged<'_> {
    pub(super) fn graft(&mut self, reserve: &Key, soil: &Key, cause: &str) -> Result<()> {
        let input = self.sim.graft_input.as_ref().ok_or("graft carrying needs an exact logged current input")?;
        let (actor, donor) = (self.stage.actor, self.stage.target.ok_or("graft donor is absent")?);
        if input.actor != actor || input.target != donor || input.process != self.stage.process
            || actor == donor || self.stage.count != 1 || self.stage.graft.is_some() {
            return Err("graft input does not name this single native operation".into());
        }
        let preview = graft::prepare(&self.stage.bodies[&actor], &self.stage.bodies[&donor],
            &input.carry, reserve, soil, &self.sim.genesis.rules, &self.sim.state.lineages)?;
        let source = &self.stage.bodies[&donor];
        let development = self.sim.state.lineages.get(&source.lineage)
            .and_then(|line| line.development.as_ref()).ok_or("graft donor development disappeared")?;
        let kinds: BTreeSet<_> = preview.branch.iter().filter_map(|part| {
            let [tagma,_,slot] = source.situs(*part)?;
            let tagma = development.recipe.tagmata.get(usize::from(tagma))?;
            if slot == 0 { Some(tagma.segment.clone()) } else { tagma.bears.clone() }
        }).collect();
        // A branch is indivisible. It asks every exact part/account and may
        // land only when the pass grants all of them; shortfall stays put.
        for part in &preview.branch {
            for (account, amount) in &self.stage.bodies[&donor].parts[part].matter {
                if *amount == 0 { continue; }
                let holder = Holder::Part(donor,*part);
                if let Some(demands) = &mut self.stage.demands {
                    demands.accounts.push((holder,account.clone(),*amount));
                    demands.graft_parts.insert(holder);
                } else if let Some(shares) = &mut self.stage.shares {
                    let left = shares.accounts.entry((holder,account.clone())).or_default();
                    if *left < *amount { return Err("graft branch did not receive its full pass-start stock grant".into()); }
                    *left -= *amount;
                }
            }
        }
        // The preview requires the recipient's own pre-transfer funds.
        // Pay before incoming retained titles could transiently look like
        // unowned reserve within this operation's capture.
        self.sim.state.projects.debit_capture.with(|context| { context.preserve = None; Ok(()) })?;
        if preview.reserve_cost_mg > 0 {
            self.take(Binding::Actor,reserve,preview.reserve_cost_mg)?;
            self.give(Binding::Place,soil,preview.reserve_cost_mg)?;
            if self.sim.flowing() { self.stage.legs.push(Leg {
                from:(Holder::Entity(actor),reserve.clone()),
                to:(Holder::Site(self.stage.place),soil.clone()),amount:preview.reserve_cost_mg,
            }); }
        }
        // Guards precede each real transfer, retaining same-account custody.
        for (old,new) in &preview.mapping {
            let holdings = self.stage.bodies[&donor].parts[old].matter.clone();
            for (account,amount) in holdings.into_iter().filter(|(_,n)| *n > 0) {
                self.sim.state.projects.debit_capture.with(|context| {
                    context.preserve = Some((crate::projects::StockSource::Entity(actor),account.clone()));
                    Ok(())
                })?;
                self.guard_owned_debit(Binding::Target,&account,amount)?;
                if self.sim.flowing() {
                    self.stage.legs.push(Leg { from:(Holder::Part(donor,*old),account.clone()),
                        to:(Holder::Part(actor,*new),account),amount });
                }
            }
            // Publish each part into the private stage before the next
            // guard reads actual balances for another part of that account.
            let mut source = self.stage.bodies.remove(&donor).ok_or("graft donor disappeared")?;
            let transfer = graft::transplant_part(self.actor(),Arc::make_mut(&mut source),&preview,*old,*new);
            self.stage.bodies.insert(donor,source);
            transfer?;
        }
        self.sim.state.projects.debit_capture.with(|context| { context.preserve = None; Ok(()) })?;
        let mut source = self.stage.bodies.remove(&donor).ok_or("graft donor disappeared")?;
        let transfer = graft::finish(self.actor(),Arc::make_mut(&mut source));
        self.stage.bodies.insert(donor,source);
        transfer?;
        for (_,part) in &preview.mapping {
            crate::validation::part(&self.sim.genesis.rules,&self.stage.bodies[&actor],*part)?;
        }
        let lineage = self.stage.bodies[&actor].lineage.clone();
        self.stage.lessons.extend(kinds.into_iter().map(|kind| (lineage.clone(),kind)));
        self.stage.graft = Some(graft::Record { actor,donor,action:cause.into(),
            process:self.stage.process.clone(),preview });
        Ok(())
    }
}
