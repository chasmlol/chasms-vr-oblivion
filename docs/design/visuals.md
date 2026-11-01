# What the game makes once a frame, and the eyes

Each eye is Oblivion's own world render, run once per eye with the world camera put at that eye
([`crate::render`](../../crates/oblivion/src/render.rs)). The adapter writes only the camera's world
transform and frustum, so anything the game takes from somewhere else, or makes once a frame and
keeps, is the same for both eyes. Where that's wrong, it shows as the picture not fusing: a thing at
the wrong depth, or swimming as the head moves. This is the list of those things in Oblivion, with
what the adapter does about each.

## Per eye and per frame, today

| | Made | How |
|---|---|---|
| The camera (world transform, frustum) | each render | written by the adapter |
| The renderer's camera data (view, projection, the world offset) | each render | set from the camera when the scene is drawn |
| Culling, LOD of what's drawn | each render | the culler takes the camera |
| Fog, specular (`EyePosition`) | each render | the shaders take the renderer's camera |
| Water reflections | each render | the world render updates the water with the camera; the reflection is rendered from that camera mirrored in the water, with the sky moved to it |
| The sky and the weather's centre | each render | copied from the game's camera node: see below |
| A skinned model's bones | once a renderer frame | see below |
| The actors' shadow maps, which actors cast them, their fade | once a frame (first eye) | the game's shadow pass, held to the first eye by the adapter: the maps are rendered from the lights, the same for both eyes |
| The clip planes of an actor's shadow | once a frame, made again for each eye | see the actors' shadows |
| A render of the sky on a timer (at the end of the shadow pass, with the world camera) | once a frame (first eye) | as the game does it once a frame |
| Animation, particles, the sky's colours and the clouds' scroll, the weather | once a frame | the game's update, before the render |

## The sky swam and bobbed

**Observed.** At its start the world render copies the position of the game's camera node
("WorldRoot CameraNode", moved by the player's camera) into the "Sky" node and the "Weather" node,
and updates both. The dome, the clouds, the sun, the moons and the stars hang under "Sky", the sun's
glare and the precipitation under "Weather". The sky's shaders (`SKY.vso`, `SKYCLOUDS.vso`) put the
sky on the far plane but keep its model-view: a dome not centred on the eye shows parallax.

**Cause.** The camera node is the game's camera, and each eye is somewhere else: the runtime puts
the eyes where the head is, which differs from the game's eye by the room-scale offset, the head's
real height and the smoothing of the game's eye over steps (`framework/crates/adapter/src/tracking.rs`).
So the dome and the clouds were centred off the eyes. Each eye saw them with a different parallax
(the sky at a few tens of metres, not at infinity), and as the game's camera stepped and the eye
was smoothed, the dome moved against the eye: the clouds swam and bobbed while walking.

**Fix.** The two copies take their node from the adapter (`render::SKY_FROM`, `WEATHER_FROM`)
instead of the camera node. For a single render they hold the camera node, as the game has it. In
the eyes:

* the sky is centred on each render's camera (each eye, and the mirror's camera): it's at infinity
  for both eyes and turns only as the head turns;
* the weather is centred on the head (between the eyes), one place for both eyes. Rain and snow are
  drawn in a box around the weather's centre, so centred per eye they'd have no depth. The sun's glare is on the same
  node; its parallax from the head is the eyes' half width against the sky's distance.

The water's reflection moves the sky to its own mirrored camera before the copy, so it needs
nothing.

## The NPCs' bodies

**Observed.** The renderer makes a skinned model's bone matrices relative to the camera's position
(the renderer's world offset, set to the camera's position for each render), and keeps them for the
rest of the renderer's frame, which moves on once a game frame. The geometry of a skinned model is
kept the same way.

**Cause** (in playtest 1, with the fix, the bodies match in both eyes). Both eyes and the mirror are drawn in one renderer frame, so the bones were made once,
relative to the first eye's camera, and the second eye and the mirror drew them relative to theirs:
in the right eye a body (skin, clothes, armour) was the eyes' width aside, as if seen from the left
eye. The head, the hair and what an actor holds are rigid models, drawn where they are. In the
mirror, the bodies were off by the distance between the first eye and the mirror's camera.

**Inferred, not checked:** the game's own shadow pass, which draws the actors from the lights before
the first eye, would make the bones relative to a light if it went through this path; it can't, or
the game's own picture would be wrong, so the first eye's are as the game makes them.

**Fix.** Each render after the first is a frame of the renderer's own: the adapter moves the
renderer's frame id on (`engine::next_render_frame`), without ending the scene. The bones (and the
skinned geometry) are made again, relative to that render's camera.

## Actor shadows

**Observed.** The game's shadow pass picks the casters (the player and up to
`iActorShadowCountExt`/`Int` actors near), places a shadow light for each from the lights around it
(camera-free), tests the shadow against the camera's frustum and fades it by its distance from the
camera, and renders each map from its light. The map's projection is kept on the light (from the
light's position: camera-free). Receivers take it per draw, so it's right in each eye as long as the
object's world is. The pixel shaders that take it (`SLS2076`, `SLS2077`, with `SLS2073`) work in the
object's space. The maps are the same for both eyes, so the adapter runs the shadow pass once a
frame (run twice, it left actors hidden from the second eye).

**Seen** (playtest 1, a rider on the road, both eyes' pictures of one frame): the horse's shadow
is under it in the left eye, and in the right eye there's none. The body is in the same place in
both.

**Observed, the cause.** An actor's shadow on what it falls on is clipped to the shadow light's
frustum with six user clip planes (`SetClipPlane`), which Direct3D 9 takes in the camera's clip
space. The first draw of a frame puts the light's planes through the current camera's view and
projection and keeps them on the light; the rest of the frame's draws take the kept ones. Only the
light's map render makes them again, and that's once a frame, with the first eye. So the second eye
(and the mirror) clipped every actor's shadow with planes made for the first eye's clip space, which
cut it away.

**Fix.** Before each render after the first, the adapter has the game make the planes again on each
of the actors' shadow lights, for that render's camera.

The tree canopy's shadows (`ShadowProj`, `ShadowProjData`, `ShadowProjTransform`: the canopy
texture tiled over the world and the cell's canopy mask) are in world space.

**What's left camera-dependent**, the same for both eyes since it's done once: which shadows show
(tested against the first eye's frustum, so a shadow seen only at the right edge of the right eye
may be missing) and their fade with distance. Not changed.

## The rest

| | Per eye? | Notes |
|---|---|---|
| Water reflections, refraction | yes | rendered inside each eye's render from its camera |
| Fog | yes | per vertex; the game's shaders take the eye's clip-space distance, the adapter's rewrite the view distance (`fog.rs`) |
| Specular | yes | `EyePosition` per draw, put in the object's space |
| Billboards (the sun, grass, particles' facing) | yes | not checked: Gamebryo turns a billboard to the culling camera, which is each eye's |
| Trees (SpeedTree leaves and LOD billboards) | yes, as the adapter turns them | the leaves' cards are given a facing per draw from each eye (below); the distant trees' cards face each eye's position (below) |
| Distant LOD (land, objects, trees) | not checked | chosen from the player's cell and position in the update, the same for both eyes |
| Culling | yes | from each eye's frustum |
| HDR's eye adaptation | each render | image space runs at the end of each eye's render. HDR is off in this player's `Oblivion.ini` (`bDoHighDynamicRange=0`, bloom on). With HDR on, the second eye may adapt on from the first; not checked |
| Precipitation | once a frame | drops wrap in the weather's box (see the sky) |
| Animation, particles' simulation | once a frame | the same for both eyes, as it should be |

## The far distance: flicker and blur

**Seen** (playtest 3): far things "flicker or flash, not like aliasing"; the middle distance is
"blotchy, off, blurry, mushy". The game's own pictures (saved before the runtime shows them) are
steady and sharp; there's no anti-aliasing in the game (`iMultiSample=0`).

**The flicker: the runtime's sampling.** The game draws each eye at the runtime's render scale (the
playtest's 1.66: 5099x5418, more pixels than the headset's image). The runtime's warp (`warp.rs`)
and its blit (`compositor.rs`, `PSBlit`) took one bilinear tap of the game's picture per headset
pixel, at its only mip level: the pixels between taps were skipped, not averaged. The supersampling
bought no smoothing, and fine far detail (branches, fences, roof edges, the far distance) fell in
and out of the taps as the head moved: a flicker that isn't the game's aliasing. Both now spread
taps over the pixel's footprint in the picture (2x2 up to twice the headset's sharpness, up to 4x4),
as a mip level would; one tap at the headset's own sharpness or below, as before.

**The blur: no anisotropic filtering.** Oblivion's texturing has no anisotropic mode
(`NiTexturingProperty`'s filters stop at `FILTER_TRILERP`); on a desktop it's forced in the graphics
driver, which doesn't reach a game on D3D9On12 (Direct3D 12 underneath). Trilinear filtering alone
blurs a surface seen at a slant (the road ahead, the ground, a long wall) a few metres off. The
adapter hooks `IDirect3DDevice9::SetSamplerState` (slot 69): a sampler the game sets to minify
linearly minifies anisotropically, 16x (`anisotropy` in `rustvr-oblivion.toml`; testing,
`RUSTVR_OBLIVION_ANISOTROPY`). A picture of the game's own sampled a texel to a pixel (its image
space passes) has a round footprint and comes out as it did.

**Not ours:** the game's 24-bit depth between its near (10 units) and far (283840) planes, the same
as on a desktop. Settings for the distance (LOD, grids, fades) are the game's: see the research note.

## Streaks across the trees (found and fixed: round 11)

**Seen**: playtest 4, the forest at Tamriel -14, 16 (x -55698, y 66703), looking south-east; playtest
6, Weye, looking across the lake at the trees on the far shore. Long thin near-horizontal streaks of
foliage from trees in the middle distance to the edge of the picture, "like a classic broken texture
stretching out". They're in the game's own pictures (before the runtime), in one eye or both: at Weye
one picture has them in the left eye only, the next in both. Seen before the anisotropic filtering
was on.

**What draws them** (from the pictures): the leaves' cards (`STLEAF000..003.vso`), dozens of them
fanning out from the trees, in half-transparent leaf texture.
- A leaf corner is the leaf's position plus its cluster's corner (`LeafBase`) scaled, turned by the
  rocking (`RockParams`, `RustleParams`) and laid along `BillboardRight` and `BillboardUp`, then
  pulled by the wind matrices (`WindMatrices`) by the leaf's wind weight.
- The wind matrices are the same for every tree, so a broken one would streak every tree, not those
  at one place.
- So, inferred: a per-tree constant (the corners or the rocking) that isn't finite or is far out of
  range, at the distance those trees are from the eye. Or a cluster index past the corners uploaded
  for the tree.
- One Direct3D difference matters. The shader's last step multiplies the wind's pull by the leaf's
  wind weight. On Direct3D 9, 0 times anything is 0; D3D9On12 follows IEEE,
  where 0 times NaN is NaN.

**Watched** ([`crate::leaves`]): the leaf shaders are known from their constant tables when the game
makes them. While one is bound, each constant the game sets in those ranges is checked:
- One that isn't finite is made 0 (the leaf collapses instead of streaking) and logged.
- One far out of range is logged.
- Every 10 s the log sums up what they held: the largest of each, and how many corners each tree
  uploads.

Testing, `RUSTVR_OBLIVION_LEAVES_AS_GAME=1`: unchecked.

**Playtest 7:** about 800,000 uploads to leaf shaders every 10 s, none non-finite, every tree
uploading all 48 corners. The largest corner was 138, the billboard vectors 1, the wind matrices 1,
and the rocking and rustle phases 46 and 27, growing with time as phases do. So not the constants.

**Playtest 7's draws:** the game made no vertex buffer in system memory at all (2,269 managed, 8
default), so not that either. The playtester: the streak moves as the head moves, up and down or
side to side, and doesn't look like it's in the world. The runtime's reprojection is off in their
settings (`reproject = false`), so it's the game's.

**Inferred, round 8: a shadow pass.** In the eye pictures the band is ground texture (grass and dirt
colours) in horizontal stripes with dark lines, on the hillside. The game's shadow receivers (the
`SLS2003`-family shaders, and `STB2001` on trees) draw the ground again with a projected shadow
texture (`ShadowProj`, `ShadowProjTransform`), its coordinates divided by the projection's w for
each vertex. Where the ground crosses that projection's w = 0 the coordinates go off to infinity
across a triangle, and the texture smears into stripes. Six clip planes per shadow normally cut
that away; each eye sees the edge differently. A second set (`ShadowProjData`) is likely the trees'
canopy shadows (`bDoCanopyShadowPass`, the texture `Trees\CanopyShadow.dds`). Two playtest runs tell
which, one with canopy shadows off and one with actor shadows off, through a new testing knob,
`RUSTVR_OBLIVION_INI_TEST` (`Section.key=value;...`, put back on close like the rest).

**Playtest 9:** neither run cleared it: canopy shadows off (`[Display] bDoCanopyShadowPass=0`,
applied at 19:56:24) and people's shadows off (`bDoActorShadows=0`, both actor shadow counts 0,
19:56:57). So not those passes.

**Round 10, leave one out** ([`crate::families`]): with `RUSTVR_OBLIVION_LEAVE_OUT=1` and the eye
pictures on, every 5 s one frame's eyes leave out one kind of draw, in turn, and are saved as
`<n>-without-<kind>-<eye>.png`. The kinds are known by their vertex shaders' constants: far land,
grass, leaves, branches, distant trees, shadow receivers, skinned, other, and no shader. The draws
are skipped in `DrawPrimitive`, `DrawIndexedPrimitive` and both `UP` forms.

**Found, round 10's pictures** (Weye, 20:02, the streak kept in view): in the frame without the
leaves (`024-without-leaves`) it was gone from both eyes. In every other kind's frames of the same
view it was there: nothing out, far land, branches, distant trees, shadow receivers, skinned.

**Cause:** the leaf shaders (vs_1_1) index their constants (the leaf's corner, of 48, and its wind
matrix) straight from vertex data, and the corner's number carries a shade in its fraction. They
were written for the drivers of their day, which loaded the address register rounding down.
Microsoft's reference for vs_1_1 says round to nearest
([Address Register](https://learn.microsoft.com/en-us/windows/win32/direct3dhlsl/dx9-graphics-reference-asm-vs-registers-address)),
and D3D9On12 does. With a fraction of .5 or more, a leaf took the next cluster's corner, and the
last cluster (47) read past the corners: another draw's leftovers, different for each eye, a corner
thrown across the view. The leaves' constants themselves were all sound (playtest 7).

**Fix** ([`crate::leaves::floor_indices`], in the `CreateVertexShader` hook): in every vs_1_1 shader
each `mov a0.x, v#.c` becomes `expp r11.y, v#.c; add r11.x, v#.c, -r11.y; mov a0.x, r11.x`. The index
is then whole, and any rounding gives the corner the game meant. `vs_2_0` and up use `mova`, round
to nearest by design, and are left as written. Checked with `D3DDisassemble` on `STLEAF000.vso`:
the six instructions where the two `mov`s were, 99 slots of vs_1_1's 128. Testing,
`RUSTVR_OBLIVION_GAME_LEAF_INDEX=1`: as written.

**Earlier suspect** ([`crate::sysmem`]): D3D9On12 copies a `D3DPOOL_SYSTEMMEM` vertex buffer to the GPU
for each draw, and only `MinVertexIndex + NumVertices` vertices from `BaseVertexIndex`
(`InputAssembly::UploadDeferredInputBufferData`). Direct3D 9's drivers read the buffer whole, so a
draw whose indices reach past its hints, harmless there, reads stale upload memory here. Its loop
over streams 1..15 also steps twice a pass, so a system memory buffer on stream 2, 4 or 6 is never
copied. The adapter widens those draws' hints to the whole buffer (up to 2^18 vertices) and counts
them every 10 s (testing, `RUSTVR_OBLIVION_DRAWS_AS_GAME=1`).

## The leaves turned with the head

**Seen** (playtest 8): "the tree leaves rotate with my head" as it leans and turns.

**Why:** the leaf cards (`STLEAF000..003.vso`) are laid along `BillboardRight` and `BillboardUp`,
the camera's right and up in the tree's own space: square to where the view looks, and leaning with
it. A flat screen's camera never leans and turns little. A head leans and turns all the time, and
every card on every tree turned with it.

**Fix** ([`crate::leaves`], at each leaf draw, through the `DrawIndexedPrimitive` hook):
- the eye in the tree's own space is solved from the draw's `ModelViewProj` (where it takes x, y
  and w to 0, which holds for any perspective, off-centre too);
- the cards' right is made level and square to the line from that eye to the tree's origin;
- their up is the game's with its lean toward that right taken out, both as long as the game's.

The two registers are set straight on the device for the draw and the game's own put back after it
(c13 is another shader's `LightDirection`). Testing, `RUSTVR_OBLIVION_GAME_LEAF_FACING=1`: the
game's. The distant trees' cards already face the eye (round 5).

## The ghosting on moving people

**Seen** (playtest 3): "a weird ghosting outline on moving stuff like characters". **Not in the
game's pictures** (saved before the runtime moves them). What comes after: the runtime's warp
(every headset frame moves the latest picture by its depth to the head's pose then; a near edge
over a far background is refilled from the other eye's picture), Virtual Desktop's own spacewarp,
and the frames the game misses (at 120 Hz it gives 103..117 pictures a second at this resolution, so
some headset frames show a picture again, its people where they were). Not yet told apart: the
playtest has the tests.

## Stutter crossing cells

**Observed.** The game's background thread (cells' and actors' 3D) changes the scene only while it
holds a lock the world render takes at its start (a 3 ms try, then the render marks it wanted and
waits) and gives back at its end. **Derived** (the adapter's
timing log): the two eyes and their picture and depth copies held it 6..12 ms of each 9 ms frame,
and 1.5 ms was left for the thread, where a desktop's one render leaves it most of the frame.
**Inferred, not measured:** the thread fell behind with the cells ahead, and the game stood waiting
for it, most of all crossing into new cells.

**Fix.** Each render takes and gives the lock, as the game's one render does: the thread can work
while each eye's picture is copied. A thing it adds then can show in the second eye a frame before
the first. The game's waits for the lock on its own thread are timed ("waits for the loader" in
the timing line) and any over 20 ms is logged, to see the stutter in the log. Testing,
`RUSTVR_OBLIVION_HOLD_LOADER=1`: held across the eyes, as before.

Not ours: the engine loads new cells' geometry largely on its main thread; the 2 GB address space
(this exe isn't large-address-aware); the community's mods (Oblivion Stutter Remover's heap, Oblivion
Display Tweaks' timer, which the adapter's clock already does): `oblivion-image-quality.md`.

**What the later playtests ran with** (from 2026-10-30, at the playtester's request; none of it is
the adapter's, which checks for none of it):
* the 4GB patch on `Oblivion.exe`;
* xOBSE 22.14, which Oblivion Stutter Remover needs;
* Oblivion Stutter Remover 4.1.37, set for VR (`oblivion-image-quality.md` §6b: no frame limiter,
  timer or fast exit, its heap at 512 MB);
* the distant objects regenerated (xEdit's TES4LODGen) into `Data\DistantLOD`, keeping only the tree
  kinds the game's own lists draw far away. Unfiltered, it doubled the far cards near the Imperial
  City with shrubs (`oblivion-image-quality.md` §4.2);
* the far land's meshes regenerated (tes4ll) for the six quads the game's cells cover completely, with
  the game's skirts and sea floor; the rest, and the far land's colour, stay the game's
  (`oblivion-image-quality.md` §3.1).

**Playtest 7** (Weye, 19:18), xOBSE and Stutter Remover loaded:
- Frames mostly 10..25 ms at their worst, 110..135 frames a second (the headset at 120 Hz wants 8.3 ms).
- One 239 ms frame ("eye renders" 216). Stutter Remover's log at that time (100 s in): up to a dozen
  threads waiting 5..26 ms each on one critical section, the renderer's texture data lock, which was
  kept as a lock (mode 3). It's suppressed now (mode 5, Stutter Remover's own default). Without it two threads can both make one texture's
  data; Stutter Remover's users have run it so for years.

**Playtest 8** (19:48): 72..108 frames a second. The hitches, 40..120 ms every 10..30 s while
moving: "between renders" (the game attaching loaded cells: 51..79 ms) and "eye renders" (things
drawn the first time: textures made, D3D9On12's pipeline states built: 30..103 ms). Stutter Remover's
lock waits were all short (2..11 ms, a renderer lock taken by its background threads). The player's
`Oblivion.ini` had the game's own threading off: `[BackgroundLoad] bLoadBackgroundFaceGen` and
`bBackgroundLoadLipFiles`, `[General] bUseThreadedMorpher` and `bUseThreadedBlood`, `[Pathfinding]
bBackgroundPathing`. They're VR floors now. `bUseThreadedTempEffects` and
`bUseThreadedParticleSystem` stay off (reported to crash).

**Playtest 11** (20:21, the streaks fixed): 80..100 frames a second, hitches of 40..56 ms every
20..30 s while moving, "between renders" and "eye renders" spiking in the same windows: cells
coming in. The game's frame makes some 90 calls, too many to time one by one. So a sampling
profiler ([`crate::profile`], `RUSTVR_OBLIVION_PROFILE=1`) stops the game's thread about once a
millisecond and notes where it is and the calls on its stack. Each frame over 30 ms is summed up:
its module, and the call sites most often on the stack.

**Pop-in.** The playtester raised `uGridsToLoad` from 5 to 7 in their own `Oblivion.ini`: saves made
at 7 are said to crash at 5 (`oblivion-image-quality.md` §4.1), so it can't be a VR-only value, and
the adapter leaves it alone. The fades, grass and far LOD are VR floors (`game_settings.rs`).

**Playtest 6** (Weye, 18:33):
- The game's waits for its loader were 0 ms throughout.
- Its 140..160 ms frames came every 10 s, each one a pair of eye pictures. They're read back on the
  game's thread, so the playtests leave them off now (`RUSTVR_OBLIVION_EYE_PNG` turns them on).
- The rest: a few 50..200 ms frames as new land came in (in "eye renders" and "between renders").
- Neither xOBSE nor Stutter Remover ran. Steam loads `obse_steam_loader.dll` only into a game it
  starts, and the adapter's launcher starts the exe itself. The adapter now loads it at the same point
  ([`crate::xobse`]).

## The game's own settings, for a headset

While in VR the adapter raises a few of `Oblivion.ini`'s values to a floor (`game_settings.rs`,
`VR_FLOORS`), keeping the player's own higher ones and putting them all back when the game closes:
the actors' shadow maps to 1024 (256 are blocks at arm's length), the trees' mip bias to 0 (the
game's -0.75 and -0.25 sharpen them into sparkle as the head moves), near trees at full detail
(`bForceFullLOD`), the water's waves at 36 frames a second over the same 2.7 s loop (12 step at
90..120 Hz; 96 frames), the cells ahead preloaded up to 250 MB (`iPreloadSizeLimit`, chosen on a
large-address-aware exe), the view distance sliders' objects, items and actors at 15, trees at 2
(`fTreeLODMax`), the grass to 57 m (its `[Grass]` keys), the distant buildings and tree cards at
40 and 30 cells, and the game's own background threads for faces, lip files, the morpher, blood
and path finding (playtest 8). The full table is `oblivion-image-quality.md` §9. Testing,
`RUSTVR_OBLIVION_FLAT_INI=1`: the player's as they are.

## The distant trees turned with the head

**Observed** (`DISTLOD2003.vso`): each instanced card of the distant land's trees is turned square
to the horizontal part of `EyeDir`, the camera's look direction, one for every tree. A head's turn turned every far tree: the far forest shimmered.

**Fix** (`distant_trees.rs`). The adapter hooks `IDirect3DDevice9::CreateVertexShader` (slot 91):
the shader whose constant table has `EyeDir`, `InstanceData`, `AlphaParam` and `DiffuseDir` (in the
player's package, `DISTLOD2003` alone of 329) is made from the adapter's HLSL instead, compiled for
`vs_2_0`: the same registers, lighting (the normal and shade packed in the instance's fractions),
fog and distance fade, but each card turned to the horizontal direction from `EyePos` to its tree.
Testing, `RUSTVR_OBLIVION_GAME_DISTANT_TREES=1`: the game's own.

## Grass only in front

**Observed:** the game builds its grass again when its camera has moved 64 units or turned far
enough, and keeps only the 512-unit patches near the player or in a cone ahead. At the flat game's
field of view that's 46 degrees either side, rebuilt after 19 degrees of turn: in a headset the
grass came and went at the edges of the view. It's the game's one camera, not the eyes'.

**Fix** (`grass.rs`): the build keeps every patch, to the game's own grass distance (the cells and
quadrants past it are dropped before), and a turn no longer builds the grass again, since it no
longer changes what's built (a move of 64 units, and a forced build, still build again).
About three times the patches are built near the player. Testing, `RUSTVR_OBLIVION_GAME_GRASS=1`:
the game's cone.

**Seen in the player's `Oblivion.ini`:** `fGrassEndDistance=7000` and `fGrassStartFadeDistance=6000`
are under `[Display]`, where the game doesn't read them; its `[Grass]` 3000 and 2000 are the ones in
use (43 m).

## Next, from the research

Web research on the game's own image quality (sources, settings, mods):
[oblivion-image-quality.md](oblivion-image-quality.md). Of what in it is this bug family (made once
from one camera, or from the flat screen's view), nothing is left to do:

* **Near trees' leaf cards**: OBVR's note on a leaf batch reusing the first tree's basis is about
  its own leaf-facing hook; the game's own leaves take the camera's up and right, the same for every
  tree. The adapter gives them a facing of its own per draw (above).
* **The frame's time step**: inside the world render only the actors' shadow pass reads it, which
  runs once a frame; the animation runs in the frame, before the render. OBVR's
  "double tick" is from rendering more of the frame twice, not the case here.

What's not built from it is listed there (§9).

## Testing

* `RUSTVR_OBLIVION_MONO_VISUALS=1`: the sky and the weather on the game's camera and one renderer
  frame for all the renders, as before, to see what the fixes do.
* `RUSTVR_OBLIVION_EYE_PNG=<folder>`: both eyes' pictures (the world, before the HUD, at most 2048
  wide) are saved there every 10 seconds from 5 seconds in, up to 60 pairs.
* `RUSTVR_OBLIVION_HOLD_LOADER=1`, `RUSTVR_OBLIVION_GAME_DISTANT_TREES=1`,
  `RUSTVR_OBLIVION_GAME_GRASS=1`, `RUSTVR_OBLIVION_FLAT_INI=1`, `RUSTVR_OBLIVION_ANISOTROPY=1`: round
  5's changes off, one each.
* The adapter's log says where the sky is from each eye every 600 frames ("the sky's centre is ...
  units from the eye"): 0 with the fix.
