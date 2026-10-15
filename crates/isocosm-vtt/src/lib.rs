// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Isocosm: VTT**, the virtual tabletop of the Isocosm family (formerly
//! Isometry; renamed 2026-09-22, wing design record rulings 110 and 111).
//! The implementation is the `isometry-*` crates of this workspace.
//!
//! This crate holds the VTT's Graphshell endpoint over the sim-host core
//! (rulings 900, 902 and 904): the retired desktop shell's six VTT panels as
//! six projections of one table seat. Table time and party, joint journeys,
//! the disclosed overmap, held map knowledge, adventures and goals, and
//! native operation authoring. Every card's actions bind to an [`Act`]; a
//! world change goes through `isometry_campaign::native::submit_game_access`
//! with the VTT's game bytes in the world's `vtt` section.

mod adventure;
mod authoring;
mod journeys;
mod knowledge;
mod overmap;
mod table;

use std::collections::BTreeMap;

use chirograph::{
    CarrierNotice, EndpointDescriptor, IntentInvocation, IntentResult, ProjectionOffer,
    ProjectionRequest, ProjectionSnapshot, ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use isocosm_host::{
    binding::Binding,
    endpoint::disclosure::{self, Disclosure, Item, rejected},
    window::{Projection, Window},
    world::World,
};
use isocosm_overlay::{
    AttentionChange, AttentionSet, Intent, ParticipantHandle, PlaceHandle, Seat, Tick,
    vtt::{Assertion, HookIntent, VttIntent, VttIntentEnvelope},
};
use isometry_campaign::native::{joint_journey::{JourneyEnvelope, JourneyInput}, overmap::ViewSettings,
    pack::{PackEnvelope, PackInput}};

/// The world section that holds the VTT's game bytes.
pub const MODE: &str = "vtt";
pub const TABLE: &str = "vtt:table";
pub const JOURNEYS: &str = "vtt:journeys";
pub const OVERMAP: &str = "vtt:overmap";
pub const KNOWLEDGE: &str = "vtt:knowledge";
pub const ADVENTURE: &str = "vtt:adventure";
pub const AUTHORING: &str = "vtt:authoring";
const SESSIONS: [(&str, &str); 6] = [(TABLE, "Table time and party"), (JOURNEYS, "Joint journeys"),
    (OVERMAP, "Disclosed overmap"), (KNOWLEDGE, "Held map knowledge"),
    (ADVENTURE, "Adventures and current goals"), (AUTHORING, "Native operation authoring")];

/// What one advertised action does when invoked.
#[derive(Clone, Debug)]
pub(crate) enum Act {
    /// Supplied game input bytes, or the intent's own payload when empty.
    Submit(Vec<u8>),
    /// A VTT intent in an envelope at the current tick.
    Send(VttIntent),
    Journey(JourneyInput),
    Pack(PackInput),
    Hook(HookIntent),
    /// A reshaped goal; the note is the intent's payload, a JSON string.
    Reshape { arc: isocosm_overlay::EventHandle, goal: String },
    /// Toggle one player in the saved roster, or configure an empty one.
    Roster(Option<u64>),
    /// Share the played body's known map with the party.
    Disclose { party: String, member: u64 },
    SelectParty(String),
    Overmap(Option<ViewSettings>),
    /// Preview a supplied operation of this kind; the operation is the payload.
    Quote(u8),
    /// Examine a site inside the seat's cut, or stop examining.
    Examine(Option<u64>),
    /// The DM's table contract assertions; the payload is their JSON array.
    Assert,
}

/// A session's cards and the acts their actions bind.
pub(crate) struct Built {
    pub items: Vec<Item>,
    pub acts: BTreeMap<(String, String), Act>,
    pub links: Vec<(String, String, &'static str)>,
}

impl Built {
    pub fn new() -> Self { Self { items: Vec::new(), acts: BTreeMap::new(), links: Vec::new() } }
    pub fn push(&mut self, item: Item, acts: Vec<Act>) {
        for (action, act) in item.actions.iter().zip(acts) {
            self.acts.insert((item.id.clone(), action.intent.0.clone()), act);
        }
        self.items.push(item);
    }
}

/// What a panel reads: the window's cut and the endpoint's local choices.
pub(crate) struct Cx<'a> {
    pub cut: Projection,
    pub party: Option<&'a str>,
    pub overmap: Option<&'a ViewSettings>,
    pub quote: Option<&'a str>,
}

pub struct VttEndpoint {
    world: World,
    window: Window,
    party: Option<String>,
    overmap: Option<ViewSettings>,
    quote: Option<String>,
    disclosures: BTreeMap<&'static str, Disclosure>,
    acts: BTreeMap<&'static str, BTreeMap<(String, String), Act>>,
}

fn game(sections: &BTreeMap<String, Vec<u8>>) -> Vec<u8> { sections.get(MODE).cloned().unwrap_or_default() }

impl VttEndpoint {
    /// `participant`'s table at `seat` (the DM or a player).
    pub fn open(world: &World, participant: ParticipantHandle, seat: Seat) -> Result<Self, String> {
        let binding = Binding { mode: MODE.into(), controls: SESSIONS.iter().map(|(_, l)| (*l).into()).collect(),
            view_defaults: Vec::new(), attention: AttentionSet::default() };
        let window = Window::attach(world, participant, seat, binding)?;
        Ok(Self { world: world.share(), window, party: None, overmap: None, quote: None,
            disclosures: SESSIONS.iter().map(|(s, _)| (*s, Disclosure::new(s))).collect(), acts: BTreeMap::new() })
    }

    pub fn world(&self) -> &World { &self.world }
    pub fn party(&self) -> Option<&str> { self.party.as_deref() }
    pub fn examined(&self) -> Option<u64> { self.window.binding().attention.examined.map(|place| place.0) }

    /// The VTT's game bytes, read at the current cut.
    pub(crate) fn read<T>(&self, read: impl FnOnce(&isocosm::Session, &[u8]) -> Result<T, String>) -> Result<T, String> {
        self.world.read_sections(|session, sections| read(session, &game(sections)))?
    }

    fn build(&self, session: &str) -> Result<Built, String> {
        let cx = Cx { cut: self.window.projection()?, party: self.party.as_deref(),
            overmap: self.overmap.as_ref(), quote: self.quote.as_deref() };
        match session {
            TABLE => table::build(self, &cx),
            JOURNEYS => journeys::build(self, &cx),
            OVERMAP => overmap::build(self, &cx),
            KNOWLEDGE => knowledge::build(self, &cx),
            ADVENTURE => adventure::build(self, &cx),
            AUTHORING => authoring::build(self, &cx),
            _ => Err("projection request names an unknown VTT session".into()),
        }
    }

    /// Game input bytes through the campaign's one admission path.
    fn submit(&mut self, bytes: &[u8]) -> Result<(), String> {
        let access = self.window.projection()?.access;
        let before = self.window.cut()?;
        self.world.product_sections(before, |session, sections| {
            let (_, next) = isometry_campaign::native::submit_game_access(session, game(sections), bytes, access)?;
            let mut sections = sections.clone();
            sections.insert(MODE.into(), next);
            Ok(((), sections))
        })
    }

    fn envelope<T: serde::Serialize>(&self, make: impl FnOnce(u64, u64) -> T) -> Result<Vec<u8>, String> {
        let access = self.window.projection()?.access;
        let tick = self.window.cut()?.tick;
        serde_json::to_vec(&make(access.participant.0, tick)).map_err(|e| e.to_string())
    }

    fn act(&mut self, act: Act, payload: &[u8]) -> Result<(), String> {
        match act {
            Act::Submit(bytes) => self.submit(if bytes.is_empty() { payload } else { &bytes }),
            Act::Send(intent) => {
                let bytes = self.envelope(|participant, tick| VttIntentEnvelope {
                    participant: ParticipantHandle(participant), tick: Tick(tick), intent: Intent::Game(intent) })?;
                self.submit(&bytes)
            },
            Act::Journey(intent) => {
                let bytes = self.envelope(|participant, tick| JourneyEnvelope { participant, tick, intent })?;
                self.submit(&bytes)
            },
            Act::Pack(intent) => {
                let bytes = self.envelope(|participant, tick| PackEnvelope { participant, tick, intent })?;
                self.submit(&bytes)
            },
            Act::Hook(intent) => self.act(Act::Send(VttIntent::Hook(intent)), payload),
            Act::Reshape { arc, goal } => {
                let note: String = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
                self.act(Act::Send(VttIntent::Hook(HookIntent::ReshapeGoal { arc, goal, note })), payload)
            },
            Act::Roster(player) => {
                let bytes = table::roster(self, player)?;
                self.submit(&bytes)
            },
            Act::Disclose { party, member } => {
                let participant = self.window.projection()?.access.participant.0;
                let input = self.world.read(|session|
                    isometry_campaign::native::party::disclose_known(session, participant, party))??;
                if input.member.0 != member { return Err("played body changed before map disclosure".into()); }
                self.act(Act::Send(VttIntent::Party(input)), payload)
            },
            Act::SelectParty(party) => { self.party = Some(party); Ok(()) },
            Act::Overmap(settings) => { self.overmap = settings; Ok(()) },
            Act::Quote(kind) => { self.quote = Some(authoring::quote(self, kind, payload)?); Ok(()) },
            Act::Assert => {
                if self.window.projection()?.access.seat != Seat::Dm {
                    return Err("native table assertions require the DM seat".into());
                }
                let assertions: Vec<Assertion> = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
                let before = self.window.cut()?;
                self.world.product_sections(before, |session, sections| {
                    isometry_campaign::native::submit_contract_assertions(session, &assertions)
                        .map_err(|e| e.to_string())?;
                    Ok(((), sections.clone()))
                })
            },
            Act::Examine(site) => {
                if site.is_some_and(|site| !self.window.projection().is_ok_and(|cut| cut.visible.sites.contains(&site))) {
                    return Err("region is outside the seat cut".into());
                }
                let change = site.map_or(AttentionChange::StopExamining, |site| AttentionChange::Examine(PlaceHandle(site)));
                let request = self.window.window_attention_request(change)?;
                self.window.apply_attention(request)
            },
        }
    }
}

impl ProjectionCatalog for VttEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor { label: "Isocosm: VTT".into(), projections: SESSIONS.iter()
            .map(|(session, label)| ProjectionOffer { label: (*label).into(), request: disclosure::request(session) })
            .collect() }
    }
}

impl ProjectionSource for VttEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        let (&session, _) = self.disclosures.get_key_value(request.session.0.as_str())
            .ok_or("projection request names an unknown VTT session")?;
        let built = self.build(session)?;
        self.acts.insert(session, built.acts);
        let adapter = format!("isocosm-vtt.{session}");
        self.disclosures.get_mut(session).expect("known session").snapshot(request, &adapter, &built.items, &built.links)
    }
}

impl PresentationSource for VttEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.disclosures.get(request.session.0.as_str()).ok_or("resource names an unknown VTT session")?
            .resource(request)
    }
}

impl IntentSink for VttEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        let (&session, disclosure) = self.disclosures.get_key_value(intent.session.0.as_str())
            .ok_or("intent names an unknown VTT session")?;
        let source = match disclosure.admit(&intent)? {
            Ok(source) => source,
            Err(refused) => return Ok(refused),
        };
        let Some(act) = self.acts.get(session).and_then(|acts| acts.get(&(source, intent.intent.clone()))).cloned()
        else { return Ok(rejected("unknown VTT endpoint intent")) };
        // One table seat: a change any projection makes stales them all.
        Ok(match self.act(act, &intent.payload) {
            Ok(()) => {
                for disclosure in self.disclosures.values_mut() { disclosure.accepted(); }
                IntentResult::Accepted
            },
            Err(why) => rejected(why),
        })
    }
}

impl ProjectionNoticeSource for VttEndpoint {
    type Error = String;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> { Ok(None) }
}
