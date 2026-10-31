# Playtest: holsters and inventory

Save: rustvr-holsters

## Playing it

From the repository root:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-holsters
```

It builds main, installs it beside the game and boots straight into the save for the headset.
`powershell -File tools\oblivion.ps1 -Close` puts the game away and the player's settings back.
The fast boot, the test saves, worktrees and the game lock are in [DEVELOPING.md](../DEVELOPING.md).

`rustvr-holsters` is the developer's own test save, not in the repository: a house, with things to
try added to the pack, the character's weapon (a Fine Iron War Axe) at the left hip and the shield
on the back. It was made from `rustvr-weye`; any save does, somewhere with things lying about for
the sounds:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-weye -Give "00009311 2,00098496 2,00009310 2,00098499 1,00033669 1,00033668 2,0003365D 3,00023D89 1,00017829 20,000229C1 20,0008DC78 2,0008992C 3,00019171 1,000229BC 1,0002CF9F 2" -Commands "SaveGame rustvr-holsters"
```

That gives Strong, ordinary and Weak Potions of Healing, a Potion of Sorcery, a cheese wheel and
wedges, apples, a bread loaf, iron and steel arrows, two Strong Poisons of Severing, three Electric
Shell scrolls, an Iron Dagger, a Steel Shortsword and two torches.

## What it does

Your things live in slots on your body. Reach into one and squeeze the grip, and its first thing
comes out in your fist as the game's own model. Bring food or a drink to your mouth to eat or drink
it, hold a scroll up to read it, touch a poison to your weapon. Let go of something in any slot to
put it back. Let go anywhere else and it drops into the world.

- **Where the slots hang.** From where your eyes would be looking level, over the neck your head
  turns on (`rig::zones::BodyAnchor`), so looking down or tilting your head doesn't move them. The
  anchor's yaw follows your head with a lag and freezes while a hand reaches into a slot.
- **Taking.** A hand counts in a slot at its middle, or anywhere deeper, up to 12 cm in toward the
  middle of your body: a hand pressed against your hip or chest at a slot is in it. At a slot's
  middle it takes from that slot, the weapon holsters first (a hip with a weapon in it gives the
  weapon). Only deeper, it takes from the slot it's furthest into: pressed against your belly
  beside the potions, you get the potions, not the axe at your hip. The same goes for putting
  things back.
- **Feel.** Entering a slot gives a tick in that hand, an empty one a fainter tick. Squeezing in an
  empty slot gives the game's "nothing happens" sound, unless the slot is set to give nothing.
- **The holsters.** The equipped weapon hangs at the left hip (a two-handed one over the right
  shoulder), the shield over the left shoulder. The hips are 13 cm round; the two over the
  shoulders are 28 cm round, reaching from beside the head down behind the shoulder, and count only
  with the hand beside or behind the head, never in front of the face (food at your mouth).
- **Held as it's made to be held.** Each thing drawn has a hold of its own, worked out from its
  model when the slots' first things load (`rig::pockets::hold`):

  | Kind | How it's held |
  |---|---|
  | A bottle (a potion, a drink, a poison) | upright round its body, a third of the way up, its mouth past your thumb |
  | A long thing (a leek, an arrow, a rolled scroll, a loaf) | along the fist, at its narrowest place near its middle, the palm on its flat side |
  | A flat thing (a book, a cheese wheel, a plate) | by an edge along its long side, the rest out past your fingers |
  | Anything else (an apple, a cheese wedge, a soul gem) | upright in your fist |

  A thing thicker than your fist closes round (Oblivion's food is big) lies against your palm, the
  fingers round it as far as they go. The left hand holds things the same way round as the right.
  A thing let go and taken again near its handle goes back into that hold.
- **Eating and drinking.** Held at the mouth for half a second (a point 7.5 cm under and 5 cm in
  front of your eyes, within 9 cm of it), with a bite or a gulp felt every 0.16 s and heard as the
  game's own chewing or swallowing. Never while the hand moves faster than 1.2 m/s. Then the game
  eats or drinks it as its inventory does: its effects, its sound, the potion limit and the quest
  items it refuses. A use the game hasn't taken in 1.5 s was refused, and the thing stays in your
  hand.
- **Reading a scroll.** Held up in front of your eyes, 12 to 55 cm away and within 23 degrees of
  where you look, for 0.8 s. The game casts it as the player's own scroll use, where you look.
- **Poisons.** Touched to the weapon in your other hand for 0.3 s: the game's own steps without its
  yes/no box (the touch is the yes), its message ("Strong Poison of Severing used.") and its own
  refusals (no weapon, one already poisoned, a staff).
- **Weapons and torches from a slot.** The game equips the thing drawn (its own equip and sound),
  and its own object takes the copy's place in the hand, by its rigged grip as every weapon is. The
  weapon or torch you had on before is a thing in the slots again. A torch burns in the hand and in
  any holster it hangs in (`crate::torch`).
- **Arrows.** Arrows of the kind on your bow come out of whichever slot keeps them as the bow's
  own, to nock (`INV_ARROW`); another kind comes out as itself until the game puts it on the bow.
  There's no quiver of its own in Oblivion: the Hands tab's Quiver setting is for games without
  body slots.
- **Dropping.** A thing let go anywhere but a slot falls, and where it comes to rest the game drops
  it into the world (its own drop): it leaves your inventory and lies there to pick up again.
- **What's in a slot isn't shown during play.** You find a slot by its tick and learn what it holds
  by drawing, or in the editor.

### The slots you start with

Right-handed; mirrored for a left hand (`crate::inventory::defaults`).

| Slot | Where | Gives out |
|---|---|---|
| Healing | the belt, in front of your left hip (your off hand reaches it while the sword hand fights) | potions and food, what heals most first; then what restores magicka or fatigue; never a potion that restores none of them |
| Arrows | over your right shoulder, from above it down behind it (where Oblivion wears its quiver) | arrows, the equipped ones first, then the strongest |
| Scrolls | high on your chest, on the right | scrolls, then poisons, the most valuable first |
| Food | at your right hip | food, what restores the most fatigue first |

The categories a slot can draw from are the game's kinds: Food, Potions, Poisons, Ingredients,
Scrolls, Arrows, Weapons, Soul Gems, Keys, Books, Sigil Stones, Apparatus, Misc and Torches.
Apparel isn't one.

### The holsters' editor

A menu of our own made of Oblivion's UI: its parchment, rows, icons and buttons, in three strips
(the slots, the categories, the buttons), floating ahead of you as your own menus do. It opens on a
grip held still in a slot for 1.5 s (a firm pulse, and the game's inventory opening sound;
whatever came out goes back), or from **Holsters** at the bottom of RustVR's own settings, beside
the pause menu. Point with the laser and pull the trigger:

- **A slot** on the left picks it. Under its name is what it gives out now.
- **A category** in the middle puts it in the slot, last. Its place in the order shows on the right
  of its row, the picked one as `< 2 >`. Pick it again to take it out.
- **Earlier** and **Later** move the picked category up or down the slot's order.
- **The order** cycles: Healing first, Magicka first, Fatigue first, Most valuable, Heaviest first,
  Lightest first, Newest first, By name, Most of first, Strongest first, My own order (what you last
  put back comes out first).
- **Empty** sets what an empty slot does: a tick, nothing, or anything (the order's first of
  everything you have).
- **Size** (small, medium, large: 6, 9 or 13 cm) is how big the slot is, where a thing let go goes
  back in. **Reach** (close, normal, far: 2, 4 or 7 cm more) is how far out you can still take from
  it.
- **Worn first**: what you have on (the equipped arrows) comes first.
- **Move**: reach where you want the slot on your body and squeeze either grip. Placed behind your
  head, it counts only there (over a shoulder).
- **New slot**: a new, empty slot ("Slot 5", "Slot 6"...), placed by your next squeeze. Then give it
  categories.
- **Remove** takes the picked slot away.
- **Ok**, or B, closes it.

While it's open every slot shows as a sphere of blue light the size of its reach, drawn over your
body: the picked one bright, the others faint, and the picked slot's first thing shows, still, at
its place. Size, Reach and Move change the sphere as you click them. Every change is kept at once.

### Sounds

Every sound the runtime makes here is one of Oblivion's own records, played from the sound pack
`oblivion-extract sounds` writes into `local_assets/oblivion/sounds` (from `Oblivion.esm` and
`Oblivion - Sounds.bsa`), at the records' own levels:

| When | You hear |
|---|---|
| You take a thing, put it away on the body, or let go of it | the inventory's sound for a thing of its kind (the game's own choice); letting go is the put-away sound at 60% |
| A thing you hold or throw knocks into the world | the game's collision sound for its Havok material and mass |
| You draw or sheathe the weapon at your hip or back | the game's draw and sheathe for its type, with the enchanted ring over an enchanted one's draw |
| The gravity gloves | the flick: a big weapon's swish, quiet; while it flies, Telekinesis's travel sound; landing in your hand, its pick-up sound |
| The gloves lock onto a thing | nothing: only the pulse in your hand |
| A hand finds a slot, the gloves point at a thing | the menus' focus tick, as quiet as the game plays it |
| A bite or a gulp at your mouth | the game's chewing (`NPCHumanChew`) or swallowing (`NPCHumanSwallow`) |
| A slot is empty, or the game refuses a use | the game's "nothing happens" sound |
| RustVR's own settings menu | the menus' OK click |
| Drawing from a slot, putting back, stowing a thing lying about | the game's own inventory sounds (RustVR adds nothing) |

## What to try

1. **Find the slots.** Move a hand slowly round your belt, chest and right shoulder: a tick in that
   hand on entering one, a fainter one if it's empty.
2. **Drink a healing potion.** Left hand in front of your left hip, at the belt, and squeeze: a
   Strong Potion of Healing comes out, upright in your fist. Hold it just under your nose: gulps in
   your hand, then the game drinks it. Wave it past your face and nothing happens. Draw again: the
   next strong one, then the ordinary ones, then the weak. Drink too many in a row and the game says
   you can't; the potion stays in your hand.
3. **Eat.** Squeeze at your right hip: food comes out. At your mouth you feel and hear bites, then
   it's eaten.
4. **Put something back.** Let go of a drawn thing in any slot, its own or another, or a holster.
5. **Drop something.** Let go of a potion in front of you, or throw it. It falls and the game drops
   it where it comes to rest. Pick it up again like anything lying about, and put it in a slot.
6. **Read a scroll.** Squeeze high on the right of your chest (the most valuable first). Hold it up
   about a forearm in front of your eyes: Flash Bolt casts where you look, Electric Shell as a
   shield on you.
7. **Poison your axe.** Draw the axe from your left hip with your right hand and swap it into your
   left. Draw a poison from the chest pouch (the scrolls come first: read them, or reorder the
   slot) and touch the bottle to the axe.
8. **Draw an arrow.** Reach over your right shoulder and squeeze; let go of it there to put it back.
   With a bow in the other hand, it comes out ready to nock.
9. **The editor.** Hold a grip still in a slot for 1.5 s. Pick each slot, look down at the spheres,
   add and reorder categories, change the order, Move a slot, make a New one.
10. **Weapons from a slot.** Make a slot with Weapons in it (New slot, squeeze at your right thigh,
    pick Weapons). Reach there: the dagger or the shortsword comes out at its grip, and the game
    equips it. Let go at your left hip and it hangs there; the axe is in the weapons slot now.
11. **A torch.** Put Torches in a slot and draw one: it burns in your hand. Hang it at a hip.
12. **Sounds.** Knock things lying about on the table and the floor; draw and sheathe the axe and
    the shield; pull a thing with the gravity gloves; reach into empty slots. Listen for anything
    synthetic, too loud or quiet next to the game's own, or heard twice.
13. **Over the shoulder.** Reach for the arrows and the shield quickly, without looking, with the
    head straight and turned. Rest your hands at your sides and on your belly: nothing should come
    out by accident.

## Settings

| Setting | Where | Default (range) |
|---|---|---|
| Eating and drinking: hold at the mouth (`holsters.mouth_time`) | Hands tab | 0.5 s (0.15 to 1.5) |
| Reading a scroll: hold before the eyes (`holsters.read_time`) | Hands tab | 0.8 s (0.2 to 2) |
| Holsters' editor: grip held in a slot (`holsters.edit_hold`) | Hands tab | 1.5 s (0 to 3; 0 turns it off) |
| A poison touched to the weapon (`holsters.coat_time`) | `settings.toml` only | 0.3 s |
| Gravity gloves, Glove range, Flick speed | Hands tab | on, 10 m (1 to 10), 1.2 m/s (0.6 to 3) |
| Interaction sounds, Volume, The game's own sounds | Hands tab | on, 0.8, on |

The slots are kept per game in `%LOCALAPPDATA%\RustVR\settings.toml` under
`[[holsters.games.oblivion]]` (each with `name`, `at`, `radius`, `reach`, `behind`, `categories`,
`order`, `equipped_first`, `empty`, `own`). The game's own four are used until you change anything.

## Decisions

- **Food restores fatigue in Oblivion.** The game is the source of truth: 33 of its 36 food
  ingredients restore fatigue first. So "what heals" is what restores health first, then what
  restores magicka or fatigue: cheese comes out of the healing slot only once the healing potions
  are gone, and a Potion of Feather never does. The food slot orders by fatigue.
- **The order is the categories' first, then the rule.** A restoring rule puts everything that
  restores its own stat ahead of the rest, whatever the category (healing food before a magicka
  potion).
- **A generous mouth, a moment's hold.** A mouth too small a target is the most complained-of thing
  about eating and drinking in VR. The bites on the way mean nothing is eaten by accident. The game
  does the rest.
- **Let go anywhere but a slot and the game drops it.** The alternative, springing back to its
  slot, was rejected: players hate auto-returns they didn't ask for, and a dropped potion should be
  a potion on the floor.
- **Weapons and torches drawn from a slot are equipped.** The game's own object takes the copy's
  place in the hand (`ObjectDesc::replaces`). What you have on isn't one of the slots' things: it's
  in its holster already.
- **The editor is a menu of our own made of the game's UI:** its own menu type, 1050, which the
  game's menu factory has no case for, so it's hooked; its XML made of Oblivion's prefabs and
  textures and the inventory's row look; the categories' icons the game's own (a thing you have in
  each, else the game's icon of its kind); the button labels ours, in the game's fonts ("Ok" is the
  game's own word, from its message box).
- **The editor opens on a grip held still in a slot** (VRIK's hold-to-move, which players found the
  easiest customisation), and from RustVR's menu. Move places a slot by your own hand on your body,
  not with sliders.
- **Slots are kept per player, per game**, in the runtime's settings: not per save.
- **Default places keep clear of the holsters.** The healing slot is in front of the left hip, not
  at it: the left hip is the sword's. If a two-handed weapon hangs over the right shoulder, the
  weapon comes before the arrows there.
- **The game's own sounds, chosen as the game chooses them.** Where the game makes a choice, the
  adapter asks the functions it uses itself. Oblivion has no sound for letting go of something
  held, so letting go is the put-away sound, softer; it knocks with its own collision sound as it
  lands. A thing drawn from a slot has no Havok body for the game to ask, so it knocks with what
  its kind is usually made of (potions and soul gems glass; weapons, keys and gold metal; scrolls,
  books and clothes cloth; food organic; bows and arrows wood). The game plays both bodies'
  collision sounds in a knock; RustVR plays the thing's. Oblivion's effects volume doesn't apply to
  the pack; RustVR's own volume does. The game's `WPNBowShoot` names a file the game hasn't got;
  its bow uses `bowShoot`, and so does the pack.
- **The index finger rests the same on and off the trigger** (`rustvr_ik::fingers`): as it does on
  the trigger, curling only as you pull. The trigger's touch sensor doesn't move it. Both the drawn
  hand and the physics fingers follow it.

## Not done / known issues

- **Opening the editor can close the game.** `slot_menu::ICONS` has 13 icons for
  `inventory::CATEGORIES`' 14 (Torches was added after it): with no torch in the inventory besides
  an equipped one, filling the Torches row reads past the icons, and with the release build's
  `panic = "abort"` the game goes. Needs a fix in the code.
- **No ghost of what's in a slot during play** (only in the editor).
- **Slots don't move with the body's crouch or lean:** they hang off the head-following anchor the
  holsters have, as the research recommends.
- **No renaming slots:** no keyboard reaches the game in VR.
- **No bite-by-bite eating:** Oblivion eats a thing whole, so one hold eats one.
- **Books** can be put in a slot and drawn; reading one is the books feature's
  ([books.md](books.md)).
- **A thing without a model the extractor can read** isn't handed out.
- **A dropped thing turns to face you** as the game takes over (the game's drop), not the way its
  copy lay.
- **Not seen in a playtest record:** the potion limit's refusal, eating a quest item, the poison's
  refusals, Earlier and Later, Empty, Size, Reach, Worn first, My own order, Remove, Ok, the
  Holsters button in RustVR's menu, a left-handed player, a slot placed behind the head.

## History

Newest first.

- **On main:** merged with archery (arrows from any slot nock on the bow) and torches (Torches a
  category, the torch a thing like a weapon).
- **Round 7.** The index finger lifted whenever it left the trigger and changed as it touched it.
  It now always rests as on the trigger and curls only as you pull; the pointing pose went.
- **Round 6.** The shoulders were still too hard to reach. The back holsters grew to 28 cm (were
  24), their middle 4 cm lower; the arrows slot to 26 cm (was 20), 6 cm lower; "behind" became
  anywhere beside or behind the head. Each draw logs where the hand was.
- **Round 5.** Drawn things sat in the hand any old way. Each now has a hold worked out from its
  model (bottle, long, flat, anything else), the fingers round its handle.
- **Round 4.** A leek stuck in the hand: the gloves' catch put a pulled thing into a hand that had
  drawn something on the way. A hand that took something else no longer catches, and a hand that
  takes anything lets go of what it held. Bites and gulps got the game's chewing and swallowing;
  the gloves' swish was halved.
- **Round 3.** Every RustVR sound was replaced by Oblivion's own records (the table above).
- **Round 2**, after the first playtest. Slots were awkward to grab: they hung off the eyes, so
  looking down swung them out in front. They now hang from the level eyes over the neck; a hand
  counts up to 12 cm deeper; the back ones grew; the editor shows every slot as a sphere.
- **The first build:** the slots, the mouth, reading, poisons, dropping and the editor, checked on
  the emulated headset (drinking, the healing order, eating, dropping, arrows, reading, poison, the
  editor's clicks, Move and New slot, weapons from a slot).
