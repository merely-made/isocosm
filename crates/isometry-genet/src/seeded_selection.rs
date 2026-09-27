//! The GM's `>choose` without Cleromancy: the VTT's own seeded draw.
//!
//! Cleromancy is an optional host feature, off by default (ruled 2026-09-26),
//! so this is the default build's path. The request's seed and domain seed the
//! generator lane's entropy tape and one draw picks a loaded declaration. The
//! same request over the same choices replays the same pick; there is no sealed
//! receipt, and the preview and commit gates are unchanged.

use isometry_campaign::{EntropyTape, GeneratorChoice};
use isometry_views::GeneratorSelectionRequest;

/// The host-local result of one seeded choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GeneratorSelection {
    pub choice_index: usize,
    candidate_id: String,
    tape: EntropyTape,
}

impl GeneratorSelection {
    pub(crate) fn candidate_id(&self) -> &str {
        &self.candidate_id
    }

    /// The tape that made the pick: its seed and its one draw.
    pub(crate) fn receipt(&self) -> &EntropyTape {
        &self.tape
    }

    pub(crate) fn status(&self, choice_name: &str) -> String {
        format!(
            "seeded draw chose {choice_name} (seed {:016x}); generating preview",
            self.tape.seed
        )
    }
}

/// Draw one declared generator choice from public GM input.
pub(crate) fn select_generator(
    choices: &[GeneratorChoice],
    request: &GeneratorSelectionRequest,
) -> Result<GeneratorSelection, String> {
    if choices.is_empty() {
        return Err("no generator packs loaded".to_owned());
    }
    if request.prompt.trim().is_empty() {
        return Err("choose needs a prompt".to_owned());
    }
    let mut tape = EntropyTape::from_seed(request_seed(&request.seed, &request.domain));
    let choice_index = (tape.draw() % choices.len() as u64) as usize;
    Ok(GeneratorSelection {
        choice_index,
        candidate_id: choices[choice_index].id.clone(),
        tape,
    })
}

/// FNV-1a over the seed, a separator and the domain: a fixed hash, so the
/// same request draws the same pick on any build.
fn request_seed(seed: &str, domain: &str) -> u64 {
    let bytes = seed.bytes().chain([0]).chain(domain.bytes());
    bytes.fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_campaign::{GenValue, GeneratorRequest};
    use isometry_system::{GeneratorCatalog, GeneratorLimits};

    use crate::boot::generator_pack_roots;

    #[test]
    fn seeded_draw_selects_a_real_loaded_generator_and_replays() {
        let catalog = GeneratorCatalog::discover(generator_pack_roots());
        let choices = catalog.choices();
        assert!(!choices.is_empty(), "bundled generator packs must load");
        let request = GeneratorSelectionRequest {
            seed: "session-4".to_owned(),
            domain: "isometry.generator-preview/v1".to_owned(),
            prompt: "What should I prepare for the next scene?".to_owned(),
        };

        let selection = select_generator(&choices, &request).unwrap();
        assert_eq!(select_generator(&choices, &request).unwrap(), selection);
        let choice = &choices[selection.choice_index];
        assert_eq!(choice.id, selection.candidate_id());
        let mut tape = EntropyTape::from_seed(7);
        let record = catalog
            .generate(
                "seeded.selected.preview",
                &GeneratorRequest {
                    generator: choice.id.clone(),
                    args: choice.default_args.clone(),
                    locks: Default::default(),
                },
                &mut tape,
                GeneratorLimits::default(),
            )
            .unwrap();
        assert_eq!(record.request.generator, selection.candidate_id());
    }

    #[test]
    fn empty_prompt_does_not_queue_a_choice() {
        let choice = GeneratorChoice {
            id: "demo:npc".to_owned(),
            name: "NPC".to_owned(),
            default_args: GenValue::Text {
                value: "example".to_owned(),
            },
            lock_presets: Vec::new(),
        };
        let request = GeneratorSelectionRequest {
            seed: "seed".to_owned(),
            domain: "isometry.generator-preview/v1".to_owned(),
            prompt: " ".to_owned(),
        };
        assert_eq!(
            select_generator(&[choice], &request).unwrap_err(),
            "choose needs a prompt"
        );
    }
}
