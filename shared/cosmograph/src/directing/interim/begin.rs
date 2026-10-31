// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Enter an admitted, already-evolved world without founding it a second time.

use super::{Interim, Mode, Pace, Start, OnCollapse, deep::{Handover, run_deep_time_using}, readings};
use crate::{Result, Session, Command, rules::DeepTimeSpan};

impl Interim {
    /// Consumes the assessment's actual native owner. Saved controllers use
    /// resume_as instead; the declared lifecycle and new control run once here.
    pub fn begin_using(mut session: Session, start: Start, mode: Mode, pace: Pace,
        advance: &mut dyn FnMut(&mut Session) -> Result<()>) -> Result<Self> {
        if pace.round == 0 || pace.scoring == 0 {
            return Err("begin requires explicitly positive round and scoring ticks".into());
        }
        if session.sim.state().population.groups.values().any(|group|
            super::super::is_participant(&group.entity)) {
            return Err("an existing controller must be resumed, not begun again".into());
        }
        let played = session.sim.genesis().founding.as_ref().and_then(|founding| founding.played)
            .ok_or("the interim plays a generated lineage (682)")?;
        let lineage = format!("lineage:{}", played.lineage);
        let span = session.sim.genesis().rules.deep_time;
        let first = run_deep_time_using(&mut session, span, advance)?;
        let mut waited = 0u32;
        while readings::habitable(&session.sim, &lineage).is_empty() {
            if waited == start.within {
                return Err(format!("{lineage} found no habitable site within {waited} epochs"));
            }
            run_deep_time_using(&mut session, DeepTimeSpan { epochs: 1 }, advance)?;
            waited += 1;
        }
        let habitable_at = session.sim.state().tick;
        let last = run_deep_time_using(&mut session, DeepTimeSpan { epochs: start.epochs }, advance)?;
        let epochs = span.epochs.checked_add(waited).and_then(|epochs| epochs.checked_add(start.epochs))
            .ok_or("begin lifecycle epoch count overflow")?;
        let handover = Handover { span: hagiograph::DeepTime { epochs },
            to_epoch: last.to_epoch, to_tick: last.to_tick, ..first };
        if session.sim.state().population.groups.values().any(|group|
            super::super::is_participant(&group.entity)) {
            return Err("begin lifecycle created an existing controller".into());
        }
        // The actual admitted receipt supplies the controller's native identity.
        let joined = session.command(Command::Join)?;
        let participant = joined.strip_prefix("participant:").and_then(|id| id.parse().ok())
            .ok_or("a join names its participant")?;
        let mut interim = Self { session, participant, mode, pace,
            on_collapse: OnCollapse::Stay, handover, habitable_at };
        let critter = interim.first_life(&lineage).ok_or("the played lineage has no living member")?;
        interim.take(critter)?;
        Ok(interim)
    }
}
