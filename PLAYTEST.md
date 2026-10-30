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
2. **Breaststroke.** Hands together in front, palms turned out and back, sweep them out and back
   firmly, then bring them forward again with the palms down or edge on (feathered).
   - Each stroke surges you on, then you glide, slowing over a second or so.
   - A steady firm stroke reaches the game's own full swim speed. A gentle one gets about half.
     Nothing goes faster than the game's speed for your Speed, Athletics, armour and load.
   - Each pushing hand buzzes with the water's resistance.
   - Your body leans forward as you swim, legs streaming out behind with a flutter kick. You can
     see this when you're under the water.
3. **Other strokes.** Any stroke works: a crawl (one arm at a time, palm back), a dog paddle,
   side strokes. It's how your palm meets the water that pushes, not a gesture to match. A hand
   coming back edge-on or back of the hand first pushes nothing.
4. **Turning.** Strokes never turn you. Turn your body, or use the right stick as on land. Turning
   on the spot, or walking about the room, pushes nothing.
5. **Treading water.** Small sculling movements, or nothing at all: you stay where you are at the
   surface.
6. **Dive.** Palms up, push up: down you go. You can also look down and push the left stick. With
   your head under:
   - the game's underwater fog and tint (your hands and body get them too);
   - the game's breath meter at the top of the view.
7. **Hold your breath.** The meter drains. With the save's Endurance it lasts about 20 seconds;
   that's the game's `fActorSwimBreath*` and your Endurance. Under a third left, your heart beats
   in both hands, faster and harder as it runs out, and the game gasps.
   **Don't drown:** dying and the game's reload crash Oblivion in VR (see "Known issues").
8. **Surface.** Palms down, push down: up you go. The game stops you at its surface.
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
    - "Stroke ease": above 1, a smaller stroke gets you to full speed.

Hands holding something (a weapon, a torch, a thing you've picked up) don't stroke. Neither does a
fist (the grip squeezed): the other hand still swims.

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
- **Diving** with palms-up pushes reached 1.5 m/s down. Surfacing with palms-down pushes, 1.7 m/s
  up.
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
  paddle, glide, full push) and `framework/crates/ik/src/swim.rs` (the body's lean and kicks).
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

- **The game decides when you swim, how fast, and your breath.** The runtime only turns strokes
  into the velocity the game's own swimming moves you at. That's its character controller, with
  its collision, its surface and its Athletics. There are no hand-tuned speeds:
  - a full steady stroke holds the game's full swim speed (its running pace with Always Run, or Run
    held, its walking pace otherwise);
  - a gentle stroke gets less.
- **The paddle model.**
  - A hand pushes when it's under the surface, empty, open, and leading with its palm. How much
    goes with how squarely the palm meets its motion.
  - The push is the hand's velocity against the head, reversed, past a 0.25 m/s dead zone.
  - Strokes accelerate you and the water's drag glides it out over 1.4 s.
  - At the surface a stroke has to take you down faster than 0.2 m/s to dive. Every stroke has a
    little up and down in it, and the game floats a swimmer back up only once they stop: before
    this, a long swim at the surface sank you a couple of centimetres a second. Now you stay within
    about 5 cm of the game's float.
- **Ducking while swimming isn't crouching.** The game's swimmer isn't crouched, so a crouch pressed
  in the water would still be on at the shore. Ducking your face into the water just puts it there.
- **The comfort vignette** (if it's on) closes in while the arms swim you, as it does for the stick.
- **No buoyancy, no auto-surfacing.** Oblivion has none under water: a still swimmer hangs where
  they are. That's also the comfortable choice, since nothing moves your view but you. When breath
  runs low you're warned (the meter, the gasp, the heartbeat in the hands), and surfacing is yours.
- **Strokes go where the hands push, the stick where the head looks.** The game turns any
  swimming step down by the player's pitch (the head's). The strokes' step is turned back first,
  so looking down while stroking forward doesn't dive.
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
