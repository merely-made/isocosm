//! Joining a hosted table as an invited guest (isocosm case 1060).
//!
//! The guest's custody identity writes its contact into the handoff folder
//! and waits for the owner's family card and its own invitation there. It
//! then dials the owner over the admitted stream, bootstraps a replica from
//! the owner's signed checkpoint, follows the signed record lane, and shows
//! the projections the owner serves for its seat. The replica keeps the
//! accepted history; the page shows what the owner's endpoint discloses.

use crate::family_table::{
    host_net::{CARD_FILE, read_json, write_json},
    invite::{RECORD_TERMS, TERMS},
    view::{invocation, projection_view},
};
use chirograph::{
    Carrier,
    CarrierRequestBody, CarrierResponseBody, IntentResult, ProjectionSnapshot, ResourceChunkRequest,
    ResourceResponse,
};
use graphshell_host::invitation::{Contact, Invitation};
use isocosm_views::{VttActionRequest, VttProjectionView};
use isonetry::family::{FamilyReplica, WireLimits};
use isonetry::family_network::{
    FamilyCheckpointDescriptor, FamilyReplicaNetwork,
    invited::{self, FamilyCard},
    stream::network_carrier::{CarrierRuntime, NetworkCarrier},
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::Duration;

type Records = FamilyReplicaNetwork<muniment::MemoryBackend>;
const CHUNK: u32 = 512 * 1024;

enum Command { Refresh, Invoke(VttActionRequest) }

enum Event {
    Status(String),
    Bootstrap { response: ResourceResponse, descriptor: FamilyCheckpointDescriptor,
        pinned: p2panda_core::VerifyingKey, records: Box<Records> },
    Projections(Result<Vec<VttProjectionView>, String>),
    Outcome(Result<IntentResult, String>),
    /// Nothing new from the owner's stream; records may still have arrived.
    Tick,
}

/// One guest's join, owned by the app's owner thread.
pub(crate) struct FamilyGuest {
    runtime: tokio::runtime::Runtime,
    commands: mpsc::Sender<Command>,
    events: mpsc::Receiver<Event>,
    replica: Option<FamilyReplica>,
    records: Option<Box<Records>>,
}

/// What a guest turn produced for the page.
#[derive(Default)]
pub(crate) struct GuestTurn {
    pub(crate) projections: Option<Vec<VttProjectionView>>,
    pub(crate) status: Option<String>,
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

impl FamilyGuest {
    /// Write this identity's contact and start waiting for the owner's files.
    pub(crate) fn start(identity: Arc<dyn personae::IdentityProvider + Send + Sync>, handoff: PathBuf,
        wake: Arc<dyn Fn() + Send + Sync>) -> Result<(Self, PathBuf), String> {
        std::fs::create_dir_all(&handoff).map_err(|e| format!("{}: {e}", handoff.display()))?;
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()
            .map_err(|e| e.to_string())?;
        let contact = Contact::create(&*identity, &TERMS)?;
        let name = format!("guest-{}", hex(&contact.root()[..4]));
        let contact_path = handoff.join(format!("{name}.contact.json"));
        if !contact_path.exists() { write_json(&contact_path, &contact)?; }
        let transport = Arc::new(runtime.block_on(invited::bind(&*identity, &TERMS))?);
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let handle = runtime.handle().clone();
        let invitation_path = handoff.join(format!("{name}.invitation.json"));
        let card_path = handoff.join(CARD_FILE);
        std::thread::Builder::new().name("isocosm-guest".into()).spawn(move || {
            let send = |event| { let _ = event_tx.send(event); wake(); };
            if let Err(error) = run(&handle, &*identity, &transport, &invitation_path, &card_path,
                &command_rx, &send) {
                send(Event::Status(format!("Joining failed: {error}")));
            }
        }).map_err(|e| e.to_string())?;
        Ok((Self { runtime, commands, events, replica: None, records: None }, contact_path))
    }

    /// Ask the owner to apply one page action for this seat.
    pub(crate) fn invoke(&self, request: VttActionRequest) {
        let _ = self.commands.send(Command::Invoke(request));
    }

    /// The owner thread's turn: install the replica, apply arrived records,
    /// and collect the page's projections and status.
    pub(crate) fn tick(&mut self) -> GuestTurn {
        let mut turn = GuestTurn::default();
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Status(status) => turn.status = Some(status),
                Event::Tick => {},
                Event::Bootstrap { response, descriptor, pinned, records } => {
                    match Records::install_checkpoint(&response, &descriptor, &pinned, WireLimits::default(),
                        cosmograph::Execution::Grouped) {
                        Ok(replica) => {
                            turn.status = Some(format!("Joined the table at record {}", replica.sequence()));
                            self.replica = Some(replica);
                            self.records = Some(records);
                        },
                        Err(error) => turn.status = Some(format!("The owner's checkpoint was refused: {error}")),
                    }
                },
                Event::Projections(Ok(views)) => turn.projections = Some(views),
                Event::Projections(Err(error)) => turn.status = Some(format!("Table view unavailable: {error}")),
                Event::Outcome(outcome) => turn.status = Some(match outcome {
                    Ok(IntentResult::Accepted) => "Action accepted".into(),
                    Ok(IntentResult::Rejected { reason }) => format!("Action refused: {reason}"),
                    Ok(IntentResult::Stale { .. }) => "That view changed first. Review the refreshed table.".into(),
                    Err(error) => format!("Action failed: {error}"),
                }),
            }
        }
        if let (Some(replica), Some(records)) = (self.replica.as_mut(), self.records.as_mut()) {
            match records.apply_ready(replica) {
                Ok(0) => {},
                Ok(_) => { let _ = self.commands.send(Command::Refresh); },
                Err(error) => turn.status = Some(format!("Replica stopped: {error}")),
            }
        }
        turn
    }

    /// The accepted record this guest's replica has reached.
    pub(crate) fn sequence(&self) -> Option<u64> { self.replica.as_ref().map(FamilyReplica::sequence) }

    #[cfg(test)]
    pub(crate) fn replica(&self) -> Option<&FamilyReplica> { self.replica.as_ref() }

    pub(crate) fn close(self) {
        let Self { runtime, records, .. } = self;
        if let Some(records) = records { let _ = runtime.block_on(records.leave_and_wait()); }
    }
}

fn run(handle: &tokio::runtime::Handle, identity: &dyn personae::IdentityProvider,
    transport: &Arc<transport::P2pandaTransport>, invitation_path: &std::path::Path,
    card_path: &std::path::Path, commands: &mpsc::Receiver<Command>, send: &dyn Fn(Event)) -> Result<(), String> {
    send(Event::Status("Waiting for the owner's invitation in the handoff folder".into()));
    while !(invitation_path.exists() && card_path.exists()) {
        std::thread::sleep(Duration::from_millis(500));
    }
    let invitation: Invitation = read_json(invitation_path)?;
    let card: FamilyCard = read_json(card_path)?;
    let pinned = card.check(&TERMS, &RECORD_TERMS, &invitation)?;
    let topic = invited::topic(&TERMS, card.card.owner.root(), invitation.session);
    let records = handle.block_on(Records::join(transport, murm_replication::MunimentStore::new(
        muniment::MemoryBackend::new()), &card.checkpoint, topic, pinned, WireLimits::default(), 64))?;
    send(Event::Status("Dialling the table's owner".into()));
    let (owner, stream) = handle.block_on(invited::dial(transport, identity, &TERMS, &invitation, &card.card))?;
    handle.block_on(transport.set_topics(owner, &[transport::sync_overlay_topic(topic)]))
        .map_err(|e| e.to_string())?;
    let mut carrier = NetworkCarrier::over(stream, CarrierRuntime::borrowed(handle.clone()));
    let mut request = |body| carrier.request(body).map_err(|e| e.to_string());
    let opened = request(CarrierRequestBody::Open(Box::new(chirograph::SessionOpen {
        version: chirograph::ProtocolVersion { major: 1, minor: 0 },
        capabilities: chirograph::CapabilityProfile::default(),
    })))?;
    if !matches!(opened, CarrierResponseBody::Opened(_)) { return Err("the owner did not open the session".into()); }
    let response = fetch_checkpoint(&mut request, &card.checkpoint)?;
    send(Event::Bootstrap { response, descriptor: card.checkpoint.clone(), pinned, records: Box::new(records) });
    let CarrierResponseBody::Descriptor(descriptor) = request(CarrierRequestBody::Discover)? else {
        return Err("the owner described no table".into());
    };
    let mut snapshots = BTreeMap::new();
    send(Event::Projections(views(&mut request, &descriptor, &mut snapshots)));
    loop {
        match commands.recv_timeout(Duration::from_millis(500)) {
            Ok(Command::Refresh) => send(Event::Projections(views(&mut request, &descriptor, &mut snapshots))),
            Ok(Command::Invoke(action)) => {
                let outcome = invocation(&snapshots, action).and_then(|intent| match request(CarrierRequestBody::Intent(intent))? {
                    CarrierResponseBody::Intent(result) => Ok(result),
                    _ => Err("the owner answered another request".into()),
                });
                send(Event::Outcome(outcome));
                send(Event::Projections(views(&mut request, &descriptor, &mut snapshots)));
            },
            // A quiet turn still wakes the owner thread so arrived records apply.
            Err(mpsc::RecvTimeoutError::Timeout) => send(Event::Tick),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let _ = request(CarrierRequestBody::Close);
    Ok(())
}

/// The checkpoint, read in chunks and checked against the card's address.
fn fetch_checkpoint(request: &mut dyn FnMut(CarrierRequestBody) -> Result<CarrierResponseBody, String>,
    descriptor: &FamilyCheckpointDescriptor) -> Result<ResourceResponse, String> {
    let mut bytes = Vec::with_capacity(usize::try_from(descriptor.byte_size).unwrap_or(0));
    loop {
        let CarrierResponseBody::ResourceChunk(chunk) = request(CarrierRequestBody::ResourceChunk(ResourceChunkRequest {
            session: descriptor.projection_session.clone(), resource: descriptor.resource,
            offset: bytes.len() as u64, length: CHUNK }))? else {
            return Err("the owner answered another request".into());
        };
        let part = chunk.decode().ok_or("a checkpoint chunk failed its address")?;
        let last = chunk.is_final(part.len()) || part.is_empty();
        bytes.extend(part);
        if last || bytes.len() as u64 >= descriptor.byte_size { break; }
    }
    let response = ResourceResponse::new(descriptor.projection_session.clone(), bytes);
    if response.resource != descriptor.resource { return Err("the checkpoint differs from the card".into()); }
    Ok(response)
}

/// Every projection the owner serves this seat, as the page shows them.
fn views(request: &mut dyn FnMut(CarrierRequestBody) -> Result<CarrierResponseBody, String>,
    descriptor: &chirograph::EndpointDescriptor, snapshots: &mut BTreeMap<String, ProjectionSnapshot>)
-> Result<Vec<VttProjectionView>, String> {
    let mut views = Vec::new();
    for offer in &descriptor.projections {
        let CarrierResponseBody::Snapshot(snapshot) = request(CarrierRequestBody::Snapshot(offer.request.clone()))? else {
            return Err("the owner answered another request".into());
        };
        let id = offer.request.session.0.clone();
        views.push(projection_view(&mut |resource| match request(CarrierRequestBody::Resource(resource))? {
            CarrierResponseBody::Resource(response) => Ok(response),
            _ => Err("the owner answered another request".into()),
        }, &offer.label, &id, &snapshot)?);
        snapshots.insert(id, *snapshot);
    }
    Ok(views)
}

#[cfg(test)]
#[path = "family_guest_tests.rs"]
mod tests;
