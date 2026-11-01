# Playtest: armour that moves like armour

Save: rustvr-base

## Playing it

From the repository's root:

```
powershell -File tools\oblivion.ps1 -Save rustvr-base
```

It builds main, installs it, and boots the Nord in iron (and the thief's boots).
`powershell -File tools\oblivion.ps1 -Close` puts it away. `rustvr-base` is the developer's own
test save, not in the repository; any save with the player in armour does, and one can be made
from any save with `-At <cell>`, saved as `rustvr-base` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

To wear another vanilla set, give its four pieces, put them on and drop the iron they replace
(here steel):

```
powershell -File tools\oblivion.ps1 -Save rustvr-base -Give "000229A2 1,0001C6D8 1,000229A3 1,000229A5 1" -Commands "player.equipitem 000229A2|player.equipitem 0001C6D8|player.equipitem 000229A3|player.equipitem 000229A5|player.removeitem 0001C6D1 1|player.removeitem 0001C6D2 1|player.removeitem 0001C6D0 1"
```

One set at a time: four heavy sets carried at once (341 lb on top of the Nord's own things) leave
the Nord too laden to walk, even with strength 100 (the game's movement speed is then 0).

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

The research: `docs/design/armour.md`. The guide to marking armour: the framework's
`docs/ARMOUR.md`.

## What it does

The player's body (`framework/crates/avatar`, put together by the runtime's `body.rs`):

- **Every joint keeps its volume.** Skin, clothes and armour are skinned about each vertex's centre
  of rotation (Le and Hodgins 2016) instead of the game's linear blending: a fully bent elbow stays
  round instead of pinching, a turned wrist stays round instead of twisting into a ribbon, knees
  keep their shape in a crouch.
- **Armour plates are rigid.** Every vanilla set, men's and women's, is marked in
  `content/armour/oblivion.toml`: its hard parts move as one piece each, riding the bones they
  belong to.
  - Gauntlets: the hard cuff stays straight on the forearm to the end of its flare and rolls with
    it; the plate over the back of the hand moves with the hand, coming apart from the cuff at its
    seam and sliding under or over the cuff's rim as the wrist bends; the wrist bends between them.
  - Elbows: the couter is a rigid cap turning half the elbow's bend (ebony's rides the upper arm
    alone: its fan runs up the arm, as the game keeps it).
  - Shoulders: pauldrons hang as the game drapes them, rigid, lifting part of the way with the arm.
  - The upper arm's plates on the upper arm, tassets hinged at the hips, cuisses on the thighs, knee
    cops half and half about the knee, the boots' shins, feet and toe caps each rigid with the ankle
    soft between, breastplates rigid at the chest with the belly soft.
  - Only the hard part of each piece is rigid: leather under the forearm, linings, gloves, mail,
    straps, fur and cloth bend; boiled-leather bracers and boots are rigid. Studs, rivets and
    buckles move with their plate.
- **Two-sided plates show their fronts:** the iron pauldrons and tassets (and other sets' double
  surfaces) no longer show their dark backs.
- **The body under the armour.** The adapter sends the race's skin under what's worn as well
  (`<key>:under`); the runtime draws it 1 cm further from the eye and 5 mm inside its own skin, so
  it never shows through armour, only where nothing worn is in front of it: the wrist in a cuff's
  open end, the armpit under a pauldron, the inside of a bent elbow. It stays inside the gloves'
  outlines.

The runtime's log (`%LOCALAPPDATA%\RustVR\rustvr.log`) names the plates each piece you wear made
(`avatar: armor\iron\m\gauntlets.nif (Iron gauntlets): 4 plates: ...`), and says where the marks
were read from.

## What to try

1. **Forearms and hands.** Hold your hands out; turn the palms up and down (the cuffs roll and stay
   round); bend the wrists down and back as far as they go (the cuff straight, the hand plate with
   the hand, the wrist bending between, no hollow showing in the cuff's open end).
2. **Elbows.** Curl a hand to the shoulder: the couter stays a rigid cap, the inside of the elbow
   keeps its volume.
3. **Shoulders.** Reach forward, cross your arms, raise them overhead. Watch the pauldrons in a
   mirror (pull your gripped hands apart in front of your chest, or the Body tab's "Stand a mirror
   in front of me"): they hang and lift a little with the arm, never bending.
4. **Look down and crouch:** the breastplate's chest flat while the belly bends, the tassets
   swinging, the greaves, shins and feet rigid, the knees round.
5. **Other sets,** one at a time (the body is put together again a second or two after each
   change). Steel, glass, ebony and daedric are heavy plate; elven, mithril and the Legion's are
   lighter plate over mail; chain mail, fur and leather are mostly soft.
6. **Against the game's own skinning.** In the PowerShell window, `$env:RUSTVR_SKINNING = "linear"`
   before booting: everything bends as the game bends it. `"centres"`: round joints, no plates.
   `Remove-Item Env:RUSTVR_SKINNING` for the marks again.

## Settings

No menu setting. Environment variables, read when the body is put together:

| Variable | What it does |
|---|---|
| `RUSTVR_SKINNING` | `linear`: the game's own skinning; `centres`: round joints, no plates; unset: the marks. |
| `RUSTVR_ARMOUR` | A marks file to read instead of `content/armour/oblivion.toml` (to try an edit without rebuilding). |

A modder's own marks for their own models go in `local_assets/oblivion/armour/*.toml`, read after
the game's. The marks file is read at run time, so an edit takes effect the next time the body is
put together.

**Marking armour.** One `[[piece]]` a model (or a male and a female model alike), each with
`[[piece.plate]]`s choosing islands (the pieces of a mesh) by mesh, bone, side, box and size, by
corners within a place (`corners_within`) or where its texture draws them (`uv_within`), coming
apart at a `seam`, and saying what each rides. The preview draws a set through a sweep of poses:

```
cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-avatar --bin rustvr-armour
target\release\rustvr-armour oblivion --set steel            # captures\armour\steel\*.png
target\release\rustvr-armour oblivion --set steel --list     # islands and plates
```

Each `<pose>.png` has a row per way of skinning (the game's, round joints, the marks) and a column
per view (front, side, back, the wearer's eyes, the forearm, the shoulder, the elbow side on, the
wrist); `islands.png`, `plates.png` and `uv-<model>.png` colour what the marks choose. More options:
`--poses`, `--views`, `--skin`, `--visibility hands|arms|full`, `--fingers relaxed|open|fist|point|
trigger`.

## Decisions

- **Plates without new bones.** A plate is its vertices all given the same weights and the same
  centre of rotation, so the skinning that rounds the joints moves it rigidly; riding two bones, it
  turns by their shares about the joint between them (the AAA helper joint, without the helper).
  Nothing extra per frame on the CPU.
- **The game's weights are the default riders** (the artist's say of what a plate follows, made
  rigid), shaped at rest with the arms at the sides (the models are made with the arms out level).
  Marks name riders where the game's are wrong for a rigid piece: couters half and half,
  breastplates on the chest only (the game's carry a little of the head), vambraces on the
  forearm's twist.
- **Hard parts chosen by place and texture.** The game blends a cuff's whole flare between the
  forearm and the hand, and one island often holds metal and the leather under it; the game's own
  texture layout says which is which (its specular mask is too faint to go by).
- **Two-sided plates:** front faces drawn first, back faces a little further off, so a double
  surface shows its front and a cut-open sleeve still shows its inside.
- **Overlaps:** a plate on a higher layer is drawn 2.5 mm a layer nearer where it meets a lower one
  (pauldron over the upper arm's plates, daedric lames over one another). No collision between
  plates: their riders keep them apart in the swept poses.
- **The skin under the armour, not the armour's own soft surface:** tried first, that showed as dark
  insides, and the breastplate's neck swung out over the chest.
- **The marks live in this repository,** read by the runtime at run time. The arm IK is untouched.

## Not done / known issues

- **Helmets and shields** (the helmet isn't drawn in first person; the shield is held), and the
  other armours (Shivering Isles' amber and madness, the Knights of the Nine, the town guards, the
  arena, the Dark Brotherhood...): round joints, no plates. Marking one is a few lines.
- **Fingers of plated gauntlets** stay soft (round knuckles), not articulated plates.
- **The pauldrons lift only the game's share** of the arm's raise (20-40%): arms straight up, the
  top of the upper arm shows under them, as in the game.
- **Upper-arm twist bones** move rigidly with the upper arm (the game's animations key them), so
  the shoulder's plates don't roll less than the arm.
- **No menu setting** to switch the marks off: the environment variable only.
- **Seen in the preview, not yet fixed:** steel's glove shows at the thumb's root through the seam
  with the wrist bent in; thin dark gaps open at the cuffs' seams as the wrist bends (iron, steel,
  glass, ebony, mithril, the Legion's) and between daedric's cuff band and hand plate; at a full
  bend ebony's elbow fan dips into the forearm (as in the game's skinning), the Blades' elbow shows
  a notch, the women's bare elbows a dark patch (the round joints). Unmarked: the steel sleeve's
  plates under the rerebrace, glass's lames under the pauldron (rigid, they'd poke through it),
  knuckle plates.
- **Never checked in the game:** the women's bodies and the beast races (Khajiit and Argonian have
  their own skeleton); sets other than iron, steel, glass and daedric (only in the preview).

## History

Newest first.

- **Playtest 5:** the wrists in the openings were right, but the hands showed slightly round the
  edges of the glass gloves (glass's fingers are up to 3.5 mm slimmer than the bare hand, its hand
  up to 6). The body under the armour is now drawn 5 mm inside its own skin.
- **Playtest 4:** "LOOKS AMAZING"; the pointing finger slightly through the steel; a gap at the
  glass wrists bent a certain way, showing the armour's hollow inside. The race's skin is now drawn
  under what's worn, for every set. The finger wasn't seen in the preview; what shows there is
  steel's leather glove at the thumb's root.
- **Playtest 3:** the player couldn't walk (the boot gave all four heavy sets at once, 341 lb): now
  one set at a time. The game's window had opened off every monitor (`Oblivion.ini`'s `iLocation`
  at -32000, saved by a game quit while minimised): the adapter now opens such a window on screen.
- **Playtest 2:** the cuffs held, but the leather under the bracer didn't bend and the couter's
  strap went through the arm at the least bend. Marks can now choose a plate's corners by where its
  texture draws them (`uv_within`); every set was marked again, only the hard part of each piece
  rigid.
- **Playtest 1:** the iron cuffs bent like cloth at the wrists: the cuff and the hand plate are one
  island, the game blending the cuff's flare between forearm and hand. Marks can now choose corners
  by place (`corners_within`) and let a plate come apart at its `seam`; the iron cuff is split at
  the end of its flare.
