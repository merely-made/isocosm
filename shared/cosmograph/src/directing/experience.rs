// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Firsthand reasons for a deliberative creature to choose a site.
use super::Aim;
use crate::{
    rules::{Need, Rules},
    schema::{Entity, Id, Method, Tick},
    simulation::Simulation,
};

pub(crate) enum Encounter {
    Meal,
    Hurt,
}

impl Entity {
    pub(crate) fn remember(&mut self, rules: &Rules, place: Id, tick: Tick, what: Encounter) {
        if self.method != Method::Deliberative
            || !self.alive
            || self.resident_site() != Some(place)
            || rules.directing_ref().place_memory.is_none()
        {
            return;
        }
        let places = std::sync::Arc::make_mut(self.places.get_or_insert_with(Default::default));
        match what {
            Encounter::Meal => places.meal = Some((place, tick)),
            Encounter::Hurt => places.hurt = Some((place, tick)),
        }
    }

    /// Retrace a short stretch of the animal's own travels if the food site
    /// is beyond the exits it can see here. Native travel checks each step.
    fn way_back(&self, meal: (Id, Tick)) -> Option<Id> {
        const VISITS: usize = 8;
        let mut next = None;
        for visit in self.visits.iter().rev().take(VISITS) {
            if visit.until < meal.1 {
                break;
            }
            if next.is_none() {
                next = Some(visit.place);
            }
            if visit.place == meal.0 && visit.from <= meal.1 {
                return next;
            }
            if visit.place == self.place {
                next = None;
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
pub(super) struct Preference {
    pub aim: Aim,
    pub site: Id,
    pub weight: u16,
}

impl Preference {
    pub fn vote(self, here: Id, to: Option<Id>) -> i64 {
        let after = to.unwrap_or(here);
        match self.aim {
            Aim::MoveToward => i64::from(here != self.site && after == self.site),
            Aim::MoveAway => i64::from(here == self.site) - i64::from(after == self.site),
            Aim::StayNear if here == self.site => {
                if after == here {
                    1
                } else {
                    -1
                }
            },
            _ => 0,
        }
    }
}

impl Simulation {
    pub(super) fn place_preferences(
        &self,
        actor: Id,
        needs: Option<&[&Need]>,
    ) -> [Option<Preference>; 2] {
        let rules = &self.genesis.rules;
        let Some(memory) = rules.directing_ref().place_memory else {
            return [None; 2];
        };
        let Some(body) = self
            .body_at_start(actor)
            .filter(|body| body.alive && body.method == Method::Deliberative)
        else {
            return [None; 2];
        };
        let (Some(places), Some(here)) = (body.places.as_ref(), body.resident_site()) else {
            return [None; 2];
        };
        let recent = |&(_, tick): &(Id, Tick)| {
            self.state
                .tick
                .checked_sub(tick)
                .is_some_and(|age| age < memory.span)
        };
        let hurt = places
            .hurt
            .filter(recent)
            .filter(|_| memory.hurt > 0)
            .map(|(site, _)| Preference {
                aim: Aim::MoveAway,
                site,
                weight: memory.hurt,
            });
        let meal = places
            .meal
            .filter(recent)
            .filter(|_| memory.meal > 0)
            .and_then(|meal| {
                let hungry = |needs: &[&Need]| {
                    needs.iter().any(|n| {
                        n.weight < 0 && crate::needs::hunger(rules, &n.query)
                    })
                };
                let hungry = needs.map_or_else(|| hungry(&self.choice_minded(actor).1), hungry);
                let (aim, site) = match (hungry, here == meal.0) {
                    (false, true) => (Aim::StayNear, here),
                    (true, false) => {
                        let direct = self
                            .site_at_start(here)?
                            .routes
                            .iter()
                            .any(|r| r.to == meal.0);
                        (
                            Aim::MoveToward,
                            if direct { meal.0 } else { body.way_back(meal)? },
                        )
                    },
                    _ => return None,
                };
                Some(Preference {
                    aim,
                    site,
                    weight: memory.meal,
                })
            });
        [meal, hurt]
    }
}
