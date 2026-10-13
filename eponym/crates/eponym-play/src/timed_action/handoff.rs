// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::contract::mode::{self,SpatialBindingInput};
use mode::actions::{self,ActionConfig,handoff::ImportedState};

impl TimedActionSession {
    /// Exact pre-action mode source for an explicitly authored ActionConfig.
    /// No native replay, mutation, or replacement owner is constructed.
    pub fn mode_source(&self,input:&SpatialBindingInput) -> Result<String,String> {
        let prior = self.save().map_err(|e|format!("timed Eponym handoff save: {e:?}"))?;
        let mode = mode::bind(self.session.game().world().session(),&prior,input)?;
        Ok(actions::game_source(&mode))
    }

    /// Moves the one native Session after complete borrowed preflight. Any
    /// refusal returns every original paid/prepared product and native value.
    pub fn into_native_mode(self,input:&SpatialBindingInput,config:&ActionConfig)
        -> Result<crate::NativeHandoff,(Self,String)> {
        let prepared = (|| {
            let prior = self.save().map_err(|e|format!("timed Eponym handoff save: {e:?}"))?;
            let game = self.session.game();
            let native = game.world().session();
            for profile in &config.profiles {
                if game.bodies().get(profile.subject).is_none_or(|body|body.revision != profile.revision) {
                    return Err("handoff cannot relabel the original current game body revision".into());
                }
            }
            let bytes = mode::bind(native,&prior,input)?;
            let imported = ImportedState {products:game.handoff_products(),site:game.world().site(),
                origin:crate::walking::to_site(game.world().volume(),[0;3])};
            let configured = actions::handoff::prepare(native,&bytes,config,self.session.control().played(),
                &self.network,&self.rules,self.action.as_ref(),imported)?;
            Ok::<_,String>((prior,configured))
        })();
        let (prior,game) = match prepared {Ok(values)=>values,Err(why)=>return Err((self,why))};
        let Self {session,network,rules,action} = self;
        match session.into_native_mode_with_prior(input,prior) {
            Ok(mut handoff)=> {handoff.game = game;Ok(handoff)},
            Err((session,why))=>Err((Self {session,network,rules,action},why)),
        }
    }
}
