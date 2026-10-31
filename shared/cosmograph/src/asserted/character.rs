// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Character's JSON retains the non-table form; its current wire includes
//! the optional field even when absent, so postcard can decode it (809).

use super::Character;
use serde::{Serialize, Serializer, ser::SerializeStruct};

impl Serialize for Character {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let table = self.table.is_some() || !serializer.is_human_readable();
        let mut value = serializer.serialize_struct("Character", if table { 6 } else { 5 })?;
        value.serialize_field("key", &self.key)?;
        value.serialize_field("name", &self.name)?;
        value.serialize_field("tags", &self.tags)?;
        value.serialize_field("faction", &self.faction)?;
        value.serialize_field("place", &self.place)?;
        if table {
            value.serialize_field("table", &self.table)?;
        }
        value.end()
    }
}
