// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<P: Palette + PartialEq + Clone> ContentPack<P> {
    /// Enlarges structural shapes by an integer factor in 1..=3.
    ///
    /// This transforms the supplied pack, so callers select each size from
    /// their unchanged base pack, rather than repeatedly enlarging a result.
    /// Sensory detail retains its original size: Sensor is an absolute small
    /// shape in core classification, not a role that survives uniform scaling.
    /// Zero-thickness axes stay zero. Shapes whose zero axes would change
    /// their core role when enlarged retain their original envelope too.
    /// Tissue mass is independently allocated
    /// by ordinary development; the new envelopes also set its capacity and
    /// footing. Factor one preserves every byte and address.
    pub fn resized(&self, size: u8) -> Result<Self, ContentError> {
        if !(1..=3).contains(&size) {
            return Err(ContentError::InvalidSize { found: size });
        }
        self.validate()?;
        if size == 1 {
            return Ok(self.clone());
        }
        let mut result = self.clone();
        for entry in &mut result.entries {
            if entry.role == Role::Sensor {
                continue;
            }
            let mut half_extent = entry.half_extent;
            for extent in &mut half_extent {
                *extent =
                    extent
                        .checked_mul(i32::from(size))
                        .ok_or(ContentError::InvalidExtent {
                            half_extent: entry.half_extent,
                        })?;
            }
            // Core classifies zero axes as one for its aspect comparison.
            // Multiplying a flat Mass [2, 0, 2] would therefore turn it into
            // a Plate. Preserve that admitted asset instead of silently
            // changing its biological meaning or inventing thickness.
            if classify(half_extent) != entry.role {
                continue;
            }
            let dimensions = envelope(half_extent)?;
            let count = volume_len(dimensions)?;
            if count > MAX_VOLUME_VOXELS {
                return Err(ContentError::TooManyVoxels { found: count });
            }
            let mut volume = Volume::empty(dimensions);
            for z in 0..dimensions[2] {
                for y in 0..dimensions[1] {
                    for x in 0..dimensions[0] {
                        let point = [x, y, z];
                        // Endpoint-preserving nearest neighbour expands each
                        // occupied source voxel into a contiguous block and
                        // preserves the source material rather than re-carving.
                        let source: [u32; 3] = std::array::from_fn(|axis| {
                            let denominator = dimensions[axis].saturating_sub(1).max(1);
                            (point[axis] * (entry.volume.size[axis] - 1) + denominator / 2)
                                / denominator
                        });
                        volume.set(x, y, z, entry.volume.get(source[0], source[1], source[2]));
                    }
                }
            }
            entry.half_extent = half_extent;
            entry.reference =
                content_ref(result.version, entry.role, entry.slot, half_extent, &volume);
            entry.volume = volume;
            result.palette.admit(
                entry.role,
                entry.slot,
                Shape {
                    volume: entry.reference,
                    half_extent,
                },
            );
        }
        result.validate()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::tests::FixturePalette;

    #[test]
    fn unchanged_and_invalid_sizes_preserve_the_source() {
        let base = ContentPack::generate(FixturePalette::base()).unwrap();
        assert_eq!(base.resized(1).unwrap(), base);
        for size in [0, 4, u8::MAX] {
            assert_eq!(
                base.resized(size),
                Err(ContentError::InvalidSize { found: size })
            );
        }
        assert!(base.validate().is_ok());
    }

    #[test]
    fn growth_resizes_resolved_geometry_and_preserves_roles_and_materials() {
        let base = ContentPack::generate(FixturePalette::base()).unwrap();
        for size in [2, 3] {
            let enlarged = base.resized(size).unwrap();
            let restored: ContentPack<FixturePalette> =
                postcard::from_bytes(&postcard::to_allocvec(&enlarged).unwrap()).unwrap();
            assert_eq!(restored, enlarged);
            assert!(restored.resolve_for(enlarged.palette).is_ok());
            for (before, after) in base.entries.iter().zip(&enlarged.entries) {
                assert_eq!(classify(after.half_extent), before.role);
                assert_eq!(
                    after.volume.size,
                    after.half_extent.map(|half| (half * 2).max(1) as u32)
                );
                assert_eq!(
                    enlarged
                        .palette
                        .template_at(after.role, after.slot)
                        .half_extent,
                    after.half_extent
                );
                let before_materials: BTreeSet<_> =
                    before.volume.clone().into_voxels().into_iter().collect();
                let after_materials: BTreeSet<_> =
                    after.volume.clone().into_voxels().into_iter().collect();
                assert_eq!(before_materials, after_materials);
                let requested_extent = before.half_extent.map(|half| half * i32::from(size));
                if before.role == Role::Sensor || classify(requested_extent) != before.role {
                    assert_eq!(before, after);
                } else {
                    assert_eq!(after.half_extent, requested_extent);
                    assert_ne!(before.reference, after.reference);
                    assert!(after.volume.solid_count() >= before.volume.solid_count());
                }
            }
        }
    }

    /// A core with two limbs flush on its flanks, shaped by `palette`.
    fn shaped(palette: &FixturePalette) -> isometer_core::BodyDocument {
        use isometer_core::fixtures::limb;
        let core = palette.template_at(Role::Mass, 0);
        let rod = palette.template_at(Role::Limb, 0);
        let mut body = isometer_core::BodyDocument::new(core.volume, core.half_extent);
        let root = body.root;
        let reach = core.half_extent[0] + rod.half_extent[0];
        for side in [1, -1] {
            limb(&mut body, root, 2, rod.half_extent, [side * reach, 0, 0]);
        }
        body
    }

    #[test]
    fn a_resized_palette_moves_bounds_and_keeps_the_tree() {
        // Isometer's half of what Mesocosm's development pinned: the
        // resized envelopes are the bounds a body is shaped by, and the part
        // tree does not change with them. Paid mass is the sim's (699).
        let base = ContentPack::generate(FixturePalette::base()).unwrap();
        let larger = base.resized(2).unwrap();
        let (small, big) = (shaped(&base.palette), shaped(&larger.palette));
        assert_ne!(small.aabb(), big.aabb());
        assert_eq!(small.parts.len(), big.parts.len());
        for (small, big) in small.parts.iter().zip(&big.parts) {
            assert_eq!(small.id, big.id);
            assert_eq!(
                small.attachment.map(|a| a.parent),
                big.attachment.map(|a| a.parent)
            );
        }
    }
}
