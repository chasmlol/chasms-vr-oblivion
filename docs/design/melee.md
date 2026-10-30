# Melee by the swing: research and design

What the best VR melee games do and what their players say about it, how a physical melee
does it, what Oblivion's engine does with a melee hit, and the design built from all of that
(`docs/OVERNIGHT.md`, section 11). Inspiration only: nothing here is copied from another game's
code. The numbers below are our own tuning; Oblivion's are the game's own, from its settings
and its own data (`Oblivion.esm`, the meshes and animations in
`Oblivion - Meshes.bsa`).

## What players love and hate

**Physics-driven melee.** Loved: every weapon has its own heft, a light tap does nothing and a hard
thrust goes in and sticks until it's pulled out; a claymore can't be swung without committing to
the swing; it's a physics sandbox you improvise in. Hated: the lag (the tip drags behind the hand,
and some read the lag as input lag, not weight), blades glancing off for no visible reason, arms
stretched out on doors and ledges, AI that hugs you and barely reacts to parries, parries that are
"too easy to get round". Its weight comes from physics alone: the hand drives the weapon through a
force-limited spring, so a heavy weapon simply can't be flicked about.

**Boneworks / Bonelab.** Loved: if something looks heavy, it is (the sledgehammer can't be swung in
one hand); a body under the hands makes heaviness believable. Hated: wobbly heavy objects and
stretched arms, melee "imprecise at normal swing speed", nausea from the body being shoved about.

**Hellsplit: Arena.** Loved: guard-breaking (hit the shield and the enemy steps back, head
exposed). Hated: floaty weapons, two-handed grips that snap to fixed points on the haft and a front
hand that does all the work (players want free sliding, the front hand a pivot), oversized
hitboxes, weapons that stick.

**Battle Talent.** Loved: muscle memory; a full-swing parry knocks a skeleton apart. Hated:
"swinging through water", weight in the wrong place, weapons binding, lag. Its developer fought
jitter-waggle by adding drag to every weapon.

**Until You Fall** (no physics, the benchmark for feel). Every interaction feeds the sense of
power; hits and blocks follow the whole gesture, broad swings do more; parries chip a guard meter
and the guard break opens big slices; heavy weapons get a delay that reads as weight.

**Skyrim VR** (vanilla). Hated: every flick is a full attack with no delay, so wiggling the wrist
kills anything; normal attacks cost no stamina; a hit is a hit-check, not a blade meeting flesh.
The community's fix is an INI gate (`fMeleeLinearVelocityThreshold` 6). Loved: holding a shield up
and bashing.

**PLANCK and HIGGS** (Skyrim VR mods, open source). The closest match to this project: physical
hits mapped onto a Bethesda engine's own attack, power attack and stagger. A hit needs the contact
point's own speed (5 m/s for a swing, 2 for a stab, 2.5 for a punch, including the blade's spin)
*and* the hand moving at 1 m/s on average (a still hand spinning a blade doesn't count). A swing is
armed by hand speed and fires at its peak, charging stamina; holding the trigger makes it a power
attack, its direction from the swing. One contact is one hit (a cooldown per target); a weapon
fades while it's passing through after a hit; HIGGS turns weapon collision off for 0.5 s after a
hit so a blade follows through instead of binding, and lets the off hand take a held weapon
anywhere along it (shift and rotation blended 50/50).

**Swordsman VR.** Loved: weight modes, and a slow-motion beat on a successful parry. Hated:
floaty weapons on Quest, weight too much for weaker players.

**AAA.** Asgard's Wrath: tuned "so players can't wiggle swords"; a static guard always takes a
hard hit, only a parry staggers; enemies telegraph when they're open; dismemberment only on
follow-through. Behemoth / Saints & Sinners: wild smashing drains stamina; a parry is the weapon
held into the enemy's strike, and the enemy reels; stuck blades yanked out with force. Creed: Rise
to Glory: a phantom body that always tracks the player and a physical one bound by collisions,
desynced on staggers; "virtual stamina" slows flailing arms. Broken Edge: names the one-way physics
problem (a game can't push the player) and pushes the weapons apart instead. Arkham Shadow: keep
the threats in front.

**What they agree on** (the rules this design keeps):

1. Judge a hit by the blade's real motion at the contact (its spin included), and require the
   arm, not just the wrist: tiny flicks are weak, not full hits.
2. One contact, one hit; fire the attack at the swing's peak, not as it starts.
3. Broad committed swings hit harder; edges cut edge-first and points stab point-first.
4. Weight is felt but never feels like input lag: light weapons nearly 1:1, heavy ones lag with
   their mass, a two-hander in one hand weak and imprecise, fixed by the second hand.
5. The second hand slides along the haft; the rear hand gives leverage, both steer.
6. A static guard blocks; moving into the blow parries, and a parry's payoff is loud (the
   enemy reels).
7. Never move the player's head: push the weapon, not the camera.
8. A blade stopped by a wall or shield pivots round the grip; the hilt stays in the fist.
9. After a clean hit the blade follows through instead of binding (stabs that lodge are the
   loved exception).
10. Stamina is the honest brake on flurries.

## Physical melee, the facts that matter

* **The weapon is the physical object.** A held item is jointed to a kinematic body following the
  controller: position spring 5000, damper 100, at most 3000 N; rotation 1000, 50, at most
  250 N m. Nothing is scaled by mass: lag is inertia against a force-limited drive (a 250 N m cap
  saturates at about 13 degrees of error, so heavy one-handed weapons lag in rotation, not
  position). The drawn hand sits on the weapon; nothing shows the controller.
* **Two hands on one haft** double each hand's spring and cap; only the dominant (front) hand
  drives rotation, the other adds position, and the lever between them does the turning (at
  30 cm apart about 1800 N m of torque against 250 one-handed). Sliding along a handle: its spring
  along the axis drops to 0 and a damper slows it. A two-hander in one hand has no special rule: it
  just lags.
* **Impact velocity** is the item's point velocity at the contact from the last physics step
  (before the solver), minus the target's: never the controller's. With the weapon a joint-driven
  body, a flick only scores if the drive can really spin the weapon.
* **Damage** is mostly the "bite": an edge that meets flesh within its angles above the material's
  minimum (slash 3 m/s on flesh, plate 8) bites in and does the big damage; off-angle falls back to
  a weak blunt hit. That is the main anti-waggle.
* **Pushes** stagger by impact speed per weapon class (slash on flesh: 8 m/s; a two-handed sword
  6, 9, 20, 24 for levels 1 to 4; massive blunt 3.75 to 12); a hit on an already staggered enemy
  escalates.
* **Parries**: an NPC's attacking weapon meeting the player's weapon or shield above 1 m/s in its
  damage window ends its swing (it can't damage again in it) and rolls a recoil, nothing or a
  riposte. A heavy player weapon pushes through an NPC's block (a maul over 13 m/s) and can
  destabilise and disarm it. The player's weapon gets no scripted push: an NPC's attacking arm is
  far stiffer than the player's drive, so its blow shoves the player's weapon, which springs back.
* **Walls**: plain contacts against the force-limited drive; a held weapon stuck past arm's
  length is let go. Blades bite wood and earth, never stone or metal.

## Oblivion's melee

Game settings (the values below are `Oblivion.esm`'s GMST records):

**The hit.** `Actor::Hit` (the same function for every actor, of
`(attacker, power, projectile, target)`; the target found by the hit cone when none is
given) does everything: the weapon's damage, sneak attacks, blocking, armour, enchantments, the
hit's sounds and blood, the target's reaction and the power attack's consequences.

* **Weapon damage** (the game's formula of `(skill, luck, strength, fatigue, damage, condition, mult,
  no_fatigue)`) is `damage x fDamageWeaponMult (0.5) x (fDamageSkillBase + fDamageSkillMult (1.5)
  x skill/100) x (fDamageStrengthBase (0.75) + fDamageStrengthMult (0.5) x min(strength, 100)/100)
  x (fDamageWeaponConditionBase (0.5) + fDamageWeaponConditionMult (0.5) x condition) x fatigue x
  mult`. The skill is luck-adjusted (`iActorLuckSkillBase + fActorLuckSkillMult x luck`
  added, clamped 0..100). The fatigue term is `fFatigueBase (1) - (1 - fatigue/max) x
  fFatigueMult (0.5)`: an exhausted fighter hits for half. `mult` is the power attack's bonus.
  The caller (the weapon's damage for an attacker) adds the attacker's AttackBonus
  (actor value 42).
* **Power attacks.** The hit's `power` flag, and the attack's animation group (the attacker's
  anim data, its weapon sequence's group):
  22 AttackPower (standing), 23 AttackForwardPower, 24 AttackBackPower, 25/26 AttackLeft/RightPower.
  * The damage bonus (by skill and group): `fDamagePowerAttackBonus` (2.5) at Novice; the
    standing one `fDamagePowerAttackStandBonus` (3) from Apprentice, the sides `...SideBonus`
    (2.5) from Journeyman, the back `...BackBonus` (2.5) from Expert, forward `...ForwardBonus`
    (2.5) at Master (mastery by `iSkillApprenticeMin` 25, `iSkillJourneymanMin` 50,
    `iSkillExpertMin` 75, `iSkillMasterMin` 100).
  * Side power attacks from Journeyman can disarm (`iPerkAttackDisarmChance`); the
    back one from Expert can knock down (`iPerkMarksmanKnockdownChance`, read by the hit for melee
    too); the forward one at Master can paralyse (`iPerkMarksmanParalyzeChance`, the default
    paralyse spell). Every power attack also rolls the game's knockdown (`(damage x
    fKnockdownDamageMult (0.3) + base) / (agility x fatigue x mult + base)`, at most
    `fKnockdownChance` 0.3) and pushes the target back (`fKnockbackForceMax`,
    `fKnockbackTime`).
  * The player's own attack picks the direction from the movement keys (forward 23,
    back 24, left 25, right 26; sneaking: always standing), and the animations that exist say which
    the player has learned: `onehandattackpower.kf`, `onehandskill2attackleftpower.kf` (Journeyman),
    `...skill3attackbackpower.kf` (Expert), `...skill4attackforwardpower.kf` (Master), and the same
    for `twohand`.
* **Fatigue.** Each attack costs `fFatigueAttackWeaponBase (7) + fFatigueAttackWeaponMult (0.1) x
  weight`, times `fPowerAttackFatiguePenalty` (5) for a power attack, taken by
  the game's damage to the attacker's fatigue as the attack starts.
* **Blocking.** A target that's blocking and facing the attacker takes
  `1 - min(fBlockMax, (fBlockSkillBase + fBlockSkillMult x skill/100) x fatigue x
  (fBlockAmountWeaponMult with a weapon / ...HandToHandMult))` of the damage and pays
  fatigue for it. The blocker plays BlockHit (group 28), and the attacker of a
  blocked melee blow **recoils** (group 30): the game's own "knocked aside". At
  Expert block a block can stagger instead (`iPerkBlockStaggerChance`); at Master it can
  disarm the attacker (`iPerkBlockDisarmChance`).
* **Reactions.** Recoil (group 30), Stagger (31), knockdown (its own,
  logged "%s is staggered down by %s's %f total point blow!").

**Weapons** (`TESObjectWEAP`): type, speed (the attack animation's rate: dagger 1.4, shortsword
1.2, war axe 1.1, longsword and battleaxe... 1.0/0.8, claymore 0.8, warhammer 0.7), reach
(dagger 0.6, war axe 0.8, longsword 1.0, claymore 1.3, warhammer 1.3), weight. Their meshes carry
their own Havok bodies: an iron dagger 5, longsword 10, claymore 14, war axe 16, warhammer 30 (its
`bhkRigidBody` mass), with convex collision shapes. The game's people weigh 605 in the same units
(the 18 bodies of `skeleton.nif`'s ragdoll): against an adult's 75 kg that's 8.07 to the kilogram,
so the longsword is 1.24 kg, the claymore 1.74, the war axe 2.0, the warhammer 3.7 and the dagger
0.62: what real ones weigh. That proportion (the game's own masses against the game's own body) is
the conversion; no weapon is weighed by hand.

**The game's own swing.** Each attack animation (`meshes\characters\_male\*attack*.kf`) moves
the `Weapon` node (the grip) through its swing and marks the moment of the blow with a `Hit` text
key. At that key the weapon's grip has a velocity and the weapon a spin: a point `d` up the blade
moves at `|v_grip + w x d|`, times the weapon's speed (the animation's rate). That is how fast the
game itself swings each weapon at each point, for a normal attack and for a power attack: the
speeds a full hit and a power hit need here.

## What the rig did before this

* Held things are dynamic bodies driven by velocity onto the hand's target, with acceleration
  limits from their mass (`heft = (1 + 2 x weight x mass)^2`), 2.2 and 3 times more in two hands.
  The world's static collision stops them, the hand holding them sits on them, and a ghost hand
  shows the controller. Creatures' body capsules are kinematic bodies they also meet.
* The weapon's collider was the box round its whole model (a sword 16 cm wide at the guard).
  Oblivion's weapons weighed `weight x 0.06` kg, clamped (hand-tuned).
* Hits were judged on where the *controller* drives the weapon (`Rig::swing_pose`), so a flick of
  the controller swung a heavy weapon's judged tip as fast as a dagger's.
* A full hit at 7 m/s and a power hit at 8 m/s for every edge (`full_and_power`), whatever the
  weapon; a glancing blow landed nothing; any clean hit was the game's full hit; power
  attacks had no direction (the game's group wasn't set, so they got no damage bonus).
* A block held the game's Block control 0.5 s; a parry was the same with a flag.
* Two-handed: the second hand took the 6 cm handle round the grip (nothing measured), the gun
  model's aiming (pivot on the main hand, aimed at the support hand).
* Creatures' weapons and shields weren't physical: blades went through them.

## The design

### Framework (any game)

* **The swing is the weapon's, not the controller's.** Every held melee weapon is judged on its
  physical body: the velocity the rig drives it with each step (before the solver stops it on
  anything), at each damager point, including its spin. A heavy weapon can't be flicked: its drive
  is limited by its weight and its inertia round the hand, so wrist flicks of a long weapon stay
  slow at the tip.
* **The arm has to move.** As PLANCK: a hit counts as more than a tap only while the holding
  hand itself (the controller) moves; a still hand spinning a blade is a weak hit.
* **Full and power speeds are the game's.** Each damager carries the speed of a full hit at both
  ends and the power attack's speed over it (`Damager::full_a`, `full_b`, `power`), from the game's
  own swing; the runtime's defaults only for a game that doesn't say. The strength
  is the speed over the full speed at that point: under 1 a weaker hit (never nothing once it's
  past the material's minimum), 1 a full hit, past the power speed a power hit.
* **Swings.** A swing is the weapon moving past half a full hit's speed until it slows again; its
  peak decides it. A swing that peaked past a full hit and hit nothing is a `CE_SWING` (the game
  charges its attack's fatigue and plays its swing); a swing that hits is the hit's. The swing's
  direction in the player's frame (a thrust, a chop down, a rising cut, a cut to the left or right)
  goes with it: the game picks its power attack by it.
* **Glancing blows** are the game's hit at the glancing strength (the flat slaps, weakly), not
  nothing.
* **Follow-through.** A clean cut lets the blade on through that creature (its body no longer
  stops that weapon) until it has come out; stabs and chops that go in still stay in. Blunt and
  glancing hits stop on the body.
* **Parries and blocks by the physics.** The player's weapon or shield meeting a creature's
  swinging weapon: a parry when the player's weapon moves into the blow faster than the blow moves
  into it (it struck the blow), a block otherwise. The blow pushes the held weapon: an impulse of
  the blow's momentum (the creature's weapon's mass from the game, at the blow's speed) into the
  weapon at the contact, which the rig's drive then pulls back to the hand at the weapon's own pace:
  heavy blows shove harder, heavy weapons are shoved less. A heavy blow (a power attack) landing on
  the player shoves whatever the hands hold the same way. Never the head.
* **Creatures' weapons and shields are solid.** The game sends them as parts (`PART_SHIELD`); the
  rig follows them as bodies, so blades stop on them. A blade meeting a blocking creature's guard
  is a hit on its guard (`CEF_GUARD`): the game's own block decides it.
* **Two hands.** A melee weapon's handle (`grip_line`: a sword's grip, an axe's or a staff's haft)
  takes either hand anywhere along it, the hand's own turn about it kept (it closes where it is:
  nothing moves). The second hand holds where it took hold; with its trigger held it slides along
  (along the handle). Both steer: the weapon turns about the main hand towards the other. Both
  swing: in two hands it accelerates and turns twice and three times as hard.
* **Weight and strength.** A melee weapon's turning is limited by its inertia round the hand (its
  mass and shape: the game's), not its mass alone, so a long or top-heavy weapon turns slowly;
  and the arm is as strong as the game says (`MeleeTable::arm`): a strong character swings the
  same weapon faster. A two-hander in one hand has one hand's torque against its whole length.

### Oblivion

* **The hit stays the game's**, scaled by the swing: while the adapter's hit runs, the weapon
  damage formula gets the swing's factor in its multiplier. The factor runs from the
  game's own weakest hit (its exhausted one: `fFatigueBase - fFatigueMult` of a rested one, 0.5)
  at the weakest swing to 1 at a full one. Skill, strength, condition, fatigue, sneak attacks,
  armour, blocking, enchantments, sounds and blood stay the game's.
* **Power attacks** are the game's: the `power` flag and, during the hit, the player's weapon
  sequence set to the group the swing's direction asks for, if the skill has learned it (the side
  ones from Journeyman, the back one from Expert, the forward one at Master; sneaking: standing),
  else the standing one. The game then applies its own bonus, knockback, knockdown, disarm and
  paralysis rolls.
* **Fatigue**: a swing or hit costs the game's own attack fatigue, five times for a
  power attack.
* **Parries**: the game's block (the Block control held) and, for a parry, the attacker's own
  recoil at once so its blow is knocked aside before it lands, plus the block skill's
  Master disarm roll.
* **Creatures** send their weapon (always, not only while attacking) and shield as guard parts,
  the weapon's mass (its Havok mass, converted as above), whether the attack is a power attack
  (its group 22..26), and a count of its blows that landed on the player (the hit cone
  returning the player inside its hit).
* **The weapon in the hand** is the game's model with its own Havok shape (the mesh's convex
  shapes, from the BSA) and mass; its grip line measured on the model (the handle from the pommel
  to the guard; a haft from its foot to its head); its full and power speeds from the game's own
  attack animations (1H or 2H) at their `Hit` keys, times the weapon's speed.
* **The arm's strength** is the game's damage formula's strength term (`fDamageStrengthBase +
  fDamageStrengthMult x strength`), the same factor the game gives strength in a blow.
