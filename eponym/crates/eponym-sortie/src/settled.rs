// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! S1's refusal scene and S2's settling, the society the sortie departs
//! from, kept here as this receipt's own fixture once Eponym's social family
//! moved onto Isocosm (wing rulings 669, 672 and 755). Aud's record with the
//! three is laid as deeds at ticks one to seven; the three answer the same
//! ask at eight; Bram and Sela are housed at eleven and twelve.

use eponym_play::identity::{SubjectId, Tick};
use isocosm::schema::Id;
use eponym_play::vocabulary::{ABANDONED, HEALING, SCOUTING, SHARED, STOOD_BY, WATCHING};
use isocosm::social::{DeedKind, Offer, Response, Terms, Work};

use crate::society::{SocialError, Society};

pub const AUD: SubjectId = SubjectId(1);
pub const BRAM: SubjectId = SubjectId(2);
pub const ODRIS: SubjectId = SubjectId(3);
pub const SELA: SubjectId = SubjectId(4);
pub const THE_THREE: [SubjectId; 3] = [BRAM, ODRIS, SELA];

pub fn bram_ask() -> Work {
    Work::new(SCOUTING, 2, 2)
}
pub fn odris_ask() -> Work {
    Work::new(WATCHING, 3, 2)
}
pub fn sela_ask() -> Work {
    Work::new(HEALING, 3, 4)
}
pub const THE_KEEP: Terms = Terms { share: 2, danger_cap: 4 };

/// The four, admitted to `society`, and Aud's deeds toward the three.
pub fn admit(society: &mut Society) -> Result<(), SocialError> {
    society.admit(AUD, "Aud", &[(SCOUTING, 2)], 0)?;
    society.admit(BRAM, "Bram", &[(SCOUTING, 4)], 0)?;
    society.admit(ODRIS, "Odris", &[(SCOUTING, 5), (WATCHING, 3)], 1)?;
    society.admit(SELA, "Sela", &[(SCOUTING, 3), (HEALING, 4)], 3)?;
    for (tick, toward, kind) in [
        (1, BRAM, STOOD_BY),
        (2, BRAM, STOOD_BY),
        (3, ODRIS, SHARED),
        (4, ODRIS, ABANDONED),
        (5, SELA, SHARED),
        (6, SELA, SHARED),
        (7, SELA, STOOD_BY),
    ] {
        society.record(Tick(tick), AUD, Some(toward), DeedKind::new(kind))?;
    }
    Ok(())
}

pub fn ask_of(society: &Society, subject: SubjectId) -> Offer {
    let work = match subject {
        BRAM => bram_ask(),
        ODRIS => odris_ask(),
        _ => sela_ask(),
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
    let work = Work { danger: 3, ..sela_ask() };
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
