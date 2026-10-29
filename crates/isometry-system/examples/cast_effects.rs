//! Run with `cargo run -p isometry-system --example cast_effects`.
//! The JSON is an execution receipt for an authored VTT board, not a Mesocosm
//! play capture or an assertion that the complete rulesets are ported.
mod effect_demo;
use effect_demo::*;
use isometry_campaign::ItemId;
use isometry_system::active_effects::*;
use serde_json::json;

fn main() -> Result<(), String> {
    let mut output = Vec::new();
    for edition in [Edition::Dnd5, Edition::Pathfinder2, Edition::Ars5] {
        let mut scene = Scene::new(edition, 73);
        let definitions: Vec<_> = scene
            .definitions
            .0
            .values()
            .filter(|d| d.edition == edition)
            .cloned()
            .collect();
        for (index, def) in definitions.iter().enumerate() {
            let request = scene.request(&def.id, &format!("demo:{index}"));
            // End each independent demonstration before the next one, so the
            // two D&D concentration spells do not hide one another here.
            let receipt = scene.cast(&request, 0, false)?;
            let effect_id = receipt.effect.ok_or("demonstration did not activate")?;
            let target = if def.personal { CASTER } else { BEARER };
            let sheet = scene.sheet(target);
            let geometry = project_body(
                &scene.map,
                target,
                &body(),
                Some(BodyChange {
                    numerator: 2,
                    denominator: 1,
                }),
            )?;
            let records = Records::read(&scene.map)?.unwrap();
            let active = records.effects[&effect_id].clone();
            let weapon_change = weapon(&scene.map, &ItemId("demo:sword".into()))?;
            let saved = serde_json::to_string(&scene.map).map_err(|e| e.to_string())?;
            scene.map = serde_json::from_str(&saved).map_err(|e| e.to_string())?;
            if Records::read(&scene.map)? != Some(records) {
                return Err("save roundtrip changed effects".into());
            }
            set_suppression(&mut scene.map, effect_id, clock(0), "demo:ward", true)?;
            let suppressed = scene.sheet(target);
            set_suppression(&mut scene.map, effect_id, clock(0), "demo:ward", false)?;
            if scene.sheet(target) != sheet {
                return Err("resume changed effect".into());
            }
            // Each case uses its own fresh board; duration is the edition's.
            advance(&mut scene.map, clock(active.expires))?;
            output.push(json!({"procedure":def.id,"receipt":receipt,"active":active,
                "sheet":sheet,"body_half_extent":geometry.parts[0].half_extent,
                "near_dark":perceive(&sheet,Lighting::NearDark,20),
                "absolute_dark":perceive(&sheet,Lighting::AbsoluteDark,20),
                "weapon":weapon_change,"suppressed_sheet":suppressed,
                "expired_sheet":scene.sheet(target),"restored":true}));
            scene = Scene::new(edition, 73);
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({"version":1,"seed":73,
        "scope":"Nine executable VTT procedures on an authored board; no headed casting controls or native biological magic.",
        "cases":output})).map_err(|e| e.to_string())?);
    Ok(())
}
