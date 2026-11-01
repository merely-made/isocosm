// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What every Cosmograph endpoint shares: cards with actions become a solved
//! scene, a portable-card manifest and disclosed resources; intents are
//! checked against the revision and against what the last snapshot
//! advertised. The core's world projection and each product's own use it.

use std::collections::{BTreeMap, HashMap};

use chirograph::{
    AdvertisedAction, BoundsRelationship, CardValueV1, CachePolicy, ContentHash, IntentEffect,
    IntentInvocation, IntentReference, IntentResult, PortableCardV1, PresentationBinding,
    PresentationCapability, PresentationCodec, PresentationKey, PresentationManifest,
    PresentationOffer, PresentationSemantics, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceRequest, ResourceResponse, Revision, SceneEpoch,
    SceneSnapshot, SemanticRole,
};
use crate::world::World;
use sceno::{
    Arrangement, Fold, Footprint, InstanceId, Placement, Representation, RoutedRelation, Scene, Score,
    ScoreItem, Size2, SourceRef, Spiral, StandIn,
};

/// One disclosed thing: its source id, its card and the actions it offers.
pub struct Item {
    pub id: String,
    pub card: PortableCardV1,
    pub actions: Vec<AdvertisedAction>,
}

pub fn value(label: impl Into<String>, value: impl ToString) -> CardValueV1 {
    CardValueV1 { label: label.into(), value: value.to_string() }
}

pub fn card(title: impl Into<String>, values: Vec<CardValueV1>, badges: &[&str]) -> PortableCardV1 {
    PortableCardV1 { title: title.into(), values, badges: badges.iter().map(|b| (*b).into()).collect(),
        media: Vec::new() }
}

/// A JSON schema for an action without a payload.
pub const NULL: &str = r#"{"type":"null"}"#;

pub fn action(intent: &str, label: &str, explanation: &str, schema: &str, effect: IntentEffect)
    -> AdvertisedAction {
    AdvertisedAction {
        intent: IntentReference(intent.into()),
        label: label.into(),
        explanation: explanation.into(),
        payload_schema: schema.into(),
        input_form: None,
        effect,
    }
}

/// A portable card's footprint, the size Graphshell's own card endpoints use.
pub const CARD: [f32; 2] = [260.0, 136.0];

pub fn arrangement() -> Arrangement { Arrangement::Spiral(Spiral::default()) }

/// The request an endpoint offers for `session`.
pub fn request(session: &str) -> ProjectionRequest {
    ProjectionRequest { version: ProtocolVersion::V1, session: ProjectionSession(session.into()),
        score: Score::new(arrangement()) }
}

/// Several disclosed items drawn as one stand-in (a sceno fold): the
/// stand-in's id, the others' ids, and the host's words for it.
pub struct Folded {
    pub stand_in: String,
    pub members: Vec<String>,
    pub label: String,
}

/// The items' solved scene, with `links` (from, to, kind) as relations.
pub fn scene(adapter: &str, items: &[Item], links: &[(String, String, &str)]) -> Scene {
    scene_folded(adapter, items, links, &[], CARD)
}

/// The items' solved scene with relations and folds, each card `card` in size.
pub fn scene_folded(adapter: &str, items: &[Item], links: &[(String, String, &str)], folds: &[Folded],
    card: [f32; 2]) -> Scene {
    let mut score = Score::new(arrangement());
    score.items = items.iter().enumerate().map(|(ordinal, item)| ScoreItem {
        source: SourceRef::new(adapter, &item.id),
        ordinal: ordinal as u32,
        footprint: Footprint::Rect { size: Size2::new(card[0], card[1]) },
        representation: Representation::Card,
        placement: Placement::Ordinal,
        layer: 0,
        visible: true,
        axis: None,
        embedding: None,
        weight: None,
    }).collect();
    let mut scene = scenomise::solve(&score);
    let instances: HashMap<String, InstanceId> = scene.items.iter().enumerate()
        .filter_map(|(index, item)| scene.sources.get(item.source.0 as usize)
            .map(|source| (source.id.clone(), InstanceId(index as u32))))
        .collect();
    for (from, to, kind) in links {
        let (Some(&from), Some(&to)) = (instances.get(from), instances.get(to)) else { continue };
        let points = vec![scene.items[from.0 as usize].transform.translate,
            scene.items[to.0 as usize].transform.translate];
        scene.relations.push(RoutedRelation { from, to, space: Scene::WORLD, points,
            kind: Some((*kind).into()), weight: None, link_key: None,
            marks: Default::default(), strength: None });
    }
    for fold in folds {
        let Some(&stand_in) = instances.get(&fold.stand_in) else { continue };
        let mut members = vec![stand_in];
        members.extend(fold.members.iter().filter_map(|id| instances.get(id).copied()));
        if members.len() < 2 { continue; }
        scene.folds.push(Fold { members, stand_in: StandIn::Member(stand_in), rule: None, boundary: None,
            label: Some(fold.label.clone()) });
    }
    scene
}

/// One session's disclosure state: the last snapshot, its resources and the
/// revision. The revision also moves when the shared world does, through
/// this endpoint or any other (ruling 910).
pub struct Disclosure {
    session: String,
    world: World,
    revision: u64,
    /// The world's revision the current revision reflects.
    seen: Option<u64>,
    snapshot: Option<ProjectionSnapshot>,
    resources: BTreeMap<ContentHash, Vec<u8>>,
}

impl Disclosure {
    pub fn new(session: &str, world: &World) -> Self {
        Self { session: session.into(), world: world.share(), revision: 1, seen: None, snapshot: None,
            resources: BTreeMap::new() }
    }

    /// A new revision if the world moved since the revision last caught up.
    fn catch_up(&mut self) -> Result<(), String> {
        let now = self.world.revision()?;
        if self.seen.is_some_and(|seen| seen != now) { self.revision += 1; }
        self.seen = Some(now);
        Ok(())
    }

    pub fn session(&self) -> &str { &self.session }

    /// A fresh snapshot of `items` at the current revision.
    pub fn snapshot(&mut self, request: ProjectionRequest, adapter: &str, items: &[Item],
        links: &[(String, String, &str)]) -> Result<ProjectionSnapshot, String> {
        self.snapshot_folded(request, adapter, items, links, &[], CARD)
    }

    /// A fresh snapshot whose scene also carries `folds`, its cards `card` in size.
    pub fn snapshot_folded(&mut self, request: ProjectionRequest, adapter: &str, items: &[Item],
        links: &[(String, String, &str)], folds: &[Folded], card: [f32; 2]) -> Result<ProjectionSnapshot, String> {
        if request.version.major != ProtocolVersion::V1.major {
            return Err("projection request uses an unsupported protocol".into());
        }
        if request.score.version != sceno::SCORE_VERSION {
            return Err("projection request uses an unsupported score".into());
        }
        if request.session.0 != self.session {
            return Err(format!("projection request names a session other than {}", self.session));
        }
        self.catch_up()?;
        let scene = SceneSnapshot::from_dense(SceneEpoch(1), Revision(self.revision),
            scene_folded(adapter, items, links, folds, card)).map_err(|error| format!("invalid scene: {error:?}"))?;
        let presentation = self.presentations(items, &scene)?;
        let snapshot = ProjectionSnapshot { version: ProtocolVersion::V1, session: request.session,
            scene, presentation, cache_policy: CachePolicy::default() };
        self.snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    fn presentations(&mut self, items: &[Item], scene: &SceneSnapshot) -> Result<PresentationManifest, String> {
        let by_id: HashMap<&str, &Item> = items.iter().map(|item| (item.id.as_str(), item)).collect();
        let mut manifest = PresentationManifest::default();
        for (instance, item) in scene.active_items_in_order() {
            let source = scene.tables.sources.get(item.source.0 as usize).and_then(Option::as_ref)
                .ok_or_else(|| format!("scene item {} has no source", instance.0))?;
            let item = by_id.get(source.id.as_str())
                .ok_or_else(|| format!("scene names an undisclosed item {}", source.id))?;
            let bytes = serde_json::to_vec(&item.card).map_err(|e| format!("could not encode a card: {e}"))?;
            let hash = ContentHash::of(&bytes);
            let byte_size = bytes.len() as u64;
            self.resources.insert(hash, bytes);
            let key = PresentationKey(source.id.clone());
            manifest.bindings.push(PresentationBinding { instance, key: key.clone() });
            manifest.offers.insert(key, vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: hash,
                byte_size,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: item.card.title.clone(),
                    role: SemanticRole::Article,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: item.actions.clone(),
                },
            }]);
        }
        Ok(manifest)
    }

    pub fn resource(&self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        let bytes = self.resources.get(&request.resource).cloned()
            .ok_or("resource was not disclosed by this session")?;
        Ok(ResourceResponse { session: request.session, resource: request.resource, bytes })
    }

    /// The targeted item's source id, or the result that refuses the intent:
    /// stale against the revision, or not advertised on its target.
    pub fn admit(&mut self, intent: &IntentInvocation) -> Result<Result<String, IntentResult>, String> {
        if self.snapshot.is_none() { return Err("intent arrived before a snapshot".into()); }
        self.catch_up()?;
        let snapshot = self.snapshot.as_ref().expect("checked above");
        if intent.observed_epoch != snapshot.scene.epoch || intent.observed_revision.0 != self.revision {
            return Ok(Err(IntentResult::Stale { current_epoch: snapshot.scene.epoch,
                current_revision: Revision(self.revision) }));
        }
        let advertised = (|| {
            let item = snapshot.scene.active_item(intent.target)?;
            let offers = snapshot.presentation.offers_for(intent.target)?;
            offers.iter().any(|o| o.semantics.actions.iter().any(|a| a.intent.0 == intent.intent)).then_some(())?;
            Some(snapshot.scene.tables.sources.get(item.source.0 as usize)?.as_ref()?.id.clone())
        })();
        Ok(advertised.ok_or_else(|| rejected("target or intent was not disclosed by this snapshot")))
    }

    /// An accepted intent that changed what the next snapshot shows.
    pub fn accepted(&mut self) -> IntentResult {
        self.revision += 1;
        self.seen = self.world.revision().ok();
        IntentResult::Accepted
    }
}

pub fn rejected(reason: impl Into<String>) -> IntentResult {
    IntentResult::Rejected { reason: reason.into() }
}
