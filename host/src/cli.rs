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
    let mut round = 1;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].to_str().ok_or("invalid argument")?;
        let value = args.get(index + 1).ok_or("each option requires a value")?;
        let word = || value.to_str().ok_or("invalid option value");
        match flag {
            "--open" => source = Some(PathBuf::from(value)),
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
    let source = source.ok_or("name --open world-save.json")?;
    let bytes = std::fs::read(source).map_err(|e| e.to_string())?;
    let world = World::resume(WorldSave::decode(&bytes)?, isocosm::Execution::Grouped)?;
    if let Some(path) = mesocosm_spatial {
        crate::desktop::mesocosm_spatial(&world, &path)?;
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

fn read_json<T: serde::de::DeserializeOwned>(path: Option<PathBuf>) -> Result<Option<T>, String> {
    path.map(|path| {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }).transpose()
}
