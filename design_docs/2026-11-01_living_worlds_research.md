# Living worlds and non-neural sensing: an open-source survey

**Status:** 2026-11-01: research brief, from case 1071's queued brief. 2026-11-02: its six questions (§C) and both licence checks are ruled by case 1079 (2026-11-01); the survey itself is not a ruling.
**For:** how openly licensed descriptions of living worlds and of sensing without nerves bear on the sim (Bodies 10 and 15, World 4, Processes 15 and 19) and on the three games.

Mark, case 1071: "we could draw upon like ways that people have designed worlds, existing open source descriptions of worlds that have a living aspect to them … maybe will lend us insight into actual, like, individuals, like critters and creatures and other types of entities that also need means of figuring out what's going on."

**Method:** sources are cited as links, never copied in (doctrine Licensing 1: an open text informing the sim is a reference). Each source's licence is named, and anything under a non-commercial licence or the OGL is left out. Summaries are original.

## A. Living worlds

| Source | Licence | What it offers |
| --- | --- | --- |
| [Fate Core SRD, the Bronze Rule](https://fate-srd.com/fate-system-toolkit/bronze-rule) | CC BY 3.0 | Anything can be written as a character, a storm or the scene itself included, with its own aspects, skills and stress. |
| [SRD 5.1, legendary creatures: lair actions and regional effects](https://5thsrd.org/gamemaster_rules/legendary_creatures/) | CC BY 4.0 | A place acts on its own initiative count, and a powerful being's home warps the land around it (the kraken's weather within six miles). The effects fade when the being dies. |
| Ars Magica Definitive core, chapter 12: realm auras, regiones, vis sources ([OriginalMadman/Ars-Magica-Open-License](https://github.com/OriginalMadman/Ars-Magica-Open-License)) | CC BY-SA 4.0 | A place carries a scored aura of a realm. Strong auras warp those who live in them, regiones layer a place into more intense levels, and vis collects at sites. |
| Ironsworn Delve moves reference ([DOKK copy](https://dokk.org/library/ironsworn_starforged_delve_moves_reference_2020)) | CC BY 4.0 for the SRD and asset texts only, checked (case 1079); the rulebooks, including the core Ironsworn book (CC BY-NC-SA), are non-commercial and so excluded | A threat has a menace track that advances on its own, and its last box means the threat reaches its goal. |
| [Daisyworld](https://en.wikipedia.org/wiki/Daisyworld) and [Gaia hypothesis](https://en.wikipedia.org/wiki/Gaia_hypothesis) (Wikipedia) | CC BY-SA 4.0 | A planet whose life regulates its temperature through albedo feedback, with no agent intending it. |
| [*This Dynamic Earth*](https://pubs.usgs.gov/gip/dynamic/dynamic.html) and the [*This Dynamic Planet* map](https://pubs.usgs.gov/imap/2800) (USGS) | US federal work, public domain, checked (case 1079): credited, and USGS's third-party images excluded | Plates, rifts, subduction, hotspots, and where volcanoes and earthquakes sit relative to them. |
| [NOAA tides and water levels tutorial](https://oceanservice.noaa.gov/education/tutorial_tides/) | US federal work, public domain | Tides as forcing by the moon and sun, shaped by the basin; spring and neap cycles. |
| [WorldEngine](https://github.com/Mindwerks/worldengine) / [plate-tectonics](https://github.com/Mindwerks/plate-tectonics) | MIT / LGPL-3.0 | A world generator that runs plate motion, erosion, rainfall and biomes as a process over time. |
| [Cataclysm: Dark Days Ahead](https://github.com/CleverRaven/Cataclysm-DDA) | CC BY-SA 3.0 (its README) | A fungal force (the Mycus) that spreads across the map and converts what it touches: a world-scale actor made of terrain and creatures. |

**What they suggest for the sim:**
- **The world as a sensing and acting entity (World 4, Bodies 15).** Daisyworld is the cleanest model. The world's "sense" is a field reading (temperature over the surface), and its "act" falls out of its members' processes, not a will. That matches World 4's inert and sentient positions, with geology as anatomy. The sense Bodies 15 says is owed can be read as aggregate fields (heat, strain, water level). The world's agentless processes then become feedback on those fields.
- **Tectonics and volcanism as metabolism (World 4, Processes 15).** USGS's plate picture maps onto native terrain operations: matter moves by declared processes (uplift, eruption, deposition) and is conserved. An eruption is a source of matter only if it is declared as one; otherwise it moves matter from below.
- **Tides as periodic forcing** (Processes 7's periods and `light.rs`'s sky). The world already has a sun on a year; a moon and a tide are another periodic field.
- **Places with a charge (Ars auras, SRD regional effects):**
  - These line up with World 9 (places carry quality and alignment) and with Processes 19's reach field. A powerful being's influence would be a field with a source and a decay, which the reach field already stores.
  - Regional effects ending when the being dies is a field without a source decaying, which the reach field already does.

**What they suggest for the games:**
- **The VTT:** lair actions and regional effects are a ready vocabulary for a place acting at the table. An encounter mode could give the world entity a turn (initiative 20) when the edition declares it.
- **Ars:** auras and warping are already in its pack's future scope.
- **Eponym and Mesocosm:**
  - The Fate fractal and Ironsworn's menace track are ways to show a world entity's agency to players: a storm or a blight with a few legible tracks, rather than an opaque process list.
  - Cataclysm's fungal spread shows the cost: a world actor that converts the map needs a counter-play, or it reads as a timer.

## B. Sensing without nerves

| Source | Licence | What it offers |
| --- | --- | --- |
| Baluška and Souza, "[Plant Signaling, Behavior and Communication](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11055140/)", *Plants* 2024 | CC BY 4.0 (MDPI) | Plants sense light, gravity, touch, chemicals and damage, and signal electrically and chemically across the whole body, with no neurons. |
| Canales et al., "[Electrical signaling in roots](https://www.frontiersin.org/articles/10.3389/fpls.2017.02173/text)", *Frontiers in Plant Science* 2018 | CC BY 4.0 (Frontiers) | A root's nutrient sensing becomes electrical signals propagated to the rest of the plant; the sensors' exact identity is still open. |
| Adamatzky, "[Language of fungi derived from their electrical spiking activity](https://doi.org/10.1098/rsos.211926)", *Royal Society Open Science* 2022 | CC BY 4.0 | Electrical spiking along mycelium, with species-specific patterns. The "language" reading is the author's interpretation, not established meaning. |
| [Physarum polycephalum](https://en.wikipedia.org/wiki/Physarum_polycephalum) (Wikipedia) | CC BY-SA 4.0 | A single-celled slime mould that finds short paths between food sources by thickening the tubes that carry most flow and starving the rest. |
| [Chemotaxis](https://en.wikipedia.org/wiki/Chemotaxis) and [Quorum sensing](https://en.wikipedia.org/wiki/Quorum_sensing) (Wikipedia) | CC BY-SA 4.0 | A bacterium climbs a gradient by comparing concentration over time (run and tumble). A population switches behaviour when a signal it secretes passes a threshold. |
| [Tropism](https://en.wikipedia.org/wiki/Tropism) (Wikipedia) | CC BY-SA 4.0 | Growth directed by a stimulus: light, gravity, touch, water. |
| [Stigmergy](https://en.wikipedia.org/wiki/Stigmergy) (Wikipedia) | CC BY-SA 4.0 | Coordination through marks left in the environment, such as pheromone trails, with no direct messages. |
| [Lenia](https://github.com/Chakazul/Lenia) ([paper](https://arxiv.org/abs/1812.05433)) | MIT (code) | Continuous cellular automata whose "creatures" persist and move only by convolving a field around them: life as pure field sensing. |

**What they suggest for the sim:**
- **The baseline sense by form (Bodies 15):**
  - Plants: tropisms are local readings that steer growth. Case 1071 already gives plants a light-share and soil-share sense that gates growth, so the next step is direction: growth toward the higher reading.
  - Microbes: chemotaxis is a temporal gradient, comparing a reading now with one a moment ago. The sim's sense readings are per tick, so this means keeping one previous reading per member.
  - Fungi: sensing along the mycelium is a network property.
- **Network sensing and transport (Bodies 10, Processes 15):** Physarum's rule (strengthen what carries flow, let the rest wither) is Bodies 10's "a narrowed trunk shorts what lies beyond", run in reverse as growth. A fungal or plant system could adapt its conducting cells by use, with matter conserved along the routes. This is the closest match in the survey to an existing doctrine mechanism.
- **Collective and field senses (Processes 19, Processes 6):**
  - Quorum sensing is a threshold on a field the population itself writes: the spread crowd's gradient that Bodies 15 lists as owed.
  - Stigmergy is the same idea with marks that persist. Processes 19's reach field (arrivals, strength, decay) can hold such a field without per-member state.
- **Electrical signalling (plants, fungi):** a slow, whole-body signal. Within one body it fits Bodies 10's nervous-system role ("joins systems … where a sense reaches both") without needing nerves. It suggests a non-neural joining system with a lower conduct rate.

**What they suggest for the games:**
- **Mesocosm** can show sensing as behaviour players learn to read: a plant leaning, a mould rerouting, a microbe bloom switching at a threshold.
- **Eponym's** critters gain legible "what's going on" senses that aren't eyes, which Mark's line asks for.
- **The VTT** gains druidic and fungal flavour with a mechanical footing: speaking with plants could read what the plant actually sensed.

## C. What needed a ruling (ruled by case 1079, 2026-11-01)

1. **Whether the world entity's acts are emergent only, or can be directed.** Daisyworld-style feedback (no will) fits World 4's inert and sentient positions. A storyteller with goals, which World 4 also allows, needs a ruling on how its directed acts enter: as processes, or as hooks (VTT 4).
   *Ruled, 1079:* a world setting with three options, "all 3 ought to be options": acts only emergent from its processes and fields; emergent and directable by a GM or the world's disposition through the intents any entity uses; or a world mind with goals that chooses like a sophont.
2. **A temporal-gradient sense:** whether a member may keep one prior reading per sense to compare (chemotaxis), which is new per-member state.
   *Ruled, 1079:* "Yes, one prior reading per sense".
3. **Use-adapting networks:** whether a system's conducting cells may grow or wither by flow (Physarum), a development process touching Bodies 6 and 10.
   *Ruled, 1079:* "Yes, use shapes growth": growth adds capacity where flow runs and idle routes shrink.
4. **A non-neural joining system:** whether Bodies 10's joining role gets a slower electrical or chemical variant for plants and fungi, or stays the nervous system's alone.
   *Ruled, 1079:* "Yes, slow signalling by form", by degree and with a lag.
5. **Field sources and decay:** whether influence fields (auras, regional effects, quorum signals, stigmergic marks) share Processes 19's reach field or get their own, since the reach field was ruled for knowledge.
   *Ruled, 1079:* "Share the machinery, separate kinds": influence fields use the reach field's machinery as a separate kind that never mixes with knowledge.
6. **Sources:** Ironsworn's Delve licence and USGS's reuse statement should be confirmed before either is cited in a shipped pack.
   *Ruled, 1079:* both checked: only Ironsworn's SRD and asset texts are CC BY 4.0 (its rulebooks are non-commercial and out), and USGS-authored material is US public domain, credited, its third-party images excluded.
