// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Why a Choice process is not offered to a critter now (case 1233): the
//! gates `consider` weighs it through, each with the reason it gives. The
//! chooser reads them as a yes or no, so the two can never disagree.

use super::choice::hunts;
use crate::{
    rules::*,
    schema::*,
    simulation::Simulation,
};
use std::borrow::Cow;

impl Simulation {
    /// The reason `p` is not feasible for `actor` at `place` on `target`,
    /// in the order its gates are read; none where it is. Its text is what
    /// the act path says where it has any (case 1233).
    pub(crate) fn infeasible(
        &self,
        actor: Id,
        target: Option<Id>,
        p: &Process,
        place: Id,
    ) -> Option<Cow<'static, str>> {
        if !self.route_ready(actor, p) {
            return Some("route actor is not resident or route is absent".into());
        }
        if let Err(why) = self.direction_ready(actor, p) {
            return Some(why.into());
        }
        if !self.target_matches(actor, target, p) {
            return Some("no target satisfies the declared scope".into());
        }
        if !self.foraging_walk_matches(actor, target, p) {
            return Some("walking has no eligible food serving a current need".into());
        }
        if !self.owned_process_ready(actor, target, p) {
            return Some("an owned stock the act draws on is not ready".into());
        }
        let part = self.bind_part(actor, p);
        for q in &p.requires {
            if let Err(why) = self.process_query(actor, target, place, part, p, q) {
                // The act binds its intake organ before it reads a requirement.
                return Some(self.bind_intake(actor, p).err().unwrap_or(why).into());
            }
        }
        None
    }

    /// Whether `p` is a Choice process `actor` has no need of: its need
    /// account is already met.
    pub(crate) fn satisfied(&self, actor: Id, p: &Process) -> bool {
        let rules = &self.genesis.rules;
        p.need_account.as_ref().is_some_and(|a| {
            let have = self.body_at_start(actor).map_or(0, |e| crate::anatomy::held(e, rules, a));
            have >= p.need_below
        })
    }

    /// The target `actor` takes `p` on, and the one a live nudge names where
    /// one does: a nudge to act on a thing names it, where it can be.
    pub(crate) fn aim(&self, actor: Id, p: &Process, live: &[usize], place: Id) -> (Option<Id>, Option<Id>) {
        let nudged = live.iter().find_map(|&i| {
            let n = &self.state.nudges[i];
            if hunts(p) && n.act.as_ref().is_some_and(|act| act != &p.id) {
                return None;
            }
            match n {
                super::Nudge {
                    aim: super::Aim::Act,
                    toward: super::Toward::Thing(t),
                    ..
                } if p.target.is_some() && self.infeasible(actor, Some(*t), p, place).is_none() => Some(*t),
                _ => None,
            }
        });
        let target = match p.target {
            Some(_) => nudged.or_else(|| self.choose_target(actor, p)),
            None => None,
        };
        (nudged, target)
    }
}
