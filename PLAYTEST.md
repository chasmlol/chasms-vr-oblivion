# Playtest: alchemy by hand

Save: rustvr-alchemy

`powershell -File tools\play.ps1 alchemy` boots straight into it: a house in the Imperial City's
Market District, standing at a table set out as an alchemy bench. On the table: a Journeyman Mortar
& Pestle in the middle, a Journeyman Alembic (back left), an Apprentice Calcinator (back middle), a
Novice Retort (back right), and ingredients (Apple, Carrot, Wheat Grain at the left; Boar Meat,
Venison at the front right; Ginseng and Garlic behind the mortar). Everything there is yours. In
your pack: a Novice and an Apprentice Mortar & Pestle, a Novice Alembic and Calcinator, an
Apprentice Retort, and three each of Apple, Carrot, Boar Meat, Venison and Aloe Vera Leaves.

## What to try, in order

0. **The recipe note.** On the table, about 20 cm in front of the mortar, lies a note: the game's
   own scroll paper (the one its book menu opens a note on, ribbon and all), written in the game's
   handwriting.
   - It reads "Potions you can make", and under it a line for each potion: its effect's icon, the
     game's name for it, and what it's made of ("Restore Health: Boar Meat, Venison"). The most
     valuable first. They're worked out by the game's own alchemy, out of what's on the table and
     in your pack, so only effects your skill knows are there. Expect two lines: Restore Health
     (Boar Meat, Venison) and Restore Fatigue (the best two of Apple, Carrot, Wheat Grain and Aloe
     Vera Leaves: of potions with the same effects, the list keeps the most valuable).
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
2. **Boar Meat into the mortar.** Take it from the table and let it go anywhere over the bowl's
   mouth, up to about 30 cm over the rim, or just past the rim. It drops into the bowl and lies on
   its floor, shrunk to fit. You hear the game's ingredient-down sound from the mortar. A card comes
   up over the mortar, cut from the game's own alchemy menu: its parchment, with the boar meat's icon
   along its foot and, at the right, the icons of the apparatus that count. One ingredient makes
   nothing at a novice's skill, so there's no name or effect yet.
3. **Venison in as well.** The card now shows what the game would make: "Restore Health" in the
   game's title font, the restoration icon and "Restore Health 3 pts for 9 secs on Self" in the
   menu's own words. That's the game's alchemy working it out: its skill rule for which effects you
   know, your mortar's, alembic's, calcinator's and retort's qualities, and its naming.
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
6. **A mix that doesn't work.** Apple and Garlic, say: they share no effect you know, so the card
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
     kind. Move the Novice Alembic from your pack onto the table beside the Journeyman one: the
     Journeyman still counts. Take the alembic away and the effects change.
10. **Other mortars.** Drop the Novice or Apprentice Mortar & Pestle from your pack (the game's own
   drop) and pick it up off the floor onto the table. Every mortar and pestle near you is a bench,
   and each has its own card.
11. **The game's own menu still works.** Use a mortar from your pack in the inventory and you get the
   game's alchemy menu as before.

## What was checked of the recipe note

Nothing in the game, emulated or real: it was built after the first playtest and only compiled,
with the unit tests of every crate touched passing. Look out for:

- **Where it lies.** It's put 20 cm in front of the mortar, towards where you stood when the bench
  appeared, and falls a centimetre onto the table. If the table's edge is nearer than that, it may
  fall to the floor (pick it up from there).
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

- **The recipe note** is the bench's own, put down beside the mortar whenever a bench appears, so
  there's nothing to bring up. It's a page of the runtime's (a thin box the hands take up and put
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
  `rustvr-chorrol` loaded dark. It's the city house, with the bench made the player's own so nothing
  there is stealing.

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
