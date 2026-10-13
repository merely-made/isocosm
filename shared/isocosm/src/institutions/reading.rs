// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnAct { pub attribution: ActWitness, pub event: Event }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionReading {
    pub polity: PolityId, pub definition: Option<Definition>, pub own_acts: Vec<OwnAct>,
    pub proposals: Vec<Proposal>, pub ready: BTreeSet<Key>, pub refusals: BTreeMap<Key,String>,
}

impl Simulation {
    /// Trusted reading only. A consumer applies its native seat/knowing cut.
    /// Member acts do not become institution acts merely by numeric identity.
    pub fn institution_reading(&self, id:PolityId) -> Result<InstitutionReading> {
        let polity = self.state.polities.get(&id.0).ok_or("unknown native institution")?;
        let proposals: Vec<_> = self.state.institutions.proposals.values().filter(|p| p.polity == id).cloned().collect();
        let mut own_acts = Vec::new();
        let mut seen = BTreeSet::new();
        let mut read = |witness:&ActWitness| -> Result<()> {
            let event = self.state.events.get(&witness.event).ok_or("institution act event is absent")?;
            if witness.polity != id || event.authored.is_some() || event.subject != witness.actor
                || event.process != witness.process {
                return Err("institution attribution differs from its native body event".into());
            }
            if seen.insert(event.id.clone()) { own_acts.push(OwnAct { attribution:witness.clone(),event:event.clone() }); }
            Ok(())
        };
        for proposal in &proposals {
            for vote in proposal.votes.values() { if let Some(act) = &vote.act { read(act)?; } }
            if let Some(done) = &proposal.applied { read(&done.act)?; }
        }
        // Derived founding points at an actual body event. The authored
        // "assert:key" cause is retained as authored data, never invented as a body act.
        if let Some(event) = self.state.events.get(&polity.constitution.founded_by).filter(|e| e.authored.is_none()) {
            read(&ActWitness { polity:id,actor:event.subject,event:event.id.clone(),process:event.process.clone(),kind:ActKind::Foundation })?;
        }
        own_acts.sort_by(|a,b| (a.event.tick,&a.event.id).cmp(&(b.event.tick,&b.event.id)));
        let mut ready = BTreeSet::new();
        let mut refusals = BTreeMap::new();
        for proposal in &proposals {
            if proposal.applied.is_some() { continue; }
            match self.institution_current(proposal).and_then(|_| self.institution_support(&proposal.operation)) {
                Ok(()) if self.institution_ready(proposal) => { ready.insert(proposal.key.clone()); },
                Ok(()) => {}, Err(why) => { refusals.insert(proposal.key.clone(),why); },
            }
        }
        Ok(InstitutionReading { polity:id,definition:self.genesis.rules.institutions.get(&polity.constitution.governance).cloned(),
            own_acts,proposals,ready,refusals })
    }
}
