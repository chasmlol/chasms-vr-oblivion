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

The arena, rack, bench and climb starts play the full melee: its buttons, and telekinesis in place of
the gravity gloves. All the areas are in one room, so you can walk between them from any start; the
fighters only come once you step into the arena.

## Controls (the melee buttons, in the melee areas)

* **Grip** takes what the hand's on, where it is (nothing snaps into the hand). Let go fast to
  throw.
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
* **Telekinesis**: raise an open empty hand and point at a loose thing or a creature up to 6 m off
  (it's marked); grip catches it, held off where you point; the trigger pulls it in, **A/X** pushes
  it away; let go with a flick throws it (a weapon point first at 20 m/s, a body at 30). Pointing at
  your own thrown weapon and pulling is the recall.

(At `range.bat`: the off hand's stick click slows time, and jump and crouch are the range's own.)

## What to try

**Holding**

1. Weight: pick up the dagger, then the 8 kg maul (rack). The maul sags, trails a fast swing and
   swings on past where you stop; the dagger keeps up. Your empty hand never lags.
2. Two hands: take the longsword or the maul in both hands. Much easier than one hand (the
   doubled drive and the leverage of the hands' spread); the weapon leans towards the second hand.
3. Sliding: on the spear or staff, hold the trigger and slide a hand along it; on a sword, slide
   from the grip over the guard onto the blade.
4. Stuck: stab something and walk away: once your hand's further than you reach, it lets go.

**Hitting** (training dummies at `range.bat`, the bench)

5. Damage from the impact: a lazy swing barely hurts, a hard one hurts. The dummies
   pale as they lose health.
6. Armour (bench): a sword's edge glances off the plate dummy; it cuts the leather one; the mace
   hurts all of them. A blow that does nothing scratches or scuffs the armour.
7. Stabs that stick: push the sword's point into a dummy slowly (it goes in by pressure), or press
   the edge on and draw it along (a draw cut). Shallow, the blade rocks where it went in; twist it
   and the body turns with it; pull it back out (blood follows). Stab the first of the two dummies
   past the table hard and push on: it runs through into the second.
8. Dismemberment at the joints: a hard, square cut across an elbow, a wrist, a knee, an ankle or
   the neck (a head blow can take the neck); never through a thigh or the torso. The arming sword
   needs about 19 m/s; the blade carries on through.
9. Knockouts: the mace on a bare head (the chainmail dummy's) knocks it out for 6 s, out cold on its
   feet (eyes shut).
10. Feedback: hits sound by what hit what (flesh, wood, metal clashing), sparks off metal, the
    controller buzzes; a stuck blade buzzes as it goes in or slides, and a blade drawn along the
    table or a body buzzes every 2 cm.
11. Blood on the blade where it went in, wounds on the body, a cut neck bleeding.
12. Being struck (arena): a fighter's blow hurts you 5 to 15 (the head x1.5, the hands x0.5).

**Defence** (arena)

13. Blocking: hold your weapon in a blow's way. Against a heavy weapon (the maul, the longsword),
    your block can stagger you.
14. Parrying: move into the blow (over 1 m/s): the swing turns back, or the fighter recoils.
15. Disarming: hit a fighter's hand hard, or stab through its arm, and it drops its weapon; or take
    hold of its weapon (by hand or telekinesis) and pull it away.

**Enemies** (arena)

16. Ragdolls on muscles: shove one, knock it down; it gets up once it's still (a push puts it back
    down). Falls and slams hurt.
17. Reactions by force: a light hit flinches it, a hard one staggers or floors it, and any stagger
    stops its swing.
18. The fighters' moves: real swings at speed, combos, lunges, shield pushes, two-sword moves; at most one
    (Easy), two (Medium) or three (Hard) at you at once, the others hanging back; they dodge back
    from thrusts; Medium and Hard ones feint.
19. Fear: disarmed, a fighter goes and picks a weapon up; with none to take, it runs for the arena's
    far corner and is gone. Voices: grunts, shouts as it swings, a last line by how it died
    (breathless from a cut throat, a long death from a stab), a scream as it falls (throw one).

**Grappling and unarmed**

20. Grab a dummy or a fighter: by the neck while it's unaware, it goes down at once; by a shoulder
    or a wrist it struggles, and may break free (your hand is forced off); lift it and it falls or
    your hand comes off; the throat chokes it (it drops its weapons).
21. Tear a limb off with both hands: take a dummy by both shoulders (or an arm and the head) and
    yank them apart hard.
22. Punches (an empty hand: only a hard punch does anything) and kicks (the movement stick's click,
    at a dummy ahead of you; a body that's down is shoved hard).

**Extras**

23. Throwing: the dagger flies point first and sticks; the arming sword tumbles; the spear flies
    point first.
24. Telekinesis on things: catch a weapon off the rack and pull it to your hand, throw one by
    flicking, pull a stuck dagger back out of a dummy.
25. Telekinesis on bodies: catch a dummy (bench) and lift it (unaware, by the neck it goes down);
    flick and let go: it's thrown at 30 m/s (it screams until it lands). With both hands on one
    body, pull apart: its head or a limb tears off.
26. Slowed time: B on your dominant hand; about 6.7 s from 30 focus (it starts at 30, holds 60, and
    refills always).
27. Imbuing: charge fire by your blade until it glows, then hit or stab a dummy: it catches fire and
    burns to death in 4 s; 10 s alight, the body is charred through: stiff, and it tears apart if
    you pull it or hit it hard. Shock (cast once to switch): a hit may stun.
28. Breaking: bottles, boxes, crates and the barrel at the range's start break if hit or thrown
    hard; they come back whole after 8 s.
29. Climbing (`climb.bat`): reach up, grip the block's top edge with both hands, pull down; click
    the dominant stick to jump up off it (a climb jump), or push down to your hips and let go to
    step up. An axe stuck in a wall holds you too.
30. Stamina: off by default. To try it, add to `%LOCALAPPDATA%\RustVR\settings.toml`:
    `[melee]` / `stamina = true` (and `stamina_time = 3` to tire fast). The HUD shows it.

## Checked here, and not

Checked with the emulated headset (scripts and captures), with the numbers seen in the logs:

* weight (the maul sags 2° and trails 60° in a 4.5 m/s swing), two hands, sliding over a sword's
  guard onto its blade;
* a cut across the elbow taking the forearm off at its joint (a cut mid-arm doesn't), a blow at
  neck height taking the head by its neck; a blade brushing armour scratching it;
* a stab running two through: into the first dummy's chest (deep at 0.1 m, out its back at 0.25 m,
  killing it), pressed on into the second (627 N) and deep in it;
* kicks (each foot into the chainmail dummy at 5.8 m/s, push level 2); the climb jump off the
  block (0.3 force up and along the gaze);
* telekinesis on a dummy (caught, lifted, thrown at 30 m/s); two telekinesis hands tearing its
  head off (3012 N at the neck); both hands tearing an upper arm off in a yank (3479 N); a dummy
  set alight by a fire-filled sword held in it, burned to death and charred through 10 s alight;
* the fighters (from the creature work): parries and their chains, Easy's one-at-a-time spacing,
  dodges from thrusts, a held fighter struggling, a neck grab flooring an unaware dummy and choking
  it to death, knockouts standing and limp, a punch's push on a body that's down, a wave cleared and
  the next arriving, a disarmed fighter fetching a sword and another fleeing;
* the climb area stays quiet; the arena's first wave comes at once;
* the runtime's own scenarios: every melee one passes (sword, stab, stab into the floor, knife,
  taking the sword by its pommel); the eight that need Metro 2033's extracted guns and sounds fail
  on this machine (it has none extracted), and crouch and wall_fade failed once in a full run but
  pass on their own. The unit tests pass, but for two arm IK tests and a bullet-hole picture test
  that already failed before this work.

Not checked: anything by feel in a real headset (the weight above all), sounds by ear (the runs
were silent: the new voice lines, the clash), the controllers' buzz, a heavy weapon's push on your
guard, a body thrown into another creature, the mouth grab silencing one, a held arm following
your hand on a fighter, a stagger stopping a swing, rocking and twisting a shallow blade, a
charred body tearing apart, the left hand taking a sword's pommel while the right slides (the
script's hand missed the pommel).

## Calls made (all in `framework/docs/MELEE.md`)

* Nothing snaps into the hand: your rule. Drawing from a
  holster and catching still go into the grip.
* Knockouts come above set speeds and last 6 s.
* A hand on the throat hurts (5 a second); a pulled weapon is wrenched away
  (by hand or telekinesis); fighters feint. You asked for these.
* Always on in the range: the telekinesis throw,
  telekinesis on bodies, tearing limbs off by telekinesis or by hand.
* The focus meter holds 60 (an earlier call said 30).
* Telekinesis and the melee buttons are the game's to ask for, so Oblivion keeps the gravity gloves
  and its own buttons; the range's melee areas ask for both.
* Each wave comes while you're in the arena.
* The runtime's gore stays its own (wounds bleed, blood runs and pools), with its blade blood and
  neck bleed.
* Fire and shock alternate as they're cast (the range has no spell wheel).

## Known issues

* Thrown and telekinesis-held things are judged as weapons only if they're melee weapons (a thrown
  bottle breaks, but doesn't hurt).
* A body held at full stretch on a stuck blade drops it (the stuck rule: the grip's past your
  reach).
