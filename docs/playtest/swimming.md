# Playtest: swimming

Save: rustvr-swim

## Playing it

From the repository's root:

    powershell -File tools\oblivion.ps1 -Save rustvr-swim -Commands "set gamehour to 13|fw 00038EEE"

It builds main, installs it beside the game and boots straight into the save for the headset
(the two console lines hold the hour at 1 pm and the weather clear, as the save's can turn to rain;
leave them out to take the save as it is). `powershell -File tools\oblivion.ps1 -Close` puts the
game away and your settings back.

`rustvr-swim` floats you in Lake Rumare just south of the Imperial Bridge, east of Weye, at 1 pm.
The water's a few metres deep, with a big rock and the bridge to the north (the shallows to stand
up in) and open water to the east and south.

It's the developer's own test save, not in the repository (`docs/DEVELOPING.md` has the test
saves). To make one from any save, boot it, put the player in deep water and save under that name
with console lines, e.g. from `rustvr-weye` (Weye, on the lake's shore):
`powershell -File tools\oblivion.ps1 -Save rustvr-weye -Commands "player.setpos x <x>|player.setpos
y <y>|SaveGame rustvr-swim"`, with a point out in the lake, then `-Close`.

The research and the design are in `docs/research/swimming.md`; how it's wired into the game is
the Swimming row of `docs/OBLIVION.md`. The numbers to tune are at the top of
`framework/crates/runtime/src/swim.rs` (the catch, the pull's pace, the glide and drag, the most
speed) and `framework/crates/ik/src/swim.rs` (the body's lean and kicks).

## What it does

- **The game decides when you swim, and your breath.** The adapter (`crates/oblivion/src/swim.rs`)
  sends the runtime the cell's water height, whether the game has the player swimming (70% under)
  and their head under (87.5%), breath against its maximum for the player's Endurance, Water
  Walking, the water's colour and fog, and the game's swim speed (its running pace with Always Run
  or Run held, else its walking pace). Water Breathing and god mode count as full breath; Water
  Walking turns the arms' swimming off.
- **Catch the water and pull yourself past it.** A hand under the surface (its palm 1 cm down),
  holding nothing, catches the water with the grip squeezed past 0.6 and lets go below 0.35. The
  hands holding the water make one stroke: their velocities (against the head) summed over two, so
  one hand alone pulls half as hard. It carries you the other way at 3 game swim speeds with the
  hands at 1 m/s, growing with their speed to the 0.4th power (1.9 at 0.3 m/s, 4 at 2 m/s), times
  the Stroke strength setting.
- **A stroke counts once it's 10 cm from the point of it the hands are furthest from**, and only
  while it goes on away from there: a grip squeezed before the reach ends, or let go after the
  hands start back, slips.
- **The glide.** Gliding (faster than 1 m/s), a stroke never takes your speed: it speeds the glide
  up to its pace along it (a slower one slips) and turns it its way. Only a push back against the
  glide (more than 120 degrees from it) slows you and takes you back, and only while the hands have
  gone that way at least half as far as they've moved apart or together, so a breaststroke's sweep
  out never does. The water's drag glides you out by e over 6 s, sooner when fast: from 5 m/s
  you're at 3.8 a second on, 2.3 three seconds on. Nothing goes past 4 times the game's swim speed.
- **At the surface** the game holds you up: a stroke's rise goes into the surface, and a stroke has
  to take you down faster than 0.2 m/s to dive.
- **Strokes go where the hands push, the stick where the head looks.** The strokes are the game's
  swimming step in `Actor::Move` (`crates/oblivion/src/movement.rs`), turned back by the pitch the
  game's swimming turns a step by; the stick's swimming is added while it's pushed. The game's
  character controller moves you, with its collision and its shore.
- **The game sees a swimmer.** While the arms swim (faster than 0.15 m/s) with the stick let go,
  the game's joystick is nudged the way the strokes go, as little as it reads
  (`crates/oblivion/src/player.rs`): it plays its swimming and trains Athletics.
- **The body** floats from the head: treading water nearly upright (legs hanging, thighs forward,
  an eggbeater kick about 0.9 a second), leaning forward up to 62 degrees when swimming ahead at
  full speed with a flutter kick (1.7 a second), a little back when backing off.
- **Under the water or above it.** The sky's underwater fog is decided where the headset's eyes are
  (the game's sky update hooked), not the game's eye half a metre above the water, so a face ducked
  in gets it. The body under the surface shows through it, tinted by the water's colour over its
  fog distance. The breath meter is a HUD piece where the game has it, at the top of the view.
- **Feel.** A pulse when a hand catches the water; a buzz while it pulls faster than 0.4 m/s; a
  pulse when a hand slaps in faster than 0.8 m/s. Under the water with under 30% breath left, the
  heart beats in both hands, every 1.1 s down to 0.5 s, harder as it runs out.
- **Splashes.** A hand going into the water faster than 1.2 m/s gets the game's splash for a thing
  falling in: `CWaterSmall` and its particles, `CWaterMedium` and a bigger splash over 3 m/s, louder
  the harder.
- **Ducking while swimming isn't crouching**: the physical crouch (sneak) is off while the game has
  you swimming.
- **A jumps** on its press, so at the surface it's the game's jump out of the water.
- The comfort vignette closes in while the arms swim you, as it does for the stick.

## What to try

1. **Float.** Look around and down: eyes about half a metre above the water, nothing pulling you up
   or down, the body treading water. Put your hands in the water: you see them under the surface,
   a little of the water's colour over them.
2. **Catch and pull.** Put a hand under the water and squeeze its grip (a pulse), pull, let go,
   reach out again with an open hand, catch and pull again. Between strokes you glide. Light strokes
   swim well: at the surface a light breaststroke is about 1.7 times the game's swim speed, a
   relaxed one 2.6, a firm one 3.
3. **Any stroke.** A breaststroke (both hands), a crawl (one hand at a time: half as hard), a dog
   paddle.
4. **Any direction.** Pull back to go ahead, pull up to dive, push down to rise, to a side to go
   that way, wherever you look. Strokes never turn your view: turn your body or use the stick.
   Walking about the room pulls nothing.
5. **Stop and go back.** While gliding, push both hands forward together: you stop and go back.
6. **Tread water.** Let go of the grips: you stay where you are at the surface.
7. **Dive** (pull up, or look down and push the left stick). Under: the game's fog and tint on the
   world, your hands and body; the breath meter at the top of the view.
8. **Hold your breath.** With the save's Endurance it lasts about 20 seconds (the game's
   `fActorSwimBreath*`). Low, the heart beats in your hands and the game gasps. **Don't drown** (see
   below).
9. **Surface** (push down), and **duck your face** into the water from the surface: the underwater
   fog comes on as your eyes go under.
10. **Slap the water**, gently and hard: the game's splash and sound, a pulse in that hand.
11. **The stick** swims as in the flat game, where your head looks; strokes add to it.
12. **Get out.** Swim north to the rocks under the bridge and stand up in the shallows, or press A
    at a ledge to jump out.
13. Holding something (a weapon, a torch), that hand doesn't catch the water.

## Settings

In the runtime's menu (the menu button), **Controls** tab (`[input]` in
`%LOCALAPPDATA%\RustVR\settings.toml`):

| Setting | Key | Default | Range |
|---|---|---|---|
| Swim with your arms (in the game's water) | `arm_swimming` | on | off: only the stick swims |
| Stroke strength | `swim_ease` | 1.0x | 0.5 to 3.0x in the menu (above 1, every pull carries you faster) |

The **Comfort** tab's "Vignette while moving" (`[comfort] vignette`, 0.5 by default, 0 to 1) also
closes in while the arms swim you.

## Decisions

- **The game decides when you swim and your breath, and its speed scales the strokes.** The
  runtime only turns your hands into the velocity the game's own swimming moves you at, so its
  character controller, surface, collision and Athletics are the game's. Strokes may go past the
  game's speed (asked for in playtest 2), up to four times it, but scale with it: skills, load,
  running or walking pace.
- **Catch the water and pull yourself past it** (since playtest 5), as Freediver: Triton Down does.
  Builds 1 to 4 made the water push on every hand as real water does (palms, the head, the stroke's
  way, plain physics, fingertips slicing). A controller can't show a palm catching the water and
  slicing back through it, so the recovery always held you back and strokes felt mushy.
- **One stroke from both hands** (playtest 8), summed over two (playtest 9), so a breaststroke's
  sweeps out and in cancel and one hand catching a moment early doesn't swerve you.
- **A stroke pulls only along its own way** (playtest 7), so an early grip or a late release
  doesn't throw you backwards.
- **A stroke never takes the glide's speed** (playtest 9), except a push back against it that's
  more push than sweep (playtest 11): the next breaststroke's sweep out, a little forward, no longer
  slows you, and going back takes one push.
- **Light strokes swim well** (playtest 10): the pace grows with the hands' speed to the 0.4th
  power, so you needn't push hard.
- **No buoyancy, no auto-surfacing.** Oblivion has none under water: a still swimmer hangs where
  they are, and nothing moves your view but you. When breath runs low you're warned (the meter, the
  gasp, the heartbeat), and surfacing is yours.
- **Ducking while swimming isn't crouching.** The game's swimmer isn't crouched, so a crouch pressed
  in the water would still be on at the shore.
- **The fog follows the headset's eyes**, not the game's eye floating above the water. The per-eye
  water shading was already the game's own (it's decided inside each eye's render).
- **The body under the water** shows through the surface the game drew, tinted by how far light
  went through the water; the surface's waves and reflections over it aren't drawn.
- **Splashes are the game's** for a thing falling into water. The game makes surface ripples only
  for wading actors (the player's own while swimming), so hands get none of their own.

## Not done / known issues

- **Dying crashed Oblivion in VR** when the game reloaded the last save (while the game made the
  save's picture), on any save, so not swimming's; seen while swimming was built and not rechecked
  on main since. Drowning kills: don't hold your breath to the end.
- **A crash on launch while loading a save from the main menu** (in the game's load) was seen with
  several saves, intermittently, before swimming ran. The other launch crash of playtest 6, the game
  resetting a device lost as the headset connected, is the one riding's round 10 addressed (a lost
  device is reset between the game's frames, `crates/oblivion/src/d3d9.rs`); not rechecked with
  swimming since.
- **Checked only as code:** Water Walking, Water Breathing, dangerous water, swimming indoors.
- **Not seen in a picture:** the floating legs (checked as stick figures and numbers only); a held
  weapon under the water seen from above (what the runtime draws gets the body's look through the
  surface only if it's drawn with the body's shading).
- **Not heard:** the game's swimming sounds while the arms swim.

## History

Newest first; each a run on the headset.

- **Playtest 11.** "Backwards is way too hard to trigger": gliding, a push back had to be 20 cm long
  and only stopped you. Now it counts from 10 cm like any stroke, if it's more push than sweep.
- **Playtest 10.** Light strokes needed far too much push; backwards barely worked. The pace now
  grows with the hands' speed to the 0.4th power; strokes count from 10 cm (were 15).
- **Playtest 9.** "Nice, it works. Faster, and go for longer", but gripping for the next stroke
  killed speed. A stroke no longer takes the glide's speed; the pull is stronger and the glide lasts
  6 s (was 4); one hand pulls half.
- **Playtest 8.** Perfect breaststrokes went nowhere and drifted sideways: each hand's way was
  measured from its catch, so the sweep out hid the pull. Both hands now make one stroke, and each
  pull steers the glide its way.
- **Playtest 7.** "A perfect breaststroke forward pulls me back and forth and to the side": hands
  still gripping as they moved forward pulled backwards. A stroke now pulls only along its own way;
  at the surface its rise goes into the surface.
- **Playtest 6.** Two crashes on launch, before swimming ran, neither swimming's (see above).
- **Playtest 5.** "Still just terrible... research proper good swimming in VR". The well-liked VR
  swimmers pull the player past an anchored hand (Freediver: Triton Down, Marine Life Exploration),
  so: squeeze the grip to catch the water, pull, let go and glide.
- **Playtest 4.** Breaststrokes "all scuffed, I barely move": reaching forward pushed back almost as
  hard as the pull. A hand sliced through the water fingertips first; the glide was doubled; a hand
  coming back into tracking no longer launched you.
- **Playtest 3.** Build 3 was "vastly overengineered": rebuilt as only the physics of hands in
  water.
- **Playtest 2.** Forward but weak, separate horizontal and vertical modes, drifting after turns;
  asked for much faster, precise strokes, steered by the arms alone.
- **Playtest 1.** Breaststrokes and crawls drifted backwards: a hand pushed only while its palm led.
  "It should be the way your hand moves, not the palms."
