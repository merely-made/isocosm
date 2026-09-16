//! B5's element receipt: what stands inside the board container, by class.
//!
//! §1 of the plan measured 632 board elements per steady frame — one
//! absolutely-positioned box per ground tile, exposed cliff face, fog shroud,
//! prop and token. The flag's claim is that the count goes to **zero** and
//! stays at the DOM board's figure without it, with one deliberate exception:
//! B4 brought the two ground markers back as DOM on both arms, and the scene
//! arm's board is one `custom_leaf`. So the census names what it counts and
//! what it allows, rather than reporting a bare number.
//!
//! **What it counts.** Every element inside a `.board` container, scoped to
//! that container and not to the screen: the side panel carries `token`-classed
//! rows of its own, and counting the whole tree would hide the thing being
//! measured. The count is taken over the retained tree the shipping host laid
//! out — [`Harness`] with `window: None` — so it is what the host would paint,
//! not what this crate believes it emitted.
//!
//! **Why it always runs.** [`super::scene_routing`]'s receipts need a drawn
//! frame and skip loudly without an adapter. This one needs none: `board_root`
//! emits the leaf whether or not a producer ever draws into it. Both arms are
//! built in one process from an explicit flag rather than from the
//! environment, so `ISOMETRY_SCENE_BOARD` changes nothing here and the DOM arm
//! is always present as the positive control — a zero that only ever means
//! "the walk found nothing" is worth nothing.

use cambium_genet_winit_host::{Harness, Init, inert_hooks};
use isometry_core::TokenId;
use layout_dom_api::LayoutDom as _;
use taproot::Selector;

use super::*;

type BoardHarness = Harness<UiState, Logic, UiChild>;

/// The window the receipt lays out in, matching `host_routing`'s, so its
/// element figure is comparable with the rest of the board's receipts.
const WINDOW: (f32, f32) = (1_100.0, 820.0);

/// The classes §1's 632 counted. `beat` is the wrapper every token sprite
/// rides (`board/tokens.rs`), so a token is two boxes and both are listed:
/// omitting the wrapper would let half a token survive the flag unnoticed.
const BOARD_ELEMENT: [&str; 6] = ["tile", "tile-face", "fog-shroud", "prop", "token", "beat"];

/// What the flag deliberately allows to stand in `.board` beside them.
const ALLOWED: [&str; 2] = ["marker", "scene-board"];

/// One walk of the board containers: how many there were, and what they held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Census {
    containers: usize,
    elements: [usize; BOARD_ELEMENT.len()],
    allowed: [usize; ALLOWED.len()],
}

impl Census {
    /// The number the flag drives to zero.
    fn total(&self) -> usize {
        self.elements.iter().sum()
    }

    fn line(&self, arm: &str) -> String {
        let counted = BOARD_ELEMENT
            .iter()
            .zip(self.elements)
            .map(|(class, n)| format!(".{class} {n}"))
            .collect::<Vec<_>>()
            .join(", ");
        let allowed = ALLOWED
            .iter()
            .zip(self.allowed)
            .map(|(class, n)| format!(".{class} {n}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "[isometry-b5] {arm}: {} .board container(s) hold {counted} = {} board elements; \
             allowed beside them: {allowed}",
            self.containers,
            self.total(),
        )
    }
}

/// A laid-out board on the arm asked for, with one token selected so B4's
/// allowed marker is actually in the tree rather than absent by accident.
///
/// `shrouded` looks through a player's eyes and then shortens the light, which
/// is the only way to get a remembered tile: explored memory is folded in the
/// instant a tile is seen, so a first wide look leaves no shroud behind it.
fn board(scene: bool, shrouded: bool) -> BoardHarness {
    let mut ui = UiState::new(demo_map());
    ui.camera = (420.0, 140.0);
    ui.viewport = (WINDOW.0 - PANEL_W, WINDOW.1);
    ui.scene_board = scene;
    ui.select_token(TokenId(1));
    if shrouded {
        ui.cycle_viewer();
        ui.recompute_fog();
        ui.sight_radius = 3;
        ui.recompute_fog();
    }
    let mut hooks = inert_hooks();
    hooks.key_intercept = Box::new(hooks::key_intercept);
    hooks.focused_text = Box::new(hooks::focused_text);
    let mut harness = Harness::with_hooks(
        Init {
            state: ui,
            logic: board_root as Logic,
            sheet: board_css(),
            fonts: Vec::new(),
            images: Vec::new(),
        },
        hooks,
    );
    harness.layout_at(WINDOW.0, WINDOW.1);
    harness
}

/// Walk every `.board` container's subtree and count it by class. The
/// container itself is not counted; everything below it is.
fn census(harness: &BoardHarness) -> Census {
    harness.with_dom(|dom| {
        let mut out = Census {
            containers: 0,
            elements: [0; BOARD_ELEMENT.len()],
            allowed: [0; ALLOWED.len()],
        };
        for root in taproot::matching(dom, &Selector::class("board")) {
            out.containers += 1;
            let mut stack: Vec<_> = dom.dom_children(root).collect();
            while let Some(node) = stack.pop() {
                for (index, class) in BOARD_ELEMENT.iter().enumerate() {
                    if dom.has_class(node, class) {
                        out.elements[index] += 1;
                    }
                }
                for (index, class) in ALLOWED.iter().enumerate() {
                    if dom.has_class(node, class) {
                        out.allowed[index] += 1;
                    }
                }
                stack.extend(dom.dom_children(node));
            }
        }
        out
    })
}

/// The control. The DOM board puts one box per tile, face, prop and token in
/// the container, which is what §1 counted and what the flag has to remove.
/// Only the shroud is absent, because an unfogged board has nothing to
/// remember; [`a_shroud_is_an_element_on_one_arm_and_a_tint_on_the_other`]
/// covers that class.
#[test]
fn the_dom_board_fills_its_container_with_one_element_per_piece() {
    let harness = board(false, false);
    let census = census(&harness);
    eprintln!("{}", census.line("DOM board"));
    assert_eq!(
        census.containers, 1,
        "the DOM board has one .board container"
    );
    for (class, count) in BOARD_ELEMENT.iter().zip(census.elements) {
        // Every class the walk claims to count is present, or its zero on the
        // other arm means nothing.
        if *class != "fog-shroud" {
            assert!(count > 0, "the DOM board draws .{class} elements");
        }
    }
    assert!(
        census.total() > 500,
        "the demo board is a few hundred elements, got {}",
        census.total()
    );
    assert_eq!(
        census.allowed[0], 1,
        "one selected token, so one DOM marker under it"
    );
    assert_eq!(census.allowed[1], 0, "and no scene leaf without the flag");
}

/// The receipt. Under the flag nothing in the board container names a tile, a
/// face, a shroud, a prop or a token — only B4's marker and the one leaf.
#[test]
fn the_scene_board_emits_no_board_elements() {
    let harness = board(true, false);
    let census = census(&harness);
    eprintln!("{}", census.line("scene board"));
    assert_eq!(
        census.containers, 2,
        "the leaf's container plus the markers' panned one"
    );
    for (class, count) in BOARD_ELEMENT.iter().zip(census.elements) {
        assert_eq!(count, 0, "the scene board emits no .{class} element");
    }
    assert_eq!(census.total(), 0, "zero board elements under the flag");
    assert_eq!(census.allowed[0], 1, "B4's marker is the allowed exception");
    assert_eq!(census.allowed[1], 1, "and the board itself is one leaf");
}

/// The sixth class, and the one most likely to come back by accident: B4 made
/// remembered ground a shrouded twin of the tile's own material, so under fog
/// the scene arm has to stay at zero where the DOM arm grows an overlay
/// diamond per remembered tile.
#[test]
fn a_shroud_is_an_element_on_one_arm_and_a_tint_on_the_other() {
    let dom = census(&board(false, true));
    eprintln!("{}", dom.line("DOM board, shrouded"));
    let shroud = BOARD_ELEMENT.iter().position(|c| *c == "fog-shroud");
    assert!(
        dom.elements[shroud.expect("the shroud is a counted class")] > 0,
        "a shortened light leaves remembered ground the DOM draws a shroud over"
    );

    let scene = census(&board(true, true));
    eprintln!("{}", scene.line("scene board, shrouded"));
    assert_eq!(
        scene.total(),
        0,
        "the scene board shrouds through the terrain palette, not the tree"
    );
}
