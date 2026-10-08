# Research monster design, cues and fair chase loops

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: backlog, monster, research

## Delivery / research

Later phase: compare visual silhouettes, audio cues and sensory-gated
search/pursuit that uses last-known position instead of omniscient tracking.
Study routes, safe hides, escape paths, spawn fairness and how
outage/flashlight affect detection. Record measurable tuning hypotheses and a
small test arena; no AI implementation in this task.

## Research findings (2026-10-08)

Research only. No model was downloaded, imported, animated, or playtested.
Candidate-page labels and secondary analyses are leads, not independent
verification of rights or internal game AI. The older `art/gameplay/README.md`
and `art/visuals/README.md` describe proposals and historical layouts; inspect
the current game before implementing anything.

### Visual candidates and production tradeoffs

| Candidate | Source observation | Work and rights to verify before adoption |
| --- | --- | --- |
| [Quaternius Ultimate Monsters](https://quaternius.com/packs/ultimatemonsters.html) | Creator page says CC0, 50 fully animated monsters, FBX/OBJ/Blend/glTF; its texture indicator is unchecked. Already listed in `art/visuals/README.md`. | Inspect downloaded clip names, rig, mesh/texture contents and appearance in the actual facility; an untextured or stylized model needs substantial art work. Preserve creator/source provenance even if attribution is optional. |
| [Purple.Point Horror Humanoid Creature](https://sketchfab.com/3d-models/horror-humanoid-creature-b5874b20f8c34b919a52eb3cb7dad94c) | Listing says CC BY 4.0, free download, about 95.8k triangles, and describes a breathing/look-around animation. Already listed in the visual README. | Walk, chase, attack and vent clips are not documented. Confirm download contents, author rights, exact attribution, rig suitability and performance before use. A listing's license is the uploader's claim. |
| [Quaternius Universal Animation Library](https://quaternius.itch.io/universal-animation-library), [community glTF mirror](https://github.com/J-Ponzo/gltf-universal-animation-library) | Possible generic humanoid animation source; the mirror is not authoritative. | Verify creator's current terms and individual clips. Retargeting onto a different skeleton is untested here; prefer original distribution over the mirror. |
| [Adobe Mixamo](https://www.mixamo.com) | Possible humanoid rig/animation workflow. | The scout did not verify Adobe's primary license text. Check current terms, redistribution limits and export compatibility; do not treat forum summaries as permission. |

An original, restrained humanoid silhouette modeled and rigged in Blender
avoids third-party model-rights questions and allows animation timing to match
readable cues, but skeleton, skin weights and clips are substantial new work
beyond the existing modular-facility generator. A licensed rig can shorten
modeling, not necessarily animation. Prototype both at player-eye distance
under normal, outage and flashlight lighting before selecting either. Minimum
*proposed* clips: idle, emerge/vent exit, search, patrol walk, chase, attack
and a distinct nonlethal apparition pose. Validate joint paths, scale, root
motion, clip transitions, shadow cost and skinned bounds with this project's
Bevy version. Bevy issues
[#11318](https://github.com/bevyengine/bevy/issues/11318) and
[#15612](https://github.com/bevyengine/bevy/issues/15612) report animation-path
mismatch across separately imported glTF rigs; a single authored glTF
containing mesh and clips is a candidate mitigation, not a verified fix for
this project.

### Fair sensory-gated encounter hypotheses

- Preserve the existing proposed Patrol -> Investigate -> Search -> Chase ->
  Recover loop as a small baseline. Target the last *sensed* position or sound
  source, not the live player transform. Reacquire only after another sight or
  hearing check; occlude vision by walls and live doors. Communicate approach,
  search, chase and recovery with distinct footsteps, vocalizations and motion.
  [Game Developer's Alien: Isolation
  account](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)
  discusses pacing and local sensing; a full omniscient director is not
  required for this prototype. [AI and Games'
  revisit](https://www.aiandgames.com/p/revisiting-alien-isolation) is
  secondary analysis.
- Compare continuous roaming against a dormant vent phase with
  sensory-triggered emergence. If using vents, select only authored exits with
  a warning cue and a clear escape route; never silently appear inside the
  player's visible/just-cleared room, beside a hiding exit, or across the only
  route out. Keep a recovery interval after a chase. The [Amnesia: The Bunker
  analysis](https://www.aiandgames.com/p/how-the-beast-works-in-amnesia-the) is
  secondary, not proof of its source implementation.
- Test darkness as an explicit, learnable modifier to detection or attack, not
  an invisible kill switch. The scout found no primary source establishing the
  proposed exact 'attacks only in dark rooms' rule. Compare that rule against
  light merely changing detection likelihood. Ensure the flashlight can reveal
  a threat and an escape decision without making the monster harmless simply by
  aiming at it. Test blackout, flashlight-off/empty, and emergency red-light
  cases separately.
- Try a clearly distinguishable, nonlethal apparition with a unique
  silhouette/animation and audio tell, a bounded frequency and a
  player-readable resolution. It must not mask a lethal spawn or teach false
  safety. This is a project proposal, not an implemented or source-confirmed
  mechanic. A staged vent sighting can introduce the monster without a surprise
  death.
- Use the level's alternative routes and hiding places to allow a mistake and
  recovery. Avoid unavoidable single-entry boiler/storage captures and
  emergence during hide/door animations. Test with audio muted and with minimal
  lighting to check that crucial directions remain legible. Keep all ranges,
  delays and cooldowns provisional: the gameplay README's 10-15 m sight and
  10-20 s search are trial values, not established tuning.

### Small test arena and measurements

Build a T-junction, one two-exit looping room, one dead-end locker, two authored vent exits and one controllable light/blackout zone. Block out geometry and cues before investing in a final rig. Record each sensory event with source, occlusion outcome, last-known point, state transition and chosen vent. Replay identical player routes under lit/dark and walking/sprinting conditions. Observe whether new players identify where the monster came from and which escape decision could have saved them; compare escape rate, deaths at vent exits and dead ends, time since last cue, and false assumptions about apparition lethality. An 80% post-death explanation rate is a *hypothesis* for a small playtest, not a research-derived pass criterion.

Possible directions to compare: (1) continuous local patrol/search/chase gives consistent presence but costs more navigation and animation; (2) discrete, sensory-triggered vent hunts with bounded respite simplify pacing but may feel scripted; (3) a visibility-constrained mostly static pursuer reduces animation work but conflicts with the requested visible chase. Recommend prototyping (2) with local sensory search and last-known positions first, then test whether continuous patrol is worth its added complexity. This recommendation is provisional and not approval to implement.

### Research references and evidence limits

- Existing project reading: `art/gameplay/README.md` cites [Alien: Isolation GDC](https://gdcvault.com/play/1021852/Building-Fear-in-Alien), [Game Studies on fair monster behavior](https://gamestudies.org/2002/articles/jaroslav_svelch), [Outlast 2 chase orientation](https://www.gamedeveloper.com/audio/the-art-of-the-chase-level-design-and-player-orientation-in-i-outlast-2-i-), and [facility storytelling notes](https://www.worch.com/files/gdc/What_Happened_Here_Web_Notes.pdf). These inform design, not asset rights or exact tuning.
- Additional secondary or fan sources: [Resident Evil 2 design examination](https://www.gamedeveloper.com/design/a-design-examination-on-resident-evil-2-remake), [Phasmophobia hunt wiki](https://phasmophobia.fandom.com/wiki/Hunt), [SCP: Containment Breach mechanics wiki](https://containmentbreach.fandom.com/wiki/Game_Mechanics). Do not attribute undocumented internal algorithms or cooldown numbers to the developers based on these alone.
- Neither the pack/model download contents nor Bevy animation behavior were independently tested. Before shipping an external model, inspect its downloaded license, source and creator, rig/animation inventory, texture rights, required credits, and in-engine visual/performance behavior. No licensed candidate or monster design is approved by this note.

## Status

Research recorded; implementation and runtime review not started.
