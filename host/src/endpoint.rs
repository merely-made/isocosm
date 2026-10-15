// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The core's Graphshell inspection endpoint (rulings 896, 899 and 901).
//! One projection of the world: a world card, a card per site with its
//! routes, and the stored groups of an examined site. Examining a site or
//! inspecting a group is curation, stepping is domain truth, and saving is an
//! external effect. It reads native state as a creative observer, so it is
//! an inspection surface, never a player's view. `disclosure` is what the
//! product endpoints share with it.

mod cards;
pub mod disclosure;

use std::path::PathBuf;

use chirograph::{
    CarrierNotice, EndpointDescriptor, IntentInvocation, IntentResult, ProjectionOffer,
    ProjectionRequest, ProjectionSnapshot, ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};

use crate::world::World;
use disclosure::{Disclosure, rejected};

pub const SESSION: &str = "isocosm:world";
pub const EXAMINE: &str = "isocosm.examine-site";
pub const INSPECT: &str = "isocosm.inspect-group";
pub const STEP: &str = "isocosm.step";
pub const SAVE: &str = "isocosm.save";
const ADAPTER: &str = "isocosm.world";

/// What the inspector has examined, kept between snapshots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Curation {
    pub site: Option<u64>,
    pub group: Option<u64>,
}

pub struct SimEndpoint {
    world: World,
    save_to: Option<PathBuf>,
    curation: Curation,
    disclosure: Disclosure,
}

impl SimEndpoint {
    pub fn new(world: World, save_to: Option<PathBuf>) -> Self {
        Self { world, save_to, curation: Curation::default(), disclosure: Disclosure::new(SESSION) }
    }

    pub fn world(&self) -> &World { &self.world }
    pub fn curation(&self) -> Curation { self.curation }
}

/// The ticks a step asks for: an empty payload is one, else `{"ticks": n}`.
fn ticks(payload: &[u8]) -> Result<u64, String> {
    if payload.is_empty() { return Ok(1); }
    let value: serde_json::Value = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    match value.get("ticks").and_then(serde_json::Value::as_u64) {
        Some(n) if n > 0 => Ok(n),
        _ => Err("a step names a positive number of ticks".into()),
    }
}

impl ProjectionCatalog for SimEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Isocosm".into(),
            projections: vec![ProjectionOffer { label: "World".into(), request: disclosure::request(SESSION) }],
        }
    }
}

impl ProjectionSource for SimEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        let curation = self.curation;
        let (items, links) = self.world.read(|s| (cards::items(s, curation), cards::links(s, curation)))?;
        self.disclosure.snapshot(request, ADAPTER, &items, &links)
    }
}

impl PresentationSource for SimEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.disclosure.resource(request)
    }
}

impl IntentSink for SimEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        let source = match self.disclosure.admit(&intent)? {
            Ok(source) => source,
            Err(refused) => return Ok(refused),
        };
        let id = |prefix: &str| source.strip_prefix(prefix).and_then(|n| n.parse::<u64>().ok());
        Ok(match intent.intent.as_str() {
            EXAMINE => match id("site:") {
                Some(site) => {
                    self.curation = Curation { site: Some(site), group: None };
                    self.disclosure.accepted()
                },
                None => rejected("only a site is examined"),
            },
            INSPECT => match id("group:") {
                Some(group) => {
                    self.curation.group = Some(group);
                    self.disclosure.accepted()
                },
                None => rejected("only a stored group is inspected"),
            },
            STEP => match ticks(&intent.payload).and_then(|n| self.world.advance(n)) {
                Ok(()) => self.disclosure.accepted(),
                Err(why) => rejected(why),
            },
            SAVE => match &self.save_to {
                None => rejected("this endpoint was opened without a save path"),
                Some(path) => match self.world.save().and_then(|save| save.encode())
                    .and_then(|bytes| crate::file_save::replace(path, &bytes).map_err(|e| e.to_string())) {
                    Ok(()) => IntentResult::Accepted,
                    Err(why) => rejected(why),
                },
            },
            _ => rejected("unknown Isocosm endpoint intent"),
        })
    }
}

impl ProjectionNoticeSource for SimEndpoint {
    type Error = String;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> { Ok(None) }
}
