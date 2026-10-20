//! Conservative, read-only preset proposal; `structure` owns the layout rules.
//! Publication and save keep their existing owners.
use super::*;
use isometry_campaign::GeneratorLockPreset;

pub(super) fn propose(
    session: &Session,
    base: u64,
    generator: &str,
    preset: GeneratorLockPreset,
) -> Result<serde_json::Value, String> {
    session.check_base(base)?;
    let (edits, expected) = super::structure::insert(session, generator, &preset)?;
    let src = session.current.text.as_ref();
    let plan = Proposal::prepare(session.source_stamp(), src, edits).map_err(edit_error)?;
    let preview = plan.preview();
    let proposed = Snapshot::build(base, preview.to_string(), session.format, None)?;
    if proposed.index_note.is_some() || decode(session.format, preview)? != expected {
        return Err(
            "unsupported insertion layout or value; proposed text does not match intended manifest"
                .into(),
        );
    }
    Ok(serde_json::json!({
        "document":session.path, "base":base, "generator":generator, "preset":preset,
        "edits":plan.edits(), "preview":preview,
        "valid":true, "applied":false, "admitted":false,
    }))
}

// JSON basic-string escapes are also TOML basic-string escapes. Always quote
// strings on one physical line, avoiding multiline/EOL changes inside values.
pub(super) fn inline(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => serde_json::to_string(s).expect("string serialization"),
        toml::Value::Array(a) => {
            format!("[{}]", a.iter().map(inline).collect::<Vec<_>>().join(", "))
        },
        toml::Value::Table(t) => format!(
            "{{ {} }}",
            t.iter()
                .map(|(key, value)| format!(
                    "{} = {}",
                    serde_json::to_string(key).expect("key serialization"),
                    inline(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => other.to_string(),
    }
}
