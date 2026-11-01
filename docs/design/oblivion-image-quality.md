# Oblivion (2006, Steam): image quality, its settings and mods

Web research for the VR adapter (dinput8.dll injection, the game's D3D9 device on D3D9On12,
each eye a full game render of about 5100x5400 in the playtests, no MSAA). It covers renderer
behaviour and ini/engine fixes, not texture or art replacements. Written 2026-10-30. What the
adapter took from it, and what it didn't, is §9.

Confidence tags: **[C]** primary source (mods' and tools' source, their notes, official docs);
**[K]** wiki or forum knowledge; **[I]** my inference or arithmetic; **[U]** unconfirmed (no
source, or only a search summary of a page that blocked reading: Nexus, Reddit, GamesAS).
Sources are [S#], listed at the end.

---

## 0. Likely causes of what was seen in the headset

| Observation | Most likely causes, best first | Section |
|---|---|---|
| Distant things flicker or flash, not like ordinary aliasing | (1) Distant LOD tree cards are turned toward the camera's *view direction*, so they turn with the head and can differ between the eyes. (2) Alpha-tested cards (leaves, grass, LOD trees) with a negative mip LOD bias and no AA. (3) Depth precision: near 10, far 283,840, 24-bit depth, so water, LOD land and LOD objects z-fight beyond about 1 km. (4) Water LOD flickers at shorelines. (5) Per-render state that each eye advances separately | §2 |
| Scenery "not too far away" looks blotchy and mushy | (1) Full-detail land ends where the loaded grid ends, about 117–175 m away at uGridsToLoad=5. Beyond that is LOD land with 1024² colour maps per 32x32-cell quad (about 1.8 m per texel) and DXT1-compressed normal maps that show dark blocks. (2) No anisotropic filtering, and some samplers use point mip filtering | §3 |
| No anti-aliasing | With HDR on, the engine forces in-game MSAA to 0. Driver overrides most likely do not reach a D3D9On12 device | §5 |

---

## 1. Oblivion.ini quick reference

The file is `Documents\My Games\Oblivion\Oblivion.ini`. The game rewrites it on exit, and
the in-game Video menu rewrites some keys: it resets `fDecalLifetime`, for example [S19][S20].
Make edits with the game closed, and do not open the Video menu afterwards [K].

| Key [section] | Default | Recommended | Why / caveat | Src |
|---|---|---|---|---|
| iTexMipMapSkip [Display] | 0 | **0** | The in-game "Texture Size" setting (presets: High/Ultra 0, Medium 1, Low 2). Anything above 0 drops the top mip of every texture | S1 S2 S9 |
| iTexMipMapMinimum [Display] | 0 | 0 | No Oblivion source explains it. A Skyrim test found that 6 lowered quality, which suggests it limits how far textures may drop [U] | S2 S8 |
| (anisotropy) | none | n/a | **Oblivion has no AF key.** `iMaxAnisotropy` is not a valid Oblivion setting. Force AF externally (§3.2) | S8 S19 |
| iMultiSample [Display] | 0 | 4 or 8 **only with HDR off** | In-game MSAA. The effective value is 0 while HDR is on (§5.1) | S2 S19 S27 |
| bDoHighDynamicRange [BlurShaderHDR] / bUseBlurShader [BlurShader] | preset | one or the other | HDR uses an FP16 render target and eye adaptation. Bloom is a cheaper blur. With Bloom, some setups turn distant LOD black | S1 S13 S17 |
| fNearDistance [Display] | 10.0 | **10–15; never 0 or 0.0001** | The near clip plane: 10 units = 14 cm. Old first-person-clipping advice sets 0 or 0.0001, which destroys distant depth precision. Raising it helps distant z-fighting but clips hands and props near the face (§2.3) | S2 S27 S35 |
| uGridsToLoad [General] | 5 | **5**, or 7 on a new save | The number of cells loaded at full detail. Must be odd. Saves are not portable between values, and there are other pitfalls (§4.1) | S7 S10 S20 S38 |
| uExterior Cell Buffer [General] | 36 | (uGridsToLoad+1)² | 64 for 7, 100 for 9. Reported to crash if left too small [K] | S7 S20 |
| iPreloadSizeLimit [General] | 26214400 | 104857600 if you raise grids | Developer-sourced tweak from 2006 | S7 S20 |
| uGridDistantCount [General] | 25 | 25–50 | How far LOD buildings and ruins are drawn. The developer-sourced list says doubling it "looks quite nice" | S9 S20 |
| uGridDistantTreeRange [General] | 15 | 15–30 | The LOD tree range. Without fog, the world "drops off suddenly" at this distance | S9 S20 |
| fLODFadeOutMultObjects / Items / Actors [LOD] | 5 / 2 / 5 | Raise (objects 15+) | The in-game fade sliders. Most likely only affects objects inside loaded cells [U] | S1 S3 |
| fLODMultTrees [LOD] | 1.2 | 1.3–1.8 (High/Ultra presets) | The in-game "Tree Fade" slider | S1 S9 |
| bForceFullLOD [SpeedTree] | 0 | 1 | Draws near SpeedTrees at full detail. Developer-sourced | S4 S20 |
| fLODTreeMipMapLODBias [SpeedTree] | **-0.75** | try 0 | A negative mip bias on tree textures: sharper, but more shimmer (§2.2) [I] | S4 S31 |
| fLocalTreeMipMapLODBias [SpeedTree] | -0.25 | try 0 | As above, for near trees [I] | S4 |
| fGrassEndDistance / fGrassStartFadeDistance [Grass] | 3000 / 2000 | Raise as performance allows | 3000 units ≈ 43 m. In VR the grass "cone" causes its own pop (§2.6) | S6 S27 |
| fSpecularLOD1/2 [Display] | 500 / 800 | Optional raise | The "Specular Dist." slider. Specular fades between 7 and 11 m, which shows as a ring | S1 S2 |
| fLightLOD1/2 [Display] | 1000 / 1500 | **≤ 4500** | Values well past 4500 put large coloured blobs over LOD objects | S9 |
| fNoLODFarDistancePct [Display] | 1.0 | 1.0 | The "View Distance" slider | S1 S2 |
| bAutoViewDistance [Display] | 0 | **0** | Undocumented. By its name and its iAutoViewHi/LowFrameRate keys, it changes view distance at runtime, which would read as pop [I] | S2 |
| bFullBrightLighting [Display] | 0 | **0** | "Causes severe glitches" | S2 |
| bDisplayLODLand / Buildings / Trees [LOD] | 1 | 1 | Distant land, buildings and trees. Setting LODTrees to 0 is a useful A/B test for §2.1 | S1 S3 |
| bUseRefractionShader [Display] | 1 | 1 | Off removes "most of the particle effects for Oblivion Gates and summoned creatures". Only needed off on old cards with AA | S11 |
| iShadowFilter [Display] | 0 | 2 | Shadow edge filtering: 0 off, 1 low, 2 high | S2 |
| iShadowMapResolution [Display] | 256 (STEP) | Raise (1024 is common [U]) | Negative values crash | S2 |
| bActorSelfShadowing [Display] | 0 | 0 | Actors shadowing themselves. The 3D Vision fix also turns it off | S2 S29 |
| bShadowsOnGrass [Display] | 0 | 0 | STEP recommends 0 | S2 |
| uSurfaceFrameCount / uSurfaceFPS [Water] | 32 / 12 | 96 / 36 (VR test) | The 12 fps wave animation steps visibly at 90 Hz (§2.5) [K] | S5 S27 |
| fSurfaceTileSize, fTileTextureDivisor, uNumDepthGrids, uDepthRange [Water] | 2048, 4.75, 3, 125 | **Leave alone** | Changing them causes gaps or seams in the water | S5 |

---

## 2. Flicker, shimmer and flashing at distance

### 2.1 Distant LOD tree cards face the view direction (VR-specific) [C]
- **Cause.** The distant land's trees are drawn instanced by `DISTLOD2003.vso` from the shader
  packages. Each quad is turned to stand across the horizontal part of `EyeDir`, which is the
  camera's *view direction*, not its position. Turning the head
  turns every distant tree by the same angle. Each eye's render has its own EyeDir, so with
  canted per-eye view directions the two eyes see the cards at different angles, which is
  binocular sparkle [I]. A single eye's picture hides this: a card kept square to the view
  never foreshortens [S27 leaf-batch-fix.md].
- **Near trees have a separate bug.** SpeedTree leaf cards are batched. The leaf loop reuses the
  camera basis it set up for the first tree, so a tree at the edge of the view can turn every
  other tree's cards edge-on. OBVR wraps the per-tree setups [S27].
- **Fix (renderer level).** Replace the distant-tree vertex shader with one that faces each
  instance by the camera's *position*. OBVR copies the game's bytecode and adds a prologue,
  turning 366 dwords into 384. `DISTLOD2002` has no EyeDir and is left alone. This is OBVR's
  `StableDistantTrees=1`, accepted in the headset [S27]. The ini-only test is
  `bDisplayLODTrees=0`, which removes the trees. **Built** in the adapter its own way
  (`distant_trees.rs`): the shader is made from the adapter's HLSL as the game creates it, each
  card turned to the direction from the eye to its tree (`visuals.md`). Near trees' leaf cards
  are turned per draw instead (`leaves.rs`: level, square to the line from the eye to the tree).

### 2.2 Alpha-tested cards sparkle (leaves, grass, LOD trees) [C/K]
- **Cause.** Foliage is alpha-*tested*: each pixel is either drawn or skipped. At distance the
  test runs on sub-pixel detail and changes every frame. MSAA alone does not help, because the
  test still runs once per pixel [S28].
- **Made worse by** the negative mip bias the game sets for tree textures:
  `fLODTreeMipMapLODBias=-0.75` and `fLocalTreeMipMapLODBias=-0.25` [S4]. DXVK's docs describe
  negative bias as "sharpness at the cost of shimmer" and offer `clampNegativeLodBias` for games
  that ship a high one [S31].
- **Fixes.** Set both biases to 0 in the ini (the adapter does: §9), or clamp negative
  `D3DSAMP_MIPMAPLODBIAS` to 0 in the adapter's `SetSamplerState` (not built) [I].
  **FoliageAA** (xOBSE, needs in-game MSAA) redraws leaves once per MSAA sample so each sample
  gets its own alpha test, or uses the vendor alpha-to-coverage hacks (`ATOC` NVIDIA/Intel, `A2M1`
  AMD), which "glitter" in VR under DXVK. It works "flat and in VR", with the game's own leaf
  shaders only [S28]. Whether D3D9On12 honours the ATOC/A2M1 FOURCC driver hacks is **[U]**.

### 2.3 Depth precision: water, LOD land and LOD objects z-fighting [C facts, I numbers]
- **Facts.** Near plane 10 units (14 cm), the `fNearDistance` default and measured by OBVR
  [S2][S27]. Far plane 283,840 units (about 4.05 km): OR's far-plane hook acts only when the
  game returns exactly `283840.0f` [S24]. Depth is 24-bit per the brief (D24S8
  likely) [U].
- **Precision with a standard projection** [I]. Quantisation error Δz ≈ z² / (n · 2²⁴). One
  unit is 1.43 cm and 70 units are 1 m [S16].

  | Distance | n = 10 | n = 15 | n = 20 |
  |---|---|---|---|
  | 1 km (70,000 u) | 29 u (0.42 m) | 19 u | 15 u |
  | 2 km (140,000 u) | 117 u (1.7 m) | 78 u | 58 u |
  | 4 km (280,000 u) | 467 u (6.7 m) | 311 u | 234 u |

  Real errors are larger, because float round-off in the transform dominates 24-bit
  quantisation [S30]. LOD land meeting a flat water plane at a shallow slope cannot hold
  together at those distances.
- **Known engine symptom.** "Flickering water LOD has always been a problem with Oblivion's
  handling" (Nexus mod 54789, via search summary [U]). Users attribute the shoreline flicker to
  z-fighting between shallow terrain LOD and water LOD [S34]. That mod also advises
  regenerating landscape LOD before water LOD [S34].
- **Fixes.**
  1. **Check the ini for `fNearDistance=0` or `0.0001`** (old first-person-clipping tips
     [S35]); both make distant z-fighting much worse.
  2. **Raise `fNearDistance`.** Error scales as 1/n. Skyrim users run 15–25 and report glitches
     above 25 [S36]; no Oblivion limit found [U]. In VR, 20 units (28 cm) clips hands and props.
  3. **Adapter: use the game's near plane per eye.** A smaller VR near (5 cm ≈ 3.5 units)
     triples the error [I]. Built: each eye's camera keeps the game's own near and far planes
     (`camera::eye_camera`).
  4. **Adapter: reversed-Z with a float depth buffer** (clear to 0, `D3DCMP_GREATEREQUAL`, swap near
     and far; not built). NVIDIA: "just use a floating-point depth buffer with reversed-Z!"; with
     24-bit integer depth it helps much less [S30]. Whether D3D9On12 offers `D3DFMT_D32F_LOCKABLE`
     or `D24FS8` is **[U]**. Every depth compare, read and clear site in the game would need
     finding, including the clear before the first-person node (OR patches it out) [I].
  5. **The far plane matters little.** OR's `FarPlaneDistance` moves it, but the near plane
     dominates precision [S30].

### 2.4 LOD meshes that flash yellow or black [K]
- Yellow at some angles: a bad or wrongly installed LOD land mesh (`Meshes\Landscape\LOD`) [S12].
- Black: the Bloom setting, or a bad `_far.nif` or `_fn` file. Fix: switch Bloom to HDR, or
  remove the offending mod [S13].
- Really AEVWD users saw yellow or black LOD at odd angles from specular and vertex colours on
  `_far` meshes. Switching Bloom to HDR fixed it for one user [S39 U].

### 2.5 Water surface shimmer [K, partly measured in VR]
- The water animation is a 32-frame loop at 12 fps (`uSurfaceFrameCount`, `uSurfaceFPS`), so
  under a 90 Hz headset the steps are visible. In an OBVR tester's headset 96 frames at 36 fps
  looked "better", but reflection artefacts remained. The engine's own reflection resets as the
  head moves, in mono too, so OBVR ships reflections off (`WaterReflectionMode=1`). Open
  hypotheses: an aliased wave normal map (no mips, point mip filter or negative bias), specular
  sparkle, and a magnified small reflection target [S27].
- Oblivion Reloaded patches a "purple water bug" and makes the reflection map size configurable
  (default 512) by overwriting code constants [S24]; so the vanilla size is hard-coded [I], value
  unknown [U].

### 2.6 Pop and transitions that read as flicker in a headset
- **Grass is built in a view cone, not all around the camera.** The cone is sized from the
  flat FOV: 46° either side at 75°, rebuilt after about 19° of turn. In a headset, grass comes
  and goes at the edge of the view. OBVR builds it all around (`GrassAllAround=1`) [S27
  engine-behavior.md]. So does the adapter (`grass.rs`, `visuals.md`).
- **Fade rings.** Specular fades at 500–800 units, lights at 1000–1500 [S2]. When a new grid
  loads, its contents appear all at once [S42 K].
- **Streamline's StreamSmooth** "changes your video settings in order to maintain the given FPS
  range" [S37]. That changes distances at runtime, so do not use it with VR.
- **Per-render state.** A second render walk in the same frame replays time-driven work:
  animation controllers and NPC look-at double-tick. OBVR zeroes the frame delta for the second
  render. Skinning palettes and FaceGen parts come out one eye
  stale [S27]. HDR tone mapping runs once per pass; whether eye adaptation advances per pass
  is **[U]**. If it does, the two eyes can be exposed differently [I]. The adapter renders less
  of the frame twice than OBVR (`visuals.md`): inside the world render the frame step's only
  reader is the actors' shadow pass, which it runs once a frame; skinned bones are made again for
  each eye (a renderer frame of their own).

---

## 3. Blurry, mushy and blotchy ground at distance

### 3.1 LOD land begins close and has very low texel density [C/K]
- **Where it starts.** Full-detail land covers the N x N loaded cells around the player; one
  cell is 4096 units = 58.5 m [S16]. The LOD boundary is (N−1)/2 to (N+1)/2 cells away [I]:
  **117–175 m at N=5**, 175–234 m at 7, 234–293 m at 9. This matches "not too far away".
- **Vanilla LOD colour maps.** The CS renders 64x64 px per 2x2 cells and stitches them into
  one 1024x1024 texture per 32x32-cell quad [S14]: 32 px per cell, about 1.8 m per texel [I].
  At 150 m with an assumed ~100° FOV over 5100 px (~50 px/degree), one texel covers about
  35 px, which is bilinear mush [I]. The colours are baked snapshots, so they do not match the
  live terrain shading beside them [S14 K].
- **Vanilla LOD normal maps** (`landscapelod\generated\*_fn.dds`) are "1024x1024 ... compressed
  with DXT1". This produces the "dark blocks on distant LOD terrain" [S33 U].
- **Fixes** (regenerated from the game's own landscape, not new art):
  - **TES4qLOD**: LOD colour maps at 1024, **2048 or 4096** per quad, plus VWD `.lod` files [S15].
    **But not sharper** (checked 2026-10-30 in tes4ll 5.15c, which bundles it [C]). It colours each
    point from a 4x4-pixel average of each landscape texture (its `tes4qlod_tex` cache), blended by
    the land's vertex weights. Above 1024 a quad (32 px a cell, the vertex spacing), it only
    interpolates, and tes4ll's README recommends 1024 for less tiling. The game's own maps are
    rendered from the textures themselves at that density. No tool here makes the far land's colour
    sharper from the game's own data.
  - **tes4ll**: LOD land meshes (adaptive vertex density, water keep-outs), normal maps
    (optionally with faked shadows), colour maps via its bundled TES4qLOD [S32].
    **Tried with tes4ll 5.15c (2026-10-30)** [C]:
    - Run as shipped, the vertex fill can fail in every quad ("This selection seems to be empty"):
      tes4ll takes the load order from `Plugins.txt`, which may list only the DLCs, since the game
      loads `Oblivion.esm` without it being listed. Its heightmap then has 1.1% of Cyrodiil. Setting
      `_modlist` (the game's order, masters first) in the `.mpb` before `ParseModList` fixes it. On
      the command line its last name loses a character, a parser bug. `ParseModList -cd` also moves
      into the game's `Data`, so the `.mpb`'s relative `ini\tes4ll\*.dat` paths have to be absolute.
    - Flags: `tes4ll_highres.bat`'s (`usedatafiles makemeshes wallremover contour high_res
      nicer_mountains`) plus `makepedestals` and `minheight=-512`. Result: 19,091..19,981 vertices a
      quad, where the game's own have 8,720..12,524.
    - **The game's far land is more than its cells.** `Oblivion.esm` has Tamriel cells only from
      x -64..69, y -69..59, and none south-west of (-1, -30). The game's far-land meshes past them are
      scenery that exists only in those meshes (the land past the border), which tes4ll makes sea
      floor. So only the six quads its cells cover completely are worth replacing: (-64,0), (-32,0),
      (0,0), (32,0), (32,-32), (32,-64): the Imperial City, Weye, Chorrol, Skingrad, Cheydinhal and
      the land east of the Niben. Bravil, Leyawiin, Anvil, Kvatch and Bruma's quads, and the other
      30, are best left as the game's.
    - **Seams.** The game's far-land meshes hang a skirt from every quad edge (each edge vertex again
      at -512), because their quads don't meet exactly either: the top surfaces differ along an edge
      by 0..99 units (median), up to 1,216. `makepedestals` gives tes4ll's quads the same skirts, to
      -512, and `minheight=-512` the game's sea floor (tes4ll's own is -2048). Where tes4ll's quads
      meet the game's, the top surfaces differ by 0..131 units (median), up to 1,390: the same range.
    - Same layout as the game's: one UV set, 0..1 from the quad's south-west corner, no normals or
      colours. The colour and normal maps stay the game's.
    - The adapter neither needs nor checks for any of this: it's the player's own choice of content.
  - **Landscape LOD NormalMap Fix**: uncompressed normal maps instead of DXT1 [S33 U].
  - **xLODGen** does *not* list Oblivion for terrain LOD [U]. **Raising uGridsToLoad** moves the
    boundary out (§4.1).

### 3.2 No anisotropic filtering, and point mip filtering [C/I]
- Oblivion has no AF option or ini key [S19][S8]. PCGamingWiki's fix is to force AF in the GPU
  control panel [S19]. The 3D Vision fix author calls 16x AF a "massive impact on visual
  quality" [S29].
- Ground seen at grazing angles (the near terrain out to the LOD boundary) is exactly where
  trilinear-only filtering blurs [I].
- **How Oblivion Reloaded adds AF** [C, S24 `RenderHook.cpp`]. It hooks the game's
  `SetSamplerState` wrapper: MINFILTER to ANISOTROPIC (unless NONE or POINT), MAGFILTER to
  LINEAR, MIPFILTER POINT to LINEAR, sets `D3DSAMP_MAXANISOTROPY`, and writes the level into
  two bytes of the game's code. The POINT-to-LINEAR rewrite suggests some
  vanilla samplers use point mip filtering, which shows as mip bands and soft patches [I].
- **Adapter fix.** The proposal [I] was to do the same in the device's `SetSamplerState`:
  anisotropic 16x with a linear mip filter, skipping samplers bound to render targets, shadow
  maps, depth, and point-filtered UI or lookup textures. OBVR sets DXVK's
  `d3d9.samplerAnisotropy = 16` for the same reason ("for the parallax shimmer") [S27][S31].
  **Built, more narrowly** (`d3d9.rs`): the hook on `IDirect3DDevice9::SetSamplerState` (slot 69)
  turns a pixel shader sampler (0..15) whose MINFILTER the game sets to LINEAR into ANISOTROPIC
  at `anisotropy` (`rustvr-oblivion.toml`, 16 by default, 1 off; testing,
  `RUSTVR_OBLIVION_ANISOTROPY`). Point sampling, magnification and the mip filter stay as the
  game sets them, so OR's POINT-to-LINEAR mip rewrite isn't done.
- **Driver override under D3D9On12.** The NVIDIA or AMD panel sees a D3D12 device. Whether
  D3D9-profile overrides (AF, AA, negative-LOD-bias clamp) apply is **[U]**; reports for DX12
  games conflict. Assume the adapter has to do it.
- UESP (NVIDIA): shimmer is worst with AF on at the "Performance" quality setting. Clamp
  negative LOD bias [S10].

### 3.3 Other sources of blotches
`fLightLOD1/2` past about 4500 (coloured blobs) [S9], Bloom turning LOD black [S13], and
`iTexMipMapSkip` above 0 [S2].

---

## 4. Distance and LOD settings in detail

### 4.1 uGridsToLoad
- **What it is.** The full-detail cell block, default 5x5. UESP: "controls the distance at
  which LOD objects and textures are scaled to their full quality" [S11]. It should be odd [K].
- **Pitfalls.**
  - `uExterior Cell Buffer` must be (1+uGrids)²: 36, 64, 100 [S20].
  - Developer-sourced 2006 advice: "quite a performance hit with anything above 5", stop at
    7–9; 11+ gives "water disappearing and hovering mountains" [S20]. UESP: "several graphical
    and gameplay glitches will arise if it is raised or lowered" [S10].
  - **Saves.** Players report only saves made at an equal or lower value load, and crashes
    after going back from 7 to 5 (workaround: load and re-save) [S38 U]. A CS bug at cell 4,4
    above 5 [U].
  - VR cost [I]: the grid is drawn once per eye, and D3D9 draw calls on D3D9On12 are
    CPU-bound. The 32-bit process also wants the LAA patch [S19].
- **Recommendation.** Keep 5 and fix the LOD instead (§3.1). If you raise it, use 7 with buffer
  64 on a new save and never lower it on that save.
- **The adapter leaves it alone.** It isn't one of the VR floors (§9): a save made at 7 must never
  be loaded at 5, so it can only be the player's own value in `Oblivion.ini`, in VR and flat alike.
  A playtester with a fast PC who found the pop-in too much ran 7 with buffer 64 that way.

### 4.2 Distant LOD (VWD) and fades
- **VWD.** A reference with the Visible When Distant flag shows `<mesh>_far.nif` "when you are
  far away" [S18], meaning beyond the loaded grid [I]. TES4qLOD (and TES4LODGen) write the
  `distantlod\*.lod` lists [S15]. Many vanilla objects have no `_far` model [S18]. Reportedly
  TES4LODGen takes any object with a `_far.nif`, flagged or not [U].
  - **Measured (2026-10-30, xEdit 4.1.5f's TES4LODGen on a vanilla game with all DLC).** The
    `.lod` lists hold the distant *trees* too (TREE base objects), not only `_far` models. Format:
    a count of groups, then for each a base form id, a count, that many positions, rotations (3
    floats each) and scales. Regenerated, Cyrodiil went from 92,917 to 125,696 far objects, and
    within 16 cells of the Imperial City from 9,854 to 18,586. Almost all of the gain was 41 shrub
    kinds (`ShrubBoxwood`, `ShrubAzaleaSU`, ...), which the game's own lists never draw far. In a
    headset that is twice the far alpha-tested cards (§2.2) for no gain.
  - **Filtered.** Keeping only the tree kinds the game's own lists use, the regenerated lists have
    93,375 far objects in Cyrodiil, in 146 more cells. The gain is small: castle walls, gates and
    towers the game's own lists miss (Skingrad's, Bravil's, Leyawiin's, a Boethia shrine), and some
    500 trees and snowy shrubs of kinds the game does draw far. 57 of the game's own far objects
    are gone (flora, a few trees), and none of the DLCs' buildings are among the additions. Six
    cells the generator left empty are best left with the game's own. TES4LODGen also writes a
    `<worldspace>.cmp` per worldspace: the cells with a `.lod`, as pairs of 16-bit coordinates (the
    game's own `.cmp`s are shorter lists of an unknown meaning) [C]. The later playtests ran with
    the filtered lists.
  - More VWD objects also means more surfaces that can z-fight in the far depth range (§2.3)
    [I].
- **LOD ranges.** `uGridDistantCount` (buildings) and `uGridDistantTreeRange` (trees) set the
  ranges [S9]. Raising them adds more alpha-tested cards at depths with poor precision [I].
- **Undocumented keys.** STEP lists defaults only for `fLODQuadMinLoadDistance=65536`,
  `iLODTextureSizePow2=8`, `iLODTextureTiling=2`, `bLODUseCombinedLandNormalMaps=1` and
  `fLODLandVerticalBias=0` [S3]. Their effects are **[U]**.

---

## 5. Anti-aliasing

### 5.1 In-game MSAA [C]
- `iMultiSample` 2, 4 or 8; the launcher will not combine it with HDR [S19]. The engine treats
  the effective count as 0 when HDR is on [S27].
- With 2+ samples the world draws straight into the multisampled back buffer and resolves it;
  without MSAA it draws into a frame-sized texture the image-space manager copies over [S27].
- **Adapter caveat.** Resolve the MSAA back buffer (StretchRect to single-sample) before
  copying or submitting it. OBVR's headset stopped at the first world frame with launcher AA x8
  until it did [S27]. The adapter copies each eye's back buffer with `StretchRect` into a render
  target of its own, and resolves a multisampled depth buffer to each pixel's nearest sample
  before copying it (`d3d9.rs`); the playtests ran without MSAA (`iMultiSample=0`).
- **Cost [I].** 27.5 MP per eye at 4x with RGBA8 plus 32-bit depth is about 0.9 GB per target
  set, about 1.3 GB with FP16 colour.

### 5.2 HDR together with MSAA [C]
- **Why vanilla cannot.** HDR draws into an FP16 *texture*, and D3D9 textures cannot be
  multisampled: an MSAA FP16 surface "can't be directly used as textures" until resolved into
  one [S21]. Bethesda blocked the combination outright [S22].
- **ATI's 2006 "Chuck" patch** attached a multisampled FP16 surface to the game's FP16 render
  texture and resolved it whenever the texture was bound; the TAB view was not covered [S21].
- **Oblivion Reloaded does it in code.** With HDR on and `iMultiSample ≥ 2` it creates
  `CreateRenderTarget(w, h, D3DFMT_A16B16G16R16F, iMultiSample)` and puts it in the default
  render-target group (`RenderManager.cpp`) [S24]. Users: "Turn on HDR in the launcher, then
  install OR, then set AA ... in the INI" [S23]. **The adapter can copy this without OR** [I].

### 5.3 Driver-forced, foliage, post-process and supersampling
- **Driver.** Historically HDR in game plus AA forced in the panel worked [S10]; the 3D Vision
  fix uses 4x from NVCP [S29]. On D3D9On12 the driver sees D3D12, so D3D9 AA overrides,
  compatibility bits and transparency supersampling most likely do not apply [U].
- **Foliage.** FoliageAA or alpha-to-coverage (§2.2); both need MSAA.
- **Post-process.** ORC has "configurable SMAA" from 1.7.2 and a TAA key [S40 U][S43]; ReShade
  and ENB are d3d9.dll proxies, and ENB "is fully incompatible with D3D9Ex" [S25]. SMAA and FXAA
  are spatial only and do nothing for temporal sparkle; TAA would need history and motion
  vectors *per eye* [I].
- **Supersampling.** At about 27.5 MP per eye, more resolution costs more than MSAA plus foliage
  AA plus AF [I].

---

## 6. Mods and engine patches: what they change, and the risk with a two-render stereo adapter

| Mod | What it changes (renderer level) | OBSE? | Conflict risk with the injected adapter | Src |
|---|---|---|---|---|
| xOBSE | Script extender, tested on English 1.2.0.416. On Steam it loads through `obse_steam_loader.dll` | n/a | Low | S26 |
| Oblivion Display Tweaks | See below | Yes | **Medium–High** for the D3D9Ex/FlipEx/borderless features; Low for timing only | S25 S19 |
| Oblivion Stutter Remover | Heap replacement, critical sections, FPS management (`bManageFPS`, `fMaximumFPS` [U]) | Yes (v17+) | **Medium**: its FPS limiter and smoothing can fight VR frame pacing. Not compatible with OR [U] | S37 |
| EngineBugFixes (Tiawar), AveSithis, Modern Engine Fixes, Blue's | Engine and crash fixes (the "Abomb" animation-timer fix, a texture-path underscore check). EBF has an InitRenderer hook [U] | Yes | Low | S19 S45 |
| Oblivion Reloaded / ORC / E3 | See below | Yes | **High**. OBVR lists OR, ORC and E3 as incompatible | S24 S27 S40 |
| Blockhead | Per-NPC and per-race head, body and animation overrides. Not rendering quality | Yes | Low; irrelevant here | S44 |
| TES4LODGen, TES4qLOD, tes4ll | Offline tools that regenerate LOD meshes, textures and `.lod` files from the game's data | No | None at runtime | S15 S32 |
| Really AEVWD and similar | Adds `_far` meshes, which are content | No | None at runtime, but more distant z-fighting surfaces and possible yellow or black LOD | S39 |
| FoliageAA | Hooks the game's D3D9 device and redraws leaf draws per MSAA sample. Needs MSAA. Only the game's own leaf shaders | xOBSE | Medium: two hookers on one device; must see the real game draws [I] | S28 |
| ReShade / ENB | d3d9.dll proxies that apply effects at Present | No | High [I]: Present is not where the per-eye images live | S25 |
| Streamline | Dynamic FPS smoothing that changes video settings | Yes | Avoid (§2.6) | S37 |

- **Oblivion Display Tweaks**: QPC timing, frame-delta fix for uncapped FPS, "true FOV fix",
  fast exit, borderless. For D3D9Ex/FlipEx it patches the game's `Direct3DCreate9` and
  `CreateDevice` calls and wraps the device. With the adapter
  use `[DirectX] bUseD3D9Ex=0`, `bUseFlipExSwapMode=0`, `bBorderless=0`, and treat `bFOVFix`
  with care since it touches the projection the adapter replaces [I].
- **Oblivion Reloaded / ORC / E3** replace the render pipeline (shaders, shadows, water, SMAA,
  INTZ depth, AF hook, HDR+MSAA target, far plane, per-camera near distance, no depth clear
  before the first-person node) and assume one camera per frame. Better to
  **copy individual techniques** (§2.3, §3.2, §5.2) into the adapter.

---

## 6b. Stutter when crossing cells (added 2026-10-30)

- **The engine.** Gamebryo as Oblivion uses it loads new cells' geometry largely on the main thread,
  so crossing into a new row of the loaded grid hitches on any hardware; storage speed, LOD content
  and memory settings change how much [K] ([Nexus forum](https://forums.nexusmods.com/topic/636678-ended-stutter-in-oblivion)).
  Content without proper LOD (some VWD mods) makes it much worse [K].
- **Oblivion Stutter Remover** (OSR, xOBSE): heap replacement and critical-section fixes. The
  community pairs it with Oblivion Display Tweaks (ODT): ODT for the timer (QueryPerformanceCounter)
  and the frame delta, so OSR's `bReplaceGetTickCount=0` and its FPS management off; for the heap,
  either OSR's replacement (algorithm 6, about 450..512 MB) or MoreHeap, not both [K]
  ([OSR](https://www.nexusmods.com/oblivion/mods/23208), [ODT](https://github.com/ersh1/Oblivion-Display-Tweaks)).
  OSR's own documentation calls its heap replacement "moderately buggy" on Oblivion [K].
  - **OSR with the adapter** (tested with OSR 4.1.37) [C, from its source and ini]. It needs xOBSE,
    which the adapter loads itself (`xobse.rs`: the game started from its exe, as the adapter's
    launcher starts it, gets no `obse_steam_loader.dll` from Steam). Set `bManageFPS=0` and
    `bReplaceGetTickCount=0`: the headset paces the frames and the adapter times them. Set
    `bFastExit=0`: its fast exit runs `con_SaveINI`, then `TerminateProcess`, which would save the
    VR floors (§9) into `Oblivion.ini` and skip the adapter's restore at exit (the player's values
    then come back only at the next start). The heap tested was algorithm 6 at 512 MB.
  - **One critical section**, the renderer's, guards making a texture's renderer data the first
    time it's needed. OSR suppresses it by default (mode 5). Since the adapter lets the background
    loader run between the eyes, it was first kept as a lock (mode 3); in playtest 7 a dozen
    threads waited 5..26 ms each on it in a 239 ms frame (`visuals.md`), so OSR's default
    (mode 5) is the recommendation, as OSR's users have run it for years.
  - None of OSR's patch sites (its import slots, the main loop's call, the memory functions, the
    hashtable sizes, `rand`) is one the adapter patches; the adapter calls the game's allocator,
    which OSR's heap replaces.
- **Address space.** The Steam `Oblivion.exe` (1.2.0.416) is **not** large-address-aware
  (checked: `IMAGE_FILE_LARGE_ADDRESS_AWARE` clear), so it has 2 GB; in VR at 5099x5418 an eye the
  adapter's log showed 550..920 MB of it free. The 4GB patch is reported to work on the Steam exe
  with xOBSE 22.13, but Steam can put the exe back when it starts online [K]
  ([Nexus 56022](https://www.nexusmods.com/oblivion/mods/56022)). A big OSR heap inside 2 GB
  risks running out. The playtests ran with the 4GB patch (the flag set and the PE checksum
  recomputed) and xOBSE 22.14 (GitHub `llde/xOBSE`); checked against xOBSE's source, none of its
  hooks overlap the adapter's. The adapter doesn't check for the patch: its timing line reports
  the address space ("address space N of M MB free": 4095 with the patch, about 2047 without).
- **Oblivion.ini**: `bBackgroundCellLoads=1`, `iPreloadSizeLimit` raised with the cell buffers
  together (26214400 is the 2006 base; 104857600 the 2 GB-RAM row of the developer-sourced list),
  never the exterior buffer alone [K]
  ([STEP](https://stepmodifications.org/wiki/Guide:Oblivion_INI/BackgroundLoad)). The adapter's
  VR floor for `iPreloadSizeLimit` is 262144000 (250 MB), chosen on a large-address-aware exe
  (§9).
- **The adapter's own part.** The world render takes the game's background loading thread's lock
  at its start and gives it back at its end. Held across both eyes and their
  copies, it left the thread 1.5 ms of each 9 ms frame (`visuals.md`, "Stutter crossing cells"),
  so each eye's render now takes and gives it, as the game's one render does (`render.rs`;
  testing, `RUSTVR_OBLIVION_HOLD_LOADER=1`: held across the eyes). The game's waits for it are
  timed in the timing line ("waits for the loader"), and one over 20 ms is logged.

## 7. Oblivion in VR: prior art and what it recommends

- **OBVR** (Naragorn, GitHub, early test builds 0.2.x, 2026) [S27].
  - Native VR: xOBSE plugin plus DXVK (gplasync, "not optional") plus OpenVR. Two full world
    renders per frame (`Stereo=dual`); `aer` (alternate eyes) is the lighter fallback.
  - Findings that matter here: distant LOD trees face EyeDir (`StableDistantTrees=1`); leaf
    batches reuse the first tree's basis (`StableLeafBillboards=1`); grass is built in a
    flat-FOV cone (`GrassAllAround=1`); water reflections reset with the head, so they ship off
    (`WaterReflectionMode=1`) with wave steps at 96/36; with AA on the back buffer is MSAA and
    must be resolved; effective MSAA is 0 with HDR; the second render must zero the frame
    delta; shadow maps can be shared between eyes (`ShareActorShadows`); `dxvk.conf` sets
    `d3d9.samplerAnisotropy = 16` for parallax shimmer.
- **FoliageAA** (same author): foliage AA for flat and VR (§2.2) [S28].
- **vorpX.** Oblivion is officially supported in G3D and DirectVR modes [S19]. No published
  per-setting advice was found [U].
- **3D Vision fix (Helixmod, 2013)** [S29]: maxed settings, **HDR on, 4x AA and 16x AF from
  NVCP**, self-shadows off. It needed shader fixes for shadows, water reflections and sky
  (clouds, sun, moons) to be right in stereo; distant dynamic shadows still clip; OBGE Liquid
  Water conflicts with it.
- **Not found** [U]: an established community ini list for Oblivion VR beyond the above. VR
  activity in 2025 is mostly UEVR on *Oblivion Remastered*, which is a different engine.

---

## 8. What I could not confirm, and what is worth measuring in the adapter
- What `iTexMipMapMinimum`, `iLODTextureSizePow2`, `fLODQuadMinLoadDistance` and
  `fLODLandVerticalBias` do in Oblivion.
- The vanilla depth-stencil format (D24S8 assumed), and whether D3D9On12 exposes float depth
  formats or the ATOC/A2M1 hacks.
- Whether HDR eye adaptation advances once per eye render, and so differs between the eyes.
- Whether `fLODFadeOutMult*` has any effect beyond the loaded grid.
- The exact uGridsToLoad save behaviour. The sources are player anecdotes.
- The Nexus pages for ODT (50348), ORC (51927), EBF (47085), OSR (23208), the water LOD mod
  (54789) and Blockhead (43752) blocked direct reading. The claims above that rely on them come
  from search summaries and are marked [U].
- Cheap A/B tests [I]: `bDisplayLODTrees=0` for the flicker; `fLODTreeMipMapLODBias=0`; log
  the sampler states (MIPFILTER, MAXANISOTROPY, MIPMAPLODBIAS) on terrain, LOD-land and tree
  draws. (The near and far planes of the adapter's per-eye projection are the game's own.)
  Testing, `RUSTVR_OBLIVION_INI_TEST="Section.key=value;..."` sets any of the game's settings for
  one run (put back with the rest).

---

## 9. What the adapter does with this

**Built** (as of main; details in `visuals.md`):
- **VR floors in `Oblivion.ini`** (`game_settings.rs`, `VR_FLOORS`): while in VR a few of the
  game's values are raised to at least these, the player's own higher ones kept, and all put back
  when the game closes (or at the next start after a crash; the player's are kept in
  `rustvr-oblivion-ini.txt` beside the DLL meanwhile). Testing, `RUSTVR_OBLIVION_FLAT_INI=1`: none
  raised.

  | Key | Floor | Why |
  |---|---|---|
  | `iShadowMapResolution` | 1024 | 256 is blocks at arm's length |
  | `fLODTreeMipMapLODBias`, `fLocalTreeMipMapLODBias` | 0 | the negative bias sparkles (§2.2) |
  | `bForceFullLOD` | 1 | near trees don't switch detail as you move |
  | `uSurfaceFPS`, `uSurfaceFrameCount` | 36, 96 | the same 2.7 s wave loop, without 12 fps steps (§2.5) |
  | `iPreloadSizeLimit` | 262144000 | the cells ahead preloaded up to 250 MB (§6b) |
  | `fLODFadeOutMultObjects`, `Items`, `Actors` | 15 | things fade in further out: a head turns to what a flat screen hides |
  | `fLODMultTrees` | 2 | the game's own limit (`fTreeLODMax`) |
  | `fGrassEndDistance`, `fGrassStartFadeDistance` ([Grass]) | 4000, 3000 | grass to 57 m all round (86 m cost too much of the frame) |
  | `uGridDistantCount`, `uGridDistantTreeRange` | 40, 30 | distant buildings and tree cards half as far again |
  | `bLoadBackgroundFaceGen`, `bBackgroundLoadLipFiles`, `bUseThreadedMorpher`, `bUseThreadedBlood`, `bBackgroundPathing` | 1 | off, they hitched as people came into view; `bUseThreadedTempEffects` and `bUseThreadedParticleSystem` stay as they are (reported to crash) |

  The game is also windowed at the size each eye wants (`iSize W`/`H`), and `SIntroSequence` is
  emptied (the intro films only play flat). `uGridsToLoad` is left to the player (§4.1).
- **Anisotropic filtering** in `SetSamplerState` (§3.2): `anisotropy`, 16 by default.
- **The distant trees' cards** face the eye's position (`distant_trees.rs`, §2.1); **the leaves'
  cards** are turned per draw, and the vs_1_1 leaf shaders' address register is floored, which
  D3D9On12 rounds (`leaves.rs`); **grass** all round (`grass.rs`, §2.6).
- **Fog by the view distance**, not the clip-space length the game's shaders use (`fog.rs`).
- **The game's near and far planes** for each eye (§2.3).
- **Once a frame**: the actors' shadow maps, shared by both eyes, their clip planes made again for
  each eye (`render.rs`).
- **The background loader's lock** given back after each eye (§6b).
- **xOBSE and its plugins** (Oblivion Stutter Remover) loaded when installed (`xobse.rs`).
- **In the runtime**: the headset's picture averages the game's over each headset pixel's
  footprint (`warp.rs`, `compositor.rs`), so a supersampled eye doesn't flicker in and out of
  single taps.

**Not built:** reversed-Z or a float depth buffer (§2.3); HDR with MSAA (§5.2); clamping negative
mip biases in `SetSamplerState` (the floors set the trees' to 0 instead); OR's POINT-to-LINEAR mip
filter rewrite; foliage AA or alpha to coverage (§2.2); water reflection changes (OBVR ships them
off); raising `uGridsToLoad`.

**Mods, for players.** Known to work with the adapter in the playtests: the 4GB patch, xOBSE 22.14,
Oblivion Stutter Remover set as in §6b, and LOD regenerated as in §3.1 and §4.2. Expected to
conflict, not tried [I]: Oblivion Reloaded, ORC and E3 (they replace the render pipeline for one
camera a frame), ReShade and ENB (d3d9 proxies that work at Present, §6), Streamline (§2.6), and
ODT's D3D9Ex, FlipEx and borderless options (they wrap the device the adapter makes). The adapter
checks for none of them.

---

## Sources
- S1 STEP, Guide:Oblivion INI (in-game option to ini key map): https://stepmodifications.org/wiki/Guide:Oblivion_INI
- S2 STEP, Oblivion INI/Display: https://stepmodifications.org/wiki/Guide:Oblivion_INI/Display
- S3 STEP, Oblivion INI/LOD: https://stepmodifications.org/wiki/Guide:Oblivion_INI/LOD
- S4 STEP, Oblivion INI/SpeedTree: https://stepmodifications.org/wiki/Guide:Oblivion_INI/SpeedTree
- S5 STEP, Oblivion INI/Water: https://stepmodifications.org/wiki/Guide:Oblivion_INI/Water
- S6 STEP, Oblivion INI/Grass: https://stepmodifications.org/wiki/Guide:Oblivion_INI/Grass
- S7 STEP, Oblivion INI/General: https://stepmodifications.org/wiki/Guide:Oblivion_INI/General
- S8 STEP forum, "Oblivion default values for all valid INI settings": https://stepmodifications.org/forum/topic/8233-oblivion-default-values-for-all-valid-ini-settings/
- S9 UESP, Oblivion Mod:Oblivion.ini: https://en.uesp.net/wiki/Oblivion_Mod:Oblivion.ini
- S10 UESP, Oblivion:Tweaking: https://en.uesp.net/wiki/Oblivion:Tweaking
- S11 UESP, Oblivion:Ini Settings: https://en.uesp.net/wiki/Oblivion:Ini_Settings
- S12 UESP, PCS "Distant objects light up yellow": https://en.uesp.net/wiki/Oblivion_Mod:PCS_Library/Distant_objects_light_up_yellow_(Oblivion)
- S13 UESP, PCS "All distant (lod) objects turn black": https://en.uesp.net/wiki/Oblivion_Mod:PCS_Library/All_distant_(lod)_objects_turn_black
- S14 UESP CS wiki, Landscape LOD Tutorial: https://cs.uesp.net/wiki/Landscape_LOD_Tutorial
- S15 UESP CS wiki, TES4qLOD: https://cs.uesp.net/wiki/TES4qLOD
- S16 UESP CS wiki, Oblivion Units: https://cs.uesp.net/wiki/Oblivion_Units
- S17 UESP CS wiki, HDR Settings: https://cs.uesp.net/wiki/HDR_Settings
- S18 UESP CS wiki, Visible When Distant Tutorial: https://cs.uesp.net/wiki/Visible_When_Distant_Tutorial
- S19 PCGamingWiki, The Elder Scrolls IV: Oblivion (read through its MediaWiki API): https://www.pcgamingwiki.com/wiki/The_Elder_Scrolls_IV:_Oblivion
- S20 AnandTech forum, "Oblivion PC tweaks" (developer-sourced ini list, 2006): https://forums.anandtech.com/threads/oblivion-pc-tweaks.1835524/post-19678391
- S21 HEXUS, ATI's "Chuck" on the Oblivion HDR+AA driver: https://www.hexus.net/tech/features/graphics/5282-atis-ninja-programmer-chuck-speaks-oblivion-driver/?page=3
- S22 bit-tech, "ATI delivers AA+HDR for Oblivion": https://bit-tech.net/news/gaming/ati_oblivion_aa_hdr/1/
- S23 STEP forum, General Oblivion support p.4 (OR with AA): https://stepmodifications.org/forum/topic/6230-general-oblivion-support/page/4/
- S24 TES Reloaded source (llde), `TESReloaded/Core/RenderHook.cpp`, `RenderManager.cpp`, `SettingManager.cpp`: https://github.com/llde/TES-Reloaded-Source-NEW
- S25 Oblivion Display Tweaks source (`D3D9Hook.cpp`, `main.cpp`): https://github.com/ersh1/Oblivion-Display-Tweaks
- S26 xOBSE: https://github.com/llde/xOBSE
- S27 OBVR, `docs/vr-modding/rendering-and-stereo.md`, `engine-behavior.md`, `open-questions-and-known-issues.md`, `docs/leaf-batch-fix.md`, `docs/water-shimmer-plugin-spec.md`, `OBVR.ini`, `HANDOFF.md`: https://github.com/Naragorn/OBVR
- S28 FoliageAA: https://github.com/Naragorn/FoliageAA
- S29 Helixmod, Oblivion 3D Vision fix: https://helixmod.blogspot.com/2013/05/oblivion.html
- S30 NVIDIA, "Depth Precision Visualized": https://developer.nvidia.com/content/depth-precision-visualized
- S31 DXVK `dxvk.conf` (samplerAnisotropy, samplerLodBias, clampNegativeLodBias): https://github.com/doitsujin/dxvk/blob/master/dxvk.conf
- S32 tes4ll: https://zff.dev/PopeRigby/tes4ll and https://www.nexusmods.com/oblivion/mods/40549
- S33 Landscape LOD NormalMap Fix (via search summary): https://www.nexusmods.com/oblivion/mods/2446 and https://www.fileplanet.com/archive/p-40422/Elder-Scrolls-IV-Oblivion-Landscape-LOD-NormalMap-Fix-v1-0 ; 2048 LOD textures listing: https://www.fileplanet.com/archive/p-65592/Elder-Scrolls-IV-Oblivion-Landscape-LOD-replacement-textures
- S34 Distant water LOD mod and shoreline z-fighting reports (via search summary): https://www.nexusmods.com/oblivion/mods/54789 and https://steamcommunity.com/app/22330/discussions/0/1290690926861452627
- S35 fNearDistance=0 / 0.0001 first-person advice (via search summary): https://forums.nexusmods.com/topic/1895855-texturemesh-bug-while-in-1st-person-view and https://www.gamesas.com/post1169261.html
- S36 STEP forum, z-fighting ini tweaks (Skyrim): https://stepmodifications.org/forum/topic/212-z-fighting-ini-tweaks/
- S37 UESP, Stabilization and Optimization Mods: https://en.uesp.net/wiki/Oblivion_Mod:Stabilization_and_Optimization_Mods
- S38 uGridsToLoad saves (via search summary): https://forums.nexusmods.com/topic/1659658-ugridstoload-question/ and https://forums.nexusmods.com/topic/293067-cannot-revert-ugridstoload-back-to-default-value
- S39 Really AEVWD thread (via search summary): https://www.gamesas.com/really-aevwd-thread-t77013-50.html
- S40 Oblivion Reloaded Combined (via search summary): https://www.nexusmods.com/oblivion/mods/51927
- S41 Microsoft D3D9On12: https://github.com/microsoft/D3D9On12 and https://devblogs.microsoft.com/directx/coming-to-directx-12-d3d9on12-and-d3d11on12-resource-interop-apis/
- S42 STEP forum, fade distances and grid loading: https://stepmodifications.org/forum/topic/3507-fade-distances-and-grid-loading/
- S43 User-shared OR config (FarPlaneDistance 283840, AnisotropicFilter, SMAA/TAA keys): https://pastebin.com/V5QkULzp
- S44 Blockhead (via search summary): https://www.nexusmods.com/oblivion/mods/43752
- S45 Blue's Engine Fixes: https://github.com/BlueAmulet/BA_EngineFixes
