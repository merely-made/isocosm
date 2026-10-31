use super::*;

/// Feed the existing dice parser, preserving every later damage term. These
/// fields are produced by the ruleset projection for its main-hand action.
pub(crate) fn damage_expression(
    action: &str,
    sheet: &SheetData,
    base: &str,
) -> Result<(String, i64, i32), String> {
    if sheet.text("effect_weapon_action") != Some(action) {
        return Ok((base.into(), 0, 0));
    }
    let mut expression = base.to_owned();
    let count = sheet.int("effect_weapon_dice").unwrap_or(0);
    if count > 0 {
        if count > 100 {
            return Err("weapon damage dice exceed the existing dice limit".into());
        }
        let (old, rest) = base
            .split_once('d')
            .ok_or("weapon action has no leading damage die")?;
        let old: i64 = old
            .parse()
            .map_err(|_| "weapon action has no leading damage count")?;
        expression = format!("{}d{rest}", old.max(count));
    }
    let size = sheet.int("effect_weapon_d4").unwrap_or(0);
    match size {
        1 => expression.push_str("+1d4"),
        -1 => expression.push_str("-1d4"),
        0 => {},
        _ => return Err("invalid enlarge/reduce weapon die".into()),
    }
    Ok((
        expression,
        sheet.int("effect_weapon_damage").unwrap_or(0),
        if size < 0 { 1 } else { 0 },
    ))
}
