//! The post-dispatch tail and the session bridge.
//!
//! `after_dispatch` runs after every click, key, and drag-step, which is why
//! each pump it calls checks its cheap request flags before cloning anything.
//! `pump_net` moves events between the local state and the authority.
//!
//! Split out of `main.rs` on 2026-07-24; behavior unchanged.

use super::*;

impl App {
    /// Consume one-shot state requests (save/load) and repaint: the
    /// tail of every dispatch.
    pub(crate) fn after_dispatch(&mut self, ctx: &mut Ctx<'_>) {
        if self.family_table.is_some() {
            let mut network_processed = false;
            let mut network_error = None;
            let mut closed_pumps = Vec::new();
            let mut budget = 4;
            if let Some(table) = self.family_table.as_mut() {
                for (index, pump) in self.family_request_pumps.iter().enumerate() {
                    if budget == 0 {
                        break;
                    }
                    match table.pump_admitted_requests(pump, budget) {
                        Ok(count) => {
                            network_processed |= count > 0;
                            budget -= count;
                        },
                        Err(error) => {
                            closed_pumps.push(index);
                            network_error.get_or_insert(error);
                        },
                    }
                }
            }
            for index in closed_pumps.into_iter().rev() {
                self.family_request_pumps.remove(index);
            }
            let network_projections = if network_processed {
                self.family_table.as_mut().map(family_table::FamilyTable::projections)
            } else {
                None
            };
            let mut action = None;
            let mut legacy_events = Vec::new();
            let mut save_or_load = false;
            let mut load_requested = false;
            ctx.runner.update(|ui| {
                action = ui.take_vtt_action_request();
                legacy_events = std::mem::take(&mut ui.net_outbox);
                let save = std::mem::take(&mut ui.save_requested);
                load_requested = std::mem::take(&mut ui.load_requested);
                save_or_load = save || load_requested;
            });
            if let Some(error) = network_error {
                ctx.runner.update(|ui| {
                    ui.status = format!("Peer session ended: {error}");
                });
            }
            if let Some(result) = network_projections {
                ctx.runner.update(|ui| match result {
                    Ok(projections) => ui.set_vtt_projections(projections),
                    Err(error) => ui.status = format!("VTT projection refresh failed: {error}"),
                });
            }
            if let Some(request) = action {
                let table = self.family_table.as_mut().expect("checked above");
                let outcome = table.invoke(request);
                let projections = table.projections();
                ctx.runner.update(|ui| {
                    ui.status = match outcome {
                        Ok(chirograph::IntentResult::Accepted) => "Action accepted".into(),
                        Ok(chirograph::IntentResult::Rejected { reason }) => {
                            format!("Action refused: {reason}")
                        },
                        Ok(chirograph::IntentResult::Stale { .. }) => {
                            "That view changed before the action was applied. Review the refreshed table.".into()
                        },
                        Err(error) => format!("Action failed: {error}"),
                    };
                    match projections {
                        Ok(projections) => ui.set_vtt_projections(projections),
                        Err(error) => {
                            ui.status = format!("family projection refresh failed: {error}")
                        },
                    }
                });
            } else if !legacy_events.is_empty() {
                ctx.runner.update(|ui| {
                    ui.status = "legacy table events are disabled; use an action shown by the VTT projection".into();
                });
            }
            if save_or_load {
                ctx.runner.update(|ui| {
                    let table_status = if self.family_table.as_ref().is_some_and(|table| table.is_durable()) {
                        "Table changes are saved automatically. Private GM data remains in its separate campaign store."
                    } else {
                        "This table has no saved campaign. Changes will be lost when the app closes."
                    };
                    let load_status = if load_requested {
                        " To switch tables, restart the app with the other campaign."
                    } else {
                        ""
                    };
                    ui.status = format!("{table_status}{load_status}");
                });
            }
            return;
        }
        if let Some(board) = &mut self.scene_board {
            board.save_preferences(ctx);
        }
        // No `set_ime_allowed` here any more: the host asks `focused_text`
        // itself at the tail of every dispatch, which is the same question this
        // used to answer by hand.
        if self.profile {
            let ui = ctx.runner.state();
            eprintln!(
                "[isocosm] post-dispatch mode={:?} selected={:?} status={:?}",
                ui.mode, ui.selected, ui.status
            );
        }
        ctx.runner.update(|ui| ui.sync_overmap_source_time());
        self.pump_native_time(ctx);
        self.pump_native_table(ctx);
        self.pump_native_scene(ctx);
        self.pump_native_physical(ctx);
        self.pump_native_items(ctx);
        self.pump_native_institutions(ctx);
        self.pump_native_composed_storylet(ctx);
        self.pump_native_party(ctx);
        self.pump_native_packs(ctx);
        // Cheap flags first: this tail runs after every dispatch, and the save
        // path below starts by cloning the journal. An ordinary click asks for
        // neither and must not pay for either.
        let wants_save_or_load = {
            let ui = ctx.runner.state();
            ui.save_requested || ui.load_requested
        };
        if !wants_save_or_load {
            self.pump_selection_rows(ctx);
            self.pump_sheets(ctx);
            self.pump_generators(ctx);
            self.pump_storylets(ctx);
            self.pump_downtime(ctx);
            self.pump_overmap(ctx);
            self.pump_overmap_orders(ctx);
            self.pump_overmap_read(ctx);
            self.pump_net(ctx);
            self.pump_native_physical(ctx);
            self.pump_native_items(ctx);
            self.pump_native_institutions(ctx);
            self.pump_native_composed_storylet(ctx);
            self.pump_native_party(ctx);
            self.pump_native_packs(ctx);
            self.refresh_source_history(ctx);
            self.sync_native_choreography(ctx);
            self.refresh_native_events(ctx);
            return;
        }
        let mut save: Option<(std::path::PathBuf, String, String, GameSnapshot)> = None;
        let mut load: Option<(std::path::PathBuf, String)> = None;
        let journal = self.journal.clone();
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                if std::mem::take(&mut ui.save_requested) {
                    match serde_json::to_string_pretty(&ui.map) {
                        Ok(json) => {
                            let name = ui.map.name.clone();
                            save = Some((
                                map_path(&name),
                                json,
                                name,
                                GameSnapshot {
                                    map: ui.map.clone(),
                                    turns: ui.turns.clone(),
                                    roll_log: ui.roll_log.clone(),
                                    journal: journal.clone(),
                                    inventories: ui.inventories.clone(),
                                    generations: ui.generations.clone(),
                                    maps: ui.campaign_maps.clone(),
                                    active_map: ui.active_map.clone(),
                                    world: ui.world.clone(),
                                    native: ui.native.world.clone(),
                                    native_table: ui.native.table.clone(),
                                    clocks: ui.clocks.clone(),

                                    party_cap: ui.party_cap,
                                    last_beats: Vec::new(),
                                    beat_seq: 0,
                                    applied_actions: Default::default(),
                                },
                            ));
                        },
                        Err(e) => ui.status = format!("save failed: {e}"),
                    }
                }
                if std::mem::take(&mut ui.load_requested) {
                    load = Some((map_path(&ui.map.name), ui.map.name.clone()));
                }
            });
        }
        if let Some((path, json, name, local_public)) = save {
            let map_result =
                std::fs::create_dir_all("maps").and_then(|_| std::fs::write(&path, json));
            let campaign = self
                .net
                .as_ref()
                .and_then(NetBridge::campaign)
                .unwrap_or_else(|| self.campaign.clone());
            let public = self
                .net
                .as_ref()
                .and_then(NetBridge::latest)
                .unwrap_or(local_public);
            let history = self
                .net
                .as_ref()
                .and_then(NetBridge::history)
                .unwrap_or_else(|| self.history.clone());
            let checkpoint =
                CampaignCheckpoint::new(public, campaign, history, self.history_origin.clone());
            let campaign_result = CampaignRepository::open(campaign_path(&name))
                .and_then(|repository| repository.save_checkpoint(&checkpoint));
            {
                let runner = &mut *ctx.runner;
                runner.update(|ui| {
                    ui.status = match (map_result.as_ref(), campaign_result.as_ref()) {
                        (Ok(()), Ok(())) => format!("saved {}", path.display()),
                        (Err(error), Ok(())) => {
                            format!("checkpoint saved, map export failed: {error}")
                        },
                        (Err(error), Err(_)) => format!("map save failed: {error}"),
                        (Ok(()), Err(error)) => {
                            format!("map saved, private campaign save failed: {error}")
                        },
                    };
                });
            }
        }
        if let Some((path, name)) = load {
            let checkpoint = CampaignRepository::open(campaign_path(&name))
                .and_then(|repository| repository.load_checkpoint());
            if let Ok(Some(checkpoint)) = checkpoint {
                self.campaign = checkpoint.private;
                self.journal = checkpoint.public.journal.clone();
                self.history = checkpoint.history;
                self.history_origin = checkpoint.history_origin;
                self.source_history_len = None;
                self.source_history_attached = false;
                {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| {
                        ui.apply_snapshot(checkpoint.public);
                        ui.status = format!("loaded checkpoint {}", path.display());
                    });
                }
                self.refresh_source_history(ctx);
                return;
            }
            let checkpoint_error = checkpoint.err();
            let loaded = std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|json| {
                    serde_json::from_str::<isocosm_core::MapDocument>(&json)
                        .map_err(|e| e.to_string())
                });
            match loaded {
                Ok(map) => {
                    let name = map.name.clone();
                    let campaign = CampaignRepository::open(campaign_path(&name))
                        .and_then(|repository| repository.load_private());
                    if let Ok(campaign) = campaign.as_ref() {
                        self.campaign = campaign.clone();
                    }
                    {
                        let runner = &mut *ctx.runner;
                        runner.update(|ui| {
                            ui.replace_map(map);
                            ui.status = match (campaign, checkpoint_error) {
                                (Ok(_), None) => format!("loaded {}", path.display()),
                                (Err(error), _) => {
                                    format!("map loaded, private campaign state failed: {error}")
                                },
                                (_, Some(error)) => format!(
                                    "loaded legacy map after checkpoint read failed: {error}"
                                ),
                            };
                        });
                    }
                },
                Err(error) => {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| ui.status = format!("load failed: {error}"));
                },
            }
        }
        self.pump_selection_rows(ctx);
        self.pump_sheets(ctx);
        self.pump_generators(ctx);
        self.pump_storylets(ctx);
        self.pump_downtime(ctx);
        self.pump_overmap(ctx);
        self.pump_overmap_orders(ctx);
        self.pump_overmap_read(ctx);
        self.pump_net(ctx);
        self.pump_native_physical(ctx);
        self.pump_native_items(ctx);
        self.pump_native_institutions(ctx);
        self.pump_native_composed_storylet(ctx);
        self.pump_native_party(ctx);
        self.refresh_source_history(ctx);
        self.sync_native_choreography(ctx);
        self.refresh_native_events(ctx);
    }

    /// In networked mode: ship the UI's queued game events to the
    /// session, and pull the latest replicated snapshot into the view
    /// when the session advanced. No-op when solo.
    pub(crate) fn pump_net(&mut self, ctx: &mut Ctx<'_>) {
        if self.net.is_none() {
            return;
        }
        // Drain the outbox and submit each event.
        let mut events = Vec::new();
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| events = std::mem::take(&mut ui.net_outbox));
        }
        // Drain queued whispers (host-side) too.
        let mut whispers = Vec::new();
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| whispers = std::mem::take(&mut ui.whisper_outbox));
        }
        let mut received = Vec::new();
        let mut players = Vec::new();
        let mut endpoints = Vec::new();
        let mut campaign_outcomes = Vec::new();
        let mut failure = None;
        if let Some(net) = self.net.as_mut() {
            // Armillary keeps the network runtime off the winit kernel. Drain
            // its typed updates before reading any mirror state.
            net.poll();
            if !events.is_empty() && self.profile {
                eprintln!("[isocosm] pump: submitting {} event(s)", events.len());
            }
            for event in events {
                net.submit(event);
            }
            for (to, text) in whispers {
                net.whisper(to, text);
            }
            // Deliver received whispers into the message log, and refresh
            // the whisper-target list from connected players.
            received = net.take_whispers();
            players = net.players();
            endpoints = net.authenticated_endpoints();
            campaign_outcomes = net.take_campaign_outcomes();
            failure = net.take_failure();
        }
        if ctx.runner.state().native.endpoints != endpoints {
            ctx.runner.update(|ui| ui.native.endpoints = endpoints);
        }
        if !received.is_empty()
            || !players.is_empty()
            || !campaign_outcomes.is_empty()
            || failure.is_some()
        {
            {
                let runner = &mut *ctx.runner;
                runner.update(|ui| {
                    for (from, text) in &received {
                        ui.receive_whisper(from, text);
                    }
                    ui.connected_players = players;
                    if let Some(outcome) = campaign_outcomes.last() {
                        // Campaign drafts and storylets share this one-shot
                        // outcome channel, so the text stays neutral to fit both.
                        ui.status = match &outcome.value {
                            Ok(()) => format!("committed (request {})", outcome.request),
                            Err(error) => {
                                format!("commit failed (request {}): {error}", outcome.request)
                            },
                        };
                    }
                    if let Some(error) = &failure {
                        ui.status = error.clone();
                    }
                });
            }
        }
        // Mirror in a new snapshot when the session version bumped.
        let version = self.net.as_ref().map(|n| n.version()).unwrap_or(0);
        if version != self.last_net_version {
            self.last_net_version = version;
            let snap = self.net.as_ref().and_then(|n| n.latest());
            if let Some(snap) = snap {
                let runner = &mut *ctx.runner;
                self.journal = snap.journal.clone();
                if let Some(history) = self.net.as_ref().and_then(NetBridge::history) {
                    self.history = history;
                }
                // Pull the authoritative host-private campaign too, or storylet
                // availability (which reads secret_ids) resolves against a stale
                // copy: a mid-session reveal would leave a secret-gated storylet
                // wrongly locked, or a removed secret wrongly playable.
                if let Some(campaign) = self.net.as_ref().and_then(NetBridge::campaign) {
                    self.campaign = campaign;
                }
                runner.update(|ui| ui.apply_snapshot(snap));
                self.capture_network_receipt(runner.state(), version);
                // The host's door sweep: any token now standing on a transition
                // point of the active map walks through it. Clients never ask in
                // words; they walk, the move replicates, and this notices. The
                // emitted list keeps one crossing from being ruled twice while
                // its echo is still in flight.
                //
                // The actor rules each crossing against current authority state
                // and commits its regional consequences under the same lock.
                // Sequential commands also handle several players leaving at once.
                if self.net_is_host && runner.state().native.world.is_none() {
                    let on_doors: Vec<TokenId> = {
                        let ui = runner.state();
                        ui.map
                            .tokens
                            .iter()
                            .filter(|t| ui.transition_at(t.at))
                            .map(|t| t.id)
                            .collect()
                    };
                    let party = runner
                        .state()
                        .viewer
                        .clone()
                        .unwrap_or_else(|| "dm".to_owned());
                    self.travel_emitted.retain(|token| on_doors.contains(token));
                    if let Some(net) = self.net.as_mut() {
                        for token in &on_doors {
                            if self.travel_emitted.contains(token) {
                                continue;
                            }
                            self.own_requests += 1;
                            if net
                                .travel(*token, RequestId::host(self.own_requests), party.clone())
                                .is_none()
                            {
                                runner.update(|ui| {
                                    ui.status = "travel authority actor stopped".to_owned()
                                });
                            } else {
                                self.travel_emitted.push(*token);
                            }
                        }
                    }
                }
            }
        }
        self.refresh_native_events(ctx);
    }

    /// Evaluate or commit a host-owned generator preview. The view asks for a
    /// one-shot action; this desktop layer loads the declared pack and owns the
    /// entropy tape, while `isonetry` remains scripting-agnostic.
    /// Build a replicated snapshot from the view's current state, for a host
    /// operation that needs to prevalidate against a clone (storylet commit).
    pub(crate) fn snapshot_of(&self, ui: &UiState) -> GameSnapshot {
        snapshot_with_journal(self.journal.clone(), ui)
    }
}

/// The same build, with the journal passed in rather than read off the app.
/// The door sweep needs it while a `&mut` borrow of the runner is live, which
/// puts the whole-`self` method out of reach.
pub(crate) fn snapshot_with_journal(
    journal: Vec<isocosm_campaign::WorldFact>,
    ui: &UiState,
) -> GameSnapshot {
    GameSnapshot {
        map: ui.map.clone(),
        turns: ui.turns.clone(),
        roll_log: ui.roll_log.clone(),
        journal,
        inventories: ui.inventories.clone(),
        generations: ui.generations.clone(),
        maps: ui.campaign_maps.clone(),
        active_map: ui.active_map.clone(),
        world: ui.world.clone(),
        native: ui.native.world.clone(),
        native_table: ui.native.table.clone(),
        clocks: ui.clocks.clone(),
        party_cap: ui.party_cap,
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
    }
}
