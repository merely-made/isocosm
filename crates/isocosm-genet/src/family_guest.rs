//! Joining a hosted table as an invited guest (isocosm case 1060).
//!
//! The guest's custody identity writes its contact into the handoff folder
//! and waits for the owner's family card and its own invitation there. It
//! then dials the owner over the admitted stream and presents its seat projections.
//! Full replicas additionally require an owner-issued replication grant and a local opt-in.

use crate::family_table::{
    host_net::{CARD_FILE, read_json, replica_file, write_json},
    invite::{ReplicaOffer, TERMS, now_ms},
    view::{invocation, projection_view},
};
use chirograph::{
    CarrierOutput, CarrierRequest,
    CarrierRequestBody, CarrierResponseBody, IntentResult, ProjectionSnapshot, ResourceChunkRequest,
    ResourceResponse,
};
use graphshell_host::invitation::{Card, Contact, Invitation};
use isocosm_views::{VttActionRequest, VttProjectionView};
use isonetry::family::{FamilyReplica, WireLimits};
use isonetry::family_network::{
    FamilyCheckpointDescriptor, FamilyRecordAccess, FamilyReplicaNetwork,
    invited::{self, FamilyCard},

};
use std::collections::BTreeMap;
use graphshell_client::pump::ItemPump;
use transport::Transport;
use tokio::sync::mpsc as command;
type Stream = <transport::P2pandaTransport as Transport>::Stream;
const TIMEOUT: Duration = Duration::from_secs(15);
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::Duration;

type Records = FamilyReplicaNetwork<muniment::MemoryBackend>;
const CHUNK: u32 = 512 * 1024;

enum Command { Invoke(VttActionRequest) }

enum Event {
    Status(String),
    Bootstrap { response: ResourceResponse, descriptor: FamilyCheckpointDescriptor,
        pinned: p2panda_core::VerifyingKey, records: Box<Records> },
    Projections(Result<Vec<VttProjectionView>, String>),
    Outcome(Result<IntentResult, String>),
    /// Nothing new from the owner's stream; records may still have arrived.
    Tick,
    Disconnected(mpsc::Sender<()>),
}

/// One guest's join, owned by the app's owner thread.
pub(crate) struct FamilyGuest {
    runtime: tokio::runtime::Runtime,
    commands: command::UnboundedSender<Command>,
    transport: Arc<transport::P2pandaTransport>,
    worker: std::thread::JoinHandle<()>,
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
        replicate: bool, wake: Arc<dyn Fn() + Send + Sync>) -> Result<(Self, PathBuf), String> {
        std::fs::create_dir_all(&handoff).map_err(|e| format!("{}: {e}", handoff.display()))?;
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()
            .map_err(|e| e.to_string())?;
        let contact = Contact::create(&*identity, &TERMS)?;
        let legacy = format!("guest-{}", hex(&contact.root()[..4]));
        let name = if read_json::<Contact>(&handoff.join(format!("{legacy}.contact.json")))
            .is_ok_and(|prior| prior == contact) {
            legacy
        } else { format!("guest-{}", hex(&contact.root())) };
        let contact_path = handoff.join(format!("{name}.contact.json"));
        // A fresh join also asks the owner to inspect an existing, possibly expired invitation.
        write_json(&contact_path, &contact)?;
        let transport = Arc::new(runtime.block_on(invited::bind(&*identity, &TERMS, replicate))?);
        let (commands, mut command_rx) = command::unbounded_channel();
        let (event_tx, events) = mpsc::channel();
        let handle = runtime.handle().clone();
        let invitation_path = handoff.join(format!("{name}.invitation.json"));
        let card_path = handoff.join(CARD_FILE);
        let worker_transport = Arc::clone(&transport);
        let offer_path = handoff.join(replica_file(&contact));
        let worker = std::thread::Builder::new().name("isocosm-guest".into()).spawn(move || {
            let send = |event| { let _ = event_tx.send(event); wake(); };
            send(Event::Status("Waiting for the owner's invitation in the handoff folder".into()));
            loop {
                if command_rx.is_closed() { break; }
                if invitation_path.exists() && card_path.exists() {
                    match run(&handle, &*identity, &worker_transport, &invitation_path, &card_path,
                        replicate.then_some(offer_path.as_path()), &mut command_rx, &send) {
                        Ok(()) => break,
                        Err(error) => {
                            send(Event::Status(format!("Reconnecting to the table: {error}")));
                            let (released, wait) = mpsc::channel();
                            send(Event::Disconnected(released));
                            if wait.recv().is_err() { break; }
                            while command_rx.try_recv().is_ok() {}
                        }
                    }
                }
                // Only commands made against a live view may reach its owner. Never replay an intent.
                match handle.block_on(async { tokio::time::timeout(Duration::from_millis(500), command_rx.recv()).await.ok().map(|cmd| cmd.is_some()) }) {
                    Some(true) => send(Event::Status("Reconnect before acting at the table".into())),
                    Some(false) => break,
                    None => {}
                }
            }
        }).map_err(|e| e.to_string())?;
        Ok((Self { runtime, commands, events, transport, worker, replica: None, records: None }, contact_path))
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
                Event::Disconnected(released) => {
                    turn.projections = Some(Vec::new());
                    self.replica = None;
                    if let Some(records) = self.records.take() {
                        let _ = self.runtime.block_on(records.leave_and_wait());
                    }
                    let _ = released.send(());
                },
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
                Ok(_) => {},
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
        let Self { runtime, commands, events, transport, worker, records, .. } = self;
        drop(commands);
        drop(events);
        let _ = runtime.block_on(transport.close());
        let _ = worker.join();
        if let Some(records) = records { let _ = runtime.block_on(records.leave_and_wait()); }
    }
}

struct GuestCarrier<'a> {
    pump: ItemPump<Stream>,
    runtime: &'a tokio::runtime::Handle,
    next_id: u64,
    dirty: bool,
}

impl GuestCarrier<'_> {
    fn request(&mut self, body: CarrierRequestBody) -> Result<CarrierResponseBody, String> {
        let id = self.next_id;
        self.next_id += 1;
        // One canonical CBOR wire item each way, as Graphshell's guests send (D210).
        let item = chirograph::wire::encode(&CarrierRequest { id, body }).map_err(|e| e.to_string())?;
        self.runtime.block_on(async {
            tokio::time::timeout(TIMEOUT, async {
                self.pump.send(&item).await.map_err(|e| e.to_string())?;
                loop {
                    let item = self.pump.recv().await.map_err(|e| e.to_string())?.ok_or("table disconnected")?;
                    match chirograph::wire::decode::<CarrierOutput>(&item).map_err(|e| e.to_string())? {
                        CarrierOutput::Response(response) if response.id == id =>
                            return response.body.map_err(|failure| failure.message),
                        CarrierOutput::Notice(_) => self.dirty = true,
                        CarrierOutput::Response(_) => return Err("the owner answered another request".into()),
                    }
                }
            }).await.map_err(|_| "table response timed out".to_owned())?
        })
    }
}

/// Older handoffs wrap the seat card with replication metadata. Only the seat card is used here.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum SeatCard { Current(Card), Legacy(FamilyCard) }

fn run(handle: &tokio::runtime::Handle, identity: &dyn personae::IdentityProvider,
    transport: &transport::P2pandaTransport, invitation_path: &std::path::Path,
    card_path: &std::path::Path, offer_path: Option<&std::path::Path>,
    commands: &mut command::UnboundedReceiver<Command>, send: &dyn Fn(Event)) -> Result<(), String> {
    let invitation: Invitation = read_json(invitation_path)?;
    let card = match read_json(card_path)? { SeatCard::Current(card) => card, SeatCard::Legacy(old) => old.card };
    card.check(&TERMS, &invitation)?;
    let (owner, stream) = handle.block_on(async {
        tokio::time::timeout(TIMEOUT, invited::dial(transport, identity, &TERMS, &invitation, &card)).await
            .map_err(|_| "table connection timed out")?
    })?;
    let mut carrier = GuestCarrier { pump: ItemPump::new(stream), runtime: handle, next_id: 1, dirty: false };
    let opened = handle.block_on(async {
        tokio::time::timeout(TIMEOUT, carrier.pump.open(chirograph::CapabilityProfile::default())).await
            .map_err(|_| "table open timed out")?.map_err(|e| e.to_string())
    })?;
    let descriptor = opened.descriptor;
    let mut snapshots = BTreeMap::new();
    send(Event::Projections(Ok(views(&mut |body| carrier.request(body), &descriptor, &mut snapshots)?)));
    send(Event::Status("Joined the table as an invited player".into()));
    let mut following = false;
    if let Some(path) = offer_path {
        let bootstrap: Result<Event, String> = (|| {
            let offer: ReplicaOffer = read_json(path)?;
            if offer.card.card != card { return Err("replica offer belongs to a different hosting session".into()); }
            let pinned = offer.verify(&invitation, transport.local_peer_id().to_bytes(), now_ms())?;
            let expires = offer.grant.grant.certificate.expires_at_ms.ok_or("replica grant lacks expiry")?;
            let topic = invited::topic(&TERMS, card.owner.root(), invitation.session);
            let records = handle.block_on(Records::join(transport, murm_replication::MunimentStore::new(
                muniment::MemoryBackend::new()), &offer.card.checkpoint, topic, pinned,
                FamilyRecordAccess::until(owner.to_bytes(), expires), WireLimits::default(), 64))?;
            handle.block_on(transport.set_topics(owner, &[transport::sync_overlay_topic(topic)]))
                .map_err(|e| e.to_string())?;
            let response = fetch_checkpoint(&mut |body| carrier.request(body), &offer.card.checkpoint)?;
            Ok(Event::Bootstrap { response, descriptor: offer.card.checkpoint, pinned, records: Box::new(records) })
        })();
        match bootstrap {
            Ok(event) => { send(event); following = true; }
            Err(error) => send(Event::Status(format!("Full replication unavailable: {error}"))),
        }
    }
    enum Incoming { Action(Option<Command>), Item(Option<Vec<u8>>), Tick }
    loop {
        if carrier.dirty {
            carrier.dirty = false;
            let next = views(&mut |body| carrier.request(body), &descriptor, &mut snapshots)?;
            send(Event::Projections(Ok(next)));
            continue;
        }
        let incoming = handle.block_on(async {
            tokio::select! {
                action = commands.recv() => Ok::<_, String>(Incoming::Action(action)),
                item = carrier.pump.recv() => Ok(Incoming::Item(item.map_err(|e| e.to_string())?)),
                _ = tokio::time::sleep(Duration::from_millis(250)), if following => Ok(Incoming::Tick),
            }
        })?;
        match incoming {
            Incoming::Action(None) => return Ok(()),
            Incoming::Action(Some(Command::Invoke(action))) => {
                let outcome = invocation(&snapshots, action).and_then(|intent| match carrier.request(CarrierRequestBody::Intent(intent))? {
                    CarrierResponseBody::Intent(result) => Ok(result),
                    _ => Err("the owner answered another request".into()),
                });
                send(Event::Outcome(outcome));
                carrier.dirty = true;
            }
            Incoming::Item(Some(item)) => match chirograph::wire::decode::<CarrierOutput>(&item).map_err(|e| e.to_string())? {
                CarrierOutput::Notice(_) => carrier.dirty = true,
                CarrierOutput::Response(_) => return Err("unexpected idle table response".into()),
            }
            Incoming::Item(None) => return Err("owner disconnected".into()),
            Incoming::Tick => send(Event::Tick),
        }
    }
}

/// The checkpoint, read in chunks and checked against the card's address.
fn fetch_checkpoint(request: &mut dyn FnMut(CarrierRequestBody) -> Result<CarrierResponseBody, String>,
    descriptor: &FamilyCheckpointDescriptor) -> Result<ResourceResponse, String> {
    if descriptor.byte_size == 0 || descriptor.byte_size > WireLimits::default().max_bytes as u64 {
        return Err("checkpoint size is outside the family wire limit".into());
    }
    let mut assembly = chirograph::ResourceAssembly::new();
    loop {
        let offset = assembly.received();
        let CarrierResponseBody::ResourceChunk(chunk) = request(CarrierRequestBody::ResourceChunk(ResourceChunkRequest {
            session: descriptor.projection_session.clone(), resource: descriptor.resource, offset, length: CHUNK }))? else {
            return Err("the owner answered another request".into());
        };
        if chunk.session != descriptor.projection_session || chunk.resource != descriptor.resource
            || chunk.offset != offset || chunk.total_len != descriptor.byte_size {
            return Err("checkpoint chunk differs from its descriptor".into());
        }
        if let Some(bytes) = assembly.accept(&chunk).map_err(|e| e.to_string())? {
            if bytes.len() as u64 != descriptor.byte_size { return Err("checkpoint length differs from its descriptor".into()); }
            return Ok(ResourceResponse::new(descriptor.projection_session.clone(), bytes));
        }
        if assembly.received() <= offset || assembly.received() >= descriptor.byte_size {
            return Err("checkpoint chunk did not advance within its descriptor".into());
        }
    }
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
