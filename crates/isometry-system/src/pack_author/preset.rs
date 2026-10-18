//! Conservative, read-only proposal. Publication and save keep their existing owners.
use super::*;
use isometry_campaign::GeneratorLockPreset;
use taplo::syntax::SyntaxKind::{ENTRY, TABLE_ARRAY_HEADER, TABLE_HEADER};

pub(super) fn propose(
    session: &Session,
    base: u64,
    generator: &str,
    preset: GeneratorLockPreset,
) -> Result<serde_json::Value, String> {
    session.check_base(base)?;
    if !matches!(session.format, Format::Toml) || session.current.index_note.is_some() {
        return Err(
            "preset insertion requires a complete TOML source index; use text edits".into(),
        );
    }
    let original = decode(session.format, &session.current.text)?;
    let matches = original
        .generators
        .iter()
        .enumerate()
        .filter(|(_, g)| g.id == generator)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let [target] = matches.as_slice() else {
        return Err("preset target must identify exactly one generator by local id".into());
    };
    if original.generators[*target]
        .lock_presets
        .iter()
        .any(|p| p.key == preset.key)
    {
        return Err("preset key already exists in target generator".into());
    }
    let mut expected = original.clone();
    expected.generators[*target]
        .lock_presets
        .push(preset.clone());
    expected.validate().map_err(|e| e.to_string())?;
    let src = session.current.text.as_ref();
    let parse = session
        .current
        .syntax
        .as_ref()
        .ok_or("no TOML source index")?;
    let mut ordinal = None;
    let mut seen = 0;
    let mut end = None;
    // Support explicit canonical array headers only. Refuse alternate layouts rather
    // than infer ownership from matching text inside strings/comments or opaque tables.
    for node in parse.clone().into_syntax().children() {
        if matches!(node.kind(), TABLE_HEADER | TABLE_ARRAY_HEADER) {
            let header = node.text().to_string();
            let compact = header
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>();
            if compact == "[[generators]]" {
                ordinal = Some(seen);
                seen += 1;
            } else if compact != "[[generators.lock_presets]]" {
                // Unknown generator/preset descendant tables could otherwise be
                // rebound to the new array element while typed decoding ignores
                // them. Decline these layouts even when typed equality would pass.
                let table: toml::Table = toml::from_str(&header)
                    .map_err(|_| "cannot classify header for safe preset insertion")?;
                if table.contains_key("generators") {
                    return Err("unsupported generator descendant header; use text edits".into());
                }
                ordinal = None;
            }
        } else if node.kind() == ENTRY && ordinal == Some(*target) {
            end = Some(usize::from(node.text_range().end()));
        }
    }
    if seen != original.generators.len() {
        return Err(
            "unsupported generator layout; expected explicit [[generators]] headers".into(),
        );
    }
    let end = end.ok_or("cannot locate target generator insertion point")?;
    // Include the complete final entry line, leaving its trailing comment attached.
    let start = src[end..].find('\n').map_or(src.len(), |n| end + n + 1);
    let newline = if src.contains("\r\n") { "\r\n" } else { "\n" };
    let key = inline(&toml::Value::String(preset.key.clone()));
    let label = inline(&toml::Value::String(preset.label.clone()));
    let value = inline(&toml::Value::try_from(&preset.value).map_err(|e| e.to_string())?);
    let text = format!(
        "{newline}[[generators.lock_presets]]{newline}key = {key}{newline}label = {label}{newline}value = {value}{newline}"
    );
    let plan = Proposal::prepare(
        session.source_stamp(),
        src,
        vec![Edit {
            start,
            end: start,
            text,
        }],
    )
    .map_err(edit_error)?;
    let preview = plan.preview();
    let proposed = Snapshot::build(base, preview.to_string(), session.format)?;
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
fn inline(value: &toml::Value) -> String {
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
