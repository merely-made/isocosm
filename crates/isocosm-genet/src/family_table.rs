//! The shipping VTT table adapter over Cosmograph's single accepted FamilyHost.
//!
//! The FamilyHost and its Graphshell endpoint stay on the winit thread because
//! the shared World is intentionally `!Send`. Only the projection card model
//! and accepted records leave this module.

use std::collections::BTreeMap;
use std::path::Path;

use chirograph::{
    IntentInvocation, IntentResult, PortableCardV1, ProjectionSnapshot, ResourceRequest, Revision,
    SceneEpoch,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use cosmograph::{Execution, Session};
use cosmograph_host::world::World;
use cosmograph_overlay::{ParticipantHandle, Seat};
use isocosm_native::VttEndpoint;
use isocosm_native::founding::{Content, GenerationOrigin, NativeWorld};
use isocosm_views::{
    VttActionRequest, VttActionView, VttCardValue, VttCardView, VttProjectionView,
};
use isonetry::family::{AcceptedRecord, FamilyHost, FamilyLedgerLimits, FamilyLimits, SessionId};
use isonetry::family_network::{
    AdmittedEndpointRequest, AdmittedRequestPump, AuthenticatedPeerKey, BridgeEndpoint,
    EndpointAnswer, EndpointCall, EndpointBridgeLimits,
};
use notochord::AdmittedSession;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::App;

const MAX_ADMITTED_FAMILY_STREAMS: usize = 8;

impl App {
    /// Register one caller-admitted stream with this app's owner-thread pump.
    /// The caller still owns admission, serving, and stream shutdown.
    pub(crate) fn register_admitted_family_session<S>(
        &mut self,
        session: &AdmittedSession<S>,
    ) -> Result<BridgeEndpoint, String> {
        if self.family_request_pumps.len() >= MAX_ADMITTED_FAMILY_STREAMS {
            return Err("family admitted-stream limit reached".into());
        }
        let wake = self
            .family_wake
            .as_ref()
            .ok_or("family owner-thread wake is unavailable")?
            .callback();
        let table = self
            .family_table
            .as_ref()
            .ok_or("family table is not open")?;
        let (endpoint, pump) = table.admitted_endpoint(session, wake)?;
        self.family_request_pumps.push(pump);
        Ok(endpoint)
    }
}

const VTT_SECTION: &str = "vtt";
const CAMPAIGN_SECTION: &str = "isocosm-native:campaign";
const ACCESS_SECTION: &str = "isocosm-native:table-access";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CampaignMetadata {
    content: Content,
    origin: Option<GenerationOrigin>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TableAccess {
    host_participant: cosmograph::schema::Id,
    peers: Vec<isonetry::native_table::PeerParticipant>,
}

/// A local VTT host imported from the app's one-time native publication.
pub(crate) struct FamilyTable {
    host: FamilyHost,
    endpoint: VttEndpoint,
    content: Content,
    origin: Option<GenerationOrigin>,
    access: TableAccess,
    durable: bool,
    snapshots: BTreeMap<String, ProjectionSnapshot>,
    peer_endpoints: BTreeMap<[u8; 32], VttEndpoint>,
}

impl FamilyTable {
    /// Import an already accepted native publication exactly once. Existing
    /// family ledgers resume as the authority and do not re-import the older
    /// `GameSnapshot` copy.
    pub(crate) fn open(
        initial: Option<&NativeWorld>,
        native_table: Option<&isonetry::NativeTable>,
        ledger_path: Option<&Path>,
    ) -> Result<Self, String> {
        let durable = ledger_path.is_some();
        let resumed = ledger_path.is_some_and(|path| path.exists());
        let host = match ledger_path {
            Some(path) if path.exists() => FamilyHost::resume_file(
                path,
                Execution::Grouped,
                FamilyLimits::default(),
                FamilyLedgerLimits::default(),
            )?,
            path => {
                let initial = initial.ok_or(
                    "this table has no accepted native world; family hosting requires a native campaign",
                )?;
                let campaign = initial
                    .load(Execution::Grouped)
                    .map_err(|error| error.to_string())?;
                let saved = campaign.save();
                let metadata = CampaignMetadata {
                    content: saved.content,
                    origin: saved.origin,
                };
                let table = native_table
                    .ok_or("family hosting requires explicit native table participant bindings")?;
                let access = TableAccess {
                    host_participant: table.host_participant,
                    peers: table.peers.clone(),
                };
                let mut sections = BTreeMap::new();
                sections.insert(VTT_SECTION.to_owned(), table.game.clone());
                sections.insert(
                    ACCESS_SECTION.to_owned(),
                    serde_json::to_vec(&access).map_err(|error| error.to_string())?,
                );
                sections.insert(
                    CAMPAIGN_SECTION.to_owned(),
                    serde_json::to_vec(&metadata).map_err(|error| error.to_string())?,
                );
                let world = World::with_sections(
                    Session::load(saved.native, Execution::Grouped)?,
                    sections,
                );
                let session = family_session_id(&campaign.save().native)?;
                match path {
                    Some(path) => FamilyHost::create_file(
                        path,
                        session,
                        world,
                        FamilyLimits::default(),
                        FamilyLedgerLimits::default(),
                    )?,
                    None => FamilyHost::new(session, world)?,
                }
            },
        };

        let (metadata, access) = host.read(|session, sections| {
            let metadata = sections
                .get(CAMPAIGN_SECTION)
                .ok_or_else(|| "family ledger has no VTT campaign content section".to_owned())
                .and_then(|bytes| {
                    serde_json::from_slice::<CampaignMetadata>(bytes)
                        .map_err(|error| error.to_string())
                })?;
            let table = sections
                .get(VTT_SECTION)
                .ok_or_else(|| "family ledger has no VTT game section".to_owned())?;
            let bindings = isocosm_native::native::GameBindings::decode(table)?;
            bindings.validate(session)?;
            let access = if let Some(access) = sections.get(ACCESS_SECTION) {
                serde_json::from_slice::<TableAccess>(access).map_err(|error| error.to_string())?
            } else {
                let table = native_table.ok_or(
                    "family ledger has no saved peer bindings; reopen from its native table metadata",
                )?;
                TableAccess { host_participant: table.host_participant, peers: table.peers.clone() }
            };
            Ok::<_, String>((metadata, access))
        })??;
        let session = host.world().session()?;
        let access_game = host
            .world()
            .section(VTT_SECTION)?
            .ok_or("family ledger has no VTT game section")?;
        isocosm_native::native::GameBindings::decode(&access_game)?.validate(&session)?;
        if !session
            .sim
            .state()
            .population
            .get(access.host_participant)
            .is_some_and(cosmograph::directing::is_participant)
        {
            return Err("family table host binding is not a native participant".into());
        }
        let mut endpoints = std::collections::BTreeSet::new();
        for peer in &access.peers {
            if !endpoints.insert(peer.endpoint)
                || !session
                    .sim
                    .state()
                    .population
                    .get(peer.participant)
                    .is_some_and(cosmograph::directing::is_participant)
            {
                return Err(
                    "family peer bindings contain a duplicate endpoint or absent participant"
                        .into(),
                );
            }
        }
        drop(session);
        if std::env::var_os("ISOCOSM_PROFILE").is_some() {
            eprintln!(
                "[isocosm-family] {} durable={} sequence={} revision={} native={}",
                if resumed { "resumed" } else { "created" }, durable,
                host.sequence(), host.revision()?,
                host.read(|session, _| cosmograph::digest(&session.save()))?,
            );
        }
        let endpoint = VttEndpoint::open(
            host.world(),
            ParticipantHandle(access.host_participant),
            Seat::Dm,
        )?;
        Ok(Self {
            host,
            endpoint,
            content: metadata.content,
            origin: metadata.origin,
            access,
            durable,
            snapshots: BTreeMap::new(),
            peer_endpoints: BTreeMap::new(),
        })
    }

    pub(crate) fn projections(&mut self) -> Result<Vec<VttProjectionView>, String> {
        let offers = self.endpoint.describe().projections;
        let mut views = Vec::with_capacity(offers.len());
        let mut snapshots = BTreeMap::new();
        for offer in offers {
            let id = offer.request.session.0.clone();
            let snapshot = self.endpoint.snapshot(offer.request)?;
            views.push(projection_view(
                &mut self.endpoint,
                &offer.label,
                &id,
                &snapshot,
            )?);
            snapshots.insert(id, snapshot);
        }
        self.snapshots = snapshots;
        Ok(views)
    }

    pub(crate) fn invoke(&mut self, request: VttActionRequest) -> Result<IntentResult, String> {
        let snapshot = self
            .snapshots
            .get(&request.projection)
            .ok_or("VTT action names a projection that is not currently displayed")?;
        let target = snapshot
            .scene
            .active_items_in_order()
            .into_iter()
            .find_map(|(instance, _)| {
                (card_identifier(instance) == request.card).then_some(instance)
            })
            .ok_or("VTT action target is absent from the displayed projection")?;
        let actions = snapshot
            .scene
            .active_items_in_order()
            .into_iter()
            .find(|(instance, _)| *instance == target)
            .and_then(|(instance, _)| snapshot.presentation.offers_for(instance))
            .into_iter()
            .flatten()
            .flat_map(|offer| offer.semantics.actions.iter())
            .collect::<Vec<_>>();
        if !actions
            .iter()
            .any(|action| action.intent.0 == request.action)
        {
            return Err("VTT action is no longer advertised by its displayed card".into());
        }
        let result = self.endpoint.invoke(IntentInvocation {
            session: snapshot.session.clone(),
            target,
            observed_epoch: SceneEpoch(request.epoch),
            observed_revision: Revision(request.revision),
            intent: request.action,
            payload: request.payload,
        })?;
        Ok(result)
    }

    pub(crate) fn drain_records(&self) -> Vec<AcceptedRecord> {
        self.host.drain_records()
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.host.sequence()
    }

    pub(crate) fn checkpoint(&mut self) -> Result<isonetry::family::Checkpoint, String> {
        self.host.checkpoint()
    }

    pub(crate) fn persisted_suffix_after(
        &self,
        sequence: u64,
    ) -> Result<Vec<AcceptedRecord>, String> {
        self.host.persisted_suffix_after(sequence)
    }

    pub(crate) fn replication_snapshot(
        &self,
    ) -> Result<cosmograph_host::world::ReplicationSnapshot, String> {
        self.host.world().replication_snapshot()
    }

    pub(crate) fn revision(&self) -> Result<u64, String> {
        self.host.revision()
    }

    pub(crate) fn vtt_section(&self) -> Result<Vec<u8>, String> {
        self.host
            .world()
            .section(VTT_SECTION)?
            .ok_or_else(|| "VTT game section is absent".into())
    }

    pub(crate) fn edition_state(&self)->Result<Option<isocosm_native::ruleset::State>,String> {
        self.host.read(|_,sections|isocosm_native::ruleset::State::read(sections))?
    }

    pub(crate) fn native_world(&self) -> Result<NativeWorld, String> {
        self.host.read(|session, _| {
            NativeWorld::from_session(session, self.content.clone(), self.origin.clone())
        })?
    }

    pub(crate) fn has_pending_records(&self) -> bool {
        self.host.outbox_bytes() > 0
    }

    pub(crate) fn participant_for_peer(&self, key: [u8; 32]) -> Option<cosmograph::schema::Id> {
        self.access
            .peers
            .iter()
            .find(|peer| peer.endpoint == key)
            .map(|peer| peer.participant)
    }

    pub(crate) fn is_durable(&self) -> bool {
        self.durable
    }

    /// Prepare the product endpoint for a session that the caller has already
    /// admitted. The transport owner retains session lifetime and admission
    /// policy; this only binds the authenticated carrier key to the owner
    /// thread's bounded VTT request pump.
    pub(crate) fn admitted_endpoint<S>(
        &self,
        session: &AdmittedSession<S>,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<(BridgeEndpoint, AdmittedRequestPump), String> {
        let peer_key = AuthenticatedPeerKey::from_admitted_session(session)
            .map_err(|error| error.to_string())?;
        let participant = self
            .participant_for_peer(*peer_key.as_bytes())
            .ok_or("admitted peer has no native table participant binding")?;
        let peer_endpoint = VttEndpoint::open(
            self.host.world(),
            ParticipantHandle(participant),
            Seat::Player,
        )?;
        let descriptor = peer_endpoint.describe();
        BridgeEndpoint::for_admitted_session_with_wake(
            session,
            descriptor,
            EndpointBridgeLimits::default(),
            wake,
        )
        .map_err(|error| error.to_string())
    }

    /// Pump a bounded number of already-admitted Graphshell requests on the
    /// owner thread. Transport identity is resolved only through saved native
    /// peer bindings; unknown peers fail closed.
    pub(crate) fn pump_admitted_requests(
        &mut self,
        pump: &AdmittedRequestPump,
        budget: usize,
    ) -> Result<usize, String> {
        let mut consumed = 0;
        while consumed < budget {
            let Some(request) = pump.try_next().map_err(|error| error.to_string())? else {
                break;
            };
            consumed += 1;
            if request.is_cancelled() {
                continue;
            }
            let answer = self.answer_admitted_request(&request);
            // The carrier can time out while a synchronous owner-thread call
            // is running. Its closed reply channel does not invalidate this
            // stream's pump or any later request.
            let _ = request.respond(answer);
        }
        Ok(consumed)
    }

    fn answer_admitted_request(
        &mut self,
        request: &AdmittedEndpointRequest,
    ) -> Result<EndpointAnswer, String> {
        let peer_key = *request.peer_key.as_bytes();
        let participant = self
            .participant_for_peer(peer_key)
            .ok_or_else(|| "admitted peer has no native table participant binding".to_owned())?;
        if !self.peer_endpoints.contains_key(&peer_key) {
            let endpoint = VttEndpoint::open(
                self.host.world(),
                ParticipantHandle(participant),
                Seat::Player,
            )?;
            self.peer_endpoints.insert(peer_key, endpoint);
        }
        let endpoint = self
            .peer_endpoints
            .get_mut(&peer_key)
            .expect("inserted endpoint");
        match &request.call {
            EndpointCall::Snapshot(projection) => endpoint
                .snapshot(projection.clone())
                .map(EndpointAnswer::Snapshot),
            EndpointCall::Resource(resource) => endpoint
                .resource(resource.clone())
                .map(EndpointAnswer::Resource),
            EndpointCall::ResourceChunk(chunk) => {
                let whole = endpoint.resource(ResourceRequest {
                    session: chunk.session.clone(),
                    resource: chunk.resource,
                })?;
                Ok(EndpointAnswer::ResourceChunk(
                    chirograph::ResourceChunkResponse::slice(&whole, chunk.offset, chunk.length),
                ))
            },
            EndpointCall::Intent(intent) => {
                endpoint.invoke(intent.clone()).map(EndpointAnswer::Intent)
            },
            EndpointCall::Resume(resume) => {
                let offer = endpoint
                    .describe()
                    .projections
                    .into_iter()
                    .find(|offer| offer.request.session == resume.session)
                    .ok_or_else(|| "resume names an unknown VTT projection".to_owned())?;
                endpoint.snapshot(offer.request).map(|snapshot| {
                    EndpointAnswer::Resume(chirograph::ResumeReply::Snapshot(Box::new(snapshot)))
                })
            },
        }
    }
}

fn family_session_id(saved: &cosmograph::history::Saved) -> Result<SessionId, String> {
    let digest = cosmograph::digest(saved);
    let mut id = [0; 16];
    for (index, byte) in id.iter_mut().enumerate() {
        let start = index * 2;
        *byte =
            u8::from_str_radix(&digest[start..start + 2], 16).map_err(|error| error.to_string())?;
    }
    Ok(SessionId(id))
}

fn projection_view(
    endpoint: &mut VttEndpoint,
    label: &str,
    id: &str,
    snapshot: &ProjectionSnapshot,
) -> Result<VttProjectionView, String> {
    let mut cards = Vec::new();
    for (instance, item) in snapshot.scene.active_items_in_order() {
        let source = snapshot.scene.tables.sources[item.source.0 as usize]
            .as_ref()
            .ok_or("VTT projection has an absent item source")?;
        let offers = snapshot
            .presentation
            .offers_for(instance)
            .ok_or("VTT card has no presentation offer")?;
        let portable = offers
            .iter()
            .find(|offer| offer.codec == chirograph::PresentationCodec::PortableCardV1);
        let card = match portable {
            Some(offer) => {
                let response = endpoint.resource(ResourceRequest {
                    session: snapshot.session.clone(),
                    resource: offer.resource.clone(),
                })?;
                serde_json::from_slice::<PortableCardV1>(&response.bytes)
                    .map_err(|error| format!("VTT portable card is invalid: {error}"))?
            },
            None => PortableCardV1 {
                title: source.id.clone(),
                values: Vec::new(),
                badges: Vec::new(),
                media: Vec::new(),
            },
        };
        let actions = offers
            .iter()
            .flat_map(|offer| offer.semantics.actions.iter())
            .map(|action| VttActionView {
                id: action.intent.0.clone(),
                label: action.label.clone(),
                explanation: action.explanation.clone(),
                payload_schema: action.payload_schema.clone(),
            })
            .collect();
        cards.push(VttCardView {
            id: card_identifier(instance),
            title: card.title,
            values: card
                .values
                .into_iter()
                .map(|value| VttCardValue {
                    label: value.label,
                    value: value.value,
                })
                .collect(),
            badges: card.badges,
            actions,
        });
    }
    Ok(VttProjectionView {
        id: id.to_owned(),
        label: label.to_owned(),
        epoch: snapshot.scene.epoch.0,
        revision: snapshot.scene.revision.0,
        cards,
    })
}

fn card_identifier(instance: impl std::fmt::Debug) -> String {
    format!("{instance:?}")
}
