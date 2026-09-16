//! The board's plain-key verbs, declared once (M3 of the isomere plan).
//!
//! These keys used to be written down twice and in two crates: as a chain of
//! `if` tests in `isometry-genet`'s `key_intercept`, and as a sentence in the
//! side panel's key crib. Nothing connected them, so a binding could move and
//! the crib would go on claiming the old one — with a crate boundary in
//! between to make sure nobody noticed.
//!
//! [`KEYMAP`] is the one declaration. The host dispatches through
//! [`Keymap::command`] and the panel reads [`key_hint`], so the sentence
//! cannot outlive a binding. Only the *plain* verbs live here: the context
//! gates above them in `key_intercept` — target-pick, an open menu, a focused
//! text lane, the compendium, the creation panel — are policy about what is on
//! screen rather than a key mapping, and they stay where they are and still
//! run first.
//!
//! [`Press`] is this crate's own, because isomere names no platform's key
//! vocabulary: the host lowers its `KeyPress` into one of these, in one place.

use std::sync::LazyLock;

use isomere::{Binding, Keymap};

/// One press as the declaration compares it.
///
/// Character keys carry the command chord because `z` and `Ctrl+Z` are two
/// different bindings; named keys do not, because the host never read one for
/// them and an arrow with Ctrl held has always panned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Press {
    /// A character key with no command chord held. Case-sensitive: the host
    /// compares what the layout produced, which is what it always did.
    Plain(String),
    /// The same key with Ctrl held.
    Ctrl(String),
    /// A named key the board reads.
    Named(NamedPress),
}

impl Press {
    /// A character key with no chord.
    pub fn plain(key: &str) -> Self {
        Self::Plain(key.to_owned())
    }

    /// A character key with Ctrl held.
    pub fn ctrl(key: &str) -> Self {
        Self::Ctrl(key.to_owned())
    }
}

/// The named keys the board claims. Every other one falls through to the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedPress {
    Enter,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

/// What a press asks the board to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKey {
    Command,
    Whisper,
    Face,
    FogView,
    /// Step the focus elevation up one, then off (B4).
    Focus,
    EndTurn,
    Undo,
    Redo,
    /// Pan by one tile in column and row. The host widens these to the
    /// fractional step `pan_tiles` takes.
    Pan(i8, i8),
}

/// The entries the key crib carries, by the keys they read as.
///
/// The crib is deliberately short — it is the one state where a key does
/// something other than what the buttons already say (the panel diet's §3.4) —
/// so it names *which* bindings it shows and never what they say. The words
/// are the declaration's.
const CRIB: [&str; 4] = ["arrows", "r", "enter", "f"];

/// The board's plain-key verbs, in the order the crib reads them.
///
/// Declaration order is the crib's order rather than the old `if` chain's,
/// which costs nothing: every press below is claimed exactly once, so the
/// order the host tries them in cannot change what a key does.
pub static KEYMAP: LazyLock<Keymap<'static, Press, BoardKey>> = LazyLock::new(|| Keymap {
    separator: " / ",
    infix: ": ",
    bindings: vec![
        Binding::group(
            "arrows",
            "pan",
            vec![
                (Press::Named(NamedPress::ArrowLeft), BoardKey::Pan(-1, 1)),
                (Press::Named(NamedPress::ArrowRight), BoardKey::Pan(1, -1)),
                (Press::Named(NamedPress::ArrowUp), BoardKey::Pan(-1, -1)),
                (Press::Named(NamedPress::ArrowDown), BoardKey::Pan(1, 1)),
            ],
        ),
        Binding::one("r", "face", Press::plain("r"), BoardKey::Face),
        Binding::one(
            "enter",
            "end turn",
            Press::Named(NamedPress::Enter),
            BoardKey::EndTurn,
        ),
        Binding::one("f", "fog view", Press::plain("f"), BoardKey::FogView),
        Binding::one("c", "focus height", Press::plain("c"), BoardKey::Focus),
        // Below the crib: the two lane openers and the history chord, which the
        // panel's own buttons already say.
        Binding::one(">", "command line", Press::plain(">"), BoardKey::Command),
        Binding::one("w", "whisper", Press::plain("w"), BoardKey::Whisper),
        Binding::one("ctrl+z", "undo", Press::ctrl("z"), BoardKey::Undo),
        Binding::one("ctrl+y", "redo", Press::ctrl("y"), BoardKey::Redo),
    ],
});

/// The side panel's key crib, read off [`KEYMAP`].
pub fn key_hint() -> String {
    KEYMAP.help_where(|binding| CRIB.contains(&binding.keys))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The crib reads exactly what the panel used to spell by hand.
    #[test]
    fn the_crib_is_the_sentence_the_panel_used_to_carry() {
        assert_eq!(
            key_hint(),
            "arrows: pan / r: face / enter: end turn / f: fog view"
        );
    }

    /// Every key the old `if` chain tested dispatches, and nothing else does.
    #[test]
    fn every_plain_verb_is_claimed_exactly_once() {
        for (press, want) in [
            (Press::plain(">"), BoardKey::Command),
            (Press::plain("w"), BoardKey::Whisper),
            (Press::plain("r"), BoardKey::Face),
            (Press::plain("f"), BoardKey::FogView),
            (Press::plain("c"), BoardKey::Focus),
            (Press::Named(NamedPress::Enter), BoardKey::EndTurn),
            (Press::ctrl("z"), BoardKey::Undo),
            (Press::ctrl("y"), BoardKey::Redo),
            (Press::Named(NamedPress::ArrowUp), BoardKey::Pan(-1, -1)),
        ] {
            assert_eq!(KEYMAP.command(&press), Some(&want), "{press:?}");
        }
        assert_eq!(KEYMAP.command(&Press::plain("z")), None);
        assert_eq!(
            KEYMAP.command(&Press::ctrl("r")),
            None,
            "a chord the declaration never claimed is not the plain key"
        );
        assert_eq!(KEYMAP.presses().count(), 12);
    }
}
