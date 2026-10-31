//! Player Core creation: legal builds finish, illegal choices are refused
//! with reasons, saved drafts reopen exactly or are refused, and only ORC
//! Remaster records load.

use crate::pf2e::{
    derive::derive,
    draft::{walk, Draft, Progress},
    records::{builtin, Records},
};

/// Answer whatever decision is pending with `value`.
fn answer(draft: &mut Draft, value: &str) -> Result<String, String> {
    let r = builtin();
    let Progress::Pending(slot) = walk(r, draft)?.progress else {
        return Err("nothing pending".into());
    };
    draft.choose(r, &slot.id, value).map(|_| slot.id)
}

fn script(values: &[&str]) -> Draft {
    let mut draft = Draft::default();
    for v in values {
        answer(&mut draft, v).unwrap_or_else(|e| panic!("{v}: {e}"));
    }
    draft
}

const DWARF_FIGHTER: &[&str] = &[
    "dwarf", "listed", "str", "rock-dwarf", "warrior", "str", "con", "fighter", "athletics", "str",
    "dex", "con", "wis", "int", "acrobatics", "medicine", "survival", "crafting", "Gnomish",
    // Dwarven Lore's Crafting is already trained, so it asks for another skill.
    "dwarven-lore", "society", "vicious-swing", "longsword", "steel-shield", "scale-mail", "done",
];

#[test]
fn a_legal_dwarf_fighter_finishes_with_derived_numbers() {
    let draft = script(DWARF_FIGHTER);
    let build = draft.finished(builtin()).unwrap();
    assert_eq!(build.modifier("str"), 3);
    assert_eq!(build.modifier("cha"), -1);
    for skill in ["intimidation", "athletics", "crafting", "religion", "society"] {
        assert!(build.trained(skill), "{skill}");
    }
    for feat in ["shield-block", "reactive-strike", "intimidating-glare", "dwarven-lore", "vicious-swing"] {
        assert!(build.feats.iter().any(|f| f == feat), "{feat}");
    }
    // 10 ancestry + 10 class + Constitution 3.
    assert_eq!(derive(builtin(), &build).hp_max, 23);
    assert_eq!(build.gear, ["clan-dagger", "longsword", "steel-shield", "scale-mail"]);
    assert_eq!(build.coins, 800);
    assert_eq!(build.languages, ["Common", "Dwarven", "Gnomish"]);
}

#[test]
fn doubled_training_and_prerequisites_are_enforced() {
    let mut draft = script(&[
        "human", "dex", "int", "skilled-human", "stealth", "street-urchin", "dex", "con", "rogue",
        // Thief's Thievery and the class's Stealth are both already trained.
        "thief", "athletics", "acrobatics", "dex", "dex", "con", "wis", "cha",
        "arcana", "crafting", "diplomacy", "medicine", "nature", "occultism", "religion", "society",
        "Draconic", "Elven", "natural-ambition",
    ]);
    let walked = walk(builtin(), &draft).unwrap();
    let Progress::Pending(slot) = walked.progress else { panic!("class feat pending") };
    let feint = slot.options.iter().find(|o| o.value == "overextending-feint").unwrap();
    assert_eq!(feint.refused.as_deref(), Some("requires training in Deception"));
    let err = answer(&mut draft, "overextending-feint").unwrap_err();
    assert!(err.contains("requires training in Deception"), "{err}");
    // Pickpocket came from the background, so Plant Evidence is open.
    answer(&mut draft, "plant-evidence").unwrap();
    answer(&mut draft, "hefty-hauler").unwrap();
    answer(&mut draft, "tumble-behind").unwrap();
    // Uncommon gear needs access; a human rogue has none for a clan dagger.
    let walked = walk(builtin(), &draft).unwrap();
    let Progress::Pending(slot) = walked.progress else { panic!("gear pending") };
    let dagger = slot.options.iter().find(|o| o.value == "clan-dagger").unwrap();
    assert_eq!(dagger.refused.as_deref(), Some("uncommon: needs access"));
    answer(&mut draft, "rapier").unwrap();
    answer(&mut draft, "done").unwrap();
    assert!(draft.finished(builtin()).is_ok());
}

#[test]
fn one_source_cannot_boost_an_attribute_twice() {
    let mut draft = script(&["human", "str"]);
    let err = answer(&mut draft, "str").unwrap_err();
    assert!(err.contains("different attributes"), "{err}");
    assert_eq!(draft.choices.len(), 2, "a refused choice is not saved");
}

#[test]
fn a_saved_draft_reopens_exactly_or_is_refused() {
    let draft = script(DWARF_FIGHTER);
    let bytes = serde_json::to_vec(&draft).unwrap();
    let reopened: Draft = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(walk(builtin(), &reopened).unwrap(), walk(builtin(), &draft).unwrap());
    let mut tampered = reopened.clone();
    tampered.choices[0].value = "elf".into();
    assert!(walk(builtin(), &tampered).is_err());
    let mut undone = reopened;
    undone.undo().unwrap();
    assert!(matches!(walk(builtin(), &undone).unwrap().progress, Progress::Pending(s) if s.id == "gear:3"));
}

#[test]
fn only_orc_remaster_records_load() {
    let record = |license: &str, remaster: bool| {
        format!(
            "version = 1\n[[ancestry]]\nid = \"x\"\nname = \"X\"\nsource = {{ book = \"B\", license = \"{license}\", remaster = {remaster} }}\nhp = 8\nsize = \"Medium\"\nspeed = 25\nvision = \"normal\"\nboosts = []\nflaws = []\nlanguages = []\nsummary = \"\"\n"
        )
    };
    assert!(Records::parse(&[&record("ORC", true)]).is_ok());
    assert!(Records::parse(&[&record("OGL", true)]).unwrap_err().contains("ORC"));
    assert!(Records::parse(&[&record("ORC", false)]).unwrap_err().contains("Remaster"));
}
