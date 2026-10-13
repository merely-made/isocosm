// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use super::super::{GameBindings, AdmissionBatch, items, physical_map, submit_game_access};
use crate::{StoryletProposal, StoryletResolution};

pub fn public_outputs(source: &StoryletProposal, resolved: StoryletResolution) -> ComposedStoryletPublic {
    ComposedStoryletPublic { key: source.key.clone(), source_digest: isocosm::digest(source),
        cast: resolved.cast, effects: resolved.effects }
}

pub(super) fn assertions(public: &ComposedStoryletPublic, occurrence: &str) -> Vec<Assertion> {
    public.effects.iter().enumerate().filter_map(|(index, effect)| match effect {
        StoryletEffect::History { event } => Some(Assertion::History(event.clone())),
        StoryletEffect::Fact { fact } => Some(Assertion::Fact(Fact { about: fact.id.clone(),
            key: format!("{occurrence}:fact:{index}"), kind: fact.kind.clone(), text: fact.text.clone(),
            tags: fact.tags.iter().cloned().collect() })),
        _ => None,
    }).collect()
}

pub(super) fn shape(input: &ComposedStoryletInput, public: &ComposedStoryletPublic) -> Result<(), String> {
    if input.version != 1 || input.storylet.key != public.key || public.source_digest.is_empty() {
        return Err("composed storylet version or accepted public source differs".into());
    }
    let mut effects = BTreeSet::new();
    let mut items = BTreeSet::new();
    for step in &input.steps {
        let effect = match step {
            StoryletStep::Map { effect, .. } | StoryletStep::Item { effect, .. } => *effect,
            StoryletStep::Placement { placements, .. } => {
                if placements.is_empty() { return Err("storylet placement needs complete supplied existing-character poses".into()); }
                continue;
            },
        };
        if !effects.insert(effect) { return Err("storylet physical effect has repeated supplements".into()); }
        match (step, public.effects.get(effect as usize)) {
            (StoryletStep::Map { .. }, Some(StoryletEffect::LocalMap { .. })) => {},
            (StoryletStep::Item { item, inputs, .. }, Some(StoryletEffect::Item { .. }))
                if !item.0.trim().is_empty() && !inputs.is_empty() && items.insert(item) => {},
            _ => return Err("storylet supplement does not name its actual physical effect".into()),
        }
    }
    for (index, effect) in public.effects.iter().enumerate() {
        if matches!(effect, StoryletEffect::LocalMap { .. } | StoryletEffect::Item { .. })
            && !effects.contains(&u32::try_from(index).map_err(|_| "storylet effect ordinal overflows")?) {
            return Err("storylet physical effect lacks its complete supplied supplement".into());
        }
    }
    Ok(())
}

pub(super) fn map_site(session: &Session, public: &ComposedStoryletPublic, effect: u32,
    input: &physical_map::PhysicalMapInput) -> Result<(), String> {
    let Some(StoryletEffect::LocalMap { map }) = public.effects.get(effect as usize) else {
        return Err("storylet map supplement has no actual map effect".into());
    };
    let sites: Vec<_> = session.sim.state().sites.iter().filter_map(|(id, site)| {
        site.authored.as_ref().filter(|place| place.map.as_ref() == Some(&map.id)).map(|_| *id)
    }).collect();
    if sites.len() != 1 || sites[0] != input.geometry.site || sites[0] != input.metadata.site.0 {
        return Err("storylet map needs its unique accepted native place-to-map association".into());
    }
    Ok(())
}

/// The trusted host attests public outputs after checking its own private conditions.
/// This door validates operations and control; it never resolves private eligibility.
pub fn readmit_composed(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &ComposedStoryletInput, public: &ComposedStoryletPublic) -> Result<(String, Vec<u8>), String> {
    current_access(session, access)?;
    if access.seat != Seat::Dm || input.storylet.participant != access.participant.0
        || input.storylet.tick != session.sim.state().tick
        || input.storylet.expected_native != isocosm::digest(&session.save())
        || input.expected_game != isocosm::digest(&game) {
        return Err("composed storylet names another DM or a changed native/product cut".into());
    }
    shape(input, public)?;
    let mut candidate = session.clone();
    let start = candidate.entries.len();
    let occurrence = format!("storylet:{}:{start}", public.key);
    let mut game = game;
    let mut links = Vec::new();
    for (index, step) in input.steps.iter().enumerate() {
        let first = candidate.entries.len();
        let mut entity = None;
        match step {
            StoryletStep::Map { effect, input } => {
                map_site(&candidate, public, *effect, input)?;
                game = physical_map::submit_physical_map(&mut candidate, game, access, input)?.1;
            },
            StoryletStep::Item { item, inputs, .. } => {
                for input in inputs { game = items::submit_item(&mut candidate, game, access, input)?.1; }
                let bindings = GameBindings::decode(&game)?;
                let bound = bindings.item_records().last().and_then(|record| record.binding.as_ref())
                    .ok_or("storylet item work lacks its exact accepted resident entity or completed output")?;
                if &bound.item != item { return Err("storylet item differs from its explicit accepted binding".into()); }
                entity = Some(bound.entity);
            },
            StoryletStep::Placement { expected_native, expected_game, placements } => {
                if *expected_native != physical_map::physical_source(&candidate) || *expected_game != isocosm::digest(&game) {
                    return Err("storylet supplied pose names a changed native/product cut".into());
                }
                let batch = AdmissionBatch { version: 2, assertions: Vec::new(), characters: Vec::new(),
                    placements: placements.clone(), controls: Vec::new(), project_carry: None };
                let bytes = serde_json::to_vec(&batch).map_err(|error| error.to_string())?;
                game = submit_game_access(&mut candidate, game, &bytes, access)?.1;
            },
        }
        links.push(StoryletLink { step: index, entity,
            entries: candidate.entries[first..].iter().map(|entry| entry.id.clone()).collect() });
    }
    let accepted = candidate.command(Command::Assert(Assertion::Storylet(Applied {
        key: occurrence, source: public.key.clone(), asserts: assertions(public, &format!("storylet:{}:{start}", public.key)),
    })))?;
    let marker = candidate.entries.last().ok_or("storylet marker has no native entry")?.id.clone();
    let mut bindings = GameBindings::decode(&game)?;
    bindings.record_composed_storylet(ComposedStoryletRecord { input: input.clone(), participant: access.participant.0,
        public: public.clone(), entries: candidate.entries[start..].iter().map(|entry| entry.id.clone()).collect(), marker, links });
    bindings.validate(&candidate)?;
    let bytes = bindings.encode()?;
    *session = candidate;
    Ok((accepted, bytes))
}

pub fn submit_composed_storylet(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &ComposedStoryletInput) -> Result<(String, Vec<u8>), String> {
    current_access(session, access)?;
    if access.seat != Seat::Dm { return Err("storylet composition requires the actual DM".into()); }
    let content = pack::active_content(session, &game, access)?;
    if input.storylet.expected_content != isocosm::digest(&content) { return Err("storylet content changed since preview".into()); }
    let source = content.storylets.get(&input.storylet.key).ok_or("storylet offer is absent")?;
    let public = public_outputs(source, resolve(session, source)?);
    readmit_composed(session, game, access, input, &public)
}

pub fn prepare_composed_storylet(session: &Session, game: &[u8], access: OrdinaryAccess,
    input: &ComposedStoryletInput) -> ComposedStoryletPreview {
    let mut candidate = session.clone();
    match submit_composed_storylet(&mut candidate, game.to_vec(), access, input) {
        Ok((_, game)) => ComposedStoryletPreview { complete: true, refusal: None,
            public: GameBindings::decode(&game).ok().and_then(|game| game.composed_storylets().last().map(|record| record.public.clone())) },
        Err(reason) => ComposedStoryletPreview { complete: false, refusal: Some(reason), public: None },
    }
}
