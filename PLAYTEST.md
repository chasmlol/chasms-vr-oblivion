# Playtest: the desktop view's effects studio, and its bob as you walk

Save: rustvr-weye

The desktop view (the window on the monitor, for recording and streaming) gets a studio of
effects only it shows. They'd make you sick in a headset, and they make footage look filmed. The
headset's view never changes.

Everything is in the RustVR menu's new **Studio** tab, and it's saved with the settings.

- **No black bars:** the letterbox is gone, as you asked after the first look.
- **The view bob branch is merged in here:** the desktop view bobs as you walk (below).
- **The desktop camera's own settings** sit at the top of the Studio tab's Camera section:
  - its field of view;
  - the overscan;
  - the bob.
- **The right stick scrolls the RustVR menu** (any tab, wherever the laser points), as it already
  scrolls the game's menus.

The save is Weye's open ground on Lake Rumare's shore: room to walk, run and jump, and water to
swim in, for the bob as well as the effects.

How the studio's built and why: `framework/docs/research/desktop-fx.md`. The bob's research:
`docs/research/view-bob.md`.

## The looks

Pick one at the top of the Studio tab. Change anything by hand and it becomes "custom".

| Look | What it is |
|---|---|
| **Clean** | The game's picture as it is: no effects. Snap turns still glide. |
| **Cinematic** | A film's look, through a 90 degree lens. |
| **Hyper-real** | A body camera's look (Unrecord, Bodycam): a wide lens bent towards a fisheye, worn on the chest. |
| **Action** | Every shot and blow felt; the view widens at a run; punchy colour. |
| **Vintage** | Old film: heavy grain, a strong glow, faded warm colour. |

- **Cinematic** also has:
  - 24 frames at a 180 degree shutter's motion blur;
  - focus pulled to what you look at, aim at or hold up;
  - warm glowing highlights, fine grain, teal shadows.
- **Hyper-real** also has:
  - the chest-worn camera turns after your body, a little down, swaying;
  - a slow shutter's smear;
  - hard sharpening, fringes and dark corners;
  - a cold, flat colour.

**For viewers**, whatever the look:

- **Snap turns, snap crouches and recentres turn as a head would** instead of cutting. On by
  default.
  - The turn gathers speed, swings through, and lands with a little overshoot.
  - A 45 degree snap settles in about a third of a second.
- **No black at the edges**: where a fast turn makes the desktop view see past the game's picture,
  the picture's edge is stretched out over the gap. It's a smear, but not black. On by default.
- **The game's HUD where a flat player has it**: the bars, the compass and the subtitles at the
  screen's edges, from the game's own picture of them. It's off by default.
  - The headset keeps the HUD where the menus feature floats it.
  - On the desktop, the same pieces stop floating in the world.

## The bob as you walk (merged from the view bob branch)

It's in the Studio tab, under Camera: "Bobs as you walk (never the headset)", and "Bob strength"
(1 by default, up to 3).

- **Off**: the camera glides.
- **The runtime's**: the framework's own bob, the same for every game.
  - About 2 steps a second walking and 3 running.
  - The view drops at each footfall and sways a little over each foot.
  - A soft dip landing from a jump or a fall; smaller crouched; a slow float swimming.
- **The game's own** (the default): Oblivion's own first-person bob, read from the game. Where the
  game gives none (third person), the runtime's.

Oblivion's own bob comes from its first-person animations moving the camera:

- **Walking:** 5 mm a stride, very subtle.
- **Running:** 1.9 cm a step.
- **Landing a jump:** a 20 cm dip.
- **Swimming:** a slow float of up to 5 cm.
- **With a weapon or fists out:** walking and running don't bob at all. That's Oblivion's own data.

**The headset changes too:** it no longer takes the game's camera animation. Before, a landing dipped
your view 20 cm. The Comfort tab's "Camera bob / shake" brings it back if you want it.

## What to try

Watch the monitor (or OBS) while you play.

1. **Snap turns.** Flick the right stick.
   - The headset snaps.
   - The desktop view should turn through it as your head would: it gathers speed, swings, and
     settles with a slight overshoot. The world turns round your body, with no cut and no black
     edge.
   - Flick twice quickly: it should become one longer turn that keeps its speed.
   - The slider "A 45 degree snap settles in" sets how long it takes. **Tell me** if it's too quick
     or too lazy.
   - Try it in Clean and in Cinematic: in Cinematic the pan is motion-blurred.
2. **Recentre** (the menu's Recenter button, or after a load). It should glide, not cut. A step to
   the side dissolves rather than pans.
3. **Turning your head fast.**
   - With motion blur on (Cinematic, Hyper-real, Action), the view should smear as a camera's pan
     does, more at the edges than the middle.
   - On the side you turn towards, the desktop view runs past the game's picture for a moment. It
     should show a stretched smear of the picture's edge there, never a black band.
4. **Walking, running, a horse.** In Cinematic and Action, the view widens as you go faster and
   eases back as you stop.
   - Galloping should widen it most.
   - Things passing close by smear.
5. **A sword or a mace.** Swing it fast. The blade and your hand should streak along their swing,
   the rest of the picture staying sharp.
6. **A bow.** Loose an arrow: the camera kicks a little.
7. **Hitting things hard, and jumping off something high.**
   - A blow that lands shakes the camera (Action most).
   - A hard landing dips it.
   - Oblivion doesn't tell the runtime when you're hit, so taking damage doesn't shake it yet.
8. **Depth of field** (Cinematic, Vintage):
   - Look at something near, then far: the focus pulls between them.
   - Hold a potion or a book up close to your face: it comes into focus and the world behind
     blurs.
9. **The bob.** Walk, run, sneak, jump and swim with each of its three choices.
   - Watch the recording, and check the headset never bobs.
   - With the game's own, walking is barely there; running, landing and swimming show it.
   - **Tell me** how each looks, and whether the strength should be different by default.
10. **The flat HUD.** Studio tab, "The game's HUD on the screen where a flat player has it".
   - The health bars, the compass and the effects should sit at the bottom of the desktop view, at
     about the size Oblivion has them on a flat screen.
   - Subtitles should sit low in the middle.
   - The activate icon should be in the middle when you look at something.
   - All of them are crisp, over any effects.
   - Open a menu: the HUD goes and the menus float in the world as before.
   - **Tell me** if anything is the wrong size or in the wrong place.
11. **Your own mix.** Every effect has its own switch and strengths below the looks.
    - Colour grades: teal and orange, bleach bypass, warm film, body cam, noir.
    - LUTs: put `.cube` files in `%LOCALAPPDATA%\RustVR\luts` and they show in the tab.

12. **The menu.** Open the RustVR menu and push the right stick forward and back: the tab
    scrolls, wherever the laser points.
    - The Studio tab's Camera section has the desktop field of view, the overscan and the bob.
    - The field of view slider moves the look's own lens where it has one (and the look becomes
      custom). In Clean it's your own.

**Tell me** which looks you'd keep and what you'd change in each: they're my picks, tuned on the
test range, not in Oblivion.

## What I checked, and what I couldn't

**Checked** in the framework's test range, on the emulated headset, with the desktop window
captured:

- each look;
- a fast head turn (with the edge stretch off and on);
- two snap turns (captured frame by frame through the turn);
- a walk and a sprint, a sword swung across the body, the rifle fired in bursts;
- the flat HUD, over Clean and over Cinematic;
- the RustVR menu scrolled by the right stick, with the laser on the pause panel.

The view bob branch checked its runtime's bob on the range (walking, running, landings) before the
merge; the merged build passes its tests and the studio's.

The unit tests cover:

- what's a jump and what isn't: a snap, a crouch, a load, a game's hitch;
- the shake wearing off;
- the flat HUD's layout;
- the looks;
- reading a `.cube` LUT.

The view bob branch is merged here with no conflicts, and the aim zoom still merges cleanly with
the result.

**Couldn't check:**

- **Oblivion itself**, above all:
  - Its HUD laid out flat. Its picture of the HUD is the eye's shape; the pieces are re-anchored
    and sized from its text.
  - Its depth for the focus and the blur.
  - How a real snap turn, a real recentre and a real horse look.
  - Oblivion's own bob in play.
    - The adapter's log says every 5 seconds how far the game's bob went (`view bob: the game's
      own in the last 5 s: ...` in `rustvr-oblivion.log`).
    - The runtime's log says what the desktop view showed (`desktop view: bobbed ...`).
  - The headset no longer taking the game's bob, and its height standing unchanged.
- **The real headset's timing**: whether the effects ever cost the headset a frame. They run after
  the headset's frame is handed over, and they're a third of a millisecond on this GPU.

## GPU times

On the test range, RTX 5090, at 2227x1252 (the window fitted to the monitor), with the runtime at
120 Hz. These are medians of ten seconds, three runs each. The desktop view logs them every ten
seconds in `rustvr.log` ("desktop view: ... ms on the GPU").

| Look | The desktop picture (before the headset's frame) | Effects and HUD (after it) |
|---|---|---|
| Clean | 0.16-0.18 ms | none |
| Cinematic | 0.15-0.17 ms | 0.30-0.35 ms |
| Hyper-real | 0.17-0.18 ms | 0.30-0.34 ms |
| Action | 0.16-0.18 ms | 0.28-0.33 ms |
| Vintage | 0.09-0.10 ms | 0.30-0.35 ms |

What the studio adds before the headset's frame (the body's depth for the blur and focus, the copy
kept for smoothed snaps, the HUD's copy) is lost in the run-to-run spread.

## For the merge

Notes for merging this branch:

- **The view bob is merged in** (framework and adapter), at your word. Its bob settings moved from
  the Display tab to the Studio tab's Camera section. The desktop field of view and the overscan are
  on both tabs (the Display tab's field of view stays where the aim zoom's lines follow it).
- **The aim zoom isn't merged.** It still merges cleanly. Its settings belong in the Studio tab's
  Camera section too; on its branch they're on the Display tab.
- **The protocol** has the view bob's `GameState::view_bob`, and nothing of the studio's.
