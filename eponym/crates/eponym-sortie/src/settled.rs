// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! S1's refusal scene and S2's settling, the society the sortie departs
//! from, kept here as this receipt's own fixture once Eponym's social family
//! moved onto Isocosm (wing rulings 669, 672 and 755). Aud's record with the
//! three is laid as deeds at ticks one to seven; the three answer the same
//! ask at eight; Bram and Sela are housed at eleven and twelve.

use eponym_play::identity::{SubjectId, Tick};
use isocosm::schema::Id;
use isocosm::social::{Craft, DeedKind, Offer, Response, Terms, Work};

use crate::society::{SocialError, Society};

pub const AUD: SubjectId = SubjectId(1);
pub const BRAM: SubjectId = SubjectId(2);
pub const ODRIS: SubjectId = SubjectId(3);
pub const SELA: SubjectId = SubjectId(4);
pub const THE_THREE: [SubjectId; 3] = [BRAM, ODRIS, SELA];

pub const BRAM_ASK: Work = Work { craft: Craft::Scouting, grade: 2, danger: 2 };
pub const ODRIS_ASK: Work = Work { craft: Craft::Watching, grade: 3, danger: 2 };
pub const SELA_ASK: Work = Work { craft: Craft::Healing, grade: 3, danger: 4 };
pub const THE_KEEP: Terms = Terms { share: 2, danger_cap: 4 };

/// The four, admitted to `society`, and Aud's deeds toward the three.
pub fn admit(society: &mut Society) -> Result<(), SocialError> {
    society.admit(AUD, "Aud", &[(Craft::Scouting, 2)], 0)?;
    society.admit(BRAM, "Bram", &[(Craft::Scouting, 4)], 0)?;
    society.admit(ODRIS, "Odris", &[(Craft::Scouting, 5), (Craft::Watching, 3)], 1)?;
    society.admit(SELA, "Sela", &[(Craft::Scouting, 3), (Craft::Healing, 4)], 3)?;
    for (tick, toward, kind) in [
        (1, BRAM, DeedKind::StoodBy),
        (2, BRAM, DeedKind::StoodBy),
        (3, ODRIS, DeedKind::Shared),
        (4, ODRIS, DeedKind::Abandoned),
        (5, SELA, DeedKind::Shared),
        (6, SELA, DeedKind::Shared),
        (7, SELA, DeedKind::StoodBy),
    ] {
        society.record(Tick(tick), AUD, Some(toward), kind)?;
    }
    Ok(())
}

pub fn ask_of(society: &Society, subject: SubjectId) -> Offer {
    let work = match subject {
        BRAM => BRAM_ASK,
        ODRIS => ODRIS_ASK,
        _ => SELA_ASK,
    };
    society.offer(AUD, subject, work, THE_KEEP)
}

/// The three answer Aud's ask, at eight, nine and ten.
pub fn answers(society: &mut Society) -> Result<Vec<Response>, SocialError> {
    let mut out = vec![];
    for (subject, tick) in THE_THREE.iter().zip(8..) {
        let offer = ask_of(society, *subject);
        out.push(society.consider(&offer, Tick(tick))?);
    }
    Ok(out)
}

pub fn sela_settled_ask(society: &Society) -> Offer {
    let work = Work { danger: 3, ..SELA_ASK };
    society.offer(AUD, SELA, work, Terms { share: 3, danger_cap: 3 })
}

/// The gatehouse and the still-room, and Bram and Sela housed in them.
pub fn housings(society: &mut Society) -> Result<[Id; 2], SocialError> {
    let gatehouse = society.found_dwelling("the-gatehouse", "the gatehouse")?;
    let still_room = society.found_dwelling("the-still-room", "the still-room")?;
    let bram = ask_of(society, BRAM);
    society.offer_home(&bram, gatehouse, Tick(11))?;
    let sela = sela_settled_ask(society);
    society.offer_home(&sela, still_room, Tick(12))?;
    Ok([gatehouse, still_room])
}
