// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Pack-authored display forms (TG7). This table never enters a native
//! registry, Genesis, saved session or rule digest.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An exact id's named display forms. Missing forms disclose the id itself.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TermTable(BTreeMap<String, BTreeMap<String, String>>);

impl TermTable {
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn form<'a>(&'a self, id: &'a str, form: &str) -> &'a str {
        self.0.get(id).and_then(|forms| forms.get(form))
            .map(String::as_str).unwrap_or(id)
    }

    pub fn validate(&self) -> Result<(), String> {
        for (id, forms) in &self.0 {
            if !id.split_once(':').is_some_and(|(scope, name)| {
                !scope.trim().is_empty() && !name.trim().is_empty()
            }) || id.trim() != id {
                return Err(format!("display term has no qualified id: {id}"));
            }
            if forms.is_empty() { return Err(format!("display term {id} has no forms")); }
            for (form, words) in forms {
                if form.trim().is_empty() || form.trim() != form || words.trim().is_empty() {
                    return Err(format!("display term {id} has an empty or invalid form {form}"));
                }
            }
        }
        Ok(())
    }
}
