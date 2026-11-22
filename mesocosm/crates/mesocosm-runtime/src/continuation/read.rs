// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{Session, Simulation, directing::readings::{self, View}, schema::{Id, Method}};
use cosmograph_overlay::{OrdinaryAccess, EntityHandle};
use crate::succession::Occasion;

pub(crate) fn watch_view(sim: &Simulation, critter: Id) -> View {
    let state = sim.state();
    let site = state.population.get(critter)
        .and_then(|body| body.resident_site().or_else(|| body.journey.map(|j| j.from)))
        .or_else(|| state.sites.keys().next().copied());
    site_view(sim, site)
}

pub(crate) fn site_view(sim: &Simulation, site: Option<Id>) -> View {
    let state = sim.state();
    let mut view = View::default();
    if let Some(site) = site {
        view.sites.insert(site);
        view.things.extend(state.population.groups.iter().filter(|(_, group)|
            group.entity.resident_site() == Some(site)
                && !cosmograph::directing::is_participant(&group.entity))
            .map(|(&first, group)| (first, group.count)));
    }
    view
}

pub fn read_loop(session: &Session, game: &Game, access: OrdinaryAccess, visible: &View)
    -> Result<LoopReading, String> {
    game.validate(session)?;
    if session.sim.plays(access.participant.0) != access.subject.map(|id| id.0) {
        return Err("loop reading names another played subject".into());
    }
    let mut out = LoopReading { cut: cut(session, game), settings: None,
        phase: LoopPhase::Running, checkpoint: None, offers: Vec::new(),
        happenings: Vec::new(), lineages: Vec::new(), starts: Vec::new(), personal: None, world_ended_at: None,
        ending: Default::default(), sheet: None, held: Vec::new(), holdable: Vec::new() };
    (out.held, out.holdable) = super::holding::reading(&session.sim, access.participant.0);
    let Some(saved) = &game.continuation else { return Ok(out); };
    if saved.participant != access.participant.0 {
        return Err("saved loop belongs to another participant".into());
    }
    out.settings = Some(saved.settings);
    out.ending = saved.ending;
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
        // A holder who lost the body keeps the heirs of its own line on the
        // list though the dead eye sees none of them: otherwise a line whose
        // kin all live out of sight leaves the question with no answer.
        let own_loss = matches!(&checkpoint.occasion, Occasion::Loss(loss) if origin == Some(loss.critter) || access.subject.map(|id| id.0) == Some(loss.critter));
        checkpoint.heirs.retain(|id| own_loss || visible.shows(*id));
        // This is a visibility projection, never the authoritative saved hold.
        if let Occasion::Loss(loss) = &mut checkpoint.occasion {
            loss.next = loss.next.filter(|id| own_loss || visible.shows(*id));
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
                cost: offer.cost, sheet: offer.sheet.clone(),
            }).collect();
            out.sheet = review.sheet.clone();
        }
    }
    out.happenings = saved.happenings.iter().filter_map(|h| match h {
        cosmograph::directing::interim::Happening::Born { parent, child }
            if (visible.shows(*parent) || origin == Some(*parent)) && visible.shows(*child) => Some(h.clone()),
        cosmograph::directing::interim::Happening::Died { critter, next }
            if visible.shows(*critter) && next.is_none_or(|id| visible.shows(id)) => Some(h.clone()),
        cosmograph::directing::interim::Happening::Boundary { tick, .. } =>
            Some(cosmograph::directing::interim::Happening::Boundary { tick: *tick, turns: Vec::new() }),
        // What befell what the participant holds (cases 1224, 1226).
        cosmograph::directing::interim::Happening::Surfaced(_) | cosmograph::directing::interim::Happening::Raised(_) => Some(h.clone()),
        cosmograph::directing::interim::Happening::LineageEnded { lineage } if session.sim.is_surface(lineage) => Some(h.clone()),
        cosmograph::directing::interim::Happening::LineageEnded { .. }
            if saved.phase == LoopPhase::LineageEnded && out.checkpoint.is_some() => Some(h.clone()),
        cosmograph::directing::interim::Happening::WorldEnded { .. }
            if saved.phase == LoopPhase::WorldEnded => Some(h.clone()),
        _ => None,
    }).collect();
    if saved.world_end.is_none() && matches!(saved.phase, LoopPhase::LineageEnded | LoopPhase::Watching) {
        let watching = saved.watched_critter.map(|id| watch_view(&session.sim, id))
            .or_else(|| saved.founding_site.map(|site| site_view(&session.sim, Some(site))));
        if let Some(view) = watching {
            out.lineages = view.things.keys().filter(|&&id| {
                session.sim.state().population.get(id)
                    .is_some_and(|e| e.alive && e.method == Method::Deliberative)
                    && (game.driver.is_none() || game.members.get(&id)
                        .is_some_and(|at| at.realized(&session.sim).is_ok()))
            })
                .copied().map(EntityHandle).collect();
            return Ok(out);
        }
        // A known line is offered too, seen or not (case 1210); none is known unless the world sets regard.
        let known = readings::known(session, access.participant.0);
        out.lineages = session.sim.state().population.groups.iter().filter(|(first, group)| {
            let e = &group.entity;
            (visible.shows(**first) || known.contains(&e.lineage)) && e.alive && e.method == Method::Deliberative
                && saved.ended_lineage.as_ref().is_some_and(|lineage| *lineage != e.lineage)
        }).map(|(first, _)| EntityHandle(*first)).collect();
        let kept: std::collections::BTreeSet<&str> = out.lineages.iter()
            .filter_map(|id| session.sim.state().population.get(id.0).map(|e| e.lineage.as_str())).collect();
        out.starts = kept.into_iter().flat_map(|lineage| readings::starts(&session.sim, lineage).into_iter()
            .map(move |at| LoopStart { lineage: lineage.to_string(), site: at.site,
                critter: EntityHandle(at.critter), journey: at.journey })).collect();
    }
    Ok(out)
}
