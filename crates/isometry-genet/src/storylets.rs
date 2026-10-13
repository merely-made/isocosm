//! Storylets, native downtime requests, and the host-event seam.
//!
//! The storylet surface refreshes through a gate: `refresh_storylet_rows`
//! rebuilds only when the authoritative world/content and secret IDs changed,
//! because it runs after every dispatch while the surface is open (see the
//! 2026-07-20 perf plan).
//! `emit_host_events` lives here too, as the one place a burst of events
//! becomes a single authority step.
//!
//! Split out of `main.rs` on 2026-07-24; behavior unchanged.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StoryletInputs {
    Table { world: CampaignWorld, secret_ids: Vec<String> },
    Native { cursor: isometry_campaign::founding::NativeCursor,
        content_digest: String, secret_ids: Vec<String> },
}

impl App {
    /// The storylet surface (C6). While the DM has it open, resolve each
    /// campaign storylet against the current world and the host-private secrets
    /// and hand the view rows; when the DM plays one, commit its effects.
    ///
    /// Host-only: matching reads secret facts, and committing is authoring. A
    /// client's `open_storylets`/`play_storylet` are gated on `can_edit_inventory`
    /// upstream, so this never runs for a joined player.
    /// Rebuild the storylet rows, but only when the answer can have changed.
    ///
    /// This runs after every dispatch while the surface is open. It used to
    /// clone the whole `CampaignWorld` and re-resolve every storylet each time,
    /// almost always producing rows identical to the ones already on screen. At
    /// 400 places / 200 storylets the 07-20 plan's scaled receipt puts that at
    /// ~623us of clone plus ~42us of resolve, per click. The clone was the
    /// larger half by an order of magnitude and was not needed at all: the rows
    /// can be built from a borrow.
    ///
    /// The gate compares the inputs the rows were built from rather than a world
    /// revision counter. A counter is a new invariant every mutation site has to
    /// honor, and one missed bump leaves the surface silently stale, which is a
    /// worse failure than the cost it saves. `(world, secret_ids)` is the
    /// complete input set for sim-off. Sim-on instead binds accepted native
    /// cursor, saved Content and secret IDs, and never falls back to that fold.
    pub(crate) fn refresh_storylet_rows(&mut self, ctx: &mut Ctx<'_>) {
        let secret_ids: Vec<String> = self.campaign.secret_ids().map(str::to_owned).collect();
        if let Some(world) = &ctx.runner.state().native.world {
            let result = (|| {
                let inputs = StoryletInputs::Native { cursor:world.cursor()?,
                    content_digest:world.content_digest()?, secret_ids:secret_ids.clone() };
                if self.last_storylet_inputs.as_ref() == Some(&inputs) { return Ok(None); }
                let campaign = world.load(isocosm::Execution::Grouped)?;
                let reading = campaign.reading();
                let rows = campaign.content().storylets.iter().map(|(key, storylet)| {
                    match reading.resolve_storylet(storylet, secret_ids.iter().map(String::as_str)) {
                        Ok(resolved) => StoryletRow { key:key.clone(), entry:storylet.entry.clone(),
                            available:true, status:"ready".into(), cast:resolved.cast.into_iter().map(|(role, character)| {
                                let name = reading.authored_entity(&character).and_then(|(_, entity)| entity.authored.as_ref())
                                    .map(|fill| fill.name.clone()).unwrap_or(character);
                                (role, name)
                            }).collect() },
                        Err(error) => StoryletRow { key:key.clone(), entry:storylet.entry.clone(),
                            available:false, status:describe_storylet_error(&error), cast:Vec::new() },
                    }
                }).collect();
                Ok::<_, String>(Some((inputs, rows)))
            })();
            match result {
                Ok(Some((inputs, rows))) => {
                    self.last_storylet_inputs = Some(inputs);
                    ctx.runner.update(|ui| {
                        ui.storylets = rows;
                        if ui.storylet_selected >= ui.storylets.len() { ui.storylet_selected = 0; }
                    });
                },
                Ok(None) => {},
                Err(why) => {
                    self.last_storylet_inputs = None;
                    ctx.runner.update(|ui| { ui.storylets.clear(); ui.status = why; });
                },
            }
            return;
        }
        let runner = &*ctx.runner;
        let world = &runner.state().world;
        if matches!(self.last_storylet_inputs.as_ref(), Some(StoryletInputs::Table { world:seen, secret_ids:ids })
            if seen == world && ids == &secret_ids) {
            return;
        }

        let rows = storylet_rows(world, &secret_ids);
        // The one clone, on the path that actually changed something.
        self.last_storylet_inputs = Some(StoryletInputs::Table { world:world.clone(), secret_ids });

        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                // Only replace on change, to keep the DM's selection stable.
                if ui.storylets != rows {
                    ui.storylets = rows;
                    if ui.storylet_selected >= ui.storylets.len() {
                        ui.storylet_selected = 0;
                    }
                }
            });
        }
    }

    pub(crate) fn pump_storylets(&mut self, ctx: &mut Ctx<'_>) {
        // Cheap flags first: with the surface closed and no request pending
        // (every ordinary dispatch), this returns before touching the world.
        let (open, request, can_edit) = {
            let r = &*ctx.runner;
            let s = r.state();
            (
                s.storylet_open,
                s.storylet_request.clone(),
                s.can_edit_inventory,
            )
        };
        if !can_edit || (!open && request.is_none()) {
            return;
        }

        // Refresh the rows while the surface is open: which storylets are
        // playable now (requirements met, roles cast) and which are still locked.
        if open {
            self.refresh_storylet_rows(ctx);
        }

        // Play a storylet: commit its effects. A storylet Item effect wants a
        // recipient. Native play needs the explicitly selected bound token;
        // table-only play retains its active-token/first-token behavior.
        let Some(key) = request else {
            return;
        };
        let (item_owner, snapshot, any_storylets) = {
            let r = &*ctx.runner;
            let s = r.state();
            let owner = if s.native.world.is_some() { s.selected_token }
                else { s.turns.active().or_else(|| s.map.tokens.first().map(|t| t.id)) };
            (owner, self.snapshot_of(s), s.native.world.is_some() || !s.world.storylets.is_empty())
        };
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.storylet_request = None);
        }
        if !any_storylets {
            return;
        }
        let remote = matches!(Some(ctx.runner.state().net_mode), Some(NetMode::Remote));
        if remote {
            if let Some(net) = self.net.as_mut() {
                let request = net.commit_storylet(key.clone(), item_owner);
                {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| {
                        ui.status = match request {
                            Some(request) => format!("playing storylet (request {request})"),
                            None => "storylet authority actor stopped".to_owned(),
                        };
                    });
                }
            }
        } else {
            let mut host =
                HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
            match host.commit_storylet(&key, item_owner) {
                Ok(_) => {
                    self.campaign = host.campaign().clone();
                    self.history = host.history().clone();
                    self.journal = host.state().journal.clone();
                    let snapshot = host.state().clone();
                    {
                        let runner = &mut *ctx.runner;
                        runner.update(|ui| {
                            ui.apply_snapshot(snapshot);
                            ui.status = format!("played storylet: {key}");
                        });
                    }
                },
                Err(error) => {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| ui.status = format!("storylet failed: {error}"));
                },
            }
        }
    }

    /// Consume retired entropy-turn requests without manufacturing world acts.
    /// Native downtime must pass unanimous consent through the native time contract.
    pub(crate) fn pump_downtime(&mut self, ctx: &mut Ctx<'_>) {
        let ui = ctx.runner.state();
        if !ui.downtime_roll_request && !ui.downtime_commit_request { return; }
        ctx.runner.update(|ui| {
            ui.downtime_roll_request = false;
            ui.downtime_commit_request = false;
            ui.faction_moves.clear();
            ui.status = "advance native downtime with every participant's consent".into();
        });
    }

    /// Commit one host-authored game event: solo through a local `HostSession`
    /// (copying its campaign/history/journal back and mirroring the snapshot),
    /// in a live session through the host bridge. Leaves `status` alone, which
    /// `apply_snapshot` preserves, so a caller may set it before or after.
    pub(crate) fn emit_host_event(&mut self, ctx: &mut Ctx<'_>, event: GameEvent) {
        self.emit_host_events(ctx, vec![event]);
    }

    /// Emit several events as one authority step.
    ///
    /// The per-event path pays a snapshot clone, a `HostSession` build, and an
    /// `apply_snapshot` (which recomputes fog and reach) for *each* event, so a
    /// burst of N costs N full round-trips. A map read revealing a dozen places
    /// is exactly that burst. Batching applies them to one session and lands one
    /// snapshot, so the fog and reach recompute happens once.
    ///
    /// Order is preserved and each event still goes through `local_event`, so
    /// this is the same adjudication the single-event path performs, not a
    /// shortcut around it.
    pub(crate) fn emit_host_events(&mut self, ctx: &mut Ctx<'_>, events: Vec<GameEvent>) {
        if events.is_empty() {
            return;
        }
        let remote = Some(&*ctx.runner)
            .map(|r| r.state().net_mode == NetMode::Remote)
            .unwrap_or(false);
        if remote {
            if let Some(net) = self.net.as_mut() {
                // The bridge orders submissions; batching is a host-side saving
                // only, and the wire sees the same sequence either way.
                for event in events {
                    net.submit(event);
                }
            }
        } else {
            let snapshot = {
                let r = &*ctx.runner;
                self.snapshot_of(r.state())
            };
            let mut host =
                HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
            for event in events {
                let _ = host.local_event(event);
            }
            self.campaign = host.campaign().clone();
            self.history = host.history().clone();
            self.journal = host.state().journal.clone();
            let snapshot = host.state().clone();
            {
                let runner = &mut *ctx.runner;
                runner.update(|ui| ui.apply_snapshot(snapshot));
            }
            self.refresh_source_history(ctx);
        }
    }
}

/// A DM-facing reason a storylet is not yet playable.
fn describe_storylet_error(error: &isometry_campaign::StoryletError) -> String {
    use isometry_campaign::StoryletError::*;
    match error {
        MissingFactionTag(tag) => format!("needs a faction tagged '{tag}'"),
        MissingHiddenFact(id) => format!("needs the secret '{id}' to be true"),
        MissingWorldLaw(id) => format!("needs the law '{id}'"),
        UncastRole(role) => format!("no character fits the role '{role}'"),
    }
}

/// Which storylets are playable now and which are still locked, given the world
/// and the host's secret-fact ids.
///
/// Free and pure on purpose. `refresh_storylet_rows` caches exactly these two
/// arguments and skips the call when neither changed, and that gate is sound
/// only if the rows depend on nothing else. Keeping the builder a function of
/// its inputs makes that structural rather than a promise in a comment: adding
/// a third input here without adding it to the cache will not compile past the
/// call site.
fn storylet_rows(world: &CampaignWorld, secret_ids: &[String]) -> Vec<StoryletRow> {
    world
        .storylets
        .iter()
        .map(|(key, storylet)| {
            match world.resolve_storylet(storylet, secret_ids.iter().map(String::as_str)) {
                Ok(resolved) => StoryletRow {
                    key: key.clone(),
                    entry: storylet.entry.clone(),
                    available: true,
                    status: "ready".to_owned(),
                    cast: resolved
                        .cast
                        .into_iter()
                        .map(|(role, character_id)| {
                            let name = world
                                .characters()
                                .get(&character_id)
                                .map(|c| c.name.clone())
                                .unwrap_or(character_id);
                            (role, name)
                        })
                        .collect(),
                },
                Err(error) => StoryletRow {
                    key: key.clone(),
                    entry: storylet.entry.clone(),
                    available: false,
                    status: describe_storylet_error(&error),
                    cast: Vec::new(),
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_views::synth_world;

    /// The storylet refresh gate skips the rebuild when `(world, secret_ids)` is
    /// unchanged. That is only safe while the rows are a pure function of those
    /// two, so pin it: identical inputs must give identical rows, or the gate
    /// starts serving a stale surface and nothing reports it.
    #[test]
    fn storylet_rows_are_a_function_of_world_and_secrets() {
        let world = synth_world(24, 12);
        let secrets: Vec<String> = vec!["secret4".to_owned()];
        assert_eq!(
            storylet_rows(&world, &secrets),
            storylet_rows(&world, &secrets),
            "unchanged inputs must give unchanged rows, or the refresh gate is unsound"
        );
    }

    /// The mirror: each cached input has to actually move the rows, or it is
    /// dead weight in the compare and the real input is missing from it.
    #[test]
    fn storylet_rows_track_both_cached_inputs() {
        let world = synth_world(24, 12);
        let none: Vec<String> = Vec::new();
        let base = storylet_rows(&world, &none);

        // A secret the world does not hold unlocks a hidden-fact storylet. Index
        // 2 is the hidden-fact shape in synth_world's rotation.
        let with_secret = storylet_rows(&world, &["secret2".to_owned()]);
        assert_ne!(
            base, with_secret,
            "a revealed secret must change the rows, or secret_ids need not be cached"
        );

        // Removing the cast breaks every role-bearing storylet.
        let mut thinner = world.clone();
        thinner
            .edit(|e| e.retain(|a| !matches!(a, isocosm::asserted::Assertion::Character(_))))
            .unwrap();
        assert_ne!(
            base,
            storylet_rows(&thinner, &none),
            "a changed world must change the rows"
        );
    }
}
