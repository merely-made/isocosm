//! The retained table downtime surface. Its rows grant no faction acts.
//! The entropy turn is retired; native time requires unanimous consent.
//!
//! The selected move renders through Cambium's catalog `summary_body`, Isocosm's
//! first use of a catalog composition here: a titled record (the verb) with the
//! faction as its eyebrow and the move's narration as its description.

use cambium::{clickable, el, summary_body, text, SummaryBody};

use crate::board::UiChild;
use crate::state::UiState;

pub fn downtime_overlay(ui: &UiState) -> Option<UiChild> {
    if !ui.downtime_open {
        return None;
    }
    if ui.native.world.is_some() { return native_downtime(ui); }
    let actions: Vec<UiChild> = vec![
        Box::new(clickable(
            el::<_, UiState, ()>("span", text("roll")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.reroll_downtime(),
        )),
        Box::new(clickable(
            el::<_, UiState, ()>("span", text("commit")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.commit_downtime(),
        )),
        Box::new(clickable(
            el::<_, UiState, ()>("span", text("close")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.close_downtime(),
        )),
    ];

    let mut body: Vec<UiChild> = Vec::new();
    if ui.faction_moves.is_empty() {
        body.push(Box::new(
            el("div", text("no faction stirred this tick")).attr("class", "side-hint"),
        ));
    } else {
        let total = ui.faction_moves.len();
        let kept = ui.faction_moves.iter().filter(|m| !m.struck).count();

        // The selector line cycles through the batch, marking each move kept or
        // struck, so the DM edits without leaving the one row.
        let selector = ui
            .selected_downtime_move()
            .map(|row| {
                let mark = if row.struck { "\u{2013}" } else { "\u{2022}" }; // – struck / • kept
                format!(
                    "Move {}/{total}: {mark} {} {}",
                    ui.downtime_selected + 1,
                    row.faction,
                    row.verb
                )
            })
            .unwrap_or_else(|| "Move".to_owned());
        body.push(Box::new(clickable(
            el("div", text(selector)).attr("class", "btn"),
            |ui: &mut UiState, _| ui.cycle_downtime(),
        )));

        if let Some(row) = ui.selected_downtime_move() {
            let summary = SummaryBody::new(
                format!("downtime-move-{}", ui.downtime_selected),
                row.verb.clone(),
            )
            .with_eyebrow(row.faction.clone())
            .with_description(row.text.clone())
            .with_fact(
                "effect",
                if row.has_change {
                    "reshapes the world"
                } else {
                    "a rumor only"
                },
            );
            body.push(Box::new(summary_body::<UiState, ()>(&summary)));

            let toggle_label = if row.struck {
                "keep this move"
            } else {
                "strike this move"
            };
            body.push(Box::new(clickable(
                el("div", text(toggle_label)).attr("class", "btn"),
                |ui: &mut UiState, _| ui.toggle_strike_downtime(),
            )));
        }

        body.push(Box::new(
            el("div", text(format!("{kept} of {total} kept"))).attr("class", "side-line"),
        ));
    }

    Some(crate::widgets::overlay_panel(
        "downtime",
        "Downtime".to_owned(),
        actions,
        body,
    ))
}

fn native_downtime(ui: &UiState) -> Option<UiChild> {
    use cosmograph_overlay::vtt::TimeIntent;
    let actions: Vec<UiChild> = vec![Box::new(clickable(
        el::<_, UiState, ()>("span", text("close")).attr("class", "btn btn-mini"),
        |ui: &mut UiState, _| ui.close_downtime()))];
    let mut body: Vec<UiChild> = Vec::new();
    let read = || -> Result<_, String> {
        let campaign = ui.native.world.as_ref().ok_or("no native world")?.load(cosmograph::Execution::Grouped)?;
        let table = ui.native.table.as_ref().ok_or("admit explicit player bindings before native downtime")?;
        table.validate_history(campaign.table_session())?;
        let game = isocosm_native::native::GameBindings::decode(&table.game)?;
        Ok((campaign, table.time_roster()?, game))
    };
    match read() {
        Err(reason) => body.push(Box::new(el("div", text(reason)).attr("class", "side-hint"))),
        Ok((campaign, roster, game)) => {
            let session = campaign.table_session();
            body.push(Box::new(el("div", text(format!("native tick {} · roster {:?} · revision {}",
                session.sim.state().tick, roster.players, roster.revision))).attr("class", "side-line")));
            if ui.can_edit_inventory {
                for ticks in [1, 10] {
                    body.push(Box::new(clickable(el("div", text(format!("propose {ticks} native ticks"))).attr("class", "btn"),
                        move |ui: &mut UiState, _| ui.request_native_time(TimeIntent::Downtime { ticks }))));
                }
            } else {
                body.push(Box::new(clickable(el("div", text("release my hold for current and future proposals")).attr("class", "btn"),
                    |ui: &mut UiState, _| ui.request_native_time(TimeIntent::ReleaseHold))));
                body.push(Box::new(clickable(el("div", text("take my hold back")).attr("class", "btn"),
                    |ui: &mut UiState, _| ui.request_native_time(TimeIntent::TakeHold))));
            }
            for (key, proposal) in game.downtime() {
                let current = proposal.require_current(session.sim.state().tick, &roster).is_ok();
                let ready = current && isocosm_native::native::time::ready(session, proposal, &roster, game.time_holds())
                    .is_ok_and(|ready| ready.is_some());
                let status = if let Some(done) = &proposal.completed { format!("advanced {} → {}", done.from, done.to) }
                    else if !current { "stale; declare a new proposal".into() }
                    else if ready { "ready for the DM's choice".into() }
                    else { "waiting for individual yeses or released holds".into() };
                body.push(Box::new(el("div", text(format!("{key}: {} ticks · {status}", proposal.declaration.ticks)))
                    .attr("class", "side-line")));
                if current && (!ui.can_edit_inventory || ready) {
                    let downtime = proposal.witness.event;
                    let intent = if ui.can_edit_inventory { TimeIntent::Advance { downtime } } else { TimeIntent::Consent { downtime } };
                    let label = if ui.can_edit_inventory { "advance this ready proposal" } else { "give my yes to this proposal" };
                    body.push(Box::new(clickable(el("div", text(label)).attr("class", "btn"),
                        move |ui: &mut UiState, _| ui.request_native_time(intent))));
                }
            }
        },
    }
    Some(crate::widgets::overlay_panel("downtime", "Native downtime".into(), actions, body))
}
