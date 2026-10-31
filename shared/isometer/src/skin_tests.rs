use isometer_core::{Attachment, BodyDocument, VolumeRef, Yaw};
use isometer_mesh::VolumeSource;

use super::*;

fn body() -> BodyDocument {
    let mut document = BodyDocument::new(VolumeRef([0; 32]), [3, 2, 2]);
    document
        .attach(
            VolumeRef([0; 32]),
            [1, 1, 3],
            Attachment { parent: PartId(0), offset: [4, 0, 0], yaw: Yaw::Zero },
            None,
        )
        .unwrap();
    document
}

fn part(id: u32, matter: &[(&str, u64)]) -> PartSkin {
    PartSkin {
        matter: matter.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
        ..PartSkin::new(PartId(id))
    }
}

#[test]
fn forms_stay_inside_each_declared_extent_and_only_volume_refs_change() {
    let document = body();
    let parts = [part(0, &[("matter:a", 10)]), part(1, &[("matter:b", 10)])];
    let skin = skin(
        [SkinBody { subject: SubjectKey(1), document: &document, parts: &parts, seed: 3 }],
        &SkinRules::default(),
    )
    .unwrap();
    let drawn = &skin.documents[&SubjectKey(1)];
    for (original, skinned) in document.parts.iter().zip(&drawn.parts) {
        let mut same = skinned.clone();
        same.volume = original.volume;
        assert_eq!(&same, original, "only the volume reference differs");
        let (_, volume) = skin.volumes.resolve_part(skinned).unwrap();
        assert_eq!(volume.size, original.half_extent.map(|h| h as u32 * 2));
    }
}

#[test]
fn colour_comes_from_matter_and_the_seed_moves_colour_only() {
    let document = body();
    let named = SkinRules {
        matter: BTreeMap::from([("matter:a".to_owned(), [200, 0, 0]), ("matter:b".to_owned(), [0, 0, 200])]),
        jitter: 0,
        ..SkinRules::default()
    };
    let mixed = [part(0, &[("matter:a", 1), ("matter:b", 1)])];
    let a = skin([SkinBody { subject: SubjectKey(1), document: &document, parts: &mixed, seed: 0 }], &named).unwrap();
    assert!(a.palettes[&SubjectKey(1)].contains(&[100, 0, 100]), "mixed by amount");

    let rules = SkinRules::default();
    let one = skin([SkinBody { subject: SubjectKey(1), document: &document, parts: &mixed, seed: 1 }], &rules).unwrap();
    let two = skin([SkinBody { subject: SubjectKey(1), document: &document, parts: &mixed, seed: 99_999 }], &rules).unwrap();
    assert_ne!(one.palettes[&SubjectKey(1)], two.palettes[&SubjectKey(1)]);
    let shapes = |s: &Skin| {
        s.documents[&SubjectKey(1)]
            .parts
            .iter()
            .map(|p| s.volumes.resolve_part(p).unwrap().1.solid_count())
            .collect::<Vec<_>>()
    };
    assert_eq!(shapes(&one), shapes(&two), "the seed never changes a form");
}

#[test]
fn function_marks_share_the_existing_channels() {
    let input = [PartSkin {
        capacity: 8,
        functions: BTreeMap::from([
            ("function:intake".into(), 4),
            ("function:store".into(), 2),
            ("function:reproduce".into(), 1),
        ]),
        ..PartSkin::new(PartId(4))
    }];
    let output = function_marks(&input);
    assert_eq!(output.len(), 2);
    assert_eq!(output[0].fraction, 0.5);
    assert_eq!(output[1].fraction, 0.375);
    assert!(function_marks(&[]).is_empty());
}
