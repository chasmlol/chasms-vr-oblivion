# Swimming

Research for swimming in VR: stroking with the arms, diving and surfacing, with Oblivion's own
water, breath and swim speeds. What players like and dislike in the VR games that do it, how those
games do it, Oblivion's own swimming as it plays, and the design built from all of that.
Inspiration only: nothing below is copied.

## What players say

**Liked.**

* **Swimming that feels physical, and simple.** Subside's swimming is called "the most polished
  swimming in VR. It's easy and intuitive. It just works", and "extremely intuitive and fluid";
  the physical controls and the water's surface and sound make it feel like real diving.
  ([Steam reviews](https://steamcommunity.com/app/2550040/reviews/?browsefilter=toprated))
* **Big strokes and small ones both work.** Hubris's developers answered complaints that you can
  swim with large, realistic strokes, or "just do minimal movements to swim quite fast once you
  get the hang of it", and that they play seated with little effort. Players who swim in real life
  loved it, and found it natural once they stroked "with palms facing directly behind, since the
  character moves opposite to the palms".
  ([Steam discussion](https://steamcommunity.com/app/1265800/discussions/0/3761103414984844398/))
* **Swimming as the game, not just a way to travel.** Freediver: Triton Down's review praises 3D
  movement that feels seamless, a breath timer that adds tension, and visible arms that make the
  body feel present; its strokes can be tuned for speed and momentum, and the default was "a good
  balance between realism and exertion".
  ([Road to VR](https://www.roadtovr.com/freediver-triton-down-vr-adventure/))
* **Arm-driven movement is comfortable for many.** Echo VR and Lone Echo (push off and glide) and
  Gorilla Tag (arms only) are among VR's best-loved movement: motion the player makes with their own
  body ([Road to VR](https://www.roadtovr.com/new-category-of-vr-game-arm-based-locomotion/)). Lone
  Echo's movement is "very comfortable" in reviews (see `framework/docs/design`).

**Disliked.**

* **Only one stroke counts.** Hubris was accused of expecting "some bizarre version of the
  doggy-paddle": breaststroke or sidestroke did nothing, normal strokes "barely moved them forward",
  breaststroke swam in place. The lesson: any reasonable stroke must work, judged by what the hands
  do (which way the palm pushes), not a gesture to match.
* **Pushes that aren't meant.** Reaching for the inventory started swimming; the controllers' angle
  pushed players upwards; armrests got in the way; you couldn't aim while swimming. Hands doing
  something else (holding, reaching for a holster or a weapon) must not swim.
* **Too slow and too tiring.** "Very slow and physically tiring"; players asked for the stick as
  well. Arm-swing in other games was "like crawling across sand". The stroke must be able to reach
  full speed without flailing, and the stick must stay.
* **Turning by one arm.** Freediver turns you when you stroke with one arm; its reviewer found it
  odd and preferred to turn his real body. No forced turns.
* **Skyrim VR's water.** Players couldn't dive (the grip-crouch doesn't work in water; you point the
  controllers down to dive with its "realistic swimming", or squat), currents push you about and
  give headaches, and the underwater view is murky.
  ([Steam](https://steamcommunity.com/app/611670/discussions/0/1696046342856549326)). Skyrim VR
  has no arm swimming of its own; DovaVR Locomotion adds one (crawl, breaststroke and butterfly
  read from the arms at different speeds, with water momentum and inertia, and a button-only swim)
  ([Nexus](https://www.nexusmods.com/skyrimspecialedition/mods/154803)); VRIK adds swimming
  animations to the body.
* **Fatigue and seasickness.** A study of a seated breaststroke method found less sickness than
  smooth movement and as little as teleporting, with more enjoyment, but sore arms and shoulders
  ([ResearchGate](https://www.researchgate.net/publication/400069478_Reduction_of_Virtual_Reality_Sickness_by_Swimming_Locomotion)).
  Forced vertical motion (bobbing, a current, being pushed up) is what players call out.

## How the best do it

* **Freediver**: both triggers held and both arms swung together stroke; momentum is a setting.
* **Subside**: physical swimming until fins and a weight belt let buttons do it.
* **Echo VR / Lone Echo**: push off and glide; momentum and a soft drag make it feel weightless.

**What makes water feel like water** (from all of the above): momentum that glides out after the
stroke instead of stopping dead; resistance felt in the hands (a buzz while a hand pushes); a push
that depends on how the palm meets the water (feathering a hand back costs nothing); the surface
holding you up when you stop; and the game's sound and look of the water.

## Oblivion's own swimming

As the game plays it, checked in the game where marked.

* **When the player swims.** Once 70% of the player's height is under the water they swim; at
  87.5% the head is under and breath runs. Water Walking keeps the player on the surface instead.
* **The water's height** is the cell's own; a cell may have none.
* **Speed.** Swimming has its own speed: the walk speed (Speed, encumbrance) times
  (1 + Swim Speed Multiplier / 100) times (`fMoveSwimWalkBase` + `fMoveSwimWalkAthleticsMult` ×
  Athletics / 100); running adds the run multipliers (`fMoveRunMult`, `fMoveRunAthleticsMult`) and
  `fMoveSwimRunBase` + `fMoveSwimRunAthleticsMult`. Their values: 0.5, 0.02, 0.5, 0.1.
* **Direction.** The stick swims at the game's own animation speed forward, tipped by where the
  head looks (looking down dives; the pitch is held to ±89°).
* **In the water** the velocity is the one wanted at once: no momentum and no buoyancy under
  water; stop pushing and the player hangs where they are. At the surface they can't rise, gravity
  holds them there, and a jump there jumps: that's how the game climbs out onto a ledge.
* **Athletics.** Swimming while moving (the stick or the move keys) trains Athletics, one use a
  second. Without a direction from the controls, no training: so the arms' swimming has to show the
  game a direction.
* **Breath.** Max breath = Endurance × `fActorSwimBreathMult` + `fActorSwimBreathBase`
  (`Oblivion.esm`: 0.3 and 4, so 19 s at 50 Endurance). With the head under and no Water Breathing,
  breath runs down; at 0 it's damage (Health × `fActorSwimBreathDamage` a second),
  `NPCHumanDrowning`. Surfacing refills it at once; a near empty breath gasps
  (`NPCHumanGaspMale`/`Female`). God mode breathes. The HUD's breath meter is its own menu
  (`menus\breath_meter_menu.xml`): shown when the player enters the water, filled with
  breath / max, at the top middle of the screen.
* **Splashes.** Falling into the water fast plays `CWaterHumanoid` and makes a splash at the
  water's height. The water's surface ripples where actors wade (`bUseWaterDisplacements`).
* **The picture.** Each eye is the game's own render (`render.rs`), so the water's surface, its
  reflections and the underwater fog and tint are the game's for that eye's camera.

## The design

### The framework: swimming by the arms, for any game

* **The game says where the water is.** `GameState::water` (`WaterState`): the surface's height in
  the world, whether the game has the player swimming, whether their head is under, breath left,
  and how fast the game lets the player swim now (its own speed: skills, attributes, load). The
  game decides when the player swims; the runtime never does.
* **The runtime strokes** (`crates/runtime/src/swim.rs`): catch the water and pull yourself past
  it, as in Freediver: Triton Down (which grew from Lone Echo's zero-gravity movement: "you
  essentially pull yourself through the world", UploadVR's review). A hand under the surface with
  its grip squeezed holds the water; pulling it carries the swimmer the other way, several times
  its pace: 3 game swim speeds with the hands at 1 m/s, growing with their speed to the 0.4th
  power, so light strokes swim well (the user had to push too hard: playtest 10). Let go and the hand reaches out again
  freely while the swimmer glides. Any stroke works, in any direction, and a pull never slows the
  swimmer (one slower than they already go its way slips). The hands holding the water make one
  stroke, their mean, so a breaststroke's sweeps out and in cancel (each hand's own way hid its
  pull behind the sweep out: playtest 8). The stroke pulls only while it goes on away from where
  it started (or turned back): nobody times the grip exactly, and hands still held as the reach
  ends or the recovery starts threw the swimmer backwards (playtest 7). Gliding, a stroke never
  takes the swimmer's speed: it adds to it and turns the glide its way, as a streamlined body
  follows the stroke. Only a push back against the glide slows them, and takes them back: one
  that's more push than sweep, its hands going that way at least half as far as they move apart
  or together (a stroke back had to be 20 cm long instead, which made going back too hard to
  trigger: playtest 11). The next
  breaststroke's sweep out goes a little forward, and slowed the swimmer at every catch (playtest
  9). Each hand's velocity is measured
  against the head (the body's own walking about taken out).
  - The other well-liked ones: Freediver's triggers start a stroke, both arms swung together (one
    alone turns you); a university project (Marine Life Exploration) has you hold the side buttons
    and pull back; Hubris reads the palms ("acceleration opposite to the direction your palms are
    facing", Keen Gamer); Subside paddles and pulls on rocks. Unreal developers suggest anchoring
    an open hand in the water like a climbing hold, with a button where the hand can't tell
    (Unreal Engine forums, "VR locomotion via swimming-moves"). Skyrim VR's own "Realistic
    Swimming" is thought awkward; mods detect strokes by kind (crawl, breaststroke) instead.
  - The playtests got here the long way round. Builds 1 to 5 made the water push on every hand,
    as real water does: by its palm (a hand round a controller faces the other hand, so strokes
    drifted backwards), then by where the head looked, then by the strokes' strongest part, then
    plain physics (force with the square of the hand's speed), then with a hand slicing fingertips
    first. A controller can't show the palm that catches the water and the one that slices back
    through it, so the recovery always held the swimmer back: "terrible, unnatural, the speed is
    shit". The grip says which.
* **Momentum and drag.** The swimmer's speed glides out by e over six seconds, sooner when fast:
  a firm stroke from still carries them about 17 m. The game's swim speed scales the pull (skills, attributes,
  load), and the strokes may go up to four times it: the user asked for strokes much faster than
  the game's swimmer, and for more glide.
* **Comfort.** Nothing turns the player but the player (the strokes push, they never turn the
  view); walking about in the room pulls nothing; there's no buoyancy pulling the view up or down
  (the game holds the player at the surface, and under water they hang where they stop, as the game
  has it); motion only ever comes from the player's own strokes, eased by the glide.
* **Feel.** A hand catching the water gives a little pulse, and buzzes softly with the water's
  resistance as it pulls; a hand slapping into the water gives a pulse.
* **The stick swims too**, as the game does (where the head looks), and the arms add to it. Strokes
  can be turned off, or made easier, in the menu's Controls tab ("Swim with your arms (in the
  game's water)", on by default; "Stroke strength", 0.5 to 3, 1 by default).
* **The body floats** (`rustvr_ik`): in the water the legs hang from the hips and kick (a slow
  treading kick at rest, a flutter kick as the player swims), and the body leans forward as the
  player swims forward, the hips trailing, instead of standing on a floor. The arms are the player's
  own, as always. The swimming pose eases in and out over half a second.
* **The protocol**: `GameState::water` from the game; `Intents::swim` (the velocity the strokes give,
  tracking space) and `Intents::swim_effort` (how hard the arms are working) back. Added in
  protocol version 12.

### Oblivion

* **The water** (`crates/oblivion/src/swim.rs`): each frame before the game's update, the player's
  cell's water height, whether the game has the player swimming, whether the head is under (87.5%
  of the player's height, as the game asks), breath against its maximum, Water Walking, and the
  game's own speed (which is the swim speed while swimming), to the runtime.
* **The strokes move the player by the game's swimming.** In `Actor::Move` (already hooked by
  `movement.rs`), while the game has the player swimming, the strokes' velocity (tracking space to
  the world, then into the player's own frame) becomes the frame's step, added to the stick's,
  never longer than the faster of the two allows. The game rotates a swimming step by the pitch
  the head gives the player, so the strokes' step is turned back by it first: the strokes go where
  the hands push, the stick where the head looks. The game's character controller moves the player with its
  own collision and its own surface (it holds the player at the surface and lets them out at the
  shore), and climbing out onto a ledge is the game's jump from the surface (A).
* **The game sees a swimmer.** While the arms swim and the stick is still, the game's joystick is
  nudged the way the strokes go: its movement flags get their direction, so the game plays its
  swimming (its strokes' sounds) and trains Athletics as it does for any swimming; the step itself is
  the strokes'.
* **Breath**: the game's own (its timer, its drowning, Water Breathing and god mode). Its meter is
  put on the HUD where the game has it while it shows (`hud.rs`: the meter's rectangle, from the
  game's own layout of it, as the item card's is). With little breath left the heart beats in both
  hands (the runtime), faster and harder as it runs out; nothing pulls the player up: surfacing is
  theirs.
* **Under the water or above it.** The renderer's underwater state (the shaders' underwater
  variants, the surface seen from below) is decided in the world render itself, from each eye's
  camera, so it's per eye already. The fog (the sky's update, from the game's camera, once a frame
  before the eyes) followed the game's eye, half a metre above the water while
  swimming: a face ducked into the water saw it clear as air. The sky's update now sees the
  headset's eyes there.
* **The body in the water** is seen through the surface: the game's water is in its depth, so a
  hand under it vanished from above. The runtime is told the water's surface, colour and fog reach,
  and the body under it shows through the surface the game drew, the water's colour over it by how
  far the light went through.
* **Splashes**: a hand slapping into the water gets the game's own splash for a thing falling in
  (`CWaterSmall` or `CWaterMedium`, and its `sSplashParticles` at the surface, scaled
  `fSplashScale3` or 2). The game's surface ripples are made for wading actors only (the player's,
  while it swims: the nudged joystick makes it a mover); hands get no ripples of their own.
* **Climbing out**: at the shore the game stands the player up; at a ledge, A is the game's jump
  from the surface (the player rig holds Jump while A is down: the bridge now jumps on its start).
