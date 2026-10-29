use super::super::*;

#[test]
fn old_map_json_is_unchanged_and_unsupported_or_tampered_state_is_refused() {
    let map = MapDocument::new("legacy", 2, 2);
    let value = serde_json::to_value(&map).unwrap();
    assert!(value.get("system_data").is_none());
    assert_eq!(serde_json::from_value::<MapDocument>(value).unwrap(), map);
    let mut scene = Scene::new(Edition::Dnd5, 73);
    let request = scene.request("dnd5:darkvision", "state");
    scene.cast(&request, 0, false).unwrap();
    let saved = Records::read(&scene.map).unwrap().unwrap();
    assert_eq!(saved.version, 1);
    assert!(saved.casts[0].experimental);
    let mut bad = saved.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
    let mut bad = saved.clone();
    bad.effects.get_mut(&1).unwrap().target = Target::Creature(CASTER);
    assert!(bad.validate().unwrap_err().contains("history disagrees"));
    let mut bad = saved.clone();
    bad.casts.push(bad.casts[0].clone());
    assert!(bad.validate().is_err());
    scene.map.system_data.insert(
        "isometry:active-effects".into(),
        FieldValue::Text("corrupt".into()),
    );
    assert!(Records::read(&scene.map).is_err());
    assert!(project_sheet(&scene.map, BEARER, None, scene.map.sheet(BEARER).unwrap()).is_err());
}
