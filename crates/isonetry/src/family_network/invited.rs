//! Invited peers on the family session (isocosm case 1060, mere stack-lift D4).
//!
//! The owner's identity, never a product key, makes everything here: the
//! transport key and the record-signing key are keys it derives under the
//! product's salts, and a guest reaches the table only with the owner's
//! invitation (`graphshell_host::invitation`). What travels out of band is a
//! [`FamilyCard`] (the owner's contact and ticket, its attested record key and
//! the checkpoint to bootstrap from) and the guest's own invitation; the
//! checkpoint bytes travel after admission.

use super::FamilyCheckpointDescriptor;
use graphshell_native::{
    admission::{CONNECT_ACTION, GRAPHSHELL_DOMAIN, PROJECTION_SERVICE},
    carrier::{projection_alpn, projection_policy},
    invitation::{Card, Contact, Invitation, InvitationTerms},
    network_carrier::dial_projection_session,
};
use notochord::{AdmittedPrincipal, LocalNetworkPolicy, NetworkId, ProfileRef, TrustedRoot};
use p2panda_core::{SigningKey, VerifyingKey};
use personae::IdentityProvider;
use serde::{Deserialize, Serialize};
use transport::{P2pandaTransport, PeerID, Transport};

/// Graphshell's projection service: an invited family peer is admitted to it
/// under the product's path, as Graphshell co-op's guests are.
pub const DOMAIN: &str = GRAPHSHELL_DOMAIN;

/// The base profile the admitted carrier speaks.
pub fn profile() -> ProfileRef {
    ProfileRef { id: "mere.base".into(), revision: 1 }
}

/// Bind the transport on the key `identity` derives under `terms.salt`, so a
/// contact's attested key is the peer id a guest dials and admission reads.
pub async fn bind(identity: &dyn IdentityProvider, terms: &InvitationTerms)
-> Result<P2pandaTransport, String> {
    let key = identity.derived_keypair(terms.salt).map_err(|e| e.to_string())?;
    P2pandaTransport::builder_for(&key).alpns(vec![projection_alpn()]).gossip().bind().await
        .map_err(|e| e.to_string())
}

/// The owner's record-signing key, derived under `records.salt`.
pub fn record_key(identity: &dyn IdentityProvider, records: &InvitationTerms) -> Result<SigningKey, String> {
    let key = identity.derive_keypair(records.salt).map_err(|e| e.to_string())?;
    Ok(SigningKey::from_bytes(&key.to_seed()))
}

/// The accepted-record topic: the invitation's network id for this session.
pub fn topic(terms: &InvitationTerms, owner: [u8; 32], session: uuid::Uuid) -> [u8; 32] {
    terms.network(owner, session)
}

/// The owner's admission policy: its own root is the one trusted issuer.
pub fn policy(terms: &InvitationTerms, owner: [u8; 32], session: uuid::Uuid) -> LocalNetworkPolicy {
    projection_policy(NetworkId(terms.network(owner, session)),
        vec![TrustedRoot { authority: owner, issuer: owner }], vec![profile()], Some(8))
}

/// The transport-side action check: connect to this session's path. The
/// owner thread still checks the stored invitation before serving.
pub fn serves(terms: &InvitationTerms, session: uuid::Uuid, principal: &AdmittedPrincipal) -> bool {
    principal.action.domain == DOMAIN
        && principal.action.path == terms.path(session)
        && principal.action.action == CONNECT_ACTION
        && terms.path(session).starts_with(PROJECTION_SERVICE)
}

/// What the owner hands every guest: its card, its attested record key and
/// the signed checkpoint a guest bootstraps from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyCard {
    pub version: u16,
    pub card: Card,
    pub records: Contact,
    pub checkpoint: FamilyCheckpointDescriptor,
}

impl FamilyCard {
    /// Check the card against the guest's invitation and return the pinned
    /// record key: the owner attested it, and it names this session.
    pub fn check(&self, terms: &InvitationTerms, records: &InvitationTerms, invitation: &Invitation)
    -> Result<VerifyingKey, String> {
        self.card.check(terms, invitation)?;
        self.records.check(records)?;
        if self.version != 1 || self.records.root() != self.card.owner.root()
            || self.checkpoint.family_session.0 != *invitation.session.as_bytes() {
            return Err("the family card names another owner or session".into());
        }
        VerifyingKey::from_bytes(&self.records.peer()).map_err(|e| e.to_string())
    }
}

/// The guest's dial: reach the card's holder from its ticket, check it is the
/// owner, and open an admitted stream with the invitation's hello.
pub async fn dial(transport: &P2pandaTransport, identity: &dyn IdentityProvider, terms: &InvitationTerms,
    invitation: &Invitation, card: &Card) -> Result<(PeerID, <P2pandaTransport as Transport>::Stream), String> {
    let peer = transport.add_peer_ticket(&card.ticket).await.map_err(|e| e.to_string())?;
    if peer.to_bytes() != card.owner.peer() {
        return Err("the card's ticket names another holder".into());
    }
    let hello = invitation.hello(terms, identity, profile(), transport.local_peer_id())?;
    let stream = dial_projection_session(transport, peer, &hello, &Default::default()).await
        .map_err(|e| e.to_string())?
        .map_err(|reason| format!("admission refused: {reason:?}"))?;
    Ok((peer, stream))
}
