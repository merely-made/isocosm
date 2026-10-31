//! Opt-in headed trial of Pathfinder 2e creation: every choice is a painted
//! endpoint button on the Character sheets page. `ISOCOSM_PF2E_SELFTEST`
//! is `draft` (stop at the ancestry feat, where refused options show their
//! reasons), `finished` (finish a dwarf fighter and show the sheet) or
//! `play` (over a `pf2e_table --ready` ledger: start an encounter, Strike
//! twice, Raise a Shield).
use super::painted::Painted;
use super::*;
use cambium_genet_winit_host::ScrollAlign;
use isocosm_native::{SHEETS, ruleset::State};
use isonetry::family::{Checkpoint, FamilyReplica};

const HUMAN_ROGUE: &[&str] = &[
    "human", "dex", "int", "skilled-human", "stealth", "street-urchin", "dex", "con", "rogue", "thief",
    "athletics", "acrobatics", "dex", "dex", "con", "wis", "cha", "arcana", "crafting", "diplomacy",
    "medicine", "nature", "occultism", "religion", "society", "Draconic", "Elven",
];
const DWARF_FIGHTER: &[&str] = &[
    "dwarf", "listed", "str", "rock-dwarf", "warrior", "str", "con", "fighter", "athletics", "str",
    "dex", "con", "wis", "int", "acrobatics", "medicine", "survival", "crafting", "Gnomish", "dwarven-lore",
    "society", "vicious-swing", "longsword", "steel-shield", "scale-mail", "done",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Select,
    Create,
    Choose(usize),
    Finish,
    /// Encounter steps: start, Strike, Strike, Raise a Shield.
    Play(usize),
    Verify,
    Capture(u8),
    Done,
}

pub(crate) struct PathfinderSelftest {
    finished: bool,
    play: bool,
    /// Log length when the last play step was clicked.
    log: usize,
    phase: Phase,
    started: Instant,
    painted: Painted,
    body: Option<u64>,
    before: Option<(Checkpoint, u64)>,
    /// A choice was clicked and its acceptance has not arrived yet.
    waiting: bool,
}

impl PathfinderSelftest {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var("ISOCOSM_PF2E_SELFTEST").ok().map(|mode| {
            assert!(["draft", "finished", "play"].contains(&mode.as_str()), "pf2e selftest mode is draft, finished or play");
            Self { finished: mode == "finished", play: mode == "play", log: 0, phase: Phase::Select, started: Instant::now(),
                painted: Painted::default(), body: None, before: None, waiting: false }
        })
    }
    pub(crate) fn done(&self) -> bool {
        self.phase == Phase::Done
    }
    fn script(&self) -> &'static [&'static str] {
        if self.finished { DWARF_FIGHTER } else { HUMAN_ROGUE }
    }
    /// The first body the Character sheets page offers to start.
    fn startable(ctx: &Ctx<'_>) -> Option<u64> {
        let state = ctx.runner.state();
        let page = state.vtt_projections.iter().find(|p| p.id == SHEETS)?;
        page.cards.iter().find(|c| c.actions.iter().any(|a| a.id == "vtt.sheet-create"))?
            .values.iter().find(|v| v.label == "Native body")?.value.parse().ok()
    }
    pub(crate) fn drive(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        if self.done() {
            return;
        }
        assert!(self.started.elapsed() < Duration::from_secs(180),
            "Pathfinder selftest stalled: {:?}; {}", self.phase, ctx.runner.state().status);
        let Some(table) = app.family_table.as_mut() else { return };
        if self.before.is_none() {
            table.drain_records();
            self.before = Some((table.checkpoint().unwrap(), table.sequence()));
        }
        let state = table.edition_state().expect("edition state readable");
        let made = |state: &Option<State>, body: Option<u64>| {
            let id = isocosm_native::ruleset::token(body?).ok()?;
            state.as_ref()?.drafts.get(&id).map(|d| d.choices.len())
        };
        match self.phase {
            Phase::Select if self.play => {
                if self.painted.tab(ctx, SHEETS) {
                    self.phase = Phase::Play(0);
                }
            },
            Phase::Play(n) => {
                let state = state.expect("a --ready ledger has edition state");
                if self.waiting {
                    if state.log.len() > self.log {
                        self.waiting = false;
                        self.phase = if n >= 3 { Phase::Verify } else { Phase::Play(n + 1) };
                    }
                    return;
                }
                let current = state.encounter.as_ref().and_then(|e| e.current()).map(|t| u64::from(t.token));
                self.body = current.or(self.body);
                let target = state.sheets.keys().map(|id| u64::from(id.0)).find(|id| Some(*id) != current);
                let (intent, body) = match n {
                    0 => ("vtt.pf2e-start-encounter".to_owned(), None),
                    1 | 2 => (format!("vtt.pf2e-rule-strike-{}", target.unwrap()), current),
                    _ => ("vtt.pf2e-raise-shield".to_owned(), current),
                };
                self.log = state.log.len();
                if self.painted.button(ctx, &intent, body) {
                    self.waiting = true;
                }
            },
            Phase::Select => {
                if self.painted.tab(ctx, SHEETS) {
                    self.body = Self::startable(ctx);
                    if self.body.is_some() {
                        self.phase = Phase::Create;
                    }
                }
            },
            Phase::Create => {
                if made(&state, self.body).is_some() {
                    self.phase = Phase::Choose(0);
                } else {
                    self.painted.button(ctx, "vtt.sheet-create", self.body);
                }
            },
            Phase::Choose(n) => {
                let script = self.script();
                let have = made(&state, self.body).unwrap_or(0);
                if have >= script.len() {
                    self.phase = if self.finished { Phase::Finish } else { Phase::Verify };
                } else if have > n {
                    self.phase = Phase::Choose(have);
                } else if !self.waiting && self.painted.button(ctx, &format!("vtt.draft-{}", script[n]), self.body) {
                    self.waiting = true;
                }
                if have > n {
                    self.waiting = false;
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
            Phase::Verify => {
                let (checkpoint, sequence) = self.before.as_ref().unwrap();
                let suffix = table.persisted_suffix_after(*sequence).unwrap();
                let replica = FamilyReplica::install(checkpoint.clone(), &suffix, cosmograph::Execution::Grouped).unwrap();
                assert_eq!(replica.snapshot().unwrap(), table.replication_snapshot().unwrap(), "durable replay");
                let state = state.expect("edition state");
                let id = isocosm_native::ruleset::token(self.body.unwrap()).unwrap();
                eprintln!(
                    "[pf2e-selftest] PASS mode={} body={} choices={} sheet={} hp={:?} accepted_records={} durable_replay=exact; last={:?}",
                    if self.play { "play" } else if self.finished { "finished" } else { "draft" },
                    self.body.unwrap(),
                    state.drafts[&id].choices.len(),
                    state.sheets.contains_key(&id),
                    state.sheets.get(&id).and_then(|s| s.int("hp_current")),
                    suffix.len(),
                    state.log.last().map(|l| (&l.note, &l.rolls))
                );
                self.phase = Phase::Capture(0);
            },
            Phase::Capture(frames) => {
                if frames == 0 {
                    let wanted = if self.play { "Condition: " } else if self.finished { "Hit Points: " } else { "Next decision: " };
                    let node = {
                        let dom = ctx.runner.dom();
                        let dom = dom.borrow();
                        dom.all_with_class(dom.document(), "vtt-value").into_iter().find(|node| {
                            dom.dom_children(*node).filter_map(|child| dom.text(child))
                                .collect::<String>().starts_with(wanted)
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
    pub(crate) fn maybe_pathfinder_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(mut test) = self.pathfinder_selftest.take() else { return };
        test.drive(self, ctx);
        self.pathfinder_selftest = Some(test);
    }
}
