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

## Round 3: the distance, and the ghosting

1. **The distance.** Walk out of the city's gate (or stay on a wall) and look at the hills,
   trees and buildings a few hundred metres off, then far off. Move your head a little.
   - Far trees, fences and roofs should stay put: no flicker or flashing as the head moves.
   - The ground and walls a few metres to a few hundred metres off should be sharper than before,
     not mushy, even where you see them at a slant (a road ahead, a long wall).
2. **The ghosting on moving people.** Watch people walking past. If you still see an outline or a
   ghost around them, try these one at a time and tell me which one makes it go:
   - In Virtual Desktop's streaming settings, turn **Synchronous Spacewarp** off.
   - In RustVR's menu (the Display tab), turn off **Smooth the game's picture to every headset frame**.
     (Walking and turning will then judder when the game is slower than the headset: that's
     expected, it's only a test.)
   - Set the headset to 90 Hz in Virtual Desktop (the game gives about 115 pictures a second
     now at your resolution, short of 120).
3. **The sky, the people's shadows, the people** (rounds 1 and 2, fixed): still right?

The streaks across the trees in one forest spot (playtest 4) are parked: they were there before the
anisotropic filtering, and may be the game's own.

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
- **Round 3, the mushy distance** (the adapter). Oblivion has no anisotropic filtering of its own;
  on a desktop players force it in the graphics driver, which doesn't reach the game in VR (it runs
  on D3D9On12). Without it, ground and walls at a slant blur a few metres off. The adapter now
  filters the game's textures anisotropically, 16x (`anisotropy` in `rustvr-oblivion.toml`).
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

- None of round 3 has been seen in the headset yet.
- The ghosting on moving people: not found in the game's picture (the saved pictures, taken before
  the runtime moves them, show none). It's after: Virtual Desktop's spacewarp, the runtime's
  smoothing, or the frames the game misses at 120 Hz. The tests above will tell which.
- Which shadows show, and how far away they fade, are still worked out once a frame from the left
  eye. A shadow at the very right edge of the right eye may be missing.
- Distance and LOD settings in `Oblivion.ini` (how far detail goes): see the research; nothing in
  your `Oblivion.ini` is changed by this branch.
