// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Harm handed to the sim as wounds (wing rulings 669, 707 to 711 and 767):
//! a whole-body harm out of a hundred becomes that share of the body's
//! living cells lost from a part drawn by its cells, and a part the game's
//! strike cut is wounded through every cell, which severs it (711). A
//! severing moves the body's revision and reconciles its admitted anatomy;
//! a wound alone does not (717).

use isocosm_overlay::eponym::{Blow, EponymHandoff};
use isocosm_overlay::{ActKey, EntityHandle, Harm, PartHandle, Wound, WoundSeverity};
use isometer_core::PartId;

use crate::identity::{BodyRevisionId, SubjectId, Tick};
use crate::{GameError, GameEvent};

use super::GameState;

#[cfg(test)]
mod native_source_tests;

impl GameState {
    /// Wounds `subject` by `harm` and through `severed`; returns its
    /// revision after, whether it died, and the events of any severing.
    pub(super) fn harm(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        harm: u16,
        severed: &[PartId],
    ) -> Result<(BodyRevisionId, bool, Vec<GameEvent>), GameError> {
        let injury = self.injury(subject, harm, severed)?;
        self.receive_harm(tick, subject, EponymHandoff::Hurt(injury))
    }

    pub(super) fn blow(
        &mut self,
        tick: Tick,
        actor: SubjectId,
        target: SubjectId,
        harm: u16,
        severed: &[PartId],
    ) -> Result<(BodyRevisionId, bool, Vec<GameEvent>), GameError> {
        let blow = Blow {
            by: EntityHandle(self.entity_of(actor)?),
            target: EntityHandle(self.entity_of(target)?),
            act: ActKey("eponym:strike".into()),
            harm: self.injury(target, harm, severed)?,
        };
        self.receive_harm(tick, target, EponymHandoff::Blow(blow))
    }

    fn injury(&self, subject: SubjectId, harm: u16, severed: &[PartId]) -> Result<Harm, GameError> {
        let entity = self.entity_of(subject)?;
        Ok(Harm {
            // The current native calibration is cells, preserving 767's lowering.
            vigour_drained: if severed.is_empty() {
                u64::from(self.cells_for(entity, harm))
            } else {
                0
            },
            wounds: severed
                .iter()
                .map(|p| Wound {
                    part: PartHandle(u64::from(p.0)),
                    severity: WoundSeverity::Severed,
                })
                .collect(),
        })
    }

    fn receive_harm(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        handoff: EponymHandoff,
    ) -> Result<(BodyRevisionId, bool, Vec<GameEvent>), GameError> {
        let entity = self.entity_of(subject)?;
        let lowered = self.world.hand_back(entity, &handoff)?;
        let (mut cut, mut died) = (vec![], false);
        for w in lowered.wounded {
            cut.extend(w.severed.iter().copied());
            died |= w.died;
        }
        let from = self.records[&subject].revision;
        let mut events = vec![];
        let revision = match cut.is_empty() {
            true => from,
            false => {
                let to = BodyRevisionId(from.0 + 1);
                self.records.get_mut(&subject).expect("recorded").revision = to;
                events = self.reconcile(tick, subject, from, to, &cut)?;
                to
            },
        };
        if died {
            self.records.get_mut(&subject).expect("recorded").died_at = Some(tick);
        }
        Ok((revision, died, events))
    }

    /// The cells a whole-body harm out of a hundred takes, at least one.
    fn cells_for(&self, entity: isocosm::schema::Id, harm: u16) -> u32 {
        if harm == 0 {
            return 0;
        }
        let Some(e) = self.world.sim().state().population.get(entity) else {
            return 0;
        };
        let living: u64 = e
            .living()
            .map(|(id, p)| u64::from(isocosm::anatomy::living_cells(e.extent(id), p)))
            .sum();
        (living * u64::from(harm)).div_ceil(100).max(1) as u32
    }

    /// Reconciles `subject`'s admitted anatomy to `revision`, the parts in
    /// `cut` severed, and lets fall what was worn on them.
    pub(super) fn reconcile(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
        cut: &[PartId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let at = self.at(subject)?;
        let mut events = vec![];
        if self
            .anatomies
            .get(subject)
            .is_some_and(|r| r.revision == from_revision)
        {
            let fresh: Vec<PartId> = cut
                .iter()
                .copied()
                .filter(|p| {
                    self.anatomies
                        .get(subject)
                        .and_then(|r| r.document.part(*p))
                        .is_some_and(|q| !q.severed)
                })
                .collect();
            self.anatomies
                .reconcile(subject, from_revision, revision, &fresh)?;
            if self.world.developed_bodies() {
                self.refresh_native_anatomy(subject)?;
            }
            events.push(GameEvent::AnatomyReconciled {
                tick,
                subject,
                from_revision,
                revision,
            });
        }
        self.records.get_mut(&subject).expect("recorded").revision = revision;
        let lost: Vec<PartId> = self
            .anatomies
            .get(subject)
            .map(|r| {
                r.document
                    .parts
                    .iter()
                    .filter(|p| p.severed)
                    .map(|p| p.id)
                    .collect()
            })
            .unwrap_or_default();
        let fallen = self.release_worn(subject, &lost, at)?;
        events.extend(fallen.into_iter().map(|item| GameEvent::ItemReleased {
            tick,
            subject,
            item,
            at,
        }));
        Ok(events)
    }
}
