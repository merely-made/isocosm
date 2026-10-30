//! The headed H7 table path, driven only through the rendered VTT page.

use super::*;
use cambium_genet_winit_host::ScrollAlign;
use isocosm_host::world::ReplicationSnapshot;
use isonetry::family::{Checkpoint, FamilyReplica};

const FAMILY_SELFTEST_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    WaitingForPage,
    Tabs(usize),
    WaitingForTab(usize),
    WaitingForTable,
    PartyRead,
    WaitingForJoin,
    Join,
    Verify,
    RestorePage,
    WaitForPageTop(u8),
    Done,
}

struct BeforeAction {
    sequence: u64,
    revision: u64,
    native_hash: String,
    vtt_section: Vec<u8>,
    checkpoint: Checkpoint,
}

/// Explicit, opt-in driver for a single headed Bench campaign fixture.
///
/// The driver discovers click targets from the endpoint projection model and
/// then sends ordinary host pointer events at their painted DOM boxes. It does
/// not invoke endpoint actions directly.
pub(crate) struct FamilySelftest {
    phase: Phase,
    started: Option<Instant>,
    tab_ids: Vec<String>,
    before: Option<BeforeAction>,
    scroll_requested: bool,
}

impl FamilySelftest {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var_os("ISOMETRY_H7_FAMILY_SELFTEST").map(|_| Self {
            phase: Phase::WaitingForPage,
            started: None,
            tab_ids: Vec::new(),
            before: None,
            scroll_requested: false,
        })
    }

    pub(crate) fn done(&self) -> bool {
        self.phase == Phase::Done
    }

    pub(crate) fn drive(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        if self.done() {
            return;
        }
        let started = *self.started.get_or_insert_with(Instant::now);
        assert!(
            started.elapsed() < FAMILY_SELFTEST_TIMEOUT,
            "H7 family selftest did not finish; phase={:?}, tabs={:?}, status={:?}",
            self.phase,
            self.tab_ids,
            ctx.runner.state().status
        );

        match self.phase {
            Phase::WaitingForPage => self.begin(app, ctx),
            Phase::Tabs(index) => self.click_tab(ctx, index),
            Phase::WaitingForTab(index) => {
                if ctx.runner.state().vtt_selected_projection == self.tab_ids[index] {
                    self.phase = Phase::Tabs(index + 1);
                }
            },
            Phase::WaitingForTable => {
                let table_id = self
                    .tab_ids
                    .iter()
                    .find(|id| {
                        self.projection_has_action(
                            &ctx.runner.state().vtt_projections,
                            id,
                            "vtt.read-party",
                        )
                    })
                    .expect("the VTT table projection remains available");
                if ctx.runner.state().vtt_selected_projection.as_str() == table_id.as_str() {
                    self.phase = Phase::PartyRead;
                }
            },
            Phase::PartyRead => self.click_action(ctx, "vtt.read-party"),
            Phase::WaitingForJoin => self.wait_for_join(app, ctx),
            Phase::Join => self.click_action(ctx, "vtt.join"),
            Phase::Verify => self.verify(app, ctx),
            Phase::RestorePage => self.restore_page_top(ctx),
            Phase::WaitForPageTop(frames) => {
                self.phase = if frames >= 2 {
                    Phase::Done
                } else {
                    Phase::WaitForPageTop(frames + 1)
                };
            },
            Phase::Done => {},
        }
    }

    fn begin(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        let ui = ctx.runner.state();
        if ui.vtt_projections.len() != 7 {
            return;
        }
        let tabs = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), "vtt-tab")
        };
        if tabs.len() != 7 {
            return;
        }
        let table = app
            .family_table
            .as_mut()
            .expect("the family selftest requires the opened native table");
        let snapshot = table
            .replication_snapshot()
            .expect("initial family snapshot is readable");
        let sequence = table.sequence();
        // Opening the endpoint may commit its initial table-window record.
        // Drain that startup prefix before measuring the user's displayed
        // actions; it remains in the durable ledger and is outside this cut.
        let startup_records = table.drain_records();
        let checkpoint = table.checkpoint().expect("initial family checkpoint");
        let (native_hash, vtt_section) = saved_parts(&snapshot);
        self.before = Some(BeforeAction {
            sequence,
            revision: snapshot.revision,
            native_hash,
            vtt_section,
            checkpoint,
        });
        self.tab_ids = ui
            .vtt_projections
            .iter()
            .map(|projection| projection.id.clone())
            .collect();
        eprintln!(
            "[h7-family-selftest] disclosed endpoint tabs: {:?}; startup_sequence={} startup_records_drained={}",
            ui.vtt_projections
                .iter()
                .map(|projection| projection.label.as_str())
                .collect::<Vec<_>>(),
            sequence,
            startup_records.len()
        );
        self.phase = Phase::Tabs(0);
    }

    fn click_tab(&mut self, ctx: &mut Ctx<'_>, index: usize) {
        if index == self.tab_ids.len() {
            let table_id = ctx
                .runner
                .state()
                .vtt_projections
                .iter()
                .find(|projection| {
                    projection
                        .cards
                        .iter()
                        .flat_map(|card| &card.actions)
                        .any(|action| action.id == "vtt.read-party")
                })
                .map(|projection| projection.id.clone())
                .expect("the table projection advertises party selection");
            if ctx.runner.state().vtt_selected_projection == table_id {
                self.phase = Phase::PartyRead;
            } else {
                self.click_tab_id(ctx, &table_id);
                self.phase = Phase::WaitingForTable;
            }
            return;
        }
        if index > self.tab_ids.len() {
            self.phase = Phase::PartyRead;
            return;
        }
        let id = self.tab_ids[index].clone();
        if ctx.runner.state().vtt_selected_projection == id {
            self.phase = Phase::Tabs(index + 1);
            return;
        }
        self.click_tab_id(ctx, &id);
    }

    fn click_tab_id(&mut self, ctx: &mut Ctx<'_>, id: &str) {
        let Some(index) = self.tab_ids.iter().position(|candidate| candidate == id) else {
            panic!("the table projection is absent from the displayed tabs: {id}");
        };
        let node = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), "vtt-tab")
                .get(index)
                .copied()
        }
        .expect("each endpoint projection has a rendered tab");
        let rect = ctx
            .painted_rect(node)
            .expect("each endpoint projection tab has a painted box");
        queue_click_rect(ctx, rect);
        self.phase = Phase::WaitingForTab(index);
    }

    fn click_action(&mut self, ctx: &mut Ctx<'_>, action_id: &str) {
        let selected = ctx.runner.state().vtt_selected_projection.clone();
        let action = ctx
            .runner
            .state()
            .vtt_projections
            .iter()
            .find(|projection| projection.id == selected)
            .and_then(|projection| {
                projection.cards.iter().find_map(|card| {
                    card.actions
                        .iter()
                        .find(|action| action.id == action_id)
                        .map(|action| action.label.clone())
                })
            });
        let Some(label) = action else {
            if action_id == "vtt.join" && self.phase == Phase::Join {
                self.phase = Phase::WaitingForJoin;
            }
            return;
        };
        let node = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), "vtt-action")
                .into_iter()
                .find(|&node| {
                    dom.dom_children(node)
                        .filter_map(|child| dom.text(child))
                        .collect::<String>()
                        == label
                })
        }
        .expect("the currently advertised action is present in the rendered page");
        if !self.scroll_requested {
            ctx.scroll_into_view(node, ScrollAlign::Nearest);
            self.scroll_requested = true;
            return;
        }
        let rect = ctx
            .painted_rect(node)
            .expect("the selected VTT action has a painted box after scrolling");
        queue_click_rect(ctx, rect);
        self.scroll_requested = false;
        self.phase = if action_id == "vtt.read-party" {
            Phase::WaitingForJoin
        } else {
            Phase::Verify
        };
    }

    fn projection_has_action(
        &self,
        projections: &[isometry_views::VttProjectionView],
        projection_id: &str,
        action_id: &str,
    ) -> bool {
        projections
            .iter()
            .find(|projection| projection.id == projection_id)
            .is_some_and(|projection| {
                projection
                    .cards
                    .iter()
                    .flat_map(|card| &card.actions)
                    .any(|action| action.id == action_id)
            })
    }

    fn wait_for_join(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        let before = self.before.as_ref().expect("baseline captured");
        let table = app
            .family_table
            .as_ref()
            .expect("family table remains open");
        let current = table
            .replication_snapshot()
            .expect("family snapshot remains readable");
        let (native_hash, vtt_section) = saved_parts(&current);
        assert_eq!(table.sequence(), before.sequence);
        assert_eq!(current.revision, before.revision);
        assert_eq!(native_hash, before.native_hash, "party selection is local");
        assert_eq!(
            vtt_section, before.vtt_section,
            "party selection is record-free"
        );
        assert!(table.drain_records().is_empty());

        let has_join = ctx
            .runner
            .state()
            .vtt_projections
            .iter()
            .find(|projection| projection.id == ctx.runner.state().vtt_selected_projection)
            .is_some_and(|projection| {
                projection
                    .cards
                    .iter()
                    .flat_map(|card| &card.actions)
                    .any(|action| action.id == "vtt.join")
            });
        if has_join {
            self.phase = Phase::Join;
        }
    }

    fn verify(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        let before = self.before.as_ref().expect("baseline captured");
        let table = app
            .family_table
            .as_mut()
            .expect("family table remains open");
        let current = table
            .replication_snapshot()
            .expect("family snapshot remains readable");
        if table.sequence() == before.sequence {
            return;
        }
        assert_eq!(table.sequence(), before.sequence + 1);
        assert_eq!(current.revision, before.revision + 1);
        let (native_hash, vtt_section) = saved_parts(&current);
        assert_ne!(
            native_hash, before.native_hash,
            "the join changes native history"
        );
        assert_eq!(
            vtt_section, before.vtt_section,
            "native party membership does not rewrite VTT character data"
        );

        let records = table.drain_records();
        assert_eq!(
            records.len(),
            1,
            "the displayed join emits one accepted record"
        );
        let record = &records[0];
        assert_eq!(record.sequence, before.sequence + 1);
        assert_eq!(record.revision, current.revision);
        assert!(
            !record.native_suffix.is_empty(),
            "join records its native party event"
        );
        assert!(!record.section_updates.contains_key("vtt"));
        assert!(
            table.is_durable(),
            "the headed campaign owns a saved family ledger"
        );
        let suffix = table
            .persisted_suffix_after(before.sequence)
            .expect("the accepted record is present in the saved family ledger");
        assert_eq!(
            suffix, records,
            "saved ledger suffix matches the accepted outbox"
        );
        let replica = FamilyReplica::install(
            before.checkpoint.clone(),
            &suffix,
            isocosm::Execution::Grouped,
        )
        .expect("the saved accepted suffix replays");
        let replayed = replica.snapshot().expect("replayed full world snapshot");
        assert_eq!(
            replayed, current,
            "native state, opaque sections and revision replay exactly"
        );

        let final_native_hash = native_hash;
        let final_revision = current.revision;
        let final_sequence = table.sequence();
        ctx.runner.update(|ui| {
            ui.status = "Bench table action accepted; saved family replay verified".into();
        });
        self.phase = Phase::RestorePage;
        eprintln!(
            "[h7-family-selftest] PASS sequence={} revision={} native_before={} native_after={} vtt_section_changed=false accepted_records=1 native_suffix_entries={} durable_replay=exact",
            final_sequence,
            final_revision,
            before.native_hash,
            final_native_hash,
            record.native_suffix.len(),
        );
    }

    fn restore_page_top(&mut self, ctx: &mut Ctx<'_>) {
        let node = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), "vtt-tab")
                .first()
                .copied()
        }
        .expect("the captured VTT page still has its endpoint tabs");
        ctx.scroll_into_view(node, ScrollAlign::Start);
        self.phase = Phase::WaitForPageTop(0);
    }
}

impl App {
    pub(crate) fn maybe_family_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(mut test) = self.family_selftest.take() else {
            return;
        };
        test.drive(self, ctx);
        self.family_selftest = Some(test);
    }
}

fn saved_parts(snapshot: &ReplicationSnapshot) -> (String, Vec<u8>) {
    let native = isocosm::digest(&snapshot.save.native);
    let section = snapshot
        .save
        .sections
        .get("vtt")
        .expect("the native table has its opaque VTT section")
        .clone();
    (native, section)
}

fn queue_click_rect(ctx: &mut Ctx<'_>, (x, y, width, height): (f32, f32, f32, f32)) {
    let (x, y) = (x + width / 2.0, y + height / 2.0);
    ctx.pointer.push(HostPointer::Moved(x, y));
    ctx.pointer.push(HostPointer::Press(x, y));
    ctx.pointer.push(HostPointer::Release(x, y));
}
