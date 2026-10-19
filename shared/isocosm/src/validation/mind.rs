// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A mind keeps strain in a strain account, and reads its needs only from
/// the member itself and its site's conditions, so a mood can never read a
/// mood and every mood reads alike across a cohort.
pub(super) fn mind(rules: &Rules) -> Result<()> {
    let Some(m) = &rules.mind else {
        return Ok(());
    };
    if rules.accounts.get(&m.strain) != Some(&AccountKind::Strain) {
        return Err(format!("{} is not a strain account", m.strain));
    }
    for need in &m.needs {
        traits(rules, &need.traits)?;
        query(rules, &need.query)?;
        let own = matches!(
            need.query,
            Query::Alive(Binding::Actor)
                | Query::Trait {
                    who: Binding::Actor,
                    ..
                }
                | Query::Account {
                    who: Binding::Actor,
                    ..
                }
                | Query::Below {
                    who: Binding::Actor,
                    ..
                }
                | Query::Age { .. }
                | Query::Condition { .. }
                | Query::Part {
                    who: Binding::Actor,
                    ..
                }
                | Query::Holds {
                    who: Binding::Actor,
                    ..
                }
                | Query::Wounded {
                    who: Binding::Actor,
                    ..
                }
                | Query::Expresses { .. }
                | Query::Routes {
                    who: Binding::Actor,
                    ..
                }
        );
        if !own {
            return Err("a need reads only its member and its site's conditions".into());
        }
    }
    traits(rules, m.bearing_traits.keys().chain(m.rise_traits.keys()))
}
