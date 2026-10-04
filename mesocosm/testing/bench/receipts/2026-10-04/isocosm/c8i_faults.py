"""Checkpoint 8's step 8i, planted faults: each a one-line edit the named
test must catch. Applies one fault, runs its test, restores the file from
a saved copy with a fresh time, and prints the verdict. Run from
shared/isocosm; an argument picks faults by index."""
import os, shutil, subprocess, sys

DOING = 'src/probe/aggregate/doing.rs'
KIN = 'src/probe/crowd/kin.rs'
FAULTS = [
    ('the crowd answers no development, so its landings cannot grow', DOING,
     '            .and_then(|l| l.development.clone())\n            .ok_or_else(|| format!("{lineage}\'s bodies cannot grow here"))',
     '            .and_then(|_| None::<Development>)\n            .ok_or_else(|| format!("{lineage}\'s bodies cannot grow here"))',
     ['--lib', 'a_body_bin_moves']),
    ('the crowd never takes a part whole', DOING,
     'if *whole && self.whole(of, &portion)? {',
     'if false && *whole && self.whole(of, &portion)? {',
     ['--lib', 'a_whole_meal_lands_alike']),
    ('the crowd takes a part whole on any share', DOING,
     'if portion.values().sum::<u64>() < all {',
     'if portion.values().sum::<u64>() < all.min(1) {',
     ['--lib', 'a_whole_meal_lands_alike']),
    ('a crowd bite takes its share whatever its part held', DOING,
     'let v = v.min(offered.get(&k).copied().unwrap_or(0));',
     'let v = v.min(offered.get(&k).copied().unwrap_or(0)).max(v);',
     ['--lib', 'a_crowd_bite_takes']),
    ('unweaned young born into the counts, not kin', 'src/probe/crowd/mod.rs',
     'let bonds = Self::bonding(&birth, after);',
     'let bonds = Self::bonding(&birth, after).filter(|_| false);',
     ['--lib', 'milk_flows_alike']),
    ('kin released while their young hunger', KIN,
     '        }) && e.alive\n',
     '        }) && e.alive && false\n',
     ['--lib', 'milk_flows_alike']),
    ('a parent nurses any kin, not its own young', KIN,
     'let related = |kind: &Key| Ok(self.relations.contains(&(id, kind.clone(), **c)));',
     'let related = |kind: &Key| Ok(self.relations.iter().any(|r| r.1 == *kind) || **c == u64::MAX);',
     ['--lib', 'milk_flows_alike']),
    ('a young never known to wean', KIN,
     'self.young.insert(c, young.clone());',
     'self.young.insert(c, format!("{young}-never"));',
     ['--lib', 'milk_flows_alike']),
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
        if line == 'no result':
            nxt = next((l.strip()[:160] for l in lines if l.startswith('error')), nxt)
        caught = 'FAILED' in line or r.returncode != 0
        print(f'[{i}] {name}: {"CAUGHT" if caught else "MISSED"} | {line.strip()} | {nxt}', flush=True)
    finally:
        shutil.move(path + '.kept', path)
        # A restored file keeps the copy's older time, and cargo, which
        # fingerprints by time, would keep the faulted build.
        os.utime(path)
