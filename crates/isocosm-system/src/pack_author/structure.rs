//! Structure-aware edits by generator id and preset key. Ownership comes from
//! the syntax tree; each command computes text edits, requires that its preview
//! decodes to exactly the intended manifest, and publishes through the shared
//! proposal path with the usual base check and receipt. Layouts it cannot prove
//! (alternate headers, generator sub-tables, inline preset arrays) refuse.
use super::*;
use isocosm_campaign::{ContentPackManifest, GeneratorLockPreset};
use taplo::syntax::{
    SyntaxKind::{
        COMMENT, ENTRY, KEY, NEWLINE, TABLE_ARRAY_HEADER, TABLE_HEADER, VALUE, WHITESPACE,
    },
    SyntaxNode,
};

pub(super) struct Block {
    header: SyntaxNode,
    entries: Vec<SyntaxNode>,
}
pub(super) struct Generator {
    header: SyntaxNode,
    entries: Vec<SyntaxNode>,
    presets: Vec<Block>,
}
pub(super) struct Layout {
    pub(super) manifest: ContentPackManifest,
    generators: Vec<Generator>,
    target: usize,
}
enum Context {
    Other,
    Generator,
    Preset,
}
pub(super) fn layout(session: &Session, generator: &str) -> Result<Layout, String> {
    if !matches!(session.format, Format::Toml) || session.current.index_note.is_some() {
        return Err(
            "structure-aware edits require a complete TOML source index; use text edits".into(),
        );
    }
    let manifest = decode(session.format, &session.current.text)?;
    let matches = manifest.generators.iter().enumerate();
    let matches = matches.filter(|(_, g)| g.id == generator).map(|(i, _)| i);
    let [target] = matches.collect::<Vec<_>>()[..] else {
        return Err("target must identify exactly one generator by local id".into());
    };
    let parse = session
        .current
        .syntax
        .as_ref()
        .ok_or("no TOML source index")?;
    let mut generators: Vec<Generator> = vec![];
    let mut context = Context::Other;
    // Explicit canonical array headers only; never infer ownership from text
    // inside strings/comments or from tables typed decoding would ignore.
    for node in parse.clone().into_syntax().children() {
        if matches!(node.kind(), TABLE_HEADER | TABLE_ARRAY_HEADER) {
            let compact = node.text().to_string().replace(char::is_whitespace, "");
            if compact == "[[generators]]" {
                generators.push(Generator {
                    header: node,
                    entries: vec![],
                    presets: vec![],
                });
                context = Context::Generator;
            } else if compact == "[[generators.lock_presets]]" {
                let owner = generators
                    .last_mut()
                    .ok_or("preset header before any generator")?;
                owner.presets.push(Block {
                    header: node,
                    entries: vec![],
                });
                context = Context::Preset;
            } else {
                // Unknown generator/preset descendant tables could be rebound to a
                // new array element while typed decoding ignores them.
                let table: toml::Table = toml::from_str(&node.text().to_string())
                    .map_err(|_| "cannot classify header for a safe structural edit")?;
                if table.contains_key("generators") {
                    return Err("unsupported generator descendant header; use text edits".into());
                }
                context = Context::Other;
            }
        } else if node.kind() == ENTRY {
            match (&context, generators.last_mut()) {
                (Context::Generator, Some(g)) => g.entries.push(node),
                (Context::Preset, Some(g)) => g.presets.last_mut().unwrap().entries.push(node),
                _ => {},
            }
        }
    }
    if generators.len() != manifest.generators.len() {
        return Err(
            "unsupported generator layout; expected explicit [[generators]] headers".into(),
        );
    }
    Ok(Layout {
        manifest,
        generators,
        target,
    })
}
impl Layout {
    fn generator(&self) -> &Generator {
        &self.generators[self.target]
    }
    fn expected(
        &self,
        change: impl FnOnce(&mut isocosm_campaign::GeneratorEntry),
    ) -> Result<ContentPackManifest, String> {
        let mut expected = self.manifest.clone();
        change(&mut expected.generators[self.target]);
        expected.validate().map_err(|e| e.to_string())?;
        Ok(expected)
    }
    /// The target's preset block for `key`, when every preset is an explicit block.
    fn preset(&self, key: &str) -> Result<(usize, &Block), String> {
        let presets = &self.manifest.generators[self.target].lock_presets;
        if self.generator().presets.len() != presets.len() {
            return Err(
                "unsupported preset layout; expected [[generators.lock_presets]] headers".into(),
            );
        }
        let index = presets
            .iter()
            .position(|p| p.key == key)
            .ok_or("no preset with that key in the target generator")?;
        Ok((index, &self.generator().presets[index]))
    }
}
fn end(node: &SyntaxNode) -> usize {
    usize::from(node.text_range().end())
}
fn start(node: &SyntaxNode) -> usize {
    usize::from(node.text_range().start())
}
/// The byte after the line containing `offset`, keeping a trailing comment attached.
fn line_end(src: &str, offset: usize) -> usize {
    src[offset..]
        .find('\n')
        .map_or(src.len(), |n| offset + n + 1)
}
fn newline(src: &str) -> &'static str {
    if src.contains("\r\n") { "\r\n" } else { "\n" }
}
/// A bare, single-segment entry key; quoted and dotted keys never match.
fn entry_key(entry: &SyntaxNode) -> Option<String> {
    let key = entry
        .children()
        .find(|n| n.kind() == KEY)?
        .text()
        .to_string();
    let key = key.trim();
    let bare = !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    bare.then(|| key.to_string())
}
/// The value's own tokens: taplo attaches trailing whitespace and a trailing
/// comment to the VALUE node, and those stay.
fn value_range(entry: &SyntaxNode) -> Option<(usize, usize)> {
    let value = entry.children().find(|n| n.kind() == VALUE)?;
    let significant = value
        .children_with_tokens()
        .filter(|e| !matches!(e.kind(), WHITESPACE | COMMENT | NEWLINE))
        .map(|e| e.text_range())
        .collect::<Vec<_>>();
    Some((
        usize::from(significant.first()?.start()),
        usize::from(significant.last()?.end()),
    ))
}
fn block_text(nl: &str, preset: &GeneratorLockPreset) -> Result<String, String> {
    let key = preset::inline(&toml::Value::String(preset.key.clone()));
    let label = preset::inline(&toml::Value::String(preset.label.clone()));
    let value = preset::inline(&toml::Value::try_from(&preset.value).map_err(|e| e.to_string())?);
    Ok(format!(
        "{nl}[[generators.lock_presets]]{nl}key = {key}{nl}label = {label}{nl}value = {value}{nl}"
    ))
}

/// Edits appending `preset` after the target generator's last entry.
pub(super) fn insert(
    session: &Session,
    generator: &str,
    preset: &GeneratorLockPreset,
) -> Result<(Vec<Edit>, ContentPackManifest), String> {
    let layout = layout(session, generator)?;
    if layout.manifest.generators[layout.target]
        .lock_presets
        .iter()
        .any(|p| p.key == preset.key)
    {
        return Err("preset key already exists in target generator".into());
    }
    let expected = layout.expected(|g| g.lock_presets.push(preset.clone()))?;
    let g = layout.generator();
    let last = g
        .entries
        .iter()
        .chain(g.presets.iter().flat_map(|b| &b.entries))
        .map(end)
        .max();
    let src = session.current.text.as_ref();
    let at = line_end(
        src,
        last.ok_or("cannot locate target generator insertion point")?,
    );
    let text = block_text(newline(src), preset)?;
    Ok((
        vec![Edit {
            start: at,
            end: at,
            text,
        }],
        expected,
    ))
}
/// Edits removing the preset block for `key`, with the blank line `insert` adds.
pub(super) fn remove(
    session: &Session,
    generator: &str,
    key: &str,
) -> Result<(Vec<Edit>, ContentPackManifest), String> {
    let layout = layout(session, generator)?;
    let (index, block) = layout.preset(key)?;
    let expected = layout.expected(|g| {
        g.lock_presets.remove(index);
    })?;
    let src = session.current.text.as_ref();
    let header = start(&block.header);
    let line = src[..header].rfind('\n').map_or(0, |n| n + 1);
    let mut from = if src[line..header].trim().is_empty() {
        line
    } else {
        header
    };
    for blank in ["\r\n\r\n", "\n\n"] {
        if src[..from].ends_with(blank) {
            from -= blank.len() / 2;
            break;
        }
    }
    let last = block
        .entries
        .iter()
        .map(end)
        .max()
        .unwrap_or(end(&block.header));
    Ok((
        vec![Edit {
            start: from,
            end: line_end(src, last),
            text: String::new(),
        }],
        expected,
    ))
}
/// Edits replacing the `key`, `label` and `value` values in place; comments and
/// any other content in the block stay.
pub(super) fn replace(
    session: &Session,
    generator: &str,
    key: &str,
    preset: &GeneratorLockPreset,
) -> Result<(Vec<Edit>, ContentPackManifest), String> {
    let layout = layout(session, generator)?;
    let (index, block) = layout.preset(key)?;
    let expected = layout.expected(|g| g.lock_presets[index] = preset.clone())?;
    let values = [
        ("key", toml::Value::String(preset.key.clone())),
        ("label", toml::Value::String(preset.label.clone())),
        (
            "value",
            toml::Value::try_from(&preset.value).map_err(|e| e.to_string())?,
        ),
    ];
    let mut edits = vec![];
    for (field, value) in values {
        let entries = block
            .entries
            .iter()
            .filter(|e| entry_key(e).as_deref() == Some(field));
        let [entry] = entries.collect::<Vec<_>>()[..] else {
            return Err(format!(
                "preset block must have one bare `{field}` entry; use text edits"
            ));
        };
        let (start, end) = value_range(entry).ok_or("preset entry has no value")?;
        edits.push(Edit {
            start,
            end,
            text: preset::inline(&value),
        });
    }
    Ok((edits, expected))
}
/// Fields set by `set_generator_field`; presets have their own commands.
const FIELDS: [&str; 5] = ["id", "name", "default_args", "script", "fixtures"];
/// Edits setting a generator field: replace its value, or add `field = value`
/// after the generator's own entries.
pub(super) fn set_field(
    session: &Session,
    generator: &str,
    field: &str,
    value: &serde_json::Value,
) -> Result<(Vec<Edit>, ContentPackManifest), String> {
    if !FIELDS.contains(&field) {
        return Err(format!(
            "settable generator fields are {}",
            FIELDS.join(", ")
        ));
    }
    let layout = layout(session, generator)?;
    let mut typed = serde_json::to_value(&layout.manifest.generators[layout.target])
        .map_err(|e| e.to_string())?;
    typed[field] = value.clone();
    let typed = serde_json::from_value(typed).map_err(|e| format!("invalid {field}: {e}"))?;
    let expected = layout.expected(|g| *g = typed)?;
    let toml_value =
        toml::Value::try_from(value).map_err(|_| format!("{field} has no TOML form"))?;
    let text = preset::inline(&toml_value);
    let g = layout.generator();
    let entries = g
        .entries
        .iter()
        .filter(|e| entry_key(e).as_deref() == Some(field))
        .collect::<Vec<_>>();
    let src = session.current.text.as_ref();
    let edit = match entries[..] {
        [entry] => {
            let (start, end) = value_range(entry).ok_or("generator entry has no value")?;
            Edit { start, end, text }
        },
        [] => {
            let after = g.entries.iter().map(end).max().unwrap_or(end(&g.header));
            let at = line_end(src, after);
            let nl = newline(src);
            // A final line without a newline gets one before the new entry.
            let lead = if at == src.len() && !src.ends_with('\n') {
                nl
            } else {
                ""
            };
            Edit {
                start: at,
                end: at,
                text: format!("{lead}{field} = {text}{nl}"),
            }
        },
        _ => {
            return Err(format!(
                "generator has several `{field}` entries; use text edits"
            ));
        },
    };
    Ok((vec![edit], expected))
}
/// Publish structural edits only when their preview has a complete index and
/// decodes to exactly `expected`.
pub(super) fn publish(
    session: &mut Session,
    base: u64,
    (edits, expected): (Vec<Edit>, ContentPackManifest),
    operation: &'static str,
) -> Result<serde_json::Value, String> {
    session.check_base(base)?;
    let stamp = session.source_stamp();
    let plan =
        Proposal::prepare(stamp.clone(), &session.current.text, edits).map_err(edit_error)?;
    let proposed = Snapshot::build(base, plan.preview().to_string(), session.format, None)?;
    if proposed.index_note.is_some() || decode(session.format, plan.preview())? != expected {
        return Err(
            "unsupported layout or value; the edit would not produce the intended manifest".into(),
        );
    }
    let edits = plan.edits().to_vec();
    let application = plan
        .application(&stamp, &session.current.text)
        .map_err(edit_error)?;
    let receipt = session.publish(application, operation)?;
    Ok(serde_json::json!({"receipt":receipt,"edits":edits}))
}
