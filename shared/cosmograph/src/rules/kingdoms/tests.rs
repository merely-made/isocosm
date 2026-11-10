use super::*;

fn rules(kingdoms: Option<Kingdoms>) -> Rules {
    let mut rules = crate::probe::BodyFounding::default().generate().unwrap().genesis.rules;
    rules.kingdoms = kingdoms;
    rules
}

fn part(cells: &[(&str, u32)]) -> Part {
    Part {
        cells: cells.iter().map(|(f, n)| (format!("function:{f}"), *n)).collect(),
        ..Default::default()
    }
}

fn plants(cells: &[(&str, u32)], price: u128) -> u128 {
    weighed(&rules(Some(Kingdoms::default())), "kingdom:flora", &part(cells), price)
}

#[test]
fn unset_or_unknown_leaves_the_price() {
    let p = part(&[("fix", 3)]);
    assert_eq!(weighed(&rules(None), "kingdom:flora", &p, 101), 101);
    assert_eq!(weighed(&rules(Some(Kingdoms::default())), "kingdom:none", &p, 101), 101);
    assert_eq!(plants(&[], 101), 101, "a part with no cells");
    assert_eq!(plants(&[("intake", 4)], 101), 101, "an unweighed function");
}

#[test]
fn weights_halve_double_and_mix_by_cell_share() {
    assert_eq!(plants(&[("fix", 2)], 100), 50);
    assert_eq!(plants(&[("contract", 2)], 100), 200);
    // One cell of each: the mean of 1/2 and 2 is 5/4.
    assert_eq!(plants(&[("fix", 1), ("contract", 1)], 100), 125);
    // Three halved cells and one unweighed: 5/8 of 100 is 62.5, ties down.
    assert_eq!(plants(&[("fix", 3), ("intake", 1)], 100), 62);
    // Past the half it rounds up: 5/8 of 102 is 63.75.
    assert_eq!(plants(&[("fix", 3), ("intake", 1)], 102), 64);
}

#[test]
fn the_first_cut_is_valid_and_bad_weights_are_refused() {
    let world = rules(None);
    Kingdoms::default().validate(&world).unwrap();
    let weigh = |f: &str, w: [u32; 2]| Kingdoms {
        weights: BTreeMap::from([("kingdom:flora".to_string(), BTreeMap::from([(f.to_string(), w)]))]),
    };
    assert!(weigh("function:fix", [1, 0]).validate(&world).is_err());
    assert!(weigh("function:nope", [1, 2]).validate(&world).is_err());
    let mut set = rules(Some(weigh("function:fix", [1, 0])));
    assert!(set.validate().is_err());
    set.kingdoms = Some(Kingdoms::default());
    set.validate().unwrap();
}
