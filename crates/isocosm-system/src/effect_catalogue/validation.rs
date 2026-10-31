use super::*;

impl Catalogue {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != SCHEMA_VERSION {
            return Err(format!(
                "unsupported effect catalogue version {}",
                self.version
            ));
        }
        nonempty(&self.scope, "catalogue scope")?;
        let families: BTreeSet<_> = self.families.iter().copied().collect();
        if families.is_empty() || families.len() != self.families.len() {
            return Err("catalogue families must be nonempty and unique".into());
        }
        let mut sources = BTreeSet::new();
        for source in &self.sources {
            identity(&source.id)?;
            if !sources.insert(source.id.as_str()) {
                return Err(format!("duplicate catalogue source {}", source.id));
            }
            nonempty(&source.edition, "source edition")?;
            nonempty(&source.corpus, "source corpus")?;
            if !source.url.starts_with("https://") || source.url.len() <= 8 {
                return Err(format!("source {} needs an HTTPS reference", source.id));
            }
        }
        if sources.is_empty() {
            return Err("effect catalogue has no source editions".into());
        }
        let mut ids = BTreeSet::new();
        for entry in &self.entries {
            identity(&entry.id)?;
            if !ids.insert(entry.id.as_str()) {
                return Err(format!("duplicate effect identity {}", entry.id));
            }
            if !families.contains(&entry.family) {
                return Err(format!("{} names an undeclared family", entry.id));
            }
            nonempty(&entry.summary, "effect summary")?;
            nonempty(&entry.limitation, "effect limitation")?;
            let mapped: BTreeSet<_> = entry.mappings.keys().map(String::as_str).collect();
            if mapped != sources {
                return Err(format!(
                    "{} must map every declared edition, including explicit gaps",
                    entry.id
                ));
            }
            for (source, mapping) in &entry.mappings {
                if mapping.relation == Correspondence::Related && mapping.references.is_empty() {
                    return Err(format!(
                        "{} / {source}: related mapping needs a source reference",
                        entry.id
                    ));
                }
                if mapping.relation != Correspondence::Unmapped && mapping.differences.is_empty() {
                    return Err(format!(
                        "{} / {source}: correspondence needs its differences or adjudication",
                        entry.id
                    ));
                }
                for reference in mapping.references.iter().chain(&mapping.differences) {
                    nonempty(reference, "mapping reference or difference")?;
                }
                let mut procedures = BTreeSet::new();
                for procedure in &mapping.procedures {
                    identity(procedure)?;
                    if mapping.relation != Correspondence::Related || !procedures.insert(procedure)
                    {
                        return Err("executable bindings require unique procedures and a reviewed correspondence".into());
                    }
                }
            }
            let mut native = BTreeSet::new();
            for binding in &entry.native {
                if !native.insert((binding.operation, binding.function.as_deref())) {
                    return Err(format!("{} repeats a native binding", entry.id));
                }
                if let Some(function) = &binding.function {
                    identity(function)?;
                    if binding.operation != EffectKind::Allocate {
                        return Err(format!(
                            "{}: function qualifier requires native allocation",
                            entry.id
                        ));
                    }
                }
            }
        }
        let covered: BTreeSet<_> = self.entries.iter().map(|entry| entry.family).collect();
        if covered != families {
            return Err("declared catalogue coverage includes an empty family".into());
        }
        Ok(())
    }

    /// Resolve executable references separately from structural admission, so
    /// externally authored catalogues can supply their own procedure registry.
    pub fn validate_procedures(
        &self,
        definitions: &crate::active_effects::Definitions,
    ) -> Result<(), String> {
        use crate::active_effects::{Edition, Kind};
        for entry in &self.entries {
            for (source, mapping) in &entry.mappings {
                for id in &mapping.procedures {
                    let def = definitions.get(id)?;
                    let expected = match (source.as_str(), entry.id.as_str()) {
                        ("dnd5-srd51", _) => Edition::Dnd5,
                        ("pf2-player-core", _) => Edition::Pathfinder2,
                        ("ars5", _) => Edition::Ars5,
                        _ => {
                            return Err(
                                "executable correspondence has an unknown source edition".into()
                            );
                        },
                    };
                    let kind = match entry.id.as_str() {
                        "wing:detect" => Kind::Sight,
                        "wing:resize" => Kind::Size,
                        "wing:enchant" => Kind::Weapon,
                        _ => {
                            return Err("procedure bound to an unrelated catalogue operation".into());
                        },
                    };
                    if def.edition != expected || def.kind != kind {
                        return Err(
                            "procedure correspondence disagrees with edition or operation".into(),
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

fn nonempty(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("empty {label}"))
    } else {
        Ok(())
    }
}

fn identity(value: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_:/.".contains(&b))
    {
        Ok(())
    } else {
        Err(format!("invalid effect catalogue identity {value:?}"))
    }
}
