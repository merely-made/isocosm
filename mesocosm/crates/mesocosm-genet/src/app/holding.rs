// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The host's side of the handle on a lineage with no critter body (cases
//! 1216, 1221, 1223, 1226): the held lines and what befell them in the
//! continuation pane, and the keys that steer. 1 to 4 switch the first held
//! surface's rot, feed, leak and enter genes; 9 and 0 lower and raise the
//! shift of the held world-class lineage by a degree. `=` and `-` are the
//! camera's zoom, so the shift takes the free digits beside the genes'.

use super::Host;
use mesocosm_runtime::continuation::{HeldLine, LoopAnswer, LoopReading};
use cosmograph::directing::interim::Happening;
use std::collections::BTreeSet;
use winit::keyboard::Key;

/// The gene each of keys 1 to 4 switches.
pub(super) const GENE_KEYS: [(&str, &str); 4] =
    [("1", "gene:rot"), ("2", "gene:feed"), ("3", "gene:leak"), ("4", "gene:enter")];
/// The keys that lower and raise a world-class lineage's shift.
pub(super) const SHIFT_KEYS: [(&str, i32); 2] = [("9", -1), ("0", 1)];

/// The answer a key gives of what `reading` holds, if it gives one: a gene
/// switched on the first held surface, or the first held world-class
/// lineage's shift moved a degree and kept within its bound.
pub(super) fn key_answer(reading: &LoopReading, key: &str) -> Option<LoopAnswer> {
    if let Some((_, gene)) = GENE_KEYS.iter().find(|(name, _)| *name == key) {
        let HeldLine::Surface(held) = reading.held.iter().find(|line| matches!(line, HeldLine::Surface(_)))? else {
            return None;
        };
        let set = BTreeSet::from([(*gene).to_string()]);
        let (on, off) = if held.traits.contains(*gene) { (BTreeSet::new(), set) } else { (set, BTreeSet::new()) };
        return Some(LoopAnswer::Steer { lineage: held.lineage.clone(), on, off });
    }
    let (_, step) = SHIFT_KEYS.iter().find(|(name, _)| *name == key)?;
    reading.held.iter().find_map(|line| match line {
        HeldLine::World { lineage, shift, bound } => {
            let next = (*shift).saturating_add(i64::from(*step));
            let next = bound.map_or(next, |b| next.clamp(-i64::from(b), i64::from(b)));
            Some(LoopAnswer::Turn { lineage: lineage.clone(), shift: i32::try_from(next).unwrap_or(0) })
        },
        HeldLine::Surface(_) => None,
    })
}

impl Host {
    fn kingdom_of(&self, lineage: &str) -> String {
        self.runtime.sim().state().lineages.get(lineage).map(|line| line.kingdom.clone()).unwrap_or_default()
    }

    /// The words of a hold answer, for the choice's row.
    pub(super) fn hold_label(&self, lineage: &str) -> String {
        format!("hold {} {}", mesocosm_views::kingdom_word(&self.kingdom_of(lineage)), mesocosm_views::short_key(lineage))
    }

    /// The pane's facts about what is held and what befell it: a line each.
    pub(super) fn held_facts(&self, reading: &LoopReading) -> Vec<(String, String)> {
        let kingdom = |key: &str| self.kingdom_of(key);
        let mut facts: Vec<(String, String)> = reading.held.iter().map(|line| {
            ("held".to_string(), mesocosm_views::held_words(line, &kingdom(line.lineage())))
        }).collect();
        let happened = reading.happenings.iter().filter_map(|happening| match happening {
            Happening::Surfaced(event) => Some(mesocosm_views::surfaced_words(event, &kingdom)),
            Happening::Raised(raised) => Some(mesocosm_views::raised_words(raised, &kingdom)),
            Happening::LineageEnded { lineage } if reading.held.iter().any(|line| line.lineage() == lineage) =>
                Some(mesocosm_views::held_ended_words(lineage, &kingdom(lineage))),
            _ => None,
        });
        facts.extend(happened.map(|words| ("happened".to_string(), words)));
        if !reading.held.is_empty() {
            let keys = if reading.held.iter().any(|line| matches!(line, HeldLine::Surface(_))) { "[1-4] rot, feed, leak, enter" }
                else { "[9] lower, [0] raise the shift" };
            facts.push(("keys".into(), keys.into()));
        }
        facts
    }

    /// Keys 1 to 4, 9 and 0, while the loop reads something they steer.
    pub(super) fn try_holding_key(&mut self, key: &Key) -> bool {
        let Key::Character(c) = key else { return false };
        if !GENE_KEYS.iter().any(|(name, _)| *name == c.as_str()) && !SHIFT_KEYS.iter().any(|(name, _)| *name == c.as_str()) {
            return false;
        }
        let Ok(reading) = self.runtime.loop_reading() else { return false };
        let Some(answer) = key_answer(&reading, c.as_str()) else { return false };
        self.answer_round(answer);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmograph::directing::readings::{Held, Tally};

    fn reading(held: Vec<HeldLine>) -> LoopReading {
        LoopReading { cut: mesocosm_runtime::continuation::LoopCut { native: String::new(), game: String::new(), question: None },
            settings: None, phase: mesocosm_runtime::continuation::LoopPhase::Running, checkpoint: None, offers: vec![],
            happenings: vec![], lineages: vec![], starts: vec![], personal: None, world_ended_at: None,
            ending: Default::default(), sheet: None, held, holdable: vec![] }
    }

    fn surface(traits: &[&str]) -> HeldLine {
        HeldLine::Surface(Held { lineage: "lineage:micro".into(), traits: traits.iter().map(|t| t.to_string()).collect(),
            revision: 1, sites: Tally::default(), places: Tally::default(), hosts: Tally::default(), transit: 0, ended: false })
    }

    #[test]
    fn keys_one_to_four_toggle_the_first_held_surfaces_genes() {
        let read = reading(vec![surface(&["gene:rot"])]);
        let steer = |key| key_answer(&read, key);
        let off = BTreeSet::from(["gene:rot".to_string()]);
        assert_eq!(steer("1"), Some(LoopAnswer::Steer { lineage: "lineage:micro".into(), on: BTreeSet::new(), off }));
        let on = BTreeSet::from(["gene:enter".to_string()]);
        assert_eq!(steer("4"), Some(LoopAnswer::Steer { lineage: "lineage:micro".into(), on, off: BTreeSet::new() }));
        assert!(steer("5").is_none() && steer("9").is_none(), "no world-class lineage held, no shift");
        assert!(key_answer(&reading(vec![]), "1").is_none(), "nothing held, no claim");
    }

    #[test]
    fn keys_nine_and_zero_shift_the_world_within_its_bound() {
        let world = |shift| reading(vec![HeldLine::World { lineage: "world:ground".into(), shift, bound: Some(2) }]);
        let turn = |shift| Some(LoopAnswer::Turn { lineage: "world:ground".into(), shift });
        assert_eq!(key_answer(&world(1), "0"), turn(2));
        assert_eq!(key_answer(&world(2), "0"), turn(2), "kept within the bound");
        assert_eq!(key_answer(&world(-2), "9"), turn(-2));
        assert_eq!(key_answer(&world(0), "9"), turn(-1));
        assert!(key_answer(&world(0), "1").is_none());
    }
}
