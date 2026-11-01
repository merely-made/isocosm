//! Ars Magica's arithmetic: experience to scores, study from a summa, and
//! Wounds by Size. Numbers that vary come from the pack.

use super::records::Records;

/// Experience an Ability needs for `score`: 5 for 1, 15 for 2, 30 for 3.
pub fn ability_xp(score: u32) -> u32 {
    5 * score * (score + 1) / 2
}

/// Experience an Art needs for `score`: 1, 3, 6, 10...
pub fn art_xp(score: u32) -> u32 {
    score * (score + 1) / 2
}

/// The score `xp` reaches under `cost`.
pub fn score(xp: u32, cost: fn(u32) -> u32) -> u32 {
    (0..).find(|n| cost(n + 1) > xp).unwrap_or(0)
}

/// Experience put in with an Affinity counts half again, rounded up.
pub fn with_affinity(xp: u32, affinity: bool) -> u32 {
    if affinity { xp + xp.div_ceil(2) } else { xp }
}

/// The least experience to spend so that, with any Affinity, at least
/// `needed` arrives.
pub fn spend_for(needed: u32, affinity: bool) -> u32 {
    (0..=needed).find(|s| with_affinity(*s, affinity) >= needed).unwrap_or(needed)
}

/// One season with a summa: the experience gained, never past its Level.
/// `None` when the summa cannot teach this character anything.
pub fn summa_gain(xp: u32, level: u32, quality: u32, affinity: bool, cost: fn(u32) -> u32) -> Option<u32> {
    let cap = cost(level);
    if xp >= cap {
        return None;
    }
    Some(with_affinity(quality, affinity).min(cap - xp))
}

/// The Wound a Damage Total past Soak inflicts at `size`, as an index into
/// the pack's order (light up to dead); `None` when nothing gets through.
pub fn wound(r: &Records, excess: i64, size: i64) -> Option<usize> {
    if excess <= 0 {
        return None;
    }
    let band = (r.wounds.first_band + size).max(1);
    let level = usize::try_from((excess - 1) / band).unwrap_or(usize::MAX);
    Some(level.min(r.wounds.order.len() - 1))
}

/// The highest Ability score allowed at creation for `age`.
pub fn ability_cap(r: &Records, age: u32, affinity: bool) -> u32 {
    let cap = r.rules.ability_caps.iter().find(|[under, _]| age < *under).map_or(5, |[_, cap]| *cap);
    cap + if affinity { r.rules.affinity_cap_bonus } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ars::records::builtin;

    #[test]
    fn scores_follow_the_experience_tables() {
        assert_eq!((ability_xp(1), ability_xp(2), ability_xp(5)), (5, 15, 75));
        assert_eq!((art_xp(1), art_xp(5), art_xp(10)), (1, 15, 55));
        assert_eq!(score(75, ability_xp), 5);
        assert_eq!(score(74, ability_xp), 4);
        assert_eq!(score(15, art_xp), 5);
        assert_eq!(with_affinity(5, true), 8);
        assert_eq!(spend_for(10, true), 7);
    }

    #[test]
    fn a_summa_teaches_its_quality_up_to_its_level() {
        assert_eq!(summa_gain(0, 5, 10, false, art_xp), Some(10));
        assert_eq!(summa_gain(10, 5, 10, false, art_xp), Some(5), "stops at the Level");
        assert_eq!(summa_gain(15, 5, 10, false, art_xp), None);
        assert_eq!(summa_gain(0, 10, 10, true, art_xp), Some(15));
    }

    #[test]
    fn wounds_band_by_size() {
        let r = builtin();
        assert_eq!(wound(r, 0, 0), None);
        assert_eq!(wound(r, 5, 0), Some(0));
        assert_eq!(wound(r, 6, 0), Some(1));
        assert_eq!(wound(r, 16, 0), Some(3));
        assert_eq!(wound(r, 21, 0), Some(4));
        assert_eq!(wound(r, 5, -1), Some(1), "Size -1 moves every boundary down one");
        assert_eq!(wound(r, 99, 0), Some(4));
    }
}
