# Playtest: swimming

Save: rustvr-swim

`powershell -File tools\play.ps1 swimming` boots Oblivion into the save `rustvr-swim`: you're
floating in Lake Rumare just south of the Imperial Bridge, east of Weye, at 1 pm (the weather may
turn to rain). The water's a few metres deep, with a big rock and the bridge to the north (the
shallows to stand up in) and open water to the east and south. When you're done,
`powershell -File tools\play.ps1 -Close` puts the game away.

The research and the design are in `docs/research/swimming.md`; how it's wired into the game is
the Swimming row of `docs/OBLIVION.md`.

## What to try, in order

1. **Float.** Look around and down. The game holds you at its surface, eyes about half a metre
   above the water, as it does any swimmer. Nothing pulls you up or down. Your body treads water
   under you: legs hanging, thighs a little forward, kicking slowly. Put your hands in the water and
   look at them: you see them under the surface, a little of the water's colour over them.
2. **Breaststroke.** Hands together in front, sweep them out and back, then bring them forward
   again. Which way your palms face doesn't matter, nor where you look: only which way your arms
   push.
   - How fast your hands move is how fast you go: a small slow stroke moves you a little (about
     half a metre), a hard fast one shoots you along. A firm one surges to about two and a half
     times the game's swim speed, and between strokes you glide at about its own. Your Speed,
     Athletics, armour and load still count: the game's speed is the strokes' scale.
   - Reaching forward again (the recovery) holds you back only a little, even if it's as quick as
     your pull: the stroke's stronger part sets your way.
   - Each pushing hand buzzes with the water's resistance.
   - Your body leans forward as you swim, legs streaming out behind with a flutter kick. You can
     see this when you're under the water.
3. **Other strokes.** Any stroke works: a crawl (one arm at a time), a dog paddle, side strokes.
   You go the other way to your hands, not a gesture to match.
4. **Steering with your arms.** Breaststroke any way at all: down (reach down, pull up to your
   chest) and you go down while looking ahead, up and you come up, to a side and you go sideways.
   Strokes never turn your view: turn your body, or use the right stick as on land.
   - Turn and stroke a new way: your hands in the water stop the old way's glide, so you don't
     drift off sideways.
   - To go back the way you came, stroke that way a little harder than before, or pause a moment
     first.
   - Looking about, or walking about the room, pushes nothing.
5. **Treading water.** Small sculling movements, or nothing at all: you stay where you are at the
   surface.
6. **Dive.** Breaststroke downwards, or push your hands up: down you go. You can also look down
   and push the left stick. At the surface a stroke has to go a little down (about 7 degrees) to
   dive, so a stroke's wobble doesn't. With
   your head under:
   - the game's underwater fog and tint (your hands and body get them too);
   - the game's breath meter at the top of the view.
7. **Hold your breath.** The meter drains. With the save's Endurance it lasts about 20 seconds;
   that's the game's `fActorSwimBreath*` and your Endurance. Under a third left, your heart beats
   in both hands, faster and harder as it runs out, and the game gasps.
   **Don't drown:** dying and the game's reload crash Oblivion in VR (see "Known issues").
8. **Surface.** Breaststroke upwards, or push your hands down: up you go. The game stops you at its
   surface.
9. **Duck your face into the water** from the surface: the underwater fog comes on when your eyes
   go under, even though the game still has you floating.
10. **Slap the water.** Bring a hand down into the water fast: the game's own splash (its droplets
    and its `CWaterSmall` sound), bigger and louder when you slap hard. Each slap gives a pulse in
    that hand.
11. **The stick.** Push the left stick: you swim as in the flat game, where your head looks (look
    down to dive). Your strokes add to it.
12. **Getting out.** Swim north to the rocks under the bridge and stand up in the shallows. Or swim
    to a ledge and press A to jump out of the water: that's the game's jump from the surface.
13. **Athletics.** The game trains Athletics for every second you swim, strokes or stick, as it
    does in the flat game.
14. **Settings.** In the runtime's menu, Controls tab:
    - "Swim with your arms": turn it off and only the stick swims.
    - "Stroke ease": above 1, every stroke is stronger (a smaller one goes as far).

Hands holding something (a weapon, a torch, a thing you've picked up) don't stroke. Neither does a
fist (the grip squeezed): the other hand still swims.

## Playtests on the headset

1. **Breaststrokes and crawls drifted you slowly backwards**, about a fifth of the game's speed;
   pushing down to rise and up to dive felt good. "It should be the way your hand moves, not the
   palms: you're holding a controller."
   - Why: a hand only pushed while its palm led the motion. A hand round a controller's handle faces
     the other hand, so the sweep back went edge-on and pushed nothing. Bringing the hands together
     and forward led with the palms, pushing you out (the two hands cancelling) and back. The game's
     log showed the strokes' step going backwards while both hands pulled back at 1.1 m/s.
   - Now: a hand pushes against whichever way it moves, palms or not. Reaching ahead (the
     recovery) pushes back only a fifth as much. Each stroke surges harder and glides out more
     slowly (two seconds). Looking about no longer pushes (it was counted as the hands sweeping
     round the head).
2. **Forward worked, but weak:** 30 to 90% of the game's speed. "WAY faster, it can go over the
   game's speed." It felt like two modes, horizontal and vertical, where you should breaststroke
   anywhere and choose up or down by looking. Stroking one way, then turning and stroking another,
   drifted you on the first way. Strokes should be precise: small ones for little moves, hard fast
   ones to shoot along.
   - Now each hand meets the water as a hand does: the faster it goes through the water, the harder
     it pushes (with the square of its speed), and you go the other way. How fast your hands move
     is how fast you can go, scaled by the game's swim speed so its skills still count, up to four
     times it. Your pull back drives you where you look, up or down. Going that way your hands slice
     through; any other way they hold you back, which stops the old way's drift after a turn.
   - The numbers, simulated before the build: a firm breaststroke (hands at 1.2 m/s) surges to
     5.4 m/s and glides to 2.4 between strokes, about 10 m in 4 s; a gentle one (0.5 m/s) 1 to
     1.6 m/s; a small one (0.3 m/s for a third of a second) half a metre. Turned 90 degrees with
     the hands still, two thirds of the old glide is gone in half a second.
3. **Before trying build 2:** "Not where my head looks, the way my breaststroke goes. I can look
   forward and breaststroke downwards and go down. It's the way my arms push that determines where
   I'm going, exactly as in real life."
   - The head no longer steers at all. What a hand slices through (the recovery) can't come from the
     palm or the head now, so it comes from the strokes: their way is where they'd push you from
     still water, set by the strongest of them. A recovery, even as quick as its pull, slices
     through it; a stroke clearly harder than the recent ones the other way (1.3 times), or any
     stroke after a pause, turns you round. Across that way the hands still meet the water, which
     stops a drift after turning.
   - Your hands, from the logs: they never came closer than about 30 cm, so a "hands together"
     streamline for the recovery wouldn't have worked for you.
   - Simulated: a breaststroke in any direction, starting with the reach or the pull, surges to
     5.4 m/s; a stroke 90 degrees off the old way leaves 0.55 m of drift; equal-speed pulls and
     reaches still swim; reversing takes two strokes the other way.

## Checked in the game (emulated headset, `tools/oblivion.ps1`)

- **The game's water reaches the runtime:**
  - Lake Rumare's water at 0;
  - swimming set when the player's 70% under, the head under at 87.5%;
  - the swim speed: 143 units/s (2.05 m/s) running;
  - breath draining from 1.00 to 0.00 over about 20 s with the head under;
  - the water's colour and fog.
- **Breaststrokes while looking down 60 degrees** swam along the surface, not down (the
  strokes go where the hands push).
- **Scripted breaststrokes** moved the player about 8.6 m in 10 s, up to 93% of the game's speed,
  surging and gliding. Swimming into the big rock slid along it, the game's collision. In the
  shallows the game stood the player up.
- **Diving** with palms-up pushes (from low, the hands staying under the water) went from the
  surface to the bottom, about 4 m down, at up to 1.5 m/s. Surfacing with palms-down pushes,
  1.7 m/s up.
- **Athletics** experience rose while swimming, both with the stick (game's own) and with strokes.
- **Underwater:** the game's underwater fog on the world, the hands and body. The breath meter
  showing at the top of the view, from the game's own layout.
- **Ducked face:**
  - with both eyes under the surface, the underwater fog came on (before the fix: clear water);
  - with the head on its side, one eye above and one below, each eye's water shading was its own
    and the fog was the head's.
- **Hands under the surface,** seen from above, now show (they were hidden by the water).
- **The game's splash** where hands slapped in, at 3.9 and 7.2 m/s (droplets seen in a capture).
- **A jumps out of the water** at the surface: the player rose clear of it.
- **The heartbeat** started at breath 0.30 under the water (the log, and the emulated controllers'
  haptic calls).
- **The floating body,** as stick figures: a swimmer streaming out behind at speed, treading water
  at rest.
- **Tests:** unit tests of the crates touched pass (protocol, IK, avatar, runtime `--lib`, the
  adapter). Two IK body tests failed before this branch (`tpose_gives_the_reference_frames`,
  `elbows_point_out_and_down_for_gun_poses`) and still do.

## Not checked

- **Real arms in a real headset.** Nothing here was played with real arms:
  - how strokes feel, and whether the glide and the speed suit you;
  - the comfort of the surge and glide;
  - the buzz and heartbeat on real controllers.

  The numbers to tune are at the top of `framework/crates/runtime/src/swim.rs` (dead zone,
  recovery, steering, the hands' strength and drag, the body's drag and glide, the most speed) and
  `framework/crates/ik/src/swim.rs` (the body's lean and kicks).
- **The floating legs in a picture of the game.** The emulated views from your own eyes don't show
  your legs well. They were checked as stick figures and numbers only.
- **The game's other water,** checked only as code: Water Walking, Water Breathing,
  dangerous water, and swimming indoors.
- **The comfort vignette while arm-swimming,** and **ducking your face into the water no longer
  crouching,** changed in code only (both small): watch for the vignette if it's on in the Comfort
  tab, and check you don't come out of the water sneaking.
- **The game's swimming sounds** while the arms swim (it plays its own while it sees a swimmer):
  the emulated headset has no ears.

## Decisions

- **The game decides when you swim and your breath, and its speed is the strokes' scale.** The
  runtime only turns strokes into the velocity the game's own swimming moves you at. That's its
  character controller, with its collision, its surface and its Athletics. Your strokes may go
  past the game's speed (you asked for it in playtest 2), but they scale with it: its running pace
  with Always Run, or Run held, its walking pace otherwise, and your skills and load.
- **The stroke model** (after playtest 2, and the head out of it before playtest 3).
  - A hand meets the water when it's under the surface, empty and not gripping. Its palm doesn't
    matter: with a controller in the hand, it can't say which way a stroke sweeps.
  - Its stroke is its velocity against the head past a 0.1 m/s dead zone, made as strong as the
    game's swimmer (a hand at 0.28 m/s past it moves the water at the game's swim speed).
  - Through the water it goes at your speed plus its stroke's, and the water pushes back with the
    square of that (0.55 per metre). Along the strokes' way it slices (a tenth); any other way, it
    meets the water fully: still hands hold back a drift.
  - The strokes' way is where they'd push you from still water (the hands' speeds squared). A
    stroke half as strong as the recent strongest steers it, 1.3 times as strong turns it round,
    and the strongest fades over 3 s.
  - Your own drag is a quarter of a hand's, plus a glide of 2 s at slow speeds. Nothing passes
    four times the game's speed. The numbers are at the top of `framework/crates/runtime/src/swim.rs`.
  - At the surface a stroke has to take you down faster than 0.1 m/s to dive. Every stroke has a
    little up and down in it, and the game floats a swimmer back up only once they stop: before
    this, a long swim at the surface sank you a couple of centimetres a second. Now you stay within
    about 5 cm of the game's float.
- **Ducking while swimming isn't crouching.** The game's swimmer isn't crouched, so a crouch pressed
  in the water would still be on at the shore. Ducking your face into the water just puts it there.
- **The comfort vignette** (if it's on) closes in while the arms swim you, as it does for the stick.
- **No buoyancy, no auto-surfacing.** Oblivion has none under water: a still swimmer hangs where
  they are. That's also the comfortable choice, since nothing moves your view but you. When breath
  runs low you're warned (the meter, the gasp, the heartbeat in the hands), and surfacing is yours.
- **Strokes go where the arms push, the stick where the head looks.** (Between playtests 2 and 3
  the strokes went where the head looked; you wanted the arms alone.) The game turns any swimming
  step down by the player's pitch (the head's). The strokes' step is turned back first, so looking
  down while stroking forward doesn't dive.
- **The game sees a swimmer.** While the arms swim with the stick let go, the game's joystick is
  nudged, as little as it reads, the way the strokes go. That gives the game its movement flags, so
  it trains Athletics, makes its wading ripples and plays its swimming. The step itself is the
  strokes'.
- **Under the water or above it.** The fog follows the headset's eyes, not the game's eye, which
  floats half a metre above the water. The per-eye shading was already the game's own (it decides it
  inside each eye's render).
- **The body under the water,** seen from above:
  - it shows through the surface the game drew;
  - the water's own colour goes over it by how far the light went through the water;
  - the reach is the water's fog distance, so under Lake Rumare's clear water it's only slightly
    tinted;
  - the surface's waves and reflections over it are not drawn.
- **Splashes** are the game's splash for a thing falling into water (its sound and particles). The
  game makes surface ripples only for wading actors (the player's own while swimming), so hands get
  none of their own.
- **A jumps again.** The player rig holds Jump while A is down, and the input bridge only jumped on
  counted presses: A didn't jump at all. It now jumps on A's press. That fix is in `input.rs`; it
  matters on land too.
- **Protocol version 12.** `GameState::water` and `Intents::swim` were added. The other overnight
  branches also bump to 12: merging needs one 12, with all the new fields.

## Known issues

- **Dying crashes Oblivion in VR** when the game reloads the last save (`access violation at eip
  ` in the game's code, creating the save's picture). It happens the same way with `player.kill` on
  `rustvr-weye` with no water, so it isn't swimming's. But drowning kills, so don't hold your
  breath to the end.
- **Held things under the water** (a drawn weapon in the hand), seen from above, weren't checked.
  The things the game draws in the hand are under its own water. What the runtime draws gets the
  body's look through the surface only if it's drawn with the body's shading.
- **The breath meter is the game's size,** small in the view. A bigger HUD is the floating menus'
  work (section 2).
