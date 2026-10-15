// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;
use isocosm_overlay::{ParticipantHandle, Seat, WindowHandle};
use crate::{products::{Input, Mode, SiteRegion}, save::WorldSave,
    ui::model::{App, Preferences}, world::World};

pub(crate) fn open(args: &[std::ffi::OsString]) -> Result<App, String> {
    let (mut source, mut participant, mut seat, mut mode) = (None, None, None, None);
    let (mut save_path, mut restore, mut input_path, mut region_path, mut preferences_path) =
        (None, None, None, None, None);
    let mut second = None;
    let mut mesocosm_spatial = None;
    let mut mesocosm_spread = None;
    let mut eponym_spatial = None;
    let mut eponym_actions = None;
    let mut eponym_clock = None;
    let mut eponym_source = None;
    let mut eponym_timed_source = None;
    let mut round = 1;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].to_str().ok_or("invalid argument")?;
        let value = args.get(index + 1).ok_or("each option requires a value")?;
        let word = || value.to_str().ok_or("invalid option value");
        match flag {
            "--open" => source = Some(PathBuf::from(value)),
            "--open-eponym" => eponym_source = Some(PathBuf::from(value)),
            "--open-eponym-timed" => eponym_timed_source = Some(PathBuf::from(value)),
            "--participant" => participant = Some(ParticipantHandle(word()?.parse::<u64>().map_err(|e| e.to_string())?)),
            "--seat" => seat = Some(match word()? { "player" => Seat::Player,
                "creative" => Seat::CreativeObserver, "dm" => Seat::Dm,
                _ => return Err("seat must be player, creative or dm".into()) }),
            "--mode" => mode = Some(Mode::parse(word()?)?),
            "--second-mode" => second = Some(Mode::parse(word()?)?),
            "--restore-window" => restore = Some(WindowHandle(word()?.parse::<u64>().map_err(|e| e.to_string())?)),
            "--save-as" => save_path = Some(PathBuf::from(value)),
            "--intent" => input_path = Some(PathBuf::from(value)),
            "--site-region" => region_path = Some(PathBuf::from(value)),
            "--preferences" => preferences_path = Some(PathBuf::from(value)),
            "--mesocosm-spatial" => mesocosm_spatial = Some(PathBuf::from(value)),
            "--mesocosm-spread" => mesocosm_spread = Some(PathBuf::from(value)),
            "--eponym-spatial" => eponym_spatial = Some(PathBuf::from(value)),
            "--eponym-actions" => eponym_actions = Some(PathBuf::from(value)),
            "--eponym-clock" => eponym_clock = Some(PathBuf::from(value)),
            "--round" => round = word()?.parse::<u64>().map_err(|e| e.to_string())?,
            _ => return Err("unknown mode-host option".into()),
        }
        index += 2;
    }
    if round == 0 { return Err("round must contain at least one native tick".into()); }
    let mode = mode.ok_or("choose --mode")?;
    if !mode.available() || second.is_some_and(|m| !m.available()) {
        return Err("requested mode is absent from this build".into());
    }
    let participant = participant.ok_or("name --participant explicitly")?;
    let seat = seat.ok_or("choose --seat explicitly")?;
    let world = match (source, eponym_source, eponym_timed_source) {
        (Some(source), None, None) => {
            let bytes = std::fs::read(source).map_err(|e| e.to_string())?;
            World::resume(WorldSave::decode(&bytes)?, isocosm::Execution::Grouped)?
        },
        (None, Some(source), None) => open_eponym(&source, eponym_spatial.take()
            .ok_or("--open-eponym requires its explicit --eponym-spatial handoff input")?)?,
        (None, None, Some(source)) => open_eponym_timed(&source, eponym_spatial.take()
            .ok_or("timed handoff requires --eponym-spatial")?, eponym_actions.take()
            .ok_or("timed handoff requires --eponym-actions")?)?,
        _ => return Err("choose one --open, --open-eponym or --open-eponym-timed source".into()),
    };
    if let Some(path) = mesocosm_spread {
        crate::desktop::mesocosm_spread(&world, &path)?;
    }
    if let Some(path) = mesocosm_spatial {
        crate::desktop::mesocosm_spatial(&world, &path)?;
    }
    if let Some(path) = eponym_spatial {
        crate::desktop::eponym_spatial(&world, &path)?;
    }
    if let Some(path) = eponym_actions {
        crate::desktop::eponym_actions(&world, &path)?;
    }
    if let Some(path) = eponym_clock {
        crate::desktop::eponym_clock(&world, &path)?;
    }
    let preferences: Option<Preferences> = read_json(preferences_path)?;
    let region: Option<SiteRegion> = read_json(region_path)?;
    let input = input_path.map(|path| std::fs::read(path).map(|bytes| Input {
        mode, bytes, round, region })).transpose().map_err(|e| e.to_string())?;
    let mut app = App::new(world, save_path);
    let first = app.open_with_saved_preferences(participant, seat, mode, preferences.clone(), restore)?;
    app.windows.get_mut(&first).ok_or("opened window is missing")?.input = input;
    if let Some(second) = second { app.open_with_saved_preferences(participant, seat, second, preferences, None)?; }
    Ok(app)
}

fn open_eponym(source: &std::path::Path, spatial: PathBuf) -> Result<World, String> {
    #[cfg(feature = "eponym")]
    {
        let bytes = std::fs::read(source).map_err(|e| e.to_string())?;
        let input = std::fs::read(spatial).map_err(|e| e.to_string())?;
        let input = serde_json::from_slice(&input).map_err(|e| e.to_string())?;
        // Restore with the product's existing solver, then consume that owner.
        let standalone = eponym_play::Session::restore_solving(&bytes,
            eponym_client::MOTION_SOLVER).map_err(|e| format!("Eponym source: {e:?}"))?;
        let handoff = standalone.into_native_mode(&input).map_err(|(_,why)| why)?;
        crate::mode_sections::take_eponym(handoff).map_err(|(_,why)| why)
    }
    #[cfg(not(feature = "eponym"))]
    { let _ = (source, spatial); Err("Eponym handoff is absent from this build".into()) }
}

fn read_json<T: serde::de::DeserializeOwned>(path: Option<PathBuf>) -> Result<Option<T>, String> {
    path.map(|path| {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }).transpose()
}

fn open_eponym_timed(source: &std::path::Path, spatial: PathBuf, actions: PathBuf) -> Result<World, String> {
    #[cfg(feature = "eponym")]
    {
        let bytes = std::fs::read(source).map_err(|e| e.to_string())?;
        let spatial = std::fs::read(spatial).map_err(|e| e.to_string())?;
        let actions = std::fs::read(actions).map_err(|e| e.to_string())?;
        let spatial = serde_json::from_slice(&spatial).map_err(|e| e.to_string())?;
        let actions = serde_json::from_slice(&actions).map_err(|e| e.to_string())?;
        let standalone = eponym_play::timed_action::TimedActionSession::restore_solving(&bytes,
            eponym_client::MOTION_SOLVER).map_err(|e| format!("Eponym timed source: {e:?}"))?;
        let handoff = eponym_client::borrowed::handoff::take_timed(standalone, &spatial, &actions)
            .map_err(|(_,why)| why)?;
        crate::mode_sections::take_eponym(handoff).map_err(|(_,why)| why)
    }
    #[cfg(not(feature = "eponym"))]
    { let _ = (source, spatial, actions); Err("Eponym timed handoff is absent from this build".into()) }
}
