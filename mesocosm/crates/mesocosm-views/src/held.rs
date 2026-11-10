// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Words for what a participant holds (cases 1216, 1221, 1223, 1224, 1226):
//! a held lineage's standing, and what befell it, in plain terms. A kingdom is
//! a plant, an animal, a fungus, a microbe, or, of the world, a world-class
//! lineage.

use cosmograph::{directing::readings::Raised, spread::{Identity, Substrate, Surfaced}};
use mesocosm_runtime::continuation::HeldLine;

/// How many characters of a lineage's key the words keep.
const SHORT_KEY: usize = 8;

/// A kingdom as a player reads it.
pub fn kingdom_word(kingdom: &str) -> &'static str {
    match kingdom.rsplit(':').next().unwrap_or(kingdom) {
        "flora" => "plant",
        "fauna" => "animal",
        "myco" => "fungus",
        "micro" => "microbe",
        _ => "world-class lineage",
    }
}

/// A lineage key without its prefixes, cut short.
pub fn short_key(key: &str) -> String {
    key.rsplit(':').next().unwrap_or(key).chars().take(SHORT_KEY).collect()
}

fn count(n: u64, one: &str, many: &str) -> String { format!("{n} {}", if n == 1 { one } else { many }) }

fn gene(traits: &std::collections::BTreeSet<String>, name: &str) -> String {
    format!("{name} {}", if traits.contains(&format!("gene:{name}")) { "on" } else { "off" })
}

/// The standing of one held lineage, for the pane's line: a surface with
/// where its matter lies and what its genes do, or the world's turn.
pub fn held_words(line: &HeldLine, kingdom: &str) -> String {
    match line {
        HeldLine::Surface(held) => {
            let mut places = Vec::new();
            for (tally, one, many, within) in [(held.sites, "site", "sites", "on"), (held.places, "place", "places", "in"),
                (held.hosts, "host", "hosts", "in")] {
                if tally.count > 0 {
                    places.push(format!("{} {within} {}", tally.total, count(tally.count, one, many)));
                }
            }
            if held.transit > 0 { places.push(format!("{} on the way", held.transit)); }
            let at = if held.ended { "ended".to_string() } else { places.join(", ") };
            let genes = ["rot", "enter", "feed", "leak"].map(|name| gene(&held.traits, name)).join(", ");
            format!("{} {}: {at}; {genes}", kingdom_word(kingdom), short_key(&held.lineage))
        },
        HeldLine::World { lineage, shift, bound } => match bound {
            Some(bound) => format!("climate shift {shift:+} of \u{b1}{bound} ({})", short_key(lineage)),
            None => format!("climate shift {shift:+}, which cannot be turned ({})", short_key(lineage)),
        },
    }
}

fn place(at: &Substrate) -> String {
    match at {
        Substrate::Site(id) => format!("site {id}"),
        Substrate::Place(place) => format!("a place at site {}", place.site),
        Substrate::Host(id) => format!("host {id}"),
    }
}

fn name(identity: &Identity, kingdom: &dyn Fn(&str) -> String) -> String {
    let key = identity.key();
    match identity {
        Identity::Surface(lineage) => format!("{} {}", kingdom_word(&kingdom(lineage)), short_key(lineage)),
        Identity::Territory(_) => format!("territory {}", short_key(&key)),
    }
}

/// What befell a held surface, in a sentence. `kingdom` says the kingdom of a
/// lineage by its key.
pub fn surfaced_words(event: &Surfaced, kingdom: &dyn Fn(&str) -> String) -> String {
    match event {
        Surfaced::Reached { identity, at } => format!("{} reached {}", name(identity, kingdom), place(at)),
        Surfaced::Left { identity, at } => format!("{} left {}", name(identity, kingdom), place(at)),
        Surfaced::Brimmed { identity, at } => format!("{} filled {} to its capacity", name(identity, kingdom), place(at)),
        Surfaced::HostDied { identity, host } => format!("host {host} of {} died", name(identity, kingdom)),
        Surfaced::Met { identity, other, at } =>
            format!("{} met {} at {}", name(identity, kingdom), name(other, kingdom), place(at)),
    }
}

/// What the world's conditions raised for the holder of a world-class lineage.
pub fn raised_words(raised: &Raised, kingdom: &dyn Fn(&str) -> String) -> String {
    let line = |key: &str| format!("{} {}", kingdom_word(&kingdom(key)), short_key(key));
    match raised {
        Raised::Extinct { lineage } => format!("{} has no living members left", line(lineage)),
        Raised::Fell { sites } => {
            let sites: Vec<String> = sites.iter().map(u64::to_string).collect();
            format!("a region fell: sites {}", sites.join(", "))
        },
        Raised::Barren { site } => format!("site {site} ran out of soil"),
        Raised::Bloomed { lineage, held } => format!("{} doubled its living matter, to {held} mg", line(lineage)),
        Raised::Turned { rising } => format!("the year turned: now {}", if *rising { "warming" } else { "cooling" }),
    }
}

/// A held surface's end.
pub fn held_ended_words(lineage: &str, kingdom: &str) -> String {
    format!("{} {} holds no matter anywhere: it has ended", kingdom_word(kingdom), short_key(lineage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmograph::directing::readings::{Held, Tally};

    fn held(sites: (u64, u64), hosts: (u64, u64), rot: bool) -> HeldLine {
        let traits = if rot { ["gene:rot", "gene:enter"].map(String::from).into() } else { Default::default() };
        HeldLine::Surface(Held { lineage: "lineage:micro".into(), traits, revision: 1,
            sites: Tally { count: sites.0, total: sites.1 }, places: Tally::default(),
            hosts: Tally { count: hosts.0, total: hosts.1 }, transit: 0, ended: false })
    }

    #[test]
    fn a_held_surface_says_where_it_lies_and_what_its_genes_do() {
        assert_eq!(held_words(&held((2, 40), (3, 12), true), "kingdom:micro"),
            "microbe micro: 40 on 2 sites, 12 in 3 hosts; rot on, enter on, feed off, leak off");
        assert_eq!(held_words(&held((1, 5), (0, 0), false), "kingdom:micro"),
            "microbe micro: 5 on 1 site; rot off, enter off, feed off, leak off");
        let HeldLine::Surface(mut gone) = held((0, 0), (0, 0), true) else { unreachable!() };
        gone.ended = true;
        assert!(held_words(&HeldLine::Surface(gone), "kingdom:micro").contains(": ended;"));
    }

    #[test]
    fn the_world_class_lineage_says_its_shift_and_bound() {
        let world = |shift, bound| HeldLine::World { lineage: "world:ground".into(), shift, bound };
        assert_eq!(held_words(&world(2, Some(5)), "kingdom:world"), "climate shift +2 of \u{b1}5 (ground)");
        assert_eq!(held_words(&world(-1, Some(5)), "kingdom:world"), "climate shift -1 of \u{b1}5 (ground)");
        assert!(held_words(&world(0, None), "kingdom:world").contains("cannot be turned"));
    }

    #[test]
    fn kingdoms_are_read_as_plain_names() {
        let words: Vec<_> = ["kingdom:flora", "kingdom:fauna", "kingdom:myco", "kingdom:micro", "kingdom:world"]
            .iter().map(|k| kingdom_word(k)).collect();
        assert_eq!(words, ["plant", "animal", "fungus", "microbe", "world-class lineage"]);
    }

    #[test]
    fn each_happening_has_words() {
        let micro = Identity::Surface("lineage:micro".into());
        let kingdom = |_: &str| "kingdom:micro".to_string();
        let at = Substrate::Host(12);
        for (event, words) in [
            (Surfaced::Reached { identity: micro.clone(), at }, "microbe micro reached host 12"),
            (Surfaced::Left { identity: micro.clone(), at: Substrate::Site(3) }, "microbe micro left site 3"),
            (Surfaced::Brimmed { identity: micro.clone(), at: Substrate::Site(3) }, "microbe micro filled site 3 to its capacity"),
            (Surfaced::HostDied { identity: micro.clone(), host: 12 }, "host 12 of microbe micro died"),
            (Surfaced::Met { identity: micro.clone(), other: Identity::Surface("lineage:strain:abcdef0123".into()), at },
                "microbe micro met microbe abcdef01 at host 12"),
        ] {
            assert_eq!(surfaced_words(&event, &kingdom), words);
        }
        let plain = |key: &str| if key == "lineage:1" { "kingdom:fauna".into() } else { "kingdom:world".into() };
        assert_eq!(raised_words(&Raised::Extinct { lineage: "lineage:1".into() }, &plain), "animal 1 has no living members left");
        assert_eq!(raised_words(&Raised::Barren { site: 4 }, &plain), "site 4 ran out of soil");
        assert_eq!(raised_words(&Raised::Fell { sites: [1, 2].into() }, &plain), "a region fell: sites 1, 2");
        assert_eq!(raised_words(&Raised::Turned { rising: true }, &plain), "the year turned: now warming");
        assert_eq!(raised_words(&Raised::Bloomed { lineage: "lineage:1".into(), held: 9 }, &plain),
            "animal 1 doubled its living matter, to 9 mg");
        assert!(held_ended_words("lineage:micro", "kingdom:micro").contains("has ended"));
    }
}
