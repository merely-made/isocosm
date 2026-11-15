// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Result,
    development::{self, Soma},
    rules::{Development, Rules},
    schema::{Entity, Part, PartId},
};
use isometer_core::BodyDocument;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

const LINEAGES: usize = 128;
const VARIANTS: usize = 8;
const MAX_CACHED_PARTS: usize = 128;

type Form = (Vec<u8>, Vec<(u8, u8)>);

#[derive(Debug)]
struct Line {
    revision: u64,
    development: Development,
    bodies: BTreeMap<Form, Anatomy>,
}

#[derive(Debug)]
struct Anatomy {
    document: BodyDocument,
    parts: BTreeMap<PartId, Arc<Part>>,
}

/// Young with the same development and soma share their empty parts. Giving
/// a child its provision copies only the parts written by that birth.
#[derive(Debug, Default)]
pub(crate) struct Cache(Mutex<BTreeMap<String, Line>>);

impl Clone for Cache {
    fn clone(&self) -> Self {
        // Forecasts and restored sessions can use different rules.
        Self::default()
    }
}

impl Cache {
    pub(crate) fn embody(
        &self, rules: &Rules, (revision, d): (u64, &Development), soma: &Soma, child: &mut Entity,
    ) -> Result<()> {
        let key = (soma.segments.clone(), soma.absent.clone());
        let mut lines = self.0.lock().expect("birth anatomy cache");
        if lines.len() >= LINEAGES && !lines.contains_key(&child.lineage) {
            lines.pop_first();
        }
        let line = lines.entry(child.lineage.clone()).or_insert_with(|| Line {
            revision,
            development: d.clone(),
            bodies: BTreeMap::new(),
        });
        if line.revision != revision || line.development != *d {
            line.revision = revision;
            line.development.clone_from(d);
            line.bodies.clear();
        }
        if let Some(anatomy) = line.bodies.get(&key) {
            child.body = Some(anatomy.document.clone());
            child.parts.clone_from(&anatomy.parts);
            return Ok(());
        }

        child.embody(development::develop(rules, d, soma)?);
        if child.parts.len() <= MAX_CACHED_PARTS {
            let anatomy = Anatomy {
                document: child.body.as_ref().expect("developed body").clone(),
                parts: child.parts.clone(),
            };
            if line.bodies.len() >= VARIANTS {
                line.bodies.pop_first();
            }
            line.bodies.insert(key, anatomy);
        }
        Ok(())
    }
}
