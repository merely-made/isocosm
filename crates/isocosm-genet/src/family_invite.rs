//! Inviting a peer to the table (isocosm case 1060, mere stack-lift D4).
//!
//! The owner's custody identity grants one guest's contact the table's
//! session through `graphshell_host::invitation`; the same accepted change
//! joins a native participant for that guest and binds it to the guest's
//! transport key, so the peer's seat comes from its admitted key and never
//! from what the peer says. Admission rechecks the stored invitation.

use super::*;
use graphshell_host::invitation::{Contact, Invitation, InvitationTerms, Signer};
use isonetry::native_table::PeerParticipant;

/// Wall-clock milliseconds, as invitation expiry reads them.
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
}

/// The table's admission terms over the lifted invitation.
pub(crate) const TERMS: InvitationTerms = InvitationTerms {
    domain: isonetry::family_network::invited::DOMAIN,
    salt: b"isocosm/table/v1",
    network_label: b"isocosm.table/v1",
    path_root: "/services/projection/isocosm-table",
};

/// The key the owner signs accepted records with; only its salt is read.
pub(crate) const RECORD_TERMS: InvitationTerms = InvitationTerms {
    salt: b"isocosm/table/records/v1",
    ..TERMS
};

impl FamilyTable {
    /// The table's session as an invitation names it.
    pub(crate) fn invitation_session(&self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.host.session().0)
    }

    /// Grant `guest` this table, joining and binding a native participant for
    /// it in one accepted change. Re-inviting a bound guest returns its grant.
    pub(crate) fn invite(&mut self, signer: &dyn Signer, guest: Contact, now: u64, expires: u64)
    -> Result<(Invitation, cosmograph::schema::Id), String> {
        guest.check(&TERMS)?;
        if signer.master_public_key().to_bytes() != self.owner_root(signer)? {
            return Err("only the table's owner identity invites".into());
        }
        if let Some(existing) = self.access.invitations.iter().find(|i| i.guest.binding == guest.binding) {
            let participant = self.participant_for_peer(guest.peer())
                .ok_or("an invited guest has lost its participant binding")?;
            return Ok((existing.clone(), participant));
        }
        if self.access.peers.iter().any(|peer| peer.endpoint == guest.peer()) {
            return Err("this transport key is already bound to a participant".into());
        }
        let invitation = Invitation::issue(signer, &TERMS, self.invitation_session(), guest, false, now, expires)?;
        let mut access = self.access.clone();
        let before = self.host.world().cut()?;
        let participant = self.host.world().product_sections(before, |session, sections| {
            let joined = session.command(cosmograph::Command::Join)?;
            let participant: cosmograph::schema::Id = joined.strip_prefix("participant:")
                .and_then(|id| id.parse().ok()).ok_or("native join named no participant")?;
            access.peers.push(PeerParticipant { endpoint: invitation.guest.peer(), participant });
            access.invitations.push(invitation.clone());
            let mut next = sections.clone();
            next.insert(ACCESS_SECTION.to_owned(), serde_json::to_vec(&access).map_err(|e| e.to_string())?);
            Ok((participant, next))
        })?;
        self.access = access;
        Ok((invitation, participant))
    }

    #[cfg(test)]
    pub(crate) fn invitation_count(&self) -> usize { self.access.invitations.len() }

    #[cfg(test)]
    pub(crate) fn world_save(&self) -> Result<cosmograph_host::save::WorldSave, String> { self.host.world().save() }

    /// The owner's admission check for an admitted peer: an unrevoked stored
    /// invitation names it and its grant, and it has a participant binding.
    pub(crate) fn admits(&self, principal: &notochord::AdmittedPrincipal,
        chain: &[insigne::delegation::SignedDelegationCertificate], now: u64)
    -> Result<cosmograph::schema::Id, String> {
        let invitation = self.access.invitations.iter().find(|i| i.guest.peer() == principal.subject)
            .ok_or("this peer holds no invitation to the table")?;
        if invitation.session != self.invitation_session() {
            return Err("the invitation names another table".into());
        }
        invitation.admits(&TERMS, principal, chain, false, now)?;
        self.participant_for_peer(principal.subject).ok_or_else(|| "the invited peer has no participant".into())
    }

    /// The owner every invitation must share: the first invitation's issuer,
    /// or the signer itself before any exists.
    fn owner_root(&self, signer: &dyn Signer) -> Result<[u8; 32], String> {
        Ok(self.access.invitations.first().map_or_else(|| signer.master_public_key().to_bytes(),
            |first| first.owner.root()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use personae::InMemoryProvider;

    fn principal(subject: [u8; 32], table: &FamilyTable) -> notochord::AdmittedPrincipal {
        notochord::AdmittedPrincipal { subject, class: notochord::TrafficClass::Interactive,
            session_id: [0; 32], action: notochord::RequestedAction { domain: TERMS.domain.into(),
                path: TERMS.path(table.invitation_session()), action: "connect".into() } }
    }

    #[test]
    fn an_invited_guest_is_admitted_as_the_participant_its_invitation_bound() {
        let mut table = crate::campaign_store::bench_family_table();
        let owner = InMemoryProvider::from_seed([31; 32]);
        let guest = InMemoryProvider::from_seed([32; 32]);
        let stranger = InMemoryProvider::from_seed([33; 32]);
        let before = table.sequence();
        let contact = Contact::create(&guest, &TERMS).unwrap();
        let (invitation, participant) = table.invite(&owner, contact.clone(), 1_000_000, 2_000_000).unwrap();
        assert_eq!(table.sequence(), before + 1, "join and binding are one accepted record");
        assert_eq!(table.participant_for_peer(contact.peer()), Some(participant));
        assert!(table.host.read(|session, _| session.sim.state().population.get(participant)
            .is_some_and(cosmograph::directing::is_participant)).unwrap());
        let chain = [invitation.grant.clone()];
        assert_eq!(table.admits(&principal(contact.peer(), &table), &chain, 1_500_000), Ok(participant));
        let other = Contact::create(&stranger, &TERMS).unwrap();
        assert!(table.admits(&principal(other.peer(), &table), &chain, 1_500_000).is_err());
        assert!(table.admits(&principal(contact.peer(), &table), &chain, 2_500_000).is_err(), "expired");
        assert!(table.admits(&principal(contact.peer(), &table), &[], 1_500_000).is_err(), "no grant");
        let again = table.invite(&owner, contact, 1_000_000, 2_000_000).unwrap();
        assert_eq!(again, (invitation, participant), "re-inviting returns the same grant and seat");
        assert_eq!(table.sequence(), before + 1);
        let intruder = Contact::create(&stranger, &TERMS).unwrap();
        assert!(table.invite(&stranger, intruder, 1_000_000, 2_000_000).is_err(), "another owner");
    }
}
