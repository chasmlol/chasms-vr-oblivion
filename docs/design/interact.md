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

Then a survey of what else could be done by hand ([More to do by hand](#more-to-do-by-hand)), and
the three of it picked next:

8. Harvesting plants by hand.
9. Keys at the lock.
10. Doomstones and runestones.

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

## 5. Shrines and offerings

### How VR games do it

* **Ritual is a gesture.** VR games that have altars (*Asgard's Wrath*'s offerings, *Skyrim VR*
  mods' prayer by kneeling, *The Elder Scrolls: Blades*' shrines on mobile VR) let the player place
  the offering with their hand and kneel or touch to pray; the reward answers the body, not a
  button.
* **Held, not clicked**: a short hold (half a second, felt building) separates a deliberate offering
  from brushing past the altar.

### Oblivion

* **Daedric shrines** (activators `DA<Prince>Shrine`, `Architecture\DaedricStatues\DaedricShrine*.NIF`),
  each with its own script (`DAAzuraStatueScript`...): on the player's activation, its own
  checks (the quest's stage, the hour, the player's level, `player.getitemcount <offering>`), its
  own message boxes ("You do not have the proper offering Azura requires.", "Azura demands a more
  powerful champion...", and the question "Do you wish to offer Glow Dust to the altar of Azura?"
  Yes / No). Answered yes (`GetButtonPressed` 0 in its `GameMode`), it takes the offering
  (`RemoveItem`), the Daedra speaks (`Say`, the player's controls held), and the quest starts. The
  forms a script uses are its referred-to list (the `Script`'s, each a variable's
  form +8): the offering among them.
* **The message box** (`MessageMenu`): its buttons (ten tiles); a click
  (its `HandleClick`) on id 4 + n is button n, which the script reads.
* **Wayshrines** (`WayshrineAltar01.NIF`, a script per god: `ShrineJulianos`...): on the
  player's activation, the god's blessing for a pilgrim with fame (its spell, `FameJulianos`), or
  "Seek greater fame to be judged worthy."; all nine visited, the Pilgrim's Grace power.
* **The chapels' altars** (`CathedralStenIntAltar02.NIF`, "Altar of <god>", a script per god:
  `AltarofAkatosh`...): on the player's activation, a sinner (a bounty, more infamy than
  fame) is refused ("Seek not the Dragon's blessing, sinner!"); one who hasn't made the pilgrimage
  to the god's wayshrine is told to; else once a day the god's blessing spell (`AltarAkatosh`: its
  cures and fortifications).

### Design

* **An offering laid on a Daedric shrine**: a thing held to the shrine (its altar or statue) for half
  a second that the shrine's own script names is the player's activation of the shrine: its script
  decides everything as ever, and its own question is answered yes by the laying (the message box's
  first button, as the player's click would). A message box without a question (not the proper
  offering, too weak a champion, the wrong hour) is left for the player to read and close. A thing
  lying about (one picked up by hand from the ground) is taken into the pack first by the game, since
  the script counts the pack.
* **A wayshrine or a chapel's altar, touched or knelt before**: an empty hand's palm on it for 0.6 s,
  or kneeling before it (the head below 72% of the standing eye height, within 1.6 m, facing it) for
  a second: its activation, its own blessing or refusal.

Calls made:

* Which thing is an offering comes from the shrine's own script (what it names), not a list of ours.
* Kneeling is the head's height and the way it faces, not a pose of the legs.

## 6. Sitting and sleeping

### How VR games do it

* **Sit for real.** VR games with seats (*Half-Life: Alyx*'s chairs are props, *The Walking Dead:
  Saints & Sinners*' and *Skyrim VR* mods' chairs, VRChat's stations) let the player sit down in
  their room and put the avatar in the seat; the one thing they get wrong when they don't is the
  double drop: the game lowers its camera to the seat and the player's real head drops too, so the
  view sinks into the floor.
* **Getting up is standing up.** The seat lets go when the head rises, not on a button.
* **Sleep is the bed's menu.** Even physical VR games keep the time skip a menu; what's physical is
  getting into the bed.

### Oblivion

* **Furniture** (`TESFurniture`, form type 0x20): chairs, benches, stools and beds, their markers
  (`FurnitureMarkerNN`) saying where and how an actor sits or sleeps. The player's activation of
  furniture decides by its markers: a seat puts the player in it (the game walks them to the marker
  and plays the sit animation, its first-person camera at the seated eye); a bed asks its own way.
* **The sit and sleep state** (the process's `GetSitSleepState`): 0 normal, 1 loading
  the sit idle, 2 want to sit, 3 waiting for the sit animation, 4 sitting, 5 want to stand, 6..10
  the sleep's own (loading, want to sleep, waiting, sleeping, want to wake). Seated, the player's
  Activate stands them up (5).
* **Beds** open the `SleepWaitMenu` (`sleep_wait_menu.xml`) after the game's own checks: "You
  cannot sleep while trespassing.", "You cannot sleep in an owned bed." (unless rented or the
  player's), "You cannot sleep while in combat.", "You cannot sleep when enemies are near by.", a
  prisoner only in a jail bed.

### Design

* **Sitting down for real on a chair, a bench or a bed** (the head below 78% of the standing eye
  height for 0.3 s, with the furniture's collision under the head within 45 cm along the floor, at a
  seat's height) is the player's activation of it: the game's furniture as ever (a seat seats them,
  a bed asks for sleep or wait with its own ownership and trespass rules and crime).
* **Standing up for real** (the head back over 90% for 0.3 s) while seated is the game's Activate,
  which stands them up.
* **The view seated**: the anchor stays at the game's standing eye height over the feet while the
  game has the player seated in furniture, so the real sitting lowers the view once, not twice.

Calls made:

* The player sits down in their room (a real chair, or the floor); the game's seat follows. A
  player who activates a chair standing (the trigger at it) is seated by the game, and their view
  stays where their head is.
* Lying down isn't asked for: sitting on the bed is enough for the game's own sleep question.

## 7. Surrendering and yielding

### How VR games do it

* **Hands up is universal.** Social VR and VR shooters (*Pavlov*, *Contractors*' hostage modes,
  *Hitman 3 VR*'s guards, *The Walking Dead: Saints & Sinners*' standoffs) read both hands raised
  over the head as surrender, held a moment so a stretch isn't mistaken for it.
* **The answer is the other side's.** Surrender is asked for; the enemy decides.

### Oblivion

* **The player's yield** is the player's activation of someone fighting them while Block is held
  (in the player's activation of an actor: the actor in combat, the game's input says Block,
  control 6): the actor's own answer (`AcceptsYield`: its disposition,
  `fAIYieldBase`, its class's "Accepts Yields"/"Rejects Yields"), and if it accepts it stops
  fighting; its words "accepts your yield." or "rejects your yield!" as a notice;
  "Creatures never accept a yield!". A guard who accepts goes back to arresting the player.
* **A guard and a criminal**: a guard who wants the player (the player's bounty) comes
  to them and forcegreets ("Stop right there, criminal scum!"), and the player's activation of a
  guard opens the same crime dialogue: pay the fine, go to jail (`GoToJail`), or resist arrest.

### Design

* **Both hands up** (each over the head, apart, held 0.7 s, felt as a faint pulse in both hands)
  before someone the head looks at, within 8 m, who is fighting the player (or anyone, while the
  player has a bounty: a guard come for them) is the game's own yield: Block held in the game's
  input for a moment, then the player's activation of them. In a fight their own answer; with a
  guard, their crime dialogue at them, worked with the laser as any dialogue.

Calls made:

* The gesture asks; the game answers (its own accept or reject, its own dialogue). Raising the hands
  before nobody does nothing.

## 8. Harvesting plants by hand

### How VR games do it

* **Grab and pull.** Survival games in VR (*Green Hell VR*, *Into the Radius*' mushrooms) take a
  plant's produce with a grab on it and a pull away: the pull is the pluck, so brushing past or
  reaching through a bush takes nothing.
* **What's taken is in the hand.** The produce ends up in the fingers that plucked it, to be put in
  a pouch, eaten or dropped, never just counted into a menu.

### Oblivion

* **Flora** (`TESFlora`, form type 0x1F; `FLOR` records, 155 of them: flowers, mushrooms,
  Nirnroot) is an activator with a produce form first in it (ahead of its
  `TESForm`): a reference's base is the `TESFlora`, the ingredient in it, and
  the four seasons' chances beside it.
* **Harvesting is its activation** (`TESFlora::Activate`): a plant not
  harvested yet (the reference's flag 0x2000) gives its ingredient by the season's chance
  (`rand % 100 < chance`): into the activator's pack (the actor's `AddItem`, or the ingredient's
  script), then "You successfully harvest %s." (`sFloraSuccessMessage`) and `ITMPickupOrganic`.
  Else "You find nothing of use." (`sFloraFailureMessage`). Either way the reference is marked
  harvested (its flag 0x2000, and a change for the save) until the cell resets; the game
  draws a harvested plant as before.

### Design

* **A plant plucked**: a hand closing its grip in a plant (inside the bounds of one of its meshes,
  with 2 cm to spare) and pulled 7 cm away while still closed is the player's activation of the
  plant: the game's harvest, its chance, words and sound. Closing on it is felt (a light tension
  that grows with the pull), the pluck snaps.
* **In the hand**: when the harvest gave the ingredient (its count went up), the hand that plucked
  it, still closed and empty, holds it: the runtime's copy of that ingredient, as if drawn from a
  slot (put in a slot it's simply in the pack, held at the mouth it's eaten, let go it drops). The
  framework's inventory region asks for it (`Inventory::give_*`, from its padding: one of the
  player's things into a hand). A hand opened before the game answered leaves it in the pack.

Calls made:

* The pull is the pluck (a grab alone takes nothing), so a hand passing through plants, or picking
  up a thing lying in one, harvests nothing.
* A plant harvested already plucks to the game's "You find nothing of use.": the game doesn't draw
  harvested plants differently, so the hand finds out as the player would.

## 9. Keys at the lock

### How VR games do it

* **Put in and turned.** VR puzzle and horror games (*The Room VR*, *Resident Evil 4 VR*) have the
  key brought to its lock and turned with the wrist, a click at each stop; the wrong key doesn't go
  in.
* **The key ring.** Players expect the key for the door in front of them to be the one they reach
  for.

### Oblivion

* **A lock** (`ExtraLock`, its data: level +0, key +4 (`TESKey*`), flags +8 (1: locked)) whose key
  the player carries opens on the player's activation (doors: `TESObjectDOOR::Activate`;
  containers: `TESObjectCONT::Activate`): the lock
  is unlocked there and then (changed for the save), and a message box with one button,
  "Ok", says so ("You opened the door with <key>.", `sOpenWithKey`; "Opened the
  container with a key.", `sOpenedContainer`). Its button (`HandleClick`, id
  4: answer 1, with the box's offset of 1) runs the box's callback, which takes
  answer 1 as Ok and activates the door or container again: it opens (a load door takes the player
  through).
* **A key with a lock it doesn't fit** does nothing there; the game's own activation would open the
  lock's menu if the player has picks, so it isn't asked.

### Design

* **The key put in and turned**: a key drawn from the body (a new Keys slot, a ring on the belt
  just behind the left hip) with its tip put to a locked door or container (as a lockpick's is) goes
  in if it's that lock's key (a click in the hand); turned in it (the hand's twist about the key,
  60° either way, with a detent felt every 20°) it's the player's activation: the game unlocks it,
  and the key's turn is its message box's Ok (the box's own click), so the door or chest opens. The
  box's words are the game's corner notice instead. The wrong key bumps and stays out. Taken more
  than 6 cm from where it went in, it's out.
* **The key that fits first**: the Keys slot gives the key to the nearest locked thing it opens
  first (the slot's Strongest order, a key's strength being how near its lock is, in metres from
  10 m), then the rest by value.

Calls made:

* The turn is the yes: the box only says what the turn did, so it doesn't wait for a laser click.
* The Keys slot is new, behind the left hip, so a sword in the hip holster keeps the hip's middle.

## 10. Doomstones and runestones

### How VR games do it

* **A palm on the stone** is the ritual, as with the wayshrines (section 5): held a moment, felt
  building, the stone answers.

### Oblivion

* **Doomstones** (twenty: the thirteen of the birthsigns, `Doomstone01Apprentice` to
  `Doomstone01Warrior`, and the seven of the sky, the Jone, Aetherius, Jode, Sithian,
  Magnus, Shezarr and Dragon Stones, to `Doomstone08Dragon`) and **rune stones** (the
  Hestra, Sidri-Ashak and Reman Rune Stones) are activators with the models
  `Dungeons\FortRuins\Exterior\RuinStonesExterior\RuneStoneExt*.NIF`, each with its own script on
  the player's activation: a birthsign's Doomstone at night asks "Do you choose to steer by the
  stars of the Atronach?" (No / Yes: its Greater Power replaces the one the player has), by day "No
  stars shine upon the doomstone."; the sky's have their own (the Dragon Stone's power for the
  famous); a rune stone "The stone stirs under your hand. Do you seek its conjured
  weapon and armor?" (No / Yes, once a day: "Stones sleep. Stones rest this day.").

### Design

* **A palm on the stone**: an empty hand on a Doomstone or rune stone for 0.6 s is the player's
  activation of it (the wayshrines' touch): its own words and question, answered by the player with
  the laser.

Calls made:

* The stone's question isn't answered by the touch: a Doomstone's power replaces the one the player
  has, a choice that's the player's.

## More to do by hand

A survey of what else in Oblivion the player could do with their hands and body, each through the
game's own system. Ranked by how good it would feel in VR and how often a player meets it (best and
most often first). 1, 3 and 5 are built (sections 8 to 10); the rest are for the user to pick from. (Already passed on: taking things out of
containers and bodies by hand, eating and drinking at the mouth, shopping at the counter with gold.)

| # | By hand | Feel | How often | The game's own system |
|---|---|---|---|---|
| 1 | **Harvesting plants** (built: section 8): pinch a flower's head, pull a mushroom's cap, pluck a Nirnroot (it hums) | High: the most tactile thing in the wild | Constantly, outdoors and in caves (155 flora forms) | `TESFlora` (form type 0x1F): the player's activation harvests by the game's chance ("You successfully harvest %s.", its season, the plant marked harvested); the ingredient into the pack, or into the hand as its model |
| 2 | **Putting armour and clothes on and off**: a helmet held to the head, a cuirass to the chest, gauntlets pulled on, a ring onto a finger; a worn piece gripped and pulled off | High: dressing is the classic VR moment | Every haul of loot | `Actor::EquipItem` / `UnequipItem` with the piece's biped slots (its `TESBipedModelForm`); needs apparel in the hands (the holsters handing out armour and clothes as their ground models, also in its `TESBipedModelForm`), which also lets carried armour be repaired and enchanted in the hand |
| 3 | **Keys at the lock** (built: section 9): a key from the Keys slot put to its door or chest | Medium-high: small, satisfying, the lock already reads the tip | Often (houses, dungeons, quests) | The player's activation of a locked thing whose key they carry (`ExtraLock` +4, the key): it opens; the lockpicking's keyhole touch reused |
| 4 | **Sigil Stones on gear**: a Sigil Stone held to the weapon in the other hand, or to a worn piece | High: the reward of every gate, made physical | Once per gate | The `SigilStoneMenu` (type 0x418) with its item chosen through the `RepairMenu` in mode 6 (made for a moment, as recharging is) |
| 5 | **Doomstones and runestones** (built: section 10): a palm on the standing stone | Medium: a ritual, like the wayshrines | Common outdoors (13 Doomstones, 26 runestones) | Each stone's own script on activation (the birthsign power once a day, the runestones' lesser powers); the wayshrines' touch reused |
| 6 | **Traps by hand**: a tripwire cut with a blade or snagged, a pressure plate pressed with a palm, stepped over to avoid | High: tension, and a reason to look down | Often in caves, forts and ruins (153 trap activators) | The trigger activators' own scripts (`CTrigTripwire01`, `ARTrigPressurePlate01`...): their activation by the player sets the trap off; a blade's cut is a hit on the wire |
| 7 | **Persuasion by gesture**: admire (a hand on the heart), boast (a fist to the chest), joke (a laugh), coerce (a pointed finger) to the speaker | Medium: social VR, but gestures can misfire | Often with quests and merchants | The `PersuasionMenu`'s four quarters clicked as its wheel turns (its own disposition and timing), the wheel floating at the speaker |
| 8 | **Ladders climbed**: rung over rung, the pull taking the player up | Medium-high (VR players love climbing) | Rarely (16 ladder doors: the palace, sewers) | Ladders are load doors: their activation at the top of the climb |
| 9 | **The Altar of Spellmaking**: a palm on it opens spellmaking beside it | Low-medium (the work is the menu) | Rarely (the Arcane University) | `ICSpellmakingingStation`'s script (`ShowSpellmaking`); the enchanting altar's machinery reused |
| 10 | **Handing a requested thing to someone**: the letter, the ring, the head of the bandit chief, held out to the one who asked | High when it fits | Often in quests, but each quest is its own | The quest's dialogue topic that takes the thing (`RemoveItem` in its result script): needs each quest's topic found; risky |
| 11 | **Bells, chimes and gongs struck**: the Chime of the Wellspring, chimes in the Shivering Isles | Medium | Rarely | Their activators' scripts (20 chimes) |
| 12 | **Training at the Arena's dummies and the archery butts** | Low-medium | Rarely | Hits on their activators (the game's skill training is its trainers') |

Notes for building them:

* 1, 3, 5 and 9 are small: activation by a gesture at the thing, as the shrines and wells are.
* 2 is the biggest and unlocks the most (it changes the holsters: apparel in the hands).
* 4 reuses recharging's machinery (the game's own item list, made for a moment).
* 6 needs a hit from a blade on a trigger that isn't an actor (the combat code's hits are on actors).
