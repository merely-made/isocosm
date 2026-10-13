// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnAct { pub attribution: ActWitness, pub event: Event }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionReading {
    pub polity: PolityId, pub definition: Option<Definition>, pub own_acts: Vec<OwnAct>,
    pub proposals: Vec<Proposal>, pub ready: BTreeSet<Key>, pub refusals: BTreeMap<Key,String>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub body_proposals: Vec<BodyProposal>,
    #[serde(default,skip_serializing_if="BTreeMap::is_empty")]
    pub body_votes: BTreeMap<Key,BTreeMap<Id,BodyVote>>,
    #[serde(default,skip_serializing_if="BTreeMap::is_empty")]
    pub body_applied: BTreeMap<Key,BodyApplied>,
}

impl Simulation {
    /// Trusted reading only. A consumer applies its native seat/knowing cut.
    /// Member acts do not become institution acts merely by numeric identity.
    pub fn institution_reading(&self, id:PolityId) -> Result<InstitutionReading> {
        let polity = self.state.polities.get(&id.0).ok_or("unknown native institution")?;
        let proposals: Vec<_> = self.state.institutions.proposals.values().filter(|p| p.polity == id).cloned().collect();
        let body_proposals:Vec<_>=self.state.institutions.body_proposals.values().filter(|p|p.core.polity==id).cloned().collect();
        let keys:BTreeSet<_>=proposals.iter().map(|p|p.key.clone()).chain(body_proposals.iter().map(|p|p.core.key.clone())).collect();
        let body_votes:BTreeMap<_,_>=self.state.institutions.body_votes.iter().filter(|(key,_)|keys.contains(*key))
            .map(|(key,votes)|(key.clone(),votes.clone())).collect();
        let body_applied:BTreeMap<_,_>=self.state.institutions.body_applied.iter().filter(|(key,_)|keys.contains(*key))
            .map(|(key,act)|(key.clone(),act.clone())).collect();
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
        for proposal in &body_proposals {
            read(&proposal.act)?;
            for vote in proposal.votes.values() {if let Some(act)=&vote.act {read(act)?;}}
            if let Some(done)=&proposal.applied {read(&done.act)?;}
        }
        for vote in body_votes.values().flat_map(|votes|votes.values()) {read(&vote.act)?;}
        for done in body_applied.values() {read(&done.act)?;}
        // Derived founding points at an actual body event. The authored
        // "assert:key" cause is retained as authored data, never invented as a body act.
        if let Some(event) = self.state.events.get(&polity.constitution.founded_by).filter(|e| e.authored.is_none()) {
            read(&ActWitness { polity:id,actor:event.subject,event:event.id.clone(),process:event.process.clone(),kind:ActKind::Foundation })?;
        }
        own_acts.sort_by(|a,b| (a.event.tick,&a.event.id).cmp(&(b.event.tick,&b.event.id)));
        let mut ready = BTreeSet::new();
        let mut refusals = BTreeMap::new();
        for key in &keys {
            let proposal=self.state.institutions.core(key)?;
            if self.state.institutions.is_applied(key)? {continue;}
            match self.institution_current(&proposal).and_then(|_| self.institution_support(&proposal.operation)) {
                Ok(()) if self.institution_ready(&proposal) => { ready.insert(proposal.key.clone()); },
                Ok(()) => {}, Err(why) => { refusals.insert(proposal.key.clone(),why); },
            }
        }
        Ok(InstitutionReading { polity:id,definition:self.genesis.rules.institutions.get(&polity.constitution.governance).cloned(),
            own_acts,proposals,ready,refusals,body_proposals,body_votes,body_applied })
    }
}
