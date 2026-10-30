# Playtest: alchemy by hand

Save: rustvr-alchemy

`playtest.bat` boots straight into it (`tools\play.ps1 alchemy -Commands "RunFile
<worktrees>\alchemy\tools\alchemy-row.txt"`): Odiil Farm's farmhouse, standing in its entrance. Two
seconds in, four tables stand in a row along the wall on your right, going away from you into the
main room; two seconds later, what's on them. Everything there is yours. Each table has a different
mortar and apparatus at the back, and the ingredients of two potions at the front, one potion's pair
on the left and the other's on the right:

| Table | Mortar and apparatus | Left | Right |
|---|---|---|---|
| 1, nearest | Novice Mortar & Pestle alone | Restore Fatigue: Apple, Carrot | Restore Health: Boar Meat, Venison |
| 2 | Apprentice Mortar & Pestle, Alembic, Calcinator | Restore Magicka: Flax Seeds, Steel-Blue Entoloma Cap | Resist Disease: Bergamot Seeds, Garlic |
| 3 | Journeyman Mortar & Pestle, Alembic, Calcinator, Retort | Cure Disease: Mandrake Root, Root Pulp | Resist Poison: Foxglove Nectar, Motherwort Sprig |
| 4, by the beds | Master Mortar & Pestle, Alembic, Calcinator | Damage Health: Nightshade, Harrada | Damage Fatigue: Rat Meat, Dreugh Wax |

Table 4 makes poisons, so it has no retort: a retort only strengthens what does good. In your pack: a
Novice and an Apprentice Mortar & Pestle, a Novice Alembic and Calcinator, an Apprentice Retort, and
the ingredients the save's character already carried (among them Wisp Stalk Caps and Stinkhorn Cap,
which make Damage Health). The save's own table in the entrance is gone, and so are the pack's Apple,
Carrot, Boar Meat, Venison and Aloe Vera Leaves: the game says so as it takes them, a message each.

## What to try, in order

0. **The recipe note.** Turn to the first table and step up to it. Once you're within 2 m of the
   mortar and facing it, a note is put down on the table about 20 cm in front of the mortar, facing
   you: the game's own scroll paper (the one its book menu opens a note on, ribbon and all), written
   in the game's handwriting. Each table's note waits for you like this, so it faces you where you
   stand at that table.
   - It reads "Potions you can make", and under it a line for each potion: its effect's icon, the
     game's name for it, and what it's made of ("Restore Health: Boar Meat, Venison"). The most
     valuable first. They're worked out by the game's own alchemy, out of what's on that table and
     in your pack, so only effects your skill knows are there (your skill knows each ingredient's
     first effect). Expect the table's two potions, and Damage Health from your pack (Wisp Stalk
     Caps, Stinkhorn Cap) on every table. On table 4 that's one Damage Health line: of potions with
     the same effects, the list keeps the most valuable.
   - Tap a line with your index fingertip (an empty hand: down on it, felt as a tap in the hand,
     and lift it off) to choose it: a page-turn sound, and the note shows that potion, the game's words for its effects, and the steps:
     "Put in the Boar Meat", "Put in the Venison", "Grind them with the pestle", "Take your
     potion". Each step fades as you do it. An ingredient that's only in your pack says ", from
     your pack". Something in the bowl that isn't in the recipe gets a "Take the ... out of the
     mortar" step first.
   - When you take the bottle, the note goes back to its list, worked out again for what you have
     left. "Other potions" at its foot (or the potion's name at its top) goes back too, and with
     more potions than fit, "Turn over (1 of 2)" at the foot turns the list's page.
   - Take it up in your hand to read it nearer (the game's scroll sound as it's taken up and put
     down), and touch it with the other hand's finger while you hold it.
1. **The bench.** The mortar and pestle is two pieces now: the bowl, and the pestle leaning in it,
   its handle sticking up. (The runtime draws both from the game's own model; the game's own copy is
   hidden while you're near it.)
2. **Boar Meat into the mortar** at table 1. Take it from the table and let it go anywhere over the bowl's
   mouth, up to about 30 cm over the rim, or just past the rim. It drops into the bowl and lies on
   its floor, shrunk to fit. You hear the game's ingredient-down sound from the mortar. A card comes
   up over the mortar, cut from the game's own alchemy menu: its parchment, with the boar meat's icon
   along its foot and, at the right, the icons of the apparatus that count. One ingredient makes
   nothing at a novice's skill, so there's no name or effect yet.
3. **Venison in as well.** The card now shows what the game would make: "Restore Health" in the
   game's title font, the restoration icon and "Restore Health ... pts for ... secs on Self" in the
   menu's own words. That's the game's alchemy working it out: its skill rule for which effects you
   know, your mortar's, alembic's, calcinator's and retort's qualities, and its naming. The same
   potion is stronger at a better table.
4. **Grind.** Take the pestle by its handle and work it round the bottom of the bowl, pressing in.
   Rough circles or back and forth both count; about three turns per ingredient. You feel a low
   buzz while it grinds and a pulse each stroke, and the game's organic crunch plays from the
   mortar. The ingredients shrink as they're ground.
5. **The potion.** When everything's ground, the game makes it: its own Create, so the effects,
   name, value (7), weight (2), the Alchemy skill use, "You have created a new potion" and the
   potion sound are all the game's. The ingredients are used up, and the bottle (the game's own
   potion model) rises out of the mortar and waits over it. Take it. Its card in your hand is the
   game's (name, value, weight). Let it go at a hip or shoulder and it goes into your pack. Let go
   of the pestle with its tip over the bowl and it settles back into the mortar.
6. **A mix that doesn't work.** Apple and Boar Meat, say: they share no effect you know, so the card
   shows the two icons and no effect. Grind them anyway and you get the game's own words for it,
   "Your potion failed and your ingredients were lost", with its "ingredient had no effect" sound.
   The ingredients are gone.
7. **Changing your mind.** Reach into the bowl, below the rim, and grip: you take out what's in it,
   full size again, and the card updates. Above the rim, your hand takes the pestle or the mortar
   instead.
8. **Carry it.** Pick up the mortar with things in it. They ride along, pestle and all, and you can
   hold it in one hand and grind with the other. Turn it over and everything falls out.
9. **The rules.**
   - Each kind of ingredient goes in once, as in the game's menu; a second apple won't go in.
   - At most four ingredients go in; the game's menu has four slots at any skill.
   - The alembic, calcinator and retort count only within 75 cm of the mortar, the best of each
     kind. Move the Novice Alembic from your pack onto table 3 beside its Journeyman one: the
     Journeyman still counts. Take the alembic away and the effects change.
10. **Down the row.** Each table is a bench of its own, with its own card and note. A bench counts
   the ingredients within 1.5 m of its mortar, and the tables are far enough apart that one table's
   don't count at the next. Carry an ingredient from one table to another and it counts there.
   Table 4's potions are poisons, in the game's poison bottle.
11. **The game's own menu still works.** Use a mortar from your pack in the inventory and you get the
   game's alchemy menu as before.

## What was checked of the recipe note

Nothing in the game, emulated or real: it was built after the first playtest and only compiled,
with the unit tests of every crate touched passing. Look out for:

- **Where it lies.** It's put 20 cm in front of the mortar, towards your head as you come to the
  bench (within 2 m of the mortar, the mortar within about 35° of the way you look), and falls a
  centimetre onto the table. If the table's edge is nearer than that, it may fall to the floor (pick
  it up from there). If a note never comes, or comes at a slant, say at which table.
- **Which way round it reads.** Its writing should face you the right way up. If it's mirrored or
  upside down, say which.
- **Touching it.** Lying on the table, your fingertip goes through the paper onto the table, and the
  touch is read there. A tap counts when the finger lifts off (2.5 cm), so taking the note up never
  chooses the line your finger landed on. Whether it takes every tap, or misses some, I can't know
  without hands.
- **How it looks.** Lit by the room as the mortar is; whether the handwriting is big enough, and
  whether it shimmers seen at a slant.
- **Sounds:** the scroll's pick-up and put-down sounds and the page turn, all the game's own.
- **The list's working out** runs the game's alchemy menu once for each pair of ingredients that
  could share an effect, a couple of milliseconds each, about one a frame. With the save's ten kinds
  of ingredient that's under a second. If you hear menu sounds repeating while it does, or the game
  stutters, say so.

## Up against things, all over the game

The game's body is wider than yours: 30 cm round you, where your front is a few centimetres ahead
of your eyes. Everywhere in the game, it stopped your view 30 to 35 cm from a table, a counter or a
wall, with your hands and IK body short of what you walked up to. (The second playtest's version
only carried the view on when the game's body stopped dead: push a little off square and the body
slid along the edge instead, and the view went no closer. Then only 17 cm more.)

Now whatever of your step the game's body doesn't go (stopped, or sliding along something), your
view goes on, at your own pace, as far as your body has room that way:
- **Something low** (a table, a counter, a bench, a bed): until your front is at its edge, about
  5 cm ahead of your eyes, your head over it. Up to about 28 cm past where the game's body stops.
- **A wall** (or anything at your head's height): until your eyes are 12 cm from it, so the view
  doesn't fade.
- **What isn't the world's fixed things** (a door, a person, a creature): your head stays 12 cm
  inside the game's body, about 17 cm on.

Walking on freely, the view comes back onto the game's body over the next metre or so. With your
feet (roomscale), stepping up to a table works the same: the game's body is walked after you and
stops, and what it didn't go comes off how far your head is from it, so you can lean that much
further over the table too.

- **Try:** walk into the tables with the stick, straight on and at a slant; walk along a wall
  rubbing it; walk into a door, the cupboard, the beds, and the fireplace. Step up to a table on
  your feet and lean over it.
- **Look out for:** the view jumping as you push into something or walk away from it; the view
  drifting while you walk in the open; your view going into or through anything; the view fading at
  walls. The adapter's log says, every 2 s while the view leads, how far and how much room it
  found ("movement: the view ... m past the game's body").

Nothing of this was checked in the game: built, with the adapter's and the Oblivion adapter's unit
tests passing. It's in the framework's tracking mapping (`set_lead`) and Oblivion's adapter
(`movement.rs`, `collision.rs`), so it's everywhere in Oblivion, every save.

## Height, sitting or standing

You were floating. The view's height was your head's over where it was when the view was last
recentred, and that's the first moment of VR (the game loaded) or the Recenter button: with the
headset on the desk or in your hands then, or put on sitting after it started standing, the view
stayed too high for good. Your body is drawn from the same height, so its feet hung over the
floor.

Now your upright head height (standing, or sat up in your chair) is always the game character's
eye height, whenever the view was recentred:
- **Putting the headset on:** once it's been still on your head a second and a half, your height
  is taken from there. Before that the view stays at the character's eye height.
- **Standing up, sitting down:** held higher for a second, your upright height rises there over a
  moment (the view comes back down to the character's height); a moment higher (a stretch, on
  tiptoe) doesn't count. Lower, it settles slowly (a slouch in a few seconds, a crouch hardly at
  all, so ducking still crouches when you're not in seated mode).
- **Recenter** (the runtime's menu) takes your height as it is right then.
- **Seated mode** (Body tab) works: sat up in your chair, you see from the character's standing
  height, and leaning or ducking doesn't crouch the game (the stick's click does). Without it,
  sitting also puts you at the character's height, but ducking 35 cm crouches.

**Your height setting:** the Body tab says 1.37 m, probably measured sitting. It sets your hands'
size (they're drawn 8.1 cm wrist to knuckle) and, with "Play at my real height", the view's
height. Set it with the slider, or measure it standing up straight ("Measure my height" now refuses
eyes under 1.35 m: a grown-up sitting).

- **Try:** start the .bat with the headset on the desk, then put it on sitting; play a while,
  stand up, sit down; turn seated mode on and off; press Recenter.
- **Look out for:** floating or sinking that lasts, the view sliding up or down by itself, your
  body's feet off the floor, ducking not crouching (seated mode off) or crouching when you lean
  (seated mode on). The Body tab shows "eyes at" your upright height as the runtime has it.

Nothing of this was checked in the game: built, with the runtime's unit tests passing (physical
crouch's among them). It's in the framework's runtime (`mechanics.rs`), so it's every game's.

## What was checked of the row of tables

Nothing in the game, emulated or real: it was set out after the second playtest and only compiled,
with the adapter's unit tests passing. Every place in `tools/alchemy-row.txt` comes from the game's
own data:
- the farmhouse's model and the cell's references in `Oblivion.esm`, sliced at knee, waist and
  table height for a plan of its walls and furniture;
- the floor's height along the wall (its model's floor there, 7092.8);
- each table's, apparatus's and ingredient's model, for how far below its middle it reaches.

Look out for:

- **The tables** standing level on the floor, square to the wall, none in a wall, the wall's post or
  a bed. Table 2 passes the post where the entrance's wall meets the room's with 1 cm to spare;
  table 4's end is a few centimetres from the foot of the bed beside it (the beds' size isn't
  known: their models aren't read).
- **What's on them** landing on the tables, a few centimetres over each top: nothing should fall
  through or roll off. If something does, say what and at which table.
- **The set-out's moment.** Each lot of placing is a frame's work: about 60 lines of the game's
  script at two seconds and 270 at four (ten for each thing placed). A short hitch then is
  expected; tell me if it's long.
- **The old table in the entrance** gone with everything on it, and nothing left floating.

## What was checked in the game (the emulated headset)

Each of these was checked with scripted hands, in captures and logs:

- The mortar and pestle split into two pieces, drawn by the runtime, with the game's model hidden.
- Boar meat, venison and an apple taken from the table and dropped into the bowl.
- The card showing the game's own result: "Restore Health" and "Restore Health 3 pts for 9 secs on
  Self", with its icons, the ingredient icons and the four apparatus.
- Taking the boar meat back out by reaching in, the card updating, and putting it back.
- Grinding with the pestle, after which the game's Create made "Restore Health" (value 7, weight 2).
  The bottle rose out of the mortar and was taken in the hand, with the game's card on it.
- Apple and boar meat (no shared effect) ground: the failure path ran and the ingredients were used
  up. A crash in this path (the game's card of a thing reading an ingredient as it was picked up)
  was found and fixed.
- The mortar lifted in the left hand and tipped to the eyes: the shrunk ingredients and the pestle
  were inside it. It was set back down, and turned over it spilled.
- After the bench's own uses of the menu, the game's alchemy menu still opened from the inventory
  (equipping the Apprentice Mortar & Pestle).
- The Novice Mortar & Pestle dropped from the pack (the game's own drop) became a bench, with its
  own pestle, where it fell.
- No stall on the first ingredient. The game's menu takes about 2 ms to make and work out, because
  it's warmed when a bench appears; its first reading of the menu file was a 37 ms hitch before that.
- The unit tests of every crate touched pass.

**Not checked:**

- **Anything heard or felt.** The emulated headset can't hear: the ingredient-down sound, the grinding
  crunch (`PlaySound3D` lines compile), the potion's sound and the failure sound. It counts haptic
  pulses (224 in a grinding run) but doesn't feel them.
- **On-screen messages.** The game's messages ("You have created a new potion", the failure words)
  aren't in the captures.
- **Real hands:** how grinding feels, whether 0.45 m of grinding per ingredient (about three turns)
  is right, and how easy the pestle's handle is to take.
- **Untested rules and states:**
  - four ingredients;
  - a duplicate refused;
  - a master's single-ingredient potion;
  - combat refusing a potion;
  - grinding in the novice and apprentice mortars (measured as the journeyman's is);
  - saving and loading with things in the bowl.

## Decisions

- **The recipe note** is the bench's own, put down beside the mortar when you come to the bench, so
  there's nothing to bring up. It used to be put down as soon as the bench appeared, facing where
  you were then. With a row of tables, that turned the far tables' notes sideways to anyone standing
  at them, so a note now waits until you're within 2 m and facing its mortar. It's a page of the runtime's (a thin box the hands take up and put
  down, the hands going through it while it lies loose, so a fingertip can press it on the table),
  with the game's art as its face: the book menu's scroll (`menus\book\scroll_background.dds`, all
  of it that's paper), the game's handwriting (font 5, `Handwritten.fnt`) for what's written on it,
  and the scroll's own print (font 1) for the game's words for each effect, in the book menu's ink
  (117, 59, 33). Its back is the scroll's paper alone. It's lit as the game lights the things you
  hold.
  - **What it lists:** every mix of two of the ingredients you have (in your pack, and lying within
    reach of the bench) that share an effect you know (the game's own count of known effects for
    your skill), and each one alone for a master; each goes through the game's alchemy menu, with
    the bench's apparatus, as the card does. Of mixes making the same effects, the most valuable;
    the most valuable first. Two ingredients a line: three or four only add to a potion.
  - **Choosing** is a touch of the index fingertip of an empty hand (a hand holding anything has its
    fingers on that). Turning the page is the game's `ITMBookPageTurn`; taking it up and putting it
    down, `ITMScrollUp` and `ITMScrollDown`.
  - **The words** "Potions you can make", "Put in the ...", "Grind them with the pestle", "Take your
    potion" and the rest are ours: the game has no recipes. Every name on it is the game's.
  - It goes when its bench does (out of reach, the mortar taken), even out of a hand.

- **The game's own alchemy menu, made for a moment.** Whenever what's in the bowl changes, the
  game's `AlchemyMenu` is made (with the bench's apparatus), its ingredient slots filled, its effects
  worked out by its own code, read, and destroyed, all in one frame. Making it would put the game in
  menu mode and stop the world, so its id comes off the interface's menu stack in that frame and the
  mode is put back. Making the potion presses its Create the same way. Nothing about the potion is
  re-implemented. Engine facts: `docs/research/alchemy.md`.
- **The card** is the menu's own effects box (a piece of `menus\alchemy\alchemy.dds`) with the
  menu's fonts (font 1 for the name, font 3 for the rows, from `Oblivion.ini`'s `SFontFile_*`), its
  brown ink, the effect icons and the item icons. The runtime paints it from the game's files.
  - It isn't the game drawing its menu: the HUD layer has no alpha, so brown text on parchment
    would have been washed out when added to the view.
  - It shows over the mortar while anything's in the bowl, 20 cm wide.
- **Four ingredients at any skill, each kind once:** the game's own menu rules. Your skill limits
  which effects you know (and so make), not how many ingredients go in.
- **Apparatus at the bench:** the best of each kind within 75 cm of the mortar counts. Apparatus in
  your pack doesn't count at a bench; that's the menu's way, when you use a mortar from the pack.
- **Grinding work** is 0.45 m of pestle travel per ingredient, shared out over what's in the bowl.
  A ground ingredient is drawn at 45% of its size.
  - What counts is how far your hand takes the pestle's tip while the tip is low in the bowl.
    Pressed into the floor, the tip itself shivers there, and that would grind on its own.
  - The pestle meets the bowl's floor and walls: anything a hand holds now meets a game bowl,
    though it still passes through the game's other props as before. Your empty hand still passes
    through the mortar, as it does through any of the game's props.
- **No shared effect.** The game has no failure of its own: its Create button is just off. So
  grinding anyway gets the game's own unused `sPotionFailed` words and its `ITMIngredientNothing`
  sound, and the ingredients are lost, as those words say.
- **The ingredients are picked up** (the game's own pick-up, with stealing) just before Create,
  which uses them up from your pack. A stack in the world (several of one in a single reference) is
  picked up whole and one is used.
- **The bottle** is the game's own drop of the potion from your pack. It's held still rising out of
  the mortar, and waits there for up to 45 s; then it's Havok's and settles.
- **Grinding sound:** Oblivion has no grinding sound. Each stroke plays the game's organic collision
  sound (`COrganicSmall`, a crunch) from the mortar; an empty mortar is silent.
- **The save** was meant to be outdoors, but `rustvr-weye` and `rustvr-city` both load indoors and
  `rustvr-chorrol` loaded dark. It's where `rustvr-city` loads: Odiil Farm's farmhouse (this file
  called it a city house before; the save says Odiil Farm). Everything set out is made the player's
  own, so nothing there is stealing.
- **The row of tables** replaced the save's one table after the second playtest. That table stood
  in the farmhouse's narrow entrance, a metre from the front door, between a cupboard and the stone
  wall, sunk 5 cm into the floor.
  - The row is four of the game's `LowerClasstable03` (the farmhouse's own kind, 1.21 by 0.85 m,
    0.83 m high), standing on the floor along the east wall from the entrance into the main room.
    It's the longest clear stretch in the house: the rest of the main room has the dining table, the
    reading table, crates, the beds' nook and the fireplace.
  - They're 1.38 m apart, middle to middle. A bench counts the ingredients within 1.5 m of its
    mortar, so each table's stay its own.
  - One mortar of each kind from novice to master (no expert: four fit), with apparatus to match.
    Each table has two potions' ingredients that share their first effect, the only one this
    character's skill knows. Every form and potion is the game's.
  - The pack's food was taken out so each note lists its own table's potions. The character's own
    ingredients stay, so "from your pack" can still be seen (Damage Health).
- **Up against things, without changing the game's body.** The game's body is a round shape
  (made from the skeleton's `BBX` box, half 23.1 by 17.6 units across): 20.4 units
  times the actor's scale, plus Havok's 0.7 units round that, so at least 0.30 m. Making the
  player's shape thinner would let the game's body itself get close, but its bottom is a cone sized
  to that radius that lets it climb steps, so thinning it would change how stairs are climbed, which
  I couldn't check without the game. So the game's body stays as it is, and the view leads it:
  - **How far it goes:** the step the stick asked of the body (`Actor::Move`, turned to the world)
    less the step the body took, every frame; walking on, it comes back onto the body by a fifth
    of the way walked. Only steps the player asked count: falling, swimming or being pushed, the
    game moves the body and the view stays on it. A jump of the body (more than 25 cm in a frame:
    a load, a door) puts it back on.
  - **How far it may go:** the world's fixed things (the collision the hands already meet) that
    way from the eyes' line, looked for every 2.5 cm of height (a table's top is a few centimetres
    thick) and 12 cm either side. Something below the head nearer than anything at it is low: the
    eyes may go to 5 cm from it (your front at it), and never more than the game's body reaches.
    Anything at the head's height: the eyes stay 12 cm from it (the runtime fades the view as a head
    that round goes into a wall). Nothing found, so the body met something that isn't the world's
    (a door, someone): the head stays 12 cm inside the game's body.
  - **With your feet:** the runtime walks the game's body after your head; what it doesn't go
    comes off how far your head is from it, so the view stays where your head is.
- **Set out at every boot, not saved.** I can't run the game to make a new save, so the set-out is
  console lines run as the save loads (`tools/alchemy-row.txt`). Two console test commands are new
  for it:
  - `Place <base> <x> <y> <z> [<angle>]` puts a new thing at a place in the cell, in game units,
    whichever way the player stands or looks meanwhile. Things placed with it, or with `PlaceAt`,
    are set level whatever angles the game's `placeatme` gave them.
  - `RunFile <path>` queues a file of timed console lines in one go: `play.ps1 -Commands` gives each
    line its own half second.

## What's not done

- **Merging with other branches.** The protocol changes will collide with other branches and need
  renumbering at the merge:
  - VERSION 12, mapping `Local\RustVR_v12`;
  - new regions `OFF_STATIONS` and `OFF_CARDS` after the item infos;
  - `OS_IN_RECEPTACLE` is bit 18 and `OBJF_DRAWN_ALWAYS` bit 12.
- **holsters:** drinking the bottle at your mouth will just work once that branch is merged, since
  the bottle is an ordinary item.
- **menus:** the bench's alchemy menu exists only inside one frame and is never drawn. A floating
  version of the alchemy menu would show the game's own menu as usual when you use a mortar from your
  pack.
- **The bowl's contents after a save.** Ingredients left in a bowl when you save come back as loose
  things where they lay: Havok pushes them out of the mortar.
- **Variety:** the potion is always the game's `Potion01` bottle, or `PotionPoison` for a poison (the
  game's own choice). There's no visual paste in the bowl, only the ingredients shrinking.
- **The Mixing Bowl** (a type-0 apparatus with no pestle) isn't a bench.
- **The note's protocol** also needs renumbering at the merge: `OBJF_SHEET` is bit 13, and
  `MAX_CARDS` went from 4 to 8 (a bench's card and its note).
- **More than four benches at once.** The runtime has room for eight cards, a bench's card and its
  note each. Drop a fifth mortar near the row and fill bowls, and some card or note may not show.
- **A save of the row.** It's set out at each boot; a `rustvr-alchemy` with the row in it would
  boot straight into it.
- **The body walked after your head while it's held up.** Standing at a table with your head more
  than 5 cm past where the view leads the body, the game still walks its body into the table, as
  before (its walk, and its footsteps if it makes any).
- **Merging:** the view's lead is in the framework's `tracking.rs` (`set_lead`, and where `update`
  applies it), which other branches may also change.
