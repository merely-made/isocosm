// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Due native birth opportunities, read before filing removes closed
//! gates. Port admission is read without the feed's hunger requirements.

use super::*;

fn traits(e: &Entity, p: &Process) -> bool {
    p.requires
        .iter()
        .filter_map(|q| match q {
            Query::Trait {
                who: Binding::Actor,
                key,
            } => Some(key),
            _ => None,
        })
        .all(|key| e.traits.contains(key))
}

impl Capture {
    fn birth(&mut self, r: BirthOpportunity) {
        self.through = self.through.max(r.tick);
        let bucket = self.bucket(r.tick);
        let key = (bucket, r.process.clone(), r.lineage.clone());
        let t = self.breeding.entry(key).or_insert_with(|| BirthTally {
            bucket,
            first_tick: r.tick,
            last_tick: r.tick,
            process: r.process.clone(),
            lineage: r.lineage.clone(),
            opportunities: 0,
            ready: 0,
            age_ready: 0,
            tissue_ready: 0,
            role_gate_ready: 0,
            port_gate_ready: 0,
            food_gate_ready: 0,
            age: Range::default(),
            tissue: Range::default(),
            role_members: Range::default(),
            port_members: Range::default(),
            food_members: Range::default(),
            edible_matter: Range::default(),
            consumer_members: Range::default(),
        });
        let first = t.opportunities == 0;
        t.opportunities += r.count;
        t.last_tick = r.tick;
        if r.ready {
            t.ready += r.count;
        }
        if r.age >= r.age_required {
            t.age_ready += r.count;
        }
        if r.tissue_held >= r.tissue_required {
            t.tissue_ready += r.count;
        }
        let passes = |members: u64| {
            r.consumer_members > 0
                && r.per_members > 0
                && u128::from(members) * u128::from(r.per_members)
                    >= u128::from(r.consumer_members) * u128::from(r.at_least)
        };
        if r.per_members == 0 || passes(r.role_members) {
            t.role_gate_ready += r.count;
        }
        if r.per_members == 0 || passes(r.port_members) {
            t.port_gate_ready += r.count;
        }
        if r.food_gate_ready == Some(true) {
            t.food_gate_ready += r.count;
        }
        t.age.include(r.age, first);
        t.tissue.include(r.tissue_held, first);
        t.role_members.include(r.role_members, first);
        t.port_members.include(r.port_members, first);
        if let Some(food) = r.food_members {
            t.food_members.include(food, first);
        }
        if let Some(matter) = r.edible_matter {
            t.edible_matter.include(matter, first);
        }
        t.consumer_members.include(r.consumer_members, first);
        let seen = self
            .birth_seen
            .entry((r.lineage.clone(), r.process.clone(), r.ready))
            .or_default();
        if self.config.focus_lineages.contains(&r.lineage) && *seen < self.config.examples_per_kind
        {
            self.birth_examples.push(r);
            *seen += 1;
        }
    }
}

impl Simulation {
    pub(crate) fn diagnostic_birth_opportunities(&self, p: &Process, due: impl Fn(Tick) -> bool) {
        if self.watch.diagnostic.is_none() || !p.id.starts_with("ecology:birth-") {
            return;
        }
        let rules = &self.genesis.rules;
        for (&id, group) in &self.state.population.groups {
            let e = &group.entity;
            if !e.alive || e.method == Method::Inert || !due(e.born) || !traits(e, p) {
                continue;
            }
            let site = e.resident_site();
            let mut r = BirthOpportunity {
                tick: self.state.tick,
                process: p.id.clone(),
                actor: id,
                lineage: e.lineage.clone(),
                count: group.count,
                site,
                age: self.state.tick.saturating_sub(e.born),
                age_required: 0,
                tissue: None,
                tissue_held: 0,
                tissue_required: 0,
                role_members: 0,
                port_members: 0,
                food_members: None,
                food_gate_ready: None,
                edible_matter: None,
                consumer_members: 0,
                at_least: 0,
                per_members: 0,
                feed_process: None,
                ready: true,
                requirements: vec![],
            };
            if let Some(founding) = &self.genesis.founding
                && let Some(index) =
                    p.id.strip_prefix("ecology:birth-")
                        .and_then(|i| i.parse::<u32>().ok())
            {
                let role = crate::ecology::Role::of(founding.web, index);
                let among = (0..founding.lineages)
                    .filter(|i| role.eats(crate::ecology::Role::of(founding.web, *i)))
                    .map(|i| format!("lineage:{i}"))
                    .collect();
                r.role_members = site.map_or(0, |s| self.members_at(s, &among));
            }
            for q in &p.requires {
                match q {
                    Query::Age { at_least } => r.age_required = r.age_required.max(*at_least),
                    Query::Account {
                        who: Binding::Actor,
                        key,
                        at_least,
                    } => {
                        r.tissue = Some(key.clone());
                        r.tissue_held = crate::anatomy::held(e, rules, key);
                        r.tissue_required = *at_least;
                    },
                    Query::MembersPerHead {
                        among,
                        per,
                        at_least,
                        per_members,
                    } => {
                        let own = BTreeSet::from([e.lineage.clone()]);
                        r.role_members = site.map_or(0, |s| self.members_at(s, among));
                        r.consumer_members =
                            site.map_or(0, |s| self.members_at(s, per.as_ref().unwrap_or(&own)));
                        r.at_least = *at_least;
                        r.per_members = *per_members;
                    },
                    Query::FoodPerHead {
                        feed,
                        at_least,
                        per_members,
                    } => {
                        r.food_members = self.available_food(id, feed).ok();
                        r.consumer_members = site.map_or(0, |s| {
                            self.members_at(s, &BTreeSet::from([e.lineage.clone()]))
                        });
                        r.at_least = *at_least;
                        r.per_members = *per_members;
                        r.food_gate_ready = Some(self.query(id, None, e.place, None, q).is_ok());
                    },
                    Query::EdiblePerHead { feed, at_least, per_members } => {
                        r.edible_matter = self.edible_matter(id, feed).ok();
                        r.consumer_members = site.map_or(0, |s| {
                            self.members_at(s, &BTreeSet::from([e.lineage.clone()]))
                        });
                        r.at_least = *at_least;
                        r.per_members = *per_members;
                    },
                    _ => {},
                }
                let reading = self.query(id, None, e.place, self.bind_part(id, p), q);
                r.ready &= reading.is_ok();
                r.requirements.push(QueryObservation {
                    query: format!("{q:?}"),
                    passed: reading.is_ok(),
                    reading: reading.unwrap_or_else(|why| why),
                });
            }
            let feed = rules.processes.values().find(|feed| {
                traits(e, feed)
                    && feed
                        .effect_declarations()
                        .iter()
                        .any(|effect| matches!(effect, Effect::Eat { .. }))
            });
            if let Some(feed) = feed {
                r.feed_process = Some(feed.id.clone());
                if let Some(site) = site {
                    for (&first, prey) in &self.state.population.groups {
                        let body = &prey.entity;
                        if !body.alive || body.resident_site() != Some(site) {
                            continue;
                        }
                        let allowed = feed.target.as_ref().is_none_or(|s| {
                            s.alive.is_none_or(|alive| alive == body.alive)
                                && s.lineage.as_ref().is_none_or(|l| *l == body.lineage)
                                && (s.among.is_empty() || s.among.contains(&body.lineage))
                        });
                        let raw_admitted = feed.port_feeding.as_ref().map_or(true, |f| {
                            f.part(e).is_ok_and(|(_, intake)| {
                                let books = crate::anatomy::books(body);
                                feed.effect_declarations()
                                    .iter()
                                    .any(|effect| match effect {
                                        Effect::Eat { of, .. } => f
                                            .offered(
                                                intake,
                                                &crate::schedule::edible(&books, of),
                                                body.alive,
                                                rules,
                                                &self.state.lineages,
                                            )
                                            .is_ok_and(|offered| !offered.is_empty()),
                                        _ => false,
                                    })
                            })
                        });
                        if allowed && raw_admitted {
                            let own_member =
                                u64::from(id >= first && id < first.saturating_add(prey.count));
                            r.port_members += prey.count.saturating_sub(own_member);
                        }
                    }
                }
            }
            if let Some(sink) = self.watch.diagnostic.as_ref() {
                sink.lock().birth(r);
            }
        }
    }
}
