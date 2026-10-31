// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Profile {
    pub fn validate(&self, session: &Session, mode: &ModeState) -> Result<(), String> {
        self.shape(mode)?;
        let sim = &session.sim;
        for (subject, inputs) in &self.terrain {
            self.subject(mode, *subject)?;
            for (key, input) in inputs {
                let process = sim.genesis().rules.processes.get(key).ok_or("clock terrain process absent")?;
                if !process.effects.iter().any(|e| matches!(e, cosmograph::rules::Effect::Terrain {..})) {
                    return Err("clock terrain input names no actual native terrain effect".into());
                }
                target(sim, input.target)?;
            }
        }
        for (subject, inputs) in &self.steps {
            self.subject(mode, *subject)?;
            for (key, input) in inputs {
                if !sim.genesis().rules.step_profiles.contains_key(key) {
                    return Err("clock step input names no native step profile".into());
                }
                target(sim, input.target)?;
            }
        }
        for input in &self.arrivals {
            self.subject(mode, input.subject)?;
            if !sim.state().sites.get(&input.from).is_some_and(|s| s.routes.iter().any(|r| r.to == input.to))
                || !sim.state().sites.contains_key(&input.to) {
                return Err("clock arrival input names no actual native route/destination".into());
            }
        }
        for input in &self.births {
            if !sim.genesis().rules.birth_profiles.contains_key(&input.profile) {
                return Err("clock birth input names no native birth profile".into());
            }
            for slot in &input.slots {
                if mode.bindings.iter().any(|b| b.subject == slot.subject) {
                    return Err("authored newborn label already names an accepted subject".into());
                }
            }
        }
        for input in &self.births {
            match &input.parent {
                BirthParent::Entity(id) if sim.state().population.get(*id).is_none() =>
                    return Err("clock birth parent is absent".into()),
                BirthParent::Lineage(key) if !sim.state().lineages.contains_key(key) =>
                    return Err("clock birth parent lineage is absent".into()),
                _ => {},
            }
            // A process may be named by several profiles, but two selectors
            // applicable to the same actual parent are never given precedence.
            for other in &self.births {
                if std::ptr::eq(input, other) { continue; }
                let same = sim.genesis().rules.birth_profiles[&input.profile].process
                    == sim.genesis().rules.birth_profiles[&other.profile].process;
                let overlap = match (&input.parent, &other.parent) {
                    (BirthParent::Entity(a), BirthParent::Entity(b)) => a == b,
                    (BirthParent::Lineage(a), BirthParent::Lineage(b)) => a == b,
                    (BirthParent::Entity(id), BirthParent::Lineage(key))
                    | (BirthParent::Lineage(key), BirthParent::Entity(id)) =>
                        sim.state().population.get(*id).is_some_and(|e| &e.lineage == key),
                };
                if same && overlap { return Err("clock birth selectors overlap".into()); }
            }
        }
        Ok(())
    }

    fn shape(&self, mode: &ModeState) -> Result<(), String> {
        if self.version != 1 { return Err("unsupported Eponym clock profile".into()); }
        for subject in self.terrain.keys().chain(self.steps.keys()) { self.subject(mode, *subject)?; }
        for inputs in self.steps.values() { for input in inputs.values() {
            if input.path_from_root.first() != Some(&[0; 3]) || input.path_from_root.len() < 2 {
                return Err("clock step needs a complete authored path beginning at its root".into());
            }
        } }
        let mut arrivals = BTreeSet::new();
        for input in &self.arrivals {
            self.subject(mode, input.subject)?;
            input.pose.validate().map_err(debug)?;
            if !arrivals.insert((input.subject, input.from, input.to)) {
                return Err("clock arrival inputs are ambiguous".into());
            }
        }
        let mut selectors = BTreeSet::new();
        let mut subjects = BTreeSet::new();
        for input in &self.births {
            let selector = match &input.parent {
                BirthParent::Entity(id) => format!("entity:{id}"),
                BirthParent::Lineage(key) => format!("lineage:{key}"),
            };
            if !selectors.insert((input.profile.clone(), selector)) {
                return Err("duplicate clock birth selector".into());
            }
            for slot in &input.slots {
                if !subjects.insert(slot.subject) { return Err("authored newborn subject labels are ambiguous".into()); }
                // Validate supplied motion policy using the same existing game rules.
                if let Some(policy) = &slot.motion { policy.rules.validate().map_err(debug)?; }
                if let Some(policy) = &slot.combat {
                    policy.rules.validate().map_err(debug)?; policy.timed.validate().map_err(debug)?;
                    policy.network.validate().map_err(debug)?;
                }
            }
        }
        Ok(())
    }

    fn subject(&self, mode: &ModeState, subject: SubjectId) -> Result<(), String> {
        if mode.bindings.iter().any(|b| b.subject == subject)
            || self.births.iter().any(|b| b.slots.iter().any(|s| s.subject == subject)) {
            return Ok(());
        }
        Err("clock inputs need an admitted subject or an explicitly authored newborn label".into())
    }
}

impl State {
    pub fn validate(&self, mode: &ModeState) -> Result<(), String> {
        self.profile.shape(mode)?;
        if let Some(sight) = &self.sight { sight.validate(mode, self)?; }
        for (subject, physical) in &self.physical {
            if *subject != physical.subject { return Err("clock physical subject key differs".into()); }
            binding(mode, *subject)?;
        }
        let mut entries = BTreeSet::new();
        for record in &self.records {
            if !entries.insert(&record.entry) { return Err("duplicate Eponym clock entry".into()); }
            for change in &record.changes {
                if change.after.subject != change.physical.subject
                    || change.before.as_ref().is_some_and(|b| b.subject != change.after.subject || b.entity != change.after.entity) {
                    return Err("clock publication changes an unrelated subject identity".into());
                }
                change.after.pose.validate().map_err(debug)?;
            }
        }
        Ok(())
    }

    /// Native replay is the host's. Product records point to its real commands;
    /// this validator never constructs another Session or replays a world.
    pub fn validate_native(&self, session: &Session) -> Result<(), String> {
        let mut last = None;
        for record in &self.records {
            let index = session.entries.iter().position(|e| e.id == record.entry).ok_or("clock native entry absent")?;
            if last.is_some_and(|before| index <= before) { return Err("clock native entry order changed".into()); }
            let entry = &session.entries[index];
            let mut before = last;
            for key in &record.before_entries {
                let at = session.entries.iter().position(|e| &e.id == key).ok_or("clock source refresh entry absent")?;
                if at >= index || before.is_some_and(|previous| at <= previous)
                    || !matches!(session.entries[at].command, Command::Patch { .. }) {
                    return Err("clock source refresh does not name an ordered real native patch".into());
                }
                before = Some(at);
            }
            let command = Command::AdvanceBirthSpatial { ticks: 1, terrain: record.terrain.clone(),
                steps: record.steps.clone(), arrivals: record.arrivals.clone(), births: record.births.clone() };
            let accepted: BTreeSet<_> = record.order.iter().filter_map(|p|
                if let native::Publication::Step(action) = p { Some(action.clone()) } else { None }).collect();
            let outcome = format!("birth spatial advanced 1; answers {}", cosmograph::digest(&(
                &record.arrival_reports, accepted, &record.birth_reports, &record.order)));
            if entry.command != command || entry.outcome != record.outcome || entry.outcome != outcome {
                return Err("clock saved answers differ from the accepted native entry".into());
            }
            let mut end = index;
            for key in &record.refresh_entries {
                let at = session.entries.iter().position(|e| &e.id == key).ok_or("clock refresh entry absent")?;
                if at <= end || !matches!(session.entries[at].command, Command::Patch { .. }) {
                    return Err("clock refresh does not name an ordered real native patch".into());
                }
                end = at;
            }
            last = Some(end);
        }
        if self.records.len() > session.sim.genesis().rules.limits.history {
            return Err("Eponym clock history exceeds the native limit".into());
        }
        Ok(())
    }
}

fn target(sim: &Simulation, id: Option<Id>) -> Result<(), String> {
    if id.is_some_and(|id| sim.state().population.get(id).is_none()) {
        return Err("clock authored target is absent".into());
    }
    Ok(())
}
fn debug(error: impl std::fmt::Debug) -> String { format!("{error:?}") }
