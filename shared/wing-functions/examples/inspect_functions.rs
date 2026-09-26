// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;
use wing_functions::{
    BodyTract, GeneratorSettings, Operator, PartRef, PreviewSettings, TractRole, WorldRules,
    generate, preview_techniques,
};

fn main() {
    let mut tracts = vec![
        BodyTract {
            part: PartRef {
                subject: 1,
                part: 0,
            },
            role: TractRole::Source,
        },
        BodyTract {
            part: PartRef {
                subject: 1,
                part: 1,
            },
            role: TractRole::Store,
        },
        BodyTract {
            part: PartRef {
                subject: 1,
                part: 2,
            },
            role: TractRole::Gate,
        },
    ];
    tracts.extend((3..7).map(|part| BodyTract {
        part: PartRef { subject: 1, part },
        role: TractRole::Actuator,
    }));
    let settings = GeneratorSettings::default();
    let rules = WorldRules {
        allowed_operators: [Operator::Strengthen, Operator::Project, Operator::Store]
            .into_iter()
            .collect(),
        allowed_costs: [10].into_iter().collect(),
        max_range: 2,
        max_hops: 4,
    };
    let mut output = Vec::new();
    for seed in 0..50_u64 {
        let batch = generate(seed, &settings, &tracts);
        let live: BTreeSet<_> = tracts.iter().map(|s| s.part).collect();
        let mut forms = Vec::new();
        for candidate in &batch.candidates {
            let before = preview_techniques(
                &candidate.blueprint,
                &live,
                &rules,
                &PreviewSettings::default(),
            );
            let mut damaged = live.clone();
            if let Some(part) = candidate.blueprint.actuators.first() {
                damaged.remove(part);
            }
            let after = preview_techniques(
                &candidate.blueprint,
                &damaged,
                &rules,
                &PreviewSettings::default(),
            );
            let encoded = serde_json::to_vec(&candidate.blueprint).unwrap();
            let restored: wing_functions::GeneratedBlueprint =
                serde_json::from_slice(&encoded).unwrap();
            forms.push(serde_json::json!({ "form": format!("{:?}", candidate.blueprint.form), "seed": candidate.blueprint.seed, "supported_before": before.iter().filter(|p| p.result.is_ok()).count(), "supported_after_part_loss": after.iter().filter(|p| p.result.is_ok()).count(), "mutation_reasons": after.iter().filter_map(|p| p.result.as_ref().err()).collect::<Vec<_>>(), "serde_continuation": restored == candidate.blueprint }));
        }
        output.push(
            serde_json::json!({ "seed": seed, "candidates": forms, "rejected": batch.rejected }),
        );
    }
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
