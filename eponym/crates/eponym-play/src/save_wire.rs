// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! GameSave anatomy decoding selects its actual founding wire, not a guessed
//! current document. This is a codec only: all restore/hash guards still run.

mod body;
mod intents;

use crate::{GameIntent, GameSave, WorldSave};
use serde::{Deserialize, Deserializer, Serialize, Serializer,
    de::{DeserializeSeed, SeqAccess, Visitor}, ser::SerializeStruct};
use std::fmt;

struct Intents { legacy:bool }
impl<'de> DeserializeSeed<'de> for Intents {
    type Value = Vec<GameIntent>;
    fn deserialize<D:Deserializer<'de>>(self, d:D) -> Result<Self::Value,D::Error> {
        if !self.legacy { return Vec::<GameIntent>::deserialize(d); }
        Vec::<intents::Wire<body::Body>>::deserialize(d)?.into_iter().map(|i|
            i.map(body::Body::native).map(|i| i.into_game()).map_err(serde::de::Error::custom))
            .collect()
    }
}

impl Serialize for GameSave {
    fn serialize<S:Serializer>(&self, s:S) -> Result<S::Ok,S::Error> {
        let mut record = s.serialize_struct("GameSave", 4)?;
        record.serialize_field("version", &self.version)?;
        record.serialize_field("world", &self.world)?;
        record.serialize_field("expected_hash", &self.expected_hash)?;
        if self.world.generator_version == 1 {
            let intents = self.intents.iter().cloned().map(|i|
                intents::Wire::from_game(i).map(|b| body::Body::old(&b)))
                .collect::<Result<Vec<_>,_>>().map_err(serde::ser::Error::custom)?;
            record.serialize_field("intents", &intents)?;
        } else {
            record.serialize_field("intents", &self.intents)?;
        }
        record.end()
    }
}

impl<'de> Deserialize<'de> for GameSave {
    fn deserialize<D:Deserializer<'de>>(d:D) -> Result<Self,D::Error> {
        if d.is_human_readable() {
            #[derive(Deserialize)]
            struct Human { version:u32, world:WorldSave, expected_hash:u64, intents:serde_json::Value }
            let h = Human::deserialize(d)?;
            let intents = if h.world.generator_version == 1 {
                serde_json::from_value::<Vec<intents::Wire<body::Body>>>(h.intents)
                    .map_err(serde::de::Error::custom)?.into_iter().map(|i|
                        i.map(body::Body::native).map(|i| i.into_game()))
                    .collect::<Result<_,_>>().map_err(serde::de::Error::custom)?
            } else {
                serde_json::from_value(h.intents).map_err(serde::de::Error::custom)?
            };
            return Ok(GameSave { version:h.version, world:h.world, expected_hash:h.expected_hash, intents });
        }
        struct Record;
        impl<'de> Visitor<'de> for Record {
            type Value = GameSave;
            fn expecting(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("four GameSave fields") }
            fn visit_seq<A:SeqAccess<'de>>(self, mut a:A) -> Result<GameSave,A::Error> {
                let version = a.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0,&self))?;
                let world:WorldSave = a.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(1,&self))?;
                let expected_hash = a.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(2,&self))?;
                let intents = a.next_element_seed(Intents { legacy:world.generator_version == 1 })?
                    .ok_or_else(|| serde::de::Error::invalid_length(3,&self))?;
                Ok(GameSave { version, world, expected_hash, intents })
            }
        }
        d.deserialize_struct("GameSave", &["version","world","expected_hash","intents"], Record)
    }
}

#[cfg(test)]
mod tests;
