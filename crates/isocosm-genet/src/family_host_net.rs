//! Hosting the table for invited peers (isocosm case 1060).
//!
//! The transport and record keys are the owner's custody identity's derived
//! keys. The owner publishes a family card into a handoff folder; a guest's
//! contact file dropped there is invited (one accepted record joins and
//! binds its participant) and its invitation written back beside it, so the
//! files can be carried to another machine. Admitted sessions are served
//! through the owner thread's request pump; accepted records go out on the
//! signed record lane.

use super::*;
use super::invite::{RECORD_TERMS, TERMS, now_ms};
use graphshell_host::invitation::{Card, Contact, Signer};
use isonetry::family_network::{
    FamilyCheckpointResource, FamilyHostNetwork, FamilyRecordBatch, FamilyRecordOutbox,
    MAX_FAMILY_RECORD_BATCH_BYTES,
    invited::{self, FamilyCard},
    stream::{carrier, lifecycle, session_loop},
};
use std::path::PathBuf;
use transport::Transport;

type Stream = <transport::P2pandaTransport as Transport>::Stream;

pub(crate) const CARD_FILE: &str = "table.card.json";
const CONTACT_SUFFIX: &str = ".contact.json";
const INVITATION_SUFFIX: &str = ".invitation.json";
const INVITATION_DAYS: u64 = 7;
const MAX_HANDOFF_BYTES: u64 = 65_536;

/// The owner's hosting of one table.
pub(crate) struct HostNet {
    runtime: tokio::runtime::Runtime,
    signer: Arc<dyn Signer + Send + Sync>,
    transport: Arc<transport::P2pandaTransport>,
    handoff: PathBuf,
    admitted: std::sync::mpsc::Receiver<notochord::AdmittedSession<Stream>>,
    outbox: FamilyRecordOutbox,
    pub(crate) card: FamilyCard,
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
    /// Start hosting: bind, republish the signed log, sign the bootstrap
    /// checkpoint, write the family card and begin admitting invited peers.
    pub(crate) fn host_peers(&mut self, signer: Arc<dyn Signer + Send + Sync>, handoff: PathBuf,
        wake: Arc<dyn Fn() + Send + Sync>) -> Result<HostNet, String> {
        std::fs::create_dir_all(&handoff).map_err(|e| format!("{}: {e}", handoff.display()))?;
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()
            .map_err(|e| e.to_string())?;
        let identity: &dyn personae::IdentityProvider = &*signer;
        let transport = Arc::new(runtime.block_on(invited::bind(identity, &TERMS))?);
        let owner = signer.master_public_key().to_bytes();
        let session = self.invitation_session();
        let topic = invited::topic(&TERMS, owner, session);
        let limits = isonetry::family::WireLimits::default();
        let network = Arc::new(runtime.block_on(FamilyHostNetwork::join(&transport,
            murm_replication::MunimentStore::new(muniment::MemoryBackend::new()), self.host.session(), topic,
            invited::record_key(identity, &RECORD_TERMS)?, limits)).map_err(|e| e.to_string())?);
        if self.durable {
            let batch = FamilyRecordBatch::persisted(self.host.persisted_suffix_after(0)?)?;
            runtime.block_on(network.publish_batch(&batch))?;
        }
        let (outbox, mut batches) = FamilyRecordOutbox::channel(8, MAX_FAMILY_RECORD_BATCH_BYTES)?;
        if outbox.try_forward(&self.host)? > 0 {
            let batch = batches.try_recv().map_err(|e| e.to_string())?;
            runtime.block_on(network.publish_batch(&batch))?;
        }
        let checkpoint = self.host.checkpoint()?;
        let resource = runtime.block_on(network.signed_checkpoint(checkpoint,
            chirograph::ProjectionSession("isocosm-family-checkpoint".into())))?;
        let card = FamilyCard {
            version: 1,
            card: Card { version: 1, session, owner: Contact::create(identity, &TERMS)?,
                ticket: runtime.block_on(transport.ticket()).map_err(|e| e.to_string())? },
            records: Contact::create(identity, &RECORD_TERMS)?,
            checkpoint: resource.descriptor().clone(),
        };
        write_json(&handoff.join(CARD_FILE), &card)?;
        self.checkpoint = Some(resource);

        let publisher = Arc::clone(&network);
        runtime.spawn(async move {
            while let Some(batch) = batches.recv().await {
                if let Err(error) = publisher.publish_batch(&batch).await {
                    eprintln!("[isocosm-family] record publication failed: {error}");
                }
            }
        });
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
                        if let Some(peer) = session.facts.authenticated_initiator
                            .and_then(|peer| transport::PeerID::from_bytes(&peer).ok()) {
                            let overlay = transport::sync_overlay_topic(topic);
                            let _ = accepting.set_topics(peer, &[overlay]).await;
                        }
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
        Ok(HostNet { runtime, signer, transport, handoff, admitted, outbox, card })
    }

    /// The signed checkpoint, served to an admitted peer that asks for exactly it.
    pub(crate) fn checkpoint_answer(&self, call: &EndpointCall) -> Option<Result<EndpointAnswer, String>> {
        let resource: &FamilyCheckpointResource = self.checkpoint.as_ref()?;
        match call {
            EndpointCall::Resource(request) => resource.respond(request).map(|r| Ok(EndpointAnswer::Resource(r))),
            EndpointCall::ResourceChunk(chunk) => resource.respond(&ResourceRequest {
                session: chunk.session.clone(), resource: chunk.resource,
            }).map(|whole| Ok(EndpointAnswer::ResourceChunk(
                chirograph::ResourceChunkResponse::slice(&whole, chunk.offset, chunk.length)))),
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
        if let Err(error) = self.invite_contacts(table, &mut notes) { notes.push(error); }
        let mut pumps = Vec::new();
        while let Ok(mut session) = self.admitted.try_recv() {
            match table.admitted_endpoint(&session, Arc::clone(&wake)) {
                Ok((mut bridge, pump)) => {
                    pumps.push(pump);
                    notes.push(format!("A peer joined as participant {}",
                        table.participant_for_peer(session.principal.subject).unwrap_or_default()));
                    let handle = self.runtime.handle().clone();
                    std::thread::spawn(move || handle.block_on(async move {
                        let authority = lifecycle::SessionAuthority::retain_admitted(&session);
                        let mut resume = |_: &mut _, _| Err("resume is not offered".to_string());
                        let ledger = std::sync::RwLock::new(notochord::RevocationLedger::default());
                        if let Err(error) = session_loop::serve_admitted_session(&mut session, &authority, &ledger,
                            &mut bridge, &mut resume, now_ms).await {
                            eprintln!("[isocosm-family] peer session ended: {error}");
                        }
                    }));
                },
                Err(error) => notes.push(format!("Peer refused: {error}")),
            }
        }
        if let Err(error) = self.outbox.try_forward(&table.host) { notes.push(error); }
        (pumps, notes)
    }

    fn invite_contacts(&self, table: &mut FamilyTable, notes: &mut Vec<String>) -> Result<(), String> {
        let entries = std::fs::read_dir(&self.handoff).map_err(|e| format!("{}: {e}", self.handoff.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(CONTACT_SUFFIX) else { continue };
            let invitation_path = self.handoff.join(format!("{stem}{INVITATION_SUFFIX}"));
            if invitation_path.exists() { continue; }
            let contact: Contact = read_json(&entry.path())?;
            let now = now_ms();
            let (invitation, participant) = table.invite(&*self.signer, contact, now,
                now + INVITATION_DAYS * 24 * 60 * 60 * 1000)?;
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
