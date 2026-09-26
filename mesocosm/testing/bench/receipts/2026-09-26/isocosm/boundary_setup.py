"""Copy the worktree's shared crates into a scratch tree and instrument the
copy to count where prey run out part way through a pass of hunters.

The copy's core logs every weighted target search (tick, the actor's place,
the process, whether a target was found, and the actor's body); the copy's
crowd logs a boundary instead of refusing it, feeding the first hunters in
bin order so the run goes on. Nothing here touches the worktree."""
import os, shutil

worktree = r"C:\Users\mark_\Code\repos\isometry\.claude\worktrees\agent-a89e04fcecf797735\shared"
scratch = r"C:\Users\mark_\AppData\Local\Temp\claude\C--Users-mark--Code\d672de3a-b76c-4707-be20-7489297a1f5b\scratchpad\boundary-core\shared"
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
/// Measurement only: weighted target searches, and crowd boundaries.
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

edit(os.path.join("probe", "crowd", "hunt.rs"), """        if capacity < u128::from(demand) {
            return Err(format!(""", """        if capacity < u128::from(demand) {
            let states = hunters.len();
            crate::CROWD_BOUNDARIES.lock().unwrap().push((
                self.tick,
                site,
                demand,
                capacity as u64,
                states,
            ));
            let mut left = capacity as u64;
            let mut fed_hunters = Vec::new();
            for (e, n) in hunters {
                let fed = n.min(left);
                left -= fed;
                if fed > 0 {
                    fed_hunters.push((e.clone(), fed));
                }
            }
            return self.hunt_fed(m, site, fed_hunters, &mut prey);
        }
        if false {
            return Err(format!(""")

edit(os.path.join("probe", "crowd", "hunt.rs"), """        let domain = format!("probe-hunt:{}", p.id);""", """        self.hunt_fed(m, site, hunters, &mut prey)?;
        let _ = p;
        Ok(())
    }

    fn hunt_fed(
        &mut self,
        m: &Meal,
        site: Id,
        hunters: Vec<(Entity, u64)>,
        prey: &mut BTreeMap<Entity, u64>,
    ) -> Result<()> {
        let rules = &self.world.genesis.rules;
        let demand: u64 = hunters.iter().map(|h| h.1).sum();
        let domain = format!("probe-hunt:{}", "measure");""")
print("ready")
