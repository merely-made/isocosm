//! The scene board: the map and its tokens as one `isometer` frame.
//!
//! This is B2's whole product side. [`BoardView`] is the snapshot the host
//! pushes each frame — the map, the pan, the pane and the projection — and
//! [`BoardSource`] is the [`SceneSource`] over it: B1's terrain adapter grown
//! into a `Ground` and uploaded as a `BrickMap`, its tile kinds bound through
//! `set_terrain_palette`, one live body per token from the DOM board's own
//! recipes, and the dimetric camera [`super::world`] derives from the board's
//! own pan.
//!
//! **What the source does not own.** The skip, the render scale and the leaf's
//! output contract are `SceneProducer`'s; selection, overlays, markers and the
//! context menu are B3 and B4 and stay on the DOM exactly as they are. The
//! source answers [`BoardSource::pick`] so the parity gate can ask it which
//! tile a pixel shows, and nothing routes a pointer through it yet.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use isometer::core::ground::Ground;
use isometer::lens::{BrickMap, Grade};
use isometer::{
    BodySignature, FrameRequest, GroundTerrain, Pick, Pose, Scene, SceneBody, SceneFrame,
    SceneHost, SceneProducer, SceneSignature, SceneSource, SceneVolumes, SlabCamera, SubjectKey,
    TerrainSource,
};
use isometry_core::{IsoGeometry, MapDocument, TileCoord, Token, TokenId};

use super::tokens::{TokenBodies, owner_tint, yaw_of};
use super::world::BoardWorld;
use super::{MapTerrain, terrain_palette};
use crate::state::UiState;

/// The producer key the board pane's scene leaf is registered under. Shared so
/// the view's `custom_leaf` and the host's `register` name one leaf, exactly as
/// `OVERMAP_LEAF_KEY` is shared. Distinct from the overmap's: producer keys and
/// painted-leaf keys are one namespace.
pub const BOARD_SCENE_LEAF_KEY: u64 = 8002;

/// Bodies one frame may project. A board is tokens, not a settlement; the cap
/// exists so a pathological map degrades by omission rather than by cost.
const BODY_BUDGET: usize = 256;

/// The board draws flat colour: no palette starving, no dither, and the fog
/// pushed past the far wall so a tile's kind colour is its own.
fn board_grade() -> Grade {
    Grade {
        fog_start: 1.0,
        ..Grade::clay()
    }
}

/// Isometry keeps no host presentation beside the scene: no capsule roster and
/// no substitute for a body that would not project.
struct PlainHost;
impl SceneHost for PlainHost {}

/// What one pixel of the scene board shows.
///
/// A tile hit carries more than the DOM board's inverse can: which voxel of
/// the column the ray met, and whether it met the top face or an exposed side.
/// B3 will want both — a click on a cliff face is a click on the tile above
/// it, not on the one the face belongs to — and the parity gate turns on the
/// distinction, because the side faces are geometry the DOM board only draws
/// where an elevation step exposes one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardPick {
    Token(TokenId),
    Tile {
        at: TileCoord,
        /// The height unit of the voxel the ray entered.
        elevation: i32,
        /// The hit was on the column's top face rather than on a side.
        top: bool,
    },
}

/// The board as the producer sees it: a snapshot the host pushes, because the
/// producer is called from the host's own loop and cannot borrow the runner's
/// state while it is doing so.
pub struct BoardView {
    pub map: MapDocument,
    pub geo: IsoGeometry,
    /// Board-origin offset inside the pane, logical px.
    pub camera: (f32, f32),
    /// The board pane's logical size, logical px.
    pub pane: (f32, f32),
    /// Bumped whenever the ground, the elevation or the tile kinds move, so
    /// the terrain is regrown exactly then.
    terrain_revision: u64,
}

/// The shared handle. Cheap to clone; every clone sees one snapshot.
pub type BoardHandle = Rc<RefCell<BoardView>>;

impl BoardView {
    pub fn new(map: MapDocument) -> Self {
        Self {
            map,
            geo: IsoGeometry::default(),
            camera: (0.0, 0.0),
            pane: (0.0, 0.0),
            terrain_revision: 1,
        }
    }

    pub fn into_handle(self) -> BoardHandle {
        Rc::new(RefCell::new(self))
    }

    /// Takes the board's current state. The terrain revision moves only for
    /// the three layers the ground is grown from — a token step or a pan is
    /// not a regrow — and the map is cloned only when one of them differs, so
    /// an ordinary frame copies nothing.
    pub fn sync(&mut self, ui: &UiState) {
        self.geo = ui.geo;
        self.camera = ui.camera;
        self.pane = ui.viewport;
        let terrain_moved = self.map.ground != ui.map.ground
            || self.map.elevation != ui.map.elevation
            || self.map.tile_kinds != ui.map.tile_kinds;
        if terrain_moved || self.map.tokens != ui.map.tokens || self.map.props != ui.map.props {
            self.map = ui.map.clone();
        }
        if terrain_moved {
            self.terrain_revision += 1;
        }
    }
}

/// One board snapshot as an [`isometer::SceneSource`].
pub struct BoardSource {
    view: BoardHandle,
    tokens: TokenBodies,
    scene: Option<Scene>,
    scene_size: [u32; 2],
    /// The grown ground and the revision it was grown from.
    ground: Option<(Ground, u64)>,
    world: Option<BoardWorld>,
    /// Subjects whose recipe colour table is already registered on the body
    /// layer. A table is per subject and survives frames, so it is set once.
    palettes: BTreeSet<SubjectKey>,
    /// Tokens the last frame drew as the placeholder body.
    placeholders: usize,
}

/// The producer the host registers: the skip and the pixel grid over the
/// board's own source.
pub type BoardProducer = SceneProducer<BoardSource>;

impl BoardSource {
    pub fn new(view: BoardHandle) -> Self {
        Self {
            view,
            tokens: TokenBodies::new(),
            scene: None,
            scene_size: [0; 2],
            ground: None,
            world: None,
            palettes: BTreeSet::new(),
            placeholders: 0,
        }
    }

    pub fn view(&self) -> &BoardHandle {
        &self.view
    }

    /// Tokens the last frame drew without a recipe.
    pub fn placeholders(&self) -> usize {
        self.placeholders
    }

    /// The board's placement, once a frame has established it.
    pub fn world(&self) -> Option<BoardWorld> {
        self.world
    }

    /// What the last drawn frame shows at normalized clip coordinates.
    ///
    /// A body pick is the token whose [`SubjectKey`] it carries; a terrain hit
    /// is the tile its entered voxel belongs to, through B1's convention, and
    /// `None` off the map. This is the scene half of the parity gate, and the
    /// seam B3 will route the pointer through.
    pub fn pick(&self, ndc: [f32; 2]) -> Option<BoardPick> {
        let scene = self.scene.as_ref()?;
        let world = self.world?;
        let view = self.view.borrow();
        match scene.pick(ndc).ok().flatten()? {
            Pick::Body(body) => Some(BoardPick::Token(TokenId(body.address.subject.0 as u32))),
            Pick::Terrain(hit) => {
                let (col, row) = world.tile_of_voxel(hit.voxel);
                view.map
                    .ground
                    .in_bounds(col, row)
                    .then_some(BoardPick::Tile {
                        at: (col, row),
                        elevation: hit.voxel[1],
                        top: hit.normal[1] > 0.5,
                    })
            },
        }
    }

    /// The camera a request would draw with, and the world it is drawn in.
    fn framing(&mut self, request: &FrameRequest<'_>) -> Option<(SlabCamera, BoardWorld)> {
        let view = self.view.borrow();
        let world = BoardWorld::new(&view.map);
        // Until the host reports a pane, the texture's own size is the pane:
        // the first frame then frames the board rather than nothing.
        let pane = if view.pane.0 > 0.0 && view.pane.1 > 0.0 {
            view.pane
        } else {
            (request.size[0] as f32, request.size[1] as f32)
        };
        let camera = world.camera(&view.geo, view.camera, pane, request.size)?;
        Some((camera, world))
    }

    /// Builds the scene on the first frame and resizes it afterwards.
    fn ensure_scene(&mut self, request: &FrameRequest<'_>) -> Result<(), String> {
        let size = [request.size[0].max(1), request.size[1].max(1)];
        match &mut self.scene {
            None => {
                self.scene = Some(Scene::new(
                    request.device.clone(),
                    request.queue.clone(),
                    size[0],
                    size[1],
                )?);
                self.ground = None;
                self.palettes.clear();
            },
            Some(scene) if self.scene_size != size => scene.resize(size[0], size[1]),
            Some(_) => {},
        }
        self.scene_size = size;
        Ok(())
    }

    /// Grows the ground and binds the tile-kind palette when the map's terrain
    /// layers have moved, and never otherwise.
    fn ensure_terrain(&mut self, revision: u64) -> Result<(), String> {
        if self.ground.as_ref().is_some_and(|(_, at)| *at == revision) {
            return Ok(());
        }
        let view = self.view.borrow();
        let ground = MapTerrain::new(&view.map).grow();
        let scene = self.scene.as_mut().ok_or("the scene was not built")?;
        scene.set_terrain_map(
            BrickMap::from_ground(&ground).map_err(|error| format!("brick map: {error}"))?,
        );
        scene.set_terrain_palette(Some(terrain_palette(&view.map)));
        drop(view);
        self.ground = Some((ground, revision));
        Ok(())
    }
}

/// One token's body placement: its recipe's scale, the tile's world stand, and
/// the yaw its facing turns it to.
///
/// Free rather than a method so a caller can hold the token table and the
/// scene at once; every field it reads is borrowed, none is owned.
fn pose_of(
    tokens: &TokenBodies,
    map: &MapDocument,
    world: &BoardWorld,
    token: &Token,
) -> (Pose, f32) {
    let scale = tokens.scale(&token.sprite);
    let elevation = *map
        .elevation
        .get(token.at.0.max(0) as u32, token.at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let feet = world.stand(token.at, elevation);
    let aabb = tokens.body(&token.sprite).0.document.aabb();
    // `grounded` stands the document's floor on `position[1]`; the horizontal
    // half has no shared spelling, so the volume's own centre comes off the
    // position, as Paredros's source does it.
    let centred = |axis: usize| feet[axis] - (aabb.min[axis] + aabb.max[axis]) as f32 * 0.5 * scale;
    (
        Pose {
            position: [centred(0), feet[1], centred(2)],
            yaw_radians: yaw_of(token.facing),
        },
        scale,
    )
}

impl SceneSource for BoardSource {
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        let (camera, world) = self
            .framing(request)
            .ok_or("the board cannot frame this pane")?;
        let view = self.view.borrow();
        let bodies = view
            .map
            .tokens
            .iter()
            .map(|token| {
                let (pose, scale) = pose_of(&self.tokens, &view.map, &world, token);
                BodySignature {
                    subject: SubjectKey(u64::from(token.id.0)),
                    // The recipes are static, so a token's geometry changes
                    // only when its sprite does: the name's own hash is the
                    // revision.
                    revision: sprite_revision(&token.sprite),
                    pose,
                    scale,
                    grounded: true,
                    tint: owner_tint(token.owner.as_deref()),
                    always_visible: false,
                }
            })
            .collect();
        Ok(SceneSignature {
            size: request.size,
            render_scale: request.render_scale,
            camera: Some(camera),
            terrain_revision: self.ground.as_ref().map(|(ground, _)| ground.revision()),
            bodies,
            // The terrain snapshot the ground is grown from: a paint that
            // changes no token still redraws.
            host: vec![view.terrain_revision],
        })
    }

    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        let (camera, world) = self
            .framing(request)
            .ok_or("the board cannot frame this pane")?;
        self.world = Some(world);
        self.ensure_scene(request)?;
        let revision = self.view.borrow().terrain_revision;
        self.ensure_terrain(revision)?;

        // A token's recipe colour table is per subject and survives frames, so
        // it is registered once — before the bodies below borrow the table the
        // documents come from.
        let drawn: Vec<Token> = self.view.borrow().map.tokens.clone();
        for token in &drawn {
            let subject = SubjectKey(u64::from(token.id.0));
            if self.palettes.insert(subject) {
                let colours = self.tokens.colours(&token.sprite);
                self.scene
                    .as_mut()
                    .expect("the scene was built")
                    .bodies_mut()
                    .set_palette(subject, Some(colours));
            }
        }

        // The bodies point into the token table for as long as the scene is
        // held mutably, which one `&mut self` chain cannot express: the scene
        // comes out of the source for the encode and goes straight back.
        let mut scene = self.scene.take().ok_or("the scene was not built")?;
        let held = self.view.borrow();
        let mut missing = 0;
        let mut bodies = Vec::with_capacity(drawn.len());
        for token in &drawn {
            let (body, placeholder) = self.tokens.body(&token.sprite);
            missing += usize::from(placeholder);
            let (pose, scale) = pose_of(&self.tokens, &held.map, &world, token);
            bodies.push(SceneBody {
                subject: SubjectKey(u64::from(token.id.0)),
                document: &body.document,
                pose,
                scale,
                grounded: true,
                tint: owner_tint(token.owner.as_deref()),
                materials: &[],
                always_visible: false,
            });
        }
        self.placeholders = missing;
        let Some((ground, _)) = &self.ground else {
            self.scene = Some(scene);
            return Err("the board grew no ground".into());
        };
        let terrain = GroundTerrain(ground);
        let mut encoder = request
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("isometry scene board"),
            });
        let result = scene.render(
            &mut encoder,
            SceneFrame {
                camera,
                bodies: &bodies,
                volumes: SceneVolumes::Voxels(&self.tokens.volumes),
                terrain: Some(&terrain as &dyn TerrainSource),
                dirty: &[],
                grade: board_grade(),
                terrain_appearance: None,
                body_budget: BODY_BUDGET,
                capsules: None,
            },
            &mut PlainHost,
        );
        drop(bodies);
        drop(held);
        // The encoded twin: the leaf declares `SCENE_ENCODING`, so the
        // compositor samples these bytes directly.
        let view = scene.encoded_view().clone();
        self.scene = Some(scene);
        // A failed encode still puts the scene back: the source owns it, and a
        // retirement is the only thing that releases it.
        result?;
        request.queue.submit(Some(encoder.finish()));
        Ok(Some(view))
    }

    fn presented_camera(&self) -> Option<SlabCamera> {
        self.scene.as_ref()?.presented_camera()
    }

    fn cached_bodies(&self) -> usize {
        self.scene.as_ref().map_or(0, Scene::cached_bodies)
    }

    /// Suspension keeps the scene and its projected geometry; the producer's
    /// own banked signature is what makes the next request redraw.
    fn suspend(&mut self) {}

    /// Retirement is this source's whole GPU release.
    fn retire(&mut self) {
        self.scene = None;
        self.scene_size = [0; 2];
        self.ground = None;
        self.palettes.clear();
    }
}

/// A sprite name as a geometry revision: the recipes are a fixed table, so two
/// tokens of one sprite share a number and a rename is a change.
fn sprite_revision(sprite: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in sprite.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}
