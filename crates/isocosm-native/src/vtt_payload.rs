// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The VTT table's payload field is JSON a person types; the endpoint reads
//! canonical CBOR (Mere engine split D231, D238). Each intent the table can
//! send with a payload is parsed into the type its decoder reads and
//! re-encoded, so a uuid typed as text crosses as the value the decoder
//! expects. An intent with no payload type here is refused.

use cosmograph_overlay::vtt::Assertion;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::native::{
    Input,
    graft::GraftInput,
    institutions::InstitutionInput,
    items::ItemInput,
    party::{LocationInput, StatementInput},
    physical_map::PhysicalMapInput,
};

fn typed<T: Serialize + DeserializeOwned>(json: &[u8]) -> Result<Vec<u8>, String> {
    let value: T = serde_json::from_slice(json)
        .map_err(|error| format!("the payload does not match this action's schema: {error}"))?;
    chirograph::wire::encode(&value).map_err(|error| error.to_string())
}

/// The canonical CBOR payload for `intent`, from the table's JSON. An empty
/// payload, which a payload-free action sends, stays empty.
///
/// # Errors
///
/// When the intent takes no typed payload here, or the JSON is not its type.
pub fn vtt_payload(intent: &str, json: &[u8]) -> Result<Vec<u8>, String> {
    if json.is_empty() {
        return Ok(Vec::new());
    }
    match intent {
        "vtt.goal-reshape" => typed::<String>(json),
        "vtt.preview-0" => typed::<InstitutionInput>(json),
        "vtt.preview-1" => typed::<PhysicalMapInput>(json),
        "vtt.preview-2" => typed::<ItemInput>(json),
        "vtt.preview-3" => typed::<GraftInput>(json),
        "vtt.preview-4" => typed::<StatementInput>(json),
        "vtt.preview-5" => typed::<LocationInput>(json),
        "vtt.apply" => typed::<Input>(json),
        "vtt.assert" => typed::<Vec<Assertion>>(json),
        "vtt.ruleset-scale" => typed::<crate::ruleset::Scale>(json),
        "vtt.ars-add-summa" => typed::<crate::ruleset::ars::NewSumma>(json),
        other => Err(format!("the table cannot send a payload for `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_intent_is_typed_and_an_unknown_one_refused() {
        let scale = br#"{"cells_per_square":4}"#;
        // Positive control: the typed payload is the canonical item of its type.
        let cbor = vtt_payload("vtt.ruleset-scale", scale).unwrap();
        let read: crate::ruleset::Scale = chirograph::wire::decode(&cbor).unwrap();
        assert_eq!(read.cells_per_square, 4);
        let error = vtt_payload("vtt.invented", scale).unwrap_err();
        assert!(error.contains("vtt.invented"), "{error}");
        assert!(
            vtt_payload("vtt.ruleset-scale", br#"{"cells":4}"#).is_err(),
            "not its schema"
        );
        assert!(
            vtt_payload("vtt.invented", b"").unwrap().is_empty(),
            "no payload, nothing to type"
        );
    }
}
