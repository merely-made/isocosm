# Planet viewer

Spin a spherical Mesocosm founding, click a site or choose an inhabited valley, then follow a grazer. Following starts the clock. Globe, Valley and Critter keep the same tick and individual. The gold trail follows the last 24 ticks of recorded site changes. Drag or use the arrow keys to rotate. Use the scale buttons, scroll wheel, Enter or Escape to move between views.

Build the Cosmograph example from the shared crate. In the local Code workspace use the guarded Cargo launcher as described in its AGENTS.md. For example, pass `build --release --locked --example planet_atlas` to the launcher. From the repository root, with the resulting binary as `$PLANET_ATLAS` and a new output directory as `$PLANET_SAVE`, run

```sh
"$PLANET_ATLAS" found mesocosm/tools/planet/founding.toml "$PLANET_SAVE"
python3 mesocosm/tools/planet/serve.py "$PLANET_SAVE" "$PLANET_ATLAS"
```

Open the printed loopback address. The map stores six connected square faces. Relief samples, neighbors, body geometry, feeding and tissue all come from the native session. The valley list selects occupied sites with higher ground on two opposing sides. Positions in this founding resolve to a site or a journey. Individual view magnifies the body, with stored tissue tinting the parts.

The founding command writes the native founding save, terrain, 25 observed frames and a tick-24 resume save. The server verifies the resumed state against the preview and advances the same simulation on demand. Later frames stay available for scrubbing during that server run. Restarting serves the founding preview again.

## Watch a spatial branch

Build the Mesocosm workspace binary with `build --release --locked -p mesocosm-runtime --bin planet_ground`. For the seed 9 atlas, start from the saved founding and open a second viewer:

```sh
"$PLANET_GROUND" branch "$PLANET_SAVE/founding.json" 462 "$GROUND_SAVE" 24
python3 mesocosm/tools/planet/serve.py "$GROUND_SAVE" "$PLANET_GROUND"
```

Use a new directory for `GROUND_SAVE`. The branch inherits the planet and founding organisms. It opens paused with a native observation window over the chosen resident's site. Play follows its accepted walking stances on the terrain. The inspector records food actually transferred and identifies its source. Select Globe to see those same stances plotted at planet scale and magnified in the route inset. Scrub or return to Critter to inspect the same moment. When the followed body chooses to migrate, its observed departure can include supported strides toward the chosen border. The native session admits its last stance before resolving the biological round. The globe retains the walked part. The rest of a site-scale crossing has no ground trail. Walking playback spreads short approaches over the viewing interval so you can see the steps. A link can open any preview tick with `?tick=6`, or the corresponding globe with `?tick=6&view=globe`.

The branch saves the native session and Mesocosm spatial state together. Serve the same directory again to replay the recorded preview and continue from its checkpoint.

## Visit sites on a field branch

To survey an ecosystem, start a sibling of the same planet founding. The following opens the field at site 1. Use an empty directory for `FIELD_SAVE`.

```sh
"$PLANET_GROUND" field "$PLANET_SAVE/founding.json" 1 "$FIELD_SAVE"
python3 mesocosm/tools/planet/serve.py "$GROUND_SAVE" "$PLANET_GROUND" --field "$FIELD_SAVE"
```

Choose **Field** in the atlas or open `/field` at the printed address. **Globe** shows inhabited sites at the current tick. Click the globe, visit a neighboring site, or use the site list. The field clock stays at the same tick while you explore. **Play** or **Next tick** replays recorded observations up to **Latest**. Beyond that point the next native round examines the chosen site. Foreground walking and contact use that attention window, so where you observe a new round can affect its outcome.

The ground view projects the living parts of every positioned body onto the same horizontal plane. The ruler uses the founding's length unit. A fine grid appears when a single unit is large enough to see. The resident list gives each body's greatest rest span and draws the silhouettes at a shared scale. Residents known only to the site stay in the census. A body without measured anatomy reads **Size unresolved**. Selecting a resident filters the material transfers for this tick.

The server writes the latest native checkpoint and each observed frame together to `FIELD_SAVE/field.sqlite3`. Restarting against that directory resumes the field branch. An address such as `/field?site=1&tick=3` returns to a recorded moment.

## Watch a fungus reach litter

At site 1 of the seed 9 planet, the two resident fungi can start at living plant collars. Each grows a connected body from the [soil body map](soil-body.toml). The collar pays fungal tissue for each extension. A living part touching a litter face can recover phosphate from the deposit. A connected plant can pay one unit of its carbon-bearing tissue for one unit of phosphate. Plant production at this site requires phosphate alongside the founding's light, soil and exposed fixing tissue.

Create two empty branch directories from the same founding, then run the paired check and serve either directory.

```sh
"$PLANET_GROUND" soil "$PLANET_SAVE/founding.json" 1 near "$NEAR_SAVE"
"$PLANET_GROUND" soil "$PLANET_SAVE/founding.json" 1 far "$FAR_SAVE"
python3 mesocosm/tools/planet/check_soil.py "$PLANET_GROUND" "$NEAR_SAVE" "$FAR_SAVE"
python3 mesocosm/tools/planet/serve.py "$GROUND_SAVE" "$PLANET_GROUND" --field "$NEAR_SAVE"
```

Open `/field?site=1&tick=0` and press **Play**. **Shallow soil** shows the litter, plant collars and living fungal parts from overhead. The vertical section cuts through the selected colony at the same spatial scale. The reading distinguishes phosphate held by the fungus, available to the plant and incorporated by the plant. **Surface** returns to grazing at the same tick. The far branch shifts the same deposit 40 mm sideways. New growth follows the body's inherited directions.

## Follow a plant family over a year

The seed 9 planet runs at one minute per tick. To keep the original branches and try a seasonal comparison, create a new parent in an empty directory. Its 52 weekly rounds span a model year. The two soil branches below inherit the same bodies, stores and terrain from this parent. Their litter is 40 mm apart.

```sh
mkdir -p "$YEAR_SAVE"
"$PLANET_GROUND" seasonal "$PLANET_SAVE/founding.json" "$YEAR_SAVE/parent.json"
"$PLANET_GROUND" soil "$YEAR_SAVE/parent.json" 1 near "$YEAR_SAVE/near"
"$PLANET_GROUND" soil "$YEAR_SAVE/parent.json" 1 far "$YEAR_SAVE/far"
for distance in near far; do
  python3 mesocosm/tools/planet/recruitment.py "$PLANET_GROUND" "$YEAR_SAVE/$distance" \
    --ancestor 174 --site 1 --through 52 > "$YEAR_SAVE/$distance.json"
done
```

A1 organisms keep their feeding, growth, birth and aging schedules in rounds. Each new producer starts as a dormant seed with eight units of provision. After each 13 weeks of its own seed life it has a one-in-four chance to wake if its site is habitable and has room. The planet's latitude and orbital phase also drive temperature. Bodies outside their comfort band spend tissue. Weather variation, drought and shade costs are disabled for this comparison. Weekly rounds leave the surface walk at its existing per-round stride budget. The trial measures seasonal recruitment with stored site energy and does not integrate a week of daylight or travel within each round.

The field clock shows the season in the visited hemisphere. The roster labels dormant seeds. To inspect a recorded birth or bite, open `/field?site=141&tick=23&organism=702` on the relevant branch. The selected body filters the transfer list.

Omit `--through` to measure an existing field database at its saved head. The report reopens the native save, checks the recorded state hash, walks parent–child relations, and matches children to birth provisions. A daughter counts as established once observed alive and awake at least one reproductive interval after birth. A daughter survives a recorded bite when she is observed alive at a later tick. Both measures accompany her actual offspring and final state, including travel between sites.
