// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Event")]
struct EventJson {
    id: Key,
    tick: Tick,
    place: EventPlace,
    subject: Id,
    /// Whom the act was toward, where it had a target: a deed's other party
    /// (Eponym's world move, 755). Absent elsewhere, which serialize and
    /// hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    object: Option<Id>,
    process: Key,
    cause: Option<Key>,
    /// A deed's agreement link, distinct from its causal event (808).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agreement: Option<Id>,
    strength: u32,
    legend: bool,
    /// An authored history line's fill (769); absent elsewhere, which serialize and hash as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authored: Option<crate::asserted::HistoryLine>,
}
#[derive(Serialize, Deserialize)]
#[serde(remote = "Event")]
struct EventBinary {
    id: Key,
    tick: Tick,
    place: EventPlace,
    subject: Id,
    /// Whom the act was toward, where it had a target: a deed's other party
    /// (Eponym's world move, 755). Absent elsewhere, which serialize and
    /// hash as before.
    object: Option<Id>,
    process: Key,
    cause: Option<Key>,
    /// A deed's agreement link, distinct from its causal event (808).
    agreement: Option<Id>,
    strength: u32,
    legend: bool,
    /// An authored history line's fill (769); absent elsewhere, which serialize and hash as before.
    authored: Option<crate::asserted::HistoryLine>,
}

/// Binary native event records are explicitly v4 typed records. Every Option
/// slot is present on binary wire. Existing VTT
/// postcard envelopes carry authored campaign/GameEvent records, not this type.
/// Witness preimages stay separately numeric for genuine Site histories.
impl Serialize for Event {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() { return EventJson::serialize(self, s); }
        struct Borrowed<'a>(&'a Event);
        impl Serialize for Borrowed<'_> {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                EventBinary::serialize(self.0, s)
            }
        }
        (4u32, Borrowed(self)).serialize(s)
    }
}
impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() { return EventJson::deserialize(d); }
        #[derive(Deserialize)]
        #[serde(transparent)]
        struct Owned(#[serde(with = "EventBinary")] Event);
        let (version, Owned(event)) = <(u32, Owned)>::deserialize(d)?;
        if version != 4 { return Err(serde::de::Error::custom("native event wire version mismatch")); }
        Ok(event)
    }
}
