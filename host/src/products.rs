// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::Session;
use isocosm_overlay::{AttentionSet, Seat};
use serde::{Deserialize, Serialize};
use crate::{binding::Binding, window::Projection};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode { Mesocosm, Vtt, Eponym }

impl Mode {
    pub fn key(self) -> &'static str { match self {
        Self::Mesocosm => "mesocosm", Self::Vtt => "vtt", Self::Eponym => "eponym",
    } }
    pub fn parse(key: &str) -> Result<Self, String> { match key {
        "mesocosm" => Ok(Self::Mesocosm), "vtt" => Ok(Self::Vtt), "eponym" => Ok(Self::Eponym),
        _ => Err("unknown play mode".into()),
    } }
    pub fn available(self) -> bool { match self {
        Self::Mesocosm => cfg!(feature = "mesocosm"), Self::Vtt => cfg!(feature = "vtt"),
        Self::Eponym => cfg!(feature = "eponym"),
    } }
    pub fn enabled() -> Vec<Self> {
        [Self::Mesocosm, Self::Vtt, Self::Eponym].into_iter().filter(|m| m.available()).collect()
    }
    pub fn binding(self, view_defaults: Vec<u8>, attention: AttentionSet) -> Result<Binding, String> {
        if !self.available() { return Err("mode is absent from this build".into()); }
        let controls: &[&str] = match self {
            Self::Mesocosm => &["Attend", "Send Mesocosm intent"],
            Self::Vtt => &["Read table", "Send native table intent", "Bind saved players", "Party map", "Downtime consent"],
            Self::Eponym => &["Read bodies", "Send Eponym intent"],
        };
        Ok(Binding { mode: self.key().into(), controls: controls.iter().map(|s| (*s).into()).collect(),
            view_defaults, attention })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Rows { pub things: Vec<String>, pub sites: Vec<String>, pub events: Vec<String> }

/// Every ordinary product view receives the same already resolved seat cut.
pub(crate) fn read(session: &Session, cut: &Projection, mode: Mode) -> Result<Rows, String> {
    if !mode.available() { return Err("mode is absent from this build".into()); }
    let mut rows = Rows { sites: cut.visible.sites.iter().map(|id| format!("Site {id}")).collect(),
        ..Rows::default() };
    match mode {
        #[cfg(feature = "mesocosm")]
        Mode::Mesocosm => {
            let reading = mesocosm_runtime::borrowed::Reading::new(session, &cut.visible);
            rows.things = reading.things().into_iter().map(|(id, count, lineage)|
                format!("{lineage}: {count} at {id}")).collect();
        },
        #[cfg(feature = "vtt")]
        Mode::Vtt => {
            let reading = isometry_campaign::native::WindowReading::new(session, &cut.visible);
            rows.things = reading.characters().into_iter().map(|(key, id)| format!("{key} ({id})")).collect();
            rows.sites = reading.sites().into_iter().map(|(key, id)| format!("{key} ({id})")).collect();
        },
        #[cfg(feature = "eponym")]
        Mode::Eponym => {
            let reading = eponym_play::contract::Reading::new(session, &cut.visible);
            rows.things = reading.bodies().into_iter().map(|(id, name, parts)|
                format!("{name} ({id}): {parts} parts")).collect();
        },
        _ => return Err("mode is absent from this build".into()),
    }
    rows.events = crate::stream::lines(session, cut)?;
    Ok(rows)
}

/// Product-only saved bindings are read beside this same native cut.
pub(crate) fn read_game(session: &Session, cut: &Projection, mode: Mode, bytes: &[u8]) -> Result<Rows, String> {
    let mut rows = read(session, cut, mode)?;
    #[cfg(feature = "eponym")]
    if mode == Mode::Eponym {
        if let Some(state) = eponym_play::contract::mode::ModeState::decode(bytes)? {
            let reading = eponym_play::contract::mode::BorrowedReading::new(session, &cut.visible, &state)?;
            rows.things.extend(reading.rows().into_iter().map(|(subject, entity, name)|
                format!("{name}: Eponym subject {} bound to native body {entity}", subject.0)));
        }
    }
    #[cfg(not(feature = "eponym"))]
    let _ = bytes;
    Ok(rows)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SiteRegion { pub site: u64, pub min: [i64; 2], pub max: [i64; 2], pub depth: i64 }

#[derive(Clone, Debug)]
pub(crate) struct Input { pub mode: Mode, pub bytes: Vec<u8>, pub round: u64, pub region: Option<SiteRegion> }

pub(crate) fn submit(session: &mut Session, cut: &Projection, mode: Mode, input: &Input)
    -> Result<String, String> {
    current(session, cut, mode, input)?;
    match mode {
        #[cfg(feature = "mesocosm")]
        Mode::Mesocosm => {
            let envelope: isocosm_overlay::mesocosm::MesocosmIntentEnvelope =
                serde_json::from_slice(&input.bytes).map_err(|e| e.to_string())?;
            if matches!(&envelope.intent, isocosm_overlay::Intent::Game(intent) if intent.is_dev()) {
                return Err("dev intents require the separate assisted-run door".into());
            }
            mesocosm_runtime::borrowed::submit(session, cut.access.participant.0, input.round, &envelope)
                .map_err(|e| format!("{e:?}"))
        },
        #[cfg(feature = "vtt")]
        Mode::Vtt => {
            if cut.access.seat != Seat::Dm { return Err("native table assertions require the DM seat".into()); }
            let assertions: Vec<isocosm_overlay::vtt::Assertion> =
                serde_json::from_slice(&input.bytes).map_err(|e| e.to_string())?;
            isometry_campaign::native::submit_contract_assertions(session, &assertions)
                .map_err(|e| e.to_string())?;
            Ok("asserted".into())
        },
        #[cfg(feature = "eponym")]
        Mode::Eponym => {
            let intent = serde_json::from_slice(&input.bytes).map_err(|e| e.to_string())?;
            let region = input.region.as_ref().ok_or("Eponym actuation needs an explicit native site region")?;
            if !cut.visible.sites.contains(&region.site) { return Err("Eponym region is outside the seat cut".into()); }
            let out = eponym_play::contract::Borrowed::new(session, region.site, region.min, region.max, region.depth)
                .map_err(|e| format!("{e:?}"))?.submit(Some(cut.access.participant.0), &intent)
                .map_err(|e| format!("{e:?}"))?;
            Ok(out.outcomes.join("\n"))
        },
        _ => Err("mode is absent from this build".into()),
    }
}

fn current(session: &Session, cut: &Projection, mode: Mode, input: &Input) -> Result<(), String> {
    if mode != input.mode || !mode.available() { return Err("intent belongs to another mode or build".into()); }
    if cut.access.seat == Seat::CreativeObserver { return Err("creative observation admits no play or edit".into()); }
    if session.sim.plays(cut.access.participant.0).map(isocosm_overlay::EntityHandle) != cut.access.subject {
        return Err("product access names a stale subject".into());
    }
    Ok(())
}

/// This mode's accepted native changes and versioned game bytes share a cut.
pub(crate) fn submit_game(session: &mut Session, cut: &Projection, mode: Mode,
    input: &Input, game: Vec<u8>) -> Result<(String, Vec<u8>), String> {
    current(session, cut, mode, input)?;
    match mode {
        #[cfg(feature = "mesocosm")]
        Mode::Mesocosm => {
            let decoded: mesocosm_runtime::spatial::Input =
                serde_json::from_slice(&input.bytes).map_err(|e| e.to_string())?;
            let saved: mesocosm_runtime::spatial::Game = if game.is_empty() {
                Default::default()
            } else { serde_json::from_slice(&game).map_err(|e| e.to_string())? };
            saved.version()?;
            let (accepted, saved) = mesocosm_runtime::borrowed::submit_game(
                session, cut.access, input.round, &decoded, saved).map_err(|e| format!("{e:?}"))?;
            Ok((accepted, serde_json::to_vec(&saved).map_err(|e| e.to_string())?))
        },
        #[cfg(feature = "vtt")]
        Mode::Vtt => {
            isometry_campaign::native::submit_game_access(session, game, &input.bytes, cut.access)
        },
        #[cfg(feature = "eponym")]
        Mode::Eponym => {
            let value: serde_json::Value = serde_json::from_slice(&input.bytes).map_err(|e| e.to_string())?;
            if value.get("kind").and_then(|kind| kind.as_str()) == Some(eponym_play::contract::mode::life::KIND) {
                return eponym_play::contract::mode::life::submit_bytes(session, game, &input.bytes, cut.access);
            }
            if value.get("kind").and_then(|kind| kind.as_str()) == Some(eponym_play::contract::mode::clock::KIND) {
                let input = serde_json::from_value(value).map_err(|e| e.to_string())?;
                return eponym_play::contract::mode::clock::submit(
                    session, game, &input, cut.access, &cut.visible);
            }
            let configured = eponym_play::contract::mode::ModeState::decode(&game)?
                .is_some_and(|state| state.actions.is_some());
            if configured {
                eponym_client::borrowed::actions::submit(
                    session, game, &input.bytes, cut.access, &cut.visible)
            } else {
                Ok((submit(session, cut, mode, input)?, game))
            }
        },
        _ => Ok((submit(session, cut, mode, input)?, game)),
    }
}

/// The ordinary host clock preserves configured Eponym physical state.
#[cfg(feature = "eponym")]
pub(crate) fn advance_eponym(session: &mut Session, bytes: Vec<u8>) -> Result<((),Vec<u8>),String> {
    let Some(mut mode) = eponym_play::contract::mode::ModeState::decode(&bytes)? else {
        session.advance(1)?; return Ok(((),bytes));
    };
    mode.validate_native(session)?;
    if mode.clock.is_none() { session.advance(1)?; return Ok(((),bytes)); }
    let mut actions = mode.actions.take().ok_or("Eponym clock requires its configured action profiles")?;
    eponym_play::contract::mode::clock::advance(session, &mut mode, &mut actions)?;
    mode.actions = Some(actions); mode.validate()?;
    Ok(((),serde_json::to_vec(&mode).map_err(|e| e.to_string())?))
}

/// The trusted owner's played loop retains geometry, questions and birth publications.
#[cfg(feature = "mesocosm")]
pub(crate) fn advance_mesocosm(session: &mut Session, bytes: Vec<u8>, access:isocosm_overlay::OrdinaryAccess)
    -> Result<((),Vec<u8>),String> {
    let mut game: mesocosm_runtime::spatial::Game = if bytes.is_empty() { Default::default() }
        else { serde_json::from_slice(&bytes).map_err(|e| e.to_string())? };
    game.validate(session)?;
    if access.seat!=Seat::Player {return Err("use the player's window to advance this loop".into());}
    let watching=game.continuation.as_ref().is_some_and(|saved|
        saved.phase==mesocosm_runtime::continuation::LoopPhase::Watching && saved.participant==access.participant.0);
    if !watching && (access.subject.is_none()
        || session.sim.plays(access.participant.0)!=access.subject.map(|subject|subject.0)) {
        return Err("played subject changed before this loop step".into());
    }
    mesocosm_runtime::continuation::advance_loop(session, &mut game, access.participant.0)
        .map_err(|error| format!("{error:?}"))?;
    Ok(((),serde_json::to_vec(&game).map_err(|e| e.to_string())?))
}

/// A window submits its product input through the shell's game bytes.
pub(crate) trait Submit {
    fn submit_product(&self, input: &Input) -> Result<String, String>;
}

impl Submit for crate::window::Window {
    fn submit_product(&self, input: &Input) -> Result<String, String> {
        use crate::mode_sections::Games;
        let cut = self.projection()?;
        let before = self.cut()?;
        let window = self.handle()?;
        let participant = self.participant();
        let mode = Mode::parse(&self.binding().mode)?;
        self.world().product_game(before, mode, |session, game| {
            session.sim.attention_window(participant.0, window.0)?;
            submit_game(session, &cut, mode, input, game)
        })
    }
}
