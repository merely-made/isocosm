// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Pack-declared presentation styles (cases 1182, 1183): tags naming looks,
//! in the shared [`StyleTable`] shape, and rules that give a body a tag by its
//! kingdom, lineage or role. Display data only, like the term table: never a
//! native registry, Genesis, saved session or rule digest.

use isometer_core::{Style, StyleTable};
use serde::{Deserialize, Serialize};

/// A body's presentation role. A closed set, so a misspelt rule is refused
/// rather than silently matching nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleRole {
    /// The critter the player controls.
    Played,
    /// A dead body still lying in the site.
    Carcass,
    /// Any other living body.
    Member,
}

impl StyleRole {
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "played" => Some(Self::Played),
            "carcass" => Some(Self::Carcass),
            "member" => Some(Self::Member),
            _ => None,
        }
    }
}

/// One `[[style_rules]]` entry. An omitted key matches any body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleRule {
    /// The declared tag a matching body takes.
    pub style: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kingdom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// What a rule reads of one body, at projection time.
#[derive(Clone, Copy, Debug)]
pub struct StyleSubject<'a> {
    pub kingdom: &'a str,
    pub lineage: &'a str,
    pub role: StyleRole,
}

/// A pack's tags and the rules that hand them out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleSheet {
    pub styles: StyleTable,
    pub rules: Vec<StyleRule>,
}

impl StyleSheet {
    /// Every rule names a declared tag, a qualified kingdom, a non-empty
    /// lineage and a known role.
    pub fn validate(&self) -> Result<(), String> {
        self.styles.validate()?;
        for rule in &self.rules {
            self.styles
                .resolve(&rule.style)
                .map_err(|unknown| format!("style rule names undeclared tag {:?}", unknown.0))?;
            let kingdom_id = |k: &str| k.strip_prefix("kingdom:").is_some_and(|n| !n.is_empty());
            if rule.kingdom.as_deref().is_some_and(|k| !kingdom_id(k)) {
                return Err(format!(
                    "style rule kingdom {:?} is not a kingdom id",
                    rule.kingdom
                ));
            }
            if rule.lineage.as_deref().is_some_and(|l| l.trim().is_empty()) {
                return Err("style rule lineage is empty".into());
            }
            if let Some(role) = &rule.role
                && StyleRole::parse(role).is_none()
            {
                return Err(format!(
                    "style rule role {role:?} is not played, carcass or member"
                ));
            }
        }
        Ok(())
    }

    /// The first matching rule's look; `None` leaves the host's default.
    pub fn resolve(&self, subject: StyleSubject<'_>) -> Option<Style> {
        let rule = self.rules.iter().find(|rule| {
            rule.kingdom.as_deref().is_none_or(|k| k == subject.kingdom)
                && rule.lineage.as_deref().is_none_or(|l| l == subject.lineage)
                && rule
                    .role
                    .as_deref()
                    .is_none_or(|r| StyleRole::parse(r) == Some(subject.role))
        })?;
        self.styles.resolve(&rule.style).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometer_core::BodyDisplay;

    #[derive(Deserialize)]
    struct Pack {
        styles: StyleTable,
        style_rules: Vec<StyleRule>,
    }

    fn sheet(toml: &str) -> StyleSheet {
        let pack: Pack = toml::from_str(toml).unwrap();
        StyleSheet {
            styles: pack.styles,
            rules: pack.style_rules,
        }
    }

    const PACK: &str = r#"
        [styles.film]
        display = "neutral"
        [styles.baseline]
        display = "srgb"
        [[style_rules]]
        style = "film"
        kingdom = "kingdom:myco"
        [[style_rules]]
        style = "baseline"
        role = "played"
    "#;

    fn body(kingdom: &str, role: StyleRole) -> StyleSubject<'_> {
        StyleSubject {
            kingdom,
            lineage: "lineage:1",
            role,
        }
    }

    #[test]
    fn a_declared_tag_resolves_to_its_style_by_kingdom_or_role() {
        let sheet = sheet(PACK);
        assert!(sheet.validate().is_ok());
        let film = sheet.resolve(body("kingdom:myco", StyleRole::Member));
        assert_eq!(film.map(|s| s.display), Some(BodyDisplay::NeutralGamma22));
        let played = sheet.resolve(body("kingdom:fauna", StyleRole::Played));
        assert_eq!(played.map(|s| s.display), Some(BodyDisplay::Srgb));
        // Control: a body no rule matches keeps the host's default.
        assert_eq!(
            sheet.resolve(body("kingdom:fauna", StyleRole::Member)),
            None
        );
    }

    #[test]
    fn an_unknown_tag_role_or_look_is_refused() {
        let unknown = sheet(&PACK.replace("style = \"film\"", "style = \"fog\""));
        assert!(
            unknown
                .validate()
                .unwrap_err()
                .contains("undeclared tag \"fog\"")
        );
        let role = sheet(&PACK.replace("role = \"played\"", "role = \"hero\""));
        assert!(role.validate().is_err());
        let look = PACK.replace("display = \"neutral\"", "display = \"sepia\"");
        assert!(toml::from_str::<Pack>(&look).is_err());
    }
}
