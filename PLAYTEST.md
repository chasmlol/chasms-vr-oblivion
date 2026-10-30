# Playtest: what goes wrong between the eyes

Save: rustvr-city

`playtest.bat` boots Oblivion into `rustvr-city` at noon and moves you to the Imperial City's Market
District (`-At ICMarketDistrict`: people about, in the sun), under a cloudy sky, with the Mirror
power selected. (`rustvr-city` itself is at Odiil Farm, in the forest.) `playtest-rain.bat` is the
same in the rain.
`playtest-before.bat` is `playtest.bat` with this branch's fixes switched off, to compare.
Run them from Explorer (double-click). Each one closes a game another .bat left running, and
closes the game and puts your settings back when you press a key.

Each run saves both eyes' pictures (the world, without the HUD) every 10 seconds for its first five
minutes, into `<builds>\visuals\playtest\<date-time>-<sky|rain|before>\eyes`, with the runtime's
log beside them. Each picture costs a short hitch. They let me hold the two eyes up to each other.

What was wrong, why, and the audit of the rest: `docs/research/visuals.md`.

## What to look at, in order (`playtest.bat`)

1. **The clouds and the sky.** Stand still in the market and look up.
   - The clouds should sit far away, like the horizon: they turn only as you turn your head.
   - Move your head side to side, then up and down (crouch and stand). They shouldn't slide or swim
     against the rooftops.
   - Close one eye, then the other: the clouds shouldn't jump.
   - Walk about with the stick, and up and down any steps. The clouds shouldn't bob.
   - The sun, and at night the moons and the stars, should behave the same way.
2. **The people's shadows.** Walk up to people in the market, to about 2 m, and look at the
   shadows at their feet.
   - Each shadow should be in both eyes, in the same place on the ground: solid, not see-through or
     flickering.
   - Close one eye, then the other: the shadow should stay.
   - This was the "cross-eyed" feeling: playtest 1's pictures show a rider's horse with its shadow in
     the left eye and none in the right.
   - Their bodies should be in one place with their heads and what they carry: one solid person.
3. **The mirror** (optional). Cast Mirror (your selected power: hold the trigger with the spell hand
   empty, let go) with people behind you. The people in the mirror should be whole: bodies with their
   heads.
4. **Anything else that looks different between your eyes.** This is the bug family; tell me what and
   where (and roughly when, so I can find it in the pictures).

## `playtest-rain.bat`

The rain comes in over a few seconds.
- Drops should be at different depths around you: near ones near, far ones far, falling past as
  you walk.
- They shouldn't look painted on your eyes, or move with your head.

## `playtest-before.bat`

Only if the difference isn't clear. The sky is centred on the game's camera and the people's bodies
are drawn for the left eye in both, as before this branch.

## What changed

- **The sky.** The game centres the sky (the dome, the clouds, the sun, the moons, the stars) and
  the weather (rain, snow, the sun's glare) on its camera. In VR each eye is somewhere else: the
  head's real height, the room-scale offset, the steps smoothed out. So the sky had parallax and
  bobbed against the eyes. Now the game centres the sky on each eye: it's at infinity for both eyes.
  It centres the weather on your head, the same place for both eyes, so rain keeps its depth.
- **The people's shadows** (round 2). The game clips each actor's shadow to its light with six clip
  planes, made in the camera's own space once a frame, for the left eye. The right eye used the left
  eye's planes, which cut its shadows away. Now the game makes them again for each eye.
- **The people.** The game's renderer works out a skinned body's bones relative to the first camera
  that draws it in a frame, and keeps them for the frame. Both eyes and the mirror were one frame, so
  the right eye and the mirror drew bodies placed for the left eye. Now each eye is a frame of its
  own for the renderer, so each eye's bodies are its own. The heads, the hair, the weapons and the
  shadows were always right: they aren't skinned.
- Both use the game's own code: the same copies and bone calculation, given the eye.

## Not done, not checked

- I can't run the game, so none of this has been seen yet: the pictures this run saves will show me.
- Round 1's pictures (your first run) showed the shadows missing from the right eye; the fix for it
  is worked out and hasn't been seen yet.
- Which actors cast shadows, and how far away they fade, are worked out once a frame from the left
  eye, as before. A shadow at the very right edge of the right eye may be missing.
- Trees' leaves, the distant land's LOD and HDR (off in your settings) weren't checked in the
  game's code. See `docs/research/visuals.md`.
