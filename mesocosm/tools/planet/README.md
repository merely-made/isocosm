# Planet viewer

Spin a spherical Mesocosm founding, click a site or choose an inhabited valley, then follow a grazer. Following starts the clock. Globe, Valley and Critter keep the same tick and individual. The gold trail follows the last 24 ticks of recorded site changes. Drag or use the arrow keys to rotate. Use the scale buttons, scroll wheel, Enter or Escape to move between views.

Build the Cosmograph example from the shared crate. In the local Code workspace use the guarded Cargo launcher as described in its AGENTS.md. For example, pass `build --release --locked --example planet_atlas` to the launcher. From the repository root, with the resulting binary as `$PLANET_ATLAS` and a new output directory as `$PLANET_SAVE`, run

```sh
"$PLANET_ATLAS" found mesocosm/tools/planet/founding.toml "$PLANET_SAVE"
python3 mesocosm/tools/planet/serve.py "$PLANET_SAVE" "$PLANET_ATLAS"
```

Open the printed loopback address. The map stores six connected square faces. Relief samples, neighbors, body geometry, feeding and tissue all come from the native session. The valley list selects occupied sites with higher ground on two opposing sides. Positions in this founding resolve to a site or a journey. Individual view magnifies the body, with stored tissue tinting the parts.

The founding command writes the native founding save, terrain, 25 observed frames and a tick-24 resume save. The server verifies the resumed state against the preview and advances the same simulation on demand. Later frames stay available for scrubbing during that server run. Restarting serves the founding preview again.
