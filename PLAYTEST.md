# Playtest: what goes wrong between the eyes

Save: rustvr-weye

`playtest.bat` boots Oblivion into `rustvr-weye`: Weye, on Lake Rumare's shore at noon, under a clear
sky, with the water, the far forest and the Imperial City across the lake, and the road along the
shore. `playtest-before.bat` is the same with this round's changes switched off, to compare.
`playtest-rain.bat` is the Imperial City's Market District in the rain. Run them from Explorer
(double-click). Each one closes a game another .bat left running, and closes the game and puts your
settings back when you press a key.

Each run saves both eyes' pictures (the world, without the HUD, scaled down to 2048 pixels wide)
every 10 seconds for its first ten minutes, into
`<builds>\visuals\playtest\<date-time>-<name>\eyes`, with the runtime's log beside them. Each
picture costs a short hitch. The game's log (`rustvr-oblivion.log` in the game's folder) now also
says how long the game waits for its loading.

What was wrong, why, and the audit of the rest: `docs/research/visuals.md`. The research on the
game's own image quality and stutter: `docs/research/oblivion-image-quality.md`.

## Round 5: stutter, and the research's fixes (`playtest.bat`)

1. **Stutter crossing cells.** Walk, or better run, along the road for a few minutes, through
   several cells (one is about 58 m). The hitches as new land loads should be fewer and shorter
   than before. (`playtest-before.bat` is the same with the old behaviour, if you want to compare.)
2. **The far trees.** Look across the lake at the far forest and turn your head slowly left and
   right. The far trees should stay put, not shimmer or swing as you turn.
3. **The near trees.** They should sparkle less as your head moves (no sharpening bias), and switch
   detail less as you walk (full detail).
4. **The water.** The waves should move smoothly, not step.
5. **Shadows.** Characters' shadows should be sharper up close (1024 shadow maps, not 256).
6. **The ground.** Roads and ground at a slant should be sharp a long way out (anisotropic
   filtering, 16x).
7. **The frame rate.** Tell me if any of this costs too much (the runtime's log has the pictures a
   second); each piece can be turned off.

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

- **Round 5, stutter.** The game's background loading thread can only change the world while no
  render holds its lock. Two eyes and their copies held it for most of each frame, so it fell behind
  with the cells ahead and the game stood waiting for it. Now each eye gives it back, so it can load
  while the eye's picture is copied.
- **Round 5, the game's own settings for a headset** (only while in VR, your own higher values kept,
  all put back when the game closes): actor shadow maps 1024, no sharpening bias on the trees, near
  trees at full detail, the waves at 36 frames a second, and 100 MB of the cells ahead preloaded.
- **Round 5, the distant trees.** The game turned every far tree card to the way the camera looks,
  so every turn of your head turned the whole far forest. Its shader is now made to turn each card
  to face your eye instead, as the game's near billboards do.
- **Round 5, anisotropic filtering** back on, 16x: the streaks across the trees were there before it.
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

- None of round 5 has been seen in the headset yet. The stutter's cause is inferred from the
  timing log, not measured: this run's log measures it (the game's waits for its loading).
- Grass grown only in front of the flat game's view (it comes and goes at the edge of a headset's
  view) and the water's reflections resetting as the head moves (the research's other VR items) are
  not done.
- Mods (Oblivion Stutter Remover, the 4GB patch, LOD regeneration) need files from outside and
  changes to your game's install: set out in the research, not done here.
- The ghosting on moving people: not found in the game's picture (the saved pictures, taken before
  the runtime moves them, show none). It's after: Virtual Desktop's spacewarp, the runtime's
  smoothing, or the frames the game misses at 120 Hz. The tests above will tell which.
- Which shadows show, and how far away they fade, are still worked out once a frame from the left
  eye. A shadow at the very right edge of the right eye may be missing.
- Distance and LOD settings in `Oblivion.ini` (how far detail goes) are left as yours: they cost
  draw calls twice over in VR. See the research.
