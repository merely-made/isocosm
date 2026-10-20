use super::*;
use logos::Logos;
use taplo::syntax::SyntaxKind;

pub(super) fn preflight(text: &str, format: Format) -> Result<(), String> {
    if text.len() > 2 * 1024 * 1024 {
        return Err("authoring limit: 2 MiB".into());
    }
    if matches!(format, Format::Toml) {
        let mut depth = 0usize;
        for token in SyntaxKind::lexer(text) {
            match token {
                SyntaxKind::BRACKET_START | SyntaxKind::BRACE_START => {
                    depth += 1;
                    if depth > 64 {
                        return Err("authoring syntax nesting limit: 64".into());
                    }
                },
                SyntaxKind::BRACKET_END | SyntaxKind::BRACE_END => depth = depth.saturating_sub(1),
                _ => {},
            }
        }
    }
    Ok(())
}
pub(super) fn parse(
    text: &str,
    format: Format,
    cache: Option<&mut NodeCache>,
) -> Result<(Option<taplo::parser::Parse>, Option<String>), String> {
    if matches!(format, Format::Json) {
        return Ok((
            None,
            Some("external JSON is retained and validated; source indexing is TOML-only".into()),
        ));
    }
    let parsed = match cache {
        Some(cache) => taplo::parser::parse_with_cache(text, cache),
        None => taplo::parser::parse(text),
    };
    if parsed.clone().into_syntax().to_string() != text {
        return Err("syntax reconstruction mismatch; revision not published".into());
    }
    let note = if parsed.errors.is_empty() {
        parsed.clone().into_dom().validate().err().map(|e| {
            format!(
                "TOML source index: {:?}; product decoder remains authoritative",
                e.collect::<Vec<_>>()
            )
        })
    } else {
        Some(format!(
            "TOML source index is partial: {}; product decoder remains authoritative",
            parsed
                .errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        ))
    };
    Ok((Some(parsed), note))
}
#[derive(Serialize)]
pub(super) struct Occurrence {
    path: String,
    key_ranges: Vec<(u32, u32)>,
    node_ranges: Vec<(u32, u32)>,
}
pub(super) fn inspect(session: &Session) -> serde_json::Value {
    let snapshot = &session.current;
    let Some(parse) = &snapshot.syntax else {
        return serde_json::json!({"indexed":false,"occurrences":[]});
    };
    if snapshot.index_note.is_some() {
        return serde_json::json!({"indexed":false,"occurrences":[],"note":snapshot.index_note});
    }
    let root = parse.clone().into_syntax();
    // Fence even private navigation through the session before constructing occurrences.
    let handle = session.handle(&root);
    if session.resolve(&handle).is_err() {
        return serde_json::json!({"indexed":false,"occurrences":[]});
    }
    let dom = parse.clone().into_dom();
    let occurrences = dom
        .flat_iter()
        .map(|(keys, node)| Occurrence {
            path: keys.to_string(),
            key_ranges: keys
                .iter()
                .filter_map(|k| k.as_key())
                .flat_map(|k| k.text_ranges())
                .map(|r| (r.start().into(), r.end().into()))
                .collect(),
            node_ranges: node
                .text_ranges(false)
                .map(|r| (r.start().into(), r.end().into()))
                .collect(),
        })
        .collect::<Vec<_>>();
    serde_json::json!({"indexed":true,"revision":snapshot.revision,"occurrences":occurrences})
}
