// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{Session, directing::readings::View, schema::Method};
use cosmograph_overlay::{OrdinaryAccess, EntityHandle};
use crate::succession::Occasion;

pub fn read_loop(session: &Session, game: &Game, access: OrdinaryAccess, visible: &View)
    -> Result<LoopReading, String> {
    game.validate(session)?;
    if session.sim.plays(access.participant.0) != access.subject.map(|id| id.0) {
        return Err("loop reading names another played subject".into());
    }
    let mut out = LoopReading { cut: cut(session, game), settings: None,
        phase: LoopPhase::Running, checkpoint: None, offers: Vec::new(),
        happenings: Vec::new(), lineages: Vec::new(), personal: None, world_ended_at: None };
    let Some(saved) = &game.continuation else { return Ok(out); };
    if saved.participant != access.participant.0 {
        return Err("saved loop belongs to another participant".into());
    }
    out.settings = Some(saved.settings);
    out.phase = saved.phase;
    out.world_ended_at = saved.world_end.as_ref().map(|source| source.tick);
    let origin = saved.round.as_ref().and_then(|source| source.played);
    out.personal = saved.checkpoint.as_ref().and_then(|c| match &c.occasion {
        Occasion::Birth(b) if origin == Some(b.parent) => Some(PersonalQuestion::Birth { parent: b.parent }),
        Occasion::Loss(l) if origin == Some(l.critter) => Some(PersonalQuestion::Loss { critter: l.critter }),
        Occasion::Epoch(_) => access.subject.map(|id| PersonalQuestion::Epoch { entity: id.0 }),
        _ => None,
    });
    out.checkpoint = saved.checkpoint.clone().filter(|checkpoint| match &checkpoint.occasion {
        Occasion::Birth(birth) => (visible.shows(birth.parent) || origin == Some(birth.parent))
            && visible.shows(birth.child),
        Occasion::Loss(loss) => visible.shows(loss.critter) || origin == Some(loss.critter),
        Occasion::Epoch(_) => access.subject.is_some_and(|id| visible.shows(id.0)),
    });
    if let Some(checkpoint) = &mut out.checkpoint {
        checkpoint.heirs.retain(|id| visible.shows(*id));
        // This is a visibility projection, never the authoritative saved hold.
        if let Occasion::Loss(loss) = &mut checkpoint.occasion {
            loss.next = loss.next.filter(|id| visible.shows(*id));
        }
        if let Occasion::Epoch(boundary) = &mut checkpoint.occasion {
            // The saved forecast is not a disclosure of unplayed lineages.
            boundary.turns.clear();
        }
    }
    if out.checkpoint.as_ref().is_some_and(|c| matches!(&c.occasion, Occasion::Epoch(_))) {
        if let Some(review) = &saved.review {
            out.offers = review.offers.iter().enumerate().map(|(index, offer)| LoopOffer {
                index, name: offer.name.clone(), score: offer.score, why_not: offer.why_not.clone(),
            }).collect();
        }
    }
    out.happenings = saved.happenings.iter().filter_map(|h| match h {
        cosmograph::directing::interim::Happening::Born { parent, child }
            if (visible.shows(*parent) || origin == Some(*parent)) && visible.shows(*child) => Some(h.clone()),
        cosmograph::directing::interim::Happening::Died { critter, next }
            if visible.shows(*critter) && next.is_none_or(|id| visible.shows(id)) => Some(h.clone()),
        cosmograph::directing::interim::Happening::Boundary { tick, .. } =>
            Some(cosmograph::directing::interim::Happening::Boundary { tick: *tick, turns: Vec::new() }),
        cosmograph::directing::interim::Happening::LineageEnded { .. }
            if saved.phase == LoopPhase::LineageEnded && out.checkpoint.is_some() => Some(h.clone()),
        cosmograph::directing::interim::Happening::WorldEnded { .. }
            if saved.phase == LoopPhase::WorldEnded => Some(h.clone()),
        _ => None,
    }).collect();
    if saved.world_end.is_none() && matches!(saved.phase, LoopPhase::LineageEnded | LoopPhase::Watching) {
        out.lineages = session.sim.state().population.groups.iter().filter(|(first, group)| {
            let e = &group.entity;
            visible.shows(**first) && e.alive && e.method == Method::Deliberative
                && saved.ended_lineage.as_ref().is_some_and(|lineage| *lineage != e.lineage)
        }).map(|(first, _)| EntityHandle(*first)).collect();
    }
    Ok(out)
}
