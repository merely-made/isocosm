// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use isocosm::{Command, Execution, Session, attention::WindowId};
use isocosm::history::Saved as NativeSaved;
use isocosm_overlay::{ParticipantHandle, PlayerAttention};
use crate::{binding::Binding, native_attention, save::{WorldSave, FORMAT_VERSION}};

mod replication;
pub use replication::ReplicationSnapshot;

#[derive(Clone)]
struct OpenWorld { session: Session, sections: BTreeMap<String, Vec<u8>>, revision: u64 }

pub struct World(Rc<RefCell<OpenWorld>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut { pub tick: u64, pub entries: usize, pub state_hash: u64 }

fn cut(session: &Session) -> Cut {
    Cut { tick: session.sim.state().tick, entries: session.entries.len(), state_hash: session.sim.state_hash() }
}

// Compare durable authority directly, avoiding two full witness encodings
// and genesis clones just to detect an unchanged product transaction.
fn same_authority(a: &Session, b: &Session) -> bool {
    a.sim.state() == b.sim.state()
        && a.branch == b.branch && a.entries == b.entries && a.checkpoints == b.checkpoints
        && (std::ptr::eq(a.sim.genesis(), b.sim.genesis()) || a.sim.genesis() == b.sim.genesis())
}

impl World {
    pub fn new(session: Session) -> Self {
        Self::with_sections(session, BTreeMap::new())
    }
    /// A world whose session and per-mode sections arrive together, as a
    /// product handoff carries them.
    pub fn with_sections(session: Session, sections: BTreeMap<String, Vec<u8>>) -> Self {
        Self(Rc::new(RefCell::new(OpenWorld { session, sections, revision: 0 })))
    }
    pub fn share(&self) -> Self { Self(self.0.clone()) }
    /// Give a session host sole ownership of this live world. Otherwise an
    /// external handle could publish outside its accepted-record sequence.
    pub fn into_exclusive(self) -> Result<Self, String> {
        if Rc::strong_count(&self.0) != 1 { return Err("world has shared owners".into()); }
        Ok(self)
    }
    /// Transfer an unshared world's session and opaque sections to a product's
    /// founding step. Shared live worlds cannot be taken apart.
    pub fn into_parts(self) -> Result<(Session, BTreeMap<String, Vec<u8>>), String> {
        let world = Rc::try_unwrap(self.0).map_err(|_|"world has shared owners")?.into_inner();
        Ok((world.session, world.sections))
    }
    pub fn resume(saved: WorldSave, execution: Execution) -> Result<Self, String> {
        if saved.format != FORMAT_VERSION { return Err("unknown mode-host save format".into()); }
        let session = Session::load(saved.native, execution)?;
        Ok(Self::with_sections(session, saved.sections))
    }
    pub fn import(saved: impl crate::save::ImportSave, execution: Execution) -> Result<Self, String> {
        Self::resume(saved.into_world_save()?, execution)
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
    /// The session and every mode's opaque section bytes, borrowed at one
    /// cut. Reading creates no Session or authority.
    pub fn read_sections<T>(&self, read: impl FnOnce(&Session, &BTreeMap<String, Vec<u8>>) -> T)
        -> Result<T, String> {
        let world = self.0.try_borrow().map_err(|_| "world is being advanced")?;
        Ok(read(&world.session, &world.sections))
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
    /// The session borrowed for reading, as a driver's accessors lend it.
    /// Hold it no longer than a read: a transaction refuses while it lives.
    pub fn session(&self) -> Result<std::cell::Ref<'_, Session>, String> {
        let world = self.0.try_borrow().map_err(|_| "world is being advanced")?;
        Ok(std::cell::Ref::map(world, |open| &open.session))
    }
    /// One mode's section bytes, if it has any.
    pub fn section(&self, mode: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(self.0.try_borrow().map_err(|_| "world is being advanced")?.sections.get(mode).cloned())
    }
    /// Replaces one mode's section alone, for a product whose native admission
    /// has already published. Identical bytes leave the revision alone.
    pub fn put_section(&self, mode: &str, bytes: Vec<u8>) -> Result<(), String> {
        let mut world = self.0.try_borrow_mut().map_err(|_| "world is being read")?;
        if world.sections.get(mode) != Some(&bytes) {
            let revision = world.revision.checked_add(1).ok_or("world revision overflow")?;
            world.sections.insert(mode.into(), bytes);
            world.revision = revision;
        }
        Ok(())
    }
    /// Changes accepted since this world opened, mode sections included,
    /// which a cut alone misses. Endpoints sharing the world stale on it.
    pub fn revision(&self) -> Result<u64, String> {
        Ok(self.0.try_borrow().map_err(|_| "world is being advanced")?.revision)
    }

    fn transaction<T>(&self, expected: Option<Cut>, admit: impl FnOnce(&mut Session) -> Result<T, String>)
        -> Result<T, String> {
        let mut world = self.0.try_borrow_mut().map_err(|_| "world is being read")?;
        if expected.is_some_and(|before| cut(&world.session) != before) {
            return Err("prepared binding is stale".into());
        }
        let mut candidate = world.session.clone();
        let accepted = admit(&mut candidate)?;
        if !same_authority(&candidate, &world.session) {
            let revision = world.revision.checked_add(1).ok_or("world revision overflow")?;
            world.session = candidate;
            world.revision = revision;
        }
        Ok(accepted)
    }

    pub fn product<T>(&self, before: Cut,
        admit: impl FnOnce(&mut Session) -> Result<T, String>) -> Result<T, String> {
        self.transaction(Some(before), admit)
    }

    /// Native admission and the mode sections it rewrites publish together.
    pub fn product_sections<T>(
        &self, before: Cut,
        admit: impl FnOnce(&mut Session, &BTreeMap<String, Vec<u8>>)
            -> Result<(T, BTreeMap<String, Vec<u8>>), String>,
    ) -> Result<T, String> {
        let mut world = self.0.try_borrow_mut().map_err(|_| "world is being read")?;
        if cut(&world.session) != before { return Err("prepared binding is stale".into()); }
        let mut candidate = world.session.clone();
        let (value, sections) = admit(&mut candidate, &world.sections)?;
        if !same_authority(&candidate, &world.session) || sections != world.sections {
            let revision = world.revision.checked_add(1).ok_or("world revision overflow")?;
            world.session = candidate;
            world.sections = sections;
            world.revision = revision;
        }
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

pub fn canonical_binding(session: &Session, participant: ParticipantHandle, mut binding: Binding)
    -> Result<Binding, String> {
    binding.attention.played = session.sim.plays(participant.0)
        .map(isocosm_overlay::EntityHandle).into_iter().collect();
    binding.attention.care = native_attention::care(session, participant, &binding.mode)?;
    Ok(binding)
}
