# Spells

Spellcasting remade from the ground up, in the framework and in Oblivion, with Skyrim VR and the
Skyrim VR mods its players swear by as the reference (the user's words: redo it, research how
Skyrim VR does it and the mods that make it feel good). What Skyrim VR does, what each loved mod
adds and why players love or hate it, what Oblivion's own magic is (its engine's facts are in
[spells-engine-notes.md](spells-engine-notes.md)), and the design built from it all, with its
reasons. Everything is paraphrased from the sources linked; nothing is copied.

## Skyrim VR's own magic

* **A spell in each hand.** Each hand holds its own spell, put there from the favourites or magic
  menu by pulling the trigger of the hand that should have it. Reviewers' favourite thing about VR
  magic is exactly this, a different spell in each hand
  ([Bethesda](https://bethesda.net/en/article/4NYGS8KXEIEyUssayc6iWy/reimagining-skyrim-for-vr)).
  The menus pause the game, and VR has no hotkeys at all, which players hate
  ([Steam](https://steamcommunity.com/app/611670/discussions/0/1696043806562466922)).
* **Hold to charge, let go to fire.** Fire-and-forget spells charge while the trigger is held, over
  the spell's own charge time (from its record), and go when it's let go; let go too soon and the
  spell fails ([UESP](https://en.uesp.net/wiki/Skyrim:Magic_Overview)). The hand shows the spell's
  art while it's equipped; there's a charge sound and a separate ready sound. There are no
  spellcasting haptics in the game itself, so players can't feel when it's ready and let go too
  early or too late ([Magic Improvements posts](https://www.nexusmods.com/skyrimspecialedition/mods/55751?tab=posts)).
* **Concentration** spells stream while the trigger is held, following the hand every frame, and
  **dual casting** (the same spell in both hands, the hands together, both triggers) gives a
  stronger cast with the school's perk. Players find dual casting unreliable and see no difference
  in power.
* **Aiming** is from the hand's magic node along the controller turned by a fixed pitch
  (`fMagicRotationPitch`, 60 degrees in the shipped INI), with no yaw or roll, so it feels like
  pointing a finger gun; turning the palm towards a target, as people naturally do, sends spells
  into the air ([r/skyrimvr](https://www.reddit.com/r/skyrimvr/comments/8ax29j/)). There's no aim
  line, only a 3D crosshair per hand that many turn off and then struggle without.
* **Self spells** need no aim; **runes** go where the hand points; **staffs** cast from a point
  along the staff; **shouts and powers** share the right grip, and accidental shouts are a common
  complaint.
* **What players complain about**: point-and-hold feels dull; the finger-gun pitch; swapping spells
  pauses the game; no felt "ready"; dual casting; the half second of animation before a stream or a
  ward starts; telekinesis throwing things sideways.

## The mods players rate

The nine Wabbajack VR modlists checked (FUS, Tahrovin, Tahrovin-Grit, Spirit of Grit, Auriel's
Dream, Yggdrasil, Stormcrown, Librum, Panda's Sovngarde) and Mad God's Overhaul all ship the same
core: VRIK, Spell Wheel VR, Magic Improvements and Haptic Skyrim VR, most with Dual Casting Fix and
3D Casting FX, many with Spellsiphon and Conduit
([wabbajack reports](https://github.com/wabbajack-tools/mod-lists/tree/master/reports)).

| Mod | What it does | Why players love or hate it |
|---|---|---|
| **Spell Wheel VR** (Shizof, [Nexus 47630](https://www.nexusmods.com/skyrimspecialedition/mods/47630)) | A wheel of orbs per hand (favourites by default, up to 60, in pages), opened by a button or a combination; the hand goes into an orb and letting go equips it to that hand; the middle unequips; time can be slowed while it's open; since 1.5, glyphs drawn in the air cast heals or armour without equipping them. | In every list. Swapping without a menu or a pause is the thing players wanted most; the slowed time makes mid-fight swaps feel deliberate. The common gripe is finding a comfortable button to open it. |
| **VRIK** (prog, [Nexus 23416](https://www.nexusmods.com/skyrimspecialedition/mods/23416)) | Gestures (press, move the hand a way, let go: up to 26 per hand) to quick-equip or cast a spell or shout; casting by opening the hand; holsters that put back the spell a hand held when a weapon is put away. | In every list. Casting a heal or armour with a flick without changing what's in the hands is loved; open-hand casting is criticised for no feedback when it goes off. |
| **Magic Improvements** (FlyingParticle, [Nexus 55751](https://www.nexusmods.com/skyrimspecialedition/mods/55751)) | Dual casting that merges between the hands and grows as they part; the hand's art scaled by the magicka left (0.35 empty to 1 full); the aim smoothed over the last 10, 15, 20, 25 or 40 frames by the spell's level; yaw and roll for the aim, which is how **palm casting** is set up (yaw about 55 to 70 degrees). | In every list. FUS and Tahrovin ship palm casting by default because it simply feels better; a minority say palms are hard to aim at range without a crosshair. Heavier smoothing on harder spells reads as weight. |
| **Haptic Skyrim VR** (Shizof, [Nexus 20364](https://www.nexusmods.com/skyrimspecialedition/mods/20364)) | Spell, bow and melee haptics. | In every list, but it buzzes through the whole charge with nothing distinct at "ready": the most repeated haptic complaint. |
| **ISPVR, Immersive Spellcasting VR** (tom300z, [Nexus 164183](https://www.nexusmods.com/skyrimspecialedition/mods/164183), [source](https://github.com/tom300z/ImmersiveSpellcastingVR)) | Close the hand to charge, open it to cast; the transition animations cut so spells answer at once; casting while jumping. Its charge haptics, from its source: a pulse every 100 ms at the start down to every 20 ms when full, as strong as the charge to the sixth (silent, then a sharp ramp), a faint tick every 50 ms held ready, ten full pulses 10 ms apart on release. | Its release thread was warmly received: the felt ramp and ready spike answer Haptic Skyrim VR's complaint. Against it: without straps the controller can slip from an open hand. |
| **Spell Auto-Aim VR** (Shizof, [Nexus 119689](https://www.nexusmods.com/skyrimspecialedition/mods/119689)) | A gentle cone onto a target: missiles 10 degrees up or down, 15 across, beams 5 and 10, to 2000 units; hostile spells only onto enemies. | Split: at low values players call it not cheaty, like willing the magic towards them; it helps shaky hands; purists find it takes the skill out. |
| **Spellsiphon, Conduit** (Arctal, [Nexus 26627](https://www.nexusmods.com/skyrimspecialedition/mods/26627), [58023](https://www.nexusmods.com/skyrimspecialedition/mods/58023)) | A new kit of spells used by hand without menus; grabbing your weapon with the spell hand infuses it. | Loved for grabbing to conduct being so natural in VR; it's a new spell system, not Skyrim's. |
| **Dragonborn Speaks Naturally** ([Nexus 16514](https://www.nexusmods.com/skyrimspecialedition/mods/16514)) | Equip favourites by voice ("equip fireball left"). | Immersive when it works; false triggers from noise. |
| **SpellBender VR**, **Weapon Throw VR** | Steering a projectile after release by the controller's movement; throwing weapons where the throw's speed is the projectile's. | Newer; no mod throws a spell with the hand's own speed. |

## What players say makes VR magic feel good

1. **A felt "ready"**, at the moment it matters, rather than a constant buzz: silence, a rising
   buzz, a spike when full, a faint tick while held, a burst on release; and a failure felt too
   (PLANCK plays the failed sound with a long pulse).
2. **No lag between the hand and the spell**: no flat-game animations in between.
3. **Fast swaps without pausing**, and getting back what the hand held.
4. **Aim from the palm**, with a little help at range, and weight on the big spells.
5. **Dual casting you can see**, in Skyrim's case; and comfort: inputs off the grips.

## Oblivion's own magic

The facts, with addresses, are in [spells-engine-notes.md](spells-engine-notes.md). In short:

* **One selected spell** at a time (the player's own), picked in the magic menu or by a quick
  key (eight global lists), and cast with the Cast control from the left hand; spells, powers once
  a day and lesser powers alike. The magic menu's list is powers, lesser powers, then spells, by
  name; each row's icon is its costliest effect's.
* **No charge, no concentration, no dual casting.** A cast plays the cast animation (self, touch or
  target, by the spell's farthest range); the spell goes at its **Hit** key (0.37 s self, 0.27 s
  touch, 0.4 s target), and the next waits for the animation to end (about 0.93, 0.6 and 0.5 s
  after the release).
* **The check** (magicka, silence, mastery, a power used today, a bound item, a ranged spell under
  water) happens as the animation starts, with the game's message and the school's failure sound;
  the release casts through the **checked cast**: the check again, the effects, the skill
  use as they land, the projectile (carrying the casting light), the cost, the fatigue, a power's
  day, invisibility dispelled.
* **Touch** goes into the nearest living actor in the caster's facing cone within the hand's reach
  (about 77 units), else what the crosshair is on (a door, a container). **Target** spells are a
  projectile from the magic node along the caster's facing. **Self** goes on the caster.
* **The art**: the main effect's model playing its `SpecialIdle_Cast` in the hands, its casting
  light fading in, its casting sound with the animation.

## The design

### What it takes from where, and why

| From | Taken | Why |
|---|---|---|
| Skyrim VR | **A spell in each hand**; the trigger charges, letting go casts; letting go early fails. | The heart of Skyrim VR's magic, the thing its reviewers loved. |
| Skyrim VR's menus | A spell picked in the game's own magic menu goes into a hand. | The game's own selection stays a way in. |
| Spell Wheel VR | **The spell wheel**, on a button, picking per hand without a menu or a pause; the middle empties the hand; favourites first; pages. | The most installed magic mod there is; swapping without pausing is what players asked for. |
| Spell Wheel VR, made Oblivion's | The wheel is **pointed at** (each hand's pointer, a slice of the ring its way from the middle), drawn from the **game's own art** (its icons, its font). | Oblivion's menus here are worked with a pointer and drawn from its own interface. A slice is quicker to hit than an orb. |
| Skyrim VR's menu rule | On the wheel, **the trigger of the hand that should have it**. | The same rule players already know; one wheel serves both hands. |
| ISPVR | **The charge's haptics, exactly**: 100 ms to 20 ms, the sixth power, a faint tick held ready, a full burst on release. | It answers the most repeated complaint (no felt ready). |
| PLANCK | A **failure felt**: a long dull pulse, with the game's own message and failure sound. | Players asked for it; Oblivion has its own words and sounds for it. |
| Skyrim VR, made right | **The aim is the way the hand casts**, arm and wrist both: the controller's own pointing ray turned 59 degrees down and 1 across (as Skyrim VR's `fMagicRotationPitch` turns its own), then 74 towards the palm, the bolt starting 39 cm out: mostly out of the palm. **The Spells tab** in RustVR's menu sets all four, against a line from the palm to where a spell would go. | The user's first playtest: palm casting off the grip came out well off where the controller pointed. The second: the controller's own pointing ray (OpenXR's aim pose, the laser's) went high, since a hand held out to cast has its wrist cocked up. The third: along the arm (shoulder through palm) the wrist didn't steer, and it should. The fourth: the user set the aim themselves in the Spells tab, casting at the zombies, and those are the defaults. |
| Magic Improvements | **Palm casting** (`palm_tilt`, 74 by default: the user's own); **aim smoothed by the spell's level**, but short (40 to 120 ms, against its 10 to 40 frames); the held art **smaller as magicka runs down**. | Some lists ship palm casting, and others find it hard to aim at range; its long smoothing trails behind a quick point. The short one only takes out the trigger's jolt. Magicka you can see in the hand. |
| Spell Auto-Aim VR | A **gentle cone** (10 degrees up or down, 15 across, 28 m), foes first for a harmful spell. | Called not cheaty at these values; it helps at range with palm casting. |
| VRIK | A hand that takes a weapon **keeps its spell** and has it back when it lets go. | Getting back what you held. |
| Curated lists | **No light from a spell only held**: the casting light comes with the charge. | Up close a palm's light dazzles; the lists drop hand-effect lights. |
| ISPVR | **No animation in between**: the hand is the cast animation. | Responsiveness. |
| Oblivion | The **charge time is the cast animation's time to its Hit**, the wait after a cast the rest of it; every check, cost, message, sound, skill use, projectile and effect is the **game's checked cast**; touch is the game's touch pick (the creature the hand touches given to it); picks are the game's own selection. | Oblivion stays Oblivion: nothing reimplemented, no effect dropped. |

### Left out, and why

* **Dual casting** and **concentration**: Oblivion has neither, and adding them would change its
  spells. Each hand casts its own, the game's way.
* **Slowed time while the wheel is open**: Oblivion has no slowed time to call; inventing one would
  change its combat. The wheel is quick instead.
* **Glyphs, gestures, voice** (Spell Wheel VR 1.5, VRIK, MageVR, Dragonborn Speaks Naturally): loved
  by some, misread by others; a follow-up once the core is played.
* **Steering a bolt after release, throwing a spell** (SpellBender): Oblivion's
  projectile is the game's, launched as it launches it; the hand only aims it.
* **An aim marker**: not in Skyrim VR's best-loved setups, and a marker would be the runtime's art,
  not the game's.

### Every effect, in VR

Every magic effect in `Oblivion.esm` (its `MGEF` records: their codes, flags and schools), and what
happens when a hand casts it. The game runs every effect itself; the question is whether an effect
touches something VR has taken over (the player's body, hands, weapons, view or controls) and so
needs the adapter to carry it across.

| Effects | What the game does | In VR |
|---|---|---|
| Damage (fire, frost, shock, health, magicka, fatigue, attributes), Drain, Absorb (health, magicka, fatigue, attributes, skills), Weakness, Disintegrate Armour or Weapon, Sun Damage, Stunted Magicka, Vampirism | On the target, or the caster for an absorb: its numbers, hit shaders, sounds | The game's, nothing to carry: a target spell lands where the palm aimed it; on the player, its shader is drawn over the body |
| Restore, Fortify (attributes, skills, health, magicka, fatigue, Fortify Magicka Multiplier), Resist, Cure, Dispel, Reflect Damage, Reflect Spell, Spell Absorption | On whoever it's cast on | The game's. Fortify Magicka raises the most the hand shows (the table's maximum includes the game's max modifier). Dispel takes a bound weapon away with the rest, and it leaves the hand |
| Shield, Fire, Frost and Shock Shield | The player's armour rating and resistances, and the shield's shader on the player | The game's, with its shader over the body and hands (the overlays) |
| Calm, Frenzy, Charm, Rally, Demoralize, Command Creature or Humanoid, Paralyze, Silence, Soul Trap, Turn Undead, Reanimate | On the target's AI or state | The game's: aimed by the palm, or a touch put into what the hand touches |
| Burden, Feather, Water Breathing | Weight carried, breath | The game's: its walking and the swimming feature's breath meter follow it |
| Water Walking | The player walks on water | The swimming feature reads it from the game (its `WaterState`) |
| Detect Life | The game's shader on creatures, through walls | The game draws it in its own picture of each eye |
| Night-Eye | The game's night-eye shader on its picture | The game's picture shows it; the body's lighting takes the game's ambient, Night-Eye's included |
| Light (self) | A light on the player | The game's light lights the world and the runtime's body; Light (target, Illuminate) is a bolt whose light stays where it lands |
| Open | Unlocks the door or container a bolt hits | The game's; the door's latch, tugged, then opens it (`world.rs`) |
| Summon (every `Z...`, the Hungers, the Shambles, the Atronachs, Golden Saint, Dark Seducer) | The creature appears ahead of the caster | Ahead of where the head looks: the player is faced along the cast while the game makes it. The game's rule against summoning in the water is kept |
| **Bound weapons** (Bound Sword, Axe, Mace, Dagger, Bow; the Order's weapons, the Priest Dagger, the Staff of Sheogorath) | The game equips a bound weapon | **Made in the hand that cast it**, by its grip, not at the hip: the game equips it, and the adapter gives it to the runtime in that hand (`ObjectDesc::into_hand`); the grip's first squeeze takes hold of it. **Kept**: let go anywhere but a slot, it goes back to the hip (a bow to the back), as the game never drops a bound item (`OBJF_KEPT`; any weapon or shield a bound effect names, its `EffectSetting` data). When it ends (its time, a dispel) the game takes it, and it's gone from the hand or the hip |
| **Bound armour** (Boots, Cuirass, Gauntlets, Greaves, Helmet; the Mythic Dawn armour and helm) | The game equips bound armour | On the body as the game equips it (the body follows what's worn). **The Bound Shield** is made in the hand that cast it, like a bound weapon |
| Telekinesis | On the caster: its Telekinesis actor value (60) is the spell's magnitude in feet while it lasts; Activate at a distance takes a thing, which floats where the player looks, and attacking pushes it away | **By hand**: while the game's Telekinesis is on the player, the hands' telekinesis takes the gravity gloves' place (the framework's, `rig/telekinesis.rs`): an open hand points at a thing within the spell's own reach (its magnitude, in metres), the grip catches it and it floats where the hand points, the trigger pulls it in, a flick throws it. Things only, as the game's own moves no one (`RULES_TK_THINGS`). The adapter says so through the game's melee rules (`RULES_SAYS_TELEKINESIS`, `MeleeRules::tk_reach`). The game's own (Activate at a distance, by a quick trigger tap where the head looks) is still there too |
| Chameleon, Invisibility (on the player) | The player's own model is drawn see-through, with its refraction | **Not carried yet**: the game doesn't draw the player's body in VR, and the runtime draws it solid. A follow-up: the body and hands drawn with the game's look for them |
| Script Effects (quests', mods') | Whatever their script does, with the look of the effect they name | The game's; their art in the palm is the effect they look like |
| Touch spells with an area that "explode with no target" (spell flag 0x80) | The burst goes off at the caster's magic node | At the palm: the magic node is moved to the palm for the instant of the cast |
| Powers, lesser powers | Cast like spells; a power once a day | The same as spells; a second power the same day is refused with the game's words |

Not spells but nearby: **scrolls** are read at the eyes (the holsters feature) and cast the game's
own animated way, from the hidden first-person hands along the head's look; casting them from the
hand like a spell is a follow-up. **The player paralysed** by someone else's spell can't move in
the game, but their hands still move in VR.

### The framework (any game with magic)

* **`crates/runtime/src/magic.rs`**: each hand's spell (either hand; at first the off hand has the
  game's selected spell and the main hand none, so its trigger stays the game's Activate); the
  trigger's charge over the spell's casting time, with ISPVR's haptics; let go when charged, a cast
  ([`CE_CAST`]): from the palm the way the hand casts, mostly out of the palm by default (smoothed a
  little by level,
  onto a creature in the cone),
  a touch spell into the first creature the charged hand touches ([`CEF_TOUCHED`]) or along the aim,
  a self spell anywhere; a quick tap with nothing charged is the game's Activate; a spell the game
  won't cast now doesn't charge ([`CE_CAST_REFUSED`]); after a cast, the game's wait. Staffs
  ([`OBJF_CASTS`]) charge at their tip and cast along themselves. Each hand's state goes to the game
  every frame ([`Casters`]) for its own art (or the runtime's glow for a game with none).
* **`crates/runtime/src/spell_wheel.rs`**: the wheel (the main hand's B), its pointers, its paging,
  its picks ([`CE_SELECT`]), painted from the game's art (each spell's icon, the game's font) with
  the game cards' painter.
* **The protocol** (v14 kept): a spell's mastery level, its charge and recovery times and its icon;
  the table's recovery, the player's own picks in the game's interface, the wheel's font and art;
  `CE_SELECT`, `CE_CAST_REFUSED`, `CEF_TOUCHED`.
* **Settings** (`[magic]`, the menu's Spells tab): `aim_down` (59: degrees below the controller's
  ray), `aim_out` (-1: degrees out to the hand's side), `palm_tilt` (74: degrees towards the palm),
  `launch_ahead` (0.39: metres ahead of the palm a target spell starts), `show_aim` (a line from each
  palm holding a spell to where it would go, for setting those), `aim_assist`, `aim_smoothing`,
  `charge_haptics`.
* **The range** (`crates/range/src/magic.rs`): six spells (a fire bolt, a heal, a frost bolt of a
  higher level, a shocking touch, a power, one too costly), magicka spent and coming back, bolts
  that fly and strike the dummies, bursts for a touch and a heal.

### Oblivion

* **`crates/oblivion/src/spells.rs`**: the table in the magic menu's order with everything the game
  says of each spell (its farthest range, the player's cost, the game's own check, its mastery,
  school, icon, light, whether it harms, its quick key), the charge and recovery times from its cast
  animations, the magicka as the HUD has it, the menus' font, the player's own picks; the cast
  through the game's checked cast with the spell as the caster's item, the projectile from the palm
  along the aim carrying its casting light, a touch into the creature the hand touched (the touch
  pick hooked), the summoning-in-water rule; the wheel's pick through the game's own selection and
  the magic menu's sound; a refused pull answered with the game's message and the school's failure
  sound; the staff's own cast.
* **`crates/oblivion/src/magic.rs`**: the art, in the palm once the trigger has charged it and for
  30 s after (smaller with the magicka), growing with its casting light and sound as it charges. A
  hand only holding a spell shows nothing: turned about, its art flickered round the hand's edges.
* **The art over the hand** (`d3d9.rs`, `render.rs`; the framework's `TGT_ART`): drawn into the
  game's picture, the art was hidden by the runtime's hand drawn over it, even in front of the
  palm. The game's renderer draws each geometry whole in one call, so while an eye's world is drawn
  the art's own geometries go into a picture of their own for that eye (in the world's depth, the
  alpha as coverage, as the HUD's is), which the runtime lays over the hand where it's nearer: palm
  up the art shows, palm down the hand hides it. With the game's HDR on it stays in the picture
  (the HDR tone curve comes after the world, and the art apart would miss it).
* **Bound weapons and shields** (`spells.rs`, `weapons.rs`): made in the hand that cast them
  (`ObjectDesc::into_hand`, which the rig honours as the thing arrives).
* **Telekinesis** (`spells.rs`): the game's melee rules say the hands' telekinesis is on while its
  Telekinesis actor value is, as far as it reaches, things only.

### Controls

| VR | Oblivion |
|---|---|
| The main hand's B held | The spell wheel, ahead at chest height |
| A hand's pointer on the wheel, its trigger | That spell into that hand (the middle: empty that hand); the arrows at its foot turn the page |
| B let go with the main hand's pointer on a spell | That spell into the main hand |
| A pick in the game's magic menu or a quick key | That spell into the off hand |
| Trigger held, a hand with a spell | Charge it (the game's casting time); felt growing, a spike when ready |
| Let go, charged | Cast: target spells from the palm where the hand points (arm and wrist), touch spells along it, self spells anywhere |
| A charged touch spell's hand on a creature | Cast into it at once |
| Trigger tapped, nothing charged | The game's Activate |
| Trigger, holding a staff | Charge at its tip; let go to cast along it |
