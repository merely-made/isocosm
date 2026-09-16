// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! M4's host assembly, as far as a windowless test can reach it.
//!
//! The seven hooks only run inside an event loop, so what is testable here is
//! the assembly's *declarations*: that a product with nothing to say compiles
//! and assembles, that the defaults are the ones the module claims, and that
//! the shared producer-error wording is the sentence the bench and the session
//! already wrote. The behaviour under a window is the products' own headed
//! receipts, which is where M4's done-condition lives.

use cambium::{AnyView, GenetCtx, GenetElement, el, text};

use crate::host::{
    Assembly, Ctx, NoProducer, Product, ScenarioLane, VIEWPORT_ERROR, viewport_error,
};
use cambium_genet_winit_host::ProducerError;

/// A product state with one error line and nothing else.
#[derive(Default)]
struct Quiet {
    published: Option<String>,
}

type Child = Box<dyn AnyView<Quiet, (), GenetCtx, GenetElement>>;
type Logic = fn(&Quiet) -> Child;

fn root(state: &Quiet) -> Child {
    Box::new(el("p", text(state.published.clone().unwrap_or_default())))
}

/// The smallest product the trait admits: three types, a prefix, and the two
/// error accessors. Everything else defaults.
struct Silent;

impl Product for Silent {
    type State = Quiet;
    type Logic = Logic;
    type View = Child;
    type Producer = NoProducer;

    const LOG_PREFIX: &'static str = "isomere-test";

    fn published_error(state: &Quiet) -> Option<&str> {
        state.published.as_deref()
    }

    fn publish_error(state: &mut Quiet, error: Option<String>) {
        state.published = error;
    }
}

/// A lane that records nothing and wants no frames, so the assembly's
/// "a live lane keeps frames coming" can be told from a product's own answer.
struct Spent;

impl ScenarioLane<Silent> for Spent {
    fn after_frame(&mut self, _ctx: &mut Ctx<'_, Silent>) {}

    fn wants_frames(&self) -> bool {
        false
    }
}

#[test]
fn the_shared_wording_is_the_sentence_both_sources_wrote() {
    let words = viewport_error(ProducerError::InvalidExtent);
    assert!(words.starts_with(VIEWPORT_ERROR), "{words}");
    assert_eq!(words, "Viewport unavailable: InvalidExtent");
}

#[test]
fn a_product_that_says_nothing_still_assembles() {
    // Nothing runs without an event loop; what this proves is that the seven
    // closures typecheck over a product with no producer, no lane and no
    // capture, which is the Isometry shape.
    let _hooks = Assembly::new(Silent).hooks();
}

#[test]
fn lanes_and_a_capture_policy_ride_the_same_assembly() {
    let _hooks = Assembly::new(Silent)
        .with_lane(Spent)
        .with_optional_lane(None::<Spent>)
        .with_capture(None)
        .hooks();
}

#[test]
fn the_error_accessors_are_the_products_own() {
    let mut state = Quiet::default();
    assert_eq!(Silent::published_error(&state), None);
    Silent::publish_error(
        &mut state,
        Some("Viewport unavailable: InvalidTexture".into()),
    );
    assert_eq!(
        Silent::published_error(&state),
        Some("Viewport unavailable: InvalidTexture")
    );
    // The root view reads it, so a published error is a DOM change and not a
    // field nobody shows.
    let _ = root(&state);
}

#[test]
fn defaults_claim_no_leaf_and_no_close() {
    assert!(!Silent::closing(&Quiet::default()));
    assert_eq!(Silent.producer_properties(0), &["color"]);
    assert!(
        Silent
            .viewport_error(0, ProducerError::DuplicateDomKey)
            .is_none()
    );
    assert!(!Silent.profiling());
}

/// A product with nowhere to put the error line, which is Isometry's shape:
/// its scene board reports a producer refusal on stderr and its side panel has
/// no error element.
struct Mute;

impl Product for Mute {
    type State = Quiet;
    type Logic = Logic;
    type View = Child;
    type Producer = NoProducer;

    const LOG_PREFIX: &'static str = "isomere-test";
    const PUBLISHES_ERROR: bool = false;
}

#[test]
fn a_product_publishes_the_error_line_unless_it_says_otherwise() {
    // The default is to publish, because two of the three wing hosts do.
    assert!(Silent::PUBLISHES_ERROR);
    // Opting out is what stops the assembly calling `runner.update` with
    // nothing to write, once per frame, for as long as an error stands.
    assert!(!Mute::PUBLISHES_ERROR);
    let _hooks = Assembly::new(Mute).hooks();
}
