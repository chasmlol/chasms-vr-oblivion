# Playtest: alchemy by hand

Save: rustvr-alchemy

## Playing it

From the repository's root:

```
powershell -File tools\oblivion.ps1 -Save rustvr-alchemy -ConsoleFile tools\alchemy-row.txt
```

It builds main, installs it, and boots into Odiil Farm's farmhouse, standing in its entrance;
`powershell -File tools\oblivion.ps1 -Close` puts it away. `tools/alchemy-row.txt` sets out the
row of tables (timed console lines, `crates/oblivion/src/console.rs`): two seconds in, four tables
stand in a row along the wall on your right, going away from you into the main room; two seconds
later, what's on them. Everything set out is yours, so nothing there is stealing.

`rustvr-alchemy` is the developer's own save, not in the repository. It was made from
`rustvr-city`, which loads in the same farmhouse, with a bench set out by `PlaceAt`. To make one,
load a save that stands in the farmhouse's entrance and save it as `rustvr-alchemy`
([DEVELOPING.md](../DEVELOPING.md), "Test saves").
The row's `Place` lines are that cell's own coordinates, so only a save in the farmhouse will do.
The file's first lines disable the old save's own bench (references `FF000C48` to `FF000C54`, made
in that save) and take the pack's Apple, Carrot, Boar Meat, Venison and Aloe Vera Leaves: in any
other save those ids are other things or nothing, so drop those lines there.

| Table | Mortar and apparatus | Left | Right |
|---|---|---|---|
| 1, nearest | Novice Mortar & Pestle alone | Restore Fatigue: Apple, Carrot | Restore Health: Boar Meat, Venison |
| 2 | Apprentice Mortar & Pestle, Alembic, Calcinator | Restore Magicka: Flax Seeds, Steel-Blue Entoloma Cap | Resist Disease: Bergamot Seeds, Garlic |
| 3 | Journeyman Mortar & Pestle, Alembic, Calcinator, Retort | Cure Disease: Mandrake Root, Root Pulp | Resist Poison: Foxglove Nectar, Motherwort Sprig |
| 4, by the beds | Master Mortar & Pestle, Alembic, Calcinator | Damage Health: Nightshade, Harrada | Damage Fatigue: Rat Meat, Dreugh Wax |

Table 4 makes poisons, so it has no retort: a retort only strengthens what does good. The save's
pack holds a Novice and an Apprentice Mortar & Pestle, a Novice Alembic and Calcinator, an
Apprentice Retort, and the character's own ingredients (among them Wisp Stalk Caps and Stinkhorn
Cap, which make Damage Health).

The research and the design: `docs/design/alchemy.md`.

## What it does

`crates/oblivion/src/alchemy.rs`, with the runtime's bowls (`framework/crates/runtime/src/rig/
receptacle.rs`) and sheets (`rig/sheet.rs`).

- **A mortar and pestle in the world is a bench.** It's two pieces: the bowl, and the pestle
  leaning in it, handle up. The runtime draws both from the game's model, whose own copy is
  hidden. A mortar dropped from the pack (the game's own drop) becomes a bench where it falls.
- **Ingredients go in by hand.** One let go over the bowl's mouth (up to 30 cm over the rim, or
  just past it) drops in, shrunk to fit, with the game's ingredient-down sound. At most four, each
  kind once: the game's menu rules at any skill. Ingredients within 1.5 m of the mortar count at
  it, so the row's tables don't share.
- **The card** over the mortar (20 cm wide) shows what the game would make: the potion's name in
  the menu's title font, each effect's icon and the menu's own words for it, the ingredients'
  icons along its foot and the apparatus that count at the right. It's cut from the alchemy menu's
  art (`menus\alchemy\alchemy.dds`) in its fonts and brown ink, painted by the runtime.
- **The game works it out.** Whenever the bowl changes, the game's `AlchemyMenu` is made with the
  bench's apparatus, its slots filled, its effects read and the menu destroyed, all in one frame
  and without its menu mode: the skill's known effects, the apparatus's qualities, the naming. The
  alembic, calcinator and retort count within 75 cm of the mortar, the best of each kind;
  apparatus in the pack doesn't count at a bench.
- **Grinding.** Take the pestle by its handle and work its tip round the bowl's floor: 0.45 m of
  the tip's travel per ingredient, about three turns. The pestle meets the bowl's floor and walls.
  A low buzz while it grinds, a pulse each stroke, and the game's organic crunch (`COrganicSmall`)
  from the mortar; ground ingredients shrink to 45%.
- **The potion** is the game's own Create: the ingredients picked up (the game's pick-up, stealing
  included) and used up, the effects, name, value, weight, skill use, "You have created a new
  potion" and its sound. The bottle (the game's own drop of it) rises out of the mortar and waits
  there 45 s for a hand, then settles. Its card in the hand is the game's. Let go of in a slot on
  the body, it goes into the pack.
- **No shared effect:** grinding anyway gives the game's unused `sPotionFailed` words and its
  `ITMIngredientNothing` sound, and the ingredients are lost.
- **Not in combat:** as the game's own use of an apparatus (unless `iAllowAlchemyDuringCombat`),
  with its words, and the ground bowl waits.
- **Reaching in** below the rim takes out what's there, full size again; above it, the hand takes
  the pestle or the mortar. Carried, everything in the bowl rides along, pestle and all; turned
  over, it spills. The pestle let go with its tip over the bowl settles back in.
- **The recipe note.** Come within 2 m of a mortar, facing it (within about 35° of where you look),
  and a note is put down 20 cm in front of it, towards you: the book menu's scroll paper
  (`menus\book\scroll_background.dds`) in the game's handwriting (font 5). It reads "Potions you
  can make", a line each: the effect's icon, the game's name, what it's made of. The most
  valuable first; of mixes with the same effects, the most valuable. Each is a pair of what you
  have (the pack, and what lies within reach of the bench) run through the game's menu with the
  bench's apparatus, and a single ingredient for a master.
  - Tap a line with an empty hand's index fingertip (down on it, then lifted 2.5 cm off): a page
    turn, and the note shows that potion, the game's words for its effects, and its steps ("Put
    in the Boar Meat", ", from your pack" for one only in the pack, "Take the ... out of the
    mortar" for what doesn't belong, "Grind them with the pestle", "Take your potion"). Each step
    fades as it's done. Taking the bottle puts the list back, worked out again. "Other potions"
    goes back; "Turn over (1 of 2)" turns the list's page.
  - It can be taken up to read nearer (the game's scroll sounds); it goes when its bench does.

**The view up against things** (Oblivion everywhere, `crates/oblivion/src/movement.rs` and
`collision.rs`, the framework's `set_lead` in `framework/crates/adapter/src/tracking.rs`). The
game's body reaches 0.30 m round you, so it used to stop the view 30 to 35 cm from a table or a
wall. Now whatever of your step the game's body doesn't go (stopped, or sliding along something),
the view goes on, as far as your own body has room:
- something low (a table, a counter, a bed): until your front is at its edge, 5 cm ahead of your
  eyes, your head over it;
- a wall or anything at head height: until the eyes are 12 cm from it, so the view doesn't fade;
- what isn't the world's fixed things (a door, a person): the head stays 12 cm inside the body.

Walking on freely, the view comes back onto the body by a fifth of the way walked. Room-scale, the
body is walked after your head and the same lead lets you lean further over a table. The adapter's
log says every 2 s while the view leads: `movement: the view ... m past the game's body`.

**Height, sitting or standing** (every game, `framework/crates/runtime/src/mechanics.rs`). Your
upright head height (standing, or sat up in a chair) is the character's eye height, however the
view was recentred:
- putting the headset on: once it's been still on your head 1.5 s, your height is taken there;
  until then the view is at the character's eye height;
- held higher for a second (standing up), the upright height rises to it; a moment higher (a
  stretch, on tiptoe) doesn't count; lower by a slouch (under half a crouch), it settles there
  slowly; lower than that, a crouch however slow, it stays, so ducking still crouches outside
  seated mode;
- Recenter takes your height as it is then.

## What to try

1. **The note.** Step up to table 1: the note goes down in front of the mortar, facing you. Expect
   the table's two potions and Damage Health from the pack on every table (one Damage Health line
   on table 4). Tap a line, follow the steps, take the bottle and watch the list come back. Take
   the note up in your hand.
2. **Into the bowl.** Boar Meat in: one ingredient makes nothing at a novice's skill, so the card
   shows its icon and no name. Venison in as well: "Restore Health" and its effect.
3. **Grind**, round or back and forth, pressing in. Then take the bottle.
4. **A mix that fails:** Apple and Boar Meat share no known effect; grind them anyway.
5. **Changing your mind:** reach into the bowl and take something out; the card updates.
6. **Carry it** in one hand and grind with the other; turn it over.
7. **The rules:** a second apple won't go in; a fifth ingredient won't. Move the Novice Alembic onto
   table 3 beside its Journeyman one (the Journeyman still counts); take the alembic away and the
   effects change.
8. **Down the row:** each table's card and note are its own; carry an ingredient to another table
   and it counts there. Table 4's potions come in the game's poison bottle.
9. **The game's own menu:** use a mortar from the pack in the inventory: the game's alchemy menu,
   floating as every menu does.
10. **Up against things:** walk into the tables with the stick, straight on and at a slant; rub
    along a wall; walk into the door, the cupboard, the beds, the fireplace; step up to a table on
    your feet and lean over it. Look out for the view jumping, drifting in the open, going into
    anything, or fading at walls.
11. **Height:** boot with the headset on the desk and put it on sitting; stand up, sit down; turn
    seated mode on and off; press Recenter. Look out for floating or sinking that lasts, the
    body's feet off the floor, ducking not crouching (seated mode off) or crouching when you lean
    (seated mode on). The Body tab shows "eyes at" your upright height.

## Settings

| Where | Setting | Default | Range |
|---|---|---|---|
| Body tab | Seated mode (you see from standing height; ducking doesn't crouch), `comfort.seated` | off | |
| Body tab | Play at my real height (otherwise the game character's), `comfort.real_height` | off | |
| Body tab | Your height (0 = measure as you stand), `body.height` | 0 | 0 to 2.2 m |
| Body tab | Measure my height (stand up straight): refuses eyes under 1.35 m (a grown-up sitting) | | |
| Body tab | Height offset, `comfort.height_offset` | 0 | -0.5 to 0.5 m |

The game's own `iAllowAlchemyDuringCombat` decides whether a bench works in combat.

## Decisions

- **Everything the game decides is the game's,** through its own alchemy menu made for a moment:
  nothing about the potion is re-implemented. Making the menu would stop the world, so it comes
  off the interface's menu stack in the same frame and the mode is put back.
- **The card is painted by the runtime from the game's files**, not the game drawing its menu: the
  HUD layer has no alpha, so brown text on parchment would wash out added to the view.
- **Four ingredients at any skill, each kind once:** the game's menu rules. Skill limits which
  effects you know, not how many go in.
- **Grinding counts the hand's travel with the tip low in the bowl**, not the tip's own: pressed
  into the floor, the tip shivers there, and that would grind on its own. Your empty hand passes
  through the mortar, as it does through any of the game's props.
- **The failure is ours:** the game has none (its Create is just off), so its unused
  `sPotionFailed` words and `ITMIngredientNothing` sound stand in.
- **Oblivion has no grinding sound**, so a stroke plays its organic crunch; an empty mortar is
  silent.
- **The note is the bench's own,** waiting until you're at the bench and facing it: put down as
  soon as the bench appeared, the far tables' notes faced sideways to anyone standing at them. Its
  lines are pairs because three or four ingredients only add to a potion. Its words ("Potions you
  can make", the steps) are ours, since the game has no recipes; every name on it is the game's.
- **The row of tables:** four of the farmhouse's own `LowerClasstable03` (1.21 by 0.85 m, 0.83 m
  high) along the east wall, the house's longest clear stretch, 1.38 m apart middle to middle so
  each table's ingredients stay its own. One mortar per grade but expert (four fit). Each table's
  two potions share their first effect, the only one this character's skill knows. The pack's food
  is taken so each note lists its own table's potions; the character's ingredients stay, so "from
  your pack" shows. Set out at every boot because no new save could be made then.
- **The view leads the game's body rather than the body being thinned.** The body's shape is round
  (made from the skeleton's `BBX` box: 20.4 units times the actor's scale, and Havok's 0.7 round
  that), and its bottom is a cone sized to that radius that climbs steps: thinning it would change
  stairs. The lead is the step asked of the body less the step taken; only steps the player asked
  count (falling, swimming or being pushed, the view stays on the body), and a jump of more than
  25 cm in a frame (a load, a door) puts it back. The room that way comes from the world's fixed
  things, looked for every 2.5 cm of height and 12 cm either side of the eyes' line.

## Not done / known issues

- **The bowl after a save:** ingredients left in it come back loose where they lay, pushed out of
  the mortar by Havok.
- **Variety:** the potion is always the game's `Potion01` bottle, or `PotionPoison` (the game's own
  choice). No paste in the bowl, only the ingredients shrinking.
- **The Mixing Bowl** (an apparatus of the mortar's kind with no pestle) isn't a bench.
- **More than four benches at once:** the runtime has room for eight cards, a card and a note each
  bench; a fifth filled bowl may lose its card or note.
- **A save with the row in it** would boot straight into it; it's set out at each boot.
- **The body walked after your head while the view leads it:** standing at a table with your head
  more than 5 cm past where the view leads, the game still walks its body into the table.
- **Never checked on the headset:** the row as set out (tables level and clear of the walls, the
  post and the beds; nothing falling through; the hitch as about 270 lines of script run at four
  seconds); the note (where it lies and which way it reads, whether every tap takes, how the
  handwriting reads, its sounds, the list's working out, a couple of milliseconds a pair); the
  view's lead and the upright height; the sounds and haptics; four ingredients, a duplicate
  refused, a master's single-ingredient potion, combat refusing a potion, grinding in the novice
  and apprentice mortars, and saving with things in the bowl. The emulated headset did check the
  bench, the card, grinding, the potion, the failure, carrying and spilling, the game's own menu
  afterwards, and a mortar dropped from the pack.

## History

Newest first.

- **Playtest 3:** the view still stopped short when the game's body slid along a table's edge
  instead of stopping (only 17 cm more), and the view floated, its height taken from wherever the
  headset was at the first recentre. Now the view goes on with whatever of the step the body
  didn't go, and the upright height is found as the headset's worn.
- **Playtest 2:** the view stopped 30 to 35 cm from tables and walls, the hands short of what was
  on them; the save's one table stood in the cramped entrance. The view now leads the body up to
  what's there; the row of four tables replaced the one, and each note waits until you face its
  bench.
- **Playtest 1:** the recipe note was built after it.
