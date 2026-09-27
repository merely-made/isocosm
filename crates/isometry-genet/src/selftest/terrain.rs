//! Bounded headed receipt: real panel pointer dispatch and host persistence.
//! The driver seeds a 1 MiB preference externally, then starts a second host
//! with `verify` to prove its saved 2 MiB value survives restart.

use super::*;

pub(crate) struct TerrainReceipt {
    verify: bool,
    stage: u8,
}

impl TerrainReceipt {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var("ISOMETRY_TERRAIN_SELFTEST")
            .ok()
            .map(|value| Self {
                verify: value == "verify",
                stage: 0,
            })
    }

    pub(crate) fn done(&self) -> bool {
        self.stage == 3
    }

    pub(crate) fn drive(&mut self, ctx: &mut Ctx<'_>, started: Option<Instant>) {
        if self.done()
            || !started
                .is_some_and(|time| time.elapsed() > Duration::from_secs(2 + u64::from(self.stage)))
        {
            return;
        }
        let settings = &ctx.runner.state().terrain_settings;
        assert!(
            settings.device_budget.is_some(),
            "headed receipt requires an acquired GPU"
        );
        let expected = if self.verify || self.stage == 2 { 2 } else { 1 };
        assert_eq!(
            settings.budget_mib, expected,
            "normal panel callback / preference load"
        );
        if expected == 1 {
            assert!(settings.omitted > 0, "same-run overflow positive control");
        } else {
            assert_eq!(
                settings.omitted, 0,
                "current omission count clears after growth of chosen budget"
            );
        }
        let (buttons, labels) = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            let panel = dom.all_with_class(dom.document(), "terrain-settings");
            assert_eq!(panel.len(), 1);
            let labels: Vec<String> = dom
                .all_with_class(panel[0], "side-line")
                .into_iter()
                .flat_map(|node| {
                    dom.dom_children(node)
                        .filter_map(|child| dom.text(child).map(str::to_owned))
                        .collect::<Vec<_>>()
                })
                .collect();
            (dom.all_with_class(panel[0], "btn"), labels)
        };
        assert!(labels.contains(&format!("Terrain memory: {expected} MiB")));
        assert!(labels.contains(&format!("{} terrain bricks omitted", settings.omitted)));
        eprintln!(
            "[terrain-host-receipt] stage={} verify={} requested={} device_bytes={} omitted={} labels={labels:?}",
            self.stage,
            self.verify,
            settings.budget_mib,
            settings.device_budget.unwrap(),
            settings.omitted
        );
        if self.verify || self.stage == 2 {
            self.stage = 3;
            eprintln!("[terrain-host-receipt] PASS");
            return;
        }
        assert_eq!(buttons.len(), 2);
        // First the disabled minimum: an actual click must leave 1 MiB alone.
        // Then the enabled increment must change it, persist, and rebuild.
        let (x, y, width, height) = ctx
            .painted_rect(buttons[self.stage as usize])
            .expect("terrain memory control must have a painted box");
        let (x, y) = (x + width / 2.0, y + height / 2.0);
        ctx.pointer.push(HostPointer::Moved(x, y));
        ctx.pointer.push(HostPointer::Press(x, y));
        ctx.pointer.push(HostPointer::Release(x, y));
        eprintln!("[terrain-host-receipt] queued real pointer click at {x},{y}");
        self.stage += 1;
    }
}
