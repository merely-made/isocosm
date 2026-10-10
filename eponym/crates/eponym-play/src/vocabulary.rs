// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym's social vocabulary, declared in its rules pack (wing ruling 778):
//! the crafts its peers hold and the deeds that move standing, each with
//! its weight on trust and liking and how it is said. The sim names only
//! the seven deeds asks and agreements record; Eponym weighs them too.

use isocosm::social::{self, DeedRule, Vocabulary};

pub const SCOUTING: &str = "craft:scouting";
pub const SMITHING: &str = "craft:smithing";
pub const HEALING: &str = "craft:healing";
pub const HAULING: &str = "craft:hauling";
pub const WATCHING: &str = "craft:watching";

pub const STOOD_BY: &str = "deed:stood-by";
pub const ABANDONED: &str = "deed:abandoned";
pub const SHARED: &str = "deed:shared";
pub const TOOK: &str = "deed:took";

/// Eponym's crafts and deeds.
pub fn vocabulary() -> Vocabulary {
    let deeds = [
        (STOOD_BY, 3, 2, "stood by me"),
        (ABANDONED, -5, -3, "left me"),
        (SHARED, 1, 2, "shared with me"),
        (TOOK, -2, -2, "took from me"),
        (social::OFFER_ACCEPTED, 1, 0, "took up my offer"),
        (social::OFFER_REFUSED, 0, 0, "turned my offer down"),
        (social::OFFER_COUNTERED, 0, 0, "answered my offer with terms of their own"),
        (social::AGREEMENT_FORMED, 1, 1, "agreed a standing arrangement with me"),
        (social::PERFORMED, 2, 1, "did the agreed work"),
        (social::RENEGOTIATED, 0, 0, "changed our terms"),
        (social::AGREEMENT_ENDED, 0, -1, "ended our arrangement"),
    ];
    Vocabulary {
        crafts: [SCOUTING, SMITHING, HEALING, HAULING, WATCHING].map(String::from).into(),
        deeds: deeds
            .into_iter()
            .map(|(k, trust, affinity, phrase)| (k.into(), DeedRule { trust, affinity, phrase: phrase.into() }))
            .collect(),
    }
}
