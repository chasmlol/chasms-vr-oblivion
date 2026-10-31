# Playtest: interactions by hand

Save: rustvr-lock

Each piece has its own `.bat` in `<worktrees>\interact`. Research and design: `docs/research/interact.md`.

## 1. Lockpicking at the lock — `playtest-lock.bat`

Boots into rustvr-city (Odiil Farm) with 20 lockpicks, two locked chests a metre ahead (Easy on
the right: two tumblers to pick; Very Hard on the left: all five), and saves as `rustvr-lock`.

Try, in order:

1. **Draw a lockpick.** Reach across with the right hand to the left chest pocket and squeeze: a
   lockpick (the game's model) in the hand. (A new slot, "Lockpicks"; repair hammers have one at the
   front of the belt on the right, "Hammers".)
2. **Put its tip to the Easy chest** (the right one), on its front where a lock would be. The game's
   lock opens there: its tumblers stand in a row just in front of where the tip touched, and the
   lock's level, your picks and your Security float beside them. No flat lock menu anywhere.
3. **Pick it.** Move the tip across the row to choose a tumbler (the game's pick scrapes under it),
   flick the tip up to push it, and squeeze the trigger while it hangs at the top. You should feel a
   tick as it rises, a pulse when it reaches the top, a click when it sets, a jolt if the pick breaks;
   and hear the game's own lock sounds. The chest unlocks when the last one sets (the bolt opens).
4. **Miss on purpose**: squeeze while a tumbler is falling. The pick breaks (one fewer), and set
   tumblers fall as your Security allows.
5. **Pull out**: take the pick a hand's width away from the lock: it shuts, as B does.
6. **The Very Hard chest** (left): five tumblers, faster.
7. **A** is Auto Attempt; **B** closes the lock.
8. **The old way**: with an empty hand, look at a locked chest and use the trigger (Activate). The
   lock should stand where you were looking on the chest, and your right hand's grip works it.

Tell me especially:

* Do the tumblers stand upright, the right size, and at the tip? (They're the game's own lock-menu
  models, put into the world; how they're turned is measured from the model at run time. If they're
  lying down, sideways or tiny, that's the thing to fix first.)
* Does choosing a tumbler by moving across feel right at 2.5 cm apart?

What I checked: the adapter and the runtime build; unit tests (the nearest point of a triangle, the
keyhole's right/up/out as the player facing it sees them, a model turned to the keyhole, the tip of a
held thing). The game's lock logic is unchanged (its own cursor, mouse and click).

What I couldn't: anything in the game (I never run it). Not tried: a locked door (the same code: any
locked door or container's collision), an owned lock's crime, the Skeleton Key.

## 2. Repairing with the hammer — `playtest-repair.bat`

Boots into rustvr-base (a Nord in iron armour) with ten repair hammers, an iron longsword and an iron
shield equipped, everything you have on worn down to 40%, and saves as `rustvr-repair`.

Try, in order:

1. **Draw a hammer** with the right hand from the front of the belt on the right ("Hammers", a new
   slot): the game's repair hammer in the hand.
2. **Draw the sword** with the left hand and hold it out; **strike its blade** with the hammer's head.
   Each strike is the game's own repair: its clang, Armorer used, a little health back (the sword's
   card over the left hand shows its condition). Both hands feel it.
3. **The shield**: take it from over the left shoulder and strike its face.
4. **Armour you're wearing**: strike your left forearm or hand (the gauntlets), your chest (the
   cuirass), your thigh (the greaves), your head (the helmet): each is repaired where it's worn.
5. **Lift the hammer off between strikes**: a strike needs the head to arrive at speed (1.2 m/s),
   and the next one needs it lifted 8 cm off first. Pressing or rubbing does nothing.
6. **Keep going** until the hammer breaks (the game's chance by your Armorer): its own break sound,
   one hammer fewer.
7. **In combat** (attack someone first): a strike gets the game's own "You cannot repair items during
   combat!".

Tell me especially: does a strike register when you'd expect it to (too easy, too hard)? Does the
head feel like it's where the hammer's head is drawn?

What I checked: builds; unit tests (the hammer's head across the far end of its model, lines against
boxes and limbs). The repair itself is the game's own menu click on the struck thing's row.

What I couldn't: anything in the game. Not tried: an enchanted thing below Journeyman (the game's
message), an Expert's repair past 100%, armour carried but not worn (not struck in the hand: put it
on, or use the hammer from the inventory as ever).

## 3. Soul gems and enchanting — `playtest-soul.bat`, then `playtest-altar.bat`

`playtest-soul.bat`: rustvr-base (Odiil Farm) with three empty petty soul gems, two filled common
ones, the Longsword of Soul Embers equipped with its charge down to 20%, the Soul Trap spell readied
and a sewer rat a few metres ahead; saves as `rustvr-soul`.

1. **Soul Gems slot** (new, the pouch at the small of the back): reach behind with either hand. The
   strongest soul comes out first: a filled common gem. **Held in the hand, a filled gem shimmers**
   with Soul Trap's own purple effect (the game draws filled and empty gems alike).
2. **Recharge by touch**: draw the sword (left hip) into one hand, the filled gem into the other, and
   hold the gem against the blade. You feel it build for about half a second, then the game's own
   recharge: the gem's soul into the sword, the gem used up (one fewer), its sound.
3. **Soul trap**: cast Soul Trap (the off hand's trigger, as any spell) at the rat, then kill it
   within the spell's time. The game says "You have captured a soul!" and plays its own art at the
   body; then the soul (Soul Trap's own projectile art) flies in an arc to a soul gem in your hand,
   or, with no gem in hand, round to the pouch at your back. Holding an empty gem up as it comes in
   is the nicest way: the hand feels it land, and a filled gem shimmers brighter for a moment.

`playtest-altar.bat`: the Arcane University's Chironasium (its Altars of Enchanting) with two filled
common soul gems and an iron longsword equipped; saves as `rustvr-altar`.

4. **Lay a gem on the altar**: draw a filled gem and hold it to the altar's book rest (the slanted top)
   for half a second: it builds, then the game's enchanting opens beside the altar with that gem
   chosen. Pick the thing to enchant in its own list, then the effects, name it, Create (laser).
5. **Gem and sword together**: the gem in one hand, the sword in the other, both held to the altar:
   the enchanting opens with both chosen.

Tell me especially: where the soul arrives (the hand, the pack), and whether the gem's shimmer reads
as "full".

What I checked: builds; unit tests (the soul's arc). The trap, the recharge and the enchanting are the
game's own code (its soul trap; its recharge and enchanting menus' own clicks).

What I couldn't: anything in the game; the soul trap hook and the flying art are untested. Not done:
armour, clothes and jewellery carried but not worn can't be held (the holsters don't hand out
apparel), so they're enchanted (and repaired) through the game's own lists.

## 4. The Sigil Stone, Ayleid wells, Welkynd and Varla stones — `playtest-sigil.bat`, `playtest-ayleid.bat`

`playtest-sigil.bat`: the top of a Sigil Keep (a random gate's Sigillum Sanguis), its stone in its
flame; saves as `rustvr-sigil`.

1. **Take the Sigil Stone**: reach into the fire and close your hand on the stone. It's solid; pull it
   towards you and it comes out of the flame after your hand. Let go early and it drifts back.
2. **Pull it 30 cm out**: a jolt, and the game takes over: its own art, the sigil stone into your pack
   (the Sigil Stones category), fame, and the gate collapses round you.

`playtest-ayleid.bat`: beside the Ayleid well outside Vilverin, with three Welkynd and two Varla
stones, the Longsword of Soul Embers on (its charge at 20%), your magicka drained, and a Welkynd stone
lying on the ground ahead; saves as `rustvr-ayleid`.

3. **The well**: put a hand into the column of light over it and hold it there: it builds for half a
   second, then the game's own well: "You draw power from the well." and its spell. Again the same
   day: "A drained well recharges slowly." (the game's rule).
4. **A Welkynd stone**: the right pouch at the small of your back ("Ayleid Stones", new) gives one.
   Squeeze the trigger hard on it: it's crushed, the game's own use (your magicka back, its words and
   sound), gone.
5. **A Varla stone**: the same: the sword's charge comes back.
6. **The stone lying ahead**: pick it up and crush it: taken and used at once.

What I checked: builds; unit tests (the stone after the hand, the well's column). The activations and
uses are the game's own (the stone's and the well's scripts, the inventory's use).

What I couldn't: anything in the game. The stone's grab (one of the runtime's constrained props, as
the chains are) and its 3D moved by hand are untested; the well's column is a guess at its light's
size (tell me if the hand has to be somewhere else).

## 5. Shrines and offerings — `playtest-azura.bat`, `playtest-wayshrine.bat`, `playtest-chapel.bat`

`playtest-azura.bat`: beside the Shrine of Azura at dusk (her hour), glow dust in your pack and a
pinch on the ground ahead; saves as `rustvr-azura`.

1. **Lay the offering**: pick up the glow dust from the ground and hold it to the shrine (its altar,
   the statue's base) for half a second. It builds, then the shrine's own script: its question ("Do
   you wish to offer Glow Dust...?") answered yes by the laying, the dust taken, and Azura speaks; her
   quest starts. If your character is below level 2 the game says "Azura demands a more powerful
   champion" instead (its rule): that's the laying working too.
2. **Something else** held to the shrine does nothing (only what its script names is an offering).

`playtest-wayshrine.bat`: beside a Wayshrine of Akatosh; saves as `rustvr-wayshrine`.

3. **Touch it**: an empty hand's palm on the wayshrine for a moment: its blessing (or "Seek greater
   fame to be judged worthy.", the game's rule).
4. **Kneel**: kneel (or crouch low) in front of it, facing it, for a second: the same.

`playtest-chapel.bat`: Chorrol's Chapel of Stendarr, its altar at the far end; saves as
`rustvr-chapel`.

5. **Touch or kneel at the altar**: its own words (the pilgrimage first, or its blessing once a day).

What I checked: builds; unit tests (which models are shrines, kneeling). The shrines' activations and
answers are their own scripts'.

What I couldn't: anything in the game. The Azura and wayshrine set-outs move you by coordinates
(`setpos`): if you arrive in the air or inside rock, say so. Things for offerings are mostly
ingredients and pelts, which no slot gives by default: give a slot those categories (the holsters'
editor), or pick the offering up off the ground as here.

## 6. Sitting and sleeping — `playtest-rest.bat`

Chorrol's Grey Mare at night (its chairs and benches; its beds upstairs, owned); saves as
`rustvr-rest`. Have a real chair behind you.

1. **Sit**: stand with your back to one of the inn's chairs or benches (close to it) and sit down on
   your real chair. After a moment the game seats you in it (its own sitting). Your view should drop
   once, as you sat, not twice.
2. **Stand up**: stand up for real: the game stands you up.
3. **A bed**: go upstairs, stand by a bed and sit down on it the same way: the game's own answer (an
   owned bed: "You cannot sleep in an owned bed."). Rent a room from the innkeeper and the bed opens
   its sleep and wait menu instead (sleep with the laser).

Tell me especially: does the view stay right when seated (not sunk into the chair, not floating)?
Does it sit you too eagerly (crouching to pick things up near a chair)?

What I checked: builds; unit tests (sitting and standing by the head's height). The sitting, the
standing up and the bed's checks are the game's own (its activation of the furniture, its Activate).

What I couldn't: anything in the game: the view's anchor while seated (the game's standing eye height
kept over the feet) is untested.

## 7. Surrendering and yielding — `playtest-yield.bat`, `playtest-surrender.bat`

`playtest-yield.bat`: Odiil Farm with a bandit a few metres ahead, fighting you; saves as
`rustvr-yield`.

1. **Yield**: look at the bandit and raise both hands over your head, apart, for a moment (a faint
   pulse in both hands while you hold them up). The game's own yield: the bandit "accepts your
   yield." and stops, or "rejects your yield!" and keeps coming (its own choice).

`playtest-surrender.bat`: Chorrol with a 100 gold bounty on you; saves as `rustvr-surrender`.

2. **Surrender to a guard**: when a guard comes for you (or walk up to one), raise both hands at them:
   their crime dialogue opens at them (pay the fine, go to jail, resist arrest), worked with the
   laser.
3. **Resist, then yield**: pick "resist arrest", let the guard fight you, then raise your hands: the
   game's yield to a guard takes you back to the arrest.

What I checked: builds; unit tests (hands up). The yield is the game's own (its activation with Block
held), the dialogue the game's.

What I couldn't: anything in the game: that the game reads the Block held for the yield the frame
before the activation is untested.

## 8. Harvesting plants by hand — `playtest-plants.bat`

Outside Vilverin with four plants set out around your feet: a Nirnroot ahead (it always gives),
Lavender to the left and Columbine to the right (four in five give), a Fly Amanita close in front
(one in two). Saves as `rustvr-plants`.

1. **Pluck**: put a hand into a plant (its flower or cap), close the grip, and pull the hand away
   about a hand's width while still closed. Closing on it is a light buzz that grows as you pull;
   the pluck snaps. The game's own harvest: "You successfully harvest Lavender." and its sound, or
   "You find nothing of use." (its season's chance).
2. **In the hand**: when it gave, the ingredient is in the hand that plucked it (keep the grip
   closed until it's there, a moment). Put it in a slot (it's already in the pack), hold it at the
   mouth to eat it, or let it go to drop it.
3. **Plucked already**: the same plant again gives "You find nothing of use." (the game marks it
   harvested until the cell resets; it doesn't look any different).
4. **Not by accident**: reach through a plant and out again with the grip open, or close the grip
   in it and open it again without pulling: nothing is harvested.

What I checked: builds; unit tests (a hand in a plant's bounds); the runtime's rig tests with the
new hand-out.

What I couldn't: anything in the game: whether a plant's meshes' bounds are where its flower is,
whether 7 cm is a natural pluck, and whether the ingredient reaches the hand before you open it.

## 9. Keys at the lock — `playtest-keys.bat`

Odiil Farm with four keys on you and two chests locked a metre ahead: the right one (Easy) opens
with the Whitmond Farm Key, the left one only with the Faregyl Inn Key (it can't be picked); the
Sleeping Mare Key fits neither, and the Odiil Farm Key is the farm's own. Saves as `rustvr-keys`.

1. **The ring**: reach just behind your left hip, on the belt, and close the grip: a key comes out,
   the key to the nearest locked chest first. (If you've changed your slots in the holsters'
   editor, the new Keys slot isn't there: add one with the Keys category, or reset to the game's
   own.)
2. **In and turned**: put its tip to the chest's front where a keyhole would be. If it's that
   chest's key it goes in (a click); turn your wrist about the key, either way, to about 60° (a
   detent every 20°, then a clunk). The chest unlocks and opens, and "Opened the container with a
   key." shows in the corner.
3. **The wrong key**: put the other chest's key, or the odd one, to a chest: a dull bump, it doesn't
   go in, nothing happens.
4. **Out**: put the key in, then pull it out (6 cm) without turning: nothing happens; put it back
   in to try again.
5. **A door**: if the farmhouse door is locked, the Odiil Farm Key opens it the same way: it
   unlocks, "You opened the door with Odiil Farm Key." shows in the corner, and it opens (into the
   house).

What I checked: builds; unit tests (the key's turn about itself, not its swing; the nearest lock's
key first).

What I couldn't: anything in the game: where a key's tip is on its model (the far end of its
longest side from the grip), and whether the game's Ok box is answered the frame it opens.

## 10. Doomstones and runestones — `playtest-stones.bat`

Three metres south of the Atronach Stone, facing it, at ten at night (its stars shine). Saves as
`rustvr-stones`. (Moved there by coordinates: if you arrive in the air or in rock, step clear.)

1. **A palm on the stone**: put an open hand flat on the standing stone and hold it there (a
   building buzz, 0.6 s). Its own question: "Marked by a special fate, you rule your destiny. Do you
   choose to steer by the stars of the Atronach?" No / Yes, answered with the laser (Yes replaces the
   Doomstone power you have).
2. **By day**: wait until morning and touch it again: "No stars shine upon the doomstone."
3. **Not by accident**: brush past it, or touch it with a hand holding something: nothing.

What I checked: builds; unit tests (the stones known by their models).

What I couldn't: anything in the game: the stone's collision is the surface touched, as at the
wayshrines.
