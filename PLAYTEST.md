# Playtest: the desktop view bobs as you walk

Save: rustvr-weye

The desktop view (the window on your monitor, for recording and streaming) no longer glides like a
floating camera when you walk with the stick: it bobs the way a flat game's view does. **Only the
desktop view: the headset never bobs.** Real steps in the room add no bob (your real head's movement
is in the picture already).

Three choices, in the RustVR menu's Display tab, under the desktop view's settings ("Desktop view
bobs as you walk (never the headset)"), saved as `[desktop_bob]` in `settings.toml`:

- **off**: the camera glides, as before;
- **the runtime's**: the framework's own bob, the same for every game: about 2 steps a second
  walking and 3 running, the view dropping at each footfall (sharper running) and swaying a little
  over each foot, 0.8 cm either way walking and 1.8 running; a soft dip landing from a jump or a
  fall; smaller crouched; a slow float swimming;
- **the game's own** (the default): Oblivion's own first-person bob, read from the game each frame
  (below). Where the game gives none (third person), the runtime's.

And **"Desktop view's bob strength"** (1 by default: Oblivion's as Oblivion has it, the runtime's as
made; up to 3).

Why these defaults: you asked for the game's bob, and the game's own is what a viewer of the flat
game knows. At its own strength, since it's the game's value; the runtime's is made to look like a
good flat game's at 1.

## Oblivion's own bob: what I found

Oblivion **does** bob its first-person camera, but not by a setting or a formula: its
first-person animations move the camera. In first person the engine puts the camera on a node of the
first-person skeleton (`Camera01`), and every first-person animation moves that node:

- **walking:** 5 mm down and up once a stride: very subtle;
- **running:** 1.9 cm at each step;
- **sneaking:** as walking and running;
- **landing from a jump:** a **20 cm dip** (and 1 cm up while in the air);
- **swimming:** a slow float up to 5 cm;
- **standing:** 2.5 mm of breathing over 6 seconds;
- **with a weapon or your fists out, walking and running don't bob at all:** those animations
  don't move the camera node (Bethesda's data; sneaking with them does).

So "the game's own" is faithful but quiet when you walk: the run, the landing and swimming are where
it shows. The strength slider scales all of it, the landing too.

**The headset changes too.** Until now all of this also reached your headset (the 20 cm landing
dip, the swim's float), because the adapter anchored the headset on the game's camera. Now the
headset is anchored on the eye without it, as the Comfort tab's **"Camera bob / shake"** (off by
default) says for every game. Turn that on to have the game's bob in the headset again; the desktop
view then adds nothing of its own (the picture has it already). On a horse nothing changes in the
headset; the desktop view gets the gait's bob under the rider's eye.

The full research (the engine's code, every animation's numbers):
[docs/research/view-bob.md](docs/research/view-bob.md).

## What to look at

Two `.bat`s in this worktree (not committed):

- **`playtest.bat`**: Oblivion booted into `rustvr-weye` (open ground on Lake Rumare's shore) with
  this build and the desktop view on. It waits while you play, then closes the game and keeps the
  runtime's log in `<builds>\viewbob\playtest-<n>`.
- **`range.bat`**: the framework's test range from this build, for the runtime's own bob.

Watch the monitor (or record it) while you play. With each setting:

| You | The game's own (default) | The runtime's | Off |
|---|---|---|---|
| Walk (weapon away) | barely: 5 mm a stride | clear steps, about 2 a second | glides |
| Run | a bob at each step (1.9 cm) | quicker, bigger steps | glides |
| Walk or run, weapon or fists out | **none** (Oblivion's own) | steps as without | glides |
| Sneak | as walking and running | smaller, slower | glides |
| Jump | a deep dip landing (20 cm) | a soft dip (about 2 cm) | none |
| Swim | a slow float (5 cm) | a slow, small float | none |
| Ride | the horse's gait | steps at the horse's speed | glides |

And in the headset, with each setting: **nothing bobs**, walking, running, sneaking, landing a jump
or swimming. (Before this, a landing dipped your view 20 cm.) Check the view's height feels as it
did standing.

Say how each looks on the recording, and whether the strength should be different by default.

## What I checked, and what I couldn't

Checked:

- The runtime's bob in the framework's test range on the emulated headset (my own link and folder),
  strafing along a table at a walk (1.4 m/s) and a run (3 m/s), then two jumps, the desktop view
  saved every twentieth of a second: walking 0.7 cm either way at 1.9 steps a second, running 1.4
  at 2.8, each landing a 2.2 cm dip; standing, not a pixel of movement. A floor line against a fixed
  row across consecutive captures: `<builds>\viewbob\analysis\bob_strips.png`.
- Unit tests: the pace and size of the steps, a stop fading out, a paused game, a landing's dip and
  a slope that isn't one, the game's own shown as given (and nothing added when the headset has
  it), the aim zoom's damping, the walk measured right however unevenly a game's steps are stamped
  (the range's step-by-step speeds read 5 to 70 m/s walking at 1.4: the bob measures over a quarter
  of a second).
- Merged with the aim zoom's branch (no conflicts); there the bob is damped while the view is
  zoomed for aiming (a quarter at its 65 of 105 degrees), with no change needed.

Not checked (only you can, in the game):

- Oblivion's own bob in play: whether the camera node read gives the numbers above. The adapter's
  log says every 5 seconds how far the game's bob went and how far the game's camera was from the
  node it's put on (`view bob: the game's own in the last 5 s: ...` in `rustvr-oblivion.log`; it
  should say about 0 units off). The runtime's log says what the desktop view showed (`desktop view:
  bobbed ...`).
- The headset no longer taking the game's bob (and its height standing unchanged).
- Riding, swimming and sneaking with the game's own.

Found on the way, not from this change: the emulated headset (`rustvr-xrsim`) crashes as it exits
after a run, with the desktop view off too; every capture and log is written before it.
