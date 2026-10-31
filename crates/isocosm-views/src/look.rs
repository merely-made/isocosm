//! The look editor: class, slots and palette, with a live preview.
//!
//! The preview follows whichever board is active (ruling 1028): on the scene
//! board it is a second scene leaf drawn by the same source the board uses,
//! and on the DOM board it is the same bake the DOM board's sprites are, at a
//! larger stamp. Either way it resolves the draft through the packs' looks,
//! exactly as the board will once the draft is accepted.

use cambium::{clickable, custom_leaf, el, text};
use isometer_mesh::bake::BakeParams;

use crate::appearance::{bake_facing_of, colour_hex};
use crate::board::UiChild;
use crate::state::UiState;
use crate::theme::SPRITE_FIT;

/// The preview leaf the host serves from a second board source.
pub const LOOK_PREVIEW_LEAF_KEY: u64 = 0x150_100C;

/// The preview's box, logical px.
pub const LOOK_PREVIEW_SIZE: (u32, u32) = (120, 150);

/// The DOM preview's stamp: the board's, three times as large.
const PREVIEW_BAKE: BakeParams = BakeParams {
    half_w: 6,
    cube_h: 6,
    facings: 4,
    margin: 2,
};

fn button(label: String, active: bool, act: impl Fn(&mut UiState) + 'static) -> UiChild {
    let class = if active { "btn btn-mini btn-active" } else { "btn btn-mini" };
    Box::new(clickable(
        el("div", text(label)).attr("class", class),
        move |ui: &mut UiState, _| act(ui),
    ))
}

fn row(label: &str, children: Vec<UiChild>) -> Vec<UiChild> {
    vec![
        Box::new(el("div", text(label.to_owned())).attr("class", "sheet-row look-label")),
        Box::new(el("div", children).attr("class", "btn-row")),
    ]
}

fn preview(ui: &UiState) -> UiChild {
    let (w, h) = LOOK_PREVIEW_SIZE;
    if ui.scene_board {
        return Box::new(
            custom_leaf::<UiState, ()>(LOOK_PREVIEW_LEAF_KEY, w, h)
                .attr("id", "look-preview")
                .attr("class", "look-preview"),
        );
    }
    let facing = bake_facing_of(ui.look.facing);
    let style = match ui
        .appearance_bakes
        .uri(&ui.appearance, &ui.look.input(), facing, &PREVIEW_BAKE)
    {
        Some(uri) => format!("background-image: url(\"{uri}\"); {SPRITE_FIT}"),
        None => "background-color: #dc28c8;".to_owned(),
    };
    Box::new(
        el("div", ())
            .attr("id", "look-preview")
            .attr("class", "look-preview")
            .attr("style", style),
    )
}

/// The editor's rows, shared by the creation panel and the token's own.
pub fn look_rows(ui: &UiState) -> Vec<UiChild> {
    let library = ui.appearance.clone();
    let look = &ui.look;
    let mut rows: Vec<UiChild> = vec![
        Box::new(
            el(
                "div",
                (
                    preview(ui),
                    el("div", button("turn".to_owned(), false, |ui| ui.turn_look()))
                        .attr("class", "look-turn"),
                ),
            )
            .attr("class", "look-stage"),
        ),
    ];
    let classes: Vec<UiChild> = library
        .classes()
        .map(|class| {
            let name = class.to_owned();
            button(class.to_owned(), look.sprite == class, move |ui| ui.set_look_class(&name))
        })
        .collect();
    rows.extend(row("Appearance", classes));
    for slot in library.slots() {
        let shown = library
            .slot_layer(&look.sprite, Some(&look.appearance), slot)
            .unwrap_or("");
        let options: Vec<UiChild> = library
            .choices(slot)
            .iter()
            .map(|layer| {
                let label = match layer.split_once(':') {
                    Some((_, name)) => name.to_owned(),
                    None if layer.is_empty() => "none".to_owned(),
                    None => layer.clone(),
                };
                let (slot, layer_name) = (slot.clone(), layer.clone());
                button(label, layer == shown, move |ui| ui.set_look_layer(&slot, &layer_name))
            })
            .collect();
        rows.extend(row(slot, options));
    }
    for swatch in library.swatches() {
        let current = library.colour(&look.sprite, Some(&look.appearance), swatch.index);
        let colours: Vec<UiChild> = swatch
            .colours
            .iter()
            .map(|colour| {
                let (index, colour) = (swatch.index, *colour);
                let class = if current == Some(colour) {
                    "look-swatch look-swatch-active"
                } else {
                    "look-swatch"
                };
                let chip: UiChild = Box::new(clickable(
                    el("div", ())
                        .attr("class", class)
                        .attr("style", format!("background-color: {};", colour_hex(colour))),
                    move |ui: &mut UiState, _| ui.set_look_colour(index, colour),
                ));
                chip
            })
            .collect();
        rows.extend(row(&swatch.label, colours));
    }
    // One strip, for what is held in the main hand (else the first item), so
    // the panel stays within the design height.
    let shown = look
        .items
        .iter()
        .find(|item| item.hand == "main")
        .or_else(|| look.items.first());
    if let Some(item) = shown {
        rows.extend(variant_strip(ui, item));
    }
    rows.push(Box::new(
        el(
            "div",
            button("reset".to_owned(), false, |ui| ui.reset_look()),
        )
        .attr("class", "btn-row"),
    ));
    rows
}

/// The item stamp for the variant strip: the board's, doubled.
const ITEM_BAKE: BakeParams = BakeParams {
    half_w: 4,
    cube_h: 4,
    facings: 4,
    margin: 1,
};

/// An equipped item beside variants of it: the same sort under other
/// material, make and condition keys from the pack, each labelled with its
/// keys. Read-only: it shows what the keys do.
fn variant_strip(ui: &UiState, item: &crate::appearance::ItemLook) -> Vec<UiChild> {
    let library = &ui.appearance;
    let Some(sort) = library.item_sort(item) else {
        return Vec::new();
    };
    let chips: Vec<UiChild> = library
        .item_variants(item, 6)
        .iter()
        .filter_map(|variant| {
            let uri = ui.appearance_bakes.item_uri(library, variant, &ITEM_BAKE)?;
            let keys = [
                variant.material.as_deref().unwrap_or("-"),
                variant.grade.as_deref().unwrap_or("-"),
                variant.condition.as_deref().unwrap_or("-"),
            ]
            .join(" ");
            let chip: UiChild = Box::new(
                el(
                    "div",
                    (
                        el("div", ())
                            .attr("class", "look-item")
                            .attr("style", format!("background-image: url(\"{uri}\"); {SPRITE_FIT}")),
                        el("div", text(keys)).attr("class", "look-item-keys"),
                    ),
                )
                .attr("class", "look-variant"),
            );
            Some(chip)
        })
        .collect();
    row(sort, chips)
}

/// A token's own look editor, omitted until opened from its menu.
pub fn look_overlay(ui: &UiState) -> Option<UiChild> {
    if !ui.look_open {
        return None;
    }
    let token = ui.look.token.and_then(|id| ui.map.token(id))?;
    let actions: Vec<UiChild> = vec![
        button("cancel".to_owned(), false, |ui| ui.cancel_look()),
        button("save".to_owned(), false, |ui| ui.accept_look()),
    ];
    let name = ui
        .map
        .sheet(token.id)
        .and_then(|sheet| sheet.text("name"))
        .map(str::to_owned)
        .unwrap_or_else(|| token.sprite.clone());
    Some(crate::widgets::overlay_panel(
        "sheet",
        format!("Appearance: {name}"),
        actions,
        look_rows(ui),
    ))
}

/// The editor's own rules, appended to the board sheet.
pub(crate) const LOOK_CSS: &str = "
.look-stage { display: flex; align-items: flex-end; margin-bottom: 6px; }
.look-preview {
    width: 120px;
    height: 150px;
    background-color: #12141b;
    border: 1px solid #3d4666;
    background-repeat: no-repeat;
    image-rendering: pixelated;
}
.look-turn { margin-left: 8px; }
.look-label { color: #8d93a6; font-size: 11px; margin-top: 4px; }
.look-swatch {
    width: 16px;
    height: 16px;
    margin-right: 4px;
    margin-bottom: 4px;
    border: 1px solid #3d4666;
}
.look-swatch-active { border: 2px solid #ffffff; }
.look-variant { margin-right: 4px; margin-bottom: 4px; }
.look-item {
    width: 34px;
    height: 58px;
    background-color: #12141b;
    border: 1px solid #3d4666;
    background-repeat: no-repeat;
    image-rendering: pixelated;
}
.look-item-keys { color: #8d93a6; font-size: 8px; width: 36px; }
";
