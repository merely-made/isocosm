// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// Historical source checks never renew a pose or supply current control.
pub fn validate_records(session:&Session,records:&[PhysicalMapRecord]) -> Result<(),String> {
    let mut keys=BTreeSet::new();
    let mut used=BTreeSet::new();
    for record in records {
        let input=&record.input;
        geometry::check(input)?;
        if input.version != 1 || !keys.insert(&input.metadata.key)
            || record.edit_entries.len()!=input.steps.len() {
            return Err("physical map record version, key or exact edit count differs".into());
        }
        let first=record.edit_entries.first().ok_or("physical map record has no native edit")?;
        let before=session.before_entry(first)?;
        let body=before.sim.state().population.get(input.actor).ok_or("recorded physical edit body is absent")?;
        if input.expected_source!=physical_source(&before) || input.expected_terrain!=before.sim.terrain_revision()
            || before.sim.plays(record.participant)!=Some(input.actor)
            || !before.sim.state().population.get(record.participant).is_some_and(isocosm::directing::is_participant)
            || !body.alive || body.body.is_none() || body.body_revision!=input.body_revision
            || body.resident_site()!=Some(input.geometry.site) || input.metadata.site.0!=input.geometry.site {
            return Err("physical map record differs from its actual native source, body or control".into());
        }
        let mut last_order=None;
        for (index,key) in record.edit_entries.iter().enumerate() {
            if !used.insert(key) { return Err("physical map records reuse an accepted native edit".into()); }
            let entry=session.entries.iter().find(|entry| &entry.id==key).ok_or("physical map edit entry is absent")?;
            if last_order.is_some_and(|previous:u64| previous.checked_add(1)!=Some(entry.order)) {
                return Err("physical map edit entries are not one contiguous admitted batch".into());
            }
            let before=session.before_entry(key)?;
            let quote=quoted(&before,input,index)?;
            let expected=Command::Owned { command:Box::new(quote.command),selections:input.steps[index].selections.clone() };
            if entry.command!=expected || !quote.supplied || quote.refusal.is_some() {
                return Err("physical map record differs from its native geometry, quantities or owned selections".into());
            }
            let outcome:isocosm::projects::OwnedCommandResult=serde_json::from_str(&entry.outcome)
                .map_err(|error|error.to_string())?;
            if outcome.debits!=quote.debits.requests { return Err("physical map debit receipt differs from actual native admission".into()); }
            last_order=Some(entry.order);
        }
        if !used.insert(&record.metadata_entry) { return Err("physical map records repeat their metadata entry".into()); }
        let metadata=session.entries.iter().find(|entry| entry.id==record.metadata_entry)
            .ok_or("physical map metadata entry is absent")?;
        let before=session.before_entry(&record.metadata_entry)?;
        let window=&input.geometry.window;
        before.sim.volume(input.geometry.site,window.min,window.max,window.depth)?;
        let expected=crate::contract::lower(&Assertion::Edit(input.metadata.clone()),&before.sim)
            .map_err(|error|format!("physical metadata: {error:?}"))?;
        if metadata.command!=Command::Assert(expected)
            || last_order.and_then(|order|order.checked_add(1))!=Some(metadata.order)
            || record.terrain_after!=before.sim.terrain_revision() {
            return Err("physical map metadata or final terrain revision differs from native history".into());
        }
    }
    Ok(())
}
