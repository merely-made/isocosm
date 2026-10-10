// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body's lineage beside isometer's document (wing rulings 721 and 756):
//! the species, and each part's mass and provenance, keyed by `PartId`.
//! Isometer keeps geometry and an opaque origin tag; mass is this ledger's
//! reading, not the document's (699).

use std::ops::{Deref, DerefMut};

use isometer_core::{AttachError, Attachment, BodyDocument, PartId, PartOrigin, VolumeRef};
use serde::{Deserialize, Serialize};

/// Identifies a lineage. Provenance records which one a part came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpeciesId(pub u32);

/// Where a part was before it became part of this body.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Origin {
    /// Present when the lineage was founded.
    Founding,
    /// Taken from another organism by incorporation.
    Incorporated {
        from_species: SpeciesId,
        /// The part's identity in the body it came from.
        from_part: PartId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: Origin,
    /// The epoch during which this part joined the body.
    pub epoch: u64,
}

impl Provenance {
    pub fn founding() -> Self {
        Self {
            origin: Origin::Founding,
            epoch: 0,
        }
    }

    /// The opaque tag isometer carries for it: the donor species plus one
    /// for a part taken in, none for the body's own.
    pub fn tag(&self) -> Option<u64> {
        match self.origin {
            Origin::Founding => None,
            Origin::Incorporated { from_species, .. } => Some(u64::from(from_species.0) + 1),
        }
    }
}

/// The wire form: flat, `None` in both fields for a part there at founding.
impl From<&Provenance> for PartOrigin {
    fn from(provenance: &Provenance) -> Self {
        match provenance.origin {
            Origin::Founding => PartOrigin {
                from_species: None,
                from_part: None,
                epoch: provenance.epoch,
            },
            Origin::Incorporated {
                from_species,
                from_part,
            } => PartOrigin {
                from_species: Some(from_species.0),
                from_part: Some(from_part.0),
                epoch: provenance.epoch,
            },
        }
    }
}

impl From<&PartOrigin> for Provenance {
    fn from(origin: &PartOrigin) -> Self {
        let incorporated = match (origin.from_species, origin.from_part) {
            (Some(species), Some(part)) => Origin::Incorporated {
                from_species: SpeciesId(species),
                from_part: PartId(part),
            },
            _ => Origin::Founding,
        };
        Provenance {
            origin: incorporated,
            epoch: origin.epoch,
        }
    }
}

/// What the sim keeps of one part beside the document.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Lineal {
    pub mass_mg: u64,
    pub provenance: Provenance,
}

/// A body document with its lineage beside it: the species, and a
/// [`Lineal`] for every part, index-aligned with the document's parts.
/// Reads through to the document; writes that add parts go through here
/// so the two stay aligned.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineageBody {
    doc: BodyDocument,
    species: SpeciesId,
    lineals: Vec<Lineal>,
}

impl Deref for LineageBody {
    type Target = BodyDocument;

    fn deref(&self) -> &BodyDocument {
        &self.doc
    }
}

/// Reshaping in place (sever, revive, plan, extents) writes through; a new
/// part comes through [`LineageBody::attach`], which keeps the lineage
/// aligned.
impl DerefMut for LineageBody {
    fn deref_mut(&mut self) -> &mut BodyDocument {
        &mut self.doc
    }
}

impl LineageBody {
    /// A body with a single founding root part.
    pub fn new(species: SpeciesId, volume: VolumeRef, mass_mg: u64, half_extent: [i32; 3]) -> Self {
        Self {
            doc: BodyDocument::new(volume, half_extent),
            species,
            lineals: vec![Lineal {
                mass_mg,
                provenance: Provenance::founding(),
            }],
        }
    }

    /// Adds a part fixed to `attachment.parent`, returning its new id.
    pub fn attach(
        &mut self,
        volume: VolumeRef,
        mass_mg: u64,
        half_extent: [i32; 3],
        attachment: Attachment,
        provenance: Provenance,
    ) -> Result<PartId, AttachError> {
        let tag = provenance.tag();
        let id = self.doc.attach(volume, half_extent, attachment, tag)?;
        self.lineals.push(Lineal {
            mass_mg,
            provenance,
        });
        Ok(id)
    }

    /// The document, for geometry.
    pub fn doc(&self) -> &BodyDocument {
        &self.doc
    }

    /// The document to reshape: sever, revive, move. Adding parts goes
    /// through [`Self::attach`].
    pub fn doc_mut(&mut self) -> &mut BodyDocument {
        &mut self.doc
    }

    pub fn into_doc(self) -> BodyDocument {
        self.doc
    }

    pub fn species(&self) -> SpeciesId {
        self.species
    }

    pub fn set_species(&mut self, species: SpeciesId) {
        self.species = species;
    }

    pub fn lineal(&self, id: PartId) -> Option<&Lineal> {
        self.lineals.get(id.0 as usize)
    }

    pub fn lineal_mut(&mut self, id: PartId) -> Option<&mut Lineal> {
        self.lineals.get_mut(id.0 as usize)
    }

    /// A part's mass, nought for an id the body does not hold.
    pub fn mass_mg(&self, id: PartId) -> u64 {
        self.lineal(id).map_or(0, |l| l.mass_mg)
    }

    pub fn set_mass_mg(&mut self, id: PartId, mass_mg: u64) {
        if let Some(l) = self.lineal_mut(id) {
            l.mass_mg = mass_mg;
        }
    }

    pub fn provenance(&self, id: PartId) -> Option<&Provenance> {
        self.lineal(id).map(|l| &l.provenance)
    }

    /// Rewrites a part's provenance and the tag isometer carries for it.
    pub fn set_provenance(&mut self, id: PartId, provenance: Provenance) {
        if let Some(part) = self.doc.parts.get_mut(id.0 as usize) {
            part.origin = provenance.tag();
        }
        if let Some(l) = self.lineal_mut(id) {
            l.provenance = provenance;
        }
    }

    /// Mass of what is still attached: severed parts weigh nothing here.
    pub fn total_mass_mg(&self) -> u64 {
        self.doc.living().map(|p| self.mass_mg(p.id)).sum()
    }

    /// The mass-weighted centre, by this ledger's masses.
    pub fn centre_of_mass(&self) -> [i32; 3] {
        self.doc.centre_of_mass(|id| self.mass_mg(id))
    }

    /// Every part taken from another organism, in id order.
    pub fn incorporated(&self) -> impl Iterator<Item = &isometer_core::Part> {
        self.doc.parts.iter().filter(move |p| {
            self.provenance(p.id)
                .is_some_and(|v| matches!(v.origin, Origin::Incorporated { .. }))
        })
    }

    /// Each part's wire origin, for a profile or a chronicle.
    pub fn part_origin(&self, id: PartId) -> PartOrigin {
        self.provenance(id)
            .map(PartOrigin::from)
            .unwrap_or_default()
    }
}
