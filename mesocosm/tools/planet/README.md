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
