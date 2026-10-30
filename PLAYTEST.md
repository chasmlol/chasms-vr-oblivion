# Playtest: what goes wrong between the eyes

Save: rustvr-weye

`playtest.bat` boots Oblivion into `rustvr-weye`: Weye, on Lake Rumare's shore at noon, under a clear
sky, with the water, the far forest and the Imperial City across the lake, and the road along the
shore. `playtest-before.bat` is the same with this round's changes switched off, to compare.
`playtest-rain.bat` is the Imperial City's Market District in the rain. Run them from Explorer
(double-click). Each one closes a game another .bat left running, and closes the game and puts your
settings back when you press a key.

The eye pictures are off now: each pair cost a 140 ms frame, every 10 seconds, and that was most of
the stutter in your last run. The game's log (`rustvr-oblivion.log` in the game's folder) says how
long the game waits for its loading, and every 10 seconds what the trees' leaves were given (below).

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
4. **The grass.** It should stand all round you: no grass growing in or vanishing at the edges of
   your view as you turn your head.
5. **The water.** The waves should move smoothly, not step.
6. **Shadows.** Characters' shadows should be sharper up close (1024 shadow maps, not 256).
7. **The ground.** Roads and ground at a slant should be sharp a long way out (anisotropic
   filtering, 16x).
8. **The frame rate.** Tell me if any of this costs too much (the runtime's log has the pictures a
   second); each piece can be turned off.

## The game's own install (every playtest runs with it now)

Set up in the game's folder at your request, for the stutter and the distance:
- **The 4GB patch.** `Oblivion.exe` may use 4 GB of memory instead of 2. The original is kept beside
  it as `Oblivion.exe.rustvr-original`. Once the patch is in effect, the timing line in the game's
  log (`rustvr-oblivion.log`) ends "address space N of 4095 MB free"; before, it said "of 2047".
- **xOBSE 22.14**, the script extender that Oblivion Stutter Remover needs. Steam loads it only into a
  game Steam starts, and the playtest starts the game itself, so in your last run neither it nor
  Stutter Remover ran. The adapter now loads it at the same point. After a run, an `obse.log` in the
  game's folder means it loaded.
- **Oblivion Stutter Remover 4.1.37** (`Data\OBSE\Plugins`), set for VR. Kept from it: its locks,
  which stop the game's threads queueing behind each other; its bigger lookup tables; and its
  replacement for the game's memory heap (512 MB). Off: its frame limiter and timer, because the
  headset and the adapter pace and time the frames. Also off: its fast exit, which would save the VR
  settings into your `Oblivion.ini`. One lock it normally removes is kept, because in VR the
  game's loader runs between the eyes. Its log, `sr_Oblivion_Stutter_Remover.log` beside it, says it
  loaded. If the game crashes more than before, set `bReplaceHeap = 0` in its `.ini` first.
- **The distant objects, regenerated** from your game's own data (xEdit's TES4LODGen) into
  `Data\DistantLOD`. A few castle walls, gates and towers (Skingrad's, Bravil's, Leyawiin's) that the
  game's own lists miss now stand in the distance too. The generator also makes every shrub a far
  card, which doubled the far cards around the Imperial City; those are left out, as the game leaves
  them.
- **The far land's shape, regenerated** (tes4ll) for six of the game's 36 far-land areas: the Imperial
  City and Weye, Chorrol, Skingrad and Cheydinhal, and the land east of the Niben in the south-east
  (`Data\Meshes\Landscape\LOD`).
  About twice the game's detail in the land past the loaded cells, so hills and ridges hold their
  shape further out. The edges of the map keep the game's own: there its far land shows scenery past
  the border that the game has no land data for. Its colours are the game's own.
- `undo-stutter-setup.bat` puts all of it back. If Steam verifies or updates the game, it puts the
  original exe back (only the 4GB patch is lost).

9. **It starts and plays as before.** If it doesn't, or something misbehaves, run
   `undo-stutter-setup.bat` and tell me.
10. **The far castles.** Skingrad's castle and the walls and towers of the towns should stand in the
    distance, not come in as you get near.
11. **Stutter again.** With Stutter Remover in, the hitches crossing cells (check 1) should be fewer
    again. Tell me about any new crash, and when it happened (loading, crossing cells, quitting).
12. **The far hills.** From the shore at Weye, look out over the hills past the lake: their shape
    should hold further out. Tell me if you see a seam, a step or a crack in the far land, and roughly
    where (towards which town).
13. **The streaks in the trees.** If you see them, note where and roughly when: the game's log has
    what the leaves were given and which draws were widened, every 10 seconds.
14. **Pop-in.** Things should appear much further out: trees, rocks, people, grass. It costs frames:
    if the game judders (frames repeating, smooth motion going uneven), try the headset at 90 Hz in
    Virtual Desktop. At 120 Hz the game has 8 ms a frame, and your last run made 110..135 frames a
    second, so it missed some.

## `playtest-rain.bat`

The rain comes in over a few seconds.
- Drops should be at different depths around you: near ones near, far ones far, falling past as
  you walk.
- They shouldn't look painted on your eyes, or move with your head.

## `playtest-before.bat`

Only if a difference isn't clear: the sky on the game's camera, the people's bodies and shadows
drawn for the left eye, and no anisotropic filtering, as before this branch. (The runtime's change
below is in both.)

## Round 10: which kind of draw the streak is (`playtest-leaveout.bat`, one run)

Neither shadow pass was it (round 8). So this run checks every kind at once. Every 5 seconds one
frame leaves out one kind of the game's draws, in turn, and both eyes' pictures of that frame are
saved, named after what it left out. The kinds: the far land, grass, leaves, tree branches, the
distant trees, the ground that takes shadows, skinned bodies, everything else, and the few drawn
without a shader. Go to where you see the streak and keep it in view for 2..3 minutes. The game
hitches every 5 seconds (the pictures), and the left-out kind blinks out for one frame. The pictures
with the streak gone say which kind it is.

## Round 8: which shadows draw the streak (two short runs)

The streak is the ground drawn again with a shadow's texture laid on it. That's the game's shadow
pass, whose texture coordinates go off to infinity at the edge of what the shadow covers, and your
eyes see that edge differently, so it moves as your head does. Two passes do that, so one run each:
- `playtest-nocanopy.bat`: the trees' canopy shadows off.
- `playtest-noshadows.bat`: people's shadows off (yours too).

Look where you usually see the streak and tell me which run (if either) has it gone. Each one closes
and puts your settings back like `playtest.bat`.

## What changed

- **Round 11, the streak, found and fixed.** Your leave-one-out run showed it: in the frame without
  the trees' leaves it was gone from both eyes, and in every other frame it was there. The leaves'
  shader picks each leaf's corner by a number with a shade hidden in its fraction, written for the
  graphics drivers of 2006, which rounded it down. The layer the game runs on now (D3D9On12) rounds
  it to the nearest, as Microsoft's own reference says. So half the leaves took the next cluster's
  corner, and the last cluster's read past the corners into whatever another draw had left there:
  a corner flung across the view, different in each eye. The shader is now given the whole number
  itself, so rounding can't move it.
- **Round 9, the leaves.** They no longer turn or lean with your head: each tree's leaf cards face
  your eye and stay upright, so they only turn as you walk round the tree.
- **Round 9, stutter.** Your last run's real hitches (40..120 ms, every 10..30 s while moving) were
  the game attaching newly loaded cells, and drawing new things the first time. Your `Oblivion.ini`
  had the game's own background loading of people's faces and lip files off, and its threaded face
  morpher, blood and path finding: they're on now while in VR. The 7x7 grid (round 7) makes each
  cell crossing load 7 cells instead of 5. If the hitches matter more to you than the pop-in, that's
  the one to take back.
- **Round 8, frame rate.** Your last run made 75..90 frames a second, down from 110..135: the longer
  distances doubled what the game draws for each eye. The grass, all round you at 86 m, was the
  heaviest: it's back to 57 m (still beyond the 43 m before). The rest stays. The headset is at
  120 Hz, which the game doesn't reach in the open now, so every frame it misses repeats (judder).
  Setting Virtual Desktop to 90 Hz would give it 11 ms a frame instead of 8.
- **Round 7, pop-in.** Your `Oblivion.ini` now loads a 7x7 grid of cells around you, not 5x5
  (`uGridsToLoad=7`, its cell buffer 64; your old file is `Oblivion.ini.rustvr-before-grids`). That
  one stays when you play flat too: a save made at 7 can crash the game if it's loaded at 5. While in
  VR, things fade in much further out: objects, items and people at the sliders' most (15), trees
  at the game's limit, grass out to 86 m (was 43), and the far buildings and tree cards half as far
  again.
- **Round 7, stutter.** Stutter Remover's log showed the 240 ms hitch: up to a dozen threads queued
  on the renderer's texture lock, the one lock I had kept. It's off now, as Stutter Remover's own
  default.
- **Round 7, the streaks.** The leaves' constants were all sound in your run, so it isn't them. The
  next suspect: vertex buffers in system memory, which D3D9On12 copies only as far as each draw's
  hints say, where Direct3D 9 read them whole. Those draws are now widened to the whole buffer, and
  the log counts them.
- **Round 6, stutter.** The eye pictures every 10 seconds were the 140 ms hitches: they're off.
  xOBSE, and with it Stutter Remover, now loads when the playtest starts the game, as it does when
  Steam starts it.
- **Round 6, the streaks.** They're the trees' leaf cards, with corners thrown across the view. The
  adapter now checks what the game gives the leaves: a corner that isn't a number collapses the leaf
  instead of streaking, and the log says what they were given.

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
- **Round 5, the grass.** The game grew grass only in a cone in front of its camera, sized for a
  flat screen (46 degrees either side) and rebuilt after a 19-degree turn. Now it grows it all round
  you, to the game's own grass distance.
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
- The water's reflections resetting as the head moves (the research's other VR item) is not done.
- Your `Oblivion.ini` has the grass distance under `[Display]`, where the game doesn't read it: the
  grass reaches 43 m (`[Grass]`'s 3000). Raising `[Grass] fGrassEndDistance` grows it further, at a
  cost (it's now all round you).
- The far land's colours are the game's own. TES4qLOD's colour maps at 2048 or 4096 aren't sharper:
  it colours from one 4x4-pixel average of each ground texture, so above 1024 it only blurs between
  the land's points.
- The ghosting on moving people: not found in the game's picture (the saved pictures, taken before
  the runtime moves them, show none). It's after: Virtual Desktop's spacewarp, the runtime's
  smoothing, or the frames the game misses at 120 Hz. The tests above will tell which.
- Which shadows show, and how far away they fade, are still worked out once a frame from the left
  eye. A shadow at the very right edge of the right eye may be missing.
- Distance and LOD settings in `Oblivion.ini` (how far detail goes) are left as yours: they cost
  draw calls twice over in VR. See the research.
