// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::attention::WindowId;
use isocosm::directing::readings::View;
use isocosm_overlay::{AttentionChange, IntentEnvelope, OrdinaryAccess, ParticipantHandle,
    PlayerAttention, Seat, Tick, WindowAttentionEnvelope, WindowHandle};
use crate::{access, binding::{self, Binding, BindingPlan}, native_attention,
    world::{canonical_binding, Cut, World}};

pub struct Window {
    world: World, participant: ParticipantHandle, seat: Seat, binding: Binding,
    active: Option<WindowId>, closed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection { pub access: OrdinaryAccess, pub visible: View, pub attention: PlayerAttention }

pub struct PreparedBinding {
    pub requested: Binding,
    before: Cut,
    window: WindowId,
    participant: ParticipantHandle,
}

impl Window {
    /// Read-only preview construction, retained for the prior core checkpoint.
    /// An actual mode lifecycle uses attach or restores an admitted native ID.
    pub fn new(world: &World, participant: ParticipantHandle, seat: Seat, binding: Binding) -> Result<Self, String> {
        let binding = world.read(|session| {
            access::resolve(session, participant, seat)?;
            canonical_binding(session, participant, binding)
        })??;
        Ok(Self { world: world.share(), participant, seat, binding, active: None, closed: false })
    }

    pub fn attach(world: &World, participant: ParticipantHandle, seat: Seat, binding: Binding) -> Result<Self, String> {
        let mut window = Self::new(world, participant, seat, binding)?;
        window.active = Some(world.attach_window(participant, &window.binding)?);
        Ok(window)
    }

    /// The trusted caller names the exact saved native window. Loading does
    /// not guess a first participant or attach/close any replayed window.
    pub fn restore(world: &World, participant: ParticipantHandle, seat: Seat,
        handle: WindowHandle, binding: Binding) -> Result<Self, String> {
        let mut window = Self::new(world, participant, seat, binding)?;
        let id = world.read(|session| -> Result<WindowId, String> {
            let (id, profile, held) = session.sim.attention_window(participant.0, handle.0)?;
            let expected = native_attention::lower(session, &window.binding.attention)?;
            if profile != window.binding.mode || held != expected {
                return Err("local restored binding differs from its native window".into());
            }
            Ok(id)
        })??;
        window.active = Some(id);
        Ok(window)
    }

    pub fn cut(&self) -> Result<Cut, String> { self.world.cut() }
    pub fn binding(&self) -> &Binding { &self.binding }
    pub fn set_view_defaults(&mut self, defaults: Vec<u8>) -> Result<(), String> {
        self.projection()?;
        self.binding.view_defaults = defaults;
        Ok(())
    }
    pub fn handle(&self) -> Result<WindowHandle, String> {
        self.active.map(|id| WindowHandle(id.id())).ok_or("window has no active native attachment".into())
    }

    pub fn projection(&self) -> Result<Projection, String> {
        if self.closed { return Err("window is closed".into()); }
        self.world.read(|session| {
            if let Some(id) = self.active { session.sim.attention_window(self.participant.0, id.id())?; }
            let access = access::resolve(session, self.participant, self.seat)?;
            let visible = access::permitted(session, access);
            let mut attention = native_attention::effective(session, self.participant)?;
            access::filter_attention(session, access, &visible, &mut attention);
            Ok(Projection { access, visible, attention })
        })?
    }

    pub fn prepare_binding(&self, requested: Binding) -> Result<BindingPlan, String> {
        if self.closed { return Err("window is closed".into()); }
        let requested = self.world.read(|session| canonical_binding(session, self.participant, requested))??;
        Ok(binding::plan(&self.binding, requested, self.participant, self.cut()?.tick))
    }

    /// Publication has one success point: no local controls, mode, camera or
    /// attention change can escape a refused candidate native Session.
    pub fn prepare_commit(&self, requested: Binding) -> Result<PreparedBinding, String> {
        let window = self.active.ok_or("window has no active native attachment")?;
        let requested = self.world.read(|session| canonical_binding(session, self.participant, requested))??;
        Ok(PreparedBinding { requested, before: self.cut()?, window, participant: self.participant })
    }

    pub fn commit_binding(&mut self, prepared: PreparedBinding) -> Result<(), String> {
        let id = self.active.ok_or("window has no active native attachment")?;
        if prepared.window != id || prepared.participant != self.participant {
            return Err("prepared binding belongs to another window".into());
        }
        let accepted = self.world.replace_window(self.participant, id, &prepared.requested, prepared.before)?;
        self.binding = accepted;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), String> {
        let id = self.active.ok_or("window has no active native attachment")?;
        self.world.close_window(self.participant, id)?;
        self.active = None;
        self.closed = true;
        Ok(())
    }

    pub fn world(&self) -> &World { &self.world }
    pub fn participant(&self) -> ParticipantHandle { self.participant }

    pub fn attention_request(&self, change: AttentionChange) -> Result<IntentEnvelope<()>, String> {
        Ok(binding::envelope(self.participant, self.cut()?.tick, change))
    }
    pub fn window_attention_request(&self, change: AttentionChange) -> Result<WindowAttentionEnvelope, String> {
        Ok(WindowAttentionEnvelope { tick: Tick(self.cut()?.tick), participant: self.participant,
            window: self.handle()?, change })
    }
    pub fn apply_attention(&mut self, request: WindowAttentionEnvelope) -> Result<(), String> {
        let before = self.cut()?;
        if request.participant != self.participant || request.window != self.handle()? || request.tick != Tick(before.tick) {
            return Err("attention request names another window, participant or tick".into());
        }
        let mut requested = self.binding.clone();
        match request.change {
            AttentionChange::Pin(point) => { requested.attention.pinned.insert(point); },
            AttentionChange::Unpin(point) => { requested.attention.pinned.remove(&point); },
            AttentionChange::Examine(place) => requested.attention.examined = Some(place),
            AttentionChange::StopExamining => requested.attention.examined = None,
        }
        // Camera movement inside one region creates no native history entry.
        if requested == self.binding { return Ok(()); }
        let prepared = self.prepare_commit(requested)?;
        self.commit_binding(prepared)
    }
}
