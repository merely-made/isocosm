// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Intake ports as part data (wing ruling 766): what an intake organ admits,
//! kept on its part, active while the function supporting it holds cells
//! there; and the feeding mode read from a body's active ports, as
//! Mesocosm reads it. Geometry gives a body its founding ports only.

use crate::kingdom;
use crate::process::{FeedingMode, IntakePort, NisKind, Process, Registry};
use crate::schema::*;

/// The function a port's supporting definition binds, natively.
fn supporting(port: IntakePort) -> Option<Key> {
    let support = port.support()?;
    let def = Registry::native().resolve(support)?;
    Some(format!("function:{}", def.id.name))
}

/// `p`'s port where it is active: declared, and either unsupported or its
/// supporting function holding cells on the part.
pub fn active(p: &Part) -> Option<IntakePort> {
    let port = p.port?;
    let held = |f: Key| p.cells.get(&f).copied().unwrap_or(0) > 0;
    match port.support() {
        None => Some(port),
        Some(_) => supporting(port).filter(|f| held(f.clone())).map(|_| port),
    }
}

/// The body's active ports, merged.
pub fn ports(e: &Entity) -> IntakePort {
    let kinds = [NisKind::Producer, NisKind::Consumer, NisKind::Decomposer];
    e.living()
        .filter_map(|(_, p)| active(p))
        .fold(IntakePort::none(), |all, port| {
            let mut merged = all;
            for kind in kinds.into_iter().filter(|k| port.admits_live(*k)) {
                merged = merged.with_live(kind);
            }
            if port.admits_deadstock() {
                merged = merged.with_deadstock();
            }
            merged
        })
}

/// What the body does with matter, read from its active ports.
pub fn feeding(e: &Entity) -> FeedingMode {
    let ports = ports(e);
    let flora = ports.admits_live(NisKind::Producer);
    let fauna = ports.admits_live(NisKind::Consumer) || ports.admits_live(NisKind::Decomposer);
    match (flora, fauna, ports.admits_deadstock()) {
        (true, true, _) => FeedingMode::Omnivore,
        (true, false, _) => FeedingMode::Grazer,
        (false, true, _) => FeedingMode::Predator,
        (false, false, true) => FeedingMode::Scavenger,
        (false, false, false) => FeedingMode::Producer,
    }
}

/// The founding ports geometry declares, as Mesocosm seeds them: none on a
/// body holding a canopy; a bulk mouth admitting producer matter and a jaw
/// consumer matter, each under the head; and, with no mouth, the root
/// taking up the dead. A part already declaring a port keeps it.
pub fn seed(e: &mut Entity) {
    let Some(doc) = e.body.as_ref() else { return };
    let registry = Registry::native();
    let by = |p: Process| registry.of_native(p).reference();
    if kingdom::of(e) == Some(kingdom::Kingdom::Producer) {
        return;
    }
    let mouths: Vec<(PartId, IntakePort)> = doc
        .living()
        .filter(|p| {
            p.attachment
                .is_some_and(|a| a.parent == doc.root && a.offset[1] < 0)
        })
        .filter_map(|p| match isometer_core::classify(p.half_extent) {
            isometer_core::Role::Mass => Some((
                p.id,
                IntakePort::live(NisKind::Producer).supported_by(by(Process::Intake)),
            )),
            isometer_core::Role::Limb => Some((
                p.id,
                IntakePort::live(NisKind::Consumer).supported_by(by(Process::Contract)),
            )),
            _ => None,
        })
        .collect();
    let declared = match mouths.is_empty() {
        true => vec![(
            doc.root,
            IntakePort::deadstock().supported_by(by(Process::Intake)),
        )],
        false => mouths,
    };
    for (id, port) in declared {
        if let Some(part) = e.parts.get_mut(&id) {
            part.port.get_or_insert(port);
        }
    }
}
