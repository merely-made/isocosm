// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Product pose transitions are authenticated by accepted native spatial entries.
//! Native replay remains the sole world authority; this checks its saved game bytes.

use super::*;
use isocosm::Command;

fn physical(a: &Placement, b: &Placement) -> bool {
    a.actor == b.actor && a.site == b.site && a.pose == b.pose && a.eye == b.eye
        && a.window == b.window && a.operations == b.operations && a.places == b.places
        && a.resident_since == b.resident_since
}

impl Game {
    pub fn validate(&self, session: &Session) -> Result<(), String> {
        self.validate_inner(session, false)
    }

    pub fn validate_replay(&self, session: &Session) -> Result<(), String> {
        self.validate_inner(session, true)
    }

    fn validate_inner(&self, session: &Session, replay: bool) -> Result<(), String> {
        self.version()?;
        let Some(driver) = &self.driver else { return Ok(()); };
        let count = usize::try_from(driver.native_entries).map_err(|_| "spatial source entry count overflow")?;
        if count > session.entries.len() || driver.native_tick > session.sim.state().tick {
            return Err("spatial driver source is outside the native history".into());
        }
        if profile::native_prefix(session, count, driver.native_tick) != driver.native_prefix {
            return Err("spatial driver references another native prefix".into());
        }
        if replay {
            // Only save/resume reconstructs the exact source through authentic
            // Session replay. Ordinary borrowed reads never replay a runtime.
            let mut prefix = session.clone();
            prefix.entries.truncate(count);
            let source = prefix.fork_at(driver.native_tick, session.branch.clone())?;
            if profile::native_source(&source) != driver.native_source {
                return Err("spatial driver source differs from native history".into());
            }
            driver.profile.validate(&source.sim)?;
            for at in driver.initial.values() { at.check(&source.sim)?; }
        }
        let mut folded = Game { version: VERSION, members: driver.initial.clone(), driver: Some(driver.clone()) };
        let mut records = driver.records.iter();
        for entry in session.entries.iter().skip(count) {
            let Command::AdvanceSpatial { ticks, steps, arrivals, .. } = &entry.command else { continue; };
            let record = records.next().ok_or("native spatial entry has no saved game transition")?;
            let outcome = format!("spatial advanced {ticks}; answers {}",
                isocosm::digest(&(&record.arrivals, &record.steps)));
            if record.entry != entry.id || outcome != entry.outcome {
                return Err("game spatial admission differs from its authenticated native outcome".into());
            }
            if record.steps.iter().any(|action| !steps.iter().any(|step| &step.request.action == action)) {
                return Err("game names no native step answer".into());
            }
            let mut changes = folded.apply_arrivals(arrivals, &record.arrivals)?;
            changes.extend(folded.apply_steps(steps, &record.steps)?);
            if changes.len() != record.changes.len() || changes.iter().zip(&record.changes).any(|(actual, saved)| {
                actual.actor != saved.actor || actual.admission != saved.admission
                    || !physical(&actual.after, &saved.after)
                    || match (&actual.before, &saved.before) {
                        (None, None) => false,
                        (Some(a), Some(b)) => !physical(a, b),
                        _ => true,
                    }
            }) { return Err("saved physical transition differs from the admitted native path".into()); }
        }
        if records.next().is_some() || folded.members.len() != self.members.len()
            || folded.members.iter().any(|(id, held)| self.members.get(id).is_none_or(|saved| !physical(held, saved)))
        { return Err("saved final poses do not follow their native-admitted transitions".into()); }
        Ok(())
    }
}
