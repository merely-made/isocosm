//! Ars Magica at the table (cases 1063, 1068): creation as a saved draft,
//! formulaic casting through the F11 procedures, Wounds that reach the
//! native body, and a season's study from a covenant summa begun and
//! completed across consented native downtime. The covenant lives in edition
//! state for now (case 1063).

use super::*;
use cosmograph::Command;
use cosmograph::asserted::{Assertion, HistoryLine};
use isocosm_system::active_effects::{CastContext, Definitions, Target, cast, Cast};
use isocosm_system::ars::{self, draft, records};

/// A summa in the covenant's library: a book on one Art or Ability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summa {
    pub subject: String,
    pub level: u32,
    pub quality: u32,
}

/// A season of study under way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Study {
    pub summa: usize,
    pub until: u64,
}

/// The covenant as edition state: its library and the studies under way.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Covenant {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<Summa>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub studies: BTreeMap<TokenId, Study>,
}

/// Every Ars draft still walks, and every Ars sheet has its draft.
pub(super) fn check(state: &State) -> Result<(), String> {
    let r = records::builtin();
    for d in state.drafts.values() {
        draft::walk(r, d)?;
    }
    for id in state.sheets.keys() {
        draft::finished(r, state.drafts.get(id).ok_or("an Ars sheet has no creation record")?)?;
    }
    Ok(())
}

/// Native ticks in one season: the native year split as the pack declares.
pub fn season_ticks(session: &Session) -> u64 {
    let unit = session.sim.genesis().rules.tick_microseconds();
    cosmograph::rules::year_ticks(unit) / records::builtin().time.seasons_per_year
}

fn year_ticks(session: &Session) -> u64 {
    cosmograph::rules::year_ticks(session.sim.genesis().rules.tick_microseconds())
}

/// A character's age now: the age chosen in the draft, plus the native years
/// since the sheet was made (case 1069).
pub fn age(sheet: &isocosm_core::SheetData, session: &Session) -> i64 {
    let made = u64::try_from(sheet.int(&ars::field("made")).unwrap_or(0)).unwrap_or(0);
    let years = session.sim.state().tick.saturating_sub(made) / year_ticks(session).max(1);
    sheet.int(&ars::field("age")).unwrap_or(0) + i64::try_from(years).unwrap_or(0)
}

fn native(session: &mut Session, command: Command) -> Result<(), String> {
    session.command(command).map(|_| ()).map_err(|e| format!("native body refused: {e}"))
}

fn unfinished<'a>(state: &'a mut State, map: &MapDocument, id: TokenId) -> Result<&'a mut draft::Draft, String> {
    if map.sheets.contains_key(&id) {
        return Err("this character is already finished".into());
    }
    state.drafts.get_mut(&id).ok_or_else(|| "this body has no creation draft".into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewSumma {
    subject: String,
    level: u32,
    quality: u32,
}

pub(super) fn apply(
    state: &mut State,
    map: &mut MapDocument,
    session: &mut Session,
    access: OrdinaryAccess,
    action: &Action,
    payload: &[u8],
) -> Result<(String, Vec<RollRecord>), String> {
    use frame::authorize;
    let r = records::builtin();
    let mut rolls = Vec::new();
    let note = match action {
        Action::Create(actor) => {
            let id = authorize(session, access, *actor)?;
            if map.sheets.contains_key(&id) || state.drafts.contains_key(&id) {
                return Err("this body already has a character or a draft".into());
            }
            state.drafts.insert(id, draft::Draft::default());
            format!("Ars Magica draft started for native body {actor}; experimental ruleset facts")
        },
        Action::Choose { actor, slot, value } => {
            let id = authorize(session, access, *actor)?;
            let made = draft::choose(r, unfinished(state, map, id)?, slot, value)?;
            format!("Character {actor} chose {} for {}", made.label, made.title)
        },
        Action::Undo(actor) => {
            let id = authorize(session, access, *actor)?;
            let d = unfinished(state, map, id)?;
            let last = draft::walk(r, d)?.made.pop().ok_or("nothing to take back")?;
            d.choices.pop();
            format!("Character {actor} took back {} for {}", last.label, last.title)
        },
        Action::Finish(actor) => {
            let id = authorize(session, access, *actor)?;
            let build = draft::finished(r, unfinished(state, map, id)?)?;
            let now = session.sim.state().tick;
            let mut sheet = ars::sheet(r, &build, &format!("Character {actor}"));
            sheet.set_int(ars::field("made"), i64::try_from(now).map_err(|_| "clock overflow")?);
            map.set_sheet(id, sheet);
            // The birth, dated before the sheet in the world's timeline, as a
            // native history line about the character (case 1069). Native
            // bodies stay born at tick 0; a body with no authored key has no
            // name a line can carry, so it keeps the age as edition state only.
            let key = session.sim.state().population.get(*actor).and_then(|b| b.authored.as_ref()).map(|a| a.key.clone());
            let born = i64::try_from(now).map_err(|_| "clock overflow")?
                - i64::from(build.age) * i64::try_from(year_ticks(session)).map_err(|_| "clock overflow")?;
            if let Some(key) = key {
                let line = HistoryLine {
                    key: format!("ars-birth-{actor}"), time: born, kind: "birth".into(),
                    text: format!("{key} is born, {} years before the saga", build.age),
                    participants: vec![key], place: None, tags: ["ars5".to_owned()].into_iter().collect(),
                };
                native(session, Command::Assert(Assertion::History(line)))?;
            }
            let house = build.house.as_deref().and_then(|h| r.house(h)).map_or(String::new(), |h| format!(" of House {}", h.name));
            format!("Character {actor} finished as a {}{house}, {} years old", if build.magus() { "magus" } else { "grog" }, build.age)
        },
        Action::Discard(actor) => {
            let id = authorize(session, access, *actor)
                .or_else(|refused| pathfinder::gm(access).map_err(|_| refused).and_then(|()| token(*actor)))?;
            unfinished(state, map, id)?;
            state.drafts.remove(&id);
            format!("Character {actor}'s unfinished draft was discarded")
        },
        Action::CastSpell { actor, spell } => {
            let id = authorize(session, access, *actor)?;
            let sheet = map.sheet(id).ok_or("no finished Ars character")?;
            if sheet.fields.get(&ars::field(&format!("spell:{spell}"))) != Some(&FieldValue::Bool(true)) {
                return Err("this magus has not learned that spell".into());
            }
            let s = r.spell(spell).ok_or("unknown spell")?;
            let procedure = s.procedure.clone()
                .ok_or_else(|| format!("{} is learned, but its procedure is not built at this table", s.name))?;
            let defs = Definitions::builtin()?;
            if defs.get(&procedure)?.kind == isocosm_system::active_effects::Kind::Weapon {
                return Err(format!("{} needs a wielded metal weapon; Ars equipment is not in this slice", s.name));
            }
            let request = Cast {
                request_id: format!("table:{}:{}:{}", session.sim.state().tick, access.participant.0, state.log.len()),
                definition: procedure,
                caster: id,
                target: Target::Creature(id),
                rank: u8::try_from(s.level).map_err(|_| "spell level")?,
                size_change: if spell.contains("growth") { 1 } else { 0 },
                willing: true,
                ruling: None,
                resisted: false,
            };
            let context = CastContext { clock: clock(session), feet_per_tile: 5, in_combat: false, line_of_effect: &|_, _| true };
            let receipt = cast(map, &state.inventories, &defs, &request, &context, &mut state.rng)?;
            let total = receipt.casting_total.unwrap_or(0);
            rolls.push(RollRecord { by: format!("{actor} {}", s.name), expr: format!("casting total vs level {}", s.level),
                dice: receipt.dice.clone(), total: i32::try_from(total).unwrap_or(i32::MAX) });
            format!("Character {actor} casts {} (casting total {total} against level {}): {}", s.name, s.level, receipt.outcome)
        },
        Action::Damage { target, amount } => {
            pathfinder::gm(access)?;
            let id = token(*target)?;
            let sheet = map.sheets.get_mut(&id).ok_or("target has no Ars sheet")?;
            if sheet.int("ars:dead").unwrap_or(0) > 0 {
                return Err("the target is already dead".into());
            }
            let soak = sheet.int("ars:sta").unwrap_or(0);
            let size = sheet.int("ars:size").unwrap_or(0);
            let excess = i64::from(*amount) - soak;
            match isocosm_system::ars::rules::wound(r, excess, size) {
                None => format!("GM ruling: {amount} damage against Soak {soak}; no Wound"),
                Some(level) => {
                    let name = r.wounds.order[level].clone();
                    if name == "dead" {
                        sheet.set_int("ars:dead", 1);
                        native(session, Command::Kill { entity: *target })?;
                        format!("GM ruling: {amount} damage against Soak {soak}; dead, and the native body dies")
                    } else {
                        let key = format!("ars:wounds:{name}");
                        sheet.set_int(&key, sheet.int(&key).unwrap_or(0) + 1);
                        // Fixed counts until case 1069's size scaling is settled.
                        let cells = r.wounds.native_cells.get(&name).copied().ok_or("Wound cells undeclared")?;
                        native(session, Command::Wound { entity: *target, part: None, cells })?;
                        format!("GM ruling: {amount} damage against Soak {soak}; a {name} Wound, which takes {cells} native cells")
                    }
                },
            }
        },
        Action::AddSumma => {
            pathfinder::gm(access)?;
            let new: NewSumma = serde_json::from_slice(payload).map_err(|_| "enter a summa: subject, level and quality")?;
            if r.art(&new.subject).is_none() && r.ability(&new.subject).is_none() {
                return Err(format!("{} is not an Art or Ability of this slice", new.subject));
            }
            if new.level == 0 || new.quality == 0 {
                return Err("a summa has a Level and a Quality above zero".into());
            }
            let covenant = state.covenant.get_or_insert_with(Covenant::default);
            covenant.library.push(Summa { subject: new.subject.clone(), level: new.level, quality: new.quality });
            format!("GM ruling: the covenant's library gains a summa on {}, Level {}, Quality {}", r.name(&new.subject), new.level, new.quality)
        },
        Action::Study { actor, summa } => {
            let id = authorize(session, access, *actor)?;
            map.sheet(id).ok_or("no finished Ars character")?;
            let covenant = state.covenant.get_or_insert_with(Covenant::default);
            let book = covenant.library.get(*summa).ok_or("no such summa in the library")?.clone();
            if covenant.studies.contains_key(&id) {
                return Err("one source a season: this character is already studying".into());
            }
            let now = session.sim.state().tick;
            let until = now.checked_add(season_ticks(session)).ok_or("clock overflow")?;
            covenant.studies.insert(id, Study { summa: *summa, until });
            format!("Character {actor} begins a season studying the summa on {}; completes after native tick {until}", r.name(&book.subject))
        },
        Action::Complete(actor) => {
            let id = authorize(session, access, *actor)?;
            let now = session.sim.state().tick;
            let covenant = state.covenant.as_mut().ok_or("no study under way")?;
            let study = covenant.studies.get(&id).ok_or("no study under way")?.clone();
            if now < study.until {
                return Err(format!("{} native ticks of the season remain; propose downtime first", study.until - now));
            }
            let book = covenant.library[study.summa].clone();
            covenant.studies.remove(&id);
            let sheet = map.sheets.get_mut(&id).ok_or("no finished Ars character")?;
            let gain = ars::study(r, sheet, &book.subject, book.level, book.quality)?;
            let score = sheet.int(&ars::field(&book.subject)).unwrap_or(0);
            format!("Character {actor} completes a season of study: {gain} experience in {}, now {score}", r.name(&book.subject))
        },
        _ => return Err("this action is not part of Ars Magica's first slice".into()),
    };
    Ok((note, rolls))
}
