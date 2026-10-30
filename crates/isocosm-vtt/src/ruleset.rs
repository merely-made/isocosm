//! Edition state beside the native world, accepted only by its current host.
//! No position, body, terrain or native custody is saved a second time here.
use isocosm::Session;
use isocosm_overlay::{EntityHandle, OrdinaryAccess, Seat};
use isometry_campaign::{Inventory, ItemId};
use isometry_core::{FieldValue, MapDocument, Rng, RollRecord, SheetData, TokenId};
use isometry_system::active_effects::{self as effects, Clock, Records};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod actions;
mod frame;

pub const SECTION: &str = "isocosm-vtt:edition";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Create(u64),
    Adjust {
        actor: u64,
        field: String,
        delta: i64,
    },
    Learn(u64),
    Give {
        actor: u64,
        template: String,
    },
    Equip {
        actor: u64,
        item: ItemId,
    },
    Cast {
        actor: u64,
        item: ItemId,
        rank: u8,
    },
    Attack {
        actor: u64,
        target: u64,
    },
    RuleAttack {
        actor: u64,
        target: u64,
    },
    Damage {
        target: u64,
        amount: u32,
    },
    Dismiss {
        actor: u64,
        effect: u64,
    },
    Calibrate,
    UseRulings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
    pub by: u64,
    pub tick: u64,
    pub action: Action,
    pub note: String,
    pub rolls: Vec<RollRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    #[serde(default = "experimental")]
    pub experimental: bool,
    pub sheets: BTreeMap<TokenId, SheetData>,
    pub inventories: BTreeMap<TokenId, Inventory>,
    pub weapon_damage: BTreeMap<ItemId, String>,
    pub system_data: BTreeMap<String, FieldValue>,
    pub rng: Rng,
    pub next_item: u64,
    /// Authored native base cells per five-foot square. None needs a DM ruling.
    pub cells_per_square: Option<u32>,
    pub log: Vec<Log>,
}

impl State {
    pub fn new(seed: u64) -> Self {
        Self {
            version: 1,
            experimental: true,
            sheets: BTreeMap::new(),
            inventories: BTreeMap::new(),
            weapon_damage: BTreeMap::new(),
            system_data: BTreeMap::new(),
            rng: Rng::new(seed),
            next_item: 1,
            cells_per_square: None,
            log: Vec::new(),
        }
    }
    pub fn read(sections: &BTreeMap<String, Vec<u8>>) -> Result<Option<Self>, String> {
        sections
            .get(SECTION)
            .map(|bytes| {
                let state: Self =
                    serde_json::from_slice(bytes).map_err(|e| format!("edition state: {e}"))?;
                if state.version != 1
                    || !state.experimental
                    || state.next_item == 0
                    || state.log.len() > 4096
                    || state.cells_per_square == Some(0)
                    || state.sheets.values().any(|sheet| sheet.system != "5e-srd")
                {
                    return Err("unsupported or invalid edition state".into());
                }
                Records::read(&state.map())?;
                Ok(state)
            })
            .transpose()
    }
    /// Pure rules input. These zero coordinates are never published as a pose.
    /// Self-touch casting is local; attacks validate range separately.
    pub fn map(&self) -> MapDocument {
        let mut map = MapDocument::new("Edition rules input", 1, 1);
        map.sheets = self.sheets.clone();
        map.system_data = self.system_data.clone();
        map.tokens = self
            .sheets
            .keys()
            .map(|id| isometry_core::Token {
                id: *id,
                at: (0, 0),
                facing: Default::default(),
                sprite: String::new(),
                owner: None,
            })
            .collect();
        map
    }
    pub fn current_map(&self, session: &Session) -> Result<MapDocument, String> {
        let mut map = self.map();
        effects::advance(&mut map, clock(session))?;
        Ok(map)
    }
    fn take_map(&mut self, map: MapDocument) {
        self.sheets = map.sheets;
        self.system_data = map.system_data;
    }
    fn store(&self, sections: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
        sections.insert(
            SECTION.into(),
            serde_json::to_vec(self).map_err(|e| e.to_string())?,
        );
        Ok(())
    }
}

fn experimental() -> bool {
    true
}

pub fn token(id: u64) -> Result<TokenId, String> {
    u32::try_from(id)
        .map(TokenId)
        .map_err(|_| "native ID exceeds this edition adapter's token range".into())
}

pub fn clock(session: &Session) -> Clock {
    Clock {
        tick: session.sim.state().tick,
        tick_microseconds: session.sim.genesis().rules.tick_microseconds(),
        next_sun: None,
        next_daily_preparation: None,
    }
}

pub fn controls(session: &Session, access: OrdinaryAccess, actor: u64) -> bool {
    (access.seat == Seat::Dm
        || access.seat == Seat::Player && access.subject == Some(EntityHandle(actor)))
        && session
            .sim
            .state()
            .population
            .get(actor)
            .is_some_and(|body| body.alive)
}

pub fn measured_reach(
    state: &State,
    session: &Session,
    sections: &BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    actor: u64,
    target: u64,
) -> Result<(), String> {
    if state.cells_per_square.is_none() {
        return Err("physical scale is undeclared".into());
    }
    frame::reach(state, session, sections, access, actor, target, &[]).map(|_| ())
}

pub fn settle(session: &Session, sections: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    if let Some(mut state) = State::read(sections)? {
        state.take_map(state.current_map(session)?);
        state.store(sections)?;
    }
    Ok(())
}

/// This runs inside World::product_sections, hence a refusal publishes no
/// prefix, no dice advance and no accepted family record.
pub fn apply(
    session: &Session,
    sections: &mut BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    action: &Action,
    payload: &[u8],
) -> Result<(), String> {
    crate::native::current_access(session, access)?;
    let mut state =
        State::read(sections)?.unwrap_or_else(|| State::new(session.sim.genesis().seed));
    if state.log.len() >= 4096 {
        return Err("edition history limit reached".into());
    }
    let mut map = state.current_map(session)?;
    let (note, rolls) = actions::apply(
        &mut state, &mut map, session, sections, access, action, payload,
    )?;
    state.take_map(map);
    state.log.push(Log {
        by: access.participant.0,
        tick: session.sim.state().tick,
        action: action.clone(),
        note,
        rolls,
    });
    state.store(sections)
}
