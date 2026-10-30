# Playtest: what goes wrong between the eyes

Save: rustvr-city

`playtest.bat` boots Oblivion into `rustvr-city` at noon and moves you to the Imperial City's Market
District (`-At ICMarketDistrict`: people about, in the sun), under a cloudy sky, with the Mirror
power selected. (`rustvr-city` itself is at Odiil Farm, in the forest.) `playtest-rain.bat` is the
same in the rain. `playtest-before.bat` is `playtest.bat` with this branch's game-side fixes
switched off, to compare. Run them from Explorer (double-click). Each one closes a game another .bat
left running, and closes the game and puts your settings back when you press a key.

Each run saves both eyes' pictures (the world, without the HUD, scaled down to 2048 pixels wide)
every 10 seconds for its first ten minutes, into
`<builds>\visuals\playtest\<date-time>-<sky|rain|before>\eyes`, with the runtime's log beside
them. Each picture costs a short hitch. They let me hold the two eyes up to each other.

What was wrong, why, and the audit of the rest: `docs/research/visuals.md`. Research on Oblivion's
own settings for distance and sharpness: `<builds>\visuals\research\oblivion-technical.md`.

## Round 4: the streaks across the trees

`playtest.bat` and `playtest-af.bat` boot `rustvr-city` and move you to the forest spot where the
streaks were (Tamriel -14, 16, looking south-east; it takes a few seconds to get there).

1. **`playtest.bat`**: this build's default, no anisotropic filtering. The streaks across the trees
   should be gone. Look round the same trees and the hillside behind the log.
2. **`playtest-af.bat`** (only if you want to confirm): anisotropic filtering 16x, as the run with the
   streaks had it. If they're back, it was the filtering.
3. **The far distance** (round 3, the runtime): far trees, fences and roofs shouldn't flicker as your
   head moves. That fix is in both .bats.

The ghosting on moving people (round 3's tests) is parked: tell me when you want it back.

## `playtest-rain.bat`

The rain comes in over a few seconds.
- Drops should be at different depths around you: near ones near, far ones far, falling past as
  you walk.
- They shouldn't look painted on your eyes, or move with your head.

## `playtest-before.bat`

Only if a difference isn't clear: the sky on the game's camera, the people's bodies and shadows
drawn for the left eye, and no anisotropic filtering, as before this branch. (The runtime's change
below is in both.)

## What changed

- **Round 3, the far distance's flicker** (the runtime). At your render scale (1.66) the game draws
  each eye at 5099x5418, more pixels than the headset shows. The runtime took one sample of the
  game's picture for each headset pixel, skipping the ones between, so far detail (branches, fences,
  roofs) flickered in and out as your head moved, and the supersampling smoothed nothing. Now each
  headset pixel averages the game's pixels it covers.
- **Round 4, the streaks.** Round 3 filtered the game's textures anisotropically, 16x (Oblivion has
  none of its own, and the driver's override doesn't reach the game in VR). The first run with it
  showed long streaks of foliage across the trees in one spot, and the game drew fewer pictures a
  second. It's now off by default (`anisotropy` in `rustvr-oblivion.toml`); `playtest-af.bat` turns
  it on to compare.
- **Round 2, the people's shadows.** The game clips each actor's shadow to its light with six clip
  planes, made in the camera's own space once a frame, for the left eye. The right eye used the left
  eye's planes, which cut its shadows away. Now the game makes them again for each eye.
- **Round 1, the sky.** The game centres the sky (the dome, the clouds, the sun, the moons, the
  stars) and the weather (rain, snow, the sun's glare) on its camera. In VR each eye is somewhere
  else, so the sky had parallax and bobbed against the eyes. Now the game centres the sky on each
  eye (at infinity for both) and the weather on your head (rain keeps its depth).
- **Round 1, the people.** The game's renderer works out a skinned body's bones relative to the
  first camera that draws it in a frame, and keeps them for the frame. Now each eye is a frame of its
  own for the renderer, so each eye's bodies are its own.

## Not done, not checked

- Round 4 hasn't been seen yet: that the filtering made the streaks is inferred from when they
  came, not proven.
- The ghosting on moving people: not found in the game's picture (the saved pictures, taken before
  the runtime moves them, show none). It's after: Virtual Desktop's spacewarp, the runtime's
  smoothing, or the frames the game misses at 120 Hz. The tests above will tell which.
- Which shadows show, and how far away they fade, are still worked out once a frame from the left
  eye. A shadow at the very right edge of the right eye may be missing.
- Distance and LOD settings in `Oblivion.ini` (how far detail goes): see the research; nothing in
  your `Oblivion.ini` is changed by this branch.
