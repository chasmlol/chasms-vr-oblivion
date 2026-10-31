# Playtest: melee, in the test range

Framework only: nothing of this is in Oblivion yet, and no save is needed. Everything is in the
framework's test range, whose melee areas play with the runtime's own numbers. How it works and
why: `framework/docs/MELEE.md`.

## Starting it

Put the headset on first. Each `.bat` closes whatever's running (the game, an older range) and
starts the range from this build:

| File | Where you start |
|---|---|
| `range.bat` | the range's usual start: training dummies ahead, the throwables' table at your left (an arming sword and a dagger on it), the melee rack behind it. Gravity gloves and the range's own buttons, as before. |
| `arena.bat` | the arena: fighters come at you (one sword, sword and shield, two swords) while you're in it, a table of weapons at your left; when a wave is down for good the next comes, harder (Easy, Medium, Hard; from wave 2 a shield-bearer in full plate). |
| `rack.bat` | facing the melee rack: dagger, mace, axe, longsword, 8 kg maul, shield, spear, staff. |
| `bench.bat` | a step from three armoured dummies (leather left, chainmail ahead, plate right), two more past the table at your left, one behind the other, to run through, a wooden post; a mace, a sword and a dagger on a table at your left. |
| `climb.bat` | in front of a 2 m stone block to climb. |

The arena, rack, bench and climb starts use the melee buttons. Every start has the gravity gloves for
pulling things in (telekinesis is off). All the areas are in one room, so you can walk between them
from any start; the fighters only come once you step into the arena.

## Controls (the melee buttons, in the melee areas)

* **Grip** near a weapon's handle takes it by that handle: your hand goes onto its grip (the way the
  weapon's made to be held, or reversed if your hand comes at it the other way round) and the
  weapon's pulled into your hand. Grip anywhere else on it and it's held where you took it. Let go
  fast to throw.
* **Trigger**, on a weapon you hold: slides the hand along it while held (a light pull slides
  stiffly, a full pull freely). On the arming sword or the longsword it slides on over the guard
  onto the blade (half-swording) and back.
* **Empty hand**: hits (a punch, open or closed). Grip on a creature takes hold of it where the
  hand is (the throat chokes, the mouth muffles); grip on an edge of the world holds it, to climb.
* **Movement stick click**: kicks (at what you look at within 2 m, with the foot on its side).
* **Dominant hand's B**: slows time (the focus meter on the HUD).
* **Dominant stick click**: jumps; hanging from a ledge, it's a climb jump off it.
* **Spell hand** (the off hand, empty) **trigger**: charges the spell (fire or shock: they alternate
  each cast; the HUD says which). Held by a weapon's blade in the other hand, the charge fills the
  blade with it (it glows).
* **Gravity gloves** (as at `range.bat`): point an empty hand at a loose thing (a weapon too), grip
  to lock on and flick back: it flies to you. Catch it and a weapon goes into the hand by its grip.

(At `range.bat`: the off hand's stick click slows time, and jump and crouch are the range's own.)

## What to try

**Holding**

1. Grips (rack): take each weapon by its handle with your hand turned any way: it turns into your
   hand at its proper grip (a sword's edge out from the knuckles, the axe's edge forward, the
   shield's face out from the back of your hand, in either hand). Come at a sword or the dagger with
   your thumb towards the pommel for a reversed grip. A second hand on the longsword's or the
   spear's handle goes onto it the same way. Grab the axe by its head instead: it's held there,
   where you took it.
2. Weight: pick up the dagger, then the 8 kg maul (rack). The maul sags, trails a fast swing and
   swings on past where you stop; the dagger keeps up. Your empty hand never lags. Chop hard with
   the maul, the spear or the staff while turning your wrist: it stays in your hand, nothing
   spinning off.
3. Two hands: take the longsword or the maul in both hands. Much easier than one hand (the
   doubled drive and the leverage of the hands' spread); the weapon leans towards the second hand.
4. Sliding: on the spear or staff, hold the trigger and slide a hand along it; on a sword, slide
   from the grip over the guard onto the blade.
5. Stuck: stab something and walk away: once your hand's further than you reach, it lets go.

**Hitting** (training dummies at `range.bat`, the bench)

6. Damage from the impact: a lazy swing barely hurts, a hard one hurts. The dummies
   pale as they lose health.
7. Armour (bench): a sword's edge glances off the plate dummy; it cuts the leather one; the mace
   hurts all of them. A blow that does nothing scratches or scuffs the armour.
8. Stabs that stick: push the sword's point into a dummy slowly (it goes in by pressure), or press
   the edge on and draw it along (a draw cut). Shallow, the blade rocks where it went in; twist it
   and the body turns with it; pull it back out (blood follows). Stab the first of the two dummies
   past the table hard and push on: it runs through into the second.
9. Dismemberment: cut into an arm, a hand, a shin, a foot or the head with an edge (any sword, the
   dagger, the axe, the spear's blade) and it comes off, at its joint (the head by its neck): the range's
   Easy Dismemberment, on by default. A thigh or the torso never comes off, a stab still sticks, and
   armour that stops the edge (the bench's chainmail and plate) stops the cut.
   For the strict cut instead (a hard, square cut within 4 cm of a joint, 12 m/s for the arming
   sword), add `[melee]` / `easy_dismemberment = false` to `%LOCALAPPDATA%\RustVR\settings.toml`.
10. Knockouts: the mace on a bare head (the chainmail dummy's) knocks it out for 6 s, out cold on
    its feet (eyes shut).
11. Feedback: hits sound by what hit what (flesh, wood, metal clashing), sparks off metal, the
    controller buzzes; a stuck blade buzzes as it goes in or slides, and a blade drawn along the
    table or a body buzzes every 2 cm.
12. Blood on the blade where it went in, wounds on the body, a cut neck bleeding.
13. Being struck (arena): a fighter's blow hurts you 5 to 15 (the head x1.5, the hands x0.5).

**Defence** (arena)

14. Blocking: hold your weapon in a blow's way. Against a heavy weapon (the maul, the longsword),
    your block can stagger you.
15. Parrying: move into the blow (over 1 m/s): the swing turns back, or the fighter recoils.
16. Disarming: hit a fighter's hand hard, or stab through its arm, and it drops its weapon; or take
    hold of its weapon and pull it away.

**Enemies** (arena)

17. Ragdolls on muscles: a dummy or a fighter stands still and moves smoothly (no shaking, in
    slowed time too); shove one, knock it down; it gets up once it's still (a push puts it back
    down). Falls and slams hurt.
18. Reactions by force: a light hit flinches it, a hard one staggers or floors it, and any stagger
    stops its swing.
19. The fighters' moves: real swings at speed, combos, lunges, shield pushes, two-sword moves; at most one
    (Easy), two (Medium) or three (Hard) at you at once, the others hanging back; they dodge back
    from thrusts; Medium and Hard ones feint.
20. Fear: disarmed, a fighter goes and picks a weapon up; with none to take, it runs for the arena's
    far corner and is gone. Voices: grunts, shouts as it swings, a last line by how it died
    (breathless from a cut throat, a long death from a stab), a scream as it falls (throw one).

**Grappling and unarmed**

21. Grab a dummy or a fighter: your hand and the body stay steady in the hold (no shaking); by the
    neck while it's unaware, it goes down at once; by a shoulder or a wrist it struggles, and may
    break free (your hand is forced off); lift it and it falls or your hand comes off; the throat
    chokes it (it drops its weapons).
22. Tear a limb off with both hands: take a dummy by both shoulders (or an arm and the head) and
    yank them apart hard.
23. Punches (an empty hand: only a hard punch does anything) and kicks (the movement stick's click,
    at a dummy ahead of you; a body that's down is shoved hard).

**Extras**

24. Throwing: the dagger flies point first and sticks; the arming sword tumbles; the spear flies
    point first.
25. Gravity gloves: point at a weapon on the rack, grip and flick: catch it and it's in your hand by
    its grip.
26. Telekinesis is off. To try it (on things and on bodies: catching, throwing, pulling a body
    apart with both hands), add `[melee]` / `telekinesis = true` to
    `%LOCALAPPDATA%\RustVR\settings.toml`; it then replaces the gloves everywhere.
27. Slowed time: B on your dominant hand; about 6.7 s from 30 focus (it starts at 30, holds 60, and
    refills always).
28. Imbuing: charge fire by your blade until it glows, then hit or stab a dummy: it catches fire and
    burns to death in 4 s; 10 s alight, the body is charred through: stiff, and it tears apart if
    you pull it or hit it hard. Shock (cast once to switch): a hit may stun.
29. Breaking: bottles, boxes, crates and the barrel at the range's start break if hit or thrown
    hard; they come back whole after 8 s.
30. Climbing (`climb.bat`): reach up, grip the block's top edge with both hands, pull down; click
    the dominant stick to jump up off it (a climb jump), or push down to your hips and let go to
    step up. An axe stuck in a wall holds you too.
31. Stamina: off by default. To try it, add to `%LOCALAPPDATA%\RustVR\settings.toml`:
    `[melee]` / `stamina = true` (and `stamina_time = 3` to tire fast). The HUD shows it.

**Guns, grenades, knives and spells on them** (the range's main hall and the bench)

32. Shoot a dummy or a fighter (the rifle and pistol on the table by the start): a rifle round goes
    through (two to the body kill), a pistol takes four (a head shot sooner); armour soaks some
    (plate sparks); shot through the arm, a fighter drops its weapon; a dead body jerks when shot.
33. Grenades (the chest pouch, or the throwables' table): a frag kills within about 2 m, floors
    everything within 4 m, tears limbs off within about a metre and throws the bodies (Metro's
    incendiary, once extracted, sets them alight). The room's tables and walls shield them.
34. Throwing knives (the table at your left): one that sticks in a dummy hurts it as a thrown
    dagger does (a head or a throat kills), and rides with it as it moves.
35. Spells: cast the fire bolt or the shock at a dummy (charge, then throw towards it): the fire
    bolt hurts and knocks it down, the shock jolts it and may stun it.

## Checked here, and not

Checked with the emulated headset (scripts and captures), with the numbers seen in the logs:

* the shaking: a standing dummy's upper arms were twisting back and forth at 94 rad/s with nothing
  touching it, and a head held by the throat at 35 rad/s; now still (0.00 rad/s idle, 0.1 held), a
  wrist held while your hand sways follows smoothly, and an armed fighter stands still (unit
  probes, run and removed);
* the rifle into the training dummy's chest: 30 a round, dead at the second; a frag let go at your
  feet half a metre from it: dead, 10 parts torn off, and the second dummy 1.5 m off killed; a
  throwing knife into the left dummy's chest at 10 m/s: in, 27.8 (temporary scenarios, removed);
  offline: a pistol round at the throat 15, a frag a metre from the bench's first dummy killing
  the three nearest and flooring the other two, a fire bolt landing;
* grips: the maul taken with the hand rolled 60 degrees and turned 30 off its grip goes into the
  hand at its grip (125 degrees off at the grab, 7 after 0.15 s), the fist round the haft; the
  staff in both hands, each hand onto the shaft; the sword taken by the middle of its grip, raised
  pointing forward and slid over the guard onto its blade; a sword reached for at the end of its
  grip is held by its grip, not its pommel. A weapon caught from the gloves goes in by its grip,
  its own way round (a unit test: the scripted aim at the rack missed);
* heavy weapons: six hard chops with the 8 kg maul, the wrist rolling 40 degrees each way: it trails
  up to 0.29 m and settles to 1.7 cm and 4 degrees of sag, never leaving the hand. Before the fix a
  chop with the wrist turning spun a test maul about its haft at 53 rad/s, its grip 0.94 m off the
  hand;
* weight (the maul sags 2° and trails 60° in a 4.5 m/s swing), two hands, sliding over a sword's
  guard onto its blade;
* a cut across the elbow taking the forearm off at its joint (a cut mid-arm doesn't), a blow at
  neck height taking the head by its neck; a blade brushing armour scratching it;
* a stab running two through: into the first dummy's chest (deep at 0.1 m, out its back at 0.25 m,
  killing it), pressed on into the second (627 N) and deep in it;
* kicks (each foot into the chainmail dummy at 5.8 m/s, push level 2); the climb jump off the
  block (0.3 force up and along the gaze);
* (with telekinesis on, before it was turned off for the gloves) telekinesis on a dummy (caught,
  lifted, thrown at 30 m/s); two telekinesis hands tearing its head off (3012 N at the neck); both hands tearing an upper arm off in a yank (3479 N); a dummy
  set alight by a fire-filled sword held in it, burned to death and charred through 10 s alight;
* the fighters (from the creature work): parries and their chains, Easy's one-at-a-time spacing,
  dodges from thrusts, a held fighter struggling, a neck grab flooring an unaware dummy and choking
  it to death, knockouts standing and limp, a punch's push on a body that's down, a wave cleared and
  the next arriving, a disarmed fighter fetching a sword and another fleeing;
* the climb area stays quiet; the arena's first wave comes at once;
* the runtime's own scenarios: the melee ones pass (stab, stab into the floor, knife, the grenade,
  the spell, taking the sword by its pommel, and the sword's chop this time; it fails about half
  its runs with or without this round's changes: each run its blade meets a different part of the
  dummy, and the check wants exactly one hit); the eight that need Metro 2033's extracted guns and sounds fail
  on this machine (it has none extracted), and crouch and wall_fade failed once in a full run but
  pass on their own. The unit tests pass, but for two arm IK tests and a bullet-hole picture test
  that already failed before this work.

Not checked: anything by feel in a real headset (the weight, the grips and a held body's
steadiness above all), the pistol on a fighter, the incendiary, a knife riding a
fighter as it moves, a spell cast at a fighter in the headset, the gloves'
pull in the headset, a fighter's weapon taken by its handle and wrenched away since the grips
snap, sounds by ear (the runs
were silent: the new voice lines, the clash), the controllers' buzz, a heavy weapon's push on your
guard, a body thrown into another creature, the mouth grab silencing one, a held arm following
your hand on a fighter, a stagger stopping a swing, rocking and twisting a shallow blade, a
charred body tearing apart, the left hand taking a sword's pommel while the right slides (the
script's hand missed the pommel).

## Calls made (all in `framework/docs/MELEE.md`)

* A weapon taken near a handle goes onto its grip and is pulled into the hand (your
  rule now: weapons snap to their rigged grips). Off its handles it's held where you took it.
* Knockouts come above set speeds and last 6 s.
* A hand on the throat hurts (5 a second); a pulled weapon is wrenched away
  (by hand or telekinesis); fighters feint. You asked for these.
* Always on in the range: tearing limbs off by hand (and,
  with telekinesis turned on in the settings, its throw, its hold on bodies and its tearing).
* The focus meter holds 60 (an earlier call said 30).
* The melee buttons are the game's to ask for: the range's melee areas ask for them, Oblivion keeps its
  own. Telekinesis is off unless the settings turn it on; the gravity gloves pull things in.
* Each wave comes while you're in the arena.
* The runtime's gore stays its own (wounds bleed, blood runs and pools), with its blade blood and
  neck bleed.
* Fire and shock alternate as they're cast (the range has no spell wheel).
* The guns' and grenades' damage on the creatures is ours (a pistol 15, a rifle 30 and
  through, a shotgun 45, by the armour and the part multipliers; a frag 120 at its centre
  falling off over 6 m, limbs torn off within 1.2 m). A thrown knife is a thrown dagger; a spell
  cast at a creature lands at once (no flight), as a fire bolt's or a lightning hit.

## Known issues

* Thrown and telekinesis-held things are judged as weapons only if they're melee weapons or
  throwing knives (a thrown bottle breaks, but doesn't hurt).
* A body held at full stretch on a stuck blade drops it (the stuck rule: the grip's past your
  reach).
