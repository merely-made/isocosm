// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{SubjectKey, hybrid::HybridSettings};

/// Fixed paired RGBA8 and Depth32Float image storage, independent of the
/// terrain brick capacity. `None` means no image can be admitted.
pub(in crate::scene) fn capacity(settings: HybridSettings, limits: &wgpu::Limits) -> Option<u32> {
    let edge = settings.resolution;
    if edge < 4 || edge > limits.max_texture_dimension_2d || settings.max_refreshes == 0 {
        return None;
    }
    let slot_bytes = u64::from(edge).pow(2).checked_mul(8)?;
    let slots = (settings.max_resident_bytes / slot_bytes)
        .min(u64::from(limits.max_texture_array_layers)) as u32;
    (slots > 0).then_some(slots)
}

#[derive(Clone, Copy)]
struct Slot {
    subject: SubjectKey,
    last_used: u64,
}

/// Slots already referenced by this frame cannot be overwritten by a later
/// bake. Protect visible hits before admitting misses, then evict the least
/// recently used unprotected slot. Ties go to the lowest slot index.
pub(in crate::scene) struct Slots(Vec<Option<Slot>>);

impl Slots {
    pub(in crate::scene) fn new(capacity: u32) -> Self {
        Self(vec![None; capacity as usize])
    }

    pub(in crate::scene) fn remove(&mut self, slot: u32) {
        self.0[slot as usize] = None;
    }

    pub(in crate::scene) fn touch(&mut self, slot: u32, frame: u64) {
        self.0[slot as usize]
            .as_mut()
            .expect("resident slot")
            .last_used = frame;
    }

    pub(in crate::scene) fn admit(
        &mut self,
        subject: SubjectKey,
        frame: u64,
    ) -> Option<(u32, Option<SubjectKey>)> {
        let index = self.0.iter().position(Option::is_none).or_else(|| {
            self.0
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| {
                    slot.filter(|s| s.last_used < frame)
                        .map(|s| (index, s.last_used))
                })
                .min_by_key(|&(index, last_used)| (last_used, index))
                .map(|(index, _)| index)
        })?;
        let old = self.0[index].replace(Slot {
            subject,
            last_used: frame,
        });
        Some((index as u32, old.map(|slot| slot.subject)))
    }

    pub(in crate::scene) fn used(&self) -> u32 {
        self.0.iter().filter(|slot| slot.is_some()).count() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_draw_slots_are_protected_and_oldest_unused_slot_is_evicted() {
        let mut slots = Slots::new(2);
        assert_eq!(slots.admit(SubjectKey(1), 1), Some((0, None)));
        assert_eq!(slots.admit(SubjectKey(2), 1), Some((1, None)));
        assert_eq!(slots.admit(SubjectKey(3), 1), None);
        slots.touch(0, 2);
        assert_eq!(
            slots.admit(SubjectKey(3), 2),
            Some((1, Some(SubjectKey(2))))
        );
        assert_eq!(slots.admit(SubjectKey(4), 2), None);
        slots.touch(1, 3);
        assert_eq!(
            slots.admit(SubjectKey(4), 3),
            Some((0, Some(SubjectKey(1))))
        );
        slots.remove(1);
        assert_eq!(slots.used(), 1);
    }
}
