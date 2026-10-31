//! Distance, range and Sneak Attack under Player Core's rules (case 1047):
//! diagonals count 5 feet, then 10, alternately.

use super::draft::Build;
use super::records::Item;

/// Feet between two native cells at `scale` cells per five-foot square,
/// counting horizontal squares with the alternating diagonal.
pub fn feet(from: [i64; 3], to: [i64; 3], scale: u32) -> u64 {
    let squares = |k: usize| from[k].abs_diff(to[k]).div_ceil(u64::from(scale));
    let (x, z) = (squares(0), squares(2));
    let diagonal = x.min(z);
    let straight = x.max(z) - diagonal;
    5 * (straight + diagonal + diagonal / 2)
}

/// A melee weapon's reach in feet: 10 with the reach trait, else 5.
pub fn reach(w: &Item) -> u64 {
    if w.has("reach") { 10 } else { 5 }
}

/// Whether a melee target is within reach. A 10-foot reach also takes the
/// second square diagonally, Player Core's exception to the count.
pub fn in_reach(w: &Item, from: [i64; 3], to: [i64; 3], scale: u32) -> bool {
    let squares = |k: usize| from[k].abs_diff(to[k]).div_ceil(u64::from(scale));
    let reach = reach(w);
    feet(from, to, scale) <= reach || (reach == 10 && squares(0) <= 2 && squares(2) <= 2)
}

/// The range penalty for a ranged Strike at `distance` feet: -2 for each full
/// increment past the first, refused past the sixth; volley adds -2 inside
/// its distance.
pub fn range_penalty(w: &Item, distance: u64) -> Result<i64, String> {
    let increment = w.range.unwrap_or(0).max(1) as u64;
    let steps = distance.max(1).div_ceil(increment);
    if steps > 6 {
        return Err(format!("{distance} feet is past the {} weapon's sixth range increment", w.name));
    }
    let volley = w.traits.iter().filter_map(|t| t.strip_prefix("volley-"))
        .filter_map(|n| n.parse::<u64>().ok()).any(|near| distance <= near);
    Ok(-2 * (steps as i64 - 1) - if volley { 2 } else { 0 })
}

/// Whether a rogue's Strike with this weapon can deal Sneak Attack damage:
/// agile, finesse or ranged weapons, and for a ruffian any simple weapon up
/// to d8 or martial weapon up to d6.
pub fn sneak_weapon(b: &Build, w: &Item) -> bool {
    if !b.feats.iter().any(|f| f == "sneak-attack") {
        return false;
    }
    if w.has("agile") || w.has("finesse") || w.range.is_some() {
        return true;
    }
    let faces = w.damage.as_deref().and_then(|d| d.split('d').nth(1)).and_then(|f| f.parse::<u32>().ok()).unwrap_or(0);
    b.racket.as_deref() == Some("ruffian")
        && match w.category.as_deref() {
            Some("simple") => faces <= 8,
            Some("martial") | Some("advanced") => faces <= 6,
            _ => false,
        }
}

/// The ammunition a ranged weapon fires, if it fires any: thrown weapons
/// are themselves what flies.
pub fn ammunition(w: &Item) -> Option<&'static str> {
    if w.range.is_none() || w.traits.iter().any(|t| t.starts_with("thrown")) {
        return None;
    }
    match (w.group.as_deref(), w.id.as_str()) {
        (_, "blowgun") => Some("blowgun-darts"),
        (Some("bow"), _) => Some("arrows"),
        (Some("crossbow"), _) => Some("bolts"),
        (Some("sling"), _) => Some("sling-bullets"),
        _ => None,
    }
}

/// Flanking on a square grid: the line between the two flankers' square
/// centres crosses the target's square through opposite sides or corners.
pub fn flanks(attacker: [i64; 2], ally: [i64; 2], target: [i64; 2]) -> bool {
    let (ax, az) = (attacker[0] as f64, attacker[1] as f64);
    let (dx, dz) = ((ally[0] - attacker[0]) as f64, (ally[1] - attacker[1]) as f64);
    let (lo_x, hi_x) = (target[0] as f64 - 0.5, target[0] as f64 + 0.5);
    let (lo_z, hi_z) = (target[1] as f64 - 0.5, target[1] as f64 + 0.5);
    // Liang-Barsky clipping of the segment to the target square.
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    let mut enter = (false, false, false, false);
    let mut leave = (false, false, false, false);
    for (p, q, side) in [(-dx, ax - lo_x, 0), (dx, hi_x - ax, 1), (-dz, az - lo_z, 2), (dz, hi_z - az, 3)] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
            continue;
        }
        let r = q / p;
        if p < 0.0 {
            if r > t0 + 1e-9 { t0 = r; enter = (side == 0, side == 1, side == 2, side == 3); }
            else if (r - t0).abs() <= 1e-9 { set(&mut enter, side); }
        } else if r < t1 - 1e-9 { t1 = r; leave = (side == 0, side == 1, side == 2, side == 3); }
        else if (r - t1).abs() <= 1e-9 { set(&mut leave, side); }
    }
    if t0 >= t1 {
        return false;
    }
    (enter.0 && leave.1) || (enter.1 && leave.0) || (enter.2 && leave.3) || (enter.3 && leave.2)
}

fn set(sides: &mut (bool, bool, bool, bool), side: u8) {
    match side { 0 => sides.0 = true, 1 => sides.1 = true, 2 => sides.2 = true, _ => sides.3 = true }
}
