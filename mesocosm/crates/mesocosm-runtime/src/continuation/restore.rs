// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::succession::{self, Occasion};
use isocosm::{Session, Command, lineage::Review};

pub(crate) fn validate_question(saved: &Continuation, session: &Session) -> Result<(), String> {
    let held = matches!(saved.phase, LoopPhase::Held | LoopPhase::LineageEnded);
    if held != saved.checkpoint.is_some() {
        return Err("saved loop phase differs from its outstanding question".into());
    }
    let Some(checkpoint) = &saved.checkpoint else {
        if saved.review.is_some() || !saved.proposed.is_empty() {
            return Err("saved review has no outstanding question".into());
        }
        return Ok(());
    };
    interval(saved, session)?;
    if checkpoint.tick != session.sim.state().tick
        || succession::opened_for(session, saved.participant, &saved.happenings).as_ref() != Some(checkpoint) {
        return Err("saved question differs from this round's actual native cut".into());
    }
    match &checkpoint.occasion {
        Occasion::Birth(birth) => {
            let state = session.sim.state();
            if state.population.get(birth.parent).is_none() || state.population.get(birth.child).is_none()
                || !state.relations.iter().any(|r| r.subject == birth.parent
                    && r.object == birth.child && r.kind == "sim:child") {
                return Err("saved birth offer lacks its native parent/child relation".into());
            }
        },
        Occasion::Loss(loss) => {
            let lost = session.sim.state().population.get(loss.critter).ok_or("saved lost member absent")?;
            if lost.alive || checkpoint.heirs != succession::heirs(session, loss.critter, loss.next)
                || (saved.phase == LoopPhase::LineageEnded
                    && (!checkpoint.heirs.is_empty() || saved.ended_lineage.as_ref() != Some(&lost.lineage))) {
                return Err("saved death/lineage hold lacks its native loss/cohort".into());
            }
        },
        Occasion::Epoch(boundary) => {
            let played = session.sim.plays(saved.participant).ok_or("held review has no played member")?;
            let member = session.sim.state().population.get(played).ok_or("held member absent")?;
            if member.lineage != boundary.lineage { return Err("held review names another lineage".into()); }
            let review = saved.review.as_ref().ok_or("saved epoch hold has no review")?;
            // Forecast numbers grant nothing. The exact current offer repertoire,
            // commands, refusal reasons and native reading must still match.
            let actual = Review::of_using(session, &boundary.lineage, saved.settings.pace.scoring,
                &mut |_, _, candidate, _| {
                    review.offers.iter().find(|offer| offer.name == candidate.name
                        && offer.commands == candidate.commands).map(|offer| offer.score)
                        .ok_or_else(|| "saved review omits an actual native candidate".into())
                })?;
            if &actual != review { return Err("saved review differs from the native offered repertoire".into()); }
            if !saved.proposed.is_empty() && saved.expression.is_none() {
                return Err("saved expression commands lack their authored source".into());
            }
            for proposal in &saved.proposed {
                if proposal.offer >= review.offers.len() { return Err("saved expression has no native offer".into()); }
                for command in &proposal.commands {
                    match command {
                        Command::Express { entity, part, .. } if *entity == played
                            && member.parts.contains_key(part) => {},
                        _ => return Err("saved expression is not the held body's actual part expression".into()),
                    }
                }
            }
        },
    }
    if !matches!(&checkpoint.occasion, Occasion::Epoch(_))
        && (saved.review.is_some() || !saved.proposed.is_empty()) {
        return Err("saved non-epoch question contains a revision".into());
    }
    Ok(())
}

fn interval(saved: &Continuation, session: &Session) -> Result<(), String> {
    let source = saved.round.as_ref().ok_or("saved question lacks its actual round source")?;
    let index = session.entries.iter().position(|e| e.id == source.first_entry)
        .ok_or("saved round's native entry absent")?;
    if source.played.is_none() || session.entries[index].tick != source.from {
        return Err("saved round starts at another native tick".into());
    }
    let mut cursor = source.from;
    for entry in &session.entries[index..] {
        if let Command::AdvanceSpatial { ticks, .. } | Command::AdvanceBirthSpatial { ticks, .. }
            = &entry.command {
            if entry.tick != cursor { return Err("saved round has a discontinuous physical clock".into()); }
            cursor = cursor.checked_add(*ticks).ok_or("saved round tick overflow")?;
        }
    }
    if cursor != session.sim.state().tick
        || cursor.checked_sub(source.from) != Some(saved.settings.pace.round) {
        return Err("saved question lacks the exact declared physical round".into());
    }
    if let Some(checkpoint) = &saved.checkpoint
        && matches!(&checkpoint.occasion, Occasion::Epoch(_)) {
        let epoch = session.sim.genesis().rules.epoch_ticks;
        if epoch == 0 || cursor / epoch <= source.from / epoch {
            return Err("saved review has no actual crossed native epoch".into());
        }
    }
    Ok(())
}

/// Historical source verification occurs on resume or an actual answer,
/// never by re-running a native world for each frame's borrowed read.
pub(crate) fn validate_round(saved: &Continuation, session: &Session) -> Result<(), String> {
    let Some(checkpoint) = &saved.checkpoint else { return Ok(()); };
    interval(saved, session)?;
    let source = saved.round.as_ref().ok_or("saved round source absent")?;
    let before = session.before_entry(&source.first_entry)?;
    if super::prefix(&before) != source.native {
        return Err("saved question references another authentic native prefix".into());
    }
    let played = before.sim.plays(saved.participant).ok_or("round source has no played member")?;
    if source.played != Some(played) {
        return Err("saved personal origin differs from actual native control".into());
    }
    match &checkpoint.occasion {
        Occasion::Birth(birth) if birth.parent == played => {
            if before.sim.state().relations.iter().any(|r| r.subject == birth.parent
                && r.object == birth.child && r.kind == "sim:child") {
                return Err("saved offspring already existed before the round".into());
            }
        },
        Occasion::Loss(loss) if loss.critter == played => {
            if !before.sim.state().population.get(played).is_some_and(|e| e.alive) {
                return Err("saved lost member was not alive at round start".into());
            }
        },
        Occasion::Epoch(boundary) if before.sim.state().population.get(played)
            .is_some_and(|e| e.lineage == boundary.lineage) => {},
        _ => return Err("saved checkpoint does not belong to this round's actual played member".into()),
    }
    Ok(())
}

pub(crate) fn proposals(saved: &Continuation, session: &Session) -> Result<Vec<SavedProposal>, String> {
    let Some(source) = &saved.expression else {
        if saved.proposed.is_empty() { return Ok(Vec::new()); }
        return Err("saved expression has no authored source".into());
    };
    let authored = crate::review::Authored::from_saved(source)?;
    authored.validate_world(session.sim.genesis())?;
    let Some(review) = &saved.review else { return Ok(Vec::new()); };
    let played = session.sim.plays(saved.participant).ok_or("expression has no played member")?;
    let body = session.sim.state().population.get(played).ok_or("expression body absent")?;
    let held = session.sim.state().lineages.get(&body.lineage)
        .and_then(|l| l.development.as_ref()).map(|d| d.tracts.iter()
            .map(|t| crate::review::qualified(&t.function)).collect::<Vec<_>>()).unwrap_or_default();
    Ok(authored.propose(review, body, &held, session.sim.genesis().seed).iter()
        .map(|p| SavedProposal { offer: p.offer, script: p.script.clone(),
            cells: p.cells.clone(), commands: p.commands(played) }).collect())
}
