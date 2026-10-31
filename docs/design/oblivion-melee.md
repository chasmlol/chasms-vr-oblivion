# The framework's melee in Oblivion: the design

The framework's melee is physical (its `docs/MELEE.md`: weight in the hands,
grips and leverage, hits judged by their speed, angle and armour, stabs that stick,
parrying, grappling, punches, throwing, climbing, stamina), proved in its test range. This is how
Oblivion plays it, replacing the adapter's older melee (its own damagers measured on each weapon,
judged by the runtime's older, simpler table, which is gone from the runtime too).

The rule throughout: **the runtime decides whether and how hard a blow lands; the game does the rest.**
Oblivion's damage, skills, sneak attacks, enchantments, its block, sounds, OnHit scripts, stagger
and knockdowns all stay its own, through its own hit routine. Its creatures stay its own actors
with its own AI: the runtime's fighters (their stances, feints, brains) are the range's only.

## Who does what

| | The runtime | Oblivion (the game's) |
|---|---|---|
| The weapon in the hand | its holding: the joint, weight, handles, grips, two hands, sliding, throwing | its weapon (`weapons.rs`): model, weight, kind |
| Whether a blow lands | its judge: the damager's angles, the material's least speed | what each part wears (`combat::materials`) |
| How hard | the swing's strength, its make of the blow (a flat, a throw), times the body part | its own hit's damage (skill, weapon, armour rating, sneak, block), scaled by that |
| A hard blow | push level 1 or more (a cut at 8 m/s, a mace at 7.5) | its power attack, in the way the blow went (its bonus by skill, its directional perks) |
| A blade gone in deep | its deep wounds (a throat x32, a head's point x32 against the trunk's x2) | one more of its hits, for what's past the trunk's |
| Knockouts, disarms | its tables (knockout speeds, disarm push levels) | its own knockout (Fatigue below nothing), its own disarm (its power attack perk's steps) |
| Blood, cuts | (its own blood and dismemberment visuals are the range's) | its gore: the game's blood art, cut at the joint the blade crossed on a killing blow (`gore.rs`) |
| Blocks and parries | the weapon meeting the attacker's blow | its block (the Block control held through the blow) |
| Grappling, choking | the hands' grabs | its ragdoll, its Fatigue knockout, and the runtime's choke damage (`grabs.rs`) |
| Stamina | what the swings spend (its swing cost) | its Fatigue (`melee.rs`) |
| Climbing | the hands on the world (its player joint) | its `Actor::Move`, the arms' step (`movement.rs`) |

## The weapons (`weapons.rs`)

Each equipped weapon is a `MeleeWeapon` measured on the game's own model with the runtime's own handles
and damagers for its kind (`rustvr_melee::weapons`, the same ones the range's weapons use):

* A one-handed blade is the range's arming sword (an edge, a point, a heavy pommel): its grip
  a light handle that slides on over the guard onto the blade (half-swording) and back, its pommel
  a second hand's. With a blade no longer than 35 cm (guard to point) it's the range's dagger instead
  (its slash and stab): held either way, thrown point first. A two-handed blade is
  the two-handed slash on a heavy grip.
* A blunt weapon whose head reaches out to one side is the range's axe (its edge along the head's far
  edge); any other's head is the range's mace (one- or two-handed). Its haft slides.
* A staff is the range's staff: held anywhere along it, both ends bludgeoning.
* A shield is the range's round shield (a strong blow over its face), held by its strap.
* Its balance is where its model's metal is (each centimetre's slice weighed by its breadth and
  depth), so a mace's or an axe's weight is out at its head. Its mass is the game's weight x0.06 kg
  (an iron longsword's 22 is 1.3 kg, the range's arming sword 1.25), up to its maul's 8.
* Its tier is the common weapons' (2) for every weapon: the game's own damage and skill say how
  good a weapon is, through its hit.

## Hits (`combat.rs`)

* **Whether**: the runtime's judge, against what each part of the creature wears (`materials`): the game's
  heavy armour is plate, its light armour leather (the game's own two kinds), bare or
  clothed flesh. The cuirass covers the trunk and the upper arms, the gauntlets the forearms and
  hands, the greaves the legs, the boots the feet, the helmet the head. A sword's edge glances off
  plate under 8 m/s; a mace still lands.
* **How hard** (`how_hard`): how hard it was swung (the runtime's strength: the speed past the least
  for its kind, up to a full swing, `rustvr_melee::full_speed`: a cut at 7 m/s, a stab at 5, a blunt
  blow at 8, a fist at 6), times what the runtime makes of this blow against the same blade's clean one at
  that speed (a bad angle's blunt blow a fraction; thrown, its thrown multiplier), with the armour's
  multiplier taken back out (the game's armour rating does its own), times the body part's (a head
  x3, the throat and the trunk x1, a limb x0.5, a hand or a foot x0.25). Not the runtime's own curve against
  speed: it's nearly flat (a sword's 3 m/s tap does 85% of a 7 m/s cut, its damage being mostly
  the blade going in), and on the game's damage for a whole attack, taps would land whole attacks
  several times a second.
  The game's hit runs (`game_hit`) with its damage formula's own multiplier (`fDamageWeaponMult`,
  read only by that formula; a fist's `fHandHealthMin` and `Max`) times that, so what follows from
  the damage follows too: the stagger's chance, the knockback, wear, the death push.
* **Power attacks**: a blow that shoves the body by the push tables (push level 1: the old melee's
  speeds too) is the game's power attack. The game takes a power attack's bonus and perks from the
  attack's anim group (in the attacker's `ActorAnimData`: 22 standing, 23 forward,
  24 back, 25 left, 26 right), which VR never plays, so the old melee's power attacks got none; the
  group is set from the way the blow went while the hit runs (`power_group`), so its bonus by the
  skill's mastery and its perks (a side one's disarm, a back one's knockdown, a forward one's
  paralysis) are the game's own.
* **Deep wounds** (`wound_beyond`): the runtime's wound of a blade gone in (a point's deep damage, a cut's,
  a point out the far side) past what the same wound does in the trunk is one more hit of that size:
  a cut throat or a point deep in a head kills; one in the trunk is the blow's own.
* **Punches** are the runtime's punch (its curve and table) on the game's hand-to-hand hit: the weapon out
  of the process's hand (its `MiddleHighProcess`) while it runs, so even with a sword in the other
  hand a fist is the game's fist. A shield's bash is one too (the game has no bash).
* **Knockouts and disarms**: a blunt blow to the head past its row's knockout speed is the game's
  own knockout (Fatigue taken below nothing); a hand struck at the disarming push level (2; 3 in
  heavy gauntlets), or an arm stabbed through, is the game's own disarm, its power attack perk's
  steps (the drawn weapon, dropped by the actor's own drop at its hand, owned, its attack
  cancelled, its combat sent for a weapon).

## The calls

| What | The range's | Oblivion's | Why |
|---|---|---|---|
| Damage | its damage on 50 health | the game's own hit, scaled by the range's against a full blow | the game is the source of truth for what a hit does |
| Armour | the material multiplies damage and gates it | the material gates (and decides going in); the game's armour rating reduces | counting both would count armour twice |
| A hard blow's stagger | its push table and brain | the game's power attack at push level 1, its stagger by the scaled damage | push level 2 never comes for a sword's cut on flesh, and the game's power attacks are its swords' too |
| Deep wounds | added damage | one more game hit for what's past the trunk's | the trunk's is the blow's own; the deadly ones stay deadly |
| Knockout | a 6 s stun | the game's own, Fatigue below nothing | the game's own knockout |
| Disarm | the brain lets go | the game's own disarm (its perk's steps), on the range's push levels | the game's AI goes for a weapon as it does |
| Choking | 5 a second of 50 | a tenth of its health a second squeezed fully, beside the Fatigue drain | scaled to the creature's own health |
| Stamina | none (ours: the runtime's) | the game's Fatigue, spent by the swings | the user chose it; the game keeps one |
| Tier | per item 0..5 | 2 for every weapon | the game's damage and skill already say how good a weapon is |
| Mass | per item | the game's weight x0.06 kg | the game's weight, in kilograms |
| Throwing | its own | the same, its hit the game's with that weapon | the user chose it |
| Climbing | its own | the same, the game moving the player by the arms' step | the user chose it |
| Slowed time, breakables, kicks, telekinesis | its own | off | the user's choice; the game's buttons come first; the gloves stay |
| The creatures' weapons | solid, blocked and wrenched away | their blows are the game's; blocks by the attack's line | not yet: the game's weapons aren't the runtime's objects |

## Engine facts

The facts this uses:

* The hit routine (the power flag, an arrow or none, the target) keeps
  every number on its own stack. Its weapon damage formula is the only reader of
  `fDamageWeaponMult`; a fist's reads `fHandHealthMin` and `Max`
  instead. Health and Fatigue come off through the actor's own `ApplyDamage`.
* `TESObjectARMO`: its slots, whether it's heavy, its armour rating (x100).
* `bhkCharacterController`: its velocity (Havok units: the game's over 7), its fall time
  and its gravity scale (every state adds gravity times it; the game swaps it itself for
  a galloping horse).

## Left alone

* **Magic and imbuing**: the spells chat (`<worktrees>\spells`) is rebuilding spellcasting from scratch;
  a spell filling a blade isn't wired into Oblivion here (the runtime may still report one).
* **Sneaking**: the stealth chat is reworking the physical crouch. Sneak attacks stay the game's own
  through its hit routine.

## Not done

* The creatures' weapons as the runtime's own solid things (`ActorItems`): blocking their real
  blades, taking hold of one and wrenching it away. Their blows stay the game's.
* Hit reactions by force beyond the power attack's stagger.
