use super::*;
use std::io::{BufRead, Write};
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    Read {
        #[serde(default)]
        text: bool,
    },
    Edit {
        base: u64,
        edits: Vec<Edit>,
    },
    Undo {
        base: u64,
        target: u64,
    },
    Check,
    Inspect,
    ProposeLockPreset {
        base: u64,
        generator: String,
        preset: Box<isometry_campaign::GeneratorLockPreset>,
    },
    InsertLockPreset {
        base: u64,
        generator: String,
        preset: Box<isometry_campaign::GeneratorLockPreset>,
    },
    RemoveLockPreset {
        base: u64,
        generator: String,
        key: String,
    },
    ReplaceLockPreset {
        base: u64,
        generator: String,
        key: String,
        preset: Box<isometry_campaign::GeneratorLockPreset>,
    },
    SetGeneratorField {
        base: u64,
        generator: String,
        field: String,
        value: serde_json::Value,
    },
    Save {
        base: u64,
    },
    Quit,
}
fn state(session: &Session, text: bool) -> serde_json::Value {
    serde_json::json!({"document":session.path,"format":session.format,"revision":session.current.revision,
        "saved_revision":session.saved_revision,"dirty":session.current.fingerprint!=session.disk_fingerprint,
        "source_fingerprint":session.current.fingerprint,"valid":session.current.validation.is_none(),
        "diagnostics":session.current.diagnostics(),"retained_revisions":session.history.len(),
        "text":text.then_some(session.current.text.as_ref()),"admitted":false})
}
/// Plan a structural edit against the current revision, then publish it.
fn structural(
    session: &mut Session,
    base: u64,
    operation: &'static str,
    plan: impl FnOnce(&Session) -> Result<(Vec<Edit>, isometry_campaign::ContentPackManifest), String>,
) -> Result<serde_json::Value, String> {
    session.check_base(base)?;
    let plan = plan(session)?;
    let mut result = structure::publish(session, base, plan, operation)?;
    result["state"] = state(session, false);
    Ok(result)
}
fn execute(session: &mut Session, command: Command) -> Result<serde_json::Value, String> {
    Ok(match command {
        Command::Read { text } => state(session, text),
        Command::Edit { base, edits } => {
            serde_json::json!({"receipt":session.edit(base, edits)?,"state":state(session,false)})
        },
        Command::Undo { base, target } => {
            serde_json::json!({"receipt":session.undo(base,target)?,"state":state(session,false)})
        },
        Command::Check => {
            let manifest = decode(session.format, &session.current.text)?;
            serde_json::json!({"revision":session.current.revision,"valid":true,"manifest":manifest,"choices":manifest.generator_choices().into_iter().map(|c|serde_json::json!({"id":c.id,"name":c.name,"default_args":c.default_args,"lock_presets":c.lock_presets})).collect::<Vec<_>>(),"admitted":false})
        },
        Command::Inspect => syntax::inspect(session),
        Command::ProposeLockPreset {
            base,
            generator,
            preset,
        } => preset::propose(session, base, &generator, *preset)?,
        Command::InsertLockPreset {
            base,
            generator,
            preset,
        } => structural(session, base, "insert_lock_preset", |s| {
            structure::insert(s, &generator, &preset)
        })?,
        Command::RemoveLockPreset {
            base,
            generator,
            key,
        } => structural(session, base, "remove_lock_preset", |s| {
            structure::remove(s, &generator, &key)
        })?,
        Command::ReplaceLockPreset {
            base,
            generator,
            key,
            preset,
        } => structural(session, base, "replace_lock_preset", |s| {
            structure::replace(s, &generator, &key, &preset)
        })?,
        Command::SetGeneratorField {
            base,
            generator,
            field,
            value,
        } => structural(session, base, "set_generator_field", |s| {
            structure::set_field(s, &generator, &field, &value)
        })?,
        Command::Save { base } => session.save(base)?,
        Command::Quit => unreachable!("handled by command loop"),
    })
}
pub(crate) fn run() -> Result<(), String> {
    const USAGE: &str =
        "usage: pack-author <pack-directory> [--check] [--history full|text|cached]";
    let mut args = std::env::args_os().skip(1);
    let root = args.next().ok_or(USAGE)?;
    let (mut check_only, mut mode) = (false, history::Mode::DEFAULT);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--check") if !check_only => check_only = true,
            Some("--history") => {
                mode = args
                    .next()
                    .and_then(|m| m.into_string().ok())
                    .ok_or(USAGE)?
                    .parse()?;
            },
            _ => return Err(USAGE.into()),
        }
    }
    let policy = history::Policy::new(mode, history::Policy::DEFAULT_LIMIT);
    let mut session = Session::open_with(std::path::Path::new(&root), policy)?;
    if check_only {
        println!("{}", execute(&mut session, Command::Check)?);
        return Ok(());
    }
    let mut out = std::io::stdout().lock();
    writeln!(
        out,
        "{}",
        serde_json::json!({"ok":true,"ready":state(&session,false)})
    )
    .map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    for line in std::io::stdin().lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let command = serde_json::from_str::<Command>(&line).map_err(|e| e.to_string());
        if matches!(command, Ok(Command::Quit)) {
            break;
        }
        let response = match command.and_then(|c| execute(&mut session, c)) {
            Ok(value) => serde_json::json!({"ok":true,"result":value}),
            Err(error) => serde_json::json!({"ok":false,"error":error}),
        };
        writeln!(out, "{response}").map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}
