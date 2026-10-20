//! App-local authoring session. No runtime catalogue or session mutation.
mod cli;
mod lease;
mod preset;
mod save;
mod syntax;
#[cfg(test)]
mod tests;

use isometry_campaign::ContentPackManifest;
use isometry_system::GeneratorPack;
use mere_source_edit::{
    Application, ChangedRange, EditError, Proposal, SourceStamp, TextEdit as Edit,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use taplo::rowan::ast::SyntaxNodePtr;

pub(super) use cli::run;
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Toml,
    Json,
}
#[derive(Clone, Debug, Serialize)]
struct Diagnostic {
    revision: u64,
    phase: &'static str,
    message: String,
}
#[derive(Debug)]
struct Snapshot {
    revision: u64,
    text: Arc<str>,
    fingerprint: String,
    syntax: Option<taplo::parser::Parse>,
    index_note: Option<String>,
    validation: Option<String>,
}
impl Snapshot {
    fn build(revision: u64, text: String, format: Format) -> Result<Arc<Self>, String> {
        syntax::preflight(&text, format)?;
        let (syntax, index_note) = syntax::parse(&text, format)?;
        let validation = decode(format, &text).err();
        Ok(Arc::new(Self {
            revision,
            fingerprint: fingerprint(text.as_bytes()),
            text: Arc::from(text),
            syntax,
            index_note,
            validation,
        }))
    }
    fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut out = vec![];
        if let Some(message) = &self.validation {
            out.push(Diagnostic {
                revision: self.revision,
                phase: "validation",
                message: message.clone(),
            });
        }
        if let Some(message) = &self.index_note {
            out.push(Diagnostic {
                revision: self.revision,
                phase: "source-index",
                message: message.clone(),
            });
        }
        out
    }
}
// Same decoders/schema/validator as GeneratorPack::load; no DOM-based admission gate.
fn decode(format: Format, text: &str) -> Result<ContentPackManifest, String> {
    let manifest: ContentPackManifest = match format {
        Format::Toml => toml::from_str(text).map_err(|e| e.to_string())?,
        Format::Json => serde_json::from_str(text).map_err(|e| e.to_string())?,
    };
    manifest.validate().map_err(|e| e.to_string())?;
    Ok(manifest)
}
fn fingerprint(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn edit_error(error: EditError) -> String {
    match error {
        EditError::RevisionExhausted => "revision overflow".into(),
        other => other.to_string(),
    }
}
#[derive(Debug, Serialize)]
struct Receipt {
    document: String,
    base: u64,
    result: u64,
    operation: &'static str,
    ranges: Vec<ChangedRange>,
}
struct Handle {
    document: PathBuf,
    instance: u64,
    revision: u64,
    ptr: SyntaxNodePtr<taplo::syntax::Lang>,
}
struct Session {
    path: PathBuf,
    instance: u64,
    format: Format,
    current: Arc<Snapshot>,
    history: BTreeMap<u64, Arc<Snapshot>>,
    limit: usize,
    disk_fingerprint: String,
    saved_revision: u64,
}
static INSTANCES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
impl Session {
    fn open(root: &std::path::Path, limit: usize) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("open pack root: {e}"))?;
        if root.join(GeneratorPack::MANIFEST_FILE).is_file()
            && root.join(GeneratorPack::MANIFEST_FILE_JSON).is_file()
        {
            return Err("a pack has one manifest; both TOML and JSON are present".into());
        }
        let path = GeneratorPack::manifest_path(&root).ok_or("no content-pack manifest")?;
        let format = if path.extension().and_then(|x| x.to_str()) == Some("toml") {
            Format::Toml
        } else {
            Format::Json
        };
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let text = String::from_utf8(bytes).map_err(|_| "authoring requires UTF-8 input")?;
        let current = Snapshot::build(1, text, format)?;
        Ok(Self {
            disk_fingerprint: current.fingerprint.clone(),
            history: BTreeMap::from([(1, current.clone())]),
            current,
            path,
            format,
            limit: limit.clamp(1, 256),
            saved_revision: 1,
            instance: INSTANCES.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        })
    }
    fn snapshot(&self, revision: u64) -> Result<Arc<Snapshot>, String> {
        self.history
            .get(&revision)
            .cloned()
            .ok_or("revision no longer retained".into())
    }
    fn check_base(&self, base: u64) -> Result<(), String> {
        if base != self.current.revision {
            Err("stale base revision".into())
        } else {
            Ok(())
        }
    }
    fn publish(
        &mut self,
        application: Application,
        operation: &'static str,
    ) -> Result<Receipt, String> {
        let base = application.receipt.base.revision;
        let revision = application.receipt.result.revision;
        let next = Snapshot::build(revision, application.text, self.format)?;
        let receipt = Receipt {
            document: self.path.display().to_string(),
            base,
            result: revision,
            operation,
            ranges: application.receipt.ranges,
        };
        self.current = next.clone();
        self.history.insert(revision, next);
        while self.history.len() > self.limit {
            self.history.pop_first();
        }
        Ok(receipt)
    }
    fn source_stamp(&self) -> SourceStamp {
        SourceStamp {
            document: self.path.display().to_string(),
            instance: self.instance,
            revision: self.current.revision,
        }
    }
    fn edit(&mut self, base: u64, edits: Vec<Edit>) -> Result<Receipt, String> {
        self.check_base(base)?;
        self.apply_edits(edits, "edit")
    }
    fn apply_edits(
        &mut self,
        edits: Vec<Edit>,
        operation: &'static str,
    ) -> Result<Receipt, String> {
        let stamp = self.source_stamp();
        let proposal =
            Proposal::prepare(stamp.clone(), &self.current.text, edits).map_err(edit_error)?;
        let application = proposal
            .application(&stamp, &self.current.text)
            .map_err(edit_error)?;
        // The host still parses, checks its limits and publishes atomically. The
        // flat CLI receipt and save/undo ownership remain app-local.
        self.publish(application, operation)
    }
    fn undo(&mut self, base: u64, target: u64) -> Result<Receipt, String> {
        self.check_base(base)?;
        let snapshot = self.snapshot(target)?;
        self.apply_edits(
            vec![Edit {
                start: 0,
                end: self.current.text.len(),
                text: snapshot.text.to_string(),
            }],
            "undo",
        )
    }
    fn handle(&self, node: &taplo::syntax::SyntaxNode) -> Handle {
        Handle {
            document: self.path.clone(),
            instance: self.instance,
            revision: self.current.revision,
            ptr: SyntaxNodePtr::new(node),
        }
    }
    fn resolve(&self, handle: &Handle) -> Result<taplo::syntax::SyntaxNode, String> {
        if handle.document != self.path
            || handle.instance != self.instance
            || handle.revision != self.current.revision
        {
            return Err("wrong document instance or stale syntax handle".into());
        }
        let parse = self.current.syntax.as_ref().ok_or("no TOML syntax index")?;
        handle
            .ptr
            .try_to_node(&parse.clone().into_syntax())
            .ok_or("invalid node pointer".into())
    }
}
