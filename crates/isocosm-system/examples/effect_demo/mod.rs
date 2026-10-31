//! Small authored board shared by the runnable example and its integration
//! checks. It supplies declared casting resources, not new schema defaults.
use isocosm_campaign::{EquipmentSlot, Inventory, ItemId, ItemInstance};
use isocosm_core::{Facing, FieldValue, MapDocument, Rng, SheetData, Token, TokenId};
use isocosm_system::{active_effects::*, pf2e_srd, srd_5e};
use std::collections::BTreeMap;

pub const CASTER: TokenId = TokenId(1);
pub const BEARER: TokenId = TokenId(2);

pub fn clock(tick: u64) -> Clock {
    Clock {
        tick,
        tick_microseconds: 1_000_000,
        next_sun: Some(21_600),
        next_daily_preparation: Some(86_400),
    }
}

pub struct Scene {
    pub map: MapDocument,
    pub inventories: BTreeMap<TokenId, Inventory>,
    pub definitions: Definitions,
    pub rng: Rng,
}

impl Scene {
    pub fn new(edition: Edition, seed: u64) -> Self {
        let definitions = Definitions::builtin().unwrap();
        let mut map = MapDocument::new("effect procedures", 20, 5);
        let mut base = match edition {
            Edition::Dnd5 => srd_5e().default_sheet(),
            Edition::Pathfinder2 => pf2e_srd().default_sheet(),
            Edition::Ars5 => SheetData::new(edition.system()),
        };
        base.set_text("name", "Bearer");
        base.set_text("body_form", "corpus");
        for (key, value) in [
            ("size_category", 2),
            ("ars_size", 0),
            ("reach_ft", 5),
            ("weight_mg", 64_000_000),
            ("con_save_bonus", 3),
            ("spell_save_dc", 15),
            ("stamina", 2),
            ("muto", 10),
            ("corpus", 10),
            ("animal", 10),
            ("terram", 10),
            ("hp_current", 30),
            ("fatigue_lost", 0),
        ] {
            base.set_int(key, value);
        }
        for key in ["can_manipulate", "can_speak", "material_or_focus"] {
            base.fields.insert(key.into(), FieldValue::Bool(true));
        }
        for rank in 1..=10 {
            base.set_int(format!("slot:{rank}"), 12);
        }
        for def in definitions.0.values().filter(|d| d.edition == edition) {
            base.fields
                .insert(format!("known:{}", def.id), FieldValue::Bool(true));
            for rank in def.level..=10 {
                base.set_int(format!("prepared:{}:{rank}", def.id), 12);
            }
        }
        for (id, at) in [(CASTER, (1, 2)), (BEARER, (2, 2))] {
            map.tokens.push(Token {
                id,
                at,
                facing: Facing::East,
                sprite: "hero".into(),
                owner: None,
            });
            map.sheets.insert(id, base.clone());
        }
        let sword = ItemInstance {
            id: ItemId("demo:sword".into()),
            template: "demo:metal-sword".into(),
            name: "Sword".into(),
            quantity: 1,
            tags: vec!["weapon".into(), "metal".into(), "edged".into()],
            modifiers: vec![],
            appearance_layers: vec![],
        };
        let inventory = Inventory {
            items: [(sword.id.clone(), sword.clone())].into(),
            equipped: [(EquipmentSlot::MainHand, sword.id)].into(),
        };
        Self {
            map,
            inventories: [(BEARER, inventory)].into(),
            definitions,
            rng: Rng::new(seed),
        }
    }

    pub fn request(&self, definition: &str, request_id: &str) -> Cast {
        let def = self.definitions.get(definition).unwrap();
        Cast {
            request_id: request_id.into(),
            definition: definition.into(),
            caster: CASTER,
            target: if def.kind == Kind::Weapon {
                Target::Item(ItemId("demo:sword".into()))
            } else {
                Target::Creature(if def.personal { CASTER } else { BEARER })
            },
            rank: def.level,
            size_change: if def.kind == Kind::Size { 1 } else { 0 },
            willing: true,
            ruling: None,
            resisted: false,
        }
    }

    pub fn cast(&mut self, request: &Cast, tick: u64, combat: bool) -> Result<Receipt, String> {
        let context = CastContext {
            clock: clock(tick),
            feet_per_tile: 5,
            in_combat: combat,
            line_of_effect: &|_, _| true,
        };
        cast(
            &mut self.map,
            &self.inventories,
            &self.definitions,
            request,
            &context,
            &mut self.rng,
        )
    }

    pub fn sheet(&self, id: TokenId) -> SheetData {
        project_sheet(
            &self.map,
            id,
            self.inventories.get(&id),
            self.map.sheet(id).unwrap(),
        )
        .unwrap()
    }
}

pub fn body() -> cosmograph::schema::BodyDocument {
    use isometer_core::{BodyDocument, VolumeRef};
    BodyDocument::new(VolumeRef::from_tag(1), [4, 8, 4])
}
