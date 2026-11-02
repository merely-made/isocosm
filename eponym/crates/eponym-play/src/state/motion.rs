// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Game-owned admission of one fixed-point motion step.

use crate::identity::{BodyRevisionId, SubjectId, Tick};

use crate::{
    AnatomyError, DeathCause, GameError, GameEvent, MotionInput, MotionRules, MovementError,
    SAFE_FALL,
};

use super::GameState;

impl GameState {
    pub(super) fn advance_motion(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        revision: BodyRevisionId,
        step: u64,
        input: MotionInput,
        rules: MotionRules,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut candidate = self.clone();
        let events = candidate.advance_motion_inner(tick, subject, revision, step, input, rules)?;
        *self = candidate;
        Ok(events)
    }

    /// A round's minute of one body's motion (cases 1117, 1120): each fixed
    /// step re-solved as its own intent would be, the anatomy revision
    /// followed past a landing, ending at the claimed pose. Events keep the
    /// end pose and what a landing did.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn advance_steps(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        revision: BodyRevisionId,
        step: u64,
        runs: &[crate::MotionRun],
        end: crate::MotionPose,
        rules: MotionRules,
    ) -> Result<Vec<GameEvent>, GameError> {
        let inputs = crate::MotionRun::inputs(runs).ok_or(MovementError::MinuteRuns)?;
        let mut candidate = self.clone();
        let (mut events, mut revision, mut step) = (Vec::new(), revision, step);
        for input in inputs {
            if !candidate.bodies.get(subject).is_some_and(|b| b.alive()) {
                return Err(MovementError::MinuteAfterDeath.into());
            }
            let stepped = candidate.advance_motion_inner(tick, subject, revision, step, input, rules)?;
            if stepped.len() > 1 {
                events.extend(stepped.into_iter().filter(|e| !matches!(e, GameEvent::MotionAdvanced { .. })));
                revision = candidate.current_anatomy(subject).map(|a| a.revision).unwrap_or(revision);
            }
            step = step.checked_add(1).ok_or(crate::MotionError::Overflow)?;
        }
        let pose = candidate.movement.pose(subject).ok_or(MovementError::MissingSubject(subject))?;
        if pose != end {
            return Err(MovementError::MinuteEnd.into());
        }
        events.insert(0, GameEvent::MotionAdvanced { tick, subject, pose, landed_distance: None });
        *self = candidate;
        Ok(events)
    }

    /// One fixed step of `subject` from `prior`, solved as an intent would
    /// solve it but recorded nowhere: a host steps its own drawn pose by it
    /// between minutes (case 1118).
    pub fn predict(&self, subject: SubjectId, prior: crate::MotionPose, input: MotionInput,
        rules: MotionRules) -> Result<crate::MotionOutcome, GameError> {
        // Validate the caller's recorded rules before the anatomy projection
        // may reduce their speed to zero. A no-support idle step is valid,
        // but a malformed requested profile is never repaired by projection.
        rules.validate()?;
        let rules = match rules.revision {
            1 => rules,
            crate::MOVEMENT_PROFILE_REVISION => {
                let projection = self
                    .movement_projection_with_speed(subject, rules.speed)?
                    .ok_or(crate::MotionError::InvalidProfile)?;
                let effective = projection.rules_for(rules)?;
                if effective.speed == 0 && (input.move_x != 0 || input.move_z != 0) {
                    return Err(crate::BodyError::Immobile(subject).into());
                }
                effective
            },
            _ => return Err(crate::MotionError::InvalidRules.into()),
        };
        Ok(self.motion.advance(self.world.ground(), prior, input, rules)?)
    }

    fn advance_motion_inner(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        revision: BodyRevisionId,
        step: u64,
        input: MotionInput,
        rules: MotionRules,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.living(subject)?;
        let current = self.current_anatomy(subject)?.revision;
        if revision != current {
            return Err(AnatomyError::StaleRevision {
                known: revision,
                current,
            }
            .into());
        }
        if (input.move_x != 0 || input.move_z != 0)
            && !self.bodies.get(subject).is_some_and(|b| b.mobile())
        {
            return Err(crate::BodyError::Immobile(subject).into());
        }

        let prior = self
            .movement
            .pose(subject)
            .ok_or(MovementError::MissingSubject(subject))?;
        let expected_step = if self.movement.has_contact_pose(subject) {
            prior
                .step
                .checked_add(1)
                .ok_or(crate::MotionError::Overflow)?
        } else {
            1
        };
        if step != expected_step {
            return Err(MovementError::WrongMotionStep {
                previous: prior.step,
                next: step,
            }
            .into());
        }

        let outcome = self.predict(subject, prior, input, rules)?;
        debug_assert_eq!(outcome.pose.step, step);
        self.movement
            .record_contact_pose(&self.world, subject, outcome.pose)?;
        self.actuate_motion(subject, outcome.landed_distance.unwrap_or(0).max(0) as u32)?;
        let mut events = vec![GameEvent::MotionAdvanced {
            tick,
            subject,
            pose: outcome.pose,
            landed_distance: outcome.landed_distance,
        }];

        if let Some(distance) = outcome
            .landed_distance
            .filter(|distance| *distance > SAFE_FALL)
        {
            let harm = distance
                .saturating_sub(SAFE_FALL)
                .saturating_mul(10)
                .clamp(0, 100) as u16;
            let (next_revision, died, reconciled) = self.harm(tick, subject, harm, &[])?;
            events.push(GameEvent::Injured {
                tick,
                subject,
                distance,
                harm,
                revision: next_revision,
            });
            events.extend(reconciled);
            if died {
                events.push(GameEvent::Died {
                    tick,
                    subject,
                    cause: DeathCause::Fall,
                });
            } else {
                events = self.exert(subject, 1, 1, tick, events)?;
            }
        }
        Ok(events)
    }
}
