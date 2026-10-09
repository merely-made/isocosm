"""Checkpoint 9's planted faults: each a one-line edit the named test must
catch. Applies one fault, runs its test, restores the file from a saved copy
with a fresh time, and prints the verdict. Run from shared/isocosm; an
argument picks faults by index."""
import os, shutil, subprocess, sys

FAULTS = [
    ('a sense reaches through parts that cannot carry (663)',
     'src/systems/nerves.rs',
     '.is_some_and(|p| !p.severed && capacity(p, rules) > 0)',
     '.is_some_and(|p| !p.severed)',
     ['--lib', 'systems::nerves_tests::the_joint_share_is_the_contracting_cells']),
    ('systems join without a nervous system (661)',
     'src/systems/nerves.rs',
     'if union(e, SENSE, Role::Source).is_none() {',
     'if false && union(e, SENSE, Role::Source).is_none() {',
     ['--lib', 'systems::nerves_tests::without_nerves_or_a_sense_nothing_joins']),
    ('the joint share ignores the degree (664)',
     'src/meaning/carriage.rs',
     'let share = u128::from(ask) * u128::from(reached) / u128::from(total.max(1));',
     'let share = u128::from(ask) * u128::from(total) / u128::from(total.max(1));',
     ['--test', 'systems', 'the_limbs_share_needs_nerves_joining_them_to_intake']),
    ('the nervous system carries the limbs\' share again (657)',
     'src/rules/systems.rs',
     '("nervous", system(&["sense"], &[], &["gate"], &[])),',
     '("nervous", system(&["sense"], &[], &["gate"], &["contract"])),',
     ['--test', 'systems', 'the_limbs_share_of_a_bite_needs_muscular_routes']),
    ('a carriage landing nothing caps its ask at its cells (the bug found)',
     'src/meaning/carriage.rs',
     'let by_cells = cells(p).saturating_mul(ask.max(1));',
     'let by_cells = cells(p);',
     ['--test', 'probe_bodies', 'a_grazers_meal']),
    ('a system naming no effects is never realized',
     'src/systems/mod.rs',
     'filled(&s.sources) && (s.effects.is_empty() || filled(&s.effects))',
     'filled(&s.sources) && filled(&s.effects)',
     ['--lib', 'systems::nerves_tests::a_grazer_with_an_eye_joins']),
    ('a part carries whatever its cells (560, 564)',
     'src/systems/carry.rs',
     'let room = capacity(&e.parts[id], rules);',
     'let room = OPEN;',
     ['--lib', 'systems::tests::parts_share_the_trunk_that_carries_to_them']),
    ('a development brings a system its body already realized (587)',
     'src/systems/vary.rs',
     'if !e.systems.contains_key(k) && realizes(e, s) && !realizes(before, s) {',
     'if !e.systems.contains_key(k) && realizes(e, s) {',
     ['--lib', 'systems::vary_tests::a_developed_gland_brings_its_system']),
    ('cells vary without their odds (578)',
     'src/systems/vary.rs',
     'if !met(recipe.vary, draw("vary")) {',
     'if false && !met(recipe.vary, draw("vary")) {',
     ['--lib', 'systems::vary_tests::with_no_odds_nothing_varies_or_riffs']),
    ('systems riff without their odds (573)',
     'src/systems/vary.rs',
     'if !met(recipe.riff, draw("riff")) {',
     'if false && !met(recipe.riff, draw("riff")) {',
     ['--lib', 'systems::vary_tests::with_no_odds_nothing_varies_or_riffs']),
    ('a riff is kept though the child no longer realizes it (491)',
     'src/systems/vary.rs',
     'if !realizes(child, &riffed) {',
     'if false && !realizes(child, &riffed) {',
     ['--lib', 'systems::vary_tests::a_riff_on_a_dormant_system_is_kept_only_where_it_wakes']),
]


def run(i):
    name, path, before, after, test = FAULTS[i]
    src = open(path, encoding='utf-8', newline='').read()
    assert src.count(before) == 1, f'{name}: anchor not unique in {path}'
    saved = path + '.fault-saved'
    shutil.copyfile(path, saved)
    try:
        open(path, 'w', encoding='utf-8', newline='').write(src.replace(before, after))
        r = subprocess.run(['cargo', 'test', '--offline', *test], capture_output=True, text=True)
        caught = r.returncode != 0 and ('FAILED' in r.stdout or 'panicked' in r.stderr + r.stdout)
        compiled = 'error[' not in r.stderr
        verdict = 'CAUGHT' if caught and compiled else ('NO-COMPILE' if not compiled else 'MISSED')
        print(f'{i:2} {verdict:10} {name} :: {" ".join(test)}', flush=True)
        return verdict
    finally:
        shutil.copyfile(saved, path)
        os.remove(saved)
        os.utime(path, None)


if __name__ == '__main__':
    picks = [int(a) for a in sys.argv[1:]] or range(len(FAULTS))
    verdicts = [run(i) for i in picks]
    print(f'{verdicts.count("CAUGHT")} of {len(verdicts)} caught')
