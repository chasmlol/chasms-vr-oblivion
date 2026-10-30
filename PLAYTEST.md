# Playtest: holsters and inventory

Save: rustvr-holsters

`tools\play.ps1 holsters` boots this build into **rustvr-holsters**: where `rustvr-weye` starts
(a house), with things to try added to your pack:

- healing potions (strong, ordinary and weak) and a Potion of Sorcery;
- food (cheese wheels and wedges, apples, bread, the house's leek and ale);
- arrows (iron and steel);
- two Strong Poisons of Severing and three Electric Shell scrolls;
- an Iron Dagger and a Steel Shortsword.

You start with the Fine Iron War Axe at your left hip and the shield on your back.
`tools\play.ps1 -Close` puts it all away.

The flat game's crosshair is hidden in VR (from the riding branch, by way of the menus branch): only
the icon of what you can activate shows where you look.

Your things now live in slots on your body. Reach into one and squeeze the grip, and its first thing
comes out in your fist as the game's own model. Bring food or a drink to your mouth to eat or drink
it. Let go of something in any slot to put it back. Let go anywhere else and it drops into the
world.

## Round 2: after your first playtest

You said the slots and holsters were awkward to grab: a hand pressed against the hip or chest
didn't take, only one halfway out did, and the back ones were hard. And the editor didn't really
show where a slot is.

- **Why it was awkward:** every slot and holster hung off your eyes. Looking down at a hip swings
  the eyes some 10 cm forward and down, and the slots went with them, out in front of your body.
  - Now they hang from where your eyes would be looking level, over the neck your head turns on (the
    same neck the drawn body has). Looking down or tilting your head no longer moves them.
- **Deeper counts:** a hand counts in a slot at its middle, as before, or anywhere deeper, up to
  12 cm in toward the middle of your body. So a hand pressed against your hip or chest at a slot is
  in it.
  - A hand at a slot's middle takes from that slot, holsters first (a hip with a weapon in it still
    gives the weapon).
  - A hand that's only deeper takes from the slot it's furthest into. Pressed against your belly
    beside the potions, you get the potions, not the axe at your hip.
  - This is the same for the weapon holsters (hips, back) and for putting things back.
- **The back is easier:**
  - the shoulders' weapon holsters are 24 cm round (were 20);
  - the arrows slot reaches 20 cm (was 16).
- **The editor shows every slot:** while it's open, each slot is a sphere of blue light the size of
  its reach, drawn over your body.
  - The slot picked in the editor is bright, the others faint.
  - The picked slot's first thing still shows at its place too.
  - Size and Reach change the sphere as you click them, and Move moves it.

Try:

1. Put a hand flat on your right hip and squeeze: food comes out. Do the same on the right of
   your chest for a scroll, and on your belly in front of your left hip for a potion. Look down
   at your hand while you do it, and also try it looking straight ahead.
2. Reach over your right shoulder for an arrow, and over your left for the shield, a few times
   each, with the head straight and turned.
3. Open the editor (hold a grip still in a slot for 1.5 s) and look down. Pick each slot on the
   left in turn: its sphere lights up.

Look out for:

- **Grabbing by accident.** Does anything come out when you don't mean it to, with your hands
  resting at your sides or your belly?
- **Over the shoulder.** Does the shield come out when you reach for the left shoulder for
  something else?
- **The spheres.** Too bright, too faint, or in the way of the editor?

## The slots you start with (right-handed; mirrored for a left hand)

| Slot | Where | Gives out |
|---|---|---|
| Healing | the belt, in front of your left hip (your off hand reaches it while the sword hand fights) | potions and food, what heals most first; then what restores magicka or fatigue; never a potion that restores nothing |
| Arrows | over your right shoulder, behind your head (where Oblivion wears its quiver) | arrows, the ones you have equipped first, then the strongest |
| Scrolls | high on your chest, on the right | scrolls, then poisons, the most valuable first |
| Food | at your right hip | food, what restores the most fatigue first (most of Oblivion's food restores fatigue, not health) |

## What to try, in order

1. **Find the slots.** Move a hand slowly round your belt, chest and right shoulder.
   - Entering a slot gives a tick in that hand.
   - An empty slot gives a fainter tick.
2. **Drink a healing potion.**
   - Put your left hand in front of your left hip, at the belt, and squeeze: a Strong Potion of
     Healing comes out, upright in your fist, with the game's sound of taking a potion out.
   - Bring it to your mouth, just under your nose, and hold it there. You feel gulps in your hand,
     then a thud, and the game drinks it: its swallow, its effect, one fewer in your inventory.
   - Hold it there only briefly, or wave it past your face, and nothing happens.
   - Draw again: the next is another strong one, then the ordinary ones, then the weak ones.
   - Drink too many in a row and the game says you can't drink more right now. The potion stays in
     your hand.
3. **Eat.** Squeeze at your right hip: the Leek, the Ale, an apple or some cheese comes out. At your
   mouth you feel bites, then it's eaten, with the game's chew.
4. **Put something back.** Draw a potion and let go of it in any slot, its own or another, or a
   holster: it goes back, with the game's sound of putting it away.
5. **Drop something.** Draw a potion and let go of it in front of you, or throw it. It falls, and
   where it comes to rest the game drops it into the world. You can pick it up again like anything
   lying about, and put it in a slot to take it back.
6. **Read a scroll.** Squeeze high on the right of your chest: a scroll (the most valuable first:
   the save's Flash Bolt or an Electric Shell). Hold it up in front of your eyes, about a forearm
   away. After a moment it's read and the game casts it: Flash Bolt where you look, Electric Shell
   as a shield on you.
7. **Poison your axe.**
   - Draw the war axe from your left hip with your right hand, and swap it into your left.
   - Draw a poison from the chest pouch with your right hand. The scrolls come first, so read them
     all, or reorder the slot (step 9).
   - Touch the bottle to the axe. The game says "Strong Poison of Severing used." and the axe is
     poisoned.
   - With no weapon in the other hand, a weapon that's already poisoned, or a staff, you get the
     game's own refusal.
8. **Draw an arrow.** Reach over your right shoulder and squeeze: an iron arrow, the equipped
   kind. Let go of it over the shoulder to put it back.
9. **The holsters' editor.** Squeeze in a slot and keep the grip held, without moving your hand, for
   a second and a half. You feel a firm pulse. Whatever came out goes back, and the editor floats
   in front of you. It's made of Oblivion's own parchment, rows, icons and buttons, in three
   strips: the slots, the categories, the buttons. Point with the laser and pull the trigger:
   - **A slot** on the left picks it. Under its name is what it gives out now. Look down: every
     slot is a sphere of blue light the size of its reach, the picked one bright, and its first
     thing shows, still, at its place.
   - **A category** in the middle puts it in the slot, last. Its place in the order shows on the
     right of the row, and the picked one shows as `< 2 >`. Pick it again to take it out of the
     slot.
   - **Earlier / Later** move the picked category up or down the slot's order.
   - **The order** button cycles through: healing first, magicka first, fatigue first, most
     valuable, heaviest, lightest, newest, by name, most of, strongest, and my own order (what you
     last put back in the slot comes out first).
   - **Empty** sets what an empty slot does: a tick, nothing, or anything (the order's first of
     everything you have).
   - **Size** and **Reach** set how big the slot is (what you let go in it goes back in) and how
     far out you can still take from it.
   - **Worn first**: the things you have on (the equipped arrows) come first.
   - **Move**: reach where you want the slot on your body and squeeze either grip. It's there now.
     Behind your head, it only counts there (over a shoulder).
   - **New slot**: a new, empty slot, placed at once by your next squeeze. Then give it
     categories.
   - **Remove** takes the picked slot away.
   - **Ok**, or B, closes it.

   Every change is kept at once, in your RustVR settings for Oblivion. The editor also opens from
   **Holsters** at the bottom of RustVR's own settings, beside the pause menu.
10. **Swap weapons from a slot.**
    - Make a slot with Weapons in it: New slot, squeeze where you want it (your right thigh, say),
      then pick Weapons.
    - Reach there and squeeze: the dagger or the shortsword comes out. The game equips it (its
      sound), and your weapon in that hand is now it, with its own reach and damage.
    - Let go at your left hip and it hangs there.
    - The war axe is in the weapons slot now, to draw back.

Under the Hands tab of RustVR's settings: how long to hold something at the mouth (0.5 s), how long
to hold a scroll up (0.8 s), and how long a grip held in a slot takes to open the editor (1.5 s; 0
turns it off).

## What was checked in the game (emulated headset, captures and logs)

- **Drinking.** A Potion of Healing drawn from the healing slot and held at the mouth: the game
  drank it (4 to 3 in the inventory), and the copy in the hand went.
- **The healing order.** On the playtest save the slot held: strong (heals 50), ordinary (35),
  weak (20), then a Potion of Sorcery (magicka 75). The strong one came out first and was drunk.
- **Eating.** Food from the right hip (the Leek: the most fatigue, then the most valuable) eaten at
  the mouth (1 to 0).
- **Dropping.** A potion let go in front of the chest fell. At rest the game dropped it there (3 to
  2), and it was offered to the hands again at exactly that place.
- **Arrows.** An iron arrow drawn from over the right shoulder, held along the fist.
- **Reading.** A Flash Bolt scroll drawn from the chest pouch and held up in front of the eyes: the
  game used the scroll and cast it (1 to 0).
- **Poison.** The axe drawn from the left hip into the left hand, a poison from the chest pouch
  touched to it: the game's own "Strong Poison of Severing used." and the poison gone (1 to 0).
- **The editor.**
  - It opened from a grip held in the food slot, the first time in one piece and now in three
    strips with the title over them.
  - The laser's clicks went through the game's own click to the editor:
    - Books put into the food slot, second in its order;
    - the order cycled from fatigue to most valuable;
    - the runtime saved both into the settings.
  - Move: the food slot was placed by the left hand's squeeze at the left thigh, and saved there.
  - New slot: a fifth slot, placed by the right hand's squeeze at the chest.
  - The picked slot's first thing (the food slot's leek) showed at the right hip while the editor
    was open, and went when it closed.
  - B closed it.
- **Weapons from a slot.** A slot of weapons at the right hip:
  - the Iron Longsword drawn from it was equipped by the game and became the weapon in the hand;
  - let go at the left hip, it hung there;
  - drawn again from the slot, the war axe replaced it.

## What wasn't checked

- **Your real headset.** Whether the mouth, the slots' sizes and the hold times feel right; how the
  bites and gulps feel on Quest controllers; reading the editor's text; comfort over a long
  session.
- **Never tried in the game:**
  - the potion limit's refusal (the copy should stay in the hand with a fail pulse after 1.5 s);
  - eating a quest item (refused by the game the same way);
  - the poison's refusals;
  - Earlier and Later (clicks reach them the same way as the rest);
  - the Empty, Size, Reach and Worn first buttons;
  - my own order (what's put back comes out first);
  - Remove;
  - Ok (B was used to close);
  - the Holsters button in RustVR's menu;
  - a left-handed player (the game's slots mirrored);
  - a slot placed behind the head.
- **Drop rotation.** A dropped thing lands where its copy came to rest, but facing the way you
  face, not the way the copy lay (the game's drop turns it to you), so it may turn a little as the
  game takes over.

## Decisions

- **Food restores fatigue in Oblivion.** The game is the source of truth: 33 of its 36 food
  ingredients restore fatigue first. So "what heals" is what restores health first. Then come
  things that restore magicka or fatigue, so cheese comes out of the healing slot only once the
  healing potions are gone. Things that restore nothing (a Potion of Feather) never come out of a
  healing slot. The food slot orders by fatigue.
- **The order is the categories' first, then the rule.** A restoring rule puts everything that
  restores its own stat ahead of the rest, whatever the category (healing food before a magicka
  potion).
- **Eating and drinking take half a second at the mouth,** with bites (food) or gulps (drink) felt
  every 0.16 s, never while the hand moves faster than 1.2 m/s or is held back by a wall.
  - The mouth is generous: within 9 cm of a point 7.5 cm under and 5 cm in front of your eyes.
    A small mouth is a common complaint in VR games.
  - The game does the rest: the drink or the chew, the effects, the potion limit, quest items.
- **Scrolls are read, not eaten:** held up in front of the eyes, 12 to 55 cm away, within 23
  degrees of where you look, for 0.8 s. The game casts it as the player's own scroll use, where you
  look.
- **Poisons go on the weapon in your other hand by touch** (0.3 s), doing the game's own steps
  without its yes/no box: the touch is the yes. You get the game's own refusals and its message.
- **Let go anywhere but a slot and the game drops it** where it comes to rest. Your thing really
  leaves the inventory and lies in the world. The alternative, springing back to its slot, was
  rejected: players hate auto-returns they didn't ask for, and a dropped potion should be a
  potion on the floor.
- **Weapons drawn from a slot are equipped** (the game's own equip and sound). The game's own
  weapon object takes the copy's place in the hand (`ObjectDesc::replaces`). The weapon you have on
  isn't one of the slots' things: it's at your hip already.
- **The editor is a menu of our own made of the game's UI:**
  - its own menu type, 1050, which the game's factory doesn't have, so it's hooked;
  - its XML is made of Oblivion's prefabs and textures and the inventory's row look (the equipped
    marker for what's in, the worn highlight under the laser);
  - the categories' icons are the game's own (a thing you have in each, else the game's icon of
    its kind);
  - the button labels are ours, in the game's fonts. "Ok" is the game's own word, from its
    message box.
- **The editor opens on a grip held still in a slot** (VRIK's hold-to-move, which players found
  the easiest customisation), and from RustVR's menu beside the pause menu. Move places a slot by
  your own hand on your body, not with sliders.
- **Slots are kept per game** in `%LOCALAPPDATA%\RustVR\settings.toml` under
  `[[holsters.games.oblivion]]`. The game's own four are used until you change anything. The
  settings file is the player's, so it's per player, not per save.
- **Default places** keep clear of the holsters the game's weapon and shield hang in. The healing
  slot is in front of the left hip, not at it: the left hip is the sword's. The arrows slot is over
  the right shoulder: if a two-handed weapon hangs there, the weapon comes first.
- **Keys don't draw:** the game's key model (`clutter\key\key.nif`) is an old NIF version the
  extractor can't read yet, so keys (and lockpicks) have no model and aren't handed out. Two cave
  ingredients have the same problem.

## Archery (the `overnight/archery` branch), when the two are merged

- **Archery has its own quiver.** It's a set of zones over the shoulder and at the hip
  (`rig::zones::quiver_zones`) that hand out the bow's `OBJ_ARROW` templates while the other hand
  holds a bow. Here the arrows are a slot like any other, giving the game's arrows as things
  (props) in the hand.
- **How they should meet:**
  - In `Rig::update_hover`, check the quiver before the slots while the other hand holds a bow by
    its grip: there an arrow is for nocking.
  - Give the quiver's zones from the slots that hold Arrows (`pockets::zones` of each slot whose
    categories include the game's arrows), not fixed places. Then moving the Arrows slot in the
    editor moves the quiver, and archery's `Quiver` setting (shoulder, hip, both) becomes the
    default Arrows slots.
  - With no bow in the other hand, the Arrows slot hands out an arrow as now.
- **The protocol:** archery's region (`OFF_BOWS`) is under the old end of the mapping, and
  this branch's regions (`OFF_INVENTORY` on) are in the 64 KB the mapping grew by. Both
  bump `VERSION` to 12: make it 13 when merging.

## Not done

- **No ghost of what's in a slot during play.** You find a slot by its tick and learn what it holds
  by drawing, or in the editor: it lists each slot's first thing, shows every slot as a sphere of
  light at its place, and the picked slot's first thing there.
- **Slots don't move with the body's crouch or lean:** they hang off the head-following anchor the
  holsters always have, as the research recommends (since round 2 from the neck, so looking down
  doesn't move them).
- **Torches** aren't a category (section 5's). **Apparel** isn't either.
- **No renaming slots:** there's no keyboard in VR. New slots are "Slot 5", "Slot 6"...
- **No bite-by-bite eating:** Oblivion eats a thing whole, so one hold eats one.
- **Books** can be put in a slot and drawn, but reading one is section 10's (the book held open).
