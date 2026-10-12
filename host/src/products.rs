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
            Self::Vtt => &["Read table", "Apply DM assertion batch"],
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
            if cut.access.seat != Seat::Dm { return Err("native table assertions require the DM seat".into()); }
            isometry_campaign::native::submit_game(session, game, &input.bytes)
        },
        _ => Ok((submit(session, cut, mode, input)?, game)),
    }
}
