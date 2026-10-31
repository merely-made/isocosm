//! State transitions for the look editor: open on a token, edit a draft,
//! then accept it into the table or cancel and leave the table as it was.

use super::*;
use crate::appearance::{LookDraft, LookInput};

impl UiState {
    /// Whether this peer may change `token`'s look (case 1030): the seat that
    /// controls the token, as the authority's own ownership check reads it,
    /// or GM authority, which this table holds as `can_edit_inventory`.
    pub fn may_edit_look(&self, token: &Token) -> bool {
        self.can_edit_inventory || self.commands(token.owner.as_deref())
    }

    /// Open the editor on `id`'s look as the table holds it now.
    pub fn open_look(&mut self, id: TokenId) {
        let Some(token) = self.map.token(id).cloned() else {
            return;
        };
        if !self.may_edit_look(&token) {
            self.status = "only a GM or the seat that controls it can change a token's look".to_owned();
            return;
        }
        let held = LookInput::of(self, &token);
        self.look = LookDraft {
            token: Some(id),
            sprite: held.sprite,
            appearance: held.appearance.unwrap_or_default(),
            items: held.items,
            facing: token.facing,
        };
        self.look_open = true;
        self.character_open = false;
        self.context_menu = None;
    }

    /// Close the editor; the table keeps the look it had.
    pub fn cancel_look(&mut self) {
        self.look_open = false;
    }

    /// Write the draft into the table: locally, or as the event the authority
    /// applies and replicates.
    pub fn accept_look(&mut self) {
        let Some(id) = self.look.token else {
            return;
        };
        let sprite = self.look.sprite.clone();
        let appearance = self.look.appearance.clone();
        if self.net_mode == NetMode::Remote {
            self.net_outbox.push(GameEvent::AppearanceSet {
                token: id,
                sprite,
                appearance,
            });
            self.status = "look sent".to_owned();
        } else if self.map.set_appearance(id, &sprite, appearance).is_ok() {
            self.status = "look saved".to_owned();
        }
        self.look_open = false;
    }

    /// Pick the draft's class. Its overrides stay: they are the token's own.
    pub fn set_look_class(&mut self, sprite: &str) {
        if self.appearance.has_class(sprite) {
            self.look.sprite = sprite.to_owned();
        }
    }

    pub fn set_look_layer(&mut self, slot: &str, layer: &str) {
        let library = self.appearance.clone();
        self.look.set_layer(&library, slot, layer);
    }

    pub fn set_look_colour(&mut self, index: u8, colour: [u8; 3]) {
        let library = self.appearance.clone();
        self.look.set_colour(&library, index, colour);
    }

    /// Drop every override: the draft becomes its bare class.
    pub fn reset_look(&mut self) {
        self.look.appearance = TokenAppearance::default();
    }

    /// Turn the preview a quarter.
    pub fn turn_look(&mut self) {
        self.look.turn();
    }
}
