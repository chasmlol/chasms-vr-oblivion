# Playtest: melee, in the test range

Framework only: nothing of this is in Oblivion yet, and no save is needed. Everything is in the
framework's test range, whose melee areas play with the runtime's own numbers. How it works and
why: `framework/docs/MELEE.md`.

## Starting it

Put the headset on first. Each `.bat` closes whatever's running (the game, an older range) and
starts the range from this build:

| File | Where you start |
|---|---|
| `range.bat` | the range's usual start: training dummies ahead, the throwables' table at your left (an arming sword and a dagger on it), the melee rack behind it. Gravity gloves, as before. |
| `arena.bat` | the arena: fighters come at you (one sword, sword and shield, two swords), a table of weapons at your left; when a wave is down for good the next comes, harder (Easy, Medium, Hard). |
| `rack.bat` | facing the melee rack: dagger, mace, axe, longsword, 8 kg maul, shield, spear, staff. |
| `bench.bat` | a step from three armoured dummies (leather left, chainmail ahead, plate right), two more behind to run through, a wooden post; a mace, a sword and a dagger on a table at your left. |
| `climb.bat` | in front of a 2 m stone block to climb. |

The arena, rack, bench and climb starts play the full melee: telekinesis takes the gravity gloves'
place there. All the areas are in one room, so you can walk between them from any start.

## Controls (the melee buttons)

* **Grip** takes what the hand's on (where it is: nothing snaps into the hand). Let go fast to throw.
* **Trigger**, on a weapon's shaft or haft (spear, staff, mace, longsword): slides the hand along it
  while held (a light pull slides stiffly, a full pull freely).
* **Empty hand**: hits (a punch, open or closed). Grip on a creature takes hold of it (the throat
  chokes); grip on an edge of the world holds it, to climb.
* **Off hand's stick click**: slows time (the focus meter on the HUD).
* **Spell hand** (the off hand, empty) **trigger**: charges the spell (fire or shock: they alternate
  each cast; the HUD says which). Held near a weapon's blade in the other hand, the charge fills the
  blade with it (it glows).
* **Telekinesis** (arena, rack, bench, climb): raise an open empty hand and point at a loose thing up
  to 6 m off (it's marked); grip catches it, held off where you point; the trigger pulls it in (it
  ends in your hand), **A/X** pushes it away; let go with a flick throws it at 20 m/s, a weapon
  point first. Pointing at your own thrown weapon and pulling is the recall.

## What to try

**Holding**

1. Weight: pick up the dagger, then the 8 kg maul (rack). The maul should lag a fast swing and swing
   on past where you stop; the dagger should keep up. Your empty hand should never lag.
2. Two hands: take the longsword or the maul in both hands. Swinging should be much easier than one
   hand (the two-hand doubling and leverage).
3. Sliding and grip spots: on the spear or staff, hold the trigger and slide a hand along it; on the
   arming sword, slide from the grip up onto the blade (half-swording), or take it by the pommel.
4. No snapping: wherever you take a weapon, it stays where it was in your hand.

**Hitting** (training dummies at `range.bat`, the bench)

5. Damage from the impact: a lazy swing barely hurts, a hard one hurts. The HUD's
   numbers aren't shown; the dummies pale as they lose health.
6. Damage types against armour (bench): a sword's edge glances off the plate dummy; it cuts the
   leather one; the mace hurts all of them.
7. Stabs that stick: push the sword's point into a dummy slowly (it goes in by pressure) or stab
   hard; push it deeper, drag it to cut, twist it, pull it out. Run the two dummies behind the bench
   through in one stab.
8. Dismemberment: a hard, square cut (above ~19 m/s for the arming sword) through an arm, a leg or
   the neck.
9. Knockouts: the mace on a bare head (the chainmail dummy's) knocks it out for 6 s (eyes shut).
10. Feedback: hits sound by what they hit (flesh, wood, metal clash); sparks off metal; the
    controller buzzes.
11. Blood on the blade where it went in, and wounds on the dummy.

**Defence** (arena)

12. Blocking: hold your weapon in a blow's way.
13. Parrying: move into the blow. The fighter's swing turns back, or it recoils, staggered.
14. Disarming: hit a fighter's hand hard, or stab through its arm, and it drops its weapon; or take
    hold of its weapon and pull it out of its reach.
15. Armour per part (bench), as in 6.

**Enemies** (arena)

16. Ragdolls on muscles: shove one, knock it down; it gets up when it's still. Falls hurt.
17. Reactions by force: a light hit makes it flinch, a hard one stagger or fall.
18. Stances and real swings: they swing at real speeds; blocks and parries work against them;
    combos; Medium and Hard ones feint (a swing that turns into another).
19. Their defence: they guard against your blade coming at them; Hard ones guard best.
20. Fear and pain: badly hurt they may run; they grunt, shout, and pick a dropped weapon back up.

**Grappling and unarmed**

21. Grab a dummy or a fighter by any part: the throat (it chokes, drops its weapons, struggles),
    an arm to drag, the body to lift and throw.
22. Punches: hit with an empty hand (only a hard punch does anything). Kicks: not done
    (they need tracked feet).

**Extras**

23. Throwing: throw the dagger (it flies point first and sticks) and the arming sword (it tumbles).
24. Telekinesis: as in the controls; catch a weapon off the rack and pull it to your hand, throw one
    by flicking, pull a stuck dagger back out of a dummy.
25. Slow motion: click the off hand's stick; it lasts about five seconds on a full meter and refills.
26. Imbuing: charge fire by your blade until it glows, then hit or stab a dummy: it catches fire
    and burns (a stab held in sets it alight in a few seconds; slashes take many hits). Charge
    shock (cast once to switch): a hit may stun.
27. Breaking: bottles, boxes, crates and the barrel at the range's start break if hit or thrown
    hard; they come back whole after 8 s.
28. Climbing (`climb.bat`): reach up, grip the block's top edge with both hands, pull down, push down
    to your hips, let go: you step up onto it.
29. Stamina: off by default. To try it, add to `%LOCALAPPDATA%\RustVR\settings.toml`:
    `[melee]` / `stamina = true` (and `stamina_time = 3` to tire fast). The HUD shows it; tired, your
    hands are weaker on a weapon.

## Checked here, and not

Checked with the emulated headset (scripts and captures), with the numbers seen in the logs:

* weight (the maul trails 12 cm in a 4.5 m/s swing and sags 1.6 cm), no snapping, two hands, sliding;
* a slash (11 m/s: 1.35 and a 7.5 cut), a pressure stab running a dummy through, blood, a sever at
  19 m/s, armour (21.8 m/s glancing off plate for 0.3; leather cut), a knockout (a mace at 5.8 m/s
  to the head: 6 s);
* fighters coming and swinging at 11–13 m/s, blocks, parries (the swing turned back), being struck
  (5–15), a shield's guard, a disarm by a stab through the arm, choking and struggling;
* throwing (the dagger at 16 m/s, 98° off when let go, turned point first and stuck 15 cm in), the
  sword tumbling; telekinesis (a weapon caught 1.8 m off, pulled into the hand; another thrown);
  slowed time (the meter drains in 5 s); imbuing (a fire-filled sword stabbed in: the dummy caught
  fire; a shock-filled one held in: shocked, again and again while it stays in); breaking (a bottle thrown at 10.9 m/s, a box chopped at 8.5 m/s; back after 8 s);
  climbing (up the block and onto it); a punch (8.8 m/s: 1.14); stamina (77% after three seconds
  of hard swings, full again after resting).

Not checked: anything by feel in a real headset (the weight above all), sounds by ear (the runs
were silent), the controllers' buzz, wrenching a weapon out of a
fighter's hand, dual-wielding fighters and the Hard wave, fear and fleeing, a fighter picking its
weapon back up, fall damage.

## Calls made (all in `framework/docs/MELEE.md`)

* Nothing snaps into the hand: your rule.
* Knockouts come above set speeds and last 6 s.
* A hand on the throat hurts (5 a second); a pulled weapon is wrenched away;
  fighters feint. You asked for these.
* The focus meter holds 30.
* Telekinesis is off by default for a game that doesn't ask, so Oblivion keeps the gravity gloves;
  the range's melee areas ask for it.
* A climber let go just below the block's top steps up onto it.
* Fire and shock alternate as they're cast (the range has no spell wheel).

## Known issues

* A fighter's blade can read very fast (30+ m/s) right after a parry; the player's damage is
  clamped to 5–15 regardless.
* Thrown and telekinesis-held things are judged as weapons only if they're melee weapons (a thrown
  bottle breaks, but doesn't hurt).
