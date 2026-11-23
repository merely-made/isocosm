//! Hosting the table for invited peers (isocosm case 1060).
//!
//! The transport and record keys are the owner's custody identity's derived
//! keys. The owner publishes a family card into a handoff folder; a guest's
//! contact file dropped there is invited (one accepted record joins and
//! binds its participant) and its invitation written back beside it, so the
//! files can be carried to another machine. Admitted sessions are served
//! through the owner thread's request pump. A separately selected replica
//! contact may also receive the signed full-world checkpoint and record log.

use super::*;
use super::invite::{RECORD_TERMS, REPLICA_TERMS, ReplicaOffer, TERMS, now_ms};
use graphshell_host::invitation::{Card, Contact, Invitation, Signer};
use isonetry::family_network::{
    FamilyRecordAccess, FamilyHostNetwork, FamilyRecordBatch, FamilyRecordOutbox,
    MAX_FAMILY_RECORD_BATCH_BYTES,
    invited::{self, FamilyCard},
    stream::{carrier, lifecycle},
};
use std::path::PathBuf;
use transport::Transport;

type Stream = <transport::P2pandaTransport as Transport>::Stream;

pub(crate) const CARD_FILE: &str = "table.card.json";
const CONTACT_SUFFIX: &str = ".contact.json";
const INVITATION_SUFFIX: &str = ".invitation.json";
pub(crate) const INVITATION_LIFETIME_MS: u64 = 7 * 24 * 60 * 60 * 1000;
const MAX_HANDOFF_BYTES: u64 = 65_536;

/// The owner's hosting of one table.
pub(crate) struct HostNet {
    runtime: tokio::runtime::Runtime,
    signer: Arc<dyn Signer + Send + Sync>,
    transport: Arc<transport::P2pandaTransport>,
    handoff: PathBuf,
    handoff_dirty: Arc<std::sync::atomic::AtomicBool>,
    admitted: std::sync::mpsc::Receiver<notochord::AdmittedSession<Stream>>,
    outbox: Option<FamilyRecordOutbox>,
    record_topic: Option<([u8; 32], FamilyRecordAccess)>,
}

pub(crate) fn replica_file(contact: &Contact) -> String {
    let key: String = contact.peer().iter().map(|byte| format!("{byte:02x}")).collect();
    format!("guest-{key}.replica.json")
}

pub(crate) fn write_json(path: &std::path::Path, value: &impl serde::Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let partial = path.with_extension("json.tmp");
    std::fs::write(&partial, bytes).map_err(|e| format!("{}: {e}", partial.display()))?;
    std::fs::rename(&partial, path).map_err(|e| format!("{}: {e}", path.display()))
}

pub(crate) fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, String> {
    let size = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?.len();
    if size > MAX_HANDOFF_BYTES {
        return Err(format!("{} is too large for a handoff file", path.display()));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

impl FamilyTable {
    /// Publish the seat card and begin admission. An explicitly selected contact
    /// additionally receives its own signed replica offer and authenticated log lane.
    pub(crate) fn host_peers(&mut self, signer: Arc<dyn Signer + Send + Sync>, handoff: PathBuf,
        replica_contact: Option<Contact>, wake: Arc<dyn Fn() + Send + Sync>) -> Result<HostNet, String> {
        std::fs::create_dir_all(&handoff).map_err(|e| format!("{}: {e}", handoff.display()))?;
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()
            .map_err(|e| e.to_string())?;
        let identity: &dyn personae::IdentityProvider = &*signer;
        let owner = signer.master_public_key().to_bytes();
        if self.owner_root(&*signer)? != owner { return Err("only the table's recorded owner can host it".into()); }
        let transport = Arc::new(runtime.block_on(invited::bind(identity, &TERMS, replica_contact.is_some()))?);
        let session = self.invitation_session();
        let card = Card { version: 1, session, owner: Contact::create(identity, &TERMS)?,
            ticket: runtime.block_on(transport.ticket()).map_err(|e| e.to_string())? };
        self.checkpoint = None;
        let mut record_topic = None;
        let outbox = if let Some(contact) = replica_contact {
            let now = now_ms();
            let (seat, _) = self.invite(&*signer, contact.clone(), now, now + INVITATION_LIFETIME_MS)?;
            let expires = seat.grant.certificate.expires_at_ms.ok_or("seat invitation has no expiry")?;
            let grant = Invitation::issue(&*signer, &REPLICA_TERMS, session, contact.clone(), false, now, expires)?;
            let access = FamilyRecordAccess::until(contact.peer(), expires);
            let topic = invited::topic(&TERMS, owner, session);
            let network = runtime.block_on(FamilyHostNetwork::join(&transport,
                murm_replication::MunimentStore::new(muniment::MemoryBackend::new()), self.host.session(), topic,
                invited::record_key(identity, &RECORD_TERMS)?, access, isonetry::family::WireLimits::default()))
                .map_err(|e| e.to_string())?;
            if self.durable {
                let batch = FamilyRecordBatch::persisted(self.host.persisted_suffix_after(0)?)?;
                runtime.block_on(network.publish_batch(&batch))?;
            }
            let (outbox, mut batches) = FamilyRecordOutbox::channel(8, MAX_FAMILY_RECORD_BATCH_BYTES)?;
            if outbox.try_forward(&self.host)? > 0 {
                let batch = batches.try_recv().map_err(|e| e.to_string())?;
                runtime.block_on(network.publish_batch(&batch))?;
            }
            let resource = runtime.block_on(network.signed_checkpoint(self.host.checkpoint()?,
                chirograph::ProjectionSession("isocosm-family-checkpoint".into())))?;
            let offer = ReplicaOffer { grant, card: FamilyCard { version: 1, card: card.clone(),
                records: Contact::create(identity, &RECORD_TERMS)?, checkpoint: resource.descriptor().clone() } };
            write_json(&handoff.join(replica_file(&contact)), &offer)?;
            self.checkpoint = Some((access, resource));
            record_topic = Some((topic, access));
            runtime.spawn(async move {
                let expiry = tokio::time::sleep(std::time::Duration::from_millis(expires.saturating_sub(now_ms())));
                tokio::pin!(expiry);
                loop {
                    tokio::select! {
                        biased;
                        _ = &mut expiry => break,
                        batch = batches.recv() => {
                            let Some(batch) = batch else { break; };
                            if let Err(error) = network.publish_batch(&batch).await {
                                eprintln!("[isocosm-family] record publication failed: {error}");
                                break;
                            }
                        }
                    }
                }
                let _ = network.leave_and_wait().await;
            });
            Some(outbox)
        } else { None };
        write_json(&handoff.join(CARD_FILE), &card)?;
        // External handoffs can arrive while the window is asleep. All local writers
        // atomically replace files, and copied files can also change in place.
        let handoff_dirty = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let dirty = handoff_dirty.clone();
        let watched = handoff.clone();
        let handoff_wake = Arc::clone(&wake);
        runtime.spawn(async move {
            let mut previous = BTreeMap::new();
            loop {
                let current: BTreeMap<_, _> = std::fs::read_dir(&watched).into_iter().flatten().flatten()
                    .filter_map(|entry| {
                        let name = entry.file_name();
                        let text = name.to_str()?;
                        if !(text.ends_with(CONTACT_SUFFIX) || text.ends_with(INVITATION_SUFFIX)) { return None; }
                        let metadata = entry.metadata().ok()?;
                        Some((name, (metadata.len(), metadata.modified().ok()?)))
                    }).collect();
                if current != previous {
                    dirty.store(true, std::sync::atomic::Ordering::Release);
                    handoff_wake();
                    previous = current;
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });
        self.publication_wake = Some(Arc::clone(&wake));
        let (admitted_tx, admitted) = std::sync::mpsc::channel();
        let accepting = Arc::clone(&transport);
        let policy = invited::policy(&TERMS, owner, session);
        runtime.spawn(async move {
            let check = move |principal: &notochord::AdmittedPrincipal, _: &[_]| invited::serves(&TERMS, session, principal);
            loop {
                let accepted = carrier::accept_projection_session_with(accepting.as_ref(), &policy,
                    &notochord::RevocationLedger::default(), now_ms(), 0, &check).await;
                match accepted {
                    Ok(Ok(session)) => {
                        if admitted_tx.send(session).is_err() { break; }
                        wake();
                    },
                    Ok(Err(refusal)) => eprintln!("[isocosm-family] peer refused: {refusal:?}"),
                    Err(error) => {
                        eprintln!("[isocosm-family] accept stopped: {error}");
                        break;
                    },
                }
            }
        });
        Ok(HostNet { runtime, signer, transport, handoff, handoff_dirty, admitted, outbox, record_topic })
    }

    /// The signed checkpoint, served to an admitted peer that asks for exactly it.
    pub(crate) fn checkpoint_answer(&self, peer: [u8; 32], call: &EndpointCall) -> Option<Result<EndpointAnswer, String>> {
        let (access, resource) = self.checkpoint.as_ref()?;
        if !access.permits(peer, now_ms()) { return None; }
        match call {
            EndpointCall::Resource(request) => resource.respond(request).map(|r| Ok(EndpointAnswer::Resource(r))),
            EndpointCall::ResourceChunk(chunk) if chunk.session == resource.descriptor().projection_session
                && chunk.resource == resource.descriptor().resource => Some(Ok(EndpointAnswer::ResourceChunk(
                chirograph::ResourceChunkResponse::slice(resource.response(), chunk.offset, chunk.length)))),
            _ => None,
        }
    }
}

impl HostNet {
    /// The owner thread's turn: invite new contacts, serve newly admitted
    /// sessions, and hand accepted records to the publisher. Returns the new
    /// sessions' request pumps and anything worth showing.
    pub(crate) fn tick(&mut self, table: &mut FamilyTable, wake: Arc<dyn Fn() + Send + Sync>)
    -> (Vec<AdmittedRequestPump>, Vec<String>) {
        let mut notes = Vec::new();
        if self.handoff_dirty.swap(false, std::sync::atomic::Ordering::AcqRel) {
            if let Err(error) = self.invite_contacts(table, &mut notes) { notes.push(error); }
        }
        let mut pumps = Vec::new();
        while let Ok(mut session) = self.admitted.try_recv() {
            match table.admitted_endpoint(&session, Arc::clone(&wake)) {
                Ok((mut bridge, pump)) => {
                    if let Some((topic, _)) = self.record_topic.filter(|(_, access)| access.permits(session.principal.subject, now_ms())) {
                        if let Ok(peer) = transport::PeerID::from_bytes(&session.principal.subject) {
                            let _ = self.runtime.block_on(self.transport.set_topics(peer, &[transport::sync_overlay_topic(topic)]));
                        }
                    }
                    pumps.push(pump);
                    notes.push(format!("A peer joined as participant {}",
                        table.participant_for_peer(session.principal.subject).unwrap_or_default()));
                    let handle = self.runtime.handle().clone();
                    let expires = session.claims.delegations.iter().filter_map(|cert| cert.certificate.expires_at_ms).min();
                    std::thread::spawn(move || handle.block_on(async move {
                        let authority = lifecycle::SessionAuthority::retain_admitted(&session);
                        let mut resume = |bridge: &mut BridgeEndpoint, request| bridge.resume(request).map_err(|e| e.to_string());
                        let ledger = std::sync::RwLock::new(notochord::RevocationLedger::default());
                        if let Err(error) = graphshell_host::session_notices::serve_admitted_session_notifying_checked(&mut session, &authority, &ledger,
                            &mut bridge, &mut resume, now_ms, std::time::Duration::from_millis(100),
                            |_| expires.is_none_or(|end| now_ms() <= end)).await {
                            eprintln!("[isocosm-family] peer session ended: {error}");
                        }
                    }));
                },
                Err(error) => notes.push(format!("Peer refused: {error}")),
            }
        }
        if let Some(outbox) = self.outbox.as_ref().filter(|_| self.record_topic.is_some_and(|(_, access)| access.is_live(now_ms()))) {
            if let Err(error) = outbox.try_forward(&table.host) { notes.push(error); }
        } else {
            // Projection-only hosting consumes accepted publications locally. Durable history remains on the host.
            table.host.drain_records();
        }
        (pumps, notes)
    }

    fn invite_contacts(&self, table: &mut FamilyTable, notes: &mut Vec<String>) -> Result<(), String> {
        let entries = std::fs::read_dir(&self.handoff).map_err(|e| format!("{}: {e}", self.handoff.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(CONTACT_SUFFIX) else { continue };
            let invitation_path = self.handoff.join(format!("{stem}{INVITATION_SUFFIX}"));
            let contact: Contact = read_json(&entry.path())?;
            let now = now_ms();
            let (invitation, participant) = table.invite(&*self.signer, contact, now,
                now + INVITATION_LIFETIME_MS)?;
            if read_json::<Invitation>(&invitation_path).is_ok_and(|prior| prior == invitation) { continue; }
            write_json(&invitation_path, &invitation)?;
            notes.push(format!("Invited {stem} as participant {participant}"));
        }
        Ok(())
    }

    pub(crate) fn close(self) {
        let transport = Arc::clone(&self.transport);
        let _ = self.runtime.block_on(async move { transport.close().await });
    }
}
