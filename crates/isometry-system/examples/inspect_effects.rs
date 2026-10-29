//! Inspect coverage and native process declarations. Never casts or edits a world.
use isocosm::{Founding, simulation::Genesis};
use isometry_system::effect_catalogue::{self, Correspondence};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct Report {
    catalogue_revision: String,
    rules_revision: String,
    process_source: String,
    candidate_meaning: &'static str,
    catalogue: effect_catalogue::Catalogue,
    native_candidates: BTreeMap<String, Vec<String>>,
}

fn run() -> Result<(), String> {
    let mut json = false;
    let mut markdown = false;
    let mut genesis = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--markdown" => markdown = true,
            "--genesis" => genesis = Some(args.next().ok_or("--genesis needs a JSON path")?),
            _ => {
                return Err(format!(
                    "unknown argument {arg}; use [--json|--markdown] [--genesis PATH]"
                ));
            },
        }
    }
    if json && markdown {
        return Err("choose either --json or --markdown".into());
    }
    let catalogue = effect_catalogue::builtin()?;
    let (g, process_source) = match genesis {
        Some(path) => {
            let bytes = std::fs::read(&path).map_err(|e| format!("read genesis: {e}"))?;
            let g: Genesis =
                serde_json::from_slice(&bytes).map_err(|e| format!("read genesis: {e}"))?;
            g.validate()?;
            (g, "supplied native Genesis JSON".into())
        },
        None => (
            Founding {
                seed: 10,
                sites: 1,
                population: 4,
                lineages: 2,
                cohort_size: 1,
                ..Default::default()
            }
            .generate()?,
            "reference founding, seed 10, four individuals; not an inhabited save".into(),
        ),
    };
    let native_candidates = catalogue
        .entries
        .iter()
        .map(|entry| (entry.id.clone(), entry.native_candidates(&g.rules)))
        .collect();
    let report = Report {
        catalogue_revision: catalogue.revision(),
        rules_revision: g.rules.revision(),
        process_source,
        candidate_meaning: "declared primitive match only; not admission, execution or a complete edition procedure",
        catalogue,
        native_candidates,
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
    } else if markdown {
        print_markdown(&report);
    } else {
        println!(
            "{} entries in {} families; {} edition sources",
            report.catalogue.entries.len(),
            report.catalogue.families.len(),
            report.catalogue.sources.len()
        );
        let mappings = report
            .catalogue
            .entries
            .iter()
            .flat_map(|e| e.mappings.values());
        let (related, gaps) = mappings.fold((0, 0), |(related, gaps), mapping| {
            if mapping.relation == Correspondence::Related {
                (related + 1, gaps)
            } else {
                (related, gaps + 1)
            }
        });
        println!("{related} reviewed related correspondences; {gaps} explicit remaining mappings");
        let procedures: usize = report
            .catalogue
            .entries
            .iter()
            .flat_map(|e| e.mappings.values())
            .map(|m| m.procedures.len())
            .sum();
        println!(
            "{procedures} specific executable VTT procedure bindings (run the cast_effects example)"
        );
        println!("{}", report.candidate_meaning);
        for entry in &report.catalogue.entries {
            println!(
                "{} [{:?}]: {:?}",
                entry.id, entry.family, report.native_candidates[&entry.id]
            );
        }
        println!("Catalogue revision: {}", report.catalogue_revision);
    }
    Ok(())
}

fn print_markdown(report: &Report) {
    let cell = |text: &str| text.replace('|', "\\|").replace(['\n', '\r'], " ");
    println!("# Wing effect coverage\n\n{}\n", report.catalogue.scope);
    println!("Catalogue revision: `{}`.\n", report.catalogue_revision);
    println!("Native candidates: {}.\n", report.candidate_meaning);
    println!("Process source: {}.\n", report.process_source);
    println!(
        "Executable bindings name specific procedures in the system lane; native candidates remain declarations. Run `cast_effects` for execution receipts.\n"
    );
    for source in &report.catalogue.sources {
        println!("- **{}:** [{}]({}).", source.id, source.edition, source.url);
    }
    println!(
        "\n| Effect | Family | Native primitives | {} |",
        report
            .catalogue
            .sources
            .iter()
            .map(|source| cell(&source.id))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!(
        "| --- | --- | --- | {} |",
        vec!["---"; report.catalogue.sources.len()].join(" | ")
    );
    for entry in &report.catalogue.entries {
        let primitives = entry
            .native
            .iter()
            .map(|binding| format!("{:?}", binding.operation))
            .collect::<Vec<_>>()
            .join(", ");
        let mappings = report
            .catalogue
            .sources
            .iter()
            .map(|source| {
                let mapping = &entry.mappings[&source.id];
                cell(&format!(
                    "{:?}{}{}",
                    mapping.relation,
                    if mapping.references.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", mapping.references.join("; "))
                    },
                    if mapping.procedures.is_empty() {
                        String::new()
                    } else {
                        format!("; executable: {}", mapping.procedures.join(", "))
                    }
                ))
            })
            .collect::<Vec<_>>()
            .join(" | ");
        println!(
            "| {} | {:?} | {} | {} |",
            cell(&entry.id),
            entry.family,
            if primitives.is_empty() {
                "unbound"
            } else {
                &primitives
            },
            mappings
        );
    }
    println!("\n## Native scope and remaining work\n");
    for entry in &report.catalogue.entries {
        println!(
            "- **{}:** {} {}",
            entry.id,
            cell(&entry.summary),
            cell(&entry.limitation)
        );
        for source in &report.catalogue.sources {
            for difference in &entry.mappings[&source.id].differences {
                println!("  - **{}:** {}", source.id, cell(difference));
            }
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
