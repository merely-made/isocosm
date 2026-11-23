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
pub(crate) const TABLE_SALT: &str = "isocosm/table/v1";
pub(crate) const RECORD_SALT: &str = "isocosm/table/records/v1";

pub(crate) const TERMS: InvitationTerms = InvitationTerms {
    domain: isonetry::family_network::invited::DOMAIN,
    salt: TABLE_SALT.as_bytes(),
    network_label: b"isocosm.table/v1",
    path_root: "/services/projection/isocosm-table",
};

/// The key the owner signs accepted records with; only its salt is read.
pub(crate) const RECORD_TERMS: InvitationTerms = InvitationTerms {
    salt: RECORD_SALT.as_bytes(),
    ..TERMS
};

/// A separately granted service using the same attested transport key as the seat.
pub(crate) const REPLICA_TERMS: InvitationTerms = InvitationTerms {
    path_root: "/services/projection/isocosm-full-replica",
    ..TERMS
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReplicaOffer {
    pub(crate) card: isonetry::family_network::invited::FamilyCard,
    pub(crate) grant: Invitation,
}

impl ReplicaOffer {
    /// The signed grant binds the full-replication scope, seat identity, owner and table.
    pub(crate) fn verify(&self, seat: &Invitation, peer: [u8; 32], now: u64)
    -> Result<p2panda_core::VerifyingKey, String> {
        self.grant.check(&REPLICA_TERMS)?;
        self.grant.guest_access(peer, false, false, now)?;
        let cert = &self.grant.grant.certificate;
        if self.grant.owner != seat.owner || self.grant.guest != seat.guest
            || self.grant.session != seat.session || cert.expires_at_ms.is_none()
            || now < cert.not_before_ms {
            return Err("replication grant does not cover this seat and table".into());
        }
        self.card.check(&TERMS, &RECORD_TERMS, seat)
    }
}

impl FamilyTable {
    /// The table's session as an invitation names it.
    pub(crate) fn invitation_session(&self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.host.session().0)
    }

    /// Grant `guest` this table, joining and binding a native participant for
    /// it in one accepted change. Re-inviting a bound guest returns its grant.
    pub(crate) fn invite(&mut self, signer: &dyn Signer, guest: Contact, now: u64, expires: u64)
    -> Result<(Invitation, cosmograph::schema::Id), String> {
        self.invite_player(signer, guest, None, now, expires)
    }

    /// The owner can bind an existing player participant when handing its character
    /// to an invited contact. The ordinary handoff creates a fresh participant.
    pub(crate) fn invite_player(&mut self, signer: &dyn Signer, guest: Contact,
        player: Option<cosmograph::schema::Id>, now: u64, expires: u64)
    -> Result<(Invitation, cosmograph::schema::Id), String> {
        guest.check(&TERMS)?;
        if signer.master_public_key().to_bytes() != self.owner_root(signer)? {
            return Err("only the table's owner identity invites".into());
        }
        if let Some(existing) = self.access.invitations.iter().find(|i| i.guest.binding == guest.binding) {
            let participant = self.participant_for_peer(guest.peer())
                .ok_or("an invited guest has lost its participant binding")?;
            if player.is_some_and(|selected| selected != participant) {
                return Err("this contact is already invited as another participant".into());
            }
            if existing.guest_access(guest.peer(), false, false, now).is_ok() {
                return Ok((existing.clone(), participant));
            }
            let renewed = Invitation::issue(signer, &TERMS, self.invitation_session(), guest, false, now, expires)?;
            let mut access = self.access.clone();
            let slot = access.invitations.iter_mut().find(|i| i.guest.peer() == renewed.guest.peer())
                .ok_or("the expiring seat grant is missing")?;
            *slot = renewed.clone();
            self.host.world().product_sections(self.host.world().cut()?, |_, sections| {
                let mut next = sections.clone();
                next.insert(ACCESS_SECTION.to_owned(), serde_json::to_vec(&access).map_err(|e| e.to_string())?);
                Ok(((), next))
            })?;
            self.access = access;
            return Ok((renewed, participant));
        }
        if self.access.peers.iter().any(|peer| peer.endpoint == guest.peer()) {
            return Err("this transport key is already bound to a participant".into());
        }
        let invitation = Invitation::issue(signer, &TERMS, self.invitation_session(), guest, false, now, expires)?;
        let mut access = self.access.clone();
        let before = self.host.world().cut()?;
        let participant = self.host.world().product_sections(before, |session, sections| {
            let participant = match player {
                Some(id) => {
                    if id == access.host_participant || access.peers.iter().any(|peer| peer.participant == id) {
                        return Err("the selected participant already belongs to the host or another peer".into());
                    }
                    if !session.sim.state().population.get(id).is_some_and(cosmograph::directing::is_participant) {
                        return Err("the selected player is not a native participant".into());
                    }
                    let game = sections.get(VTT_SECTION).ok_or("missing native VTT section")?;
                    if isocosm_native::native::GameBindings::decode(game)?.seats().is_some_and(|s| s.holds_gm(id)) {
                        return Err("the selected participant holds a GM grant".into());
                    }
                    id
                }
                None => {
                    let joined = session.command(cosmograph::Command::Join)?;
                    joined.strip_prefix("participant:").and_then(|id| id.parse().ok())
                        .ok_or("native join named no participant")?
                }
            };
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
    pub(super) fn owner_root(&self, signer: &dyn Signer) -> Result<[u8; 32], String> {
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
    fn an_owner_selected_player_binding_is_exclusive_and_cannot_replace_a_saved_seat() {
        let mut table = crate::campaign_store::bench_family_table();
        let owner = InMemoryProvider::from_seed([21; 32]);
        let guest = Contact::create(&InMemoryProvider::from_seed([22; 32]), &TERMS).unwrap();
        let used = table.access.peers[0].participant;
        let host = table.access.host_participant;
        let before = table.sequence();
        for id in [host, used, u64::MAX] {
            assert!(table.invite_player(&owner, guest.clone(), Some(id), 1_000, 2_000).is_err());
        }
        assert_eq!(table.sequence(), before);
        assert_eq!(table.participant_for_peer(guest.peer()), None);

        let joined = table.host.world().command(cosmograph::Command::Join).unwrap();
        let free = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        let before = table.sequence();
        let (grant, bound) = table.invite_player(&owner, guest.clone(), Some(free), 1_000, 2_000).unwrap();
        assert_eq!(bound, free);
        assert_eq!(table.invite_player(&owner, guest.clone(), Some(free), 1_000, 2_000).unwrap().0, grant);
        assert!(table.invite_player(&owner, guest, Some(used), 1_000, 2_000).is_err());
        let other = Contact::create(&InMemoryProvider::from_seed([23; 32]), &TERMS).unwrap();
        assert!(table.invite_player(&owner, other, Some(free), 1_000, 2_000).is_err());
        assert_eq!(table.sequence(), before + 1);
    }

    #[test]
    fn a_seat_without_a_saved_vtt_window_attaches_once_and_restores_its_handle() {
        let mut table = crate::campaign_store::bench_family_table();
        let owner = InMemoryProvider::from_seed([61; 32]);
        let guest = InMemoryProvider::from_seed([62; 32]);
        let (_, participant) = table.invite(&owner, Contact::create(&guest, &TERMS).unwrap(),
            1_000_000, 2_000_000).unwrap();
        let window = || table.host.read(|session, _| session.sim.window_for(participant, isocosm_native::MODE)).unwrap();
        assert!(window().is_none());
        let before = table.sequence();
        let attach = || VttEndpoint::for_table(table.host.world(), ParticipantHandle(participant), Seat::Player).unwrap();
        drop(attach());
        let original = window().expect("first attachment created a native window");
        assert_eq!(table.sequence(), before + 1);
        drop(attach());
        assert_eq!(window(), Some(original));
        assert_eq!(table.sequence(), before + 1);
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
        let renewed_contact = Contact::create(&guest, &TERMS).unwrap();
        let (renewed, bound) = table.invite(&owner, renewed_contact, 2_500_000, 3_500_000).unwrap();
        assert_eq!(bound, participant);
        assert!(table.admits(&principal(renewed.guest.peer(), &table), &[renewed.grant], 3_000_000).is_ok());
        assert_eq!(table.sequence(), before + 2);
        assert_eq!(table.invitation_count(), 1);
        let intruder = Contact::create(&stranger, &TERMS).unwrap();
        assert!(table.invite(&stranger, intruder, 1_000_000, 2_000_000).is_err(), "another owner");
    }
}
