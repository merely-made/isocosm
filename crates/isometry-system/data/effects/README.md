# Executable effect examples

Nine experimental VTT procedures, selected under functional-generation D1:

| Ruleset | Senses | Body size | Lasting weapon change |
| --- | --- | --- | --- |
| D&D SRD 5.1 (2014 rules) | Darkvision | Enlarge/Reduce | Magic Weapon |
| Pathfinder 2e Player Core, Remaster | Darkvision | Enlarge | Runic Weapon |
| Ars Magica Definitive Fifth Edition | Eyes of the Cat | Preternatural Growth and Shrinking | Edge of the Razor |

Run `cargo run -p isometry-system --example cast_effects` from the repository
root. It executes all nine on an authored board, serializes/restores the map,
suppresses/resumes each change, and advances through its actual expiry. The
JSON reports costs, dice, accepted definitions, body dimensions, senses and
weapon changes. This is a headless demonstration, not a Mesocosm magic
playtest or a complete port of these editions.

The desktop's **Character sheets** tab now exposes one free-play loop:
D&D SRD 5.1 Magic Weapon on the caster's own wielded longsword or club.
The DM creates a sheet for a native body, grants the spell and a slot, adds
table equipment and wields it. The controlling player or DM can then cast.
The running effect, enhancement bonuses and actual attack/concentration dice
appear on the same page. Sheet equipment is explicitly unmarked table data;
this does not manufacture, carry or enchant a native physical item.

The current `isocosm-vtt` endpoint accepts these changes through the family
host. Damage from its weapon attacks or its explicit external-damage buttons
uses the same concentration procedure. Accepted native downtime expires the
effect through the existing player-consent path. Checkpoints and accepted
family records carry the resulting edition state and dice stream. Replica
replay is verified; live peer transport remains a separate session gate.

Attacks use current native resident poses or admitted VTT positions when the
DM declares native cells per five-foot square in **Native operation
authoring**. This first adapter measures root distance and a terrain ray,
not articulated weapon reach or whole-body space. Without that calibration,
only the DM can choose **Rule within reach and attack**, recording the ruling.
No position is guessed from the rules input's temporary map coordinates.

`active_effects::cast` takes the current map, existing inventories, procedure
definitions, a request, a calibrated board/clock context, the host's
line-of-effect predicate and the existing dice stream. Refusal changes neither
map nor dice. Accepted casts consume declared sheet resources and the existing
turn counters. A repeated identical request returns its previous receipt.
Every receipt is marked experimental until the edition is calibrated.

The system owns the versioned records in `MapDocument.system_data`. Accepted
definitions and credits travel with them; reopening a save does not replace a
running spell with today's pack definition. Creature targets use stable map
token IDs; weapons use campaign item IDs. Hosts must retire or migrate effects
when replacing tokens or moving maps, and must not recycle an affected ID.

At accepted clock or concentration events, call `advance` or
`concentration_damage`. `Clock::from_native` reads the simulation's tick unit
and next native sky crossing; the table can supply its accepted calendar.
Time cannot run backwards or change units. An unrepresentable duration is
refused. Daily preparation is an explicit calendar boundary.

`project_sheet` and `System::effective_map_sheet` derive changes from the
current base sheet and equipped inventory. Desktop sheet/action readers use
the latter. The existing damage resolver reads enlarge/reduce weapon dice and
runic weapon dice; the PF2e pack reads added reach, clumsy AC penalties and
melee status damage. `visible_tiles` combines observer senses and lighting
with the existing terrain height tracer. `project_body` scales the current
part graph while preserving IDs, origins and injuries. Integer coordinates
must scale exactly; shrinking may require a finer geometry unit. Caches can
bind `Records::revision`.

Expiry never restores an old sheet or inventory snapshot. Later wounds,
equipment transfers and base edits survive. Suppression names independent
sources; removing one cannot lift another or revive an expired effect.
Dispel/counteract outcomes retain their adjudication. This is a lasting-effect
lifecycle; permanent device manufacture is separate work.

Implemented profiles and remaining boundaries:

- D&D: known-spell slot casting, components, willingness or Enlarge/Reduce's
  Constitution save, bonus-action restrictions and concentration. Creature
  resizing and main-hand damage are projected. Confined-space fitting,
  resized equipment meshes, object targets and automatic Strength advantage
  rolls remain unwired. Critical attacks now repeat damage dice and add flat
  modifiers, including Magic Weapon, once. This is not a complete
  Enlarge/Reduce port.
- PF2e: prepared casts and the existing three-action budget. Darkvision's
  personal/touch targeting and daily-preparation heightening are distinct.
  Enlarge supports single-target ranks; rank-six group casting needs one group
  transaction. Restricted manipulation and overlapping polymorph need recorded
  adjudication. Runic Weapon does not lower stronger equipment bonuses.
- Ars Magica: calm, unmastered formulaic casting, Animal requisite for cat
  eyes, casting totals, fatigue and Sun duration. Stress/botch procedures, full
  Magic Resistance adjudication and laboratory enchantment are pending. Edge
  of the Razor reports penetration and that the changed weapon is resisted.
  Ars Size remains its own quantity; geometry needs an authored scale.

Desktop casting for the other eight procedures, concentration integration
with other damage sources, initiative/action economy, scene consumers for
body/lighting projections, live multiplayer admission and cross-map migration
remain integration work. The API does not install a second native simulator
or authorize peer-side adjudication.

Sources and licences:

- [D&D SRD 5.1](https://media.wizards.com/2023/downloads/dnd/SRD_CC_v5.1.pdf).
  Based on the System Reference Document 5.1 by Wizards of the Coast LLC,
  licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
  The adaptation uses original summaries and executable fields.
- Player Core Remaster mechanics in `pf2.toml`: ORC licensed, Paizo Inc.
  The [PF2e system's spell records](https://github.com/foundryvtt/pf2e/tree/master/packs/pf2e/spells/spells)
  supplied the source checks: rank-2 Darkvision and Enlarge, rank-1 Runic Weapon.
  Pathfinder is a trademark of Paizo Inc. No art or other reserved material is
  included. See the [ORC licence](https://azoralaw.com/orclicense/).
- [Atlas's open Ars Magica text](https://www.atlas-games.com/arsmagica/openars):
  reviewed Definitive core, Git blob `41860c6a10eedc246c730ca485bed6927060ced9`
  in [OriginalMadman/Ars-Magica-Open-License](https://github.com/OriginalMadman/Ars-Magica-Open-License).
  `ars5.toml` adapts these mechanics into executable fields and original
  descriptions under CC BY-SA 4.0. No illustrations or layout are included.

Based on the material for Ars Magica, ©1993–2024, licensed by Trident, Inc.
d/b/a Atlas Games®, under
[Creative Commons Attribution-ShareAlike 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

Ars Magica Open License Logo ©2024 Trident, Inc. The Ars Magica Open License
Logo, Ars Magica, and Mythic Europe are trademarks of Trident, Inc., and are
used with permission.
