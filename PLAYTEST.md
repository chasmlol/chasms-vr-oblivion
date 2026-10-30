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
