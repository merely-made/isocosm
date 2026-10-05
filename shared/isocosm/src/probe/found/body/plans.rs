// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The probe's bodies as recipes (rulings 510, 513 and 528 to 531): a
//! producer is a frond, a grazer a lump bearing its limb or pair of limbs
//! with an eye before it, and each root gives reproduce its cells. A world
//! draws the kinds' extents, each recipe's variance and absence odds, the
//! grazers' clutch and each lineage's tissue domain. With no reproduce cell,
//! no variance and no absences, the recipes develop checkpoint 7's bodies:
//! 513's control.

use super::BodyFounding;
use crate::{anatomy, rules::*, schema::*};
use std::collections::BTreeMap;

pub(super) const FROND: &str = "kind:frond";
pub(super) const LUMP: &str = "kind:lump";
const LIMB: &str = "kind:limb";
const EYE: &str = "kind:eye";

fn capacity(half_extent: [i32; 3]) -> u32 {
    anatomy::capacity(&Part {
        half_extent,
        ..Default::default()
    })
}

/// A kind of `half_extent` whose cells go to these functions; its name is
/// what its box reads (494).
fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells
            .iter()
            .filter(|(_, n)| *n > 0)
            .map(|(f, n)| (format!("function:{f}"), *n))
            .collect(),
        shape: String::new(),
    }
}

fn segment(kind: &str, facing: Facing) -> Tagma {
    Tagma {
        segments: 1,
        segment: kind.into(),
        bears: None,
        per_segment: 0,
        parent: None,
        anchor: Anchor::Tip,
        facing,
        socket: Facing::Right,
        variance: None,
    }
}

impl BodyFounding {
    /// The kinds: a frond that fixes and a lump that takes in and stores,
    /// each giving reproduce the cells drawn for it, a limb that contracts
    /// and an eye that senses. The limb's reach is one per world.
    pub(super) fn kinds(&self) -> BTreeMap<Key, Template> {
        let extent = |axis| self.pick("body-frond", axis, self.frond.map(|h| h as u64)) as i32;
        let frond = [extent(0), extent(1), 1];
        let lump = [2, 2, 2];
        let reproduce = self.pick("body-reproduce", 0, self.reproduce.map(u64::from)) as u32;
        let fixing = capacity(frond);
        let sown = reproduce.min(fixing - 1);
        let room = capacity(lump);
        let borne = reproduce.min(room - 2);
        let store = (self.pick("body-store", 0, self.store) as u32).min(room - borne - 1);
        let reach = self.pick("body-limb", 0, self.limb.map(|h| h as u64)) as i32;
        let kinds = [
            (
                FROND,
                template(frond, &[("fix", fixing - sown), ("reproduce", sown)]),
            ),
            (
                LUMP,
                template(
                    lump,
                    &[
                        ("intake", room - store - borne),
                        ("store", store),
                        ("reproduce", borne),
                    ],
                ),
            ),
            (
                LIMB,
                template([reach, 1, 1], &[("contract", capacity([reach, 1, 1]))]),
            ),
            (EYE, template([1, 1, 1], &[("sense", capacity([1, 1, 1]))])),
        ];
        kinds.map(|(k, t)| (k.to_string(), t)).into()
    }

    /// Each lineage's development: its recipe with the variance and absence
    /// odds drawn for it, its policy bilateral where it bears a pair, its
    /// tissue domain and, for the grazers, its clutch.
    pub(super) fn developments(&self, affinity: &Affinity) -> [Development; 2] {
        let limbs = self.pick("body-limbs", 0, self.limbs);
        let grazer = vec![
            Tagma {
                bears: Some(LIMB.into()),
                per_segment: 1,
                ..segment(LUMP, Facing::Back)
            },
            segment(EYE, Facing::Front),
        ];
        let plans = [(vec![segment(FROND, Facing::Back)], 1), (grazer, limbs)];
        let mut i = 0;
        plans.map(|(tagmata, limbs)| {
            let variance = self.pick("body-variance", i, self.variance.map(u64::from)) as u8;
            let one_in = self.pick("body-absence", i, self.absence.map(u64::from)) as u32;
            let recipe = Recipe {
                tagmata,
                variance,
                absence: if one_in == 0 { [0, 1] } else { [1, one_in] },
                riff: [0, 1],
                vary: [0, 1],
            };
            let domains = u64::from(affinity.domains.max(1));
            let d = Development {
                lexicon: recipe.kinds(),
                recipe,
                policy: Policy {
                    bilateral: limbs == 2,
                    ..Policy::default()
                },
                domain: (crate::draw(self.seed, "body-domain", &[i]) % domains) as u16,
                clutch: match i {
                    0 => 1,
                    _ => self.pick("body-clutch", i, self.clutch.map(u64::from)) as u32,
                },
                anamorphic: false,
            };
            i += 1;
            d
        })
    }
}
