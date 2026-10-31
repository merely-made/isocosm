//! Edition state beside the native world, accepted only by its current host.
//! No position, body, terrain or native custody is saved a second time here.
use isocosm::Session;
use isocosm_overlay::{EntityHandle, OrdinaryAccess, Seat};
use isometry_campaign::{Inventory, ItemId};
use isometry_core::{FieldValue, MapDocument, Rng, RollRecord, SheetData, TokenId};
use isometry_system::active_effects::{self as effects, Clock, Records};
use isometry_system::pf2e::{self, draft::Draft};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod actions;
mod frame;
pub(crate) mod pathfinder;

pub const SECTION: &str = "isocosm-vtt:edition";
const CAMPAIGN: &str = "isocosm-vtt:campaign";

pub use crate::founding::{EDITIONS, default_edition};

/// The edition the campaign was founded under (case 1029); a world with no
/// campaign record plays the 5e adapter it always has.
pub fn edition(sections: &BTreeMap<String, Vec<u8>>) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Founded {
        content: Founding,
    }
    #[derive(Deserialize)]
    struct Founding {
        #[serde(default = "default_edition")]
        edition: String,
    }
    let Some(bytes) = sections.get(CAMPAIGN) else {
        return Ok(default_edition());
    };
    let founded: Founded =
        serde_json::from_slice(bytes).map_err(|e| format!("campaign founding: {e}"))?;
    if !EDITIONS.contains(&founded.content.edition.as_str()) {
        return Err(format!("unknown edition {}", founded.content.edition));
    }
    Ok(founded.content.edition)
}

pub use crate::founding::edition_label as label;

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
    /// Answer the pending decision of a creation draft.
    Choose {
        actor: u64,
        slot: String,
        value: String,
    },
    /// Take back a draft's last choice.
    Undo(u64),
    /// Record a complete draft as the character's sheet.
    Finish(u64),
    /// Abandon an unfinished draft.
    Discard(u64),
    /// Roll initiative and begin encounter mode (a GM seat).
    StartEncounter,
    EndEncounter,
    EndTurn(u64),
    RaiseShield(u64),
    /// A skill check against the table's current DC.
    Check {
        actor: u64,
        skill: String,
    },
    /// A GM seat moves the DC that skill checks are made against.
    SetDc(i64),
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
    /// Creation drafts: the choices only, the character rebuilt from them.
    /// A finished draft stays as the record of how its sheet was made.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub drafts: BTreeMap<TokenId, Draft>,
    /// Encounter mode as the edition declares it (case 1026): initiative
    /// order, actions, reaction and attacks per turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter: Option<pf2e::play::Encounter>,
    /// The DC a GM seat set for skill checks; absent is the 1st-level 15.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_dc: Option<i64>,
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
            drafts: BTreeMap::new(),
            encounter: None,
            check_dc: None,
        }
    }
    pub fn read(sections: &BTreeMap<String, Vec<u8>>) -> Result<Option<Self>, String> {
        sections
            .get(SECTION)
            .map(|bytes| {
                let state: Self =
                    serde_json::from_slice(bytes).map_err(|e| format!("edition state: {e}"))?;
                let edition = edition(sections)?;
                if state.version != 1
                    || !state.experimental
                    || state.next_item == 0
                    || state.log.len() > 4096
                    || state.cells_per_square == Some(0)
                    || state.sheets.values().any(|sheet| sheet.system != edition)
                    || (edition != pf2e::SYSTEM
                        && (!state.drafts.is_empty() || state.encounter.is_some() || state.check_dc.is_some()))
                {
                    return Err("unsupported or invalid edition state".into());
                }
                pathfinder::check(&state)?;
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
    session: &mut Session,
    sections: &mut BTreeMap<String, Vec<u8>>,
    access: OrdinaryAccess,
    action: &Action,
    payload: &[u8],
) -> Result<(), String> {
    crate::native::current_access(session, access)?;
    let edition = edition(sections)?;
    let mut state =
        State::read(sections)?.unwrap_or_else(|| State::new(session.sim.genesis().seed));
    if state.log.len() >= 4096 {
        return Err("edition history limit reached".into());
    }
    let mut map = state.current_map(session)?;
    let (note, rolls) = if edition == pf2e::SYSTEM {
        pathfinder::apply(&mut state, &mut map, session, sections, access, action, payload)?
    } else {
        actions::apply(&mut state, &mut map, &*session, sections, access, action, payload)?
    };
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
