//! Opt-in headed trial of Ars Magica (case 1063): every step is a painted
//! endpoint button. `ISOCOSM_ARS_SELFTEST` is `draft` (stop at Virtues and
//! Flaws, where a refused option shows its reason), `finished` (draft the
//! sample Bonisagus magus and finish the sheet) or `study` (over an
//! `ars_table --ready` ledger: study the covenant's summa, propose the rest of
//! the season, advance it, and complete the study).
use super::painted::Painted;
use super::*;
use cambium_genet_winit_host::ScrollAlign;
use isocosm_native::{SHEETS, TABLE};
use isocosm_system::ars::sample::BONISAGUS_MAGUS;
use isonetry::family::{Checkpoint, FamilyReplica};

/// How many sample choices the draft trial makes: type, House, its benefit.
const DRAFT_CHOICES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Select,
    Create,
    Choose,
    Finish,
    /// Study steps: begin, propose, advance (on the table tab), complete.
    Study(u8),
    Verify,
    Capture(u8),
    Done,
}

pub(crate) struct ArsSelftest {
    mode: &'static str,
    phase: Phase,
    started: Instant,
    painted: Painted,
    body: Option<u64>,
    before: Option<(Checkpoint, u64)>,
    log: usize,
    waiting: bool,
}

impl ArsSelftest {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var("ISOCOSM_ARS_SELFTEST").ok().map(|mode| {
            let mode = ["draft", "finished", "study"].into_iter().find(|m| *m == mode).expect("ars selftest mode");
            Self { mode, phase: Phase::Select, started: Instant::now(), painted: Painted::default(), body: None,
                before: None, log: 0, waiting: false }
        })
    }
    pub(crate) fn done(&self) -> bool {
        self.phase == Phase::Done
    }
    /// The first body the Character sheets page offers `intent` on.
    fn offering(ctx: &Ctx<'_>, intent: &str) -> Option<u64> {
        let page = ctx.runner.state().vtt_projections.iter().find(|p| p.id == SHEETS)?;
        page.cards.iter().find(|c| c.actions.iter().any(|a| a.id == intent))?
            .values.iter().find(|v| v.label == "Native body")?.value.parse().ok()
    }
    pub(crate) fn drive(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        if self.done() {
            return;
        }
        // A season of native downtime takes a while in a debug build.
        assert!(self.started.elapsed() < Duration::from_secs(600),
            "Ars selftest stalled: {:?}; {}", self.phase, ctx.runner.state().status);
        let Some(table) = app.family_table.as_mut() else { return };
        if self.before.is_none() {
            table.drain_records();
            self.before = Some((table.checkpoint().unwrap(), table.sequence()));
        }
        let state = table.edition_state().expect("edition state readable");
        let made = |body: Option<u64>| {
            let id = isocosm_native::ruleset::token(body?).ok()?;
            state.as_ref()?.drafts.get(&id).map(|d| d.choices.len())
        };
        let logged = state.as_ref().map_or(0, |s| s.log.len());
        match self.phase {
            Phase::Select => {
                if self.painted.tab(ctx, SHEETS) {
                    let intent = if self.mode == "study" { "vtt.ars-study-0" } else { "vtt.sheet-create" };
                    self.body = Self::offering(ctx, intent);
                    if self.body.is_some() {
                        self.phase = if self.mode == "study" { Phase::Study(0) } else { Phase::Create };
                    }
                }
            },
            Phase::Create => {
                if made(self.body).is_some() {
                    self.phase = Phase::Choose;
                } else {
                    self.painted.button(ctx, "vtt.sheet-create", self.body);
                }
            },
            Phase::Choose => {
                let wanted = if self.mode == "draft" { DRAFT_CHOICES } else { BONISAGUS_MAGUS.len() };
                let have = made(self.body).unwrap_or(0);
                if have >= wanted {
                    self.phase = if self.mode == "draft" { Phase::Verify } else { Phase::Finish };
                } else if have != self.log {
                    self.log = have;
                    self.waiting = false;
                } else if !self.waiting
                    && self.painted.button(ctx, &format!("vtt.draft-{}", BONISAGUS_MAGUS[have].1), self.body) {
                    self.waiting = true;
                }
            },
            Phase::Finish => {
                let id = isocosm_native::ruleset::token(self.body.unwrap()).unwrap();
                if state.as_ref().is_some_and(|s| s.sheets.contains_key(&id)) {
                    self.phase = Phase::Verify;
                } else {
                    self.painted.button(ctx, "vtt.draft-finish", self.body);
                }
            },
            Phase::Study(step) => {
                if self.waiting {
                    // The proposal is native time, not an edition log entry.
                    let moved = if step == 1 { ctx.runner.state().vtt_projections.iter().any(|p| p.id == TABLE
                        && p.cards.iter().any(|c| c.actions.iter().any(|a| a.id == "vtt.advance"))) } else { logged > self.log };
                    let advanced = step == 2 && Self::offering(ctx, "vtt.ars-complete").is_some();
                    if moved || advanced {
                        self.waiting = false;
                        self.phase = if step >= 3 { Phase::Verify } else { Phase::Study(step + 1) };
                    }
                    if step == 2 && !advanced {
                        self.painted.tab(ctx, SHEETS);
                    }
                    return;
                }
                self.log = logged;
                let clicked = match step {
                    0 => self.painted.button(ctx, "vtt.ars-study-0", self.body),
                    1 => self.painted.button(ctx, "vtt.ars-propose-season", self.body),
                    2 => self.painted.tab(ctx, TABLE) && self.painted.button(ctx, "vtt.advance", None),
                    _ => self.painted.tab(ctx, SHEETS) && self.painted.button(ctx, "vtt.ars-complete", self.body),
                };
                if clicked {
                    self.waiting = true;
                }
            },
            Phase::Verify => {
                let (checkpoint, sequence) = self.before.as_ref().unwrap();
                let suffix = table.persisted_suffix_after(*sequence).unwrap();
                let replica = FamilyReplica::install(checkpoint.clone(), &suffix, cosmograph::Execution::Grouped).unwrap();
                assert_eq!(replica.snapshot().unwrap(), table.replication_snapshot().unwrap(), "durable replay");
                let state = state.expect("edition state");
                let id = isocosm_native::ruleset::token(self.body.unwrap()).unwrap();
                eprintln!("[ars-selftest] PASS mode={} body={} choices={} sheet={} corpus_xp={:?} accepted_records={} durable_replay=exact; last={:?}",
                    self.mode, self.body.unwrap(), state.drafts[&id].choices.len(), state.sheets.contains_key(&id),
                    state.sheets.get(&id).and_then(|s| s.int("ars:xp:corpus")), suffix.len(),
                    state.log.last().map(|l| &l.note));
                self.phase = Phase::Capture(0);
            },
            Phase::Capture(frames) => {
                if frames == 0 {
                    let wanted = match self.mode { "draft" => "Next decision: ", "finished" => "Character: ", _ => "Result: " };
                    let node = {
                        let dom = ctx.runner.dom();
                        let dom = dom.borrow();
                        dom.all_with_class(dom.document(), "vtt-value").into_iter().find(|node| {
                            dom.dom_children(*node).filter_map(|child| dom.text(child)).collect::<String>().starts_with(wanted)
                        })
                    };
                    if let Some(node) = node {
                        ctx.scroll_into_view(node, ScrollAlign::Start);
                    }
                }
                self.phase = if frames >= 3 { Phase::Done } else { Phase::Capture(frames + 1) };
            },
            Phase::Done => {},
        }
    }
}

impl App {
    pub(crate) fn maybe_ars_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(mut test) = self.ars_selftest.take() else { return };
        test.drive(self, ctx);
        self.ars_selftest = Some(test);
    }
}
