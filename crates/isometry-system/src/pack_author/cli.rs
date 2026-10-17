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
        Command::Save { base } => session.save(base)?,
        Command::Quit => unreachable!("handled by command loop"),
    })
}
pub(crate) fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let root = args
        .next()
        .ok_or("usage: pack-author <pack-directory> [--check]")?;
    let check_only = match args.next() {
        None => false,
        Some(arg) if arg == "--check" => true,
        _ => return Err("usage: pack-author <pack-directory> [--check]".into()),
    };
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut session = Session::open(std::path::Path::new(&root), 32)?;
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
