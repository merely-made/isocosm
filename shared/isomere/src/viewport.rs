// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The viewport card and the error line beneath it (M1 of the isomere plan).
//!
//! The 2026-09-15 inventory found the same card three times: a `custom_leaf`
//! with `role="img"` and a label, inside a `.scene-card` container, with an
//! optional overlay over it and an error line under the column. The Mesocosm
//! bench and the Eponym session wrote it twice in the same shape with
//! different words; this module is that shape once.
//!
//! **What crosses the boundary is a leaf key, not a scene.** [`ViewportCard`]
//! holds a `u64` and a box, exactly what the host registered its leaf under.
//! isomere never names a voxel and depends on `shared/isometer` for nothing.
//!
//! The classes and ids are the ones already in the two views — `.scene-card`,
//! `.viewport`, `#notice`, `#viewport-error` — because promotion is a move and
//! not a rename, and because both products' acceptance scenarios address the
//! leaf as `role="img"` containing its label.

use cambium::{
    AnyView, El, GenetCtx, GenetElement, OptionalAction, PointerEvent, custom_leaf, el, on_pointer,
    text,
};

/// The boxed child the products already spell as their own `Child` alias, so a
/// view function here drops straight into a product's tree without a wrapper.
pub type Child<State, Action> = Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// The container class every product's card carries.
pub const CARD_CLASS: &str = "scene-card";

/// The leaf class every product's viewport carries.
pub const LEAF_CLASS: &str = "viewport";

/// The class the shared sheet styles an error line by, beside the two ids the
/// products already emit.
pub const ERROR_CLASS: &str = "error-line";

/// What a product has to say about its viewport: the key the host registered
/// the leaf under, the leaf's box, the label the scenarios address it by, and
/// the four places a product still differs.
///
/// The last four are `None` for "what everyone else does": the shared
/// [`LEAF_CLASS`] and [`CARD_CLASS`], no `aria-description`, and the leaf's own
/// box style. A `Some` replaces that one thing and nothing else, so the emitted
/// element is byte-for-byte what the product wrote by hand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewportCard<'a> {
    /// The `LeafRegistry` key the host painted this leaf under.
    pub key: u64,
    /// The leaf's intrinsic box, in logical pixels.
    pub size: (u32, u32),
    /// The `aria-label`. Both scenarios select the viewport by it.
    pub label: &'a str,
    /// The leaf's `class`. `None` is [`LEAF_CLASS`].
    pub class: Option<&'a str>,
    /// The leaf's `id`, when the product gave it one.
    pub id: Option<&'a str>,
    /// The leaf's `aria-description`, when the product publishes one.
    pub description: Option<&'a str>,
    /// A `style` that *replaces* the leaf's own box style. The bench's
    /// transform demonstration is the only caller; an empty string is a
    /// deliberate erasure, not a no-op, because that is what the bench wrote.
    pub style: Option<&'a str>,
    /// The container's `class`. `None` is [`CARD_CLASS`].
    pub card_class: Option<&'a str>,
}

impl<'a> ViewportCard<'a> {
    /// A card at `key` with the shared classes and no extras.
    pub const fn new(key: u64, size: (u32, u32), label: &'a str) -> Self {
        Self {
            key,
            size,
            label,
            class: None,
            id: None,
            description: None,
            style: None,
            card_class: None,
        }
    }

    /// The leaf's effective class.
    pub fn leaf_class(&self) -> &str {
        self.class.unwrap_or(LEAF_CLASS)
    }

    /// The container's effective class.
    pub fn container_class(&self) -> &str {
        self.card_class.unwrap_or(CARD_CLASS)
    }

    /// Every attribute the leaf carries, in the order it carries them.
    ///
    /// This is what [`viewport_card`] applies and what the tests read, so the
    /// element cannot drift from what this crate claims about it. The first two
    /// rows are `custom_leaf`'s own — reapplying them with the same values is
    /// how a `style` override lands in the leaf's own slot rather than after
    /// the accessibility attributes, which is where the bench had it.
    pub fn leaf_attrs(&self) -> Vec<(&'static str, String)> {
        let (width, height) = self.size;
        let style = self.style.map_or_else(
            || format!("display:block;width:{width}px;height:{height}px"),
            str::to_owned,
        );
        let mut attrs = vec![("key", self.key.to_string()), ("style", style)];
        if let Some(id) = self.id {
            attrs.push(("id", id.to_owned()));
        }
        attrs.push(("class", self.leaf_class().to_owned()));
        attrs.push(("role", "img".to_owned()));
        attrs.push(("aria-label", self.label.to_owned()));
        if let Some(description) = self.description {
            attrs.push(("aria-description", description.to_owned()));
        }
        attrs
    }
}

/// The bare leaf: a `custom_leaf` carrying [`ViewportCard::leaf_attrs`].
///
/// Public because a product that wraps the leaf in something other than the
/// shared card still wants the same element; the card below is the ordinary
/// path.
pub fn viewport_leaf<State, Action>(card: &ViewportCard<'_>) -> El<(), State, Action>
where
    State: 'static,
    Action: 'static,
{
    let mut leaf = custom_leaf::<State, Action>(card.key, card.size.0, card.size.1);
    for (name, value) in card.leaf_attrs() {
        leaf = leaf.attr(name, value);
    }
    leaf
}

/// The `.scene-card` container over arbitrary children.
///
/// The card outlives the leaf: the bench swaps a "specimen hidden" placeholder
/// into the same container, and the overlay has to stay over it.
pub fn scene_card<State, Action>(
    class: Option<&str>,
    children: Vec<Child<State, Action>>,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    Box::new(el("div", children).attr("class", class.unwrap_or(CARD_CLASS)))
}

/// The whole card: the container, the leaf with its pointer handler, and an
/// optional overlay child over it.
///
/// `pointer` is optional because a card is a picture before it is a control;
/// when it is `Some`, the handler is the product's, taking the product's own
/// state exactly as [`cambium::on_pointer`] does.
pub fn viewport_card<State, Action, F, OA>(
    card: &ViewportCard<'_>,
    overlay: Option<Child<State, Action>>,
    pointer: Option<F>,
) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, PointerEvent) -> OA + 'static,
{
    let leaf = viewport_leaf::<State, Action>(card);
    let leaf: Child<State, Action> = match pointer {
        Some(handler) => Box::new(on_pointer(leaf, handler)),
        None => Box::new(leaf),
    };
    let mut children: Vec<Child<State, Action>> = Vec::with_capacity(2);
    children.push(leaf);
    children.extend(overlay);
    scene_card(card.card_class, children)
}

/// Every attribute an [`error_line`] carries, in order.
pub fn error_attrs(id: &str) -> Vec<(&'static str, String)> {
    vec![
        ("class", ERROR_CLASS.to_owned()),
        ("id", id.to_owned()),
        ("role", "status".to_owned()),
    ]
}

/// The error line under the scene column: a live region the host publishes
/// producer errors into.
///
/// `id` stays the product's — `notice` on the bench, `viewport-error` in the
/// session — because both scenarios address it by id; the shared
/// [`ERROR_CLASS`] rides alongside so the sheet has one selector to style
/// rather than a growing list of ids.
pub fn error_line<State, Action>(id: &str, message: impl Into<String>) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let mut line = el("p", text(message.into()));
    for (name, value) in error_attrs(id) {
        line = line.attr(name, value);
    }
    Box::new(line)
}
