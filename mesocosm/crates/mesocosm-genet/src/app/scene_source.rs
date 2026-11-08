// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Derived scene dependencies and the optional component-store comparison.

use std::rc::Rc;

use cosmograph::directing::readings::Mode;
use cosmograph::schema::Id;
use cosmograph_host::world::NativeRevision;
use mesocosm_runtime::Runtime;

use crate::{played::components::ComponentScene, section::SiteScene};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SceneBranch {
    Physical,
    Coarse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SceneSourceKey {
    pub native: NativeRevision,
    pub played: Option<Id>,
    pub site: Id,
    pub branch: SceneBranch,
    pub components: bool,
    mode: Mode,
    spatial: Option<String>,
}

impl SceneSourceKey {
    pub fn new(
        runtime: &Runtime,
        native: NativeRevision,
        played: Option<Id>,
        site: Id,
        branch: SceneBranch,
        components: bool,
    ) -> Self {
        let spatial = (branch == SceneBranch::Physical).then(|| {
            let game = runtime.spatial();
            // Poses, native walk records and sight configuration reach the
            // picture. The full ecology witness and continuation do not.
            cosmograph::digest(&(
                game.version,
                &game.members,
                &game.driver,
                &game.sight,
                game.continuation.is_some(),
                game.ecology.is_some(),
            ))
        });
        Self {
            native,
            played,
            site,
            branch,
            components,
            mode: runtime.loop_settings().mode,
            spatial,
        }
    }

    pub fn context_matches(
        &self,
        runtime: &Runtime,
        native: &NativeRevision,
        played: Option<Id>,
        site: Id,
        branch: SceneBranch,
        components: bool,
    ) -> bool {
        self.native == *native
            && self.played == played
            && self.site == site
            && self.branch == branch
            && self.components == components
            && self.mode == runtime.loop_settings().mode
    }
}

/// Both backends share native admission and terrain extraction. The optional
/// backend retains typed component columns and materializes the renderer's
/// existing body vector; it is not an authoritative Population migration.
pub(super) fn store(
    mut scene: SiteScene,
    components: bool,
    retained: &mut Option<ComponentScene>,
    style: impl FnOnce(&mut SiteScene),
) -> Result<Rc<SiteScene>, String> {
    if components {
        let columns = ComponentScene::capture(&scene);
        scene.bodies = columns.bodies()?;
        *retained = Some(columns);
    } else {
        *retained = None;
    }
    style(&mut scene);
    Ok(Rc::new(scene))
}
