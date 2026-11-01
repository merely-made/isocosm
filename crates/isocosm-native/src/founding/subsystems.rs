// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rules subsystems by registry key (case 1064). Each declares the decision
//! points it governs, named by the adapters' existing action groups (case
//! 1067). Subsystems that govern different points mix; two governing one
//! point are refused at founding unless a named hybrid profile settles it.
//! Every subsystem is compiled today; a pack's scripted `System` is reserved
//! and will be judged by this same registry.

use serde::{Deserialize, Serialize};

/// A decision a subsystem governs, by the action group that makes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionPoint {
    /// Making a character: `Create` and the draft's choices.
    Creation,
    /// `Give` and `Equip`.
    Equipment,
    /// `Learn`, `Cast` and `Dismiss`.
    Casting,
    /// `Attack`, `RuleAttack` and the defender's reactions.
    Attack,
    /// How a hit becomes harm: `Damage`.
    Damage,
    /// What a turn is: `StartEncounter`, `EndTurn`, `EndEncounter`.
    Encounter,
    /// `Check` and the DC it is made against.
    Check,
    /// `Stride` and its ruling.
    Movement,
    /// Healing across consented time: `BattleMedicine`, `Begin`, `Complete`.
    Healing,
    /// An edition's unit of time and what fills it (case 1068): Ars's season
    /// and its activities.
    Time,
}

/// How a subsystem runs. A pack's scripted `System` is reserved (case 1064).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Implementation {
    Compiled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subsystem {
    pub key: &'static str,
    pub governs: &'static [DecisionPoint],
    pub implementation: Implementation,
}

/// A named profile settling overlaps by naming, for each contested point,
/// the subsystem that governs it (VTT 5). None is registered yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HybridProfile {
    pub name: &'static str,
    pub settles: &'static [(DecisionPoint, &'static str)],
}

use DecisionPoint::*;

/// The compiled subsystems: each edition adapter as it handles actions today.
pub const REGISTRY: &[Subsystem] = &[
    Subsystem {
        key: "5e-srd",
        governs: &[Creation, Equipment, Casting, Attack, Damage],
        implementation: Implementation::Compiled,
    },
    Subsystem {
        key: "pf2e-srd",
        governs: &[Creation, Equipment, Attack, Damage, Encounter, Check, Movement, Healing],
        implementation: Implementation::Compiled,
    },
    Subsystem {
        key: "ars5",
        governs: &[Creation, Casting, Damage, Time],
        implementation: Implementation::Compiled,
    },
];

pub const HYBRID_PROFILES: &[HybridProfile] = &[];

pub fn subsystem(key: &str) -> Option<&'static Subsystem> {
    REGISTRY.iter().find(|s| s.key == key)
}

/// The founding-time conflict check over a campaign's declared subsystems.
pub fn check(keys: &[&str], hybrid: Option<&str>) -> Result<(), String> {
    check_in(REGISTRY, HYBRID_PROFILES, keys, hybrid)
}

fn check_in(
    registry: &[Subsystem], profiles: &[HybridProfile], keys: &[&str], hybrid: Option<&str>,
) -> Result<(), String> {
    let mut governed: Vec<(DecisionPoint, &str)> = Vec::new();
    for key in keys {
        let found = registry.iter().find(|s| s.key == *key).ok_or_else(|| {
            format!("subsystem {key} is not registered; scripted pack subsystems are reserved")
        })?;
        for point in found.governs {
            governed.push((*point, found.key));
        }
    }
    let profile = match hybrid {
        Some(name) => Some(
            profiles
                .iter()
                .find(|p| p.name == name)
                .ok_or_else(|| format!("no hybrid profile is named {name}"))?,
        ),
        None => None,
    };
    governed.sort();
    for pair in governed.windows(2) {
        let ((point, a), (next, b)) = (pair[0], pair[1]);
        if point != next || a == b {
            continue;
        }
        let settled = profile.is_some_and(|p| {
            p.settles.iter().any(|(at, by)| *at == point && (*by == a || *by == b))
        });
        if !settled {
            return Err(format!(
                "subsystems {a} and {b} both govern {point:?}; a named hybrid profile must settle it"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_overlapping_subsystems_mix_and_overlaps_refuse() {
        assert!(check(&["pf2e-srd"], None).is_ok());
        assert!(check(&["5e-srd"], None).is_ok());
        let both = check(&["5e-srd", "pf2e-srd"], None).unwrap_err();
        assert!(both.contains("both govern"), "{both}");
        assert!(check(&["5e-srd", "pf2e-srd"], Some("unregistered")).unwrap_err().contains("no hybrid"));
        assert!(check(&["house-rules"], None).unwrap_err().contains("reserved"));
    }

    const TURNS: Subsystem = Subsystem { key: "turns", governs: &[Encounter], implementation: Implementation::Compiled };
    const HARM: Subsystem = Subsystem { key: "harm", governs: &[Damage], implementation: Implementation::Compiled };
    const WOUNDS: Subsystem = Subsystem { key: "wounds", governs: &[Damage, Healing], implementation: Implementation::Compiled };
    const SETTLED: HybridProfile = HybridProfile { name: "wounds-lead", settles: &[(Damage, "wounds")] };

    #[test]
    fn a_named_profile_settles_only_the_points_it_names() {
        let registry: &[Subsystem] = &[TURNS, HARM, WOUNDS];
        assert!(check_in(registry, &[], &["turns", "harm"], None).is_ok(), "disjoint points mix");
        assert!(check_in(registry, &[], &["harm", "wounds"], None).is_err());
        assert!(check_in(registry, &[SETTLED], &["harm", "wounds"], Some("wounds-lead")).is_ok());
        let unsettled: &[HybridProfile] = &[HybridProfile { name: "other", settles: &[(Healing, "wounds")] }];
        assert!(check_in(registry, unsettled, &["harm", "wounds"], Some("other")).is_err());
    }
}
