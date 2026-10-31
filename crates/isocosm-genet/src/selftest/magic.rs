//! Opt-in headed receipt. Every choice goes through painted endpoint buttons.
use super::*;
use super::painted::Painted;
use cambium_genet_winit_host::ScrollAlign;
use isocosm_native::{
    SHEETS, TABLE,
    ruleset::{Action, State},
};
use isocosm_system::active_effects::{End, Records};
use isonetry::family::{Checkpoint, FamilyReplica};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Select,
    Cast,
    WaitCast,
    Attack,
    WaitAttack,
    Damage,
    WaitDamage,
    Again,
    WaitAgain,
    Propose,
    WaitPropose,
    TimePage,
    Advance,
    WaitExpiry,
    Verify,
    Capture(u8),
    Done,
}

pub(crate) struct MagicSelftest {
    mode: String,
    phase: Phase,
    started: Instant,
    painted: Painted,
    before: Option<(Checkpoint, u64)>,
    caster: u64,
    target: u64,
    wait_log: usize,
    wait_revision: u64,
    damage_checks: u8,
}

impl MagicSelftest {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var("ISOMETRY_MAGIC_WEAPON_SELFTEST")
            .ok()
            .map(|mode| {
                assert!(
                    ["active", "lost", "expired"].contains(&mode.as_str()),
                    "magic selftest mode must be active, lost or expired"
                );
                Self {
                    mode,
                    phase: Phase::Select,
                    started: Instant::now(),
                    painted: Painted::default(),
                    before: None,
                    caster: 0,
                    target: 0,
                    wait_log: 0,
                    wait_revision: 0,
                    damage_checks: 0,
                }
            })
    }
    pub(crate) fn done(&self) -> bool {
        self.phase == Phase::Done
    }
    pub(crate) fn drive(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        if self.done() {
            return;
        }
        assert!(
            self.started.elapsed() < Duration::from_secs(180),
            "Magic Weapon selftest stalled: {:?}; {}",
            self.phase,
            ctx.runner.state().status
        );
        let Some(table) = app.family_table.as_mut() else {
            return;
        };
        let Some(state) = table.edition_state().expect("edition state readable") else {
            return;
        };
        if self.before.is_none() {
            let ids: Vec<_> = state.sheets.keys().map(|id| u64::from(id.0)).collect();
            assert_eq!(ids.len(), 2, "use the two-character Bench fixture");
            self.caster = ids[0];
            self.target = ids[1];
            table.drain_records();
            self.before = Some((table.checkpoint().unwrap(), table.sequence()));
        }
        match self.phase {
            Phase::Select => {
                if self.painted.tab(ctx, SHEETS) {
                    self.phase = Phase::Cast;
                }
            },
            Phase::Cast => {
                if self.painted.button(ctx, "vtt.weapon-cast-2", None) {
                    self.phase = Phase::WaitCast;
                }
            },
            Phase::WaitCast => {
                if Self::effects(&state).effects.contains_key(&1) {
                    self.phase = Phase::Attack;
                }
            },
            Phase::Attack => {
                if self.painted.button(
                    ctx,
                    &format!("vtt.weapon-rule-attack-{}", self.target),
                    None,
                ) {
                    self.phase = Phase::WaitAttack;
                }
            },
            Phase::WaitAttack => {
                if state
                    .log
                    .last()
                    .is_some_and(|log| matches!(log.action, Action::RuleAttack { .. }))
                {
                    self.phase = if self.mode == "active" {
                        Phase::Verify
                    } else {
                        Phase::Damage
                    };
                }
            },
            Phase::Damage => {
                self.wait_log = state.log.len();
                if self.painted.button(ctx, "vtt.sheet-damage-1", Some(self.caster)) {
                    self.damage_checks += 1;
                    self.phase = Phase::WaitDamage;
                }
            },
            Phase::WaitDamage => {
                if state.log.len() > self.wait_log {
                    if Self::effects(&state).effects[&1].ended.is_some() {
                        self.phase = if self.mode == "lost" {
                            Phase::Verify
                        } else {
                            Phase::Again
                        };
                    } else {
                        assert!(
                            self.damage_checks < 8,
                            "Bench caster did not fail concentration in eight hits"
                        );
                        self.phase = Phase::Damage;
                    }
                }
            },
            Phase::Again => {
                if self.painted.button(ctx, "vtt.weapon-cast-2", None) {
                    self.phase = Phase::WaitAgain;
                }
            },
            Phase::WaitAgain => {
                if Self::effects(&state).effects.contains_key(&2) {
                    self.phase = Phase::Propose;
                }
            },
            Phase::Propose => {
                self.wait_revision = table.revision().unwrap();
                if self.painted.button(ctx, "vtt.effect-propose-expiry", None) {
                    self.phase = Phase::WaitPropose;
                }
            },
            Phase::WaitPropose => {
                if table.revision().unwrap() > self.wait_revision {
                    self.phase = Phase::TimePage;
                }
            },
            Phase::TimePage => {
                if self.painted.tab(ctx, TABLE) {
                    self.phase = Phase::Advance;
                }
            },
            Phase::Advance => {
                if self.painted.button(ctx, "vtt.advance", None) {
                    self.phase = Phase::WaitExpiry;
                }
            },
            Phase::WaitExpiry => {
                if Self::effects(&state).effects[&2]
                    .ended
                    .as_ref()
                    .is_some_and(|(_, end)| *end == End::Expired)
                {
                    self.phase = Phase::Verify;
                }
            },
            Phase::Verify => {
                if !self.painted.tab(ctx, SHEETS) {
                    return;
                }
                self.verify(app, ctx, &state);
                self.phase = Phase::Capture(0);
            },
            Phase::Capture(frames) => {
                if frames == 0 {
                    let node = {
                        let dom = ctx.runner.dom();
                        let dom = dom.borrow();
                        dom.all_with_class(dom.document(), "vtt-card-title")
                            .into_iter()
                            .filter(|node| {
                                dom.dom_children(*node)
                                    .filter_map(|child| dom.text(child))
                                    .collect::<String>()
                                    == "Magic Weapon"
                            })
                            .last()
                    };
                    if let Some(node) = node {
                        ctx.scroll_into_view(node, ScrollAlign::Start);
                    }
                }
                self.phase = if frames >= 3 {
                    Phase::Done
                } else {
                    Phase::Capture(frames + 1)
                };
            },
            Phase::Done => {},
        }
    }
    fn effects(state: &State) -> Records {
        Records::read(&state.map())
            .unwrap()
            .expect("cast ledger exists")
    }
    fn verify(&self, app: &mut App, ctx: &mut Ctx<'_>, state: &State) {
        let records = Self::effects(state);
        let id = isocosm_native::ruleset::token(self.caster).unwrap();
        let slots = state.sheets[&id].int("slot:2").unwrap();
        assert_eq!(slots, if self.mode == "expired" { 0 } else { 1 });
        let attack = state
            .log
            .iter()
            .find(|log| matches!(log.action, Action::RuleAttack { .. }))
            .unwrap();
        assert_eq!(
            attack.rolls[0].total,
            i32::from(attack.rolls[0].dice[0]) + 3,
            "enhancement consumed by the existing attack resolver"
        );
        match self.mode.as_str() {
            "active" => assert!(records.effects[&1].applies(records.tick)),
            "lost" => assert!(matches!(
                records.effects[&1].ended,
                Some((_, End::Concentration | End::Incapacitated))
            )),
            "expired" => assert_eq!(records.effects[&2].ended, Some((60, End::Expired))),
            _ => unreachable!(),
        }
        let table = app.family_table.as_ref().unwrap();
        let (checkpoint, sequence) = self.before.as_ref().unwrap();
        let suffix = table.persisted_suffix_after(*sequence).unwrap();
        let replica =
            FamilyReplica::install(checkpoint.clone(), &suffix, cosmograph::Execution::Grouped)
                .unwrap();
        assert_eq!(
            replica.snapshot().unwrap(),
            table.replication_snapshot().unwrap(),
            "durable native and edition replay"
        );
        eprintln!(
            "[magic-weapon-selftest] PASS mode={} slots={} native_tick={} effects={} damage_checks={} accepted_records={} durable_replay=exact",
            self.mode,
            slots,
            records.tick,
            records.effects.len(),
            self.damage_checks,
            suffix.len()
        );
        ctx.runner.update(|ui| {
            ui.status = format!("Magic Weapon {}: saved family replay verified", self.mode)
        });
    }
}

impl App {
    pub(crate) fn maybe_magic_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(mut test) = self.magic_selftest.take() else {
            return;
        };
        test.drive(self, ctx);
        self.magic_selftest = Some(test);
    }
}
