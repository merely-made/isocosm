// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, geometry::Body, schema::*};
use std::collections::BTreeMap;

pub(crate) struct Fragment {
    pub child: Entity,
    pub mapping: Vec<(PartId, PartId)>,
}

impl Fragment {
    pub(crate) fn legs(&self, parent: Id, child: Id) -> Vec<crate::flows::Leg> {
        use crate::flows::{Holder, Leg};
        self.mapping
            .iter()
            .flat_map(|(old, new)| {
                self.child.parts[new]
                    .matter
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .map(move |(key, n)| Leg {
                        from: (Holder::Part(parent, *old), key.clone()),
                        to: (Holder::Part(child, *new), key.clone()),
                        amount: *n,
                    })
            })
            .collect()
    }
}

pub(crate) fn detach(e: &mut Entity, part: PartId, tick: Tick, alive: bool) -> Result<Fragment> {
    let doc = e.body.as_ref().ok_or("a severing with no geometry")?;
    let subtree = doc.descendants(part);
    if subtree.is_empty() || part == doc.root {
        return Err("a severing with no subtree".into());
    }
    let mapping: Vec<_> = subtree
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, PartId(i as u32)))
        .collect();
    let ids: BTreeMap<_, _> = mapping.iter().copied().collect();
    let mut body = Body {
        doc: doc.clone(),
        parts: BTreeMap::new(),
    };
    body.doc.root = PartId(0);
    body.doc.parts = subtree
        .iter()
        .map(|old| {
            let mut g = doc.part(*old).expect("a descendant exists").clone();
            g.id = ids[old];
            if *old == part {
                g.attachment = None;
                g.situs = Some([0; 3]);
            } else if let Some(a) = &mut g.attachment {
                a.parent = ids[&a.parent];
            }
            g.severed = false;
            g
        })
        .collect();
    for (old, new) in &mapping {
        let p = e.parts.get_mut(old).ok_or("a subtree missing physiology")?;
        let mut copied = p.clone();
        copied.matter = std::mem::take(&mut p.matter);
        body.parts.insert(*new, copied);
    }
    e.body.as_mut().unwrap().sever(part);
    let mut child = crate::meaning::births::newborn(e, body, e.soma.clone(), tick);
    child.alive = alive;
    if !alive {
        child.provenance = e.provenance.clone();
        child.born = e.born;
    }
    child.patch = e.patch;
    Ok(Fragment { child, mapping })
}
