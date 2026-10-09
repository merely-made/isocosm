// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Carriage: one operation carrying what several effect sites ask, as far as
//! the routes carry it. Edge capacity is shared among the sites within the
//! operation, as if each fed one sink by its ask, and nothing fails for want
//! of charge or room: the receipt says what reached each site.

use crate::{EvaluationError, FunctionalNetwork, NodeId, NodeKind, Operator, PartRef, SupplyDebit};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// What one carriage delivered.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Carriage {
    /// Each site asked, in the order asked, with what reached it.
    pub delivered: Vec<(NodeId, u64)>,
    /// The supplies debited, in the order first drawn.
    pub supplied_by: Vec<SupplyDebit>,
}

impl Carriage {
    /// Everything that reached the sites.
    pub fn total(&self) -> u64 {
        self.delivered.iter().map(|(_, n)| n).sum()
    }
}

impl FunctionalNetwork {
    /// Carries `asks` to their effect sites in one first-fit, breadth-first
    /// operation, debiting the supplies drawn. A site whose part is not live
    /// takes nothing; a site asked twice takes both, reported at its first.
    pub fn carry(
        &mut self,
        asks: &[(NodeId, u64)],
        max_hops: u32,
        live: &BTreeSet<PartRef>,
    ) -> Result<Carriage, EvaluationError> {
        self.validate().map_err(EvaluationError::InvalidNetwork)?;
        let mut want: BTreeMap<NodeId, u64> = BTreeMap::new();
        for &(site, ask) in asks {
            let node = self
                .nodes
                .get(&site)
                .ok_or(EvaluationError::UnknownTarget(site))?;
            let NodeKind::Effect { part } = node.kind else {
                return Err(EvaluationError::WrongTarget {
                    operator: Operator::Strengthen,
                    target: site,
                });
            };
            if live.contains(&part) {
                let w = want.entry(site).or_default();
                *w = w
                    .checked_add(ask)
                    .ok_or(EvaluationError::ArithmeticOverflow)?;
            }
        }
        let mut clone = self.clone();
        let mut used = vec![0_u64; clone.edges.len()];
        let mut reached: BTreeMap<NodeId, u64> = BTreeMap::new();
        let mut debits: Vec<SupplyDebit> = Vec::new();
        while let Some((source, path)) = clone.find_path(
            |n| want.get(&n).is_some_and(|w| *w > 0),
            max_hops,
            live,
            &used,
        ) {
            let site = clone.edges[*path.last().expect("a route has an edge")].to;
            let room = path
                .iter()
                .map(|e| clone.edges[*e].capacity - used[*e])
                .min()
                .unwrap_or(0);
            let charge = match &mut clone.nodes.get_mut(&source).expect("route source").kind {
                NodeKind::Source { charge, .. } | NodeKind::Store { charge, .. } => charge,
                _ => unreachable!("routes start at supplies"),
            };
            let amount = want[&site].min(*charge).min(room);
            if amount == 0 {
                break;
            }
            *charge -= amount;
            for e in &path {
                used[*e] += amount;
            }
            *want.get_mut(&site).expect("asked") -= amount;
            *reached.entry(site).or_default() += amount;
            match debits.iter_mut().find(|d| d.source == source) {
                Some(d) => d.amount += amount,
                None => debits.push(SupplyDebit { source, amount }),
            }
        }
        *self = clone;
        let mut taken = reached;
        let delivered = asks
            .iter()
            .map(|(site, _)| (*site, taken.remove(site).unwrap_or(0)))
            .collect();
        Ok(Carriage {
            delivered,
            supplied_by: debits,
        })
    }
}
