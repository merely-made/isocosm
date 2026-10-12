// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Id, Journey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Human-readable site records remain numeric; binary wire has explicit tags.
/// Native legacy witnesses use their separate preimage, never these wire tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPlace { Site(Id), Journey(Journey) }

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum JsonPlace { Site(Id), Journey(Journey) }
#[derive(Serialize, Deserialize)]
enum BinaryPlace { Site(Id), Journey(Journey) }

impl EventPlace {
    pub fn site(self) -> Option<Id> {
        match self { Self::Site(id) => Some(id), Self::Journey(_) => None }
    }
}
impl From<Id> for EventPlace {
    fn from(id: Id) -> Self { Self::Site(id) }
}
impl Serialize for EventPlace {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            match *self { Self::Site(id) => JsonPlace::Site(id),
                Self::Journey(j) => JsonPlace::Journey(j) }.serialize(s)
        } else {
            match *self { Self::Site(id) => BinaryPlace::Site(id),
                Self::Journey(j) => BinaryPlace::Journey(j) }.serialize(s)
        }
    }
}
impl<'de> Deserialize<'de> for EventPlace {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(if d.is_human_readable() {
            match JsonPlace::deserialize(d)? { JsonPlace::Site(id) => Self::Site(id),
                JsonPlace::Journey(j) => Self::Journey(j) }
        } else {
            match BinaryPlace::deserialize(d)? { BinaryPlace::Site(id) => Self::Site(id),
                BinaryPlace::Journey(j) => Self::Journey(j) }
        })
    }
}
