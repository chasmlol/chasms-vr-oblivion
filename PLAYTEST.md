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
2. **Catch the water and pull.** Put a hand under the water and squeeze its grip: that hand holds
   the water (a little pulse says so). Pull it and you're pulled past it the other way, about
   four times as fast as your hand goes. Let go, reach out again (an open hand goes through the
   water freely), catch and pull again. Between strokes you glide.
   - At the surface a relaxed breaststroke (hands wide out, back and in over most of a second)
     swims about 2.3 times the game's swim speed, a firm one 3 times. One firm stroke from still
     carries you about 17 m.
   - Both hands make one stroke together: their sweeps out and in cancel, and their pull back
     carries you. One hand alone pulls half as hard. The grip needn't be timed exactly: the stroke
     pulls only while your hands go on away from where it started (or turned back), once they're
     15 cm from there. Squeeze a little before the reach ends, or let go a little after your hands
     start back, and it just slips.
   - The next stroke never takes your glide's speed: it only adds to it, and turns it its way, so
     uneven hands don't leave you drifting sideways. Only a stroke back against your glide, 25 cm
     or longer, slows you: that's how to stop.
   - Pulling down as well as back takes you up as well as ahead (at the surface the game holds
     you there): to swim level under the water, pull straight back.
   - A pull slower than you already go just slips.
   - A hand holding the water buzzes as it pulls (not as it slips).
   - Your body leans forward as you swim, legs streaming out behind with a flutter kick. You can
     see this when you're under the water.
3. **Any stroke.** A breaststroke (both hands), a crawl (one hand at a time, letting go as it comes
   round: half as hard as both hands), a dog paddle: squeeze for the pull, let go to reach.
4. **Any direction.** Pull back and you go forward, pull up and you dive, push down and you rise,
   pull to a side and you go that way: where your hands pull, wherever you look. Strokes never turn
   your view: turn your body, or use the right stick as on land. Your glide lasts several seconds:
   from 5 m/s you're still at 3.8 a second on, 2.3 three seconds on.
   Walking about the room pulls nothing.
5. **Treading water.** Let go of the grips, or keep your hands still: you stay where you are at the
   surface.
6. **Dive.** Catch the water and pull it up: you go down. You can also look down and push the left
   stick. At the surface a stroke has to take you down faster than 0.2 m/s to dive, so a stroke's
   wobble doesn't. With your head under:
   - the game's underwater fog and tint (your hands and body get them too);
   - the game's breath meter at the top of the view.
7. **Hold your breath.** The meter drains. With the save's Endurance it lasts about 20 seconds;
   that's the game's `fActorSwimBreath*` and your Endurance. Under a third left, your heart beats
   in both hands, faster and harder as it runs out, and the game gasps.
   **Don't drown:** dying and the game's reload crash Oblivion in VR (see "Known issues").
8. **Surface.** Catch the water and push it down: you come up. The game stops you at its surface.
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
    - "Stroke strength": above 1, every pull carries you faster.

Hands holding something (a weapon, a torch, a thing you've picked up) don't catch the water.

## Playtests on the headset

1. **Breaststrokes and crawls drifted you slowly backwards.** A hand only pushed while its palm led,
   and a hand round a controller faces the other hand. "It should be the way your hand moves, not
   the palms."
2. **It went forward but weakly** (30 to 90% of the game's speed), felt like separate horizontal and
   vertical modes, and drifted on after turning. You asked for much faster strokes, precise from
   small to hard. Build 2 steered by where the head looked; before trying it you asked for the arms
   alone to steer, and build 3 worked out a stroke's way from its strongest part.
3. **Build 3 was "vastly overengineered and works like shit".** Its log shows the speed swinging
   between directions second to second. "Start again: no breaststroke or crawl, just the physics of
   your hands, exactly like real life. Pushing my hands fast pushes me; you go back slowly and then
   push hard again."
   - Now it's only that: each hand in the water pushes you the other way to its motion, as hard as
     the square of its speed, and the water's drag glides your speed out. Everything else from
     builds 1 to 3 (palms, the head, the recovery and the strokes' way) is gone.
   - Simulated: pushing at 1.2 m/s for a third of a second and coming back at 0.4 surges to
     3.2 m/s and glides to 0.9 each stroke, 8 m in 4.6 s; a single push at 1 m/s carries you
     1.9 m; pushing and coming back at one speed goes nowhere.
4. **Up and down made sense, but breaststrokes were "all scuffed, I barely move. I should be
   breaststroking and gliding through the water."** Your strokes, from the log: pulls back and out
   at about 1.2 m/s, then in, then forward again at about 0.6 m/s. Reaching forward pushed you
   backwards almost as hard as the pull pushed you on: simulated, your stroke surged to 2.4 m/s,
   fell back to -0.6 (backwards) and averaged 0.7.
   - A hand now slices through the water fingertips first, as a real one reaching forward does:
     only the part of its motion across the hand meets the water fully. A pull back (wrist first),
     a push down or up, and a sweep to the side are as before.
   - The glide is twice as long: from 3 m/s you glide about 6 m.
   - Simulated, your stroke now surges to 4.1 m/s, glides to 1.5 and averages 2.7 (1.3 times the
     game's speed); quicker strokes average 4.6. With your hands pointing 35 degrees off the way
     they reach, it's 2.4.
   - A hand coming back into tracking jumped from where it was lost and launched you (twice in the
     log: up at 4.4 m/s, and to four times the game's speed). Now only a hand tracked both frames
     moves, and a jump faster than any arm counts for nothing.
5. **"Still just terrible, so unnatural, and the speed is shit: I need more speed, and to glide
   more. Research proper good swimming in VR and implement it."** (This run's log was replaced by
   the next feature's playtest before it was kept.)
   - The well-liked VR swimmers don't simulate water pushing on a hand. Freediver: Triton Down
     (grown from Lone Echo's zero-gravity movement) has you pull the trigger and swing your arms:
     "you essentially pull yourself through the world". Marine Life Exploration: hold the side
     buttons, pull back, let go. Hubris reads the palms, which a controller can't show (build 1).
     Unreal developers' advice: anchor an open hand in the water like a climbing hold, and use a
     button where the hand can't say.
   - So now: squeeze the grip and that hand holds the water; you're pulled past it, at about
     three times its pace (the game's swim speed's worth per 0.65 m/s of hand). Let go and you
     glide (by e over 4 s); the hand reaches out again freely. A pull never slows you.
   - Simulated: relaxed breaststrokes 3.3 m/s (1.6 times the game's speed), firm ones 4.8, hard
     ones 6.4; a firm stroke carries you 16 m.
6. **Crashed on launch, twice,** before swimming ran: both crashes are other features' too (see
   "Known issues").
7. **"I like this new approach, but a perfect breaststroke forward pulls me back and forth and
   to the side."** The log shows the grip still squeezed while your hands moved forward: squeezed
   as the reach ended, or let go only after they'd started back. Those hands pulled you backwards:
   at 13:34:43, hands going forward at 0.46 m/s took you from 2.3 m/s to 0.9; at 13:34:46, at
   0.67 m/s, threw you backwards at 1.9. Turning back from that left a sideways drift. At the
   surface your pulls (down as well as back) also stored up a rise the game held you from, which
   slowed the glide.
   - Now a stroke pulls only while the hand goes on away from where that stroke started (or
     turned back), once it's 10 cm from there: back over its own stroke, a hand slips.
   - At the surface a stroke's rise goes into the surface, not into the glide.
   - Simulated, your stroke with the grip squeezed early and let go late: 2.5 m/s (1.2 times the
     game's speed), never backwards; before, 1.0 with a dip backwards every stroke. Timed well,
     3.1 relaxed and 4.2 firm (1.5 and 2 times).
8. **"A load of perfect breaststrokes, perfect grip and form: I didn't move, just slowly drifted
   sideways."** (Log: `<builds>\swimming\playtest-5`.)
   - Your hands sweep wide out first, then back and in. Each hand's stroke was measured from where
     it caught the water, so after the sweep out its way pointed mostly sideways, and the pull back
     and in counted for little: at 13:44:20 hands going back at 0.5 m/s pulled at 0.02. You swam
     at half the game's speed.
   - Sideways: the strokes only ever added speed their way, so any sideways bit glided on for
     seconds and built up. You drifted about 20 degrees left of where you looked, with your hands
     even.
   - Down: your first catch, at 13:44:13, came with your hands going up, which dove you; nothing
     brought you back up, and you sank to 11 m.
   - Now both hands make one stroke (their mean), so a breaststroke's sweeps out and in cancel and
     its pull back carries you, however the grip is timed. Each pull steers your glide its way.
   - Simulated, your wide stroke, gripped early and let go late: 1.5 times the game's speed at the
     surface (2 times firm); before, 0.6 to 1.0. Uneven hands (one sweeping a quarter wider) drift
     about 6 degrees, not on and on.
9. **"Nice, it works. Faster, and go for longer. And gripping for the next stroke while moving
   kills some of my speed at the start."** (This run's log was replaced by the next feature's
   playtest before it was kept.)
   - The sweep out of a breaststroke goes a little forward (0.6 m/s in your playtest 4 log), and
     held, that's a stroke backwards: simulated, your strokes fell from 4.2 m/s nearly to a stop at
     each catch. And the steering took away whatever went across a stroke's early way, your glide
     included.
   - Now a stroke never takes your glide's speed: it only adds to it along the glide and turns it
     its way. Only a stroke back against the glide, 25 cm or more, slows you. A stroke counts from
     15 cm (10 before), so the sweep out's bit forward never does, from still either.
   - Faster: the pull takes you 2 game swim speeds for each m/s of your hands (1.5 before). Longer:
     the glide fades by e over 6 s (4 before), and less when fast.
   - One hand alone pulls half as hard (both hands' velocities' sum over two): a hand catching a
     moment before the other no longer swerves you.
   - Simulated, your wide stroke, gripped early and let go late, one hand a moment before the
     other: relaxed 2.3 times the game's speed, firm 3 (1.3 and 1.8 before); speed lost at the
     catch beyond the glide's own: none (most of it before); a firm stroke from still carries
     17 m. Strokes back against the glide stop you in about half a second and take you back.

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

  The numbers to tune are at the top of `framework/crates/runtime/src/swim.rs` (the catch, the
  pull's pace, the glide and drag, the most speed) and
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

- **The game decides when you swim and your breath, and its speed scales the strokes.** The
  runtime only turns your hands into the velocity the game's own swimming moves you at. That's its
  character controller, with its collision, its surface and its Athletics. Your strokes may go past
  the game's speed (you asked for it in playtest 2), up to four times it, but they scale with it:
  its running pace with Always Run, or Run held, its walking pace otherwise, and your skills and
  load.
- **Catch the water and pull yourself past it** (since playtest 5), as in Freediver: Triton Down.
  - A hand catches the water under the surface (1 cm down) with its grip squeezed past 0.6, holding
    nothing else, and lets go below 0.35. Its velocity is against the head.
  - The hands holding the water make one stroke: their velocities' sum over two (one stroke since
    playtest 8, the sum since playtest 9, so one hand alone pulls half). It takes you the other way
    at 2 game swim speeds for each m/s of its pace, in about 0.08 s.
  - It pulls only while it goes on away from the point of the stroke it's furthest from, once
    15 cm from it (since playtest 7), so a grip squeezed early or let go late doesn't throw you
    backwards, and a breaststroke's sweeps out and in cancel.
  - Gliding (faster than 1 m/s), a stroke never takes your speed (since playtest 9): it speeds you
    up to its pace along your glide (a slower one slips), and turns your glide its way (by e in
    0.25 s, as fast as it's strong against your speed), as a swimmer's streamlined body follows
    the stroke. Only a stroke back against the glide (more than 120 degrees from it), once 25 cm
    long, slows you. Nearly still, a stroke takes you its way at its pace.
  - At the surface (your head out) a stroke's rise is lost into the surface, as the game holds
    you there.
  - The water's drag: your speed glides out by e over 6 s, and sooner when fast (0.025 more for
    each m/s).
  - At the surface a stroke has to take you down faster than 0.2 m/s to dive: every stroke has a
    little up and down in it, and the game floats a swimmer back up only once they stop.
  - Before: builds 1 to 5 made the water push on every hand as real water does (palms, then the
    head, then the strokes' way, then plain physics, then fingertips slicing). A controller can't
    show a palm catching the water and slicing back through it, so the recovery always held you
    back and the strokes felt mushy.
- **Ducking while swimming isn't crouching.** The game's swimmer isn't crouched, so a crouch pressed
  in the water would still be on at the shore. Ducking your face into the water just puts it there.
- **The comfort vignette** (if it's on) closes in while the arms swim you, as it does for the stick.
- **No buoyancy, no auto-surfacing.** Oblivion has none under water: a still swimmer hangs where
  they are. That's also the comfortable choice, since nothing moves your view but you. When breath
  runs low you're warned (the meter, the gasp, the heartbeat in the hands), and surfacing is yours.
- **Strokes go where the hands push, the stick where the head looks.** The game turns any swimming
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
- **Two crashes on launch** (playtest 6, before swimming ran), neither swimming's: both happen with
  other features' builds too.
  - Direct3D lost its device when the runtime connected, 30 s into the game (the headset wasn't
    streaming yet), and the game crashed resetting it (an access violation in the game's own code). The
    riding build crashed the same way at 11:23. Put the headset on, with Virtual Desktop
    connected, before running the .bat.
  - Loading `rustvr-swim` straight from the main menu crashed (in the game's own load code), as
    loading `rustvr-weye` and `rustvr-books` did this morning
    (in the same code). It's intermittent: run the .bat again.
- **Held things under the water** (a drawn weapon in the hand), seen from above, weren't checked.
  The things the game draws in the hand are under its own water. What the runtime draws gets the
  body's look through the surface only if it's drawn with the body's shading.
- **The breath meter is the game's size,** small in the view. A bigger HUD is the floating menus'
  work (section 2).
