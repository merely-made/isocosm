// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Print the same paired observations that the native specimen bench displays.
fn main() -> Result<(), String> {
    let nudge = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [choice] if choice == "nudge" => true,
        [choice] if choice == "alone" => false,
        _ => return Err("choose nudge or alone".into()),
    };
    let mut observations = Vec::new();
    mesocosm_runtime::refuge::compare(nudge, |pair| {
        observations.push(pair);
        true
    })?;
    serde_json::to_writer(std::io::stdout().lock(), &observations).map_err(|why| why.to_string())
}
