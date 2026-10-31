// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Quantities are chosen in the real controlled session, then admitted natively.

use cambium::{el, text};
use eponym_play::{GameIntent, ItemId, identity::{SubjectId, Tick}};
use cosmograph::{projects::{DebitPreview, DebitSelection, LotQuantity}, rules::{Causation, Query}};
use super::{SessionApp, view::{Child, button}};

pub(super) struct Selection {
    subject: SubjectId,
    process: String,
    target: Option<u64>,
    meal: Option<ItemId>,
    game_tick: Tick,
    native_tick: u64,
    rows: Vec<DebitSelection>,
}

pub(super) fn panel(state: &SessionApp) -> Child {
    let model = state.model.borrow();
    let game = model.game();
    let sim = game.world().sim();
    let mut rows: Vec<Child> = vec![Box::new(el("h2", text("Stock quantities")))];
    for process in sim.genesis().rules.processes.values().filter(|process|
        process.causation == Causation::Choice
            && !process.requires.iter().any(|query| matches!(query, Query::Project { .. } | Query::ProjectDecision)))
    {
        let key = process.id.clone();
        rows.push(button(format!("Choose stocks for {}", key.split_once(':').map_or(key.as_str(), |(_, label)| label)),
            "control", move |state| state.prepare_stocks(key.clone())));
    }
    if let Some(plan) = &state.stock_selection {
        let exact = game.next_tick() == plan.game_tick && sim.state().tick == plan.native_tick
            && model.played() == plan.subject;
        rows.push(Box::new(el("p", text(if exact {
            format!("{}: choose exact quantities, then apply.", plan.process)
        } else { "The world has changed. Choose this action again before applying.".into() }))));
        for (index, row) in plan.rows.iter().enumerate() {
            rows.push(Box::new(el("h3", text(format!("{}: {}mg from {:?}",
                row.request.account, row.request.amount, row.request.source)))));
            for (_, lot) in sim.available_lots().filter(|(source, lot)|
                *source == row.request.source && lot.account == row.request.account)
            {
                let key = lot.key.clone();
                let selected = row.lots.iter().find(|portion| portion.key == key).map_or(0, |portion| portion.amount);
                rows.push(Box::new(el("p", text(format!("{}: chosen {}mg of {}mg", key, selected, lot.amount)))));
                quantity_controls(&mut rows, index, Some(key), lot.amount);
            }
            let actual = sim.stock_balance(row.request.source, &row.request.account).unwrap_or(0);
            let claimed = sim.available_lots().filter(|(source, lot)|
                *source == row.request.source && lot.account == row.request.account)
                .fold(0u64, |sum, (_, lot)| sum.saturating_add(lot.amount));
            let available = actual.saturating_sub(claimed);
            rows.push(Box::new(el("p", text(format!("Unowned: chosen {}mg of {}mg", row.unowned, available)))));
            quantity_controls(&mut rows, index, None, available);
        }
        if exact { rows.push(button("Apply selected quantities".into(), "control", |state| state.apply_stocks())); }
        rows.push(button("Cancel selection".into(), "control", |state| state.stock_selection = None));
    }
    Box::new(el("aside", rows).attr("class", "panel"))
}

fn quantity_controls(rows: &mut Vec<Child>, index: usize, key: Option<String>, available: u64) {
    for (label, delta) in [("+1mg", 1), ("+10mg", 10), ("-1mg", -1), ("-10mg", -10)] {
        let key = key.clone();
        rows.push(button(label.into(), "control", move |state|
            state.adjust_stock(index, key.as_deref(), available, delta)));
    }
}

impl SessionApp {
    pub(super) fn prepare_meal(&mut self, item: ItemId) {
        let subject = self.model.borrow().played();
        let prepared = self.model.borrow().game().prepare_owned_meal(subject, item);
        match prepared {
            Ok((process, preview)) => self.stock_preview(subject, process, Some(item.0), Some(item), preview),
            Err(error) => self.status = vec![format!("Meal refused: {error:?}")],
        }
    }
    fn prepare_stocks(&mut self, process: String) {
        let model = self.model.borrow();
        let subject = model.played();
        let target = self.project_target.map(|item| item.0)
            .or_else(|| self.selected.and_then(|(subject, _)| model.game().entity(subject)));
        let preview = model.game().prepare_stock_act(subject, &process, target);
        drop(model);
        match preview {
            Ok(preview) => self.stock_preview(subject, process, target, None, preview),
            Err(error) => self.status = vec![format!("Stock action refused: {error:?}")],
        }
    }
    fn stock_preview(&mut self, subject: SubjectId, process: String, target: Option<u64>,
        meal: Option<ItemId>, preview: DebitPreview)
    {
        if let Some(why) = preview.refusal {
            self.stock_selection = None;
            self.status = vec![format!("Stock action refused: {why}")];
            return;
        }
        let rows = preview.requests.into_iter().map(|request|
            DebitSelection { request, lots:vec![], unowned:0 }).collect();
        let model = self.model.borrow();
        let game_tick = model.game().next_tick();
        let native_tick = model.game().world().sim().state().tick;
        drop(model);
        self.stock_selection = Some(Selection { subject, process, target, meal, game_tick, native_tick, rows });
        self.status = vec!["Choose the lots and quantities for this action.".into()];
    }
    fn adjust_stock(&mut self, index: usize, key: Option<&str>, available: u64, delta: i64) {
        let Some(row) = self.stock_selection.as_mut().and_then(|plan| plan.rows.get_mut(index)) else { return; };
        let chosen = row.lots.iter().fold(row.unowned, |sum, lot| sum.saturating_add(lot.amount));
        let old = key.map_or(row.unowned, |key| row.lots.iter().find(|lot| lot.key == key).map_or(0, |lot| lot.amount));
        let room = row.request.amount.saturating_sub(chosen.saturating_sub(old));
        let amount = if delta >= 0 { old.saturating_add(delta as u64) } else { old.saturating_sub(delta.unsigned_abs()) }
            .min(available).min(room);
        if let Some(key) = key {
            row.lots.retain(|lot| lot.key != key);
            if amount > 0 { row.lots.push(LotQuantity { key:key.into(), amount }); }
        } else { row.unowned = amount; }
    }
    fn apply_stocks(&mut self) {
        let Some(plan) = self.stock_selection.take() else { return; };
        let tick = self.model.borrow().game().next_tick();
        let intent = match plan.meal {
            Some(item) => GameIntent::EatOwned { tick, subject:plan.subject, item, selections:plan.rows },
            None => GameIntent::StockAct { tick, subject:plan.subject, process:plan.process,
                target:plan.target, selections:plan.rows },
        };
        self.status = vec![match self.model.borrow_mut().apply(intent) {
            Ok(_) => "Selected stock quantities admitted.".into(),
            Err(error) => format!("Stock action refused: {error:?}"),
        }];
    }
}
