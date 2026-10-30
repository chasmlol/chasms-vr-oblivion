# Interactions by hand: research and design

More of Oblivion done with the player's own hands and body instead of a menu or a button. Each
piece runs on the game's own system (its numbers, skills, scripts, sounds and models); only how the
player does it changes. The pieces, in the order they're built:

1. Lockpicking at the lock itself.
2. Repairing with the hammer.
3. Soul gems and enchanting.
4. The big magic moments: the Sigil Stone, Ayleid wells, Welkynd and Varla stones.
5. Shrines and offerings.
6. Sitting and sleeping.
7. Surrendering and yielding.

Then a survey of what else could be done by hand ([More to do by hand](#more-to-do-by-hand)).

The game here is Oblivion 1.2.0.416 (Steam GOTY).

## 1. Lockpicking at the lock

### How VR games do it

* **The lock where it is.** The VR lockpicking players like keeps the lock in the world: the pick
  goes into the keyhole of the door or chest in front of the player, and the lock's insides are
  felt more than seen. *Thief VR* and *Assassin's Creed Nexus* have the pick in the keyhole and the
  hand turning it, with the controller's rumble and the lock's clicks saying when it's right;
  *Half-Life: Alyx*'s multitool puzzles stand at the thing being worked, not in front of the face.
* **The flat version is the one players complain about.** *Skyrim VR* and *Fallout 4 VR* kept their
  flat-screen lock menus, floating in front of the player with the world gone, worked by turning
  the controller: the moment the game stops being VR.
* **Feel and sound carry the timing.** Where a lock has a moment to catch (a pin at the shear
  line), the games give it in the hand (a tick, a click) and in the ear, so the eyes needn't be
  glued to a diagram.
* **The hand's own movement is the input.** The pick moves where the hand moves it; nothing snaps
  the hand to the lock.

### Oblivion's lock

The lock is the `LockPickMenu` (menu type 0x3F6, `Data\Menus\lockpick_menu.xml`), made by
the reference's own `Activate` when a locked door or container is
activated by a player who has lockpicks and not its key. It's a menu: on the interface's menu
stack, so the world stops while the lock is picked. Its update runs only
while it's the top of the menu stack: taken off the stack (as a book is) its
tumblers would never move. So the world stopping is the game's own rule, and stays.

* **The lock's level** (`ExtraLock`'s: 0 Very Easy .. 4 Very Hard) picks how many of
  the five tumblers are to be picked (the first already set): level + 1
  (Very Easy one, Very Hard all five). The rest start set, up.
* **The tumblers** (five), each with its height (the time into its model's `UP`
  animation, 0 down .. the animation's end, about 0.667: up), when it reached the top
  (-1 not there; the game's tick), how long it hangs there (ms), its speed, whether moving,
  whether set, its `Tile3D` (the model `Lockpicking\Tumbler02.NIF` in the menu), and its
  `UILockTumblerMoveLP` loop.
* **A tumbler's rise** (from the pick's nudge): its speed and how long it hangs at the top
  come from the lock's level and the player's Security (in tens, between the game's tables at
  each level) and a random draw. It rises, reaches the top (`UILockClickNow`), hangs there,
  then falls back unless set.
* **The pick** (the tumbler it's under, the tumbler the cursor is over; it moves one
  tumbler at a time towards it, with `UILockPickScrape`; its place, its lift,
  its lift's speed; the menu's state): the mouse moved up more than 3 counts while the pick
  is under a tumbler lifts it (0.34), and the lift nudges that tumbler
  (`UILockTumblerNudge`) if it isn't already moving or set. The cursor's
  place across the lock is which tumbler.
* **Setting** is the mouse's click (the menu's `HandleClick`): while the tumbler hangs at
  the top it's set (`UILockTumblerLock`, the player's Security used: `UseSkill(30)`); at any other
  time it fails: the pick breaks (`UILockPickBreak`, one lockpick removed from the
  player, the player's "Lockpicks Broken" statistic counted) and set tumblers fall, as many
  kept as the player's Security mastery allows (one more with the Skeleton Key, which
  never breaks). Out of picks: "You have run out of lockpicks" and the menu shuts.
* **The crime.** The first attempt on an owned lock (`HandleClick`, through the player's own test
  against the lock's owner) is the game's own crime check, once per lock.
* **Auto Attempt** (button 5): the game's odds (`fLockPickAuto*` and the player's Security,
  its own roll); on success every tumbler is set, else the same failure.
* **Opening** (state 1): the bolt's `Open` animation, the door or container unlocked,
  the menu shut.
* **Its models** (NIF 10.1, which the runtime's extractor doesn't read, but the game's own model
  cache does): `meshes\menus\lockpicking\tumbler02.nif` (sequences `UP`, `Down`,
  `bump_up_and_bump_down`, `drop`, `set`, `reset`, `wiggle`), `pick.nif` and `skeletonkeypick.nif`
  (the menu's pick, chosen by whether the player has the Skeleton Key), `boltgearsloop01.nif` (the
  bolt), `menus\lockpicking\mech.dds` and `base.dds` (the art round them).
* **The lockpick** is a misc item (`meshes\clutter\key\lockpick.nif`); the Skeleton Key
  another. The menu holds both forms (the lockpicks, the Skeleton Key).

The merged adapter (`lock.rs`) already works the open menu by hand: the hand moved sideways is the
game's cursor along its tumblers, moved up is the mouse's lift, the trigger its click; A is Auto
Attempt and B closes it. The menu itself floated flat beside the door.

### Design

The game's lock stays the whole of the lock: its menu runs as ever (the world stops, its tumblers,
timing, Security, picks breaking, crime, sounds, auto attempt), worked through its own cursor, mouse
and click, so every rule is the game's. Only where it's seen and what moves it change:

* **The pick at the lock opens it.** A lockpick drawn from the body's slots (the holsters: it's a
  misc item) is the game's model in the hand. Its tip (the end of its model away from the hand)
  touching a locked door's or container's surface is the player's own `Activate` of it: the game
  decides (its key, its picks, "not while a crime alert is on"). Where the tip touched is the
  keyhole.
* **The lock at the keyhole.** While the menu is open its flat picture isn't shown. The lock's own
  tumbler models stand in a row at the keyhole, a little proud of the surface, upright, facing out,
  drawn by the game in its world (lit, in its depth), each held at its height in the game's own
  `UP` animation as the menu has it. The row is as wide as the hand's places for the tumblers.
* **The pick's tip works it.** The tip's place across the lock (along its row) is which tumbler the
  game's pick is under; the tip moved up is the lift; the pick hand's trigger catches the tumbler at
  the top (the game's click). The game's own sounds (scrape, nudge, the tumbler's loop, click-now,
  lock, break) play as ever, and the hand feels them: a tick as the tumbler rides up, a pulse when it
  reaches the top, a click when it sets, a jolt when the pick breaks.
* **Pulled out, it's over.** The tip taken away from the lock (more than a hand's width) closes the
  menu, as B does (the game's own close: "DRSLockOpenFail" if it didn't open).
* **What it says**: the menu's own lock level, picks left and Security, cut from its picture, float
  beside the lock (not the tumblers, which are there in the world).
* **Opened another way** (the trigger's Activate at the door, a door tugged by its latch): the lock
  stands where the eyes were on the thing; the hand is the pick as before (an empty hand's grip,
  if no lockpick is held).
* **The hands work under it**: the runtime's hands go idle under a menu. A menu the hands work in
  the world says so (`HUDL_HANDS`, the framework's), and the hands, and what they hold, keep going.

Calls made:

* The keyhole is where the tip first touched the locked thing (any of its surface), not a place
  worked out from the model: most locked doors are load doors with no hinge or handle the game
  knows, and the player puts the pick where a lock would be anyway.
* The world stays stopped while picking, as in the game (its menu can't run off the stack).
* The lock's tumblers are the menu's own models, not a new lock: a cutaway at the keyhole, as the
  game's own menu is a cutaway.
