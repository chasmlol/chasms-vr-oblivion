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

## 2. Repairing with the hammer

### How VR games do it

* **The strike is the input.** VR smithing (the anvil games, sandbox forge mods, the
  crafting in *The Walking Dead: Saints & Sinners*) has the player swing a real hammer at the thing:
  a strike counts when the head arrives fast enough, and each strike is answered at once with a
  clang, a spark of feedback and a jolt in both hands.
* **The work piece is held or set down where the player can hit it**, never a list. Holding a blade
  in one hand and hammering it with the other is the gesture players reach for first.
* **Repeated strikes, small gains**: the satisfaction is in the rhythm, each strike doing a little.

### Oblivion's repair

* **The hammer**: a misc item ("Repair Hammer", `Clutter\RepairHammer.NIF`;
  the game's own form). Using one from the inventory (the misc item's use) is
  refused in combat ("You cannot repair items during combat!", `iAllowRepairDuringCombat`,
  the game's own check) and otherwise opens the `RepairMenu` in its hammer mode.
* **The `RepairMenu`** (`Data\Menus\repair_menu.xml`, made for one of its modes)
  is the game's one list of the player's things to pick from: its mode is
  1 the player's hammers, 2 a merchant's repair for gold, 3 alchemy's, 4 and 5 the enchanting
  menu's item and soul gem, 6 the Sigil Stone's item. It counts the hammers the player has. Its rows
  (ids from 51) carry the thing's place in the player's inventory (value 4025, 0xFB9; the player's
  inventory makes a copy of that entry by its place: its form, its extra
  data lists, freed again with the game's own free).
* **A strike is a click on a row** (`HandleClick`, an id from 51 and the row's tile). In
  the hammer mode:
  * an enchanted thing needs a Journeyman armorer ("Only a journeyman armorer or higher may repair
    magic items."); no hammers left, its own message;
  * what a strike restores is the game's (the Armorer skill with `fRepairArmorerBase`,
    `fRepairArmorerMult`), up to the thing's full health, or a quarter past it for an Expert or
    Master (Armorer mastery 3 or more); one of a stack is split off to be repaired;
  * the hammer may break (`fRepairSkillBreakBase`, `fRepairSkillBreakMult`,
    `fRepairBreakApprenticeMult`; never for a Master): `UIArmorWeaponRepairBreak`, one hammer
    removed from the player, and the menu shuts with none left;
  * Armorer is used (`UseSkill(12)`), the player's equipment is updated, the list
    made again, and `UIArmorWeaponRepair` plays.
* **Health**: a thing's health percent is its inventory entry's own.
* **Worn things** are the player's inventory entries (`ExtraContainerChanges`) with an extra data list
  marked worn (`ExtraWorn` 0x1B, `ExtraWornLeft` 0x1C); an armour's slots are its
  `TESBipedModelForm`'s: head 0x1, upper body 0x4, lower body 0x8, hands 0x10, feet
  0x20, shield 0x2000.

### Design

The game's hammer repair stays the whole of the repair: each strike is one click of its own
`RepairMenu` in its hammer mode on the thing struck, made for that moment and gone again in the
same frame (as alchemy's menu is): its skill, what it restores, its cap, its break chance and the
hammer removed, its sounds, its messages, its refusal in combat. Only how a strike is made changes:

* **The hammer in the hand**: a repair hammer drawn from the body (the Hammers slot, the front of the
  belt on the right), the game's model. Its head is the end of its model away from the hand.
* **What it strikes**: what the player holds or wears, where it is:
  * the weapon in a hand (the equipped weapon, drawn), anywhere on its model's box;
  * the shield on the arm;
  * armour worn, struck on the body where it's worn: the helmet at the head, the cuirass on the
    chest, the greaves on the thighs and shins, the boots at the feet, the gauntlets on the other
    hand and forearm.
* **A strike** is the head arriving at one of them at 1.2 m/s or more (its fastest in the last tenth
  of a second: held things are solid to each other, so a hammer on a held blade stops there); the
  next needs the head lifted off again. Each strike is felt in both hands, the hammer hand harder.
* **What it says**: the game's own messages (the journeyman rule, the combat rule, out of hammers)
  and sounds. The thing's card over the hand (its condition) shows the gain.

Calls made:

* Armour that's carried, not worn, isn't struck in the hand: the holsters don't hand out apparel
  (its model is the ground model, which the slots don't draw yet). Put it on to repair it, or use the
  hammer from the inventory as ever (the game's own menu stays).
* The world doesn't stop for a strike (the game's menu would stop it: it's made and gone at once).

## 3. Soul gems and enchanting

### How VR games do it

* **Magic you can see travel.** *Skyrim VR*'s soul trap shows a stream of light from the body to the
  caster; VR spellcasting games (*Waltz of the Wizard*) make every magical
  transfer a visible thing moving through space to the hand, and the hand feels it land.
* **Crafting by placing.** VR crafting stations (*Saints & Sinners*' benches, *Waltz of the
  Wizard*'s cauldron) start the craft from what the player sets down on them; the choices that are
  really choices (which effect, how strong) stay a panel at the station.
* **Charging by contact**: holding a power source to the thing it charges (*Half-Life: Alyx*'s
  resin and batteries, *Into the Radius*'s batteries) is the gesture players reach for.

### Oblivion's soul gems, soul trap and recharge

* **Soul gems** (`TESSoulGem`): the soul it holds (none, petty .. grand),
  how big a soul it takes. A gem filled in play has its soul as `ExtraSoul` (the soul
  in it) on its extra data. The game draws a filled gem the same as an empty one (every lesser to
  grand gem is `Clutter\SoulGemLesser01.NIF`, filled or not). Azura's Star is 0x193.
* **Soul trap** (`SoulTrapEffect`, the effect `MGEF` STRP 0x189F, its model
  `MagicEffects\soulTrap.NIF`): when its target dies under it, the active
  effect fills the player's gem by the game's rules (the smallest
  that fits; "There is no Soul Gem large enough to capture the soul."), says "You have captured a
  soul!", plays its UI sound and counts it (the player's "Souls Trapped"), then puts the
  effect's model at the body playing `SpecialIdle_Soultrap` (a temporary effect). The model
  also has `SpecialIdle_Projectile`, `SpecialIdle_Cast`, `SpecialIdle_HitEffect` and
  `SpecialIdle_AreaEffect`; its enchant shader (the `EffectSetting`'s) is the purple shimmer of a
  thing enchanted with soul trap.
* **Recharging with a gem** (a soul gem used from the inventory): refused in combat unless
  `iAllowRechargeDuringCombat` (1 by default; its own words), refused for an empty
  gem ("This soul gem is empty..."), else the `RechargeMenu` (made for the gem's entry)
  lists the enchanted things that aren't full. A row clicked (its `HandleClick`;
  rows from id 51, their place in the inventory as a value): the soul's charge
  added up to the enchantment's maximum (its `TESEnchantableForm`'s), the gem used up (Azura's
  Star emptied instead), `UIRechargeItem`'s sound, and the menu shut when nothing's left to
  recharge. A thing's charge is `ExtraCharge` on its extra data.

### Oblivion's enchanting

* **The altar**: the Arcane University's Altar of Enchanting (`ICEnchantingStation`, four in
  the Chironasium, `ICArcaneUniversityEnchantingCenter`; one in the Arch-Mage's quarters), its
  script `EnchantingStationScript`: on the player's activation, `ShowEnchantment`.
* **The `EnchantmentMenu`**: its soul gem and item are inventory
  entries chosen through the `RepairMenu` in modes 5 and 4 (rows of the player's gems and of things
  that can be enchanted); choosing either sets it and works the menu out again;
  choosing an item of the other kind (a weapon after apparel) clears the effects chosen.
  The effects (from the player's known spells' effects), magnitudes, area, duration, the name, then
  Create: the game makes the enchanted thing, uses the gem, takes the item.

### Design

* **The soul flies to the gem.** The game's soul trap runs as ever (its gem, its message, its art at
  the body); as it traps a soul the effect's own model, playing its `SpecialIdle_Projectile` art,
  flies from the body to the soul gem in the player's hand, else into the pack (the Soul Gems slot at
  the small of the back), in an arc over 1.2 s, and the hand feels it land.
* **Filled gems look filled**: a gem with a soul in the hand wears the Soul Trap effect's own enchant
  shimmer (its effect shader, over the gem: the overlays `crate::magic` draws a weapon's enchantment
  with), fading in brighter as a soul has just arrived.
* **Recharging by touch**: a filled gem held to the enchanted weapon in the other hand for 0.6 s
  (felt building) is the game's own recharge with that gem: its recharge menu for the gem made for
  a moment, the weapon's row clicked.
* **The slot**: Soul Gems (the small of the back) gives the strongest soul first (to recharge with),
  then the biggest empty gem.
* **Enchanting at the altar**: laying a filled soul gem and something enchantable on the Altar of
  Enchanting (let go on its top, they stay there, still the player's) is the player's activation of
  the altar (its own script, `ShowEnchantment`), with those two chosen in its menu as its own lists
  choose them; the menu floats beside the altar for the effects, the name and Create, worked with the
  laser. Taken off the altar again, they're no longer chosen.

Calls made:

* The soul goes to a gem in the hand when there is one, whichever gem the game filled (the game picks
  the smallest that fits; the flight doesn't change which).
* The choices that are menus in the game (which effects, how strong, the name) stay its menu, at the
  altar: only the gem and the item are laid on it.

## 4. The big magic moments: the Sigil Stone, Ayleid wells, Welkynd and Varla stones

### How VR games do it

* **The climax is taken by hand.** VR's big set pieces (*Half-Life: Alyx*'s cores pulled from their
  sockets, *Asgard's Wrath*'s relics) have the player physically take the thing: it resists a
  little, comes free, and the world answers.
* **Magic is touched, not clicked**: a hand put into light or energy to draw from it, felt building
  in the controller before it gives.
* **Consumables are used by a gesture with the thing itself** (crushed, eaten, drunk), not a button.

### Oblivion

* **The Sigil Stone** (activators named "Sigil Stone", `Oblivion\Sigil\SigilStone01.NIF`; the Great
  Sigil Stone `SigilStoneGREAT01.NIF`), hanging in its column of fire (`SigilLightTower`, a separate
  reference) at the top of the Sigil Keep (`Sigillum Sanguis`, e.g. `OblivionRDCitadel01Lord`). Its
  script (`SigilStoneRandomSCRIPT`; the main quest's have their own) on the player's
  activation plays its `Unequip` sequence, and once that's done: a sigil stone into the player's
  inventory (`LL2SigilStones100`), fame, the fire shield shader on the player, its parent activated
  (the gate's collapse: the player out to Tamriel) and the stone destroyed. Its model's sequences:
  `SpecialIdle` (its turning), `Unequip` (taken, `AMBSigilStoneTakePU`).
* **Ayleid wells** (`ARWellGrate01`, outside the ruins), their script `AyleidWell`
  gives, on the player's activation, once a day (`GetDayOfWeek`), "You draw power from the
  well." and the `AyleidPower` spell cast on the player, its `forward` animation; on the same day
  again: "A drained well recharges slowly." Between uses its `Backward` loop (its light and sound).
* **Welkynd stones** (misc 0x191) and **Varla stones** (0x194),
  used from the inventory (the misc item's use: the inventory's click is
  `Actor::EquipItem`): a Welkynd stone restores the player's magicka ("You have used a Welkynd
  Stone's power...", `ITMWelkyndStoneUse`); a Varla stone recharges every enchanted thing the player
  has on (refused in combat unless `iAllowRechargeDuringCombat`); each is used up.

### Design

* **The Sigil Stone is taken by hand**: offered to the hands as one of the game's own props on the
  game's constraints (as the hanging chains are, `OBJF_CONSTRAINED`), a ball the size of the stone.
  Taken in a hand where it is, it comes out of the flame after the hand (dragged a tenth of a second
  behind), the game's own 3D with it; pulled 30 cm out, it's the player's own activation, and its
  script does the rest (its art, the stone into the pack, the collapse), the stone still in the hand
  until the game takes it. Let go before then, it drifts back into its flame.
* **A hand into a well's light** (a column 45 cm round its middle, from 20 cm over the well to 3 m)
  held there half a second, felt building, is the player's activation of the well: its own script,
  so once a day, its words and its spell.
* **Welkynd and Varla stones crushed in the hand**: the trigger squeezed hard on one held (drawn from
  the pack's right pouch, the Ayleid Stones slot, or taken from where it lies in a ruin, then picked
  up by the game first) is its use as the inventory's click: its power, its words, its sound, the
  stone gone. A crack in the hand.

Calls made:

* The stone follows the hand without turning with it (its own turning is its art's).
* The well's light is a fixed column round its reference's place, not measured on its model.
