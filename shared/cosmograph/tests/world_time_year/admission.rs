// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Structural admission of the actual native founders, not an ecology result.

use cosmograph::{
    anatomy,
    ecology::{Rates, Web},
    rules::{Amount, Binding, Effect, Query},
    schema::{Id, Key, PartId},
    simulation::Genesis,
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Founder {
    pub group: Id,
    pub members: u64,
    pub lineage: Key,
    pub account: Key,
    pub store: PartId,
    pub ground_y: i32,
    pub exposed_parts: usize,
    pub stretch_segments: Vec<u8>,
    pub root_matter: u64,
    pub exposed_matter: u64,
    pub other_matter: u64,
    pub root_is_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Admission {
    pub scope: &'static str,
    pub qualified: bool,
    pub failures: Vec<String>,
    pub producers: Vec<Founder>,
}

impl Admission {
    fn check(&mut self, valid: bool, why: impl Into<String>) {
        if !valid {
            self.failures.push(why.into());
        }
    }

    pub fn inspect(g: &Genesis) -> Self {
        let mut out = Self {
            scope: "structural founder admission for cases 1114/1115; behavioral regressions belong to the frozen build evidence; trajectory verdict is separate",
            qualified: false,
            failures: vec![],
            producers: vec![],
        };
        let Some(founding) = &g.founding else {
            out.failures.push("no saved founding parameters".into());
            return out;
        };
        out.check(
            founding.ecology
                && founding.ecology_rates == Rates::WorldTime
                && founding.web == Web::Diets,
            "not the declared world-time diets ecology",
        );
        out.check(
            founding.cohort_size == 1,
            "founders are not single-member bodies",
        );
        let bounded = founding.bodies.as_ref().is_some_and(|b| {
            b.clutch == [1, 1] && b.sensing && b.grazing && b.hazard == [0, 0] && b.wound == [0, 0]
        });
        out.check(
            bounded,
            "saved bodies lack single-clutch sensing/grazing/nohazard bounds",
        );
        out.check(
            g.rules.tick_microseconds() == 60_000_000,
            "the native clock is not one minute",
        );
        out.check(g.rules.senses.is_some(), "native sensing is not configured");
        out.check(
            !g.rules.processes.contains_key("body:hazard"),
            "a body hazard is configured",
        );
        let mut checked = BTreeMap::new();
        for (&id, group) in &g.population.groups {
            let e = &group.entity;
            if !e.alive || !e.traits.contains("life:producer") {
                continue;
            }
            let prefix = format!("producer {id} ({})", e.lineage);
            out.check(
                group.count == 1 && group.pool.is_empty(),
                format!("{prefix}: not an unpooled individual founder"),
            );
            let Some(index) = e.lineage.strip_prefix("lineage:") else {
                out.failures
                    .push(format!("{prefix}: no native founding lineage index"));
                continue;
            };
            let account = format!("matter:{index}-0");
            let Some(a) = anatomy::root_store::allocation(e, &g.rules, &account) else {
                out.failures
                    .push(format!("{prefix}: no admitted own-tissue root allocation"));
                continue;
            };
            out.check(
                a.rule.exposed == 40 && a.rule.stored == 20 && a.rule.seed == 10,
                format!("{prefix}: adult/seed shares differ from exposed40/store20/seed10"),
            );
            out.check(
                e.body.is_some() && e.lives(a.store),
                format!("{prefix}: no live native store body"),
            );
            let root_is_exposed = anatomy::root_store::exposed(e, &g.rules, a.store);
            out.check(
                !root_is_exposed,
                format!("{prefix}: root geometry reaches above its ground plane"),
            );
            out.check(
                (5..=17).contains(&a.exposed.len())
                    && a.exposed.values().all(|n| *n > 0)
                    && a.exposed.values().sum::<u64>() == 40,
                format!(
                    "{prefix}: exposed part allocation is outside five-to-seventeen paid parts"
                ),
            );
            out.check(
                a.exposed
                    .keys()
                    .all(|&part| e.lives(part) && anatomy::root_store::exposed(e, &g.rules, part)),
                format!("{prefix}: an exposed allocation is not a live aboveground part"),
            );
            let held = |part| {
                e.parts
                    .get(&part)
                    .and_then(|p| p.matter.get(&account))
                    .copied()
                    .unwrap_or(0)
            };
            let root_matter = held(a.store);
            let exposed_matter = a.exposed.keys().map(|&id| held(id)).sum::<u64>();
            out.check(
                a.exposed.iter().all(|(&id, &share)| held(id) == share),
                format!("{prefix}: an exposed native ledger differs from its adult share"),
            );
            let other_matter = e.accounts.get(&account).copied().unwrap_or(0)
                + e.parts
                    .iter()
                    .filter(|(id, _)| **id != a.store && !a.exposed.contains_key(id))
                    .map(|(_, p)| p.matter.get(&account).copied().unwrap_or(0))
                    .sum::<u64>();
            out.check(
                root_matter == 20 && exposed_matter == 40 && other_matter == 0,
                format!("{prefix}: native matter is not exactly root20/exposed40/other0"),
            );
            let development = g
                .lineages
                .get(&e.lineage)
                .and_then(|line| line.development.as_ref());
            let stretches: Vec<_> = development
                .into_iter()
                .flat_map(|d| &d.recipe.tagmata)
                .filter(|t| t.segment == "kind:rooted-frond")
                .map(|t| t.segments)
                .collect();
            out.check(
                development.is_some_and(|d| d.clutch == 1 && !d.anamorphic)
                    && (1..=2).contains(&stretches.len())
                    && stretches.iter().all(|n| (4..=8).contains(n)),
                format!(
                    "{prefix}: recipe is not one seed and one-to-two four-to-eight-part stretches"
                ),
            );
            if checked.insert(e.lineage.clone(), ()).is_none() {
                out.processes(g, index, &account, &a.rule.identity);
            }
            out.producers.push(Founder {
                group: id,
                members: group.count,
                lineage: e.lineage.clone(),
                account,
                store: a.store,
                ground_y: a.rule.ground_y,
                exposed_parts: a.exposed.len(),
                stretch_segments: stretches,
                root_matter,
                exposed_matter,
                other_matter,
                root_is_exposed,
            });
        }
        out.check(
            !out.producers.is_empty(),
            "no actual living rooted producer founders",
        );
        out.qualified = out.failures.is_empty();
        out
    }

    fn processes(&mut self, g: &Genesis, index: &str, account: &str, identity: &str) {
        let minutes = |n| cosmograph::rules::minutes_ticks(n, g.rules.tick_microseconds());
        let owns = |p: &cosmograph::rules::Process| {
            p.requires
                .iter()
                .any(|q| matches!(q, Query::Trait { who: Binding::Actor, key } if key == identity))
        };
        let process = |verb: &str| g.rules.processes.get(&format!("ecology:{verb}-{index}"));
        let grow = process("produce");
        self.check(grow.is_some_and(|p| p.period == Some(minutes(720)) && owns(p)
            && p.effects.iter().any(|e| matches!(e, Effect::Transfer {
                from: Binding::Place, to: Binding::Actor, account, amount: Amount::Fixed(3000)
            } if account == "sim:energy"))
            && p.effects.iter().any(|e| matches!(e, Effect::Transform {
                who: Binding::Actor, take, give, ..
            } if take.get("world:soil") == Some(&2) && give.get(account) == Some(&2)))
            && p.requires.iter().any(|q| matches!(q, Query::Sensed { reading, at_least: 3000 } if reading == "sense:light-share"))
            && p.requires.iter().any(|q| matches!(q, Query::Sensed { reading, at_least: 2 } if reading == "sense:soil-share"))),
            format!("lineage:{index}: ordinary 720-minute light/soil growth is absent"));
        self.check(
            process("regrow").is_some_and(|p| {
                p.period == Some(minutes(240))
                    && owns(p)
                    && p.effects
                        == vec![Effect::Regrow {
                            account: account.into(),
                            amount: 2.into(),
                        }]
            }),
            format!("lineage:{index}: conservative 240-minute root regrowth is absent"),
        );
        self.check(
            process("upkeep").is_some_and(|p| {
                p.period == Some(minutes(1440))
                    && owns(p)
                    && p.effects.iter().any(|e| {
                        matches!(e, Effect::Transform { who: Binding::Actor, take, give, .. }
                if take.get(account) == Some(&1) && give.get("world:detritus") == Some(&1))
                    })
            }),
            format!("lineage:{index}: ordinary daily root upkeep is absent"),
        );
        self.check(process("birth").is_some_and(|p| p.period == Some(minutes(90 * 1440)) && owns(p)
            && p.effects.iter().any(|e| matches!(e, Effect::Birth { provision, .. } if provision.get(account) == Some(&10)))
            && p.requires.iter().any(|q| matches!(q, Query::Account {
                who: Binding::Actor, key, at_least: 70
            } if key == account))), format!("lineage:{index}: ordinary single-seed seasonal birth is absent"));
        self.check(
            process("death").is_some_and(|p| {
                owns(p)
                    && p.requires.contains(&Query::EmptyBody)
                    && p.effects.contains(&Effect::Death)
                    && !p.requires.iter().any(|q| matches!(q, Query::Below { .. }))
            }),
            format!("lineage:{index}: death is not gated on the entire empty body"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmograph::{Founding, bodied::Bodies};

    fn fixture() -> Genesis {
        Founding {
            seed: 9,
            sites: 4,
            population: 30,
            cohort_size: 1,
            lineages: 5,
            ecology: true,
            web: Web::Diets,
            ecology_rates: Rates::WorldTime,
            experimental_food_contract: true,
            bodies: Some(Bodies {
                clutch: [1, 1],
                ..Bodies::default()
            }),
            ..Default::default()
        }
        .generate()
        .unwrap()
    }

    #[test]
    fn actual_native_founders_admit_the_root_structure() {
        let g = fixture();
        let admission = Admission::inspect(&g);
        assert!(admission.qualified, "{:?}", admission.failures);
        assert!(
            admission
                .producers
                .iter()
                .all(|p| p.root_matter == 20 && p.exposed_matter == 40 && !p.root_is_exposed)
        );
        let s = cosmograph::Session::new(g, cosmograph::Execution::Individuals).unwrap();
        let report = crate::support::Report::new(&s);
        assert!(report.root_store_qualified);
        for tissue in report.samples[0]
            .rooted_tissue
            .values()
            .flat_map(|by| by.values())
        {
            assert_eq!(tissue.root_matter, u128::from(tissue.members) * 20);
            assert_eq!(tissue.exposed_matter, u128::from(tissue.members) * 40);
            assert_eq!(tissue.other_part_matter, 0);
            assert_eq!(tissue.entity_ledger_matter, 0);
            assert_eq!(tissue.unassigned_cohort_pool, 0);
        }
    }

    #[test]
    fn missing_or_invalid_root_contracts_are_unqualified() {
        let mut g = fixture();
        g.rules.root_stores.clear();
        assert!(!Admission::inspect(&g).qualified);
        let mut g = fixture();
        g.rules.root_stores.values_mut().next().unwrap().stored = 19;
        assert!(!Admission::inspect(&g).qualified);
        let mut g = fixture();
        g.rules.root_stores.values_mut().next().unwrap().ground_y = i32::MIN;
        assert!(
            !Admission::inspect(&g).qualified,
            "a store tag cannot override exposed geometry"
        );
    }

    #[test]
    fn a_native_ledger_mismatch_is_unqualified() {
        let mut g = fixture();
        let account = g.rules.root_stores.keys().next().unwrap().clone();
        let group = g
            .population
            .groups
            .values_mut()
            .find(|group| {
                anatomy::root_store::contract(&group.entity, &g.rules, &account).is_some()
            })
            .unwrap();
        let root = group.entity.body.as_ref().unwrap().root;
        group
            .entity_mut()
            .part_mut(&root)
            .unwrap()
            .matter
            .insert(account, 19);
        assert!(!Admission::inspect(&g).qualified);
    }
}
