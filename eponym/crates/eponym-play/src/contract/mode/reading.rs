// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::{directing::readings::View, schema::BodyDocument};
use isometer_space::volume::Volume;

pub struct PictureBody {
    pub subject: SubjectId, pub entity: Id, pub document: BodyDocument,
    pub revision: u64, pub pose: MotionPose, pub alive: bool,
}

/// A derived snapshot for the GPU consumer, containing no runtime or ledger.
pub struct Picture {
    pub site: Id, pub volume: Volume, pub bodies: Vec<PictureBody>,
    pub played: Option<Id>, pub presentation: Presentation, pub fingerprint: [u64;4],
}

pub struct BorrowedReading<'a> { session: &'a Session, visible: &'a View, state: &'a ModeState }

impl<'a> BorrowedReading<'a> {
    pub fn new(session: &'a Session, visible: &'a View, state: &'a ModeState) -> Result<Self,String> {
        state.validate()?;
        Ok(Self {session,visible,state})
    }

    pub fn rows(&self) -> Vec<(SubjectId,Id,String)> {
        self.state.bindings.iter().filter(|binding| self.visible.shows(binding.entity)).filter_map(|binding| {
            let body = self.session.sim.state().population.get(binding.entity)?;
            Some((binding.subject,binding.entity,body.authored.as_ref().map(|a| a.name.clone())
                .unwrap_or_else(|| body.lineage.clone())))
        }).collect()
    }

    /// Actual native PLAYS is mapped explicitly; subject numbers infer nothing.
    pub fn played(&self, participant: Id) -> Result<Option<SubjectId>,String> {
        let Some(entity) = self.session.sim.plays(participant) else {return Ok(None);};
        self.state.bindings.iter().find(|binding| binding.entity == entity)
            .map(|binding| Some(binding.subject)).ok_or("played native body has no Eponym subject binding".into())
    }

    pub fn picture(&self, site: Id, frame: &Window, participant: Id, require_played: bool) -> Result<Picture,String> {
        if !self.visible.sites.contains(&site) {return Err("Eponym picture site is outside native awareness".into());}
        let volume = self.session.sim.volume(site,frame.min,frame.max,frame.depth)?;
        let played = self.session.sim.plays(participant);
        let mut bodies = Vec::new();
        for binding in &self.state.bindings {
            if binding.site != site || !self.visible.shows(binding.entity) {continue;}
            let pose = binding.in_frame(self.session,&volume)?;
            let body = self.session.sim.state().population.get(binding.entity).ok_or("Eponym picture entity is absent")?;
            let document = body.body.clone().ok_or("Eponym picture has no actual native body geometry")?;
            let cell = crate::walking::to_site(&volume,pose.cell().map_err(|e|format!("Eponym framed pose: {e:?}"))?);
            if !volume.holds(cell[0],cell[2]) || cell[1] < volume.datum || cell[1] >= volume.ceiling() {continue;}
            bodies.push(PictureBody {subject:binding.subject,entity:binding.entity,document,
                revision:body.body_revision,pose,alive:body.alive});
        }
        if bodies.is_empty() || require_played && played.is_none_or(|id|!bodies.iter().any(|body|body.entity == id)) {
            return Err("Eponym picture lacks current bindings for its admitted played region".into());
        }
        let digest = isocosm::digest(&(source(self.session),self.visible,site,frame,self.state,played));
        let mut fingerprint = [0;4];
        for (i,word) in fingerprint.iter_mut().enumerate() {
            *word = u64::from_str_radix(&digest[i*16..(i+1)*16],16).map_err(|e|e.to_string())?;
        }
        Ok(Picture {site,volume,bodies,played,presentation:self.state.presentation.clone(),fingerprint})
    }
}
