# Alchemy by hand

Oblivion's own apparatus (mortar and pestle, alembic, calcinator, retort) set out on a table and
used by hand: ingredients dropped into the mortar and ground with the pestle, the potion made by
the game's own alchemy routine and handed over as its own bottle. This page is the research behind
it: what players say about making things by hand in VR, and the design built from it.

## What players love and hate

Inspiration only; nothing here is copied.

**Skyrim VR's alchemy is its flat menu**, and reviewers rank the menus among the port's worst parts:
taking something off a shelf just makes it vanish into a window
([Road to VR](https://roadtovr.com/skyrim-vr-review-pc-rift-vive-windows)). In r/skyrimvr's "How to
make alchemy fun" thread players ask for exactly this feature: grind in a mortar, pour into a bottle.
A Nexus user asks to hold the mortar in one hand, add ingredients with the other and smash them
with the pestle. No released mod makes Skyrim VR's alchemy physical. The HIGGS-era mods all share
one pattern: the game does its own action, and the result lands in your hand. Immersive Harvesting
VR does this for picking ingredients, Physical Checkout VR for the barter menu, and Interactive Herb
Smoking VR adds repeated knife strikes for crafting. There's no Oblivion VR alchemy mod at all.

**Waltz of the Wizard**: almost anything on the table goes in the pot and the result depends on the
mix. Players love that experimenting ([Mogura VR](https://www.moguravr.com/?p=28105), [Road to
VR](https://roadtovr.com/waltz-of-the-wizard-preview/): an hour just mixing potions). They also
complain it's pure trial and error with nothing telling you what anything does.

**Potion Craft** (flat, but the best-documented grinding): reviewers find bashing with the pestle
very satisfying and like the mortar staining with what's in it
([LadiesGamers](https://ladiesgamers.com/potion-craft-alchemist-simulator-early-access-review/)).
Repetition wears thin, though: stirring the same things again and again bores players
([review](https://www.goodreads.com/author_blog_posts/22271157-game-review-potion-craft)).

**Design talks.**
- Owlchemy (Job Simulator, Vacation Simulator): the worst thing in VR is to reach for something and
  have it do nothing. Haptics go on specific moments, not everywhere
  ([Game Developer](https://gamedeveloper.com/design/lessons-on-vr-motion-and-interactivity-from-leading-vr-devs)).
  State shows on the thing itself (steam, colour going from raw to cooked to burnt), and unclear
  feedback is inaccessible feedback
  ([Meta](https://developers.meta.com/vr/blog/owlchemy-labs-case-study-lessons-learned-from-job-to-vacation/)).
- Half-Life: Alyx: nearly every object has its own sound, item state shows on the item (a light per
  round on a magazine), and the most-repeated interactions get the most polish
  ([Road to VR](https://www.roadtovr.com/these-details-make-half-life-alyx-unlike-any-other-vr-game-inside-xr-design/)).
- I Expect You To Die: things knocked out of the hand by scenery frustrated playtesters
  ([Game Developer](https://www.gamedeveloper.com/programming/engineering-a-super-spy-in-i-expect-you-to-die-)).
  Their fix (held things pass through the world) breaks this project's rule that nothing passes
  through hands. Only the goal is taken: never knock a thing out of the player's grip.

**Repetitive motion.** A study of VR pot-stirring found vibration made people feel they did better
and raised their immersion
([Aalborg University](https://vbn.aau.dk/en/publications/effects-of-vibrotactile-feedback-in-commercial-virtual-reality-sy)).
Quick, decisive gestures register more reliably than slow ones, and long gesturing tires the arms
([The Wizards review](https://skarredghost.com/2017/08/03/wizards-review-entertaining-vr-spellcaster-game-early-access/)).
No source gives a stroke count at which grinding becomes a chore.

**Common complaints** about making things by hand in VR:
- fiddly small objects moved one at a time (Cooking Simulator VR);
- things passing through containers, and things rolling away when dropped (PotionBrew, a jam game);
- not knowing what's in the pot or whether stirring is working: players ask for a sound and a sign
  when the mix is done ([itch.io](https://itch.io/post/7215673));
- a failure that says nothing.

### What we take from it

- Every contact answers. An ingredient lands in the bowl with the game's own sound, the pestle
  grinds with the game's sounds and a buzz in the hand only while it's grinding, and the game's own
  sound and message play when the potion is made.
- State shows on the apparatus itself. The ingredients sit in the bowl as their own models and
  shrink as they're ground, and the effects show on a card beside the mortar, not in a menu.
- Grinding is short and forgiving: any rough circling or pressing in the bowl counts, about three
  or four turns per ingredient. The potion comes when all of them are ground.
- The bowl keeps what's dropped in it. Something let go over its mouth goes in and stays there,
  even while the mortar is carried. The bottle never rolls off: it rises out of the mortar and waits
  to be taken.
- What an ingredient does is the game's to say, by its own rule for how many effects the player
  knows.
- A mix with no shared effect fails visibly and audibly, with the game's own words and sound.
- It's a rest moment at a table, the hands low: nothing held up for long.

## The design

What was built, on main: `crates/oblivion/src/alchemy.rs` (the adapter), the runtime's
`rig/receptacle.rs` and the bowl handling in `rig/mod.rs`, and the protocol's `stations` and
`cards`.

### For any game: the framework

- **Receptacles** (`rustvr_protocol::stations`, at most 8 bowls): the adapter says which of its
  objects is a bowl: where its floor is, its mouth's radius, how deep it is and its wall's
  thickness (object space, up its +Y), how many things it holds, which objects may go in now (at
  most 40), and what grinds in it (`REC_GRIND`: a grinder object, its tip, how much grinding each
  thing takes, where the grinder rests).
  - Something let go over the mouth goes in (not when it's let go faster than 2.5 m/s, and not
    into a bowl tipped more than 60 degrees). The bowl keeps it wherever it's carried, laid on its
    floor (the first in the middle, the rest round it, each shrunk to fit if it's bigger than the
    bowl).
    Turned over, the bowl spills what's in it. A hand reaching into the mouth takes what's in it;
    above it, the hand takes the bowl or the grinder.
  - The bowl's own collision becomes a bowl (a floor and a ring of 12 wall pieces), so the pestle
    and the hand go into it. The grinder rests in it where the game's model has it, and goes back
    there when let go over the mouth.
  - Contents are reported `OS_IN_RECEPTACLE` with the bowl as `parent` and how ground each is in
    `fuse`. The bowl's state carries the share of everything ground (`slide`) and the grinding
    strokes so far (`shots`, each one a sound for the game to play).
- **Grinding** (`rig/receptacle.rs`): the grinder's tip pressed low in a bowl with something in
  it (within 2.5 cm of its floor) and worked about grinds it by the distance the hand takes the
  tip: round, back and forth, or pounding (a vertical stroke counts half), counted no faster
  than 0.8 m/s. Each 9 cm of grinding is a stroke. The hand feels it: a texture while it grinds,
  stronger the faster, and a pulse on each stroke. What's ground is drawn smaller.
- **Pieces the runtime draws always** (`OBJF_DRAWN_ALWAYS`): a game model the runtime splits in
  two (the mortar and its pestle) is drawn by the runtime, lit as the game lights it, while the game
  hides its own.
- **Cards of a game's own UI** (`rustvr_protocol::cards`, at most 8): the adapter describes a card
  as the game's own pieces, its menu art cut out (a texture and the part of it) and lines of text
  in its own bitmap font, colour and wrap. The runtime draws it beside an object, turned to the
  eyes, fading in and out. A card anchored on a sheet (`OBJF_SHEET`, a thin prop) is that sheet's
  face instead, and a fingertip's taps on the sheet come back as touches (`shots`, with where the
  last one landed in `slide` and `fuse`). Fonts and the art come from the asset cache
  (`oblivion-extract export-ui` writes the game's fonts and menu textures).

### For Oblivion: the adapter

- **The bench** (`alchemy.rs`). Every mortar and pestle among the things near the player is two
  pieces, split and measured by `oblivion-extract bench` (the pieces `#mortar` and `#pestle` in
  the cache, the measurements in `bench.toml`): the mortar (the game's object, its game model
  hidden, which the runtime draws as its `#mortar` piece) and the pestle (the runtime's own, a
  capsule resting where it lies in the model, given its share of the apparatus's weight by size).
  The alembic, calcinator and retort count when they're within 75 cm of the mortar. Of each kind
  the best is used, as the game takes the best the player carries. When a bench appears the game's
  alchemy menu is made and gone once, so its file is read before the first ingredient goes in.
- **Ingredients** within 1.5 m of the mortar may go into it while it has room, at most four as in
  the game's menu, and never the same kind twice, as the menu allows each once. Each one lies in
  the bowl as itself (the game's object, posed there) and shrinks as it's ground. One going in
  plays `ITMIngredientDown` from the mortar (`PlaySound3D` on it, as a script line).
- **The card**: whenever what's in the bowl changes, the game's own alchemy menu is made for a
  moment and gone again, without the menu mode it would bring. Its ingredient slots are filled
  with what's in the bowl and the menu's recompute works the potion out. Then the
  interface's mode, its stack of open menus, the tiles under the cursor and the menu background's
  flag are put back as they were, the menu's id is cleared, its fade taken off the list
  and the menu destroyed. Its effect list (what the player's skill shows) becomes the
  card beside the mortar, cut from the menu's effects box: the potion's name in font 1, each
  effect's icon and words in font 3, and along its foot the icons of the ingredients in the bowl
  and of the apparatus used.
- **Grinding** plays the game's organic crunch (`COrganicSmall`) from the mortar on the pestle's
  strokes.
- **The potion**: with everything ground, the ingredients in the bowl are taken (the game's own
  pick-up, stealing included) and the menu's Create is pressed in code. So the potion is the game's
  own, with its effects, name, weight, value, skill use, message and sound. Its bottle leaves the
  inventory into the world (`player.drop`, the game's own drop) and rises out of the mortar over
  0.9 s to 12 cm above the rim, where it waits up to 45 s to be taken before it's the game's
  again. With no shared effect there's nothing to make: the game's own failure message
  (`sPotionFailed`) and sound (`ITMIngredientNothing`), and one of each ingredient is removed, as
  that message says.
- **Not in combat**: as the game's own use of an apparatus, unless `iAllowAlchemyDuringCombat`
  says so: its words ("You cannot create potions during combat!"), and the ground bowl waits,
  trying again every 3 s.
- **The recipe note** (built after this research). Each bench has a note: the book menu's scroll
  (`menus\book\scroll_background.dds`) as a sheet 26 cm wide, written in the game's handwriting
  (font 5) with the effects in its print (font 1). It's put on the table 20 cm from the mortar
  towards the player once they're within 2 m of the bench and facing it. Its list page, "Potions
  you can make", has every potion the player can make from what they carry and what lies at the
  bench, worked out through the game's own menu (each pair of ingredients sharing an effect the
  player knows, and a master's single ingredients; a few mixes a frame, looked at again once a
  second): of each set of effects the most valuable, the most valuable first, with "Turn over"
  when it runs to more pages. Touching a row opens its page: the potion, its effects in the
  game's words, and the steps ("Put in the ...", "from your pack" when none lies at the bench,
  "Grind them with the pestle", "Take your potion"), each faded once it's done, and "Take the ...
  out of the mortar" for anything in the bowl that isn't in the recipe. Touching the title or
  "Other potions" goes back to the list. Taking it up and putting it down play the game's scroll
  sounds, a touch its page turn.
- **The save**: `rustvr-alchemy`, the developer's own test save (not in the repository), made from
  `rustvr-city` (which loads in a house in the Market District): a table set out (`PlaceAt`, the
  console's test command) with the journeyman mortar and pestle and alembic, the apprentice
  calcinator, the novice retort and nine ingredients, all the player's own; the novice and
  apprentice mortars, a novice alembic and calcinator, an apprentice retort and more ingredients
  in the inventory.
