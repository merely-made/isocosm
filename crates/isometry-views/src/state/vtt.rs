use super::UiState;

/// One disclosed VTT projection, prepared by the host adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VttProjectionView {
    pub id: String,
    pub label: String,
    /// Observation cut from the snapshot rendered to the user.
    pub epoch: u64,
    pub revision: u64,
    pub cards: Vec<VttCardView>,
}

/// A portable card value already lowered to display text by the endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VttCardValue {
    pub label: String,
    pub value: String,
}

/// A card and only the actions its current projection advertised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VttCardView {
    pub id: String,
    pub title: String,
    pub values: Vec<VttCardValue>,
    pub badges: Vec<String>,
    pub actions: Vec<VttActionView>,
}

/// A named endpoint action. Its payload remains opaque to the view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VttActionView {
    pub id: String,
    pub label: String,
    pub explanation: String,
    pub payload_schema: String,
}

impl VttActionView {
    /// Whether the endpoint advertised a structured input contract.
    pub fn requires_payload(&self) -> bool {
        self.payload_schema.trim() != r#"{"type":"null"}"#
    }
}

/// One-shot request formed from the currently displayed projection action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VttActionRequest {
    pub projection: String,
    pub card: String,
    pub action: String,
    pub payload: Vec<u8>,
    /// The exact observation cut shown when the user chose this action.
    pub epoch: u64,
    pub revision: u64,
}

impl UiState {
    /// Replace the endpoint's current projection views while retaining a valid
    /// local tab choice. Any action queued against the old snapshot is dropped.
    pub fn set_vtt_projections(&mut self, projections: Vec<VttProjectionView>) {
        if !projections
            .iter()
            .any(|view| view.id == self.vtt_selected_projection)
        {
            self.vtt_selected_projection = projections
                .first()
                .map_or_else(String::new, |view| view.id.clone());
        }
        self.vtt_projections = projections;
        self.vtt_action_request = None;
    }

    /// Select one projection that the host has actually supplied.
    pub fn select_vtt_projection(&mut self, id: &str) {
        if self.vtt_projections.iter().any(|view| view.id == id) {
            if self.vtt_selected_projection != id {
                self.vtt_action_request = None;
            }
            self.vtt_selected_projection = id.to_owned();
        }
    }

    /// Drain the view's action request. This carries no authority; the host
    /// must invoke the action against its current endpoint disclosure.
    pub fn take_vtt_action_request(&mut self) -> Option<VttActionRequest> {
        self.vtt_action_request.take()
    }

    pub(crate) fn request_vtt_action(&mut self, projection: &str, card: &str, action: &str) {
        let view = self
            .vtt_projections
            .iter()
            .find(|view| view.id == projection);
        let action_view = view.and_then(|view| {
            view.cards
                .iter()
                .find(|item| item.id == card)
                .and_then(|item| item.actions.iter().find(|candidate| candidate.id == action))
        });
        if projection != self.vtt_selected_projection {
            return;
        }
        if let (Some(view), Some(action_view)) = (view, action_view) {
            self.vtt_action_request = Some(VttActionRequest {
                projection: projection.to_owned(),
                card: card.to_owned(),
                action: action.to_owned(),
                payload: if action_view.requires_payload() {
                    self.vtt_action_payload.text().as_bytes().to_vec()
                } else {
                    Vec::new()
                },
                epoch: view.epoch,
                revision: view.revision,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_core::MapDocument;

    fn view(id: &str, action: &str) -> VttProjectionView {
        VttProjectionView {
            id: id.into(),
            label: id.into(),
            epoch: 2,
            revision: 7,
            cards: vec![VttCardView {
                id: "card-1".into(),
                title: "Card".into(),
                values: Vec::new(),
                badges: Vec::new(),
                actions: vec![VttActionView {
                    id: action.into(),
                    label: "Act".into(),
                    explanation: String::new(),
                    payload_schema: "{\"type\":\"object\"}".into(),
                }],
            }],
        }
    }

    #[test]
    fn only_currently_advertised_actions_can_be_drained_and_refresh_clears_them() {
        let mut ui = UiState::new(MapDocument::new("VTT control", 4, 3));
        ui.set_vtt_projections(vec![view("vtt:table", "vtt.join")]);
        ui.vtt_action_payload = cambium::TextInput::new("{\"name\":\"Mira\"}");
        ui.request_vtt_action("vtt:table", "card-1", "vtt.join");
        assert_eq!(
            ui.take_vtt_action_request(),
            Some(VttActionRequest {
                projection: "vtt:table".into(),
                card: "card-1".into(),
                action: "vtt.join".into(),
                payload: br#"{"name":"Mira"}"#.to_vec(),
                epoch: 2,
                revision: 7,
            })
        );
        ui.request_vtt_action("vtt:table", "card-1", "invented");
        assert!(ui.take_vtt_action_request().is_none());
        ui.request_vtt_action("vtt:table", "card-1", "vtt.join");
        ui.set_vtt_projections(vec![view("vtt:table", "vtt.leave")]);
        assert!(ui.take_vtt_action_request().is_none());
    }

    #[test]
    fn projection_refresh_preserves_a_valid_local_tab_or_selects_first() {
        let mut ui = UiState::new(MapDocument::new("VTT control", 4, 3));
        ui.set_vtt_projections(vec![view("vtt:table", "a"), view("vtt:journeys", "b")]);
        ui.select_vtt_projection("vtt:journeys");
        ui.set_vtt_projections(vec![view("vtt:table", "a"), view("vtt:journeys", "b")]);
        assert_eq!(ui.vtt_selected_projection, "vtt:journeys");
        ui.set_vtt_projections(vec![view("vtt:table", "a")]);
        assert_eq!(ui.vtt_selected_projection, "vtt:table");
    }

    #[test]
    fn null_schema_actions_always_send_empty_bytes() {
        let mut ui = UiState::new(MapDocument::new("VTT control", 4, 3));
        let mut view = view("vtt:table", "vtt.join");
        view.cards[0].actions[0].payload_schema = r#"{"type":"null"}"#.into();
        ui.set_vtt_projections(vec![view]);
        ui.vtt_action_payload = cambium::TextInput::new("stale details");
        ui.request_vtt_action("vtt:table", "card-1", "vtt.join");
        assert!(ui.take_vtt_action_request().unwrap().payload.is_empty());
    }
}
