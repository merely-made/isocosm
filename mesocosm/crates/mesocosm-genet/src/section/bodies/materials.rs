// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Product meanings for the existing five neutral tissue channels.
use isocosm::attention::spatial::PartAllocation;
use isocosm::schema::PartId;
use isometer::render::PartMaterial;
use std::collections::BTreeMap;

pub(crate) fn function_materials(parts: &BTreeMap<PartId, PartAllocation>) -> Vec<PartMaterial> {
    let mut out = Vec::new();
    for (&part, allocation) in parts {
        if allocation.capacity == 0 {
            continue;
        }
        let mut counts = [0u64; 5];
        for (function, &cells) in &allocation.cells {
            let channel = match function.as_str() {
                "function:contract" | "function:support" | "function:grip"
                | "function:adhesion" => 0,
                "function:intake" | "function:excrete" => 1,
                "function:sense" | "function:conduct" | "function:gate" | "function:circulate" => 2,
                "function:fix" | "function:respire" => 3,
                "function:store" | "function:reproduce" | "function:secrete" => 4,
                _ => continue,
            };
            counts[channel] += u64::from(cells);
        }
        for (channel, count) in counts.into_iter().enumerate().filter(|(_, n)| *n > 0) {
            out.push(PartMaterial {
                part,
                material: channel as u8,
                fraction: count as f32 / allocation.capacity as f32,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_native_functions_share_the_existing_channels() {
        let input = BTreeMap::from([(
            PartId(4),
            PartAllocation {
                capacity: 8,
                cells: BTreeMap::from([
                    ("function:intake".into(), 4),
                    ("function:store".into(), 2),
                    ("function:reproduce".into(), 1),
                ]),
            },
        )]);
        let output = function_materials(&input);
        assert_eq!(output.len(), 2);
        assert_eq!(output[0].fraction, 0.5);
        assert_eq!(output[1].fraction, 0.375);
        assert!(function_materials(&BTreeMap::new()).is_empty());
    }
}
