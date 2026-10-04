"""Checkpoint 8's step 8h, planted faults: each a one-line edit the named
test must catch. Applies one fault, runs its test, restores the file from
a saved copy with a fresh time, and prints the verdict. Run from
shared/isocosm; an argument picks faults by index."""
import os, shutil, subprocess, sys

PHYS = 'src/probe/found/body/physiology.rs'
FAULTS = [
    ('starvation read as non-zero, rent minus funds (the bug found)', PHYS,
     'Query::Computed(at_least(upkeep(i), add(vec![funds, c(1)]))),',
     'Query::Computed(add(vec![upkeep(i), mul(vec![c(-1), funds])])),',
     ['--test', 'probe_bodies', 'a_body_starves_only']),
    ('hunger without 551\'s store cap', PHYS,
     'let budget = least(vec![horizon, cells_mass_of(STORE, who)]);',
     'let budget = least(vec![horizon]);',
     ['--test', 'probe_bodies', 'a_grazers_meal']),
    ('a bud severing at its frond\'s ceiling, not a provision\'s worth (552)',
     'src/stage/staged/births.rs',
     'let full = anatomy::ceiling(part, rules.body()).min(worth);',
     'let full = anatomy::ceiling(part, rules.body()).max(worth);',
     ['--test', 'births', 'a_bud_fills']),
    ('the control keeps a reproduce cell (513)', 'src/probe/found/body/plans.rs',
     'let sown = reproduce.min(fixing - 1);',
     'let sown = reproduce.max(1).min(fixing - 1);',
     ['--test', 'probe_recipes', 'the_control']),
    ('a pair of limbs not mirrored', 'src/probe/found/body/plans.rs',
     'bilateral: limbs == 2,',
     'bilateral: false,',
     ['--test', 'probe_recipes', 'the_control']),
    ('fixing at 12 mg per 144 faces', PHYS,
     'FIXES_PER_FACE: (i64, i64) = (11, 144)',
     'FIXES_PER_FACE: (i64, i64) = (12, 144)',
     ['--lib', 'the_median_body_earns']),
    ('grazing at 13 mg per 269 voxels', PHYS,
     'GRAZES_PER_VOXEL: (i64, i64) = (12, 269)',
     'GRAZES_PER_VOXEL: (i64, i64) = (13, 269)',
     ['--lib', 'the_median_body_earns']),
    ('a hungry parent nurses', 'src/probe/found/body/life.rs',
     'let fed = less(c(1), hungry(i));',
     'let fed = hungry(i);',
     ['--test', 'births', 'a_fed_parent_nurses']),
    ('milk never flows', 'src/probe/found/body/life.rs',
     'let fed = less(c(1), hungry(i));',
     'let fed = c(0);',
     ['--test', 'births', 'a_fed_parent_nurses']),
    ('a pass takes a part whole whatever its earlier acts did to it',
     'src/stage/mod.rs',
     '                && removed\n',
     '                && (removed || true)\n',
     ['--test', 'incorporation', 'a_pass_takes_a_part_whole']),
    ('a pass lands a change on a part its earlier acts removed',
     'src/stage/mod.rs',
     'if base.parts.contains_key(id) && !live.parts.contains_key(id) {',
     'if false && base.parts.contains_key(id) && !live.parts.contains_key(id) {',
     ['--test', 'incorporation', 'a_pass_gives_nothing']),
    ('a pass puts back a part its earlier acts removed',
     'src/schedule/frame.rs',
     'if !base.contains_key(&id) {',
     'if true || !base.contains_key(&id) {',
     ['--test', 'incorporation', 'a_pass_takes_a_part_whole']),
]

only = set(int(a) for a in sys.argv[1:])
for i, (name, path, old, new, test) in enumerate(FAULTS):
    if only and i not in only:
        continue
    s = open(path, encoding='utf-8').read()
    assert s.count(old) == 1, (name, s.count(old))
    shutil.copy(path, path + '.kept')
    try:
        open(path, 'w', encoding='utf-8', newline='').write(s.replace(old, new))
        r = subprocess.run(['cargo', 'test', '-q'] + test, capture_output=True, text=True)
        out = r.stdout + r.stderr
        line = next((l for l in out.splitlines() if 'test result' in l), 'no result')
        nxt = ''
        lines = out.splitlines()
        for j, l in enumerate(lines):
            if 'panicked' in l and j + 1 < len(lines):
                nxt = lines[j + 1].strip()[:160]
                break
        caught = 'FAILED' in line or r.returncode != 0
        print(f'[{i}] {name}: {"CAUGHT" if caught else "MISSED"} | {line.strip()} | {nxt}', flush=True)
    finally:
        shutil.move(path + '.kept', path)
        # A restored file keeps the copy's older time, and cargo, which
        # fingerprints by time, would keep the faulted build.
        os.utime(path)
