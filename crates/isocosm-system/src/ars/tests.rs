use super::*;
use draft::{Draft, Progress, choose, finished, walk};
use records::builtin;
use sample::BONISAGUS_MAGUS;

fn drive(script: &[(&str, &str)]) -> Draft {
    let r = builtin();
    let mut d = Draft::default();
    for (slot, value) in script {
        choose(r, &mut d, slot, value).unwrap_or_else(|e| panic!("{slot}={value}: {e}"));
    }
    d
}

#[test]
fn a_bonisagus_magus_is_built_from_explained_choices() {
    let r = builtin();
    let d = drive(BONISAGUS_MAGUS);
    let build = finished(r, &d).unwrap();
    assert_eq!(build.house.as_deref(), Some("bonisagus"));
    assert_eq!(build.age, 25);
    assert_eq!(build.art("muto"), 10);
    assert_eq!(build.ability("dead-language"), 4);
    assert_eq!(build.ability("magic-theory"), 3);
    assert_eq!(build.spells.len(), 7);
    let s = sheet(r, &build, "Magus");
    assert_eq!(s.int("ars:muto"), Some(10));
    assert_eq!(s.int("ars:puissant:muto"), Some(3));
    assert_eq!(s.int("ars:puissant:magic-theory"), Some(2), "the House's free Virtue");
    assert_eq!(s.fields.get("ars:deficient:ignem"), Some(&FieldValue::Bool(true)));
    assert_eq!(s.int("ars:living-language"), Some(5));
}

#[test]
fn refused_options_say_why_and_cannot_be_chosen() {
    let r = builtin();
    let mut d = drive(&BONISAGUS_MAGUS[..3]);
    let Progress::Pending(slot) = walk(r, &d).unwrap().progress else { panic!("pending") };
    let focus = slot.options.iter().find(|o| o.value == "minor-magical-focus").unwrap();
    assert!(focus.refused.as_deref().unwrap().contains("storyguide"));
    assert!(choose(r, &mut d, "virtue", "minor-magical-focus").is_err());
    let done = slot.options.iter().find(|o| o.value == "done").unwrap();
    assert!(done.refused.is_none(), "no Virtues yet, so none outweigh Flaws");
    choose(r, &mut d, "virtue", "puissant-art").unwrap();
    choose(r, &mut d, "target", "muto").unwrap();
    let Progress::Pending(slot) = walk(r, &d).unwrap().progress else { panic!("pending") };
    assert!(slot.options.iter().find(|o| o.value == "done").unwrap().refused.is_some());
}

#[test]
fn a_grog_takes_only_minor_traits_and_may_stop_spending() {
    let r = builtin();
    let mut script = vec![("type", "grog"), ("virtue", "warrior"), ("virtue", "covenant-upbringing"), ("virtue", "done")];
    for c in ["int", "per", "pre", "com", "str", "sta", "dex", "qik"] {
        script.push((Box::leak(format!("char-{c}").into_boxed_str()), "0"));
    }
    script.extend([("age", "20"), ("childhood", "done"), ("later-life", "done"), ("bonus-warrior", "single-weapon"),
        ("bonus-warrior", "done")]);
    let d = drive(&script);
    let build = finished(r, &d).unwrap();
    assert!(build.has("covenfolk") && !build.has("the-gift"));
    assert_eq!(build.ability("single-weapon"), 1);
    let Progress::Pending(slot) = walk(r, &drive(&script[..1])).unwrap().progress else { panic!() };
    let enemies = slot.options.iter().find(|o| o.value == "enemies");
    assert!(enemies.is_none(), "Enemies is a magus's Flaw in this slice");
    assert!(slot.options.iter().find(|o| o.value == "puissant-art").is_none());
}

#[test]
fn a_season_with_a_summa_raises_the_score_to_its_level() {
    let r = builtin();
    let build = finished(r, &drive(BONISAGUS_MAGUS)).unwrap();
    let mut s = sheet(r, &build, "Magus");
    let before = s.int("ars:xp:corpus").unwrap();
    let gain = study(r, &mut s, "corpus", 12, 10).unwrap();
    assert_eq!(gain, 10);
    assert_eq!(s.int("ars:xp:corpus"), Some(before + 10));
    assert_eq!(s.int("ars:corpus"), Some(i64::from(rules::score(u32::try_from(before).unwrap() + 10, rules::art_xp))));
    assert!(study(r, &mut s, "corpus", 2, 10).is_err(), "a summa below the score teaches nothing");
}
