// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Snapshot {
    pub fn site(&self) -> Id {
        self.scene.site
    }
    pub fn played(&self) -> Option<Id> {
        self.scene.played
    }
    pub fn body_ids(&self) -> Vec<Id> {
        self.scene.bodies.iter().map(|body| body.id).collect()
    }
    pub fn camera(&self, aspect: f32) -> SlabCamera {
        if let Some(view) = &self.presentation {
            if let Some(camera) = self
                .eye
                .and_then(|eye| section::perspective::camera(&view.looking, eye, aspect))
            {
                return camera;
            }
            return SlabCamera {
                projection: Default::default(),
                centre: view.centre,
                forward: view.forward,
                half_height: view.half_height,
                aspect,
                depth: view
                    .depth
                    .unwrap_or_else(|| view.mode.depth(view.half_height, aspect)),
                cutaway: None,
            };
        }
        let mode = CameraMode::default();
        let at = self
            .scene
            .played
            .and_then(|id| self.scene.at(id))
            .unwrap_or_else(|| layout::centre(&self.scene.ground));
        SlabCamera {
            projection: Default::default(),
            centre: section::centre_on(at, Pan::default(), section::SLAB_HALF_HEIGHT, mode),
            forward: forward(self.quarter_turn),
            half_height: section::SLAB_HALF_HEIGHT,
            aspect,
            depth: mode.depth(section::SLAB_HALF_HEIGHT, aspect),
            cutaway: None,
        }
    }
}

impl SceneHandle {
    /// A product's validated picture inputs, published without a second world.
    pub(crate) fn present(
        &self,
        scene: &Rc<SiteScene>,
        revision: u64,
        presentation: Presentation,
    ) -> Result<(), String> {
        let key = format!("{presentation:?}");
        let old = self.snapshot();
        let same_picture = old.as_ref().is_some_and(|old| {
            old.source_scene
                .as_ref()
                .is_some_and(|original| Rc::ptr_eq(original, scene))
                && old.presentation_key.as_ref() == Some(&key)
        });
        if same_picture
            && old
                .as_ref()
                .is_some_and(|old| old.revision == Some(revision))
        {
            return Ok(());
        }
        // Authority publication and picture publication have different scopes.
        // A section-only update changes action admission while sharing the
        // exact body storage and already presented image.
        let sequence = if same_picture {
            self.picture_sequence.get()
        } else {
            self.picture_sequence
                .get()
                .checked_add(1)
                .ok_or("picture sequence overflow")?
        };
        let digest = cosmograph::digest(&(sequence, &key));
        let mut fingerprint = [0; 4];
        for (i, word) in fingerprint.iter_mut().enumerate() {
            *word = u64::from_str_radix(&digest[i * 16..(i + 1) * 16], 16)
                .map_err(|why| why.to_string())?;
        }
        if same_picture {
            fingerprint = old.as_ref().unwrap().fingerprint;
        }
        let (picture, eye) = if same_picture {
            let old = old.as_ref().unwrap();
            (old.scene.clone(), old.eye)
        } else if presentation.looking.view == isometer::View::Overview {
            (scene.clone(), None)
        } else {
            let (picture, eye) = section::perspective::seen(scene, &presentation.looking);
            (Rc::new(picture), eye)
        };
        self.picture_sequence.set(sequence);
        *self.input.borrow_mut() = Some(Rc::new(Snapshot {
            scene: picture,
            source_scene: Some(scene.clone()),
            eye,
            fingerprint,
            quarter_turn: 0,
            presentation: Some(presentation),
            revision: Some(revision),
            presentation_key: Some(key),
        }));
        self.error.borrow_mut().take();
        Ok(())
    }
}
