# Playtest: armour that moves like armour

Save: rustvr-base

The Nord in iron (and the thief's boots) from `rustvr-base`. Start it with:

```
powershell -File tools\play.ps1 armour
```

To wear another vanilla set, give its four pieces and put them on, dropping the iron they replace
(the set below is steel). One set at a time: four heavy sets carried at once (341 lb on top of the
Nord's own things) leave the Nord too laden to walk, even with strength 100.

```
powershell -File tools\play.ps1 armour -Give "000229A2 1,0001C6D8 1,000229A3 1,000229A5 1" -Commands "player.equipitem 000229A2|player.equipitem 0001C6D8|player.equipitem 000229A3|player.equipitem 000229A5|player.removeitem 0001C6D1 1|player.removeitem 0001C6D2 1|player.removeitem 0001C6D0 1"
```

| Set | Cuirass | Gauntlets | Greaves | Boots |
|---|---|---|---|---|
| Iron | 0001C6D1 | 0001C6D2 | 0001C6D0 | 0001C6CF |
| Steel | 000229A2 | 0001C6D8 | 000229A3 | 000229A5 |
| Leather (the thief's models) | 0002319A | 00023199 | 00023198 | 0002319B |
| Chainmail | 0001C6D3 | 0001C6D6 | 0001C6D5 | 0001C6D4 |
| Fur | 00024766 | 00024765 | 00024764 | 00024767 |
| Elven | 0002299C | 0002299E | 0002299D | 0002299F |
| Mithril | 0002C0FE | 0002C100 | 0002C102 | 0002C0FC |
| Glass | 00036343 | 00036340 | 00036342 | 00036341 |
| Dwarven | 00036349 | 00036346 | 00036348 | 00036347 |
| Orcish | 0003634F | 0003634C | 0003634E | 0003634D |
| Ebony | 0002AD85 | 00036352 | 00036354 | 00036353 |
| Daedric | 0003635B | 00036358 | 0003635A | 00036359 |
| Legion | 00028ADF | 00028AE0 | 00028AE1 | 00028ADE |
| Blades | 00023318 | 0002391C | 00023329 | 00023921 |

## What changed

* **Every joint keeps its volume.** The whole body (skin, clothes, armour) is skinned about each
  vertex's centre of rotation (Le and Hodgins 2016) instead of the game's linear blending: a fully
  bent elbow stays round instead of pinching to a crease, a turned wrist stays round instead of
  twisting into a ribbon, knees keep their shape in a crouch.
* **Armour plates are rigid.** Every vanilla set is marked (`content/armour/oblivion.toml`): its
  hard parts move as one piece each, riding the bones they belong to.
  * Gauntlets: the vambrace (the cuff over the forearm) stays straight on the forearm and rolls
    with it; the plate over the back of the hand moves with the hand; the wrist bends between them.
    The iron cuff, flare and all, is one rigid piece, and its hand plate slides under the cuff's
    rim as the wrist bends in and over it as the wrist bends back (playtest 1).
  * Elbows: the couter (the elbow cop) is a rigid cap turning half the elbow's bend about it.
  * Shoulders: pauldrons hang as the game drapes them over the shoulder and lift part of the way
    with the arm, rigid. (Each plate keeps the shape the game gives it standing with the arms at
    the sides; the models are made with the arms out level.)
  * The upper arm's plates on the upper arm, tassets hinged at the hips, cuisses on the thighs,
    knee cops half and half about the knee (dwarven, the women's iron), boots' shins, feet and toe
    caps each rigid with the ankle soft between, breastplates rigid at the chest with the belly soft.
  * Mail, fur, cloth and soft leather stay soft (with round joints); boiled-leather bracers and
    boots are rigid.
  * Studs, rivets and buckles move with the plate they're on.
* **Plates seen from both sides no longer go black.** The iron pauldrons and tassets (and other
  sets' two-sided plates) were drawn as their dark backs (two surfaces in one place facing apart);
  now their fronts show.

## What to try, in order

1. **Your forearms and hands** (most of what you see in VR). Hold your hands out in front and look
   at them. Turn the palms up and down: the iron cuffs roll with the forearm and stay round. Bend
   the wrists down and back as far as they go: the cuff stays straight, the back-of-hand plate goes
   with the hand, the wrist bends between them (before: the whole metal cuff bent like rubber).
2. **Elbows.** Curl a hand up to the shoulder: the elbow plate stays a rigid cap over the point of
   the elbow and the inside of the elbow keeps its volume (before: both collapsed into a crease).
3. **Shoulders.** Reach forward, cross your arms, raise them overhead, and look at your shoulders
   (a mirror or the third-person view isn't there yet; turn your head as far as it goes): the
   pauldrons hang over the shoulder and lift a little with the arm, never bending.
4. **Look down and crouch.** The breastplate's chest stays flat while the belly bends, the tassets
   swing at the hips, the greaves, the boots' shins and the feet stay rigid, knees keep their round.
5. **Other sets.** Equip the sets above one at a time (the body is put together again in a second or
   two after each change). Steel, glass, ebony and daedric are the heavy plate; elven, mithril and
   the Legion's are lighter plate over mail; chain mail, fur and leather are mostly soft.
6. **Compare with the game's own skinning.** Close the game, then in the same PowerShell window
   `$env:RUSTVR_SKINNING = "linear"` and start it again: everything bends as the game bends it.
   `"centres"`: round joints but no rigid plates. Remove it (`Remove-Item Env:RUSTVR_SKINNING`)
   for the marked armour.

The runtime's log (`%LOCALAPPDATA%\RustVR\rustvr.log`) says, for each piece you wear, the plates
its marks made (`avatar: armor\iron\m\gauntlets.nif (Iron gauntlets): 4 plates: ...`).

## Marking armour (for a modder, or to tune a set)

The marks are hand-written, one `[[piece]]` a model (or a male and a female model alike), each
with `[[piece.plate]]`s choosing islands (the pieces of a mesh) by mesh, bone, side, box and size,
or the corners of an island on a bone, and what each rides. The framework's `docs/ARMOUR.md` is
the guide. Editing `content/armour/oblivion.toml` takes effect the next time the game starts (it's
read at run time, not built in). The preview draws any set through a sweep of poses:

```
cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-avatar --bin rustvr-armour
target\release\rustvr-armour oblivion --set steel            # captures\armour\steel\*.png
target\release\rustvr-armour oblivion --set steel --list     # islands and plates
```

Each `<pose>.png` has a row per way of skinning (the game's, round joints, the marks) and a column
per view (front, side, back, the wearer's eyes, the forearm, the shoulder, the elbow side on, the
wrist); `islands.png` and `plates.png` colour what the marks choose.

## What was checked

* **In the preview** (the runtime's own renderer, headless): every set, men's, through the poses
  (rest, arms raised, crossed, elbows bent fully, palms up and down, wrists bent in and back,
  reaching, holding a gun, a squat), the game's skinning against the marks; the women's sets
  listed (every one makes its plates) and a few drawn. The hands and arms views (`--visibility`).
* **In the game, on the emulated headset**: `rustvr-base` (iron): the forearms out, wrists bent,
  palms up, elbows bent, arms raised, a crouch, looking down; nothing broken, the plates in the log.
  Then steel, glass and daedric equipped by console in one run: the body put together again for
  each, plates made, nothing in the log going wrong. The same run with `RUSTVR_SKINNING=linear` to
  compare.
* The framework's existing tests (avatar, asset) pass; the shaders compile.

**Not checked**: the real headset; the women's bodies and the beast races (Khajiit, Argonian:
their own skeleton) in the game; sets other than iron, steel, glass and daedric in the game (only
in the preview); the Metro adapter (its avatar now skins with round joints too, untouched
otherwise).

## Decisions

* **Plates without new bones.** A plate is its vertices all given the same weights and the same
  centre of rotation, so the same skinning that rounds the joints moves it rigidly; riding two
  bones it turns by their shares about the joint between them (the AAA helper joint, without the
  helper). Nothing extra per frame on the CPU.
* **The game's weights are the default riders** (the artist's say of what a plate follows, made
  rigid), shaped at rest (arms at the sides). Marks name riders where the game's are wrong for a
  rigid piece: couters half and half, breastplates on the chest only (the game's carry a little of
  the head, and a breastplate turning with your head is wrong), vambraces on the forearm's twist.
* **Two-sided plates**: front faces are drawn first, back faces a little further off, so a
  game's double surface shows its front; a cut-open sleeve still shows its inside.
* **Overlaps**: a plate on a higher layer is drawn 2.5 mm a layer nearer where it meets a lower
  one (pauldron over the upper arm's plates, daedric lames over one another). No collision between
  plates: their riders keep them apart in the swept poses.
* **Hard things only**: mail, fur, cloth and soft leather are soft; boiled leather is rigid.
* **The marks live in this repository** (`content/armour/oblivion.toml`), read by the runtime at
  run time; `RUSTVR_ARMOUR=<file>` reads another file, and `local_assets/oblivion/armour/*.toml`
  adds a modder's own.
* The arm IK is untouched.

## Playtests

1. **The iron cuffs bent like cloth at the wrists** (the headset, `rustvr-base`). The marks loaded
   and made their plates; the fault was in them. The iron cuff and the plate over the back of the
   hand are one island, the hand plate a strip of long triangles from the cuff's rim to the
   knuckles, and the game blends the cuff's whole flare (the last 5 cm, past the wrist) between the
   forearm and the hand: the right cuff's mostly on the forearm, the left's mostly on the hand.
   Split by weight (`firm = 0.7`), the flare stayed soft and folded with the hand. Fixed: the
   framework's marks choose corners by place (`corners_within`) and let a plate come apart at its
   `seam` (its own copy of the corners it shares); the iron cuff is split at the end of its flare,
   the cuff rigid on the forearm and the hand plate on the hand, sliding past the cuff's rim. In
   the preview the steel cuffs fold the same way (and likely the other plated gauntlets: glass,
   ebony, daedric, dwarven, orcish, mithril, elven); not changed yet.
2. **The cuffs held, but the leather under the bracer didn't bend, and the rings at the elbows went
   through the arm at the least bend.** The iron bracer is metal over the back of the forearm and
   leather (the glove's) underneath with a strap band, one island; the couter is a plate round the
   back of the elbow and the leather strap holding it on across the inside of the elbow, one island
   too. Rigid, the strap was in the way of the forearm the moment the elbow bent. The game's own
   textures say which is which (its specular mask too faintly to go by, the layout of the texture
   exactly): the framework's marks choose a plate's corners by where its texture draws them
   (`uv_within`), the leather's corners where it meets the plate going with the plate, and the
   preview draws each model's textures with its triangles coloured by plate (`uv-<model>.png`) to
   mark them from. Iron's bracers and couters are only their metal now.

   **Every set marked again the same way** (men's and women's): every hard cuff rigid on the
   forearm to the end of its flare (split by place where the game blends the flare towards the
   hand), the plate over the back of the hand rigid on the hand and coming apart from the cuff at
   its seam, and only the hard part of each piece rigid: leather under the forearm, linings, gloves,
   mail and straps bend. Couters are their plates only (ebony's rides the upper arm alone: its fan
   runs 9-12 cm up the arm, as the game keeps it); the Legion's elbow band is rigid now (it crumpled)
   with its strap soft; the Blades' and the dwarven forearm plates rigid to the wrist; the daedric
   men's cuffs (whose rule chose nothing) and the women's arms (which had no rules) have plates now.
   The women's iron, ebony, orcish and leather models have pieces of their own where the men's boxes
   didn't fit. Each checked in the preview (wrists both ways, palms turned, a slight and a full
   bend of the elbow, a squat). Left as they were: fur (soft), the knees (no rigid strap behind
   one).

   **Still to look at, from the preview:** steel's glove shows at the thumb's root through the
   seam when the wrist bends in; thin dark gaps open at the cuffs' seams as the wrist bends (iron,
   steel, glass, ebony, mithril, the Legion's), and between daedric's cuff band and hand plate
   bending in; at a full bend, ebony's elbow fan dips into the forearm (as in the game's skinning),
   the Blades' elbow shows a small notch, and the women's bare elbows a dark patch (the round
   joints, not the marks). Unmarked still: the steel sleeve's plates under the rerebrace, glass's
   lames under the pauldron (rigid, they'd poke through it raising the arm), knuckle plates.

3. **The player couldn't walk, and the game started "tabbed out" on the PC.** Two things.
   * **Too laden to walk**, in every armour boot: the playtest's .bat gave all four heavy sets at
     once (341 lb) on top of what the Nord carries, and strength 100 wasn't enough: the game's own
     movement speed was 0 (the adapter's `movement: ... at 0 units/s`), as Oblivion has it for an
     over-encumbered player. The .bats now wear one set at a time (`playtest.bat` the save's iron;
     `playtest-steel`, `-glass`, `-ebony`, `-daedric` give their set, put it on and drop the iron).
   * **The window off-screen**, in every feature's boot since about 14:30: `Oblivion.ini` had
     `iLocation X`/`Y` at -32000 (a game quit while minimised saves that), so the window was made
     off every monitor, out of reach on the desktop (the game still plays in the headset: it's
     always the active window to the game). The ini's values were put back to 0, and the adapter
     opens a window the game makes off every monitor at the screen's corner.

4. **"LOOKS AMAZING"; the pointing finger slightly through the steel; a gap at the glass wrists
   bending a certain way (and likely on many sets).** The gap: a rigid cuff stays straight on the
   forearm as the hand bends out of it, so its open end showed the armour's hollow inside (the
   game never did: its armour bends with the body, closed). Fixed for every set at once: the
   adapter sends the race's skin under what's worn as well (`<key>:under`), and the runtime draws
   it 1 cm away from the eye, so it never shows through armour, only where nothing worn is in front
   of it: the wrist in a cuff's open end, the arm where plates part over a joint (the armpit under
   a pauldron, the inside of a bent elbow). Tried first and dropped: the armour's own soft surface
   under the plates (it showed as dark insides, and the breastplate's neck swung out over the
   chest). The finger: not seen in the preview (`--fingers point`, the `hand` view); what shows
   is the leather glove under steel's plates at the thumb's root, through the seam with the wrist
   bent in.

## Not done

* Helmets and shields (the helmet isn't drawn in first person; the shield is held), and the other
  armours (Shivering Isles' amber and madness, the Knights of the Nine, the town guards, the arena,
  the Dark Brotherhood...): they get the round joints, not plates. Marking one is a few lines.
* Fingers of plated gauntlets stay soft (round knuckles), not articulated plates.
* The pauldrons lift only the game's share of the arm's raise (20-40%): arms straight up, the top
  of the upper arm shows under them, as in the game.
* Oblivion's upper-arm twist bones move rigidly with the upper arm here (the game's animations key
  them; reading those is a separate piece of work), so the shoulder's plates don't roll less than
  the arm when it rolls.
* No setting in the headset's menu to switch the marks off: the environment variable only.
