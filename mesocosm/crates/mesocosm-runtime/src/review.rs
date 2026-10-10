// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The played line's turn at the boundary, `lineage::Review` over the
//! native session (ruling 762), and beside a declared-tract offer what a
//! pack's expression scripts would make of it on the played body (781):
//! the cells they ask for, placed through `Allocation::commit` on a copy.
//! Built when the question opens, never per frame; nothing is written back.

use std::path::Path;

use isocosm::directing::interim::Interim;
use isocosm::history::Command;
use isocosm::process::Registry;
use isocosm::schema::Entity;
pub use isocosm::lineage::{Offer, Reading, Review};
use mesocosm_phenotype::express::{Entropy, Policy, Request, Runner, lower};
use mesocosm_phenotype::{Admission, asset, discover};

/// The review for the played critter's line, when it has one.
pub(crate) fn of(interim: &Interim) -> Option<Review> {
    let critter = interim.critter()?;
    let pop = &interim.session.sim.state().population;
    let lineage = pop.get(critter)?.lineage.clone();
    Review::of(&interim.session, &lineage, interim.pace.scoring).ok()
}

/// A pack's declared expression scripts, the review's second source.
#[derive(Clone, Debug, Default)]
pub struct Authored {
    scripts: Vec<(String, String)>,
    policy: Policy,
}

/// What one script made of one offer: the cells it would change on the
/// played body, or why it could not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposed {
    pub offer: usize,
    pub script: String,
    pub cells: Result<u32, String>,
}

impl Authored {
    pub fn load(root: &Path) -> Result<Self, Admission> {
        let manifest = discover(root)?;
        let mut scripts = Vec::with_capacity(manifest.expression.len());
        for relative in &manifest.expression {
            let path = asset(root, &manifest, relative)?;
            let source = std::fs::read_to_string(&path).map_err(|e| Admission::Unreadable {
                path: path.display().to_string(),
                why: e.to_string(),
            })?;
            scripts.push((relative.clone(), source));
        }
        Ok(Self {
            scripts,
            policy: Policy::default(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }

    /// Every script's proposal for every offer that declares a tract the
    /// line does not hold (`held` functions), on a copy of `body`.
    pub fn propose(&self, review: &Review, body: &Entity, held: &[String], seed: u64) -> Vec<Proposed> {
        let registry = Registry::native();
        let entropy = Entropy::from_seed(isocosm::draw(seed, "authored", &[review.tick]));
        let material = body.accounts.values().fold(0u64, |a, b| a.saturating_add(*b));
        let mut out = Vec::new();
        for (index, offer) in review.offers.iter().enumerate() {
            let mut declared = declared(offer);
            declared.retain(|f| !held.contains(f));
            if declared.is_empty() {
                continue;
            }
            let request = Request::frozen(registry, body, declared, material, vec![]);
            for (name, source) in &self.scripts {
                let cells = Runner::load(source, self.policy)
                    .and_then(|mut runner| runner.propose(&request, &entropy))
                    .and_then(|proposal| lower(registry, body, &proposal))
                    .and_then(|allocation| allocation.commit(&mut body.clone()))
                    .map_err(|refused| refused.words());
                out.push(Proposed {
                    offer: index,
                    script: name.clone(),
                    cells,
                });
            }
        }
        out
    }
}

/// A tract function as the registry names its definition (773).
pub(crate) fn qualified(function: &str) -> String {
    format!("mesocosm:{}", function.trim_start_matches("function:"))
}

/// The functions an offer's revision declares, as the registry names them.
fn declared(offer: &Offer) -> Vec<String> {
    let mut out: Vec<String> = offer
        .commands
        .iter()
        .filter_map(|c| match c {
            Command::Revise { development, .. } => Some(&development.tracts),
            _ => None,
        })
        .flatten()
        .map(|t| qualified(&t.function))
        .collect();
    out.sort();
    out.dedup();
    out
}
