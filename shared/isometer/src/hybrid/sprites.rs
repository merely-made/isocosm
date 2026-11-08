// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::num::NonZeroU64;

use isometer_mesh::BodyDependencyRevision;

use crate::SubjectKey;

/// Revisions of everything baked into a local-pose image apart from the pose.
/// Use one cache per view/bake configuration, or include configuration changes
/// in these stamps. Revisions must change whenever their baked inputs change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteRevisions {
    /// Geometry dependencies, including growth, lost parts and attachments.
    pub anatomy: BodyDependencyRevision,
    /// Baked palette, tissue, equipment appearance and other image settings.
    pub appearance: u64,
    /// Facing relative to the view, including any baked camera orientation.
    pub facing: u64,
    pub lighting: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteSource {
    pub revisions: SpriteRevisions,
    /// When the input local pose was sampled, on the view's displayed-frame
    /// clock. Advance the clock even during live rendering or absence.
    pub pose_frame: u64,
}

/// Current placement and identity, supplied on every draw. Facing is already
/// baked. The caller uses the subject to address the current scene for picks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpritePlacement {
    pub subject: SubjectKey,
    pub origin: [f32; 3],
    pub scale: f32,
}

pub struct SpriteInstance<'a, Image> {
    pub placement: SpritePlacement,
    pub image: &'a Image,
}

struct Resident<Image> {
    source: SpriteSource,
    image: Image,
}

/// One local image per resident subject. The image may be owned pixels or a
/// handle that retains an external atlas entry. The caller controls residency.
pub struct SpriteCache<Image> {
    images: HashMap<SubjectKey, Resident<Image>>,
}

impl<Image> Default for SpriteCache<Image> {
    fn default() -> Self {
        Self {
            images: HashMap::new(),
        }
    }
}

impl<Image> SpriteCache<Image> {
    /// Resolve an image on entry, re-entry or a held frame. A revision change
    /// invalidates it immediately. A pose is usable while its age is strictly
    /// less than `period`, including when the selected period has shortened.
    /// `bake` must produce a local image from `source`, without the world root.
    /// If a bake is needed but the supplied pose is too old or from the future,
    /// return `None`. A failed bake leaves no reusable stale image.
    pub fn resolve<E>(
        &mut self,
        placement: SpritePlacement,
        source: SpriteSource,
        frame: u64,
        period: NonZeroU64,
        bake: impl FnOnce() -> Result<Image, E>,
    ) -> Result<Option<SpriteInstance<'_, Image>>, E> {
        let fresh = |pose_frame| {
            frame
                .checked_sub(pose_frame)
                .is_some_and(|age| age < period.get())
        };
        let subject = placement.subject;
        let hit = self.images.get(&subject).is_some_and(|entry| {
            entry.source.revisions == source.revisions && fresh(entry.source.pose_frame)
        });
        if !hit {
            self.images.remove(&subject);
            if !fresh(source.pose_frame) {
                return Ok(None);
            }
            self.images.insert(
                subject,
                Resident {
                    source,
                    image: bake()?,
                },
            );
        }
        Ok(self.images.get(&subject).map(|entry| SpriteInstance {
            placement,
            image: &entry.image,
        }))
    }

    /// Drop a subject's image on retirement, or before reusing its key.
    pub fn remove(&mut self, subject: SubjectKey) {
        self.images.remove(&subject);
    }

    /// Drop all images, for example when replacing the roster or frame clock.
    pub fn clear(&mut self) {
        self.images.clear();
    }
}
