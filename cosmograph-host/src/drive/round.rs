// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Two time modes (case 1080). In a round, world time runs at one, two, four
//! or eight times real time: examined bodies step sixty times a real second
//! in the product's foreground, each step a sixtieth of a second of world
//! time times the speed, and the sim's minute tick falls due every sixty
//! world seconds, every 60/speed real seconds. Out of a round, simming runs
//! the background dial as the core's fixed-step clock always has.
//!
//! The minute tick runs off the frame thread by default (case 1081): the
//! foreground keeps stepping toward its last targets while a tick is in
//! flight, and each boundary's handback is queued behind it. Running the
//! tick in step on the frame thread is a setting. Either way ticks run in
//! the order they fell due, one at a time, so both modes end alike.
//!
//! Pace is product state: it never enters the sim's record, so a product
//! logs a change of pace with its own inputs.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread::JoinHandle;

use serde::{Deserialize, Serialize};

use super::clock::Clock;

/// Foreground steps a real second, at any speed.
pub const STEPS_PER_SECOND: u32 = 60;
/// World time one background tick counts (case 1072).
pub const MINUTE_US: u64 = 60_000_000;
const SECOND_US: u64 = 1_000_000;

/// A round's world time against real time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Speed {
    X1,
    X2,
    X4,
    X8,
}

impl Speed {
    pub const ALL: [Speed; 4] = [Speed::X1, Speed::X2, Speed::X4, Speed::X8];

    pub fn factor(self) -> u64 {
        match self {
            Speed::X1 => 1,
            Speed::X2 => 2,
            Speed::X4 => 4,
            Speed::X8 => 8,
        }
    }

    pub fn from_factor(factor: u64) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.factor() == factor)
    }
}

/// How a product's time runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pace {
    /// In a round: a sixty-step foreground and the minute in the background.
    Round { speed: Speed },
    /// Watching or simming the world: background ticks on the dial.
    Simming { ticks_per_second: u32 },
}

/// What elapsed real time authorised.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Due {
    /// Foreground steps; none while simming.
    pub steps: u64,
    /// Background ticks.
    pub ticks: u64,
}

/// Turns elapsed real time into foreground steps and background ticks.
/// Integer throughout, so the totals depend only on the time elapsed and the
/// pace it elapsed under, never on how the time arrived.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pacer {
    pace: Pace,
    steps: Clock,
    dial: Option<Clock>,
    /// Round world time not yet counted into a tick, in microseconds.
    world_us: u128,
    ticks: u64,
}

impl Pacer {
    pub fn new(pace: Pace) -> Self {
        let mut pacer = Self { pace, steps: Clock::new(STEPS_PER_SECOND), dial: None, world_us: 0, ticks: 0 };
        pacer.set_pace(pace);
        pacer
    }

    pub fn pace(&self) -> Pace {
        self.pace
    }

    /// Changes pace from now on; time already counted keeps its own pace.
    pub fn set_pace(&mut self, pace: Pace) {
        self.pace = pace;
        self.dial = match pace {
            Pace::Simming { ticks_per_second } => Some(Clock::new(ticks_per_second)),
            Pace::Round { .. } => None,
        };
    }

    /// Background ticks this pacer has ever authorised.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// World time one foreground step counts, as microseconds over a
    /// denominator, exact: a sixtieth of a second times the speed.
    pub fn step_world_us(&self) -> Option<(u64, u64)> {
        match self.pace {
            Pace::Round { speed } => Some((SECOND_US * speed.factor(), u64::from(STEPS_PER_SECOND))),
            Pace::Simming { .. } => None,
        }
    }

    pub fn advance(&mut self, elapsed_us: u64) -> Due {
        let due = match self.pace {
            Pace::Round { speed } => {
                let steps = self.steps.advance(elapsed_us, u64::MAX).steps;
                self.world_us += u128::from(elapsed_us) * u128::from(speed.factor());
                let ticks = (self.world_us / u128::from(MINUTE_US)) as u64;
                self.world_us -= u128::from(ticks) * u128::from(MINUTE_US);
                Due { steps, ticks }
            },
            Pace::Simming { .. } => {
                let dial = self.dial.as_mut().expect("simming keeps a dial");
                Due { steps: 0, ticks: dial.advance(elapsed_us, u64::MAX).steps }
            },
        };
        self.ticks += due.ticks;
        due
    }
}

/// Where a background tick runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TickMode {
    /// Off the frame thread (case 1081's default).
    #[default]
    Worker,
    /// On the frame thread, before the next foreground step.
    InStep,
}

type Run<J, O> = Box<dyn FnMut(J) -> O + Send>;

/// Runs a product's background ticks in the order they fell due, one at a
/// time, on a worker or in step. The frame thread submits each boundary's
/// job (the tick's inputs and the queued handback) and polls for answers;
/// in worker mode neither call waits on a tick.
pub struct Ticker<J: Send + 'static, O: Send + 'static> {
    mode: TickMode,
    inline: Option<Run<J, O>>,
    jobs: Option<Sender<J>>,
    answers: Option<Receiver<O>>,
    worker: Option<JoinHandle<()>>,
    in_flight: usize,
    done: VecDeque<O>,
}

impl<J: Send + 'static, O: Send + 'static> Ticker<J, O> {
    pub fn new(mode: TickMode, mut run: impl FnMut(J) -> O + Send + 'static) -> Self {
        match mode {
            TickMode::InStep => Self { mode, inline: Some(Box::new(run)), jobs: None, answers: None,
                worker: None, in_flight: 0, done: VecDeque::new() },
            TickMode::Worker => {
                let (jobs, inbox) = channel::<J>();
                let (outbox, answers) = channel::<O>();
                let worker = std::thread::spawn(move || {
                    for job in inbox {
                        if outbox.send(run(job)).is_err() {
                            return;
                        }
                    }
                });
                Self { mode, inline: None, jobs: Some(jobs), answers: Some(answers), worker: Some(worker),
                    in_flight: 0, done: VecDeque::new() }
            },
        }
    }

    pub fn mode(&self) -> TickMode {
        self.mode
    }

    /// Ticks submitted whose answers have not been taken.
    pub fn pending(&self) -> usize {
        self.in_flight + self.done.len()
    }

    /// Queues one tick. In step, it runs now.
    pub fn submit(&mut self, job: J) -> Result<(), String> {
        match self.mode {
            TickMode::InStep => {
                let run = self.inline.as_mut().expect("an in-step ticker runs inline");
                self.done.push_back(run(job));
            },
            TickMode::Worker => {
                self.jobs.as_ref().expect("a worker ticker keeps its queue").send(job)
                    .map_err(|_| "the tick worker stopped".to_string())?;
                self.in_flight += 1;
            },
        }
        Ok(())
    }

    /// The answers ready now, in submission order; never waits.
    pub fn poll(&mut self) -> Result<Vec<O>, String> {
        if let Some(answers) = &self.answers {
            loop {
                match answers.try_recv() {
                    Ok(out) => {
                        self.in_flight -= 1;
                        self.done.push_back(out);
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) if self.in_flight > 0 => {
                        return Err("the tick worker stopped".into());
                    },
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        }
        Ok(self.done.drain(..).collect())
    }

    /// Waits for every submitted tick: for saving, closing or a test.
    pub fn drain(&mut self) -> Result<Vec<O>, String> {
        if let Some(answers) = &self.answers {
            while self.in_flight > 0 {
                let out = answers.recv().map_err(|_| "the tick worker stopped".to_string())?;
                self.in_flight -= 1;
                self.done.push_back(out);
            }
        }
        Ok(self.done.drain(..).collect())
    }
}

impl<J: Send + 'static, O: Send + 'static> Drop for Ticker<J, O> {
    fn drop(&mut self) {
        self.jobs = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
#[path = "round_tests.rs"]
mod tests;
