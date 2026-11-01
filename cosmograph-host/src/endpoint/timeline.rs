// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The viewer's timeline (case 1038, Record 4): the main line is the
//! endpoint's world; scrubbing reads a past tick by replay and never changes
//! a line; a branch is forked by `Session::fork_at` at the viewed tick and
//! becomes its own when a change is made inside it (case 1048).

use std::collections::BTreeMap;

use cosmograph::{Command, Session};

use crate::world::World;

/// One played line and the past states read from it, by tick.
struct Line {
    world: World,
    past: BTreeMap<u64, Session>,
    /// The line and tick it was forked from; none for the main line.
    from: Option<(usize, u64)>,
}

impl Line {
    fn new(world: World, from: Option<(usize, u64)>) -> Result<Self, String> {
        let head = world.read(|s| (s.sim.state().tick, s.clone()))?;
        Ok(Self { world, past: BTreeMap::from([head]), from })
    }
    fn head(&self) -> Result<u64, String> { self.world.read(|s| s.sim.state().tick) }
}

pub struct Timeline {
    lines: Vec<Line>,
    line: usize,
    /// The viewed tick; none is the line's head.
    at: Option<u64>,
}

impl Timeline {
    pub fn new(main: &World) -> Result<Self, String> {
        Ok(Self { lines: vec![Line::new(main.share(), None)?], line: 0, at: None })
    }

    pub fn line(&self) -> usize { self.line }
    pub fn lines(&self) -> usize { self.lines.len() }
    pub fn forked_from(&self) -> Option<(usize, u64)> { self.lines[self.line].from }
    pub fn head(&self) -> Result<u64, String> { self.lines[self.line].head() }
    /// Whether the view is at the current line's head, where changes are made.
    pub fn at_head(&self) -> bool { self.at.is_none() }
    pub fn world(&self) -> &World { &self.lines[self.line].world }

    /// Reads the viewed state: the head, or a past tick replayed once and kept.
    pub fn read<T>(&mut self, read: impl FnOnce(&Session) -> T) -> Result<T, String> {
        let line = &mut self.lines[self.line];
        let Some(tick) = self.at else { return line.world.read(read) };
        if !line.past.contains_key(&tick) {
            let past = line.world.read(|s| s.fork_at(tick, s.branch.clone()))??;
            line.past.insert(tick, past);
        }
        Ok(read(&line.past[&tick]))
    }

    /// Advances the current line tick by tick, keeping each state for scrubbing.
    pub fn advance(&mut self, ticks: u64) -> Result<(), String> {
        let line = &mut self.lines[self.line];
        for _ in 0..ticks {
            line.world.advance(1)?;
            let (tick, state) = line.world.read(|s| (s.sim.state().tick, s.clone()))?;
            line.past.insert(tick, state);
        }
        self.at = None;
        Ok(())
    }

    /// Views a past tick of the current line; the head returns to it.
    pub fn scrub(&mut self, tick: u64) -> Result<(), String> {
        let head = self.head()?;
        if tick > head { return Err(format!("tick {tick} is past this line's head, {head}")); }
        self.at = (tick < head).then_some(tick);
        Ok(())
    }

    /// Forks a branch at the viewed tick and plays it.
    pub fn branch(&mut self) -> Result<(), String> {
        let tick = match self.at { Some(tick) => tick, None => self.head()? };
        let key = format!("branch:viewer:{}:{}:{}", self.line, tick, self.lines.len());
        let fork = self.world().read(|s| s.fork_at(tick, key))??;
        self.lines.push(Line::new(World::new(fork), Some((self.line, tick)))?);
        self.line = self.lines.len() - 1;
        self.at = None;
        Ok(())
    }

    /// Plays another line at its head.
    pub fn switch(&mut self, line: usize) -> Result<(), String> {
        if line >= self.lines.len() { return Err(format!("there is no line {line}")); }
        self.line = line;
        self.at = None;
        Ok(())
    }

    /// A change inside a branch, at its head (case 1048): domain truth for
    /// that branch alone. The main line takes none from the viewer.
    pub fn change(&mut self, command: Command) -> Result<String, String> {
        if self.line == 0 { return Err("changes are made inside a branch".into()); }
        if self.at.is_some() { return Err("changes are made at the branch's head".into()); }
        let line = &mut self.lines[self.line];
        let outcome = line.world.command(command)?;
        let (tick, state) = line.world.read(|s| (s.sim.state().tick, s.clone()))?;
        line.past.insert(tick, state);
        Ok(outcome)
    }
}
