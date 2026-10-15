// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym's next-life endpoint (rulings 900 and 903): the retired desktop
//! shell's Eponym panel and stepping as a Graphshell projection over the
//! core's world. An Eponym card, a card per visible body, and, once the
//! player's taken body has died, a card per eligible next life. Stepping
//! keeps a configured Eponym clock; without one it advances one tick. The
//! game lives in the world's `eponym` section as the mode state's bytes.

use std::collections::BTreeMap;

use chirograph::{
    CarrierNotice, EndpointDescriptor, IntentEffect, IntentInvocation, IntentResult, ProjectionOffer,
    ProjectionRequest, ProjectionSnapshot, ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use isocosm::{Command, Session};
use isocosm_host::{
    binding::Binding,
    endpoint::disclosure::{self, Disclosure, Item, NULL, action, card, rejected, value},
    window::Window,
    world::World,
};
use isocosm_overlay::{AttentionSet, ParticipantHandle, Seat};

use crate::contract::{Reading, mode::{ModeState, clock, life}};

pub const SESSION: &str = "eponym:next-life";
/// The world section that holds the mode state.
pub const MODE: &str = "eponym";
pub const STEP: &str = "eponym.step";
pub const TAKE: &str = "eponym.take-life";
const ADAPTER: &str = "eponym.next-life";

type Sections = BTreeMap<String, Vec<u8>>;

fn game(sections: &Sections) -> Vec<u8> { sections.get(MODE).cloned().unwrap_or_default() }

fn with_game(sections: &Sections, game: Vec<u8>) -> Sections {
    let mut next = sections.clone();
    next.insert(MODE.into(), game);
    next
}

/// Whether the player's last taken body has died, awaiting a next life.
fn awaiting(session: &Session, participant: u64) -> bool {
    life::previous_take(session, participant).and_then(|entry| match &entry.command {
        Command::Take { critter, .. } => session.sim.state().population.get(*critter).map(|body| !body.alive),
        _ => None,
    }).unwrap_or(false)
}

/// The ordinary clock: a configured Eponym clock keeps its physical state.
fn advance(session: &mut Session, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let Some(mut mode) = ModeState::decode(&bytes)? else { session.advance(1)?; return Ok(bytes) };
    mode.validate_native(session)?;
    if mode.clock.is_none() { session.advance(1)?; return Ok(bytes); }
    let mut actions = mode.actions.take().ok_or("Eponym clock requires its configured action profiles")?;
    clock::advance(session, &mut mode, &mut actions)?;
    mode.actions = Some(actions);
    mode.validate()?;
    serde_json::to_vec(&mode).map_err(|e| e.to_string())
}

pub struct EponymEndpoint {
    world: World,
    window: Window,
    disclosure: Disclosure,
    lives: BTreeMap<String, life::Input>,
}

impl EponymEndpoint {
    /// `participant`'s Eponym view, read and acted through a player window.
    pub fn open(world: &World, participant: ParticipantHandle) -> Result<Self, String> {
        let binding = Binding { mode: MODE.into(), controls: vec!["Next life".into()],
            view_defaults: Vec::new(), attention: AttentionSet::default() };
        let window = Window::attach(world, participant, Seat::Player, binding)?;
        Ok(Self { world: world.share(), window, disclosure: Disclosure::new(SESSION), lives: BTreeMap::new() })
    }

    pub fn world(&self) -> &World { &self.world }

    fn items(&mut self) -> Result<Vec<Item>, String> {
        let cut = self.window.projection()?;
        let access = cut.access;
        let (bodies, waiting, choices) = self.world.read_sections(|session, sections| {
            let bodies = Reading::new(session, &cut.visible).bodies();
            let waiting = awaiting(session, access.participant.0);
            let choices = waiting.then(|| life::death_choices(session, &game(sections), access, &cut.visible));
            (bodies, waiting, choices)
        })?;
        let state = if waiting { "Awaiting a next life" } else { "Living" };
        let mut items = vec![Item { id: "eponym".into(),
            card: card("Eponym", vec![value("Life", state), value("Visible bodies", bodies.len())], &["eponym"]),
            actions: vec![action(STEP, "Step", "Advance the world, keeping a configured Eponym clock.",
                NULL, IntentEffect::DomainTruth)] }];
        for (id, name, parts) in bodies {
            items.push(Item { id: format!("body:{id}"),
                card: card(name, vec![value("Body", id), value("Parts", parts)], &["body"]), actions: Vec::new() });
        }
        self.lives.clear();
        match choices {
            None => {},
            Some(Err(reason)) => items.push(Item { id: "next-life".into(),
                card: card("Next life", vec![value("Refused", reason)], &["next-life"]), actions: Vec::new() }),
            Some(Ok(choices)) if choices.is_empty() => items.push(Item { id: "next-life".into(),
                card: card("Next life", vec![value("Choices", "No eligible next life is visible in this window.")],
                    &["next-life"]), actions: Vec::new() }),
            Some(Ok(choices)) => for input in choices {
                let id = format!("life:{}", input.subject.0);
                let label = format!("Take subject {}", input.subject.0);
                items.push(Item { id: id.clone(), card: card(label.clone(), Vec::new(), &["next-life"]),
                    actions: vec![action(TAKE, &label, "Take this subject as your next life.", NULL,
                        IntentEffect::DomainTruth)] });
                self.lives.insert(id, input);
            },
        }
        Ok(items)
    }

    fn take(&mut self, input: life::Input) -> Result<IntentResult, String> {
        let access = self.window.projection()?.access;
        let before = self.window.cut()?;
        let bytes = serde_json::to_vec(&input).map_err(|e| e.to_string())?;
        let result = self.world.product_sections(before, |session, sections| {
            let (_, next) = life::submit_bytes(session, game(sections), &bytes, access)?;
            Ok(((), with_game(sections, next)))
        });
        Ok(match result { Ok(()) => self.disclosure.accepted(), Err(why) => rejected(why) })
    }

    fn step(&mut self) -> Result<IntentResult, String> {
        let before = self.window.cut()?;
        let result = self.world.product_sections(before, |session, sections| {
            let next = advance(session, game(sections))?;
            Ok(((), with_game(sections, next)))
        });
        Ok(match result { Ok(()) => self.disclosure.accepted(), Err(why) => rejected(why) })
    }
}

impl ProjectionCatalog for EponymEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor { label: "Eponym".into(),
            projections: vec![ProjectionOffer { label: "Next life".into(), request: disclosure::request(SESSION) }] }
    }
}

impl ProjectionSource for EponymEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        let items = self.items()?;
        self.disclosure.snapshot(request, ADAPTER, &items, &[])
    }
}

impl PresentationSource for EponymEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.disclosure.resource(request)
    }
}

impl IntentSink for EponymEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        let source = match self.disclosure.admit(&intent)? {
            Ok(source) => source,
            Err(refused) => return Ok(refused),
        };
        match intent.intent.as_str() {
            STEP => self.step(),
            TAKE => match self.lives.get(&source).cloned() {
                Some(input) => self.take(input),
                None => Ok(rejected("this next life is no longer offered")),
            },
            _ => Ok(rejected("unknown Eponym endpoint intent")),
        }
    }
}

impl ProjectionNoticeSource for EponymEndpoint {
    type Error = String;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> { Ok(None) }
}
