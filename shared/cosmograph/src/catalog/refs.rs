// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The closed vocabularies a row may cite, each checked against the code it
//! names, so a datasheet cannot claim a rule the sim does not have.

use crate::process::Registry;
use crate::rules::{EffectKind, default_functions, default_shapes};

/// The kingdom keys the code uses (`generate.rs` draws a lineage's kingdom
/// from these four; `rules/kingdoms.rs` weighs the first three; the memo's
/// section 7 item 1 types matter by them). The first three are the owner's
/// plant-like, animal-like and fungus-like kingdoms; `micro` is the fourth
/// the code holds (lineage-rounds D20) and serves the parasite model.
pub const KINGDOMS: [&str; 4] =
    ["kingdom:flora", "kingdom:fauna", "kingdom:myco", "kingdom:micro"];

/// The three kingdoms the owner named.
pub const MAIN_KINGDOMS: [&str; 3] = ["kingdom:flora", "kingdom:fauna", "kingdom:myco"];

/// What a part may admit or emit. Living matter and dead stock follow
/// `process::NisKind` and `IntakePort` (live producer, consumer, decomposer;
/// dead stock apart); the dead splits by age and kind per lineage-rounds
/// D37; the rest are the soil, light and trade goods the five models move.
pub const FLOWS: &[&str] = &[
    "light",
    "water",
    "soil:nutrient",
    "soil:scarce",
    "sugar",
    "nectar",
    "pollen",
    "seed",
    "spore",
    "toxin",
    "exudate",
    "shade",
    "signal",
    "pathogen",
    "live:producer",
    "live:consumer",
    "live:decomposer",
    "live:tough",
    "dead:fresh",
    "dead:aged",
    "dead:toxic",
    "dead:litter",
    "dead:dung",
    "dead:wood",
    "host:tissue",
    "host:matter",
];

/// Code modules a row may cite as `rule:<name>`, each with its source file
/// under `src/` (a test checks the file exists).
pub const RULES: &[(&str, &str)] = &[
    ("hosting", "rules/hosting.rs"),
    ("dormancy", "rules/dormancy.rs"),
    ("climate", "rules/climate.rs"),
    ("competition", "rules/competition.rs"),
    ("defence", "rules/defence.rs"),
    ("exertion", "rules/exertion.rs"),
    ("pace", "rules/pace.rs"),
    ("senses", "senses.rs"),
    ("light", "light.rs"),
    ("rot", "ecology/rot.rs"),
    ("spoilage", "ecology/spoilage.rs"),
    ("categories", "ecology/categories.rs"),
    ("guard", "ecology/guard.rs"),
    ("web", "ecology/web.rs"),
    ("limits", "ecology/limits.rs"),
    ("port-admission", "mosaic/ports/admission.rs"),
    ("healing", "harm/healing.rs"),
    ("feeding", "rules/feeding.rs"),
];

/// The namespace a native process lives in (`process/registry.rs`).
pub const NATIVE_NAMESPACE: &str = "mesocosm";
/// The namespace a proposed, not yet executable process is named in.
pub const PROPOSED_NAMESPACE: &str = "proposed";

/// Whether `reference` names something that exists: `function:<n>` in the
/// default catalogue and the native registry, `kind:<n>` an `EffectKind`,
/// `rule:<n>` one of [`RULES`].
pub fn resolves(reference: &str) -> bool {
    let Some((scheme, name)) = reference.split_once(':') else {
        return false;
    };
    match scheme {
        "function" => {
            default_functions().contains_key(&format!("function:{name}"))
                && native_process(name)
        }
        "kind" => serde_json::from_value::<EffectKind>(serde_json::Value::String(name.into())).is_ok(),
        "rule" => RULES.iter().any(|(rule, _)| *rule == name),
        _ => false,
    }
}

/// Whether the native registry holds `mesocosm:<name>`.
pub fn native_process(name: &str) -> bool {
    Registry::native().all().any(|def| def.id.namespace == NATIVE_NAMESPACE && def.id.name == name)
}

/// The function name a process string names, whichever namespace.
pub fn process_name(process: &str) -> &str {
    process.split_once(':').map_or(process, |(_, name)| name)
}

pub fn is_proposed(process: &str) -> bool {
    process.split_once(':').is_some_and(|(namespace, _)| namespace == PROPOSED_NAMESPACE)
}

/// Whether `shape` is one of the eight shapes the rules adopt.
pub fn shape_exists(shape: &str) -> bool {
    default_shapes().contains(shape)
}

/// Whether the native function `process` (`mesocosm:<fn>`) fits `shape`;
/// proposed processes fit any real shape.
pub fn fits(process: &str, shape: &str) -> bool {
    if is_proposed(process) {
        return shape_exists(shape);
    }
    default_functions()
        .get(&format!("function:{}", process_name(process)))
        .is_some_and(|f| f.shapes.contains(shape))
}
