// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use cambium::ProjectionId;
use cambium_rootstock::HostTree;
use cambium_genet_winit_host::{CloseDisposition, HostHooks, HostOptions, WindowHooks,
    WindowsInit, WindowTree, run_windows};
use crate::ui::{model::App, view::{self, Child, Logic}};

/// Operator-supplied author input over the opened world, before any seat packet.
pub(crate) fn mesocosm_spread(world: &crate::world::World, path: &std::path::Path) -> Result<(), String> {
    #[cfg(feature = "mesocosm")]
    {
        let authored = std::fs::read(path).map_err(|why| why.to_string())?;
        let setup: mesocosm_runtime::author_spread::Setup =
            serde_json::from_slice(&authored).map_err(|why| why.to_string())?;
        world.product_game(world.cut()?, crate::products::Mode::Mesocosm, |session, bytes| {
            let mut game: mesocosm_runtime::spatial::Game = if bytes.is_empty() {
                Default::default()
            } else { serde_json::from_slice(&bytes).map_err(|why| why.to_string())? };
            setup.apply(session, &mut game)?;
            Ok(((), serde_json::to_vec(&game).map_err(|why| why.to_string())?))
        })
    }
    #[cfg(not(feature = "mesocosm"))]
    {
        let _ = (world, path);
        Err("authored Mesocosm spread setup is absent from this build".into())
    }
}

/// Explicit authored startup data, separate from ordinary player packets.
pub(crate) fn mesocosm_spatial(world: &crate::world::World, path: &std::path::Path) -> Result<(), String> {
    #[cfg(feature = "mesocosm")]
    {
        let authored = std::fs::read(path).map_err(|why| why.to_string())?;
        let setup: mesocosm_runtime::spatial::Setup =
            serde_json::from_slice(&authored).map_err(|why| why.to_string())?;
        world.product_game(world.cut()?, crate::products::Mode::Mesocosm, |session, bytes| {
            let mut game: mesocosm_runtime::spatial::Game = if bytes.is_empty() {
                Default::default()
            } else { serde_json::from_slice(&bytes).map_err(|why| why.to_string())? };
            game.setup(session, &setup)?;
            let bytes = serde_json::to_vec(&game).map_err(|why| why.to_string())?;
            Ok(((), bytes))
        })
    }
    #[cfg(not(feature = "mesocosm"))]
    {
        let _ = (world, path);
        Err("authored Mesocosm spatial setup is absent from this build".into())
    }
}

fn options() -> HostOptions { HostOptions { title: "Isocosm".into(),
    initial_logical_size: (960.0, 720.0), ..Default::default() } }

/// Explicit mode-only Eponym poses over this existing native Session.
pub(crate) fn eponym_spatial(world: &crate::world::World, path: &std::path::Path) -> Result<(), String> {
    #[cfg(feature = "eponym")]
    {
        let bytes = std::fs::read(path).map_err(|why| why.to_string())?;
        let input: eponym_play::contract::mode::SpatialBindingInput =
            serde_json::from_slice(&bytes).map_err(|why| why.to_string())?;
        world.product_game(world.cut()?, crate::products::Mode::Eponym, |session, game| {
            let bytes = eponym_play::contract::mode::bind(session, &game, &input)?;
            Ok(((), bytes))
        })
    }
    #[cfg(not(feature = "eponym"))]
    {
        let _ = (world, path);
        Err("Eponym spatial binding is absent from this build".into())
    }
}

fn bind_routes(tree: &WindowTree<App, Logic, Child>, routes: &mut HashMap<ProjectionId, u64>, initial: &[u64]) {
    if !routes.is_empty() { return; }
    let ids: Vec<_> = tree.multi().projection_ids().collect();
    assert_eq!(ids.len(), initial.len(), "actual native projections match the declared lenses");
    routes.extend(ids.into_iter().zip(initial.iter().copied()));
}

pub(crate) fn run(app: App) -> Result<(), String> {
    let sheet = view::accepted_sheet(&app)?;
    let accepted_sheet = Rc::new(RefCell::new(sheet.clone()));
    let keys: Vec<u64> = app.windows.keys().copied().collect();
    if keys.is_empty() { return Err("at least one explicitly bound window is required".into()); }
    let routes = Rc::new(RefCell::new(HashMap::<ProjectionId, u64>::new()));
    let initial = Rc::new(keys.clone());
    let core = Rc::new(RefCell::new(None));
    #[cfg(any(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
    let registrations = Rc::new(RefCell::new(crate::viewport::Registrations::default()));
    let mut hooks: WindowHooks<App, Logic, Child> = HostHooks::inert();
    hooks.frame = Box::new({
        let routes = routes.clone(); let core = core.clone(); let initial = initial.clone();
        move |ctx| {
            bind_routes(ctx.runner, &mut routes.borrow_mut(), &initial);
            if let Some(actual) = ctx.render_core {
                let mut held = core.borrow_mut();
                if let Some(first) = held.as_ref() {
                    assert!(Arc::ptr_eq(first, actual), "all mode windows use the same real render core");
                } else { *held = Some(actual.clone()); }
            }
            #[cfg(any(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
            {
                let errors = registrations.borrow_mut().sync(ctx.runner.state(), ctx.producers);
                if !errors.is_empty() {
                    ctx.runner.update(|app| for (key, why) in errors { app.record(key, Err(why)); });
                }
            }
            false
        }
    });
    hooks.after_dispatch = Box::new({
        let routes = routes.clone(); let initial = initial.clone();
        let accepted_sheet = accepted_sheet.clone();
        move |ctx| {
            bind_routes(ctx.runner, &mut routes.borrow_mut(), &initial);
            match view::accepted_sheet(ctx.runner.state()) {
                Ok(sheet) if sheet != *accepted_sheet.borrow() => {
                    *accepted_sheet.borrow_mut() = sheet.clone();
                    *ctx.set_sheet = Some(sheet);
                },
                Err(reason) => {
                    if let Some(key) = routes.borrow().get(&ctx.runner.window()).copied() {
                        ctx.runner.update(|app| app.record(key, Err(reason)));
                    }
                },
                _ => {},
            }
            let opened = ctx.runner.state().pending_open.clone();
            if !opened.is_empty() {
                ctx.runner.update(|app| app.pending_open.clear());
                for key in opened {
                    let id = ctx.runner.open(view::lens(key), options());
                    routes.borrow_mut().insert(id, key);
                }
            }
            let closing: Vec<_> = routes.borrow().iter().filter_map(|(id, key)| {
                ctx.runner.state().windows.get(key).is_some_and(|w| w.close_requested)
                    .then_some((*id, *key))
            }).collect();
            for (id, key) in closing {
                let mut accepted = false;
                ctx.runner.update(|app| {
                    let result = app.close(key); accepted = result.is_ok(); app.record(key, result);
                });
                if accepted { routes.borrow_mut().remove(&id); ctx.runner.close(id); }
            }
        }
    });
    hooks.close_request = Box::new(move |ctx, _| {
        bind_routes(ctx.runner, &mut routes.borrow_mut(), &initial);
        let Some(key) = routes.borrow().get(&ctx.runner.window()).copied() else {
            return CloseDisposition::KeepVisible;
        };
        let mut accepted = false;
        ctx.runner.update(|app| {
            let result = app.close(key); accepted = result.is_ok(); app.record(key, result);
        });
        if accepted { routes.borrow_mut().remove(&ctx.runner.window()); CloseDisposition::Exit }
        else { CloseDisposition::KeepVisible }
    });
    let windows = keys.into_iter().map(|key| (view::lens(key), options())).collect();
    run_windows(move |_| WindowsInit { state: app, sheet,
        fonts: Vec::new(), images: Vec::new(), windows }, hooks).map_err(|e| e.to_string())
}
