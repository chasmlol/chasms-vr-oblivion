# Playtest: melee by the swing

Save: rustvr-melee

```
powershell -File tools\play.ps1 melee
```

The save is Weye (the house) with an iron longsword at the left hip and an iron shield over the left
shoulder, an iron claymore, war axe and warhammer in the inventory, health raised to 400, and
two goblins (a skirmisher with a weapon, and a plain one) a few metres off. They come for you at
once; load the save again for another go. Design and research: `docs/research/melee.md`.

## What to try, in order

1. **Draw the longsword** (grip at the left hip) and look at it: the game's longsword, 1.24 kg (its
   own Havok mass). Waggle it with the wrist, the hand still: the tip lags and can't be flicked
   about, and a goblin touched that way isn't hit at all (a hit needs the arm moving).
2. **Hit a goblin** with a committed swing, edge first: the game's hit (its sound, blood, the
   goblin's reaction), as strong as the swing. A firm cut is most of the game's damage, a hard full
   swing all of it, a tap with the arm moving about half (the game's own weakest hit, an exhausted
   fighter's). The flat of the blade is a weak glancing hit. A clean cut goes on through the body;
   a slow push or a flat slap stops on it.
3. **Power attack:** swing really hard (the arm fast, the blade faster than the game's own power
   attack swings it). A chop down is the game's standing power attack (2.5 to 3 times the damage,
   its knockback); across left or right, a rising swing and a thrust are the side, back and
   forward power attacks once Blade is Journeyman, Expert and Master (the save's character is a
   Novice: they're all standing power attacks). Each swing as hard as a full hit costs fatigue (a
   longsword's 9, a power swing 45: watch the fatigue bar), and one that misses swishes (the game's
   own sound).
4. **Block:** hold the sword (or the shield, from over the left shoulder) across a goblin's swing:
   the game blocks it (its Block skill, less damage) and the blow knocks the guard back, the hand
   with it (a ghost hand shows the controller). A goblin's power attack that gets through jolts
   both hands and shoves what they hold.
5. **Parry:** swing the blade across a goblin's blow as it comes: it recoils (its own recoil
   animation), the blades clash (the game's block sound), and its blow never lands. A light weapon
   or a slow guard against a heavy blow is only a block.
6. **Walls and floors:** swing into a wall, the floor, a table: the blade stops on it and the hand
   with it (the ghost hand at the controller); the empty other hand still follows its controller
   exactly. A blade also stops on a goblin's weapon or shield.
7. **Two hands:** equip the claymore (inventory, laser), draw it from over the right shoulder, and
   put the left hand on the handle below the right: it closes where it is (nothing moves) and both
   hands steer and swing it. Hold the left trigger and move that hand along the handle: it slides.
   Let go with the left: the claymore in one hand is heavy, slow to turn (a third as fast) and
   slow to get moving. The claymore needs less tip speed for a full hit than the longsword (it's a
   slower weapon in the game: speed 0.8) and hits harder for its weight.
8. **The war axe and the warhammer:** a hard axe chop into a goblin or wood sticks (pull it out
   along the way it went in); the warhammer is very top-heavy (3.7 kg, its weight at its head).

**A training dummy** (the user's first playtest: two goblins that fight at once leave no room to
try a swing). The same save with these console lines (`-Commands`, `dummy.bat` in the worktree):

```
powershell -File tools\play.ps1 melee -Commands "ff000c49.disable|ff000c48.setrestrained 1|ff000c48.setav aggression 0|ff000c48.setav confidence 100|ff000c48.stopcombat|ff000c48.setav health 1000|ff000c48.resethealth|PlaceInFront ff000c48 1.3"
```

The save's goblins are the placed references `ff000c48` and `ff000c49` (the same in every load of
it). One is put away; the other is restrained (the game's own hold on a prisoner: it doesn't move),
won't start a fight (aggression 0), won't flee (confidence 100), has 1000 health so it lasts, and
stands 1.3 m in front of the player, facing them. A hit is still the game's assault, so it may
want to fight back; restrained, it shouldn't be able to.

For a look at what's judged: set `RUSTVR_DEBUG_COMBAT=1` before starting; the runtime then logs
every contact, swing, block and parry with its numbers, and draws the creatures' capsules and the
weapon's damagers over the picture.

## What was checked in the game (emulated headset, `tools/oblivion.ps1`)

- The game's own hit scaled by the swing: a moderate chop 69 to 75% of the game's damage, a hard
  chop a power attack (the game's 2.5 times bonus applied: 6.6 damage against 1.8 to 2.0), taps
  and glancing blows about half. The scaling touches only the hit's own weapon damage (the goblin's
  answer keeps its own).
- Power attacks: the game's group set for the hit (22, standing, for a Novice).
- Fatigue: a full swing 9, a power swing 45, as the game's formula says for a longsword.
- Landed blows: every goblin blow that reached the player was seen (`Actor::Hit` finding the player
  in its hit cone), power attacks flagged; blocked ones did less damage (the player's action was
  Block: 3 to 4 against 6).
- Blocks and parries: blocks registered with the guard held; parries with the guard swept across
  the blow (the goblin's own recoil each time); blows knock the guard (logged impulse).
- Two hands on the claymore: the left took the haft below the right; in two hands it turns at up to
  287 rad/s^2, in one 96 (its 0.52 kg m^2 about the grip).
- The game's own swing read from its attack animations as the game runs (the same numbers as
  measured offline).
- Masses and shapes from the game's Havok for the longsword, claymore, war axe, mace, shortsword and
  the iron shield.
- The floor stops the blade: the longsword pushed down into it, the hand was held 0.4 m above its
  controller ("blocked", what it holds touching something, in the runtime's log). Pushed on deep
  for seconds (the controller 40 cm under the floor), the blade slid about on it and the drawn hand
  with it (up to 25 cm); the rig does the same in a unit test with a floor of the game's kind (its
  tip creeping round the grip). A real floor stops the real controller long before that; a wall
  the controller goes through might show it.

## What wasn't checked

- Anything in a real headset: how a full hit and a power hit feel for a real arm (the thresholds
  are the game's own swings scaled to the runtime's 7 m/s full cut), the strength of the blows'
  push on the guard, the haptics.
- The side, back and forward power attacks landing (with Blade set to 100 the swings were seen
  going left, rising and down as power swings, and a chop landed as the standing power attack
  with its Apprentice bonus, 3 times, 23.3 damage; the sweeps and the thrust missed the short
  goblin), the block's Master disarm, the knockdown and paralysis perks.
- A blade stopping on a creature's shield or weapon: the guard parts are there (a skeleton's sword
  and its shield's two capsules on its forearm, seen in a capture of the debug view) and followed as
  bodies like the body's parts, but a blade wasn't seen stopping on one. A hit on a blocking
  creature's guard being the game's block wasn't seen either (the goblins didn't block).
- The swish and the parry's clash sound (no audio in the checks).
- Two-handed weapons hitting (the goblin was too close for the claymore's swings), the war axe and
  warhammer.

## Decisions

- **Full and power hits from the game's own swings, scaled to an arm.** The game's attack
  animations swing a longsword's mid-blade at 15 m/s and its power attack at 23 (its characters are
  superhuman). Their proportions are kept (along the blade, between weapons by their speed, one-
  against two-handed, power against normal), scaled so a one-handed sword's full cut at the middle
  of the blade is the runtime's own 7 m/s. The creatures' blows are taken at the
  same scale (0.47).
- **Masses from the game's Havok,** converted by the game's own people: its ragdoll is 605 Havok
  masses, an adult 75 kg. The iron shield comes out at 0.62 kg (its Havok mass is half the
  longsword's): light, so it's shoved more and parries less than a real one would.
- **The arm behind a hit:** a hit needs the hand moving (0.5 m/s), counts for no more than the
  hand's speed over 1 m/s (PLANCK's), and a power hit wants the hand as fast as the game's own
  swing moves its grip (4.3 m/s for a one-handed sword, 2.2 for a claymore). Wrist waggles are weak.
- **Stabs are the runtime's own** (5 m/s full, along the blade): the game's
  attacks cut and chop, and its only thrust is the forward power attack, so there's no game thrust
  to measure a normal stab against.
- **The weakest hit is half the game's** (its exhausted fighter's: `fFatigueBase - fFatigueMult`),
  not nothing, so a deliberate tap still hurts a little and a full swing is the game's own.
- **A parry** is the guard swung across the blow's blade (1 m/s or more) with the momentum (the
  weapon and the arm behind it) to beat what the blow has against it; a block otherwise. A parry
  recoils the attacker before its blow lands, so the game's block itself (skill use, fatigue) only
  happens for blocks.
- **Swings cost fatigue** only when as hard as a full hit (waggles cost nothing, as they do nothing);
  a power swing costs a power attack's, hit or miss.
- **A clean cut follows through** (the body stops a blunt hit, a glancing one, a slow one).
- **Melee weapons turn by their inertia about the hand** (a long or top-heavy one slowly); the
  arm's torque (`ARM_TORQUE`) is set so a one-handed sword turns as everything held always has.
- **The flat game's crosshair is hidden** (riding's `reticle.rs`, cherry-picked): the hands aim in
  VR, and a dot fixed in the middle of the view is only in the way. The activate icons stay.
- **A free grab of a weapon's handle moves nothing** (the hand closes where it is); drawing from a
  holster still puts it in the hand the rigged way. A rig test that expected the old snap was made to
  let the sword settle first.

## Not done

- A blade pressed on into a wall or the floor far past it slides about on it (see above): the hold
  could give up turning it into the surface once it's stopped there.

- The gravity sag of a heavy weapon held out in one hand (it's slower to turn, but it doesn't droop).
- The block's skill use and fatigue for a parry (the blow never lands, so the game's block doesn't
  run); the side, back and forward power attacks' own animations aren't played (there's no game
  animation for the player in VR), only their effects.
- A shield bash.

## For merging

- Protocol v12 (`Actor::weapon_mass`, `blows`, `ACT_POWER_ATTACK`, `Actors::blow_scale`,
  `Damager::full_a`/`full_b`, `MeleeDesc::grip_speed`/`power`, `MeleeTable::arm`, `CE_SWING`,
  `CEF_GUARD`, `CombatEvent::swing`, `FULL_SLASH_SPEED`). `CE_HIT`'s meaning is unchanged for gore
  (which creature, which part, which damager, how strong); hits on a guard have `CEF_GUARD` and a
  guard part's index (past the body's parts), which gore should leave alone.
- New hooks in `crates/oblivion/src/combat.rs`: the weapon damage, `Actor::Hit`
  and the hit cone's target. Another branch hooking those would clash.
- `objects.rs` now writes the game hulls for both owners (the weapons' and the items'); `items.rs`
  goes through it.
- Branches: `overnight/melee` in chasms-vr-oblivion and in chasms-vr-framework.
