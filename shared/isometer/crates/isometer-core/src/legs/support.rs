// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Integer convex bearing polygons. Contact cells and mass remain caller-owned.
pub type Point = [i64; 2];
fn cross(a: Point, b: Point, c: Point) -> i128 {
    (i128::from(b[0]) - i128::from(a[0])) * (i128::from(c[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(c[0]) - i128::from(a[0]))
}
pub fn hull(mut points: Vec<Point>) -> Vec<Point> {
    points.sort();
    points.dedup();
    if points.len() < 3 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for &p in &points {
        while out.len() >= 2 && cross(out[out.len() - 2], out[out.len() - 1], p) <= 0 {
            out.pop();
        }
        out.push(p);
    }
    let lower = out.len();
    for &p in points.iter().rev().skip(1) {
        while out.len() > lower && cross(out[out.len() - 2], out[out.len() - 1], p) <= 0 {
            out.pop();
        }
        out.push(p);
    }
    if out.len() < 4 { Vec::new() } else { out }
}
pub fn contains(poly: &[Point], p: Point) -> bool {
    poly.len() >= 4 && poly.windows(2).all(|e| cross(e[0], e[1], p) >= 0)
}
/// Closest boundary point nudged toward the polygon's interior. An already
/// supported centre stays put; the margin is a fixed-point distance.
pub fn balance(poly: &[Point], p: Point, margin: i64) -> Option<Point> {
    if poly.len() < 4 {
        return None;
    }
    if contains(poly, p) {
        return Some(p);
    }
    let mut best = None;
    for e in poly.windows(2) {
        let d = [
            i128::from(e[1][0]) - i128::from(e[0][0]),
            i128::from(e[1][1]) - i128::from(e[0][1]),
        ];
        let length = d[0] * d[0] + d[1] * d[1];
        if length == 0 {
            continue;
        }
        let along = ((i128::from(p[0]) - i128::from(e[0][0])) * d[0]
            + (i128::from(p[1]) - i128::from(e[0][1])) * d[1])
            .clamp(0, length);
        let q = [0, 1].map(|k| (i128::from(e[0][k]) + d[k] * along / length) as i64);
        let distance = (0..2)
            .map(|k| (i128::from(q[k]) - i128::from(p[k])).pow(2))
            .sum::<i128>();
        if best.is_none_or(|(old, _)| distance < old) {
            best = Some((distance, q));
        }
    }
    let q = best?.1;
    let n = poly.len() - 1;
    let centre = [0, 1]
        .map(|k| (poly[..n].iter().map(|p| i128::from(p[k])).sum::<i128>() / n as i128) as i64);
    let d = [centre[0] - q[0], centre[1] - q[1]];
    let length = ((i128::from(d[0]).pow(2) + i128::from(d[1]).pow(2)) as u128).isqrt() as i64;
    if length == 0 {
        return Some(centre);
    }
    let m = margin.max(1).min(length);
    let target =
        [0, 1].map(|k| q[k] + (i128::from(d[k]) * i128::from(m) / i128::from(length)) as i64);
    Some(if contains(poly, target) {
        target
    } else {
        centre
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn balance_stays_on_real_support_and_rejects_degenerate_contacts() {
        let h = hull(vec![
            [0, 0],
            [1024, 0],
            [1024, 1024],
            [0, 1024],
            [0, 0],
            [512, 512],
        ]);
        assert!(contains(&h, [512, 512]));
        assert_eq!(balance(&h, [512, 512], 128), Some([512, 512]));
        let p = balance(&h, [-2048, 512], 128).unwrap();
        assert!(p[0] > 0 && contains(&h, p));
        assert!(hull(vec![[0, 0], [1, 0], [2, 0]]).is_empty());
        assert!(!contains(&[], [0, 0]));
    }
}
