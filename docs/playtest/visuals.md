# Playtest: what goes wrong between the eyes

Save: rustvr-weye

Oblivion draws its world once a frame from one camera, and a few things it makes once a frame for
that camera (the sky's centre, a body's bones, a shadow's clip planes, the grass, the leaves and far
trees facing the view) are wrong for a second eye, or for a head that never stops moving. This is
the set of fixes for them, the game's own settings raised for a headset, and the work on the stutter
crossing into new cells. The engine facts are in `docs/research/visuals.md`; the research on the
game's image quality, its settings and the community's mods, in
`docs/research/oblivion-image-quality.md`.

## Playing it

From the repository's root:

    powershell -File tools\oblivion.ps1 -Save rustvr-weye -Hour 12 -Weather 00038EEE

It builds main, installs it beside the game and boots straight into the save for the headset:
Weye, on Lake Rumare's shore at noon under a clear sky, with the water, the far forest and the
Imperial City across the lake, and the road along the shore. For the rain:

    powershell -File tools\oblivion.ps1 -Save rustvr-city -At ICMarketDistrict01 -Weather 00038EF2

`powershell -File tools\oblivion.ps1 -Close` puts the game away and your settings back.

`rustvr-weye` and `rustvr-city` are the developer's own test saves, not in the repository
(`docs/DEVELOPING.md` has the test saves, the fast boot and the game lock). To make one from any
save, boot it somewhere on the lake's shore,
`powershell -File tools\oblivion.ps1 -Save <any save> -At <cell>`, and save it as `rustvr-weye`
([DEVELOPING.md](../DEVELOPING.md), "Test saves").

Every fix below is on by default. Each has a testing switch that turns it off for a run (see
"Testing switches"), to compare; set the variable in the shell before `tools\oblivion.ps1` and the
game inherits it (`$env:RUSTVR_OBLIVION_MONO_VISUALS = "1"` in PowerShell).

## What it does

The adapter (`crates/oblivion`):

- **The sky on each eye** (`render.rs`). The world render centres the "Sky" node (the dome, the
  clouds, the sun, the moons, the stars) and the "Weather" node (rain, snow, the sun's glare) on the
  game's camera. Instead, the sky is centred on each eye's camera (at infinity for both, turning
  only as the head turns) and the weather on the middle of the two eyes, so rain and snow keep their
  depth. Every 600 frames the log says how far the sky's centre is from each eye (0 with the fix).
- **People's bodies per eye** (`render.rs`, `engine::next_render_frame`). The renderer makes a
  skinned model's bones relative to the first camera that draws it in a frame. Each render after
  the first is a frame of the renderer's own, so the second eye (and the mirror) draws the bodies
  where it sees them.
- **People's shadows in both eyes** (`render.rs`). An actor's shadow is clipped to its light with
  six user clip planes, made in the camera's clip space by the first draw of a frame and kept.
  Before each render after the first, the mark on each shadow light is cleared, so the game makes
  the planes again for that camera. The actors' shadow pass runs once a frame, with the first eye;
  both eyes use its shadow maps.
- **The background loader's lock per eye** (`render.rs`). The game's background thread (cells and
  actors' 3D) changes the scene only while no render holds its lock. Two eyes and their copies held
  it for most of the frame, so it fell behind loading the cells ahead. Each eye's render now takes
  and gives it, so it can work while that eye's picture is copied. The game's waits for it are timed
  ("waits for the loader" in the timing line) and one over 20 ms is logged (`render: the game waited
  N ms for its background thread (loading)`).
- **The far trees face the eye** (`distant_trees.rs`). The game's distant-tree shader
  (`DISTLOD2003.vso`) turns every card to the camera's look direction, so each turn of the head
  turned the whole far forest. When the game makes it, the adapter's own HLSL replaces it: the same
  registers, light, fog and fade, each card turned to the direction from the eye to its tree.
- **The near trees' leaves** (`leaves.rs`). Three fixes to SpeedTree's leaf cards:
  - the cards stay upright and turn only as you walk round a tree, not as your head turns or leans:
    for each draw their right is made level and square to the line from the eye to the tree;
  - the streaks: the leaves' `vs_1_1` shaders index their corners and wind matrices by a vertex
    value with a shade in its fraction, written for drivers that rounded it down. D3D9On12 rounds to
    the nearest, so half the leaves took the next cluster's corner and the last cluster read past
    the corners: a corner thrown across the view, differently in each eye. The shaders are made to
    take the whole part of the index first;
  - the constants the game gives a leaf shader are checked: a value that isn't finite is made 0, and
    every 10 s the log sums up what they held (`leaves: ...`).
- **System memory vertex buffers drawn whole** (`sysmem.rs`). D3D9On12 copies only the vertices a
  draw's hints name from a system memory buffer, where Direct3D 9's drivers read it whole. An
  indexed draw from one has its hints widened to the whole buffer (up to 262,144 vertices); the log
  counts them every 10 s (`sysmem: ...`).
- **Grass all round you** (`grass.rs`). The game grows grass only in a cone in front of its camera
  (46 degrees either side at the flat game's field of view, built again after a 19-degree turn). The
  adapter skips two of its checks: every patch is kept, to the game's own grass distance, and a turn
  of the head no longer rebuilds it (a move of 64 units still does).
- **Fog by distance** (`fog.rs`, added on main after the rounds below). The game's vertex shaders fog
  by the length of the clip-space position, which grows towards a frustum's edges: the headset's
  wide, off-centre eyes fogged a far hill differently, its haze changed as the head turned, the
  overscan thinned the fog, and the land didn't meet the water. As the game makes each shader that
  fogs, it's rewritten to fog by the view-space distance (the frustum's terms in `c255`, set for
  each camera). A shader that can't be rewritten keeps its own fog.
- **Anisotropic filtering** (`d3d9.rs`, `anisotropy` in `rustvr-oblivion.toml`, 16 by default).
  Oblivion has none of its own; on a desktop players force it in the graphics driver, which doesn't
  reach a game on D3D9On12. A pixel shader's sampler the game sets to filter linearly as it shrinks
  a texture filters anisotropically instead; its other filters, and its image-space passes, are as
  it sets them.
- **The game's own settings for a headset** (`game_settings.rs`, `VR_FLOORS`). While in VR, a few of
  `Oblivion.ini`'s values are raised to at least these; your own higher ones are kept, and all of
  them go back when the game closes (or, after a crash, at the next start before anything else,
  from `rustvr-oblivion-ini.txt` beside the DLL; `tools\oblivion.ps1 -Close` does it too):

  | Section | Key | VR floor | Why |
  |---|---|---|---|
  | Display | `iShadowMapResolution` | 1024 | 256 is blocks at arm's length |
  | SpeedTree | `fLODTreeMipMapLODBias`, `fLocalTreeMipMapLODBias` | 0 | the game's negative bias sparkles as the head moves |
  | SpeedTree | `bForceFullLOD` | 1 | near trees don't switch detail as you move |
  | Water | `uSurfaceFPS`, `uSurfaceFrameCount` | 36, 96 | the waves' 12-frame loop steps at 90..120 Hz (the same 2.7 s loop) |
  | General | `iPreloadSizeLimit` | 262144000 (250 MB) | the cells ahead preloaded (the game's 25 MB loads them as you go) |
  | LOD | `fLODFadeOutMultObjects`, `...Items`, `...Actors` | 15 | the view distance sliders' most: a head turns to what a flat screen hides |
  | LOD | `fLODMultTrees` | 2 | the game's own limit |
  | Grass | `fGrassEndDistance`, `fGrassStartFadeDistance` | 4000, 3000 | grass to 57 m (the `[Grass]` keys, the ones the game reads) |
  | General | `uGridDistantCount`, `uGridDistantTreeRange` | 40, 30 | far buildings and tree cards further out |
  | BackgroundLoad | `bLoadBackgroundFaceGen`, `bBackgroundLoadLipFiles` | 1 | faces and lip files on the game's background threads |
  | General | `bUseThreadedMorpher`, `bUseThreadedBlood` | 1 | off the frame's thread |
  | Pathfinding | `bBackgroundPathing` | 1 | off the frame's thread |

  The log says which were raised (`game settings: for the headset: ...`).
- **xOBSE when it's installed** (`xobse.rs`). Steam loads xOBSE's `obse_steam_loader.dll` only into a
  game it starts; the adapter's launcher starts the exe itself. If the loader is in the game's
  folder and nothing has loaded it, the adapter loads it on the game's main thread as its C runtime
  starts (its one call of `GetCommandLineA`), where xOBSE's own launcher would. Then xOBSE loads its
  plugins, Oblivion Stutter Remover among them. `obse.log` in the game's folder means it loaded.
- **The timing line** (`timing.rs`). Every 5 s the adapter's log has the frames a second, the worst
  frame, the time of each part of a frame (the eye renders, the picture and depth copies, the waits
  for the loader, "between renders": the game's update, its menus, Present), the address space left
  (`address space N of M MB free`: 4095 once the 4GB patch is in effect, 2047 without) and the video
  memory against its budget.

The runtime (`framework/crates/runtime/src/compositor.rs`):

- **The game's picture averaged over each headset pixel.** A game drawn sharper than the headset
  shows it (Supersampling above 1) has more pixels than the headset. One sample a pixel skipped the
  ones between, so far detail (branches, fences, roofs) flickered as the head moved. Each headset
  pixel now averages up to 4x4 samples over the game's pixels it covers.

## The developer's install

The developer's own game has these, set up at their request for the stutter and the distance.
None of it is in the repository or done by its scripts; a player without them gets the fixes above
all the same. What each is and how it was set up: `docs/research/oblivion-image-quality.md`.

- **The 4GB patch** on `Oblivion.exe` (the original kept beside it as
  `Oblivion.exe.rustvr-original`). If Steam verifies or updates the game, it puts the original back.
- **xOBSE 22.14**, which Oblivion Stutter Remover needs (the adapter loads it, above).
- **Oblivion Stutter Remover 4.1.37** (`Data\OBSE\Plugins`), set for VR: its locks, bigger lookup
  tables and its heap (512 MB) kept; its frame limiter, timer and fast exit off (the headset and the
  adapter pace and time the frames, and the fast exit would save the VR values into
  `Oblivion.ini`); the renderer's texture lock suppressed, as its own default. Its log is
  `sr_Oblivion_Stutter_Remover.log` beside it. If the game crashes more than before, set
  `bReplaceHeap = 0` in its `.ini` first.
- **The distant objects regenerated** with xEdit's TES4LODGen into `Data\DistantLOD`, keeping only
  the tree kinds the game's own lists draw far away (unfiltered it makes every shrub a far card). A
  few castle walls, gates and towers the game's lists miss (Skingrad's, Bravil's, Leyawiin's) stand
  in the distance.
- **The far land's shape regenerated** with tes4ll for the six far-land quads the game's cells cover
  completely (the Imperial City and Weye, Chorrol, Skingrad and Cheydinhal, the land east of the
  Niben in the south-east), in `Data\Meshes\Landscape\LOD`: about twice the detail past the loaded
  cells. The map's edges and the far land's colours stay the game's.
- **`uGridsToLoad=7`** in `Oblivion.ini` (cell buffer 64), a 7x7 grid of cells loaded round the player
  instead of 5x5. It stays when the game is played flat: a save made at 7 is said to crash at 5, so
  it can't be a VR-only value. It makes each cell crossing load 7 cells instead of 5; if the hitches
  matter more than the pop-in, it's the one to take back.

The developer's own undo script (not in the repository) puts all of it back.

## What to try

1. **Stutter crossing cells.** Walk, or better run, along the road for a few minutes through several
   cells (one is about 58 m). The hitches as new land loads should be few and short. Afterwards the
   adapter's log (`rustvr-oblivion.log` in the game's folder) has the timing line every 5 s and each
   wait for the loader over 20 ms.
2. **The far trees.** Look across the lake at the far forest and turn your head slowly left and
   right: the far trees stay put, not shimmering or swinging as you turn.
3. **The near trees.** Their leaves don't turn or lean with your head, only as you walk round a tree.
   They sparkle little as your head moves, keep their detail as you walk, and no streaks of leaf
   run across the view.
4. **The grass** stands all round you: none grows in or vanishes at the edges of your view as you
   turn your head.
5. **The water.** The waves move smoothly, not in steps.
6. **The sky.** The clouds, the sun and the moons sit at infinity: no parallax between the eyes, no
   bobbing as you walk.
7. **People.** Their bodies are where their heads and weapons are in both eyes, and their shadows
   are under them in both eyes, sharp up close.
8. **The ground and the haze.** Roads and ground at a slant are sharp a long way out. A far hill is
   hazed the same in both eyes and its haze doesn't change as you turn towards it; the land meets
   the water.
9. **Pop-in.** Trees, rocks, people and grass appear far out.
10. **The far distance's detail** (with Supersampling above 1): far branches, fences and roofs hold
    steady as your head moves.
11. **The rain** (the rain command above; it comes in over a few seconds): drops at different depths
    round you, near ones near, falling past as you walk; not painted on your eyes, not moving with
    your head.
12. **The frame rate.** Say if any of this costs too much: each piece can be turned off for a run.
    The longer distances draw more for each eye; a headset at 120 Hz gives the game 8 ms a frame, at
    90 Hz 11 ms, and a frame it misses repeats (judder). Virtual Desktop keeps the rate its own
    settings have.

## Settings

- **The runtime's menu, Display tab:**
  - "Supersampling (the game's resolution, each way)": 0.5 to 2, default 1. Above 1 the averaging
    over each headset pixel smooths far detail; it costs the game's frame.
  - "Refresh rate": highest (the default), 72, 90 or 120.
  - "Overscan: drawn past the headset's view": 0 to 20 degrees, default 0; a new value takes when
    the game starts again.
  - "Smooth the game's picture to every headset frame (walking and turning too)": on by default.
- **`rustvr-oblivion.toml`** beside the game's exe (`crates/oblivion/src/settings.rs`):
  - `anisotropy`: 1 (none, as the game has it) to 16, default 16. For one run:
    `tools\oblivion.ps1 ... -Settings "anisotropy = 1"`.
  - `eye_width`, `eye_height`: the size the game draws each eye at; 0 (the default) for the
    headset's own, as the runtime last reported it.
- **`Oblivion.ini`**: the VR floors above, raised only while in VR.

### Testing switches

Environment variables the adapter reads at start, each for one run:

| Variable | What it does |
|---|---|
| `RUSTVR_OBLIVION_MONO_VISUALS=1` | the sky and the weather on the game's camera, one renderer frame and one set of shadow clip planes for all the renders |
| `RUSTVR_OBLIVION_HOLD_LOADER=1` | the loader's lock held across all the renders of a frame |
| `RUSTVR_OBLIVION_GAME_DISTANT_TREES=1` | the game's own distant-tree shader |
| `RUSTVR_OBLIVION_GAME_LEAF_FACING=1` | the leaf cards square to the way the head looks, as the game has them |
| `RUSTVR_OBLIVION_GAME_LEAF_INDEX=1` | the leaves' indices as the game wrote them (the streaks) |
| `RUSTVR_OBLIVION_LEAVES_AS_GAME=1` | the leaves' constants unchecked |
| `RUSTVR_OBLIVION_DRAWS_AS_GAME=1` | the draws' vertex hints as the game gives them |
| `RUSTVR_OBLIVION_GAME_GRASS=1` | the game's grass cone |
| `RUSTVR_OBLIVION_GAME_FOG=1` | the game's own fog |
| `RUSTVR_OBLIVION_ANISOTROPY=<n>` | the anisotropy, over the setting |
| `RUSTVR_OBLIVION_FLAT_INI=1` | no VR floors: your `Oblivion.ini` as it is |
| `RUSTVR_OBLIVION_INI_TEST="Section.key=value;..."` | any of the game's settings for one run, put back with the rest |
| `RUSTVR_OBLIVION_NO_XOBSE=1` | xOBSE not loaded |
| `RUSTVR_OBLIVION_EYE_PNG=<folder>` | both eyes' pictures saved there every 10 s from 5 s in, up to 60 pairs (each pair is a hitch: they're read back on the game's thread) |
| `RUSTVR_OBLIVION_LEAVE_OUT=1` | with `RUSTVR_OBLIVION_EYE_PNG`: every 5 s one picture frame leaves out one kind of draw, in turn (far land, grass, leaves, branches, distant trees, shadow receivers, skinned bodies, the rest, draws without a shader) |
| `RUSTVR_OBLIVION_PROFILE=1` | the game's thread sampled about once a millisecond; each frame over 30 ms summed up in the log (its module, the game's addresses most often on its stack) |

## Decisions

- **Every fix is on, each with a switch off for a run.** They fix what the game makes once from one
  camera; the switches exist to compare, not as player options.
- **The game's settings are floors, only while in VR.** Your own higher values are kept and all are
  put back at close, so the flat game is as you left it.
- **`uGridsToLoad` isn't a VR floor**: saves made at 7 are said to crash loaded at 5, so it can't
  change between VR and flat play.
- **Grass to 57 m, not 86.** All round you at 86 m it was the heaviest part of the frame.
- **The loader's lock is given back after each eye.** The thread keeps up with the cells ahead; the
  cost is that a thing it adds between the eyes can show in the second eye a frame before the first.
- **The actors' shadow pass runs once a frame.** Run again for the second eye it left actors hidden
  from it; the maps are the same for both eyes.
- **The weather is centred on the head, the sky on each eye.** The sky is at infinity; rain and snow
  are near and keep their depth.
- **The leaves' index is made whole in the shader.** Given the whole number, no driver's rounding
  can move it.
- **The threaded particles and temporary effects stay as you have them** (not floors): those are the
  ones reported to crash.
- **The eye pictures are off by default.** Each pair cost a frame of 140..160 ms.

## Not done / known issues

- **Hitches crossing cells remain**: in round 11's run, 40..56 ms every 20..30 s while moving, the
  game attaching newly loaded cells and drawing new things the first time. Run with
  `RUSTVR_OBLIVION_PROFILE=1` to see what code they're in; that run's result isn't recorded.
- **Which shadows show, and how far away they fade,** are still worked out once a frame from the
  first eye: a shadow at the very right edge of the right eye may be missing.
- **The water's reflections resetting as the head moves** (the research's other VR item): not done.
- **Ghosting on moving people**: not in the game's pictures (saved before the runtime moves them).
  It's after: the runtime's smoothing, Virtual Desktop's spacewarp or the frames the game misses at
  120 Hz; not yet told apart.
- **The 250 MB preload floor** was chosen on the developer's 4GB-patched install; without the patch
  the game has 2 GB of address space, and it isn't checked there.
- **The far land's colours** are the game's own. TES4qLOD's colour maps at 2048 or 4096 aren't
  sharper: it colours from one 4x4-pixel average of each ground texture, so above 1024 it only blurs.
- **Distance and LOD settings** beyond the floors are left as yours: they cost draw calls twice over
  in VR (the research has them).

## History

Newest first.

- **On main, after round 12:** fog by distance, so both eyes haze a far hill alike.
- **Round 12:** the streak gone; the remaining hitches were cells coming in. Added the sampling
  profiler (now behind `RUSTVR_OBLIVION_PROFILE=1`).
- **Round 11:** the leave-one-out run showed the streak gone only without the leaves: the leaves'
  rounded index (D3D9On12 rounding to the nearest). Fixed by making the index whole.
- **Round 10:** neither shadow pass was it; a run left out each kind of draw in turn
  (`RUSTVR_OBLIVION_LEAVE_OUT`).
- **Round 9:** the leaves no longer turn or lean with the head. The hitches were cells attaching and
  first draws; the game's own background loading and threading turned on in VR.
- **Round 8:** the frame rate fell to 75..90 from 110..135 with the longer distances; grass back to
  57 m from 86. Two runs (canopy shadows off, people's shadows off) to tell which pass the streak was.
- **Round 7:** pop-in: the 7x7 grid on the developer's `Oblivion.ini`, longer fades as VR floors. A
  240 ms hitch was Stutter Remover's kept texture lock: suppressed. System memory draws widened.
- **Round 6:** the 140 ms hitches every 10 s were the eye pictures: off. xOBSE (and Stutter Remover)
  loaded when the adapter starts the game. The streaks were the leaf cards; their constants checked.
- **Round 5:** the loader's lock given back after each eye; the game's settings raised for a
  headset; the far trees face the eye; grass all round; anisotropic filtering back on.
- **Round 3:** far detail flickered: the runtime averages the game's picture over each headset
  pixel. The distance was mushy: anisotropic filtering, 16x.
- **Round 2:** people's shadows were in the left eye only: clip planes made again for each eye.
- **Round 1:** the sky had parallax and bobbed: centred on each eye, the weather on the head. The
  right eye drew bodies where the left eye's camera saw them: a renderer frame for each eye.
