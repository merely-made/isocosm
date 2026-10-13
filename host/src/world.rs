// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use isocosm::{Command, Execution, Session, attention::WindowId};
use isocosm::history::Saved as NativeSaved;
use isocosm_overlay::{ParticipantHandle, PlayerAttention};
use crate::{binding::Binding, native_attention, save::{WorldSave, FORMAT_VERSION}};

#[derive(Clone)]
struct OpenWorld { session: Session, sections: BTreeMap<String, Vec<u8>> }

pub(crate) struct World(Rc<RefCell<OpenWorld>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cut { pub tick: u64, pub entries: usize, pub state_hash: u64 }

fn cut(session: &Session) -> Cut {
    Cut { tick: session.sim.state().tick, entries: session.entries.len(), state_hash: session.sim.state_hash() }
}

impl World {
    pub fn new(session: Session) -> Self {
        Self(Rc::new(RefCell::new(OpenWorld { session, sections: BTreeMap::new() })))
    }
    #[cfg(all(feature = "desktop", feature = "eponym"))]
    pub fn take_eponym(handoff: eponym_play::NativeHandoff)
        -> Result<Self, (eponym_play::NativeHandoff, String)> {
        let checked = (|| {
            let state = eponym_play::contract::mode::ModeState::decode(&handoff.game)?
                .ok_or("Eponym handoff lacks its marked mode state")?;
            for binding in &state.bindings { binding.current(&handoff.native)?; }
            Ok::<(), String>(())
        })();
        if let Err(why) = checked { return Err((handoff, why)); }
        let sections = crate::mode_sections::with_game(&BTreeMap::new(),
            crate::products::Mode::Eponym, handoff.game.clone());
        match sections {
            Err(why) => Err((handoff, why)),
            Ok(sections) => Ok(Self(Rc::new(RefCell::new(OpenWorld {
                session: handoff.native, sections,
            })))),
        }
    }
    pub fn share(&self) -> Self { Self(self.0.clone()) }
    pub fn resume(saved: WorldSave, execution: Execution) -> Result<Self, String> {
        if saved.format != FORMAT_VERSION { return Err("unknown mode-host save format".into()); }
        let session = Session::load(saved.native, execution)?;
        Ok(Self(Rc::new(RefCell::new(OpenWorld { session, sections: saved.sections }))))
    }
    pub fn resume_native(saved: NativeSaved, execution: Execution) -> Result<Self, String> {
        Ok(Self::new(Session::load(saved, execution)?))
    }
    pub fn save(&self) -> Result<WorldSave, String> {
        let world = self.0.try_borrow().map_err(|_| "world is being advanced")?;
        Ok(WorldSave { format: FORMAT_VERSION, native: world.session.save(), sections: world.sections.clone() })
    }
    pub fn read<T>(&self, read: impl FnOnce(&Session) -> T) -> Result<T, String> {
        let world = self.0.try_borrow().map_err(|_| "world is being advanced")?;
        Ok(read(&world.session))
    }
    /// The requested available mode's opaque bytes and current native owner
    /// are borrowed at one cut. Reading creates no Session or authority.
    #[cfg(feature = "desktop")]
    pub fn read_game<T>(&self, mode: crate::products::Mode,
        read: impl FnOnce(&Session, &[u8]) -> Result<T, String>) -> Result<T, String> {
        let world = self.0.try_borrow().map_err(|_| "world is being advanced")?;
        let game = crate::mode_sections::game(&world.sections, mode)?;
        read(&world.session, &game)
    }
    // Only the admitted world driver calls these. A candidate also protects
    // compound native operations whose individual implementation can refuse.
    pub fn command(&self, command: Command) -> Result<String, String> {
        self.transaction(None, |session| session.command(command))
    }
    pub fn advance(&self, ticks: u64) -> Result<(), String> {
        self.transaction(None, |session| session.advance(ticks).map(|_| ()))
    }
    pub fn cut(&self) -> Result<Cut, String> { self.read(cut) }

    fn transaction<T>(&self, expected: Option<Cut>, admit: impl FnOnce(&mut Session) -> Result<T, String>)
        -> Result<T, String> {
        let mut world = self.0.try_borrow_mut().map_err(|_| "world is being read")?;
        if expected.is_some_and(|before| cut(&world.session) != before) {
            return Err("prepared binding is stale".into());
        }
        let mut candidate = world.session.clone();
        let accepted = admit(&mut candidate)?;
        world.session = candidate;
        Ok(accepted)
    }

    pub fn product<T>(&self, before: Cut,
        admit: impl FnOnce(&mut Session) -> Result<T, String>) -> Result<T, String> {
        self.transaction(Some(before), admit)
    }

    /// Native admission and this mode's opaque game bytes publish together.
    #[cfg(feature = "desktop")]
    pub fn product_game<T>(
        &self, before: Cut, mode: crate::products::Mode,
        admit: impl FnOnce(&mut Session, Vec<u8>) -> Result<(T, Vec<u8>), String>,
    ) -> Result<T, String> {
        let mut world = self.0.try_borrow_mut().map_err(|_| "world is being read")?;
        if cut(&world.session) != before { return Err("prepared binding is stale".into()); }
        let game = crate::mode_sections::game(&world.sections, mode)?;
        let mut candidate = world.session.clone();
        let (value, game) = admit(&mut candidate, game)?;
        let sections = crate::mode_sections::with_game(&world.sections, mode, game)?;
        world.session = candidate;
        world.sections = sections;
        Ok(value)
    }

    pub fn attach_window(&self, participant: ParticipantHandle, binding: &Binding) -> Result<WindowId, String> {
        self.transaction(None, |session| {
            let value = native_attention::lower(session, &binding.attention)?;
            session.attach_window(participant.0, binding.mode.clone(), value)
        })
    }
    pub fn replace_window(&self, participant: ParticipantHandle, window: WindowId,
        binding: &Binding, before: Cut) -> Result<Binding, String> {
        self.transaction(Some(before), |session| {
            let value = native_attention::lower(session, &binding.attention)?;
            session.replace_attention(participant.0, window, binding.mode.clone(), value)?;
            canonical_binding(session, participant, binding.clone())
        })
    }
    pub fn close_window(&self, participant: ParticipantHandle, window: WindowId) -> Result<(), String> {
        self.transaction(None, |session| session.close_attention(participant.0, window).map(|_| ()))
    }
    pub fn player_attention(&self, participant: ParticipantHandle) -> Result<PlayerAttention, String> {
        self.read(|session| native_attention::effective(session, participant))?
    }
}

pub(crate) fn canonical_binding(session: &Session, participant: ParticipantHandle, mut binding: Binding)
    -> Result<Binding, String> {
    binding.attention.played = session.sim.plays(participant.0)
        .map(isocosm_overlay::EntityHandle).into_iter().collect();
    binding.attention.care = native_attention::care(session, participant, &binding.mode)?;
    Ok(binding)
}
