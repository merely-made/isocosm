// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's played-turn endpoint (rulings 900 and 903): the retired
//! desktop shell's Mesocosm panel as a Graphshell projection over the core's
//! world. One turn card (settings, phase, happenings), with configure and
//! step where the loop allows them, and a card per answer the held turn
//! offers. Every answer carries the reading's cut, so a stale answer is
//! refused by the loop as well as by the revision. Only a player window
//! answers; the saved game lives in the world's `mesocosm` section.

use std::collections::BTreeMap;

use chirograph::{
    CarrierNotice, EndpointDescriptor, IntentEffect, IntentInvocation, IntentResult, ProjectionOffer,
    ProjectionRequest, ProjectionSnapshot, ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use cosmograph::directing::interim::Happening;
use cosmograph_host::{
    binding::Binding,
    endpoint::disclosure::{self, Disclosure, Item, NULL, action, card, rejected, value},
    window::Window,
    world::World,
};
use cosmograph_overlay::{
    AttentionSet, CandidateHandle, EntityHandle, ParticipantHandle, Seat, Tick,
    mesocosm::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, DeathAnswer, RevisionAnswer},
};

use crate::Occasion;
use crate::continuation::{LoopAction, LoopAnswer, LoopCut, LoopInput, LoopPhase, LoopReading,
    LoopSettings, PersonalQuestion};
use crate::spatial::{Game, Input};

pub const SESSION: &str = "mesocosm:played-turn";
/// The world section that holds the saved game.
pub const MODE: &str = crate::runtime::MODE;
pub const CONFIGURE: &str = "mesocosm.configure";
pub const STEP: &str = "mesocosm.step";
pub const ANSWER: &str = "mesocosm.answer";
const ADAPTER: &str = "mesocosm.played-turn";

type Sections = BTreeMap<String, Vec<u8>>;

fn game(sections: &Sections) -> Result<Game, String> {
    match sections.get(MODE) {
        Some(bytes) if !bytes.is_empty() => serde_json::from_slice(bytes).map_err(|e| e.to_string()),
        _ => Ok(Game::default()),
    }
}

fn with_game(sections: &Sections, game: &Game) -> Result<Sections, String> {
    let mut next = sections.clone();
    next.insert(MODE.into(), serde_json::to_vec(game).map_err(|e| e.to_string())?);
    Ok(next)
}

fn phase(phase: LoopPhase) -> &'static str {
    match phase {
        LoopPhase::Running => "Playing",
        LoopPhase::Held => "Waiting for your answer",
        LoopPhase::LineageEnded => "Your lineage has ended",
        LoopPhase::Watching => "Watching",
        LoopPhase::NewWorldRequested => "New world requested",
        LoopPhase::WorldEnded => "This world has ended",
    }
}

fn happening(happening: &Happening) -> String {
    match happening {
        Happening::Born { parent, child } => format!("Birth: member {parent} had member {child}"),
        Happening::Died { critter, next } => format!("Member {critter} died; next life: {next:?}"),
        Happening::Boundary { tick, .. } => format!("Epoch boundary at tick {tick}"),
        Happening::Collapsed { moved_to, .. } => format!("A region collapsed; next life: {moved_to:?}"),
        Happening::LineageEnded { lineage } => format!("Lineage {lineage} ended"),
        Happening::WorldEnded { tick } => format!("This world ended at tick {tick}"),
    }
}

/// The answers the held or ended turn offers, as the panel listed them.
fn answers(reading: &LoopReading, subject: Option<EntityHandle>) -> (Vec<(String, LoopAnswer)>, Vec<String>) {
    let checkpoint = |entity: u64, kind| LoopAnswer::Checkpoint(CheckpointAnswer { entity: EntityHandle(entity), kind });
    let (mut offered, mut withheld) = (Vec::new(), Vec::new());
    if reading.phase == LoopPhase::Held {
        if reading.checkpoint.is_none() && let Some(PersonalQuestion::Birth { parent }) = &reading.personal {
            offered.push(("Keep parent".into(), checkpoint(*parent, CheckpointAnswerKind::Birth(BirthAnswer::KeepParent))));
        }
        if let Some(held) = &reading.checkpoint {
            match &held.occasion {
                Occasion::Birth(birth) => for (label, kind) in
                    [("Keep parent", BirthAnswer::KeepParent), ("Take offspring", BirthAnswer::TakeOffspring)] {
                    offered.push((label.into(), checkpoint(birth.parent, CheckpointAnswerKind::Birth(kind))));
                },
                Occasion::Loss(loss) => for &heir in &held.heirs {
                    offered.push((format!("Take member {heir}"), checkpoint(loss.critter,
                        CheckpointAnswerKind::Death(DeathAnswer { next: EntityHandle(heir) }))));
                },
                Occasion::Epoch(_) => for offer in &reading.offers {
                    match (&offer.why_not, subject) {
                        (Some(reason), _) => withheld.push(format!("{}: {reason}", offer.name)),
                        (None, Some(subject)) => offered.push((format!("Choose {}", offer.name),
                            LoopAnswer::Checkpoint(CheckpointAnswer { entity: subject,
                                kind: CheckpointAnswerKind::EpochReview(RevisionAnswer {
                                    revision: CandidateHandle(offer.index as u64) }) }))),
                        (None, None) => {},
                    }
                },
            }
        }
    }
    if matches!(reading.phase, LoopPhase::LineageEnded | LoopPhase::Watching | LoopPhase::WorldEnded) {
        for &member in &reading.lineages {
            offered.push((format!("Play member {}", member.0), LoopAnswer::OtherLineage(member)));
        }
        if matches!(reading.phase, LoopPhase::LineageEnded | LoopPhase::WorldEnded) {
            offered.push(("Keep watching".into(), LoopAnswer::Watch));
        }
        offered.push(("Request a new world".into(), LoopAnswer::NewWorld));
    }
    if reading.phase == LoopPhase::NewWorldRequested {
        offered.push(("Keep watching this world".into(), LoopAnswer::Watch));
    }
    (offered, withheld)
}

pub struct MesocosmEndpoint {
    world: World,
    window: Window,
    disclosure: Disclosure,
    answers: BTreeMap<String, LoopAnswer>,
    cut: Option<LoopCut>,
}

impl MesocosmEndpoint {
    /// The played turn of `participant`, read and answered through a player window.
    pub fn open(world: &World, participant: ParticipantHandle) -> Result<Self, String> {
        let binding = Binding { mode: MODE.into(), controls: vec!["Played turn".into()],
            view_defaults: Vec::new(), attention: AttentionSet::default() };
        let window = Window::attach(world, participant, Seat::Player, binding)?;
        Ok(Self { world: world.share(), window, disclosure: Disclosure::new(SESSION, world),
            answers: BTreeMap::new(), cut: None })
    }

    pub fn world(&self) -> &World { &self.world }

    fn reading(&self) -> Result<LoopReading, String> {
        let cut = self.window.projection()?;
        self.world.read_sections(|session, sections| {
            let game = game(sections)?;
            if game.continuation.as_ref().is_some_and(|saved| !saved.queued.is_empty()) {
                return Err("This saved turn has pending standalone inputs. Resume it in the standalone host.".into());
            }
            crate::continuation::read_loop(session, &game, cut.access, &cut.visible)
        })?
    }

    fn items(&mut self, reading: &LoopReading) -> Result<Vec<Item>, String> {
        let mut values = vec![value("Phase", phase(reading.phase))];
        values.push(match &reading.settings {
            Some(s) => value("Settings", format!("{:?} · {} ticks per round · {:?}", s.mode, s.pace.round, s.on_collapse)),
            None => value("Settings", "Choose a loop setup: mode, pace and collapse choice"),
        });
        if let Some(tick) = reading.world_ended_at {
            values.push(value("World ended", format!("at tick {tick}; its history remains watchable")));
        }
        values.extend(reading.happenings.iter().enumerate().map(|(n, h)| value(format!("Happening {}", n + 1), happening(h))));
        let mut actions = Vec::new();
        if reading.settings.is_none() {
            actions.push(action(CONFIGURE, "Configure", "Set the loop's mode, pace and collapse choice.",
                r#"{"type":"object","required":["mode","pace","on_collapse"]}"#, IntentEffect::DomainTruth));
        } else if matches!(reading.phase, LoopPhase::Running | LoopPhase::Watching) {
            actions.push(action(STEP, "Step", "Advance the played loop one round.", NULL, IntentEffect::DomainTruth));
        }
        let mut items = vec![Item { id: "turn".into(), card: card("Played turn", values, &["turn"]), actions }];
        let subject = self.window.projection()?.access.subject;
        let (offered, withheld) = answers(reading, subject);
        self.answers.clear();
        for (n, (label, answer)) in offered.into_iter().enumerate() {
            let id = format!("answer:{n}");
            self.answers.insert(id.clone(), answer);
            items.push(Item { id, card: card(label.clone(), Vec::new(), &["answer"]),
                actions: vec![action(ANSWER, &label, "Answer the held turn.", NULL, IntentEffect::DomainTruth)] });
        }
        for (n, reason) in withheld.into_iter().enumerate() {
            items.push(Item { id: format!("withheld:{n}"), card: card(reason, Vec::new(), &["withheld"]),
                actions: Vec::new() });
        }
        Ok(items)
    }

    /// One loop input through the saved game, admitted with the native world.
    fn submit(&mut self, action: LoopAction) -> Result<IntentResult, String> {
        let expected = self.cut.clone().ok_or("no loop reading was disclosed")?;
        let access = self.window.projection()?.access;
        let before = self.window.cut()?;
        let input = Input::Loop(LoopInput { kind: "mesocosm:loop".into(), version: 1,
            participant: access.participant, tick: Tick(before.tick), expected, action });
        let result = self.world.product_sections(before, |session, sections| {
            let saved = game(sections)?;
            saved.version()?;
            let (_, saved) = crate::borrowed::submit_game(session, access, 0, &input, saved)
                .map_err(|e| format!("{e:?}"))?;
            Ok(((), with_game(sections, &saved)?))
        });
        Ok(match result { Ok(()) => self.disclosure.accepted(), Err(why) => rejected(why) })
    }

    /// One round of the played loop, by its trusted owner's player window.
    fn step(&mut self) -> Result<IntentResult, String> {
        let access = self.window.projection()?.access;
        let before = self.window.cut()?;
        let result = self.world.product_sections(before, |session, sections| {
            let mut game = game(sections)?;
            game.validate(session)?;
            let watching = game.continuation.as_ref().is_some_and(|saved|
                saved.phase == LoopPhase::Watching && saved.participant == access.participant.0);
            if !watching && (access.subject.is_none()
                || session.sim.plays(access.participant.0) != access.subject.map(|subject| subject.0)) {
                return Err("played subject changed before this loop step".into());
            }
            crate::continuation::advance_loop(session, &mut game, access.participant.0)
                .map_err(|e| format!("{e:?}"))?;
            Ok(((), with_game(sections, &game)?))
        });
        Ok(match result { Ok(()) => self.disclosure.accepted(), Err(why) => rejected(why) })
    }
}

impl ProjectionCatalog for MesocosmEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor { label: "Mesocosm".into(),
            projections: vec![ProjectionOffer { label: "Played turn".into(), request: disclosure::request(SESSION) }] }
    }
}

impl ProjectionSource for MesocosmEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        let reading = self.reading()?;
        let items = self.items(&reading)?;
        self.cut = Some(reading.cut);
        self.disclosure.snapshot(request, ADAPTER, &items, &[])
    }
}

impl PresentationSource for MesocosmEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.disclosure.resource(request)
    }
}

impl IntentSink for MesocosmEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        let source = match self.disclosure.admit(&intent)? {
            Ok(source) => source,
            Err(refused) => return Ok(refused),
        };
        match intent.intent.as_str() {
            CONFIGURE => match serde_json::from_slice::<LoopSettings>(&intent.payload) {
                Ok(settings) => self.submit(LoopAction::Configure(settings)),
                Err(why) => Ok(rejected(format!("a loop setup names mode, pace and collapse choice: {why}"))),
            },
            STEP => self.step(),
            ANSWER => match self.answers.get(&source).cloned() {
                Some(answer) => self.submit(LoopAction::Answer(answer)),
                None => Ok(rejected("this answer is no longer offered")),
            },
            _ => Ok(rejected("unknown Mesocosm endpoint intent")),
        }
    }
}

impl ProjectionNoticeSource for MesocosmEndpoint {
    type Error = String;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> { Ok(None) }
}
