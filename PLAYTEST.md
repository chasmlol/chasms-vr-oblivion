# Playtest: armour that moves like armour

Save: rustvr-base

The Nord in iron (and the thief's boots) from `rustvr-base`. Start it with:

```
powershell -File tools\play.ps1 armour
```

To bring the other vanilla sets along, add `-Give` (each set's four pieces; equip them from the
inventory), and strength enough to carry four heavy sets on top of the iron without being stuck in
place (the playtest's `playtest.bat` boots it this way):

```
powershell -File tools\play.ps1 armour -Give "000229A2 1,0001C6D8 1,000229A3 1,000229A5 1,00036343 1,00036340 1,00036342 1,00036341 1,0003635B 1,00036358 1,0003635A 1,00036359 1,0002AD85 1,00036352 1,00036354 1,00036353 1" -Commands "player.setav strength 100"
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
