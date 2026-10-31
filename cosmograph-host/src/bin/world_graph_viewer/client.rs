// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The viewer as a Graphshell client of the core's endpoint (Host 2): the
//! endpoint is carried in process, every request round-trips the wire
//! encoding, and what the viewer shows and offers is only what the world
//! graph's snapshot disclosed.

use std::collections::HashSet;

use chirograph::{
    AdvertisedAction, Carrier, CarrierRequestBody, CarrierResponseBody, IntentInvocation,
    IntentResult, PortableCardV1, ProjectionRequest, ProjectionSnapshot, ResourceRequest,
    ResumeReply, ResumeRequest,
};
use cosmograph_host::endpoint::{GRAPH, SimEndpoint};
use graphshell_client::{ClientState, MountedScene};
use graphshell_endpoint::local::LocalCarrier;
use sceno::{InstanceId, StandIn};

type Resume = fn(&mut SimEndpoint, ResumeRequest) -> Result<ResumeReply, String>;

fn no_resume(_: &mut SimEndpoint, _: ResumeRequest) -> Result<ResumeReply, String> {
    Err("the viewer does not resume".into())
}

/// One disclosed item as the viewer reads it.
pub struct Shown {
    pub id: String,
    pub instance: InstanceId,
    pub card: PortableCardV1,
    pub actions: Vec<AdvertisedAction>,
    /// Behind a fold's stand-in.
    pub hidden: bool,
}

impl Shown {
    pub fn value(&self, label: &str) -> Option<&str> {
        self.card.values.iter().find(|v| v.label == label).map(|v| v.value.as_str())
    }
}

pub struct Client {
    carrier: LocalCarrier<SimEndpoint, Resume>,
    request: ProjectionRequest,
    state: ClientState,
    snapshot: Option<ProjectionSnapshot>,
    pub shown: Vec<Shown>,
    /// The last refusal or failure, for the panel.
    pub notice: String,
}

impl Client {
    pub fn new(endpoint: SimEndpoint) -> Result<Self, String> {
        let mut carrier = LocalCarrier::new(endpoint, no_resume as Resume);
        let CarrierResponseBody::Descriptor(descriptor) = carrier.request(CarrierRequestBody::Discover)
            .map_err(|e| format!("{e:?}"))? else { return Err("discovery answered no descriptor".into()) };
        let request = descriptor.projections.into_iter().find(|p| p.request.session.0 == GRAPH)
            .ok_or("the endpoint offers no world graph")?.request;
        let mut client = Self { carrier, request, state: ClientState::default(), snapshot: None,
            shown: Vec::new(), notice: String::new() };
        client.refresh()?;
        Ok(client)
    }

    pub fn mounted(&self) -> Option<&MountedScene> { self.state.mounted(&self.request.session) }

    /// The acknowledged revision, which the board keys its resync on.
    pub fn revision(&self) -> Option<u64> { self.snapshot.as_ref().map(|s| s.scene.revision.0) }

    pub fn world(&self) -> Option<&Shown> { self.shown.iter().find(|s| s.id == "world") }

    /// Takes a fresh snapshot and reads every disclosed card.
    pub fn refresh(&mut self) -> Result<(), String> {
        let CarrierResponseBody::Snapshot(snapshot) = self.carrier
            .request(CarrierRequestBody::Snapshot(self.request.clone())).map_err(|e| format!("{e:?}"))?
            else { return Err("a snapshot request answered no snapshot".into()) };
        let snapshot = *snapshot;
        self.state.apply_snapshot(snapshot.clone()).map_err(|e| format!("{e:?}"))?;
        let hidden: HashSet<u32> = snapshot.scene.tables.folds.iter().flatten().flat_map(|fold| {
            let stand_in = match fold.stand_in { StandIn::Member(s) => Some(s), StandIn::Summary { .. } => None };
            fold.members.iter().copied().filter(move |m| Some(*m) != stand_in).map(|m| m.0)
        }).collect();
        let mut shown = Vec::new();
        for (instance, item) in snapshot.scene.active_items_in_order() {
            let Some(Some(source)) = snapshot.scene.tables.sources.get(item.source.0 as usize) else { continue };
            let Some(offer) = snapshot.presentation.offers_for(instance).and_then(|o| o.first()) else { continue };
            let CarrierResponseBody::Resource(response) = self.carrier.request(CarrierRequestBody::Resource(
                ResourceRequest { session: snapshot.session.clone(), resource: offer.resource }))
                .map_err(|e| format!("{e:?}"))? else { return Err("a resource request answered no resource".into()) };
            let card: PortableCardV1 = serde_json::from_slice(&response.bytes).map_err(|e| e.to_string())?;
            shown.push(Shown { id: source.id.clone(), instance, card, actions: offer.semantics.actions.clone(),
                hidden: hidden.contains(&instance.0) });
        }
        self.shown = shown;
        self.snapshot = Some(snapshot);
        Ok(())
    }

    /// Invokes `intent` on the item `id` against the last snapshot, then
    /// refreshes; a refusal is kept as the notice.
    pub fn invoke(&mut self, id: &str, intent: &str, payload: &[u8]) {
        let result = (|| {
            let snapshot = self.snapshot.as_ref().ok_or("no snapshot yet")?;
            let target = self.shown.iter().find(|s| s.id == id).ok_or_else(|| format!("{id} is not shown"))?.instance;
            let invocation = IntentInvocation { session: snapshot.session.clone(), target,
                observed_epoch: snapshot.scene.epoch, observed_revision: snapshot.scene.revision,
                intent: intent.into(), payload: payload.to_vec() };
            match self.carrier.request(CarrierRequestBody::Intent(invocation)).map_err(|e| format!("{e:?}"))? {
                CarrierResponseBody::Intent(IntentResult::Accepted) => Ok(String::new()),
                CarrierResponseBody::Intent(IntentResult::Rejected { reason }) => Ok(reason),
                CarrierResponseBody::Intent(other) => Ok(format!("{other:?}")),
                _ => Err("an intent answered no result".to_string()),
            }
        })();
        self.notice = match result {
            Ok(notice) => notice,
            Err(why) => why,
        };
        if let Err(why) = self.refresh() { self.notice = why; }
    }
}
