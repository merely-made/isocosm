// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Quick-start worlds (cases 1023, 1041): the picker lists the packs the sim
//! carries and those the content-pack folders offer, and founds the chosen
//! one in a single host commit.

use super::*;
use isocosm_views::QuickstartChoice;

/// Every quick-start world on offer: carried first, then each content pack's.
/// A datasheet that will not read or parse is left out with a diagnostic.
pub(crate) fn choices(catalog: &GeneratorCatalog) -> Vec<QuickstartChoice> {
    let mut out: Vec<QuickstartChoice> = cosmograph::quickstart::PACKS.iter()
        .filter_map(|(_, text)| cosmograph::quickstart::parse(text).ok()
            .map(|pack| QuickstartChoice { name: pack.name, source: "carried".into(), text: (*text).into() }))
        .collect();
    for (pack, path) in catalog.quickstarts() {
        let read = std::fs::read_to_string(&path).map_err(|e| e.to_string())
            .and_then(|text| cosmograph::quickstart::parse(&text).map(|p| (p, text)));
        match read {
            Ok((parsed, text)) => out.push(QuickstartChoice { name: parsed.name, source: pack, text }),
            Err(why) => eprintln!("[isocosm] quick-start {} skipped: {why}", path.display()),
        }
    }
    out
}

impl App {
    pub(crate) fn pump_quickstart(&mut self, ctx: &mut Ctx<'_>) {
        let mut chosen = None;
        ctx.runner.update(|ui| chosen = ui.native.quickstart_request.take()
            .and_then(|index| ui.native.quickstarts.get(index).cloned()));
        let Some(choice) = chosen else { return };
        // A seed nobody chose: the table's generator tape (Method 1).
        let seed = self.generation_tape.draw();
        let mode = cosmograph::Execution::Individuals;
        if let Some(net) = self.net.as_mut() {
            let request = net.commit_native_quickstart(choice.text.clone(), seed, mode);
            ctx.runner.update(|ui| ui.status = match request {
                Some(request) => format!("founding {} (request {request})", choice.name),
                None => "campaign authority actor stopped".into(),
            });
            return;
        }
        let snapshot = self.snapshot_of(ctx.runner.state());
        let mut host = HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
        match host.commit_native_quickstart(&choice.text, seed, mode) {
            Ok(_) => {
                self.campaign = host.campaign().clone();
                self.history = host.history().clone();
                self.journal = host.state().journal.clone();
                let snapshot = host.state().clone();
                ctx.runner.update(|ui| {
                    ui.apply_snapshot(snapshot);
                    ui.status = format!("founded {} under seed {seed}", choice.name);
                });
            },
            Err(why) => ctx.runner.update(|ui| ui.status = format!("{} was not founded: {why}", choice.name)),
        }
    }
}
