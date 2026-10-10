// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body's geometry, read through isometer (rulings 674, 699 and 719):
//! the body document holds each part's box, attachment, place in the plan,
//! declared name and tombstone; the sim keeps the physiology, keyed by the
//! document's part ids. An entity without a document keeps parts with no
//! geometry, as parts did before bodies.

use crate::{Result, schema::*};
use isometer_core::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
use std::collections::BTreeMap;

/// A part's geometry before it joins a document: what development,
/// growth and incorporation decide.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub parent: Option<PartId>,
    pub half_extent: [i32; 3],
    /// Its pivot from its parent's, in the parent's frame (462, 510).
    pub offset: [i32; 3],
    pub situs: Option<[u8; 3]>,
    pub shape: Key,
}

/// A body's geometry and physiology together, as development makes one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body {
    pub doc: BodyDocument,
    pub parts: BTreeMap<PartId, Part>,
}

impl Body {
    /// A body of `parts` in id order, the first the root, each frame's
    /// parent named by its id.
    pub fn of(parts: impl IntoIterator<Item = (Frame, Part)>) -> Result<Body> {
        let (frames, parts): (Vec<Frame>, Vec<Part>) = parts.into_iter().unzip();
        let doc = assemble(&frames)?;
        let parts = parts
            .into_iter()
            .enumerate()
            .map(|(i, p)| (PartId(i as u32), p))
            .collect();
        Ok(Body { doc, parts })
    }
}

/// A part as a fixture writes one, geometry and physiology together; ids
/// are the keys it is sketched under.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sketch {
    pub parent: Option<u32>,
    pub half_extent: [i32; 3],
    pub offset: [i32; 3],
    pub situs: Option<[u8; 3]>,
    pub shape: Key,
    pub severed: bool,
    pub part: Part,
}

impl Body {
    /// A body as `parts` sketch it, keyed by id; ids left out stand as
    /// tombstones holding nothing, and a parent may come after its child.
    pub fn sketch(parts: impl IntoIterator<Item = (u32, Sketch)>) -> Body {
        let parts: BTreeMap<u32, Sketch> = parts.into_iter().collect();
        let len = parts.keys().next_back().map_or(0, |n| n + 1);
        let root = parts
            .iter()
            .find(|(_, s)| s.parent.is_none())
            .map_or(0, |(n, _)| *n);
        let mut doc = document(&Frame::default());
        doc.root = PartId(root);
        doc.parts.clear();
        let mut kept = BTreeMap::new();
        for n in 0..len {
            let s = parts.get(&n).cloned().unwrap_or(Sketch {
                severed: true,
                ..Default::default()
            });
            let at = s.parent.map(|p| Attachment {
                parent: PartId(p),
                offset: s.offset,
                yaw: Yaw::Zero,
            });
            doc.parts.push(isometer_core::Part {
                id: PartId(n),
                volume: UNDRAWN,
                mass_mg: 0,
                half_extent: s.half_extent,
                pivot: s.half_extent,
                attachment: at,
                provenance: Provenance::founding(),
                severed: s.severed,
                situs: s.situs,
                shape: s.shape.clone(),
            });
            if parts.contains_key(&n) {
                kept.insert(PartId(n), s.part);
            }
        }
        Body { doc, parts: kept }
    }
}

/// No volume drawn yet: the sim names none.
const UNDRAWN: VolumeRef = VolumeRef([0; 32]);

/// A document rooted at `root`. Mass is the sim's ledger's (699) and the
/// lineage fields isometer still carries are left neutral until 721 lands.
pub fn document(root: &Frame) -> BodyDocument {
    let mut d = BodyDocument::new(SpeciesId(0), UNDRAWN, 0, root.half_extent);
    let part = &mut d.parts[0];
    part.situs = root.situs;
    part.shape = root.shape.clone();
    d
}

/// Attaches `f` under its parent, returning its id.
pub fn attach(d: &mut BodyDocument, f: &Frame) -> Result<PartId> {
    let parent = f.parent.ok_or("a second root")?;
    let at = Attachment {
        parent,
        offset: f.offset,
        yaw: Yaw::Zero,
    };
    let id = d
        .attach(UNDRAWN, 0, f.half_extent, at, Provenance::founding())
        .map_err(|e| format!("{e:?}"))?;
    let part = &mut d.parts[id.0 as usize];
    part.situs = f.situs;
    part.shape = f.shape.clone();
    Ok(id)
}

/// A document of `frames` in order, the first the root, each parent named
/// by its index among them.
pub fn assemble(frames: &[Frame]) -> Result<BodyDocument> {
    let (root, rest) = frames.split_first().ok_or("a body of no parts")?;
    let mut d = document(root);
    for f in rest {
        attach(&mut d, f)?;
    }
    Ok(d)
}

impl Entity {
    /// Takes `b` as its body, geometry and physiology.
    pub fn embody(&mut self, b: Body) {
        self.body = Some(b.doc);
        self.parts = b.parts;
    }

    /// The document's record of part `id`.
    pub fn geo(&self, id: PartId) -> Option<&isometer_core::Part> {
        self.body.as_ref()?.part(id)
    }

    /// Its half-extents, nought where it has no geometry.
    pub fn extent(&self, id: PartId) -> [i32; 3] {
        self.geo(id).map_or([0; 3], |g| g.half_extent)
    }

    pub fn parent_of(&self, id: PartId) -> Option<PartId> {
        self.geo(id)?.attachment.map(|a| a.parent)
    }

    pub fn situs(&self, id: PartId) -> Option<[u8; 3]> {
        self.geo(id)?.situs
    }

    /// The name it was declared by, empty where its box is its name.
    pub fn declared(&self, id: PartId) -> &str {
        self.geo(id).map_or("", |g| g.shape.as_str())
    }

    /// Whether the part is held and not a tombstone.
    pub fn lives(&self, id: PartId) -> bool {
        match &self.body {
            Some(b) => b.is_living(id) && self.parts.contains_key(&id),
            None => self.parts.contains_key(&id),
        }
    }

    /// Whether it has a box to hold matter (453).
    pub fn bodied(&self, id: PartId) -> bool {
        self.extent(id) != [0; 3]
    }

    /// The living parts, in id order.
    pub fn living(&self) -> impl Iterator<Item = (PartId, &Part)> {
        self.parts
            .iter()
            .filter(|(id, _)| self.lives(**id))
            .map(|(id, p)| (*id, p))
    }

    /// The living parts attached directly to `id`.
    pub fn children_of(&self, id: PartId) -> Vec<PartId> {
        self.body
            .as_ref()
            .map(|b| {
                b.children(id)
                    .filter(|c| self.parts.contains_key(c))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The part's geometry as a frame.
    pub fn frame(&self, id: PartId) -> Frame {
        Frame {
            parent: self.parent_of(id),
            half_extent: self.extent(id),
            offset: self
                .geo(id)
                .and_then(|g| g.attachment)
                .map_or([0; 3], |a| a.offset),
            situs: self.situs(id),
            shape: self.declared(id).to_string(),
        }
    }

    /// Adds a part of geometry `f` and physiology `p`: the root of a new
    /// document where the body has none and `f` names no parent.
    pub fn add_part(&mut self, f: &Frame, p: Part) -> Result<PartId> {
        let id = match (&mut self.body, f.parent) {
            (Some(d), _) => attach(d, f)?,
            (None, None) if self.parts.is_empty() => {
                self.body = Some(document(f));
                PartId(0)
            },
            (None, _) => return Err("a part with geometry on a body without".into()),
        };
        self.parts.insert(id, p);
        Ok(id)
    }

    /// Takes part `id` off the body whole, as a bud or an incorporated part
    /// leaves (rulings 516 and 524): the document keeps its tombstone, its
    /// place in the plan cleared so the recipe may grow the place again, as
    /// before tombstones; its geometry and physiology are handed back.
    pub fn take_part(&mut self, id: PartId) -> Option<(Frame, Part)> {
        let frame = self.frame(id);
        let part = self.parts.remove(&id)?;
        if let Some(g) = self
            .body
            .as_mut()
            .and_then(|d| d.parts.get_mut(id.0 as usize))
        {
            g.severed = true;
            g.situs = None;
        }
        Some((frame, part))
    }
}
