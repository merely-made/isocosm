"""Copy the worktree's shared crates into a scratch tree and instrument the
copy to count where prey run out part way through a pass of hunters, and
whether those hunters shared one state.

The copy's core logs every weighted target search (tick, the actor's place,
the process, whether a target was found, and what the actor holds); a pass
at a site where some hunters found prey and some found none is a shortage.
The copy's crowd logs each shortage with the number of hunting states, and
where the real crowd would refuse (more than one state) it feeds the first
hunters in bin order instead, so the run goes on. Nothing here touches the
worktree."""
import os, shutil, sys

worktree = sys.argv[1]
scratch = sys.argv[2]
if os.path.exists(scratch):
    shutil.rmtree(scratch)
for crate in ["isocosm", "wing-impresa", "wing-glyphs"]:
    shutil.copytree(os.path.join(worktree, crate), os.path.join(scratch, crate),
                    ignore=shutil.ignore_patterns("target"))
src = os.path.join(scratch, "isocosm", "src")


def edit(rel, old, new):
    p = os.path.join(src, rel)
    s = open(p, encoding="utf-8").read()
    assert s.count(old) == 1, (rel, old)
    s = s.replace(old, new)
    open(p, "w", encoding="utf-8", newline="\n").write(s)


edit("lib.rs", "mod targets;", """mod targets;
/// Measurement only: weighted target searches, and crowd shortages.
pub static SEARCHES: std::sync::Mutex<Vec<(u64, u64, String, bool, u64)>> =
    std::sync::Mutex::new(Vec::new());
pub static CROWD_BOUNDARIES: std::sync::Mutex<Vec<(u64, u64, u64, u64, usize)>> =
    std::sync::Mutex::new(Vec::new());""")

edit("targets.rs", """                debug_assert_eq!(found, everywhere(), "the filed search missed a group");
                found""", """                debug_assert_eq!(found, everywhere(), "the filed search missed a group");
                if selector.weighted {
                    let body = entity.accounts.values().sum::<u64>();
                    crate::SEARCHES.lock().unwrap().push((
                        self.state.tick,
                        place,
                        p.id.clone(),
                        found.is_some(),
                        body,
                    ));
                }
                found""")

edit(os.path.join("probe", "crowd", "hunt.rs"), """        if 0 < meals && meals < demand && hunters.len() > 1 {
            return Err(format!(""", """        if 0 < meals && meals < demand {
            crate::CROWD_BOUNDARIES.lock().unwrap().push((
                self.tick,
                site,
                demand,
                meals,
                hunters.len(),
            ));
        }
        if false {
            return Err(format!(""")
print("ready")
