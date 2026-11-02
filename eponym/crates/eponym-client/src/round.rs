// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The standalone client's round (cases 1096, 1118, 1120, 1121): the played
//! body is stepped locally sixty times a real second and drawn from that
//! pose; at each minute its runs go to the game as one intent with the
//! minute itself, applied on a worker over a copy of the held session, and
//! the frame adopts the answer. One minute is in flight at a time.

use cosmograph_host::drive::{Pace, Pacer, Speed, TickMode, Ticker};
use eponym_play::identity::SubjectId;
use eponym_play::{GameEvent, GameIntent, MotionInput, MotionPose, MotionRules, MotionRun, SAFE_FALL};

use crate::producer::{Held, SceneModel};

type Job = (Held, Vec<GameIntent>);
type Answer = (Held, Result<Vec<GameEvent>, String>);

/// The played body's motion since the last minute it handed in.
#[derive(Clone, Debug)]
struct Drawn {
    subject: SubjectId,
    /// Where the runs start: the game's pose, or the end it was last sent.
    from: MotionPose,
    pose: MotionPose,
    runs: Vec<MotionRun>,
    rules: MotionRules,
}

pub struct Round {
    pacer: Pacer,
    speed: Speed,
    ticker: Ticker<Job, Answer>,
    drawn: Option<Drawn>,
    in_flight: bool,
    /// Minutes that fell due while one was in flight.
    owed: u64,
    /// Motion cut at a damaging landing while a minute was in flight.
    cut: bool,
}

/// What one frame of a round did.
#[derive(Debug, Default)]
pub struct Frame {
    /// Fixed steps the frame stepped locally.
    pub steps: u64,
    /// Minutes the frame submitted.
    pub minutes: u64,
}

/// What adopting the worker's answers did.
#[derive(Debug, Default)]
pub struct Adopted {
    pub events: Vec<GameEvent>,
    pub refusals: Vec<String>,
}

impl Adopted {
    pub fn any(&self) -> bool {
        !self.events.is_empty() || !self.refusals.is_empty()
    }
}

/// Local steps one frame may take, so a stalled frame does not freeze the
/// next one catching up; the excess real time is let go.
const MAX_FRAME_STEPS: u64 = 8;

impl Round {
    pub fn new(speed: Speed, mode: TickMode) -> Self {
        let ticker = Ticker::new(mode, |(mut held, intents): Job| {
            let events = held.apply_batch(&intents).map_err(|e| format!("{e:?}"));
            (held, events)
        });
        Self { pacer: Pacer::new(Pace::Round { speed }), speed, ticker, drawn: None, in_flight: false, owed: 0, cut: false }
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }

    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
        self.pacer.set_pace(Pace::Round { speed });
    }

    pub fn in_flight(&self) -> bool {
        self.in_flight
    }

    /// The played body's locally stepped pose, once it has stepped.
    pub fn drawn(&self) -> Option<(SubjectId, MotionPose)> {
        self.drawn.as_ref().map(|d| (d.subject, d.pose))
    }
}

impl SceneModel {
    /// Begins playing in rounds when the world was founded in them.
    pub fn start_rounds(&mut self, speed: Speed, mode: TickMode) -> bool {
        if !self.game().world().rounds() {
            return false;
        }
        self.round = Some(Round::new(speed, mode));
        true
    }

    pub fn round(&self) -> Option<&Round> {
        self.round.as_ref()
    }

    pub fn round_mut(&mut self) -> Option<&mut Round> {
        self.round.as_mut()
    }

    /// Whether a minute's answer is still owed: game intents wait for it.
    pub fn minute_pending(&self) -> bool {
        self.round.as_ref().is_some_and(|r| r.in_flight)
    }

    /// The pose a body is drawn at: the played body's locally stepped pose
    /// in a round, else the game's (case 1118).
    pub fn drawn_pose(&self, subject: SubjectId) -> Option<MotionPose> {
        self.round.as_ref().and_then(|r| r.drawn.as_ref()).filter(|d| d.subject == subject)
            .map(|d| d.pose).or_else(|| self.game().pose(subject))
    }

    /// The cell a body is drawn in.
    pub fn drawn_position(&self, subject: SubjectId) -> Option<[i32; 3]> {
        match self.round.as_ref().and_then(|r| r.drawn.as_ref()).filter(|d| d.subject == subject) {
            Some(d) => d.pose.cell().ok(),
            None => self.game().movement().position(subject),
        }
    }

    /// Real time elapsed in a round: up to the frame's due fixed steps of
    /// the played body under `input` (none while it is at rest), then each
    /// due minute submitted, or owed while one is in flight.
    pub fn advance_round(&mut self, elapsed_us: u64, rules: MotionRules,
        mut input: impl FnMut(&SceneModel) -> Option<MotionInput>) -> Result<Frame, String> {
        let Some(round) = self.round.as_mut() else { return Ok(Frame::default()) };
        let due = round.pacer.advance(elapsed_us);
        let speed = round.speed.factor();
        let mut frame = Frame::default();
        for _ in 0..(due.steps * speed).min(MAX_FRAME_STEPS * speed) {
            let Some(input) = input(self) else { break };
            self.step_drawn(input, rules)?;
            frame.steps += 1;
        }
        let round = self.round.as_mut().expect("a round");
        round.owed += due.ticks;
        while self.round.as_ref().is_some_and(|r| r.owed > 0 && !r.in_flight) {
            self.submit_minute(true)?;
            self.round.as_mut().expect("a round").owed -= 1;
            frame.minutes += 1;
        }
        Ok(frame)
    }

    /// One fixed step of the played body outside the pacer: a scenario's
    /// movement act, which steps outright rather than waiting on real time.
    pub fn step_played(&mut self, input: MotionInput, rules: MotionRules) -> Result<(), String> {
        if self.round.is_none() {
            return Err("not in a round".into());
        }
        self.step_drawn(input, rules)
    }

    /// Sends the played body's motion so far as one intent, without the
    /// minute, so the game stands it where it is drawn before an act that
    /// reads its place. `false` when nothing was sent.
    pub fn hand_in_motion(&mut self) -> Result<bool, String> {
        let unsent = self.round.as_ref().is_some_and(|r| !r.in_flight
            && r.drawn.as_ref().is_some_and(|d| !d.runs.is_empty()));
        if unsent {
            self.submit_minute(false)?;
        }
        Ok(unsent)
    }

    /// The minute now, as a scenario's fixture control; owed while one is
    /// in flight.
    pub fn minute_now(&mut self) -> Result<(), String> {
        match self.round.as_mut() {
            None => Err("not in a round".into()),
            Some(r) if r.in_flight => { r.owed += 1; Ok(()) },
            Some(_) => self.submit_minute(true),
        }
    }

    /// Forgets the drawn motion, as a load replaces the session it began on.
    pub(crate) fn forget_drawn(&mut self) {
        if let Some(r) = self.round.as_mut() {
            r.drawn = None;
            r.cut = false;
        }
    }

    /// One fixed step of the played body, solved as its intent will be and
    /// held as a run. A damaging landing hands the runs in at once, since
    /// the game's harm changes what the next step can do.
    fn step_drawn(&mut self, input: MotionInput, rules: MotionRules) -> Result<(), String> {
        let played = self.played();
        let base = match self.round.as_ref().and_then(|r| r.drawn.clone()) {
            Some(d) if d.subject == played => d,
            _ => {
                let pose = self.game().movement().pose(played).ok_or("the played body has no pose")?;
                Drawn { subject: played, from: pose, pose, runs: Vec::new(), rules }
            },
        };
        let outcome = self.game().predict(played, base.pose, input, rules).map_err(|e| format!("{e:?}"))?;
        let mut drawn = base;
        drawn.pose = outcome.pose;
        drawn.rules = rules;
        match drawn.runs.last_mut() {
            Some(run) if run.input == input => run.steps += 1,
            _ => drawn.runs.push(MotionRun { input, steps: 1 }),
        }
        let round = self.round.as_mut().expect("a round");
        round.drawn = Some(drawn);
        if outcome.landed_distance.is_some_and(|d| d > SAFE_FALL) {
            if round.in_flight { round.cut = true } else { self.submit_minute(false)?; }
        }
        Ok(())
    }

    /// The played body's runs as one intent, then the minute when `minute`,
    /// sent to the worker over a copy of the held session.
    fn submit_minute(&mut self, minute: bool) -> Result<(), String> {
        let game = self.game();
        let mut tick = game.next_tick().0;
        let mut intents = Vec::new();
        let drawn = self.round.as_ref().and_then(|r| r.drawn.clone()).filter(|d| !d.runs.is_empty());
        if let Some(d) = &drawn {
            let revision = game.bodies().get(d.subject).ok_or("the played body is absent")?.revision;
            intents.push(GameIntent::Steps { tick: eponym_play::identity::Tick(tick), subject: d.subject, revision,
                step: d.from.step + 1, runs: d.runs.clone(), end: d.pose, rules: d.rules });
            tick += 1;
        }
        if minute {
            intents.push(GameIntent::Minute { tick: eponym_play::identity::Tick(tick) });
        }
        if intents.is_empty() {
            return Ok(());
        }
        let held = self.held.clone();
        let round = self.round.as_mut().expect("a round");
        if let Some(d) = round.drawn.as_mut().filter(|_| drawn.is_some()) {
            d.from = d.pose;
            d.runs.clear();
        }
        round.cut = false;
        round.in_flight = true;
        round.ticker.submit((held, intents))
    }

    /// Takes the worker's answers: an accepted minute replaces the held
    /// session; a refused one keeps it and draws the body where the game has
    /// it. Owed minutes and cut motion go out next.
    pub fn poll_round(&mut self) -> Result<Adopted, String> {
        let Some(round) = self.round.as_mut() else { return Ok(Adopted::default()) };
        let answers = round.ticker.poll()?;
        self.adopt(answers)
    }

    /// Waits for the minute in flight: for saving, closing or a test.
    pub fn drain_round(&mut self) -> Result<Adopted, String> {
        let Some(round) = self.round.as_mut() else { return Ok(Adopted::default()) };
        let answers = round.ticker.drain()?;
        self.adopt(answers)
    }

    fn adopt(&mut self, answers: Vec<Answer>) -> Result<Adopted, String> {
        let mut adopted = Adopted::default();
        for (held, result) in answers {
            let round = self.round.as_mut().expect("a round");
            round.in_flight = false;
            match result {
                Ok(events) => {
                    self.held = held;
                    adopted.events.extend(events);
                },
                Err(why) => {
                    round.drawn = None;
                    round.cut = false;
                    adopted.refusals.push(why);
                },
            }
        }
        if self.round.as_ref().is_some_and(|r| !r.in_flight && r.cut) {
            self.submit_minute(false)?;
        }
        while self.round.as_ref().is_some_and(|r| r.owed > 0 && !r.in_flight) {
            self.submit_minute(true)?;
            self.round.as_mut().expect("a round").owed -= 1;
        }
        Ok(adopted)
    }
}
