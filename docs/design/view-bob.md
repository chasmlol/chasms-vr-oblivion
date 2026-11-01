# View bob

Research for the desktop view's bob: the window people watch when the player records or streams
(the runtime's `spectator` and `view_bob`). Walking by the stick, the desktop view glided like a
floating camera; a flat game's view bobs with the steps. What good flat games and smoothed
spectator cameras do, Oblivion's own first-person bob as its animations have it, and the design
built from both. The headset's view never bobs.

## How flat games and spectator cameras do it

* **Quake and Half-Life:** the bob is the horizontal speed times `cl_bob` (0.02), on a cycle of its
  own (`cl_bobcycle` 0.6 s), its rise shaped by `cl_bobup` (0.5): 0.3 of it steady and 0.7 a sine,
  clamped. It moves the view straight up and tilts it a little.
  ([WinQuake `view.c`](https://ara-mdk.googlesource.com/platform/external/quake/+/9fd67c44777b350dc56f3e70c88963b0d966ffc7/quake/src/WinQuake/view.c),
  [Half-Life `view.cpp`](https://github.com/ValveSoftware/halflife/blob/c7240b965743a53a29491dd49320c88eecf6257b/cl_dll/view.cpp))
  Its cycle doesn't follow the steps: the [TWHL tutorial](https://twhl.info/wiki/page/Tutorial:_View_bobbing:_Part_1)
  works around exactly that.
* **Modern controllers tie it to the walk:** cadence and size from the actual speed, softer and
  slower walking, quicker and stronger sprinting
  ([s&box's head bob](https://sbox.game/notpointless/chomnr_head_bobbing_effect)); OpenMW's
  first-person bob (Morrowind's engine, re-made) runs on the distance walked, a step length and a
  height, with a fifth of a degree of roll
  ([OpenMW](https://git.cijber.net/mirror/openmw/commit/9d350ec824e8f8985b31746cc5ed10bf79734f45)).
* **Strong bob sickens:** Skyrim's bob mods offer light, standard and heavy, and warn that heavy
  can make players sick ([Nexus](https://www.nexusmods.com/skyrim/mods/47086)); body-cam style
  bob is smoothed to a middle ground
  ([GameDev.net](https://gamedev.net/forums/topic/719653-bodycam-style-realistic-head-bobbing-motion-for-first-person-controllers)).
  In VR, oscillation of the view is worse still (see [riding.md](riding.md)): the headset never
  takes it.
* **Spectator cameras** (Half-Life: Alyx's, LIV's smoothed first person) steady the head's turns
  and keep the horizon level; the runtime's desktop view already does (`spectator`). The bob goes
  on that steadied camera, so it stays the only movement a viewer sees besides the walk and the
  look.
* **People walking:** the head's vertical movement peaks at the step frequency, about 2 steps a
  second at an easy pace, from 1.5 at 0.6 m/s to 2.4 at 2.2 m/s
  ([MacDougall and Moore 2005](https://cspeech.ucd.ie/Fred/docs/MacDougallMoore.pdf)); the body's
  centre rises and falls about 2 cm a step at a slow walk
  ([PMC12945828](https://pmc.ncbi.nlm.nih.gov/articles/PMC12945828/table/tab2)), more the faster.

## Oblivion's own first-person bob

From the game's own `Oblivion - Meshes.bsa`. **Oblivion has a first-person bob: its first-person
animations move its camera.** There is no bob setting (no game setting or INI value names one); the
bob is in the animations' data.

* **The camera is on a node.** In first person the game's camera sits on the first-person
  skeleton's `Camera01` and turns with the look and with `Camera01`'s own turn. The skeleton hangs
  from the first-person node, at the eye; at rest `Camera01` is on that node. `skeleton.nif` (`characters\_1stperson`) has `Camera01` at
  (0, -3.74, 118) units, turned a quarter about X: no turn of the camera.
* **Every first-person animation keys `Camera01`** (all 323 of `characters\_1stperson\*.kf` name
  it). How they move it (units; 70 to the metre):

  | Animations | `Camera01`'s height | |
  |---|---|---|
  | `walkforward`, `walkleft`, `walkbackward`, ... (1 s a stride) | 118 down to 117.62 | 5 mm, once a stride |
  | `walkfastforward`, ... (running, 0.87 s a stride) | 117.62 to 118.93 | 1.9 cm, at each step |
  | `sneak*` | as walking and running | and 4.6 units further forward (a lean) |
  | `idle` | 117.82 to 118 over 6 s | 2.5 mm: breathing |
  | `jumploop`, `jumpland` (0.57 s) | 118.76 in the air, landing down to 104.7 | **20 cm landing dip** |
  | `dodge*` | down to 99.7 | 26 cm |
  | `swim*` | 117 to 121.5 over 1.3 to 1.6 s | up to 5 cm, sideways 0.7 cm swimming fast |

  The weapon versions of walking and running (`onehandforward`, `handtohandforward`, `bowforward`,
  ...) don't key `Camera01`, the sneaking ones do, and the jumps' for every weapon do: the game
  keeps the node where the last animation left it, so with a weapon out a walk doesn't bob. None
  key its turn.
* **What the headset had:** the adapter anchored the headset on the game's camera, so all of this
  reached the headset's view (with the Comfort tab's "Camera bob / shake" off): a 20 cm dip on
  every landing, the swim's 5 cm.

## The design

* **Oblivion's "Game" bob is the engine's own value, read each frame:** where `Camera01` is against
  the first-person node, up and to the side in the node's frame (forward isn't a bob: the rest is a
  little behind the node, and a sneak's lean is a posture), into the world, and `Camera01`'s turn as
  the game's camera takes it (`camera::first_person_bob`). It's given to the runtime in the game
  camera's own frame (`GameState::view_bob`). On a horse, the gait's bob under the rider's eye
  (`riding::carry`'s eye against its eye held steady).
* **The headset no longer takes it:** the anchor's eye without the bob (`Anchor::base_eye`), so
  the Comfort tab's "Camera bob / shake" decides, as it does for every game: off by default. At
  most 0.2 m is held out (`BOB_HELD`): the adapter SDK takes a gap past 0.25 m for a scripted
  camera and follows it; the rest of a dodge's 26 cm dip goes with the view rather than jumping.
  With "Camera bob / shake" on, the headset's picture has the bob and the desktop view (which shows
  that picture) adds nothing more.
* **The runtime's own bob ("Generic"),** for every game, and Oblivion's when it gives none (third
  person): steps at the walking pace above (`1.16 + 0.56 v` a second, 3 at most), dropping at each
  footfall and rising over the foot (a walker's roll, a runner's sharp landing), swaying a little
  over each foot, a slight nod; 0.8 cm either way at a walk, 1.8 at a run, eased with the speed; a
  landing's dip on a spring (about 2.5 cm from a jump); smaller crouched, a slow float in water.
  Real steps in the room add none (the walk the runtime measures leaves them out).
* **Settings** (the menu's Studio tab, "Bobs as you walk (never the headset)"; `[desktop_bob]`):
  off, the runtime's, or the game's own (`mode`, the default: what the flat game shows), and "Bob
  strength" (`strength`, 0 to 3, 1 by default: the game's as it is, the runtime's as made). Damped
  as the view narrows for aiming (the cube of the narrowing: a quarter at the aim zoom's 65 of 105
  degrees).
