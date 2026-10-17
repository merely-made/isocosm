// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The S3 scene: the settled three go out, and one comes back changed
//! toward another.
//!
//! Continuity all the way down: the world is S0's seed and S0's hillside,
//! the society is S2's settled state (Bram in the gatehouse, Sela in the
//! still-room, Odris living as before), and the sortie departs from the
//! surface above the carved chamber. Aud negotiates Bram's scouting for
//! this expedition as a fresh bounded agreement; Sela's part rides her
//! standing settlement agreement, which also carries the tag-in pact.
//!
//! The sortie authors a native material scarp in the grown site. Its walk
//! crosses a six-cell drop, so the fall remains a world fact that the receipt
//! can replay alongside the tag-in, tend, and return.

use eponym_client::room::{Room, SEED};
use eponym_play::identity::{Control, SubjectId, Tick};
use eponym_play::vocabulary::{ABANDONED, HEALING, SCOUTING};
use eponym_play::{WorldIntent, walking};
use isocosm::social::{DeedKind, Response, Terms, Work};
use isometer_space::{Edit, Op, edit::Shape, volume::Volume};

use crate::march;
use crate::party::{self, Pact};
use crate::settled::{self, AUD, BRAM, ODRIS, SELA};
use crate::society::Society;
use crate::sortie::Sortie;

pub fn outing() -> Work {
    Work::new(SCOUTING, 3, 4)
}

pub const OUTING_TERMS: Terms = Terms {
    share: 2,
    danger_cap: 5,
};

/// Tending the fallen: inside Sela's standing settlement agreement.
pub fn tend() -> Work {
    Work::new(HEALING, 3, 3)
}

/// Departure day. The settled scene's last deed is tick 12.
pub const DEPART: Tick = Tick(13);

/// Where the sortie is headed, relative to home, in x and z. Calibrated:
/// this heading crosses a scarp the walker can only descend as a forced
/// drop, which is the hazard the receipt depends on, and the site itself
/// sits in the trench below it.
pub const SITE_OFFSET: [i32; 2] = [-15, -8];

/// A six-cell drop across the departure route, cut into the native site.
const SCARP_DROP: i32 = 6;

/// The settled society: S2's people, homes formed, twelve deeds deep.
pub fn settled_society() -> Society {
    let mut room = Room::grow(SEED).expect("S0's seed has a hillside");
    let home = march::stand(room.world.volume(), room.centre[0], room.centre[2])
        .expect("the hill above the room has a surface to stand on");
    let edit = Edit {
        op: Op::Carve,
        shape: Shape::Box {
            min: walking::to_site(room.world.volume(), [home[0] - 12, home[1] - SCARP_DROP, home[2] - 11]),
            max: walking::to_site(room.world.volume(), [home[0] - 5, home[1] + 16, home[2] - 3]),
        },
    };
    room.world.apply(WorldIntent::MaterialEdit { tick: Tick(0), edit })
        .expect("the scarp is an authored native material edit");
    room.ground = room.world.ground().clone();
    let mut society = Society::on(room.world);
    settled::admit(&mut society).expect("the four arrive and Aud's record is laid");
    settled::answers(&mut society).expect("the settling scene admits everyone");
    settled::housings(&mut society).expect("the settled offers hold");
    society
}

pub fn muster(society: Society) -> (Vec<Response>, Sortie) {
    let volume = society.world().volume().clone();
    let home = home_stance(&volume);
    let site = site_stance(&volume);
    let way = way_home(home);
    muster_at(society, home, site, way)
}

fn muster_at(
    mut society: Society,
    home: [i32; 3],
    site: [i32; 3],
    way_home: Vec<[i32; 2]>,
) -> (Vec<Response>, Sortie) {
    // Sela's part and the pact ride her standing settlement agreement.
    let sela = society.id(SELA);
    let sela_home = society
        .agreements()
        .find(|agreement| agreement.holder == sela && agreement.standing())
        .map(|agreement| agreement.id)
        .expect("the settled scene housed Sela");
    let healer = party::healer_part(&society, sela_home, &tend())
        .expect("her standing agreement covers the tending");

    // Bram's part is negotiated fresh, and the answer is his.
    let (answer, scout) =
        party::negotiate_scout(&mut society, AUD, BRAM, outing(), OUTING_TERMS, DEPART)
            .expect("the muster asks people the society knows");

    let mut parts = Vec::new();
    let mut outing = None;
    if let Some(part) = scout {
        outing = Some(part.under());
        parts.push((BRAM, part));
    }
    parts.push((SELA, healer));

    let sortie = Sortie::muster(
        society,
        Control::begin(AUD, DEPART),
        &[AUD, BRAM, ODRIS, SELA],
        parts,
        Some(Pact {
            under: sela_home,
            successor: SELA,
        }),
        outing,
        tend(),
        home,
        site,
        way_home,
        DEPART,
    );
    (vec![answer], sortie)
}

pub fn way_home(home: [i32; 3]) -> Vec<[i32; 2]> {
    WAY_OFFSETS
        .iter()
        .map(|offset| [home[0] + offset[0], home[2] + offset[1]])
        .chain([[home[0], home[2]]])
        .collect()
}

/// Detour waypoints of the way home, relative to home. Calibrated with
/// `SITE_OFFSET` by the survey in `tests/calibrate.rs`.
pub const WAY_OFFSETS: [[i32; 2]; 1] = [[-16, 16]];

/// The surface above the carved chamber: the settlement's ground.
pub fn home_stance(volume: &Volume) -> [i32; 3] {
    let room = Room::grow(SEED).expect("S0's seed has a hillside");
    march::stand(volume, room.centre[0], room.centre[2])
        .expect("the hill above the room has a surface to stand on")
}

/// The far site, standing on the edited surface at the scarp's far side.
pub fn site_stance(volume: &Volume) -> [i32; 3] {
    let home = home_stance(volume);
    march::stand(volume, home[0] + SITE_OFFSET[0], home[2] + SITE_OFFSET[1])
        .expect("the site column has a surface to stand on")
}

/// The whole run: settled society, muster, march to done.
pub fn played_through() -> (Vec<Response>, Sortie) {
    let (answers, mut sortie) = muster(settled_society());
    sortie.run();
    (answers, sortie)
}

/// A run at an arbitrary site offset: the calibration probe's door. The
/// shipped scene is `played_through`; this exists so `SITE_OFFSET` is
/// chosen from a printed survey of the real terrain rather than by hand.
pub fn surveyed(offset: [i32; 2], way_offsets: &[[i32; 2]]) -> Option<(Vec<Response>, Sortie)> {
    let society = settled_society();
    let volume = society.world().volume().clone();
    let home = home_stance(&volume);
    let site = march::stand(&volume, home[0] + offset[0], home[2] + offset[1])?;
    let way = way_offsets
        .iter()
        .map(|way| [home[0] + way[0], home[2] + way[1]])
        .chain([[home[0], home[2]]])
        .collect();
    let (answers, mut sortie) = muster_at(society, home, site, way);
    sortie.run();
    Some((answers, sortie))
}

pub fn grudged() -> (Vec<Response>, Sortie) {
    let mut society = settled_society();
    society
        .record(Tick(DEPART.0 - 1), AUD, Some(BRAM), DeedKind::new(ABANDONED))
        .expect("Aud's desertion is recorded");
    let (answers, mut sortie) = muster(society);
    sortie.run();
    (answers, sortie)
}

/// The ask S2 saw counteroffered, put again after the sortie. Same work,
/// same terms, one expedition later.
pub fn ask_again(society: &mut Society, at: Tick) -> Response {
    let offer = settled::ask_of(society, SELA);
    society.consider(&offer, at).expect("Sela is still of the society")
}

/// Odris stayed home and stays home: a subject with no part.
pub fn the_stay_at_home() -> SubjectId {
    ODRIS
}
