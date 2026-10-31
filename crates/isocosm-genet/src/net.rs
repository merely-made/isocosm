//! Bridge between the async iroh session and the synchronous winit loop.
//!
//! The winit kernel owns the view-facing session data. A typed Armillary actor
//! owns Tokio and `HostNet` / `ClientNet`; it receives commands and emits
//! snapshots, campaign state, and status updates for the kernel to drain.

mod native;
mod party;
mod packs;
mod physical;
mod items;
mod composed_storylet;
mod institutions;
mod commands;
mod messages;
use messages::{BridgeCommand, BridgeUpdate};
use commands::drain_commands;

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use armillary::{ActorHandle, Correlated, Emitter, RequestId, RequestIds, Wake};
use isocosm_campaign::GenerationRecord;
use isocosm_campaign::CampaignStore;
use isocosm_core::TokenId;
use isonetry::iroh_link::{ClientNet, HostNet};
use isonetry::{ActionIntent, GameEvent, GameSnapshot};
use muniment::Journal;

/// Which side of the session this process runs.
pub enum Role {
    /// The DM: authoritative, prints a join ticket.
    Host {
        state: GameSnapshot,
        campaign: CampaignStore,
        history: Journal<GameEvent>,
    },
    /// A player: dials the host's ticket, announcing a name.
    Client { ticket: String, name: String },
}

/// The winit-thread handle to the background session actor.
pub struct NetBridge {
    actor: ActorHandle<BridgeCommand>,
    updates: Receiver<BridgeUpdate>,
    snapshot: Option<GameSnapshot>,
    campaign: Option<CampaignStore>,
    history: Option<Journal<GameEvent>>,
    version: u64,
    ticket: Option<String>,
    inbox: Vec<(String, String)>,
    players: Vec<String>,
    endpoints: Vec<[u8; 32]>,
    local_sender: Option<isonetry::native_table::TableSender>,
    request_ids: RequestIds,
    campaign_outcomes: Vec<Correlated<Result<(), String>>>,
    party_readings: Vec<Correlated<Result<isocosm_views::PartyView, String>>>,
    pack_readings: Vec<Correlated<Result<isocosm_views::PackReading,String>>>,
    physical_results: Vec<Correlated<Result<isocosm_views::PhysicalOutcome,String>>>,
    items_results: Vec<Correlated<Result<isocosm_views::ItemsOutcome,String>>>,
    storylet_physical_results: Vec<Correlated<Result<isocosm_views::StoryletPhysicalOutcome,String>>>,
    institution_results: Vec<Correlated<Result<isocosm_views::InstitutionOutcome,String>>>,
    failure: Option<String>,
    action_intents: Vec<ActionIntent>,
}

impl NetBridge {
    /// Spawn the session actor.
    ///
    /// `wake` is the host's own wake handle, in the callback shape Armillary
    /// takes. The bridge used to be polled from an idle tick with a no-op wake;
    /// now the actor schedules one drain turn (`after_wake`) and one redraw
    /// when it actually has something, so a still table costs nothing and a
    /// peer's move arrives without waiting out a poll interval.
    pub fn spawn(role: Role, wake: Wake) -> Self {
        let (actor, updates) =
            armillary::spawn_named("isocosm-session", wake, move |commands, out| {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("session runtime");
                rt.block_on(run(role, commands, out));
            });
        Self {
            actor,
            updates,
            snapshot: None,
            campaign: None,
            history: None,
            version: 0,
            ticket: None,
            inbox: Vec::new(),
            players: Vec::new(),
            endpoints: Vec::new(),
            local_sender: None,
            request_ids: RequestIds::default(),
            campaign_outcomes: Vec::new(),
            party_readings: Vec::new(),
            pack_readings: Vec::new(),
            physical_results: Vec::new(),
            items_results: Vec::new(),
            storylet_physical_results: Vec::new(),
            institution_results: Vec::new(),
            failure: None,
            action_intents: Vec::new(),
        }
    }

    /// Drain actor updates on the winit thread. Returns true when a new
    /// snapshot was accepted and the view needs rebuilding.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.updates.try_recv() {
                Ok(BridgeUpdate::LocalSender(sender)) => { self.local_sender=sender; changed=true; },
                Ok(BridgeUpdate::PackReading(read)) => { self.pack_readings.push(read); changed = true; },
                Ok(BridgeUpdate::PhysicalResult(result)) => { self.physical_results.push(result); changed = true; },
                Ok(BridgeUpdate::ItemsResult(result)) => { self.items_results.push(result); changed = true; },
                Ok(BridgeUpdate::StoryletPhysicalResult(result)) => { self.storylet_physical_results.push(result); changed = true; },
                Ok(BridgeUpdate::InstitutionResult(result)) => { self.institution_results.push(result); changed = true; },
                Ok(BridgeUpdate::PartyReading(read)) => {
                    self.party_readings.push(read);
                    changed = true;
                },
                Ok(BridgeUpdate::NativeEndpoints(endpoints)) => {
                    self.endpoints = endpoints;
                    changed = true;
                },
                Ok(BridgeUpdate::HostReady {
                    ticket,
                    snapshot,
                    campaign,
                    history,
                }) => {
                    self.ticket = Some(ticket);
                    self.campaign = Some(campaign);
                    self.history = Some(history);
                    self.set_snapshot(snapshot);
                    changed = true;
                },
                Ok(BridgeUpdate::HostState {
                    snapshot,
                    campaign,
                    history,
                    players,
                }) => {
                    self.campaign = Some(campaign);
                    self.history = Some(history);
                    self.players = players;
                    self.set_snapshot(snapshot);
                    changed = true;
                },
                Ok(BridgeUpdate::ClientState(snapshot)) => {
                    self.set_snapshot(snapshot);
                    changed = true;
                },
                Ok(BridgeUpdate::ActionIntents(mut intents)) => {
                    self.action_intents.append(&mut intents);
                    changed = true;
                },
                Ok(BridgeUpdate::Whispers(mut whispers)) => self.inbox.append(&mut whispers),
                Ok(BridgeUpdate::CampaignFinished(outcome)) => self.campaign_outcomes.push(outcome),
                Ok(BridgeUpdate::Failed(error)) => self.failure = Some(error),
                Err(TryRecvError::Disconnected) => { self.local_sender=None; changed=true; break; },
                Err(TryRecvError::Empty) => break,
            }
        }
        changed
    }

    fn set_snapshot(&mut self, snapshot: GameSnapshot) {
        self.snapshot = Some(snapshot);
        self.version = self.version.wrapping_add(1);
    }

    /// Queue a local game event for the session actor.
    pub fn submit(&self, event: GameEvent) {
        let _ = self.actor.command(BridgeCommand::Event(event));
    }

    pub fn travel(
        &mut self,
        token: TokenId,
        crossing: isonetry::RequestId,
        party: String,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::Travel {
                request,
                token,
                crossing,
                party,
            })
            .then_some(request)
    }

    /// Ask the host to resolve an action (client side). The client decides
    /// nothing: it names an actor, a victim and an action, and waits.
    pub fn submit_action(&self, intent: ActionIntent) {
        let _ = self.actor.command(BridgeCommand::Action(intent));
    }

    /// Drain client action requests awaiting adjudication (host side).
    pub fn take_action_intents(&mut self) -> Vec<ActionIntent> {
        std::mem::take(&mut self.action_intents)
    }

    pub fn commit_campaign(
        &mut self,
        record: GenerationRecord,
        item_owner: Option<TokenId>,
        party: Option<String>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::Campaign {
                request,
                record,
                item_owner,
                party,
            })
            .then_some(request)
    }

    /// Ask the host to play a storylet (session path). Its effects replicate.
    pub fn commit_storylet(
        &mut self,
        key: String,
        item_owner: Option<TokenId>,
    ) -> Option<RequestId> {
        let request = self.request_ids.issue();
        self.actor
            .command(BridgeCommand::Storylet {
                request,
                key,
                item_owner,
            })
            .then_some(request)
    }

    pub fn take_campaign_outcomes(&mut self) -> Vec<Correlated<Result<(), String>>> {
        std::mem::take(&mut self.campaign_outcomes)
    }

    /// Host: send a directed whisper to a named player.
    pub fn whisper(&self, to: String, text: String) {
        let _ = self.actor.command(BridgeCommand::Whisper { to, text });
    }

    /// Client: take whispers received since the last call.
    pub fn take_whispers(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.inbox)
    }

    /// Host: connected player names (whisper targets).
    pub fn players(&self) -> Vec<String> {
        self.players.clone()
    }

    pub fn authenticated_endpoints(&self) -> Vec<[u8; 32]> { self.endpoints.clone() }
    pub fn local_sender(&self) -> Option<isonetry::native_table::TableSender> { self.local_sender }

    /// The current change version; the UI redraws when it advances.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// The latest replicated snapshot, if one has arrived.
    pub fn latest(&self) -> Option<GameSnapshot> {
        self.snapshot.clone()
    }

    /// Host-only GM state as of the latest session poll.
    pub fn campaign(&self) -> Option<CampaignStore> {
        self.campaign.clone()
    }

    pub fn history(&self) -> Option<Journal<GameEvent>> {
        self.history.clone()
    }

    /// The host's join ticket, once bound.
    pub fn ticket(&self) -> Option<String> {
        self.ticket.clone()
    }

    /// A background bind/join failure. Reading clears the pending message.
    pub fn take_failure(&mut self) -> Option<String> {
        self.failure.take()
    }
}

async fn run(role: Role, commands: Receiver<BridgeCommand>, out: Emitter<BridgeUpdate>) {
    match role {
        Role::Host {
            state,
            campaign,
            history,
        } => run_host(state, campaign, history, commands, out).await,
        Role::Client { ticket, name } => run_client(ticket, name, commands, out).await,
    }
}

async fn run_host(
    state: GameSnapshot,
    campaign: CampaignStore,
    history: Journal<GameEvent>,
    commands: Receiver<BridgeCommand>,
    out: Emitter<BridgeUpdate>,
) {
    let host = match HostNet::bind_with_history(state, campaign, history).await {
        Ok(host) => host,
        Err(error) => {
            out.emit(BridgeUpdate::LocalSender(None));
            out.emit(BridgeUpdate::Failed(format!("host bind failed: {error}")));
            return;
        },
    };
    host.spawn_accept();
    out.emit(BridgeUpdate::LocalSender(Some(isonetry::native_table::TableSender::Host)));
    let ticket = host.ticket().await;
    println!("[isometry] hosting. share this ticket to join:\n\n  {ticket}\n");
    out.emit(BridgeUpdate::HostReady {
        ticket,
        snapshot: host.snapshot().await,
        campaign: host.campaign().await,
        history: host.history().await,
    });

    let mut last_seq = host.seq().await;
    let mut last_players = Vec::new();
    let mut last_endpoints = Vec::new();
    loop {
        let commands = match drain_commands(&commands) {
            Ok(commands) => commands,
            Err(()) => break,
        };
        for command in commands {
            match command {
                BridgeCommand::NativePack { request,intent,choreography } => {
                    let read = matches!(&intent,isocosm_views::PackRequest::Read);
                    packs::deliver(&out,request,packs::host_request(&host,intent,choreography).await,read);
                },
                BridgeCommand::NativePhysical { request,input } => { out.emit(BridgeUpdate::PhysicalResult(
                    Correlated::new(request,physical::host_request(&host,input).await))); },
                BridgeCommand::NativeItems { request,input } => { out.emit(BridgeUpdate::ItemsResult(
                    Correlated::new(request,items::host_request(&host,input).await))); },
                BridgeCommand::NativeComposedStorylet { request,input } => { out.emit(BridgeUpdate::StoryletPhysicalResult(
                    Correlated::new(request,composed_storylet::host_request(&host,input).await))); },
                BridgeCommand::NativeInstitution { request,input } => { out.emit(BridgeUpdate::InstitutionResult(
                    Correlated::new(request,institutions::host_request(&host,input).await))); },
                BridgeCommand::NativeTime { request, intent } => {
                    let result = native::host_time(&host, intent).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, result)));
                },
                BridgeCommand::NativeParty { request, intent } => {
                    let read = matches!(&intent, isocosm_views::PartyRequest::Read { .. } | isocosm_views::PartyRequest::ReadAt { .. });
                    match party::host_request(&host, intent).await {
                        Ok(Some(view)) => { out.emit(BridgeUpdate::PartyReading(Correlated::new(request, Ok(view)))); },
                        Ok(None) => { out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, Ok(())))); },
                        Err(why) if read => { out.emit(BridgeUpdate::PartyReading(Correlated::new(request, Err(why)))); },
                        Err(why) => { out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, Err(why)))); },
                    }
                },
                BridgeCommand::Event(event) => {
                    if let Err(reason) = host.submit_event(event).await { out.emit(BridgeUpdate::Failed(reason)); }
                },
                BridgeCommand::NativeTableBinding { request, input } => {
                    let result = host.commit_native_table_binding(&input).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, result)));
                },
                BridgeCommand::NativeTableAdmission { request, input } => {
                    let result = host.commit_native_table_admission(&input).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, result)));
                },
                // The DM resolves its own swings directly (it *is* the rules
                // system), so it never routes one through here. Handled anyway,
                // as the same request a player would send.
                BridgeCommand::Action(intent) => {
                    if host.snapshot().await.native.is_some() {
                        out.emit(BridgeUpdate::Failed("native actions require an admitted body or institution operation".into()));
                    } else {
                        out.emit(BridgeUpdate::ActionIntents(vec![intent]));
                    }
                },
                BridgeCommand::Travel {
                    request,
                    token,
                    crossing,
                    party,
                } => {
                    let result = host
                        .commit_transition_for_party(token, crossing, &party)
                        .await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::NativeSetup { request, setup, expected, record, item_owner } => {
                    let result = host.commit_native_setup(&setup, &expected, &record, item_owner).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, result)));
                },
                BridgeCommand::NativeCampaign { request, profile, mode, record, item_owner } => {
                    let result = host.commit_native_campaign(&profile, &record, item_owner, mode).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, result)));
                },
                BridgeCommand::Campaign {
                    request,
                    record,
                    item_owner,
                    party,
                } => {
                    let result = match party {
                        Some(party) => {
                            host.commit_campaign_for_party(record, item_owner, &party)
                                .await
                        },
                        None => host.commit_campaign(record, item_owner).await,
                    };
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::Storylet {
                    request,
                    key,
                    item_owner,
                } => {
                    // Reuse the campaign-outcome channel: both are one-shot
                    // "commit this, tell me if it took" results.
                    let result = host.commit_storylet(&key, item_owner).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request, result,
                    )));
                },
                BridgeCommand::Whisper { to, text } => host.whisper("dm", &to, &text).await,
            }
        }

        let intents = host.take_action_intents().await;
        if !intents.is_empty() {
            out.emit(BridgeUpdate::ActionIntents(intents));
        }

        let seq = host.seq().await;
        let players = host.player_names().await;
        let endpoints = host.authenticated_endpoints().await;
        if endpoints != last_endpoints {
            last_endpoints = endpoints.clone();
            out.emit(BridgeUpdate::NativeEndpoints(endpoints));
        }
        if seq != last_seq || players != last_players {
            last_seq = seq;
            last_players = players.clone();
            out.emit(BridgeUpdate::HostState {
                snapshot: host.snapshot().await,
                campaign: host.campaign().await,
                history: host.history().await,
                players,
            });
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
    out.emit(BridgeUpdate::LocalSender(None));
}

async fn run_client(
    ticket: String,
    name: String,
    commands: Receiver<BridgeCommand>,
    out: Emitter<BridgeUpdate>,
) {
    let client = match ClientNet::join(&ticket, &name).await {
        Ok(client) => client,
        Err(error) => {
            out.emit(BridgeUpdate::LocalSender(None));
            out.emit(BridgeUpdate::Failed(format!("join failed: {error}")));
            return;
        },
    };
    println!("[isometry] joined session as {name}; replaying host log.");
    let sender=isonetry::native_table::TableSender::Endpoint(client.endpoint_key());
    let mut sender_announced=false;
    let mut last_applied = u64::MAX;
    loop {
        if client.connection_closed() { break; }
        let commands = match drain_commands(&commands) {
            Ok(commands) => commands,
            Err(()) => break,
        };
        for command in commands {
            match command {
                BridgeCommand::NativePack { request,intent,choreography:_ } => packs::deliver(&out,request,
                    Err("adventure pack controls require the actual DM host".into()),matches!(&intent,isocosm_views::PackRequest::Read)),
                BridgeCommand::NativePhysical { request,input } => { out.emit(BridgeUpdate::PhysicalResult(
                    Correlated::new(request,physical::client_request(&client,input).await))); },
                BridgeCommand::NativeItems { request,input } => { out.emit(BridgeUpdate::ItemsResult(
                    Correlated::new(request,items::client_request(&client,input).await))); },
                BridgeCommand::NativeComposedStorylet { request,input } => { out.emit(BridgeUpdate::StoryletPhysicalResult(
                    Correlated::new(request,composed_storylet::client_request(&client,input).await))); },
                BridgeCommand::NativeInstitution { request,input } => { out.emit(BridgeUpdate::InstitutionResult(
                    Correlated::new(request,institutions::client_request(&client,input).await))); },
                BridgeCommand::NativeTime { request, intent } => {
                    let result = native::client_time(&client, intent).await;
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, result)));
                },
                BridgeCommand::NativeParty { request, intent } => {
                    let read = matches!(&intent, isocosm_views::PartyRequest::Read { .. } | isocosm_views::PartyRequest::ReadAt { .. });
                    match party::client_request(&client, intent).await {
                        Ok(Some(view)) => { out.emit(BridgeUpdate::PartyReading(Correlated::new(request, Ok(view)))); },
                        Ok(None) => { out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, Ok(())))); },
                        Err(why) if read => { out.emit(BridgeUpdate::PartyReading(Correlated::new(request, Err(why)))); },
                        Err(why) => { out.emit(BridgeUpdate::CampaignFinished(Correlated::new(request, Err(why)))); },
                    }
                },
                BridgeCommand::Event(event) => {
                    let _ = client.intent(event).await;
                },
                // The player asks; the host answers. Nothing about the outcome
                // travels with this.
                BridgeCommand::Action(intent) => {
                    let _ = client.action(intent).await;
                },
                BridgeCommand::Travel { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("travel is ruled by the host".to_owned()),
                    )));
                },
                BridgeCommand::Campaign { request, .. } | BridgeCommand::NativeCampaign { request, .. }
                    | BridgeCommand::NativeSetup { request, .. } | BridgeCommand::NativeTableBinding { request, .. }
                    | BridgeCommand::NativeTableAdmission { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("campaign commits require the host".to_owned()),
                    )));
                },
                BridgeCommand::Storylet { request, .. } => {
                    out.emit(BridgeUpdate::CampaignFinished(Correlated::new(
                        request,
                        Err("storylets are played by the host".to_owned()),
                    )));
                },
                BridgeCommand::Whisper { .. } => {},
            }
        }

        let applied = client.applied().await;
        if applied != last_applied {
            if let Some(snapshot) = client.state().await {
                if !sender_announced {out.emit(BridgeUpdate::LocalSender(Some(sender)));sender_announced=true;}
                last_applied = applied;
                out.emit(BridgeUpdate::ClientState(snapshot));
            }
        }
        let whispers = client.take_whispers().await;
        if !whispers.is_empty() {
            out.emit(BridgeUpdate::Whispers(whispers));
        }
        for reason in client.take_rejections().await { out.emit(BridgeUpdate::Failed(reason)); }
        tokio::time::sleep(Duration::from_millis(60)).await;
    }
    out.emit(BridgeUpdate::LocalSender(None));
}

#[cfg(test)]
mod tests;
