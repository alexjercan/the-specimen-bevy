# Gameplay direction: keys, evidence, and a fair pursuer

Research and a testable prototype proposal, not a locked design. Aim for one
compact facility, a short escape objective, and a monster whose actions the
player can learn. The player should fear being found, not fear that the game is
secretly moving the monster behind them.

## Proposed short session

The player sees a locked service exit near the start. Restore its access with
three keycards or fuses found in distinctive rooms. Show the count and the
exit's location without putting arrows on every pickup. Put possible pickups in
authored locations, not arbitrary corners: a maintenance locker, a lab desk, a
security station. If replay matters, choose three from a larger set of valid
placements and check that each run remains traversable and readable. A key
pickup could also restore one section of power: it improves wayfinding but
starts noisy machinery, giving each success a tangible cost.

A simple loop: orient in a safe entrance -> inspect a room and hear a clue ->
take a key -> make noise/alter the facility -> evade or lose the monster ->
regroup -> return to the exit. Give each room one identifying sight and one
identifying sound. Use a single change (a broken lamp, moved cart, open vent)
to make a familiar route unsettling on the return trip. Make the final run to
the exit a *known route under new pressure*, not a newly generated maze.

## Monster: legible hunting before complex AI

Prototype one pursuer with Patrol -> Investigate -> Search -> Chase -> Recover.
On sight or a sufficiently loud sound, it travels to a **last known position**,
not the player's continuously updated coordinates. After losing sight, search
nearby briefly, then return to patrol with a recovery window. Let footsteps,
doors, and quiet growls disclose approximate location and state. Avoid
teleporting into a room the player just cleared. Avoid a scripted capture that
contradicts what the player could see or hear.

Start with one floor and connected loops rather than dead ends only. Add hide
spots or alternate doors only where a player can understand them under stress.
Before the first lethal chase, let the player see or hear the monster
investigating a different room. In a chase, make the next turn visible through
light, color, signs, or layout; give players a second chance after a small
navigation error. Break up pursuits with real respite, or high intensity
becomes routine.

**Prototype tuning, not researched constants:** try a 10-15 m sight range, a
finite vision cone with occlusion checks, noisy sprint steps that travel
farther than walking steps, a 10-20 second search, and a short recovery period
after losing the player. Tune by watching new players. A key count may raise
patrol activity or sound sensitivity, but show the escalation through behavior
and audio, not through unexplained instant detections. Keep the player's
position out of the monster's target selection except when a sight/hearing test
succeeds.

## Anticipation and scare budget

A jumpscare is a release, not the main mechanic. First establish a pattern
(buzzing light -> nearby maintenance closet), then break it (buzzing light,
empty closet, footsteps behind a wall). An occasional false alarm should be
grounded in the facility, not a random loud sample. Hold off on a full monster
reveal until its audio and traces have taught the player what it might do. Use
at most one or two authored surprise beats in an initial short prototype; do
not always place a monster at the end of a long corridor. A silent return
through a previously safe, changed room can be more effective and cheaper to
build.

**Fairness playtest:** after every death, ask the player where they think the
monster came from and which choice could have saved them. If they cannot
answer, fix cue timing, sightlines, or escape options before raising
difficulty. Track deaths per room, time between sightings, and whether players
can find the exit unaided. Test navigation with sound off and threat detection
with music off.

## Design references (inspiration, not assets)

- [Building Fear in Alien: Isolation (GDC)](https://gdcvault.com/play/1021852/Building-Fear-in-Alien) and [The Perfect Organism (Game Developer)](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation): pacing and the difference between threat behavior and overarching tension control. A full director is likely beyond this prototype's scope.
- [Should the Monster Play Fair? (Game Studies)](https://gamestudies.org/2002/articles/jaroslav_svelch): players disagree about invisible AI assistance; believable causality matters when designing a pursuer.
- [How the Beast Works in Amnesia: The Bunker (AI and Games)](https://www.aiandgames.com/p/how-the-beast-works-in-amnesia-the): a secondary analysis of noise, search, and a persistent threat; do not assume its internal implementation is confirmed by the developer.
- [The art of the chase: Outlast 2 (Game Developer)](https://www.gamedeveloper.com/audio/the-art-of-the-chase-level-design-and-player-orientation-in-i-outlast-2-i-): use strong visual guidance under chase pressure and vary the rhythm.
- [Creating Horror through Level Design (Game Developer)](https://www.gamedeveloper.com/design/creating-horror-through-level-design-tension-jump-scares-and-chase-sequences): anticipation, contrast, and why repeated scare tactics lose effect.
- [What Happened Here? (GDC notes)](https://www.worch.com/files/gdc/What_Happened_Here_Web_Notes.pdf): environmental clues that suggest an event without exposition.

The keycard/power tradeoff, room loop, numbers, and state machine above are
proposals synthesized for this project, not mechanics mandated by those
sources. Implement the cheapest blockout first and change the design after
playtesting.
