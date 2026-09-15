// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The status lines and the control-help line, and the keymap the help line is
//! derived from (M3 of the isomere plan).
//!
//! Three shapes, all of them lines of words a host publishes and nobody
//! clicks:
//!
//! - **A status line**: one live region with the product's own id —
//!   `#specimen-status` on the bench, `#selection` and `#journal-summary` in
//!   the session. [`status_line`] is [`error_line`](crate::error_line)'s
//!   sibling and keeps the id for the same reason: both products' scenarios
//!   read it by id.
//! - **A status panel**: a heading over a stack of lines, which the session's
//!   `.status-lines` is. Isometry's `.side-status` did *not* move: it is a
//!   bare `div` with no id, styled by a sheet that is not isomere's, so it
//!   moves when M4 puts Isometry on the shared host and sheet.
//! - **The control-help line**: `#controls-help`, the one line that says what
//!   the keys do.
//!
//! # Why the help line carries a keymap and not a string
//!
//! The session had the same key list written twice: once as the `match` in its
//! key handler, once as a sentence in its header. Nothing connected them, so a
//! binding could move and the line would go on claiming the old one. §2.2 of
//! the plan asks for the help line to come from a *declared* keymap, and
//! [`Keymap`] is that declaration: every binding carries the words it reads as
//! **and** the presses it claims, [`Keymap::help`] joins the words, and
//! [`Keymap::command`] is what the handler dispatches through. A key the
//! keymap does not claim cannot be handled, and a key it claims that the
//! handler ignores is a missing match arm.
//!
//! **The press type is the product's.** isomere depends on `cambium` and not
//! on `cambium-rootstock`, whose `Key` is a *different* type from Cambium's
//! and which would drag `wgpu` and `netrender` into a crate that emits
//! elements. So `Keymap` is generic over whatever a product's key press
//! compares as; the product lowers its own platform key into it, in one place,
//! which is a normalization rather than a second mapping table.
//!
//! **Nothing in Cambium covered this.** `command_surface`'s [`CommandItem`]
//! carries a `shortcut: Option<String>` and is the closest thing, but it is a
//! *palette*: a filterable overlay of activatable rows with its own selection
//! and dismiss state. A help line is a sentence, and a keymap is a dispatch
//! table; neither is that surface. There is no keymap, status-line or
//! help-line helper in the family at 876320fd. Read and not adopted, as M2
//! recorded for `detail_panel`.
//!
//! [`CommandItem`]: cambium::CommandItem

use crate::viewport::Child;
use cambium::{el, text};

/// The class a [`status_line`] carries beside its id. The shared sheet does
/// not style it: the three status lines are three different sizes in two
/// products, so their rules stay the products' own and this is the selector a
/// later milestone reaches for.
pub const STATUS_LINE_CLASS: &str = "status-line";

/// The class a [`status_panel`]'s stack of lines carries. The session's own.
pub const STATUS_LINES_CLASS: &str = "status-lines";

/// The class a [`help_line`] carries beside its id. The M0 sheet already
/// styles `#controls-help, .controls-help` together, so this changes no pixel.
pub const HELP_CLASS: &str = "controls-help";

/// Every attribute a [`status_line`] carries, in order.
pub fn status_attrs(id: &str) -> Vec<(&'static str, String)> {
    vec![
        ("class", STATUS_LINE_CLASS.to_owned()),
        ("id", id.to_owned()),
        ("role", "status".to_owned()),
    ]
}

/// One published line: a live region the host writes a reading into.
pub fn status_line<State, Action>(id: &str, words: impl Into<String>) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let mut line = el("p", text(words.into()));
    for (name, value) in status_attrs(id) {
        line = line.attr(name, value);
    }
    Box::new(line)
}

/// What a product has to say about its status panel.
#[derive(Clone, Debug, Default)]
pub struct StatusPanel<'a> {
    /// The panel's heading. `None` for a product that arranges its own.
    pub title: Option<&'a str>,
    /// The lines, in the order the product reads them.
    pub lines: Vec<String>,
    /// The panel's `class`. `None` is no class at all.
    pub class: Option<&'a str>,
    /// The stack's `class`. `None` is [`STATUS_LINES_CLASS`].
    pub lines_class: Option<&'a str>,
}

impl<'a> StatusPanel<'a> {
    /// A panel titled `title` with nothing in it yet.
    pub fn new(title: &'a str) -> Self {
        Self {
            title: Some(title),
            ..Self::default()
        }
    }
}

/// The whole panel: a `section` carrying the heading over the stack of lines.
///
/// The lines are plain paragraphs rather than [`status_line`]s: they have no
/// ids of their own, and one live region over the stack is what a screen
/// reader wants rather than a dozen competing ones.
pub fn status_panel<State, Action>(panel: &StatusPanel<'_>) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let lines: Vec<Child<State, Action>> = panel
        .lines
        .iter()
        .map(|line| Box::new(el("p", text(line.clone()))) as Child<State, Action>)
        .collect();
    let mut children: Vec<Child<State, Action>> = Vec::with_capacity(2);
    if let Some(title) = panel.title {
        children.push(Box::new(el("h2", text(title.to_owned()))));
    }
    children.push(Box::new(el("div", lines).attr(
        "class",
        panel.lines_class.unwrap_or(STATUS_LINES_CLASS).to_owned(),
    )));
    let section = el("section", children);
    Box::new(match panel.class {
        Some(class) => section.attr("class", class),
        None => section,
    })
}

/// One entry in a [`Keymap`]: how it reads, and every press it claims.
///
/// `keys` and `action` are two halves of one phrase so the help line reads as
/// a sentence — `"WASD"` plus `"move"`. An entry may claim several presses
/// (the four movement keys are one phrase), or none at all, which is how a
/// pointer verb the help line has to name still lives in the declaration.
pub struct Binding<'a, Press, Command> {
    /// How the keys read in the help line. The product's own spelling.
    pub keys: &'a str,
    /// What they do, in the product's words.
    pub action: &'a str,
    /// Each press this entry claims and the command it sends.
    pub presses: Vec<(Press, Command)>,
}

impl<'a, Press, Command> Binding<'a, Press, Command> {
    /// An entry claiming one press.
    pub fn one(keys: &'a str, action: &'a str, press: Press, command: Command) -> Self {
        Self {
            keys,
            action,
            presses: vec![(press, command)],
        }
    }

    /// An entry whose several presses read as one phrase.
    pub fn group(keys: &'a str, action: &'a str, presses: Vec<(Press, Command)>) -> Self {
        Self {
            keys,
            action,
            presses,
        }
    }

    /// An entry the help line names and the keyboard does not reach.
    pub fn said(keys: &'a str, action: &'a str) -> Self {
        Self {
            keys,
            action,
            presses: Vec::new(),
        }
    }

    /// How this entry reads on its own, with `infix` between the keys and what
    /// they do — a space in the session's line, a colon and a space in
    /// Isometry's crib.
    pub fn words(&self, infix: &str) -> String {
        format!("{}{infix}{}", self.keys, self.action)
    }
}

/// A product's declared keymap: the one place its keys are written down.
pub struct Keymap<'a, Press, Command> {
    /// The entries, in the order the help line reads them.
    pub bindings: Vec<Binding<'a, Press, Command>>,
    /// What joins two entries in the help line.
    pub separator: &'a str,
    /// What joins an entry's keys to what they do.
    pub infix: &'a str,
}

impl<'a, Press, Command> Keymap<'a, Press, Command> {
    /// The separator both products already wrote their help line with.
    pub const SEPARATOR: &'static str = " · ";

    /// What joins an entry's keys to what they do, unless a product says
    /// otherwise.
    pub const INFIX: &'static str = " ";

    /// A keymap over `bindings`, joined by [`Keymap::SEPARATOR`] and
    /// [`Keymap::INFIX`].
    pub fn new(bindings: Vec<Binding<'a, Press, Command>>) -> Self {
        Self {
            bindings,
            separator: Self::SEPARATOR,
            infix: Self::INFIX,
        }
    }

    /// The help line's text: every entry's phrase, in order.
    pub fn help(&self) -> String {
        self.help_where(|_| true)
    }

    /// The same, narrowed to the entries `keep` admits.
    ///
    /// A product whose help line is a *crib* rather than a full list — the one
    /// or two keys that do something the buttons do not already say — names
    /// those entries here rather than writing the sentence a second time.
    pub fn help_where(&self, keep: impl Fn(&Binding<'a, Press, Command>) -> bool) -> String {
        self.bindings
            .iter()
            .filter(|binding| keep(binding))
            .map(|binding| binding.words(self.infix))
            .collect::<Vec<_>>()
            .join(self.separator)
    }

    /// The command `press` sends, or `None` when no binding claims it.
    ///
    /// This is what a handler dispatches through, so a key that is not in the
    /// declaration above cannot be handled at all.
    pub fn command(&self, press: &Press) -> Option<&Command>
    where
        Press: PartialEq,
    {
        self.bindings
            .iter()
            .flat_map(|binding| binding.presses.iter())
            .find(|(claimed, _)| claimed == press)
            .map(|(_, command)| command)
    }

    /// Every press the keymap claims, in declaration order. For a product test
    /// that wants to prove its handler answers all of them.
    pub fn presses(&self) -> impl Iterator<Item = &Press> {
        self.bindings
            .iter()
            .flat_map(|binding| binding.presses.iter())
            .map(|(press, _)| press)
    }
}

/// Every attribute a [`help_line`] carries, in order.
pub fn help_attrs(id: &str) -> Vec<(&'static str, String)> {
    vec![("class", HELP_CLASS.to_owned()), ("id", id.to_owned())]
}

/// The control-help line, from the words a [`Keymap`] already carries.
///
/// Not a live region: it says what the keys are, which does not change while
/// anyone is reading it.
pub fn help_line<State, Action>(id: &str, words: impl Into<String>) -> Child<State, Action>
where
    State: 'static,
    Action: 'static,
{
    let mut line = el("p", text(words.into()));
    for (name, value) in help_attrs(id) {
        line = line.attr(name, value);
    }
    Box::new(line)
}
