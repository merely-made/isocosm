// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A part's cells placed exactly by command (ruling 787): what an authored
//! expression script proposed for a committed review offer, through
//! [`super::propose`]. All or nothing: a refused placement changes nothing.

use super::{CellId, propose};
use crate::{Result, schema::*, simulation::Simulation};

impl Simulation {
    /// Places `part`'s cells on `entity` as `tracts` say, each function one
    /// the world's catalogue holds. Returns how many cells changed.
    pub fn express(&mut self, entity: Id, part: PartId, tracts: &[(Key, Vec<CellId>)]) -> Result<u32> {
        if let Some((f, _)) = tracts.iter().find(|(f, _)| !self.genesis.rules.functions.contains_key(f)) {
            return Err(format!("{f} is not in the world's catalogue"));
        }
        let e = self.state.population.get(entity).ok_or("unknown entity")?;
        if !e.alive || !e.lives(part) {
            return Err("no living part to express on".into());
        }
        let half = e.extent(part);
        let mut p = e.parts.get(&part).cloned().ok_or("an unlaid part")?;
        let changed = propose(&mut p, half, tracts)?;
        let e = self.state.population.lift(entity)?;
        e.parts.insert(part, p);
        e.body_revision = e.body_revision.saturating_add(1);
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{dims, path};
    use crate::{Execution, Session, history::Command, schema::*};

    fn session() -> (Session, Id, PartId, [i32; 3]) {
        let genesis = crate::probe::BodyFounding::default().generate().unwrap().genesis;
        let s = Session::new(genesis, Execution::Individuals).unwrap();
        let pop = &s.sim.state().population;
        let (id, part, half) = pop
            .groups
            .iter()
            .find_map(|(id, g)| {
                let e = &g.entity;
                let (part, _) = e.living().next()?;
                e.body.as_ref()?;
                Some((*id, part, e.extent(part)))
            })
            .expect("a bodied founding has a body");
        (s, id, part, half)
    }

    #[test]
    fn an_express_places_the_cells_and_replays() {
        let (mut s, id, part, half) = session();
        let first = path(dims(half))[0];
        let tracts = vec![("function:store".to_string(), vec![first])];
        let out = s
            .command(Command::Express { entity: id, part, tracts })
            .unwrap();
        assert!(out.starts_with("expressed"), "{out}");
        let e = s.sim.state().population.get(id).unwrap();
        let p = &e.parts[&part];
        assert_eq!(p.cells.get("function:store"), Some(&1));
        assert!(p.tracts.iter().any(|t| t.cells == vec![first]));
        let json = serde_json::to_vec(&s.save()).unwrap();
        let replayed = Session::load_json(&json, Execution::Individuals).unwrap();
        assert_eq!(replayed.sim.state_hash(), s.sim.state_hash());
    }

    #[test]
    fn a_refused_express_changes_nothing() {
        let (mut s, id, part, half) = session();
        let before = s.sim.state_hash();
        let past = path(dims(half)).len() as u16;
        let outside = vec![("function:store".to_string(), vec![CellId(past)])];
        let unknown = vec![("function:sing".to_string(), vec![path(dims(half))[0]])];
        for tracts in [outside, unknown] {
            assert!(s.command(Command::Express { entity: id, part, tracts }).is_err());
        }
        assert_eq!(s.sim.state_hash(), before);
    }

    use super::CellId;
}
