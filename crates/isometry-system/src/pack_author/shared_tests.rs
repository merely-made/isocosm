//! Adapter contracts: host identities, legacy wire receipts and publication refusal.
use super::*;

#[test]
fn shared_plan_uses_host_identity_and_reopen_instance() {
    let f = Fixture::new();
    let s = f.session();
    let other = f.session();
    let stamp = s.source_stamp();
    assert_eq!(stamp.document, s.path.display().to_string());
    assert_eq!(stamp.revision, s.current.revision);
    assert_eq!(stamp.instance, s.instance);
    assert_ne!(stamp.instance, other.instance);
    let plan = Proposal::prepare(
        stamp.clone(),
        &s.current.text,
        vec![Edit {
            start: 0,
            end: 0,
            text: "# reviewed\n".into(),
        }],
    )
    .unwrap();
    assert!(
        plan.application(&other.source_stamp(), &other.current.text)
            .is_err()
    );
    let mut wrong_document = stamp.clone();
    wrong_document.document.push_str(".other");
    assert!(plan.application(&wrong_document, &s.current.text).is_err());
    assert!(
        plan.application(&stamp, "same revision is not sufficient")
            .is_err()
    );
    let applied = plan.application(&stamp, &s.current.text).unwrap();
    assert_eq!(applied.text, plan.preview());
    assert_eq!(applied.receipt.result.revision, stamp.revision + 1);
    assert_eq!(applied.receipt.result.instance, stamp.instance);
    assert_eq!(applied.receipt.result.document, stamp.document);
}

#[test]
fn shared_edits_preserve_flat_cli_receipts_and_strict_input() {
    let f = Fixture::new();
    let mut s = f.session();
    let original = s.current.text.to_string();
    let end = original.len();
    let edits: Vec<Edit> = serde_json::from_value(serde_json::json!([
        {"start":end,"end":end,"text":"# tail 雪\n"},
        {"start":0,"end":0,"text":"# front 🦀\n"}
    ]))
    .unwrap();
    let front = "# front 🦀\n";
    let tail = "# tail 雪\n";
    let receipt = s.edit(1, edits).unwrap();
    assert_eq!(
        serde_json::to_value(receipt).unwrap(),
        serde_json::json!({
            "document":s.path.display().to_string(), "base":1, "result":2,
            "operation":"edit", "ranges":[
                {"base_start":0,"base_end":0,"result_start":0,"result_end":front.len()},
                {"base_start":end,"base_end":end,"result_start":end+front.len(),
                 "result_end":end+front.len()+tail.len()}
            ]
        })
    );
    assert_eq!(s.current.text.as_ref(), format!("{front}{original}{tail}"));
    assert_eq!(fs::read_to_string(&s.path).unwrap(), original);
    assert_eq!(s.saved_revision, 1);
    assert!(
        serde_json::from_value::<Edit>(serde_json::json!({
            "start":0,"end":0,"text":"x","unknown":true
        }))
        .is_err()
    );
    let before = s.current.clone();
    for edits in [
        vec![],
        vec![
            Edit {
                start: 0,
                end: 0,
                text: "a".into(),
            },
            Edit {
                start: 0,
                end: 0,
                text: "b".into(),
            },
        ],
    ] {
        assert!(s.edit(2, edits).is_err());
        assert!(Arc::ptr_eq(&before, &s.current));
    }
}

#[test]
fn shared_revision_overflow_does_not_publish_or_save() {
    let f = Fixture::new();
    let mut s = f.session();
    let disk = fs::read(&s.path).unwrap();
    s.current = Snapshot::build(u64::MAX, s.current.text.to_string(), s.format).unwrap();
    let before = s.current.clone();
    assert_eq!(
        s.edit(
            u64::MAX,
            vec![Edit {
                start: 0,
                end: 0,
                text: "# overflow\n".into()
            }]
        )
        .unwrap_err(),
        "revision overflow"
    );
    assert!(Arc::ptr_eq(&before, &s.current));
    assert_eq!(s.history.len(), 1);
    assert_eq!(s.saved_revision, 1);
    assert_eq!(fs::read(&s.path).unwrap(), disk);
    assert_eq!(s.undo(u64::MAX, 1).unwrap_err(), "revision overflow");
    assert!(Arc::ptr_eq(&before, &s.current));
    assert_eq!(s.history.len(), 1);
    assert_eq!(s.saved_revision, 1);
    assert_eq!(fs::read(&s.path).unwrap(), disk);
}

#[test]
fn undo_uses_shared_ranges_and_revision_without_implicit_save() {
    let f = Fixture::new();
    let mut s = f.session();
    let original = s.current.clone();
    let end = original.text.len();
    s.edit(
        1,
        vec![Edit {
            start: end,
            end,
            text: "# draft\n".into(),
        }],
    )
    .unwrap();
    let edited_len = s.current.text.len();
    let receipt = s.undo(2, 1).unwrap();
    assert_eq!(
        serde_json::to_value(receipt).unwrap(),
        serde_json::json!({
            "document":s.path.display().to_string(), "base":2, "result":3,
            "operation":"undo", "ranges":[{"base_start":0,"base_end":edited_len,
                "result_start":0,"result_end":end}]
        })
    );
    assert_eq!(s.current.text, original.text);
    assert_eq!(s.history.len(), 3);
    assert_eq!(s.saved_revision, 1);
    assert_eq!(fs::read_to_string(&s.path).unwrap(), original.text.as_ref());
    let before = s.current.clone();
    assert!(s.undo(2, 1).is_err());
    assert!(Arc::ptr_eq(&before, &s.current));
    assert_eq!(s.undo(3, 3).unwrap().result, 4);
    assert_eq!(s.current.text, original.text);
    assert_eq!(s.saved_revision, 1);
}
