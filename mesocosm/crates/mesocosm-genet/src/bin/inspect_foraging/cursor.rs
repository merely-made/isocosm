// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Reuse native command replay between exact diagnostic cuts.
use super::*;

pub(super) struct Cursor<'a> {
    source: &'a Session,
    cut: Option<(usize, Session)>,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(source: &'a Session) -> Self {
        Self { source, cut: None }
    }

    pub(super) fn before(&mut self, index: usize) -> Result<&Session, String> {
        let entry = self
            .source
            .entries
            .get(index)
            .ok_or("inspection entry absent")?;
        if self.cut.as_ref().is_none_or(|(next, _)| index < *next) {
            self.cut = Some((index, self.source.before_entry(&entry.id)?));
        }
        let (next, cut) = self.cut.as_mut().ok_or("inspection cut absent")?;
        for previous in &self.source.entries[*next..index] {
            // Older histories can include unlogged native clock intervals.
            // The existing prefix reader owns that case too.
            if cut.sim.state().tick != previous.tick {
                *cut = self.source.before_entry(&entry.id)?;
                break;
            }
            cut.command(previous.command.clone())?;
            if cut.entries.last() != Some(previous) {
                return Err("inspection cursor differs from authenticated history".into());
            }
        }
        *next = index;
        Ok(cut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_keeps_exact_cuts_across_commands_clock_gaps_and_backwards_reads() {
        let g = isocosm::Founding {
            seed: 7,
            sites: 2,
            population: 8,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut source = Session::new(g, Execution::Individuals).unwrap();
        let actor = *source.sim.state().population.groups.keys().next().unwrap();
        for _ in 0..2 {
            source.command(Command::Inspect(actor)).unwrap();
        }
        source.advance(2).unwrap();
        for _ in 0..2 {
            source.command(Command::Inspect(actor)).unwrap();
        }
        let mut cursor = Cursor::new(&source);
        for index in [0, 1, 3, 2] {
            let expected = source.before_entry(&source.entries[index].id).unwrap();
            let actual = cursor.before(index).unwrap();
            assert_eq!(actual.entries, expected.entries);
            assert_eq!(actual.sim.state_hash(), expected.sim.state_hash());
        }
    }
}
