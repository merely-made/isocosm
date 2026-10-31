// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use super::super::{items::PhysicalItemRecord, physical_map::PhysicalMapRecord};

/// Existing child records prove native operations; product hashes remain live
/// submission conditions. Private eligibility is never reconstructed here.
pub fn validate_composed_records(session: &Session, records: &[ComposedStoryletRecord],
    maps: &[PhysicalMapRecord], items: &[PhysicalItemRecord]) -> Result<(), String> {
    let mut used = BTreeSet::new();
    for record in records {
        composition::shape(&record.input, &record.public)?;
        let first = record.entries.first().ok_or("composed storylet has no native entry")?;
        let before = session.before_entry(first)?;
        if record.input.storylet.expected_native != cosmograph::digest(&before.save())
            || record.input.storylet.tick != before.sim.state().tick
            || record.input.storylet.participant != record.participant
            || !before.sim.state().population.get(record.participant).is_some_and(cosmograph::directing::is_participant)
            || record.entries.last() != Some(&record.marker) || record.links.len() != record.input.steps.len() {
            return Err("composed storylet differs from its actual native cut or participant".into());
        }
        for character in record.public.cast.values() {
            if before.sim.authored_entity(character).is_none() {
                return Err("storylet public cast names no actual native authored character".into());
            }
        }
        let start = before.entries.len();
        for (offset, key) in record.entries.iter().enumerate() {
            if !used.insert(key) || session.entries.get(start + offset).is_none_or(|entry| &entry.id != key) {
                return Err("composed storylet reuses or changes its contiguous native entry span".into());
            }
        }
        let occurrence = format!("storylet:{}:{start}", record.public.key);
        let expected = Command::Assert(Assertion::Storylet(Applied { key: occurrence.clone(),
            source: record.public.key.clone(), asserts: composition::assertions(&record.public, &occurrence) }));
        if session.entries.get(start + record.entries.len() - 1).is_none_or(|entry| entry.command != expected) {
            return Err("storylet public facts/history differ from the admitted marker".into());
        }
        let mut consumed = Vec::new();
        for (index, (step, link)) in record.input.steps.iter().zip(&record.links).enumerate() {
            if link.step != index || link.entries.is_empty() { return Err("storylet step has no exact native link".into()); }
            consumed.extend(link.entries.clone());
            match step {
                StoryletStep::Map { effect, input } => {
                    let source = session.before_entry(&link.entries[0])?;
                    composition::map_site(&source, &record.public, *effect, input)?;
                    let matched: Vec<_> = maps.iter().filter(|map| map.input == *input
                        && map.participant == record.participant).collect();
                    if matched.len() != 1 || link.entity.is_some() { return Err("storylet map link is missing or ambiguous".into()); }
                    let mut entries = matched[0].edit_entries.clone(); entries.push(matched[0].metadata_entry.clone());
                    if entries != link.entries { return Err("storylet map link changes its actual physical edit entries".into()); }
                },
                StoryletStep::Item { item, inputs, .. } => {
                    let mut entries = Vec::new();
                    let mut bound = None;
                    for input in inputs {
                        let matched: Vec<_> = items.iter().filter(|item| item.input == *input
                            && item.participant == record.participant).collect();
                        if matched.len() != 1 { return Err("storylet item link is missing or ambiguous".into()); }
                        entries.extend(matched[0].entries.clone()); bound = matched[0].binding.as_ref();
                    }
                    if entries != link.entries || bound.is_none_or(|bound| &bound.item != item || Some(bound.entity) != link.entity) {
                        return Err("storylet item link differs from its exact resident binding or native output".into());
                    }
                },
                StoryletStep::Placement { expected_native, placements, .. } => {
                    let first = link.entries.first().ok_or("storylet pose has no native entry")?;
                    let source = session.before_entry(first)?;
                    if *expected_native != super::super::physical_map::physical_source(&source)
                        || link.entity.is_some() || placements.len() != link.entries.len() {
                        return Err("storylet supplied poses differ from their actual native cut".into());
                    }
                    for (placement, key) in placements.iter().zip(&link.entries) {
                        let source = session.before_entry(key)?;
                        let (entity, _) = source.sim.authored_entity(&placement.character).ok_or("storylet pose character is absent")?;
                        placement.check(&source, entity, false)?;
                        let entry = session.entries.iter().find(|entry| &entry.id == key).ok_or("storylet pose entry is absent")?;
                        if entry.command != (Command::Patch { entity, patch: Some(placement.patch) }) || entry.outcome != "placed" {
                            return Err("storylet supplied pose differs from native placement admission".into());
                        }
                    }
                },
                StoryletStep::SpatialPlacement { expected_native,placements,.. }=>{
                    let first=link.entries.first().ok_or("storylet spatial pose has no native entry")?;
                    let source=session.before_entry(first)?;
                    if *expected_native!=super::super::physical_map::physical_source(&source)
                        || link.entity.is_some() || placements.len()!=link.entries.len() {
                        return Err("storylet spatial poses differ from their actual native cut".into());
                    }
                    for (spatial,key) in placements.iter().zip(&link.entries) {
                        let source=session.before_entry(key)?;
                        let placement=&spatial.placement;
                        let (entity,_)=source.sim.authored_entity(&placement.character)
                            .ok_or("storylet spatial character is absent")?;
                        placement.check(&source,entity,false)?;
                        spatial.spatial.check(&source,placement)?;
                        let entry=session.entries.iter().find(|entry|&entry.id==key)
                            .ok_or("storylet spatial pose entry is absent")?;
                        if entry.command!=(Command::Patch {entity,patch:Some(placement.patch)}) || entry.outcome!="placed" {
                            return Err("storylet spatial pose differs from its exact native Patch admission".into());
                        }
                    }
                },
            }
        }
        consumed.push(record.marker.clone());
        if consumed != record.entries { return Err("storylet step links omit or rearrange native operations".into()); }
    }
    Ok(())
}
