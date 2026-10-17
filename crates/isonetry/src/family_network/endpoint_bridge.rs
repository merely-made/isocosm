//! Bounded handoff from Graphshell's admitted stream worker to a product's
//! owner-thread endpoint. The endpoint adapter is `Send`; the product `World`
//! and endpoint never leave their UI owner thread.

use chirograph::{
    EndpointDescriptor, IntentInvocation, IntentResult, ProjectionRequest, ProjectionSnapshot,
    ResourceChunkRequest, ResourceChunkResponse, ResourceRequest, ResourceResponse, ResumeReply,
    ResumeRequest,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use notochord::AdmittedSession;
use serde::Serialize;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

pub const MAX_ENDPOINT_BRIDGE_QUEUE: usize = 16;
pub const MAX_ENDPOINT_MESSAGE_BYTES: usize = 1_048_576;
pub const MAX_ENDPOINT_RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);

/// Authenticated transport identity retained from the admitted carrier.
///
/// This is the exact 32-byte `SessionFacts.authenticated_initiator` value,
/// never a Notochord principal or a product participant ID. Create it from an
/// admitted session; an absent carrier identity cannot open a peer endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthenticatedPeerKey([u8; 32]);

impl AuthenticatedPeerKey {
    pub fn from_admitted_session<S>(
        session: &AdmittedSession<S>,
    ) -> Result<Self, MissingAuthenticatedPeer> {
        session
            .facts
            .authenticated_initiator
            .map(Self)
            .ok_or(MissingAuthenticatedPeer)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// The admitted carrier proved no stable peer key, so this bridge cannot
/// route its requests to a native peer binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingAuthenticatedPeer;

impl std::fmt::Display for MissingAuthenticatedPeer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("admitted carrier did not authenticate a peer key")
    }
}

impl std::error::Error for MissingAuthenticatedPeer {}

/// Limits for the synchronous Graphshell-to-owner handoff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointBridgeLimits {
    /// Maximum queued requests waiting for the UI owner to pump them.
    pub queue_capacity: usize,
    /// Maximum serialized size of one request or response crossing the bridge.
    pub max_message_bytes: usize,
    /// Maximum time the carrier worker waits for the owner-thread reply.
    pub response_timeout: Duration,
}

impl Default for EndpointBridgeLimits {
    fn default() -> Self {
        Self {
            queue_capacity: 8,
            max_message_bytes: 1_048_576,
            response_timeout: Duration::from_secs(5),
        }
    }
}

impl EndpointBridgeLimits {
    fn validate(self) -> Result<Self, EndpointBridgeError> {
        if self.queue_capacity == 0 {
            return Err(EndpointBridgeError(
                "request queue capacity must be nonzero".into(),
            ));
        }
        if self.queue_capacity > MAX_ENDPOINT_BRIDGE_QUEUE {
            return Err(EndpointBridgeError(format!(
                "request queue capacity exceeds {MAX_ENDPOINT_BRIDGE_QUEUE}"
            )));
        }
        if self.max_message_bytes == 0 {
            return Err(EndpointBridgeError(
                "message byte limit must be nonzero".into(),
            ));
        }
        if self.max_message_bytes > MAX_ENDPOINT_MESSAGE_BYTES {
            return Err(EndpointBridgeError(format!(
                "message byte limit exceeds {MAX_ENDPOINT_MESSAGE_BYTES}"
            )));
        }
        if self.response_timeout.is_zero() {
            return Err(EndpointBridgeError(
                "response timeout must be nonzero".into(),
            ));
        }
        if self.response_timeout > MAX_ENDPOINT_RESPONSE_TIMEOUT {
            return Err(EndpointBridgeError(format!(
                "response timeout exceeds {} seconds",
                MAX_ENDPOINT_RESPONSE_TIMEOUT.as_secs()
            )));
        }
        Ok(self)
    }
}

/// Fail-closed request or response bridge error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointBridgeError(pub String);

impl std::fmt::Display for EndpointBridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for EndpointBridgeError {}

/// A request dispatched on the owner thread. The owner must resolve
/// `peer_key` through its explicit peer-to-participant table; this crate does
/// not infer a participant from transport identity.
#[derive(Debug, Serialize)]
pub enum EndpointCall {
    Snapshot(ProjectionRequest),
    Resource(ResourceRequest),
    ResourceChunk(ResourceChunkRequest),
    Intent(IntentInvocation),
    Resume(ResumeRequest),
}

/// Typed endpoint answer; its variant must correspond to the request.
#[derive(Debug, Serialize)]
pub enum EndpointAnswer {
    Snapshot(ProjectionSnapshot),
    Resource(ResourceResponse),
    ResourceChunk(ResourceChunkResponse),
    Intent(IntentResult),
    Resume(ResumeReply),
}

/// One request for the owner-thread pump. Dropping it closes the response
/// path; the waiting carrier receives a bounded failure.
pub struct AdmittedEndpointRequest {
    pub peer_key: AuthenticatedPeerKey,
    pub call: EndpointCall,
    reply: SyncSender<Result<EndpointAnswer, String>>,
    cancelled: Arc<AtomicBool>,
}

impl std::fmt::Debug for AdmittedEndpointRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedEndpointRequest")
            .field("peer_key", &self.peer_key)
            .field("call", &self.call)
            .field("cancelled", &self.is_cancelled())
            .finish_non_exhaustive()
    }
}

impl AdmittedEndpointRequest {
    /// Whether the carrier stopped waiting before the owner began dispatch.
    /// The UI pump should discard canceled requests before calling a product
    /// endpoint. A cancellation racing an already-running synchronous intent
    /// cannot roll back product work; the endpoint's normal stale checks and
    /// accepted-record journal remain authoritative.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Return the actual product endpoint's answer to the admitted stream.
    pub fn respond(
        self,
        answer: Result<EndpointAnswer, String>,
    ) -> Result<(), EndpointBridgeError> {
        if let Ok(answer) = &answer {
            if !matches_pair(&self.call, answer) {
                return Err(EndpointBridgeError(
                    "owner-thread answer variant does not match the request".into(),
                ));
            }
        }
        self.reply
            .send(answer)
            .map_err(|_| EndpointBridgeError("carrier stopped waiting for endpoint answer".into()))
    }
}

/// UI-thread side of the bridge. Poll `try_next` during normal owner-thread
/// turns, dispatch each request to the already-open product endpoint, and
/// respond before returning to the event loop. `try_next` never waits.
pub struct AdmittedRequestPump {
    requests: Receiver<AdmittedEndpointRequest>,
    limits: EndpointBridgeLimits,
}

impl AdmittedRequestPump {
    /// Fetch one pending request without blocking the UI loop.
    pub fn try_next(&self) -> Result<Option<AdmittedEndpointRequest>, EndpointBridgeError> {
        match self.requests.try_recv() {
            Ok(request) => Ok(Some(request)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(EndpointBridgeError(
                "admitted stream request bridge is closed".into(),
            )),
        }
    }

    /// Maximum serialized payload accepted by this bridge.
    pub const fn max_message_bytes(&self) -> usize {
        self.limits.max_message_bytes
    }
}

/// Graphshell-facing endpoint proxy. It contains no World or product endpoint
/// and may be moved into the admitted carrier task.
pub struct BridgeEndpoint {
    peer_key: AuthenticatedPeerKey,
    descriptor: EndpointDescriptor,
    requests: SyncSender<AdmittedEndpointRequest>,
    limits: EndpointBridgeLimits,
    wake: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl BridgeEndpoint {
    /// Bind one already-admitted, carrier-authenticated peer to a bounded
    /// owner-thread request queue. Call only after Graphshell admission. The
    /// Graphshell session loop remains responsible for retained authority,
    /// expiry, revocation, and stream shutdown. Graphshell endpoint traits are
    /// synchronous: serve this proxy on a carrier worker, not on the UI thread
    /// or a current-thread executor that must also run UI/network progress.
    pub fn new(
        peer_key: AuthenticatedPeerKey,
        descriptor: EndpointDescriptor,
        limits: EndpointBridgeLimits,
    ) -> Result<(Self, AdmittedRequestPump), EndpointBridgeError> {
        Self::new_with_wake_inner(peer_key, descriptor, limits, None)
    }

    /// Like [`Self::new`], with an optional callback invoked after a request
    /// successfully enters the bounded owner-thread queue. The callback is a
    /// scheduling signal only: it receives no request data or authority and
    /// should return promptly. Full or closed queues do not invoke it.
    pub fn new_with_wake(
        peer_key: AuthenticatedPeerKey,
        descriptor: EndpointDescriptor,
        limits: EndpointBridgeLimits,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<(Self, AdmittedRequestPump), EndpointBridgeError> {
        Self::new_with_wake_inner(peer_key, descriptor, limits, Some(wake))
    }

    fn new_with_wake_inner(
        peer_key: AuthenticatedPeerKey,
        descriptor: EndpointDescriptor,
        limits: EndpointBridgeLimits,
        wake: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Result<(Self, AdmittedRequestPump), EndpointBridgeError> {
        let limits = limits.validate()?;
        check_size(&descriptor, limits.max_message_bytes, "endpoint descriptor")?;
        let (requests, receiver) = mpsc::sync_channel(limits.queue_capacity);
        Ok((
            Self {
                peer_key,
                descriptor,
                requests,
                limits,
                wake,
            },
            AdmittedRequestPump {
                requests: receiver,
                limits,
            },
        ))
    }

    /// Construct the per-stream bridge from a Graphshell session that has
    /// already cleared carrier admission. This is the narrow hook a transport
    /// accept loop uses before calling `serve_admitted_session`.
    pub fn for_admitted_session<S>(
        session: &AdmittedSession<S>,
        descriptor: EndpointDescriptor,
        limits: EndpointBridgeLimits,
    ) -> Result<(Self, AdmittedRequestPump), EndpointBridgeError> {
        let peer_key = AuthenticatedPeerKey::from_admitted_session(session)
            .map_err(|error| EndpointBridgeError(error.to_string()))?;
        Self::new(peer_key, descriptor, limits)
    }

    /// Construct an admitted-session bridge that wakes the owner after each
    /// successful enqueue. Use the host's coalesced wake callback so a queued
    /// request is serviced even when no unrelated input redraws the window.
    pub fn for_admitted_session_with_wake<S>(
        session: &AdmittedSession<S>,
        descriptor: EndpointDescriptor,
        limits: EndpointBridgeLimits,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<(Self, AdmittedRequestPump), EndpointBridgeError> {
        let peer_key = AuthenticatedPeerKey::from_admitted_session(session)
            .map_err(|error| EndpointBridgeError(error.to_string()))?;
        Self::new_with_wake(peer_key, descriptor, limits, wake)
    }

    /// Route a resume callback supplied to `serve_admitted_session` through
    /// the same owner-thread pump.
    pub fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, EndpointBridgeError> {
        match self.call(EndpointCall::Resume(request))? {
            EndpointAnswer::Resume(reply) => Ok(reply),
            _ => Err(EndpointBridgeError("invalid resume reply variant".into())),
        }
    }

    fn call(&self, call: EndpointCall) -> Result<EndpointAnswer, EndpointBridgeError> {
        check_size(&call, self.limits.max_message_bytes, "endpoint request")?;
        let expected = EndpointKind::of(&call);
        let (reply, response) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = AdmittedEndpointRequest {
            peer_key: self.peer_key,
            call,
            reply,
            cancelled: Arc::clone(&cancelled),
        };
        self.requests
            .try_send(request)
            .map_err(|error| match error {
                TrySendError::Full(_) => {
                    EndpointBridgeError("owner-thread request queue is full".into())
                },
                TrySendError::Disconnected(_) => {
                    EndpointBridgeError("owner-thread request pump is closed".into())
                },
            })?;
        if let Some(wake) = &self.wake {
            wake();
        }
        let answer = match response.recv_timeout(self.limits.response_timeout) {
            Ok(answer) => answer,
            Err(error) => {
                cancelled.store(true, Ordering::Release);
                return Err(EndpointBridgeError(format!(
                    "owner-thread endpoint reply: {error}"
                )));
            },
        }
        .map_err(EndpointBridgeError)?;
        if EndpointKind::of_answer(&answer) != expected {
            return Err(EndpointBridgeError("invalid endpoint answer".into()));
        }
        check_size(&answer, self.limits.max_message_bytes, "endpoint response")?;
        Ok(answer)
    }
}

impl ProjectionCatalog for BridgeEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        self.descriptor.clone()
    }
}

impl ProjectionSource for BridgeEndpoint {
    type Error = EndpointBridgeError;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        match self.call(EndpointCall::Snapshot(request))? {
            EndpointAnswer::Snapshot(snapshot) => Ok(snapshot),
            _ => Err(EndpointBridgeError("invalid snapshot reply variant".into())),
        }
    }
}

impl PresentationSource for BridgeEndpoint {
    type Error = EndpointBridgeError;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        match self.call(EndpointCall::Resource(request))? {
            EndpointAnswer::Resource(resource) => Ok(resource),
            _ => Err(EndpointBridgeError("invalid resource reply variant".into())),
        }
    }

    fn resource_chunk(
        &mut self,
        request: ResourceChunkRequest,
    ) -> Result<ResourceChunkResponse, Self::Error> {
        match self.call(EndpointCall::ResourceChunk(request))? {
            EndpointAnswer::ResourceChunk(chunk) => Ok(chunk),
            _ => Err(EndpointBridgeError(
                "invalid resource-chunk reply variant".into(),
            )),
        }
    }
}

impl IntentSink for BridgeEndpoint {
    type Error = EndpointBridgeError;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        match self.call(EndpointCall::Intent(intent))? {
            EndpointAnswer::Intent(result) => Ok(result),
            _ => Err(EndpointBridgeError("invalid intent reply variant".into())),
        }
    }
}

fn matches_pair(call: &EndpointCall, answer: &EndpointAnswer) -> bool {
    matches!(
        (call, answer),
        (EndpointCall::Snapshot(_), EndpointAnswer::Snapshot(_))
            | (EndpointCall::Resource(_), EndpointAnswer::Resource(_))
            | (
                EndpointCall::ResourceChunk(_),
                EndpointAnswer::ResourceChunk(_)
            )
            | (EndpointCall::Intent(_), EndpointAnswer::Intent(_))
            | (EndpointCall::Resume(_), EndpointAnswer::Resume(_))
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndpointKind {
    Snapshot,
    Resource,
    ResourceChunk,
    Intent,
    Resume,
}

impl EndpointKind {
    fn of(call: &EndpointCall) -> Self {
        match call {
            EndpointCall::Snapshot(_) => Self::Snapshot,
            EndpointCall::Resource(_) => Self::Resource,
            EndpointCall::ResourceChunk(_) => Self::ResourceChunk,
            EndpointCall::Intent(_) => Self::Intent,
            EndpointCall::Resume(_) => Self::Resume,
        }
    }

    fn of_answer(answer: &EndpointAnswer) -> Self {
        match answer {
            EndpointAnswer::Snapshot(_) => Self::Snapshot,
            EndpointAnswer::Resource(_) => Self::Resource,
            EndpointAnswer::ResourceChunk(_) => Self::ResourceChunk,
            EndpointAnswer::Intent(_) => Self::Intent,
            EndpointAnswer::Resume(_) => Self::Resume,
        }
    }
}

fn check_size(
    value: &impl Serialize,
    limit: usize,
    label: &str,
) -> Result<(), EndpointBridgeError> {
    let size = serde_json::to_vec(value)
        .map_err(|error| EndpointBridgeError(format!("serialize {label}: {error}")))?
        .len();
    if size > limit {
        return Err(EndpointBridgeError(format!(
            "{label} is {size} bytes; bridge limit is {limit}"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "endpoint_bridge_tests.rs"]
mod tests;
