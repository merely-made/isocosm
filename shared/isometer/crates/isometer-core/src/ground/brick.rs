// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One 8³ brick, kept as its one material while every voxel shares it.
//!
//! The form is canonical: a brick whose voxels all hold one material within
//! the palette is uniform, however it got there, so equal bricks compare and
//! encode alike and the ground's hash witness holds across edit histories.

use std::fmt;

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize, Serializer};

use super::{AIR, BRICK, MAX_MATERIAL};

/// Voxels in one brick.
const CELLS: usize = (BRICK * BRICK * BRICK) as usize;

/// Every palette material as a filled brick, so a uniform brick answers
/// [`Brick::raw`] without storing its voxels.
static FILLED: [[u8; CELLS]; MAX_MATERIAL as usize + 1] = {
    let mut rows = [[0; CELLS]; MAX_MATERIAL as usize + 1];
    let mut m = 0;
    while m < rows.len() {
        rows[m] = [m as u8; CELLS];
        m += 1;
    }
    rows
};

/// One 8³ brick, y-major then z then x.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Brick(Form);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Form {
    Uniform(u8),
    Dense(Box<[u8; CELLS]>),
}

impl Brick {
    pub(super) fn empty() -> Self {
        Self(Form::Uniform(AIR))
    }

    /// A brick from all its voxels, uniform when they share a palette material.
    pub(super) fn from_materials(materials: Vec<u8>) -> Self {
        let cells: Box<[u8; CELLS]> = materials
            .into_boxed_slice()
            .try_into()
            .expect("a brick holds exactly 8³ voxels");
        Self::settled(cells)
    }

    fn settled(cells: Box<[u8; CELLS]>) -> Self {
        let first = cells[0];
        if first <= MAX_MATERIAL && cells.iter().all(|m| *m == first) {
            Self(Form::Uniform(first))
        } else {
            Self(Form::Dense(cells))
        }
    }

    pub(super) fn index(local: [i32; 3]) -> usize {
        ((local[1] * BRICK + local[2]) * BRICK + local[0]) as usize
    }

    pub fn get(&self, local: [i32; 3]) -> u8 {
        self.raw()[Self::index(local)]
    }

    pub(super) fn set(&mut self, local: [i32; 3], material: u8) {
        let at = Self::index(local);
        match &mut self.0 {
            Form::Uniform(held) if *held == material => {},
            Form::Uniform(held) => {
                let mut cells = Box::new([*held; CELLS]);
                cells[at] = material;
                self.0 = Form::Dense(cells);
            },
            Form::Dense(cells) => {
                cells[at] = material;
                // Only a write of the material at the first voxel can make
                // the brick uniform, so most writes skip the pass.
                if cells[0] == material
                    && material <= MAX_MATERIAL
                    && cells.iter().all(|m| *m == material)
                {
                    self.0 = Form::Uniform(material);
                }
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.uniform() == Some(AIR)
    }

    /// The one material every voxel holds, if the brick is uniform.
    pub fn uniform(&self) -> Option<u8> {
        match self.0 {
            Form::Uniform(material) => Some(material),
            Form::Dense(_) => None,
        }
    }

    /// The brick as raw material bytes, for building a mesh-crate volume.
    pub fn raw(&self) -> &[u8] {
        match &self.0 {
            Form::Uniform(material) => &FILLED[*material as usize],
            Form::Dense(cells) => &cells[..],
        }
    }
}

/// A uniform brick encodes as its material, a dense one as its voxel bytes.
impl Serialize for Brick {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        struct Bytes<'a>(&'a [u8]);
        impl Serialize for Bytes<'_> {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_bytes(self.0)
            }
        }
        match &self.0 {
            Form::Uniform(material) => s.serialize_newtype_variant("Brick", 0, "Uniform", material),
            Form::Dense(cells) => {
                s.serialize_newtype_variant("Brick", 1, "Dense", &Bytes(&cells[..]))
            },
        }
    }
}

impl<'de> Deserialize<'de> for Brick {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename = "Brick")]
        enum Wire {
            Uniform(u8),
            Dense(Cells),
        }
        match Wire::deserialize(d)? {
            Wire::Uniform(material) if material <= MAX_MATERIAL => {
                Ok(Self(Form::Uniform(material)))
            },
            Wire::Uniform(material) => Err(de::Error::custom(format_args!(
                "uniform material {material} past the palette bound {MAX_MATERIAL}"
            ))),
            // Settling keeps one ground one encoding, even from foreign bytes.
            Wire::Dense(Cells(cells)) => Ok(Self::settled(cells)),
        }
    }
}

/// A dense brick's voxel bytes, exactly 8³ of them.
struct Cells(Box<[u8; CELLS]>);

impl<'de> Deserialize<'de> for Cells {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Expect;
        impl Expect {
            fn exact<E: de::Error>(bytes: Vec<u8>) -> Result<Cells, E> {
                let len = bytes.len();
                bytes
                    .into_boxed_slice()
                    .try_into()
                    .map(Cells)
                    .map_err(|_| E::invalid_length(len, &Expect))
            }
        }
        impl<'de> Visitor<'de> for Expect {
            type Value = Cells;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{CELLS} voxel bytes")
            }

            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Cells, E> {
                Self::exact(v.to_vec())
            }

            fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Cells, E> {
                Self::exact(v)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Cells, A::Error> {
                let mut bytes = Vec::with_capacity(CELLS);
                while let Some(b) = seq.next_element()? {
                    bytes.push(b);
                }
                Self::exact(bytes)
            }
        }
        d.deserialize_byte_buf(Expect)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Cavity, Ground, ROCK, SOIL, Terrain};
    use super::*;
    use crate::snapshot::{decode, encode};

    struct Hills;

    impl Terrain for Hills {
        fn sea_level(&self, _extent: i32) -> i32 {
            2
        }

        // Gentle enough that buried bricks are all rock.
        fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
            14 + (x / 5 + z / 7).rem_euclid(4)
        }

        fn cavities(&self, _extent: i32) -> Vec<Cavity> {
            vec![Cavity {
                rooms: vec![([4, 6, 4], 3)],
                route: Vec::new(),
            }]
        }
    }

    #[test]
    fn a_brick_is_uniform_exactly_when_its_voxels_agree() {
        let mut brick = Brick::empty();
        assert_eq!(brick.uniform(), Some(AIR));
        assert!(brick.is_empty());
        brick.set([1, 2, 3], ROCK);
        assert_eq!(brick.uniform(), None);
        assert_eq!(brick.get([1, 2, 3]), ROCK);
        assert_eq!(brick.raw().iter().filter(|m| **m == ROCK).count(), 1);
        brick.set([1, 2, 3], AIR);
        assert_eq!(
            brick,
            Brick::empty(),
            "writing the odd voxel back settles it"
        );

        let filled = Brick::from_materials(vec![SOIL; CELLS]);
        assert_eq!(filled.uniform(), Some(SOIL));
        assert_eq!(filled.raw(), &[SOIL; CELLS][..]);
        // Past the palette there is no filled row to answer from.
        assert_eq!(Brick::from_materials(vec![200; CELLS]).uniform(), None);
    }

    #[test]
    fn an_edited_ground_brought_back_encodes_as_grown() {
        let grown = Ground::grow(&Hills, 20);
        let uniform = grown
            .keys()
            .filter(|k| grown.brick_materials(*k).unwrap().0.uniform().is_some())
            .count();
        assert!(
            uniform > 0 && uniform < grown.brick_count(),
            "both forms present"
        );

        let mut edited = grown.clone();
        // Dig into a buried uniform brick and a surface brick, then refill.
        let dug: Vec<_> = [[2, 3, 2], [2, 4, 2], [-7, 1, 5], [10, 20, -3]]
            .into_iter()
            .map(|at| (at, edited.material(at).unwrap()))
            .collect();
        edited.write(dug.iter().map(|(at, _)| (*at, AIR))).unwrap();
        assert_ne!(encode(&edited).unwrap(), encode(&grown).unwrap());
        edited.write(dug.iter().copied()).unwrap();

        // Revisions differ by design; the bricks must not.
        assert_eq!(edited.revision(), 2);
        let a = Ground {
            revision: 0,
            ..edited
        };
        let b = Ground {
            revision: 0,
            ..grown
        };
        assert_eq!(a, b);
        assert_eq!(encode(&a).unwrap(), encode(&b).unwrap());
    }

    #[test]
    fn bricks_round_trip_and_foreign_bytes_settle() {
        let grown = Ground::grow(&Hills, 12);
        let back: Ground = decode(&encode(&grown).unwrap()).unwrap();
        assert_eq!(back, grown);

        // A dense encoding of a uniform brick decodes to the uniform form.
        // Variant 1, then 512 as a varint, then the bytes.
        let mut wire = vec![1u8, 0x80, 0x04];
        wire.extend([SOIL; CELLS]);
        let brick: Brick = postcard::from_bytes(&wire).unwrap();
        assert_eq!(brick.uniform(), Some(SOIL));
        assert_eq!(postcard::to_allocvec(&brick).unwrap(), vec![0, SOIL]);

        // Short dense bytes and an out-of-palette uniform are refused.
        assert!(postcard::from_bytes::<Brick>(&[1, 3, 1, 2, 3]).is_err());
        assert!(postcard::from_bytes::<Brick>(&[0, MAX_MATERIAL + 1]).is_err());
    }
}
