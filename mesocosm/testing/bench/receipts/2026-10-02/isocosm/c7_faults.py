"""Checkpoint 7's planted faults: each a one-line edit the named test must
catch. Applies one fault, runs its test, restores the file from a saved
copy with a fresh time, and prints the verdict. Run from shared/isocosm;
an argument picks faults by index."""
import os, shutil, subprocess, sys

FAULTS = [
    ('take spread evenly, not by holdings', 'src/anatomy.rs',
     '    let split = apportion(&weights, amount);\n    Some(apply(e, key, &split, debit)',
     '    let even: Vec<(Id, u64)> = weights.iter().map(|w| (w.0, 1)).collect();\n    let split = apportion(&even, amount);\n    Some(apply(e, key, &split, debit)',
     ['--test', 'anatomy', 'a_take_spreads']),
    ('the reserve bounded by the whole part, not its stores', 'src/anatomy.rs',
     '        cells.saturating_mul(cell_mass(p, b))',
     '        let _ = cells;\n        ceiling(p, b)',
     ['--test', 'anatomy', 'the_reserve_lives_only']),
    ('a bite drawn evenly, not by holdings', 'src/anatomy.rs',
     '            (id, edible)',
     '            (id, u64::from(edible > 0))',
     ['--test', 'anatomy', 'a_bite_lands']),
    ('a rod with one child read as a branch', 'src/anatomy.rs',
     '"part-shape:rod" if children >= 2',
     '"part-shape:rod" if children >= 1',
     ['--test', 'anatomy', 'a_part_is_named']),
    ('flow legs left whole, not split among parts', 'src/flows.rs',
     '    if routed.is_empty() {',
     '    if !routed.is_empty() || routed.is_empty() {',
     ['--test', 'flows', 'the_record_names_the_parts']),
    ('a body\'s books leave out its parts', 'src/anatomy.rs',
     '    for p in e.parts.values() {\n        for (k, v) in &p.matter {',
     '    for p in e.parts.values().filter(|_| false) {\n        for (k, v) in &p.matter {',
     ['--test', 'probe_bodies', 'bodies_run_alike']),
    ('fixing at 5 mg per 63 faces', 'src/probe/found/body/physiology.rs',
     'FIXES_PER_FACE: (i64, i64) = (4, 63)',
     'FIXES_PER_FACE: (i64, i64) = (5, 63)',
     ['--lib', 'the_median_body_earns']),
    ('grazing at 12 mg per 294 voxels', 'src/probe/found/body/physiology.rs',
     'GRAZES_PER_VOXEL: (i64, i64) = (11, 294)',
     'GRAZES_PER_VOXEL: (i64, i64) = (12, 294)',
     ['--lib', 'the_median_body_earns']),
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
        why = next((l.strip() for l in out.splitlines() if 'panicked' in l or 'assert' in l), '')
        nxt = ''
        lines = out.splitlines()
        for j, l in enumerate(lines):
            if 'panicked' in l and j + 1 < len(lines):
                nxt = lines[j + 1].strip()[:160]
                break
        caught = 'FAILED' in line or r.returncode != 0
        print(f'[{i}] {name}: {"CAUGHT" if caught else "MISSED"} | {line.strip()} | {nxt}')
    finally:
        shutil.move(path + '.kept', path)
        # A restored file keeps the copy's older time, and cargo, which
        # fingerprints by time, would keep the faulted build.
        os.utime(path)
