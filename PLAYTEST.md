# Playtest: the desktop view's effects studio

Save: rustvr-city

The desktop view (the window on the monitor, for recording and streaming) gets a studio of
effects only it shows. They'd make you sick in a headset, and they make footage look filmed. The
headset's view never changes.

Everything is in the RustVR menu's new **Studio** tab, and it's saved with the settings. How it's
built and why is in `framework/docs/research/desktop-fx.md`.

## The looks

Pick one at the top of the Studio tab. Change anything by hand and it becomes "custom".

| Look | What it is |
|---|---|
| **Clean** | The game's picture as it is: no effects. Snap turns still glide. |
| **Cinematic** | A film's look: a 2.39:1 letterbox and a 90 degree lens. |
| **Hyper-real** | A body camera's look (Unrecord, Bodycam): a wide lens bent towards a fisheye, worn on the chest. |
| **Action** | Every shot and blow felt; the view widens at a run; 2:1. |
| **Vintage** | Old film: 4:3, heavy grain, a strong glow, faded warm colour. |

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

- **Snap turns, snap crouches and recentres glide** over a quarter of a second instead of cutting.
  This is on by default.
- **The game's HUD where a flat player has it**: the bars, the compass and the subtitles at the
  screen's edges, from the game's own picture of them. It's off by default.
  - The headset keeps the HUD where the menus feature floats it.
  - On the desktop, the same pieces stop floating in the world.

## What to try

Watch the monitor (or OBS) while you play.

1. **Snap turns.** Flick the right stick.
   - The headset snaps.
   - The desktop view should pan through the turn smoothly, the world turning round your body,
     with no cut and no black edge.
   - Try it in Clean and in Cinematic: in Cinematic the pan is motion-blurred.
2. **Recentre** (the menu's Recenter button, or after a load). It should glide, not cut. A step to
   the side dissolves rather than pans.
3. **Turning your head fast.** With motion blur on (Cinematic, Hyper-real, Action), the view should
   smear as a camera's pan does, more at the edges than the middle.
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
9. **The flat HUD.** Studio tab, "The game's HUD on the screen where a flat player has it".
   - The health bars, the compass and the effects should sit at the bottom of the desktop view, at
     about the size Oblivion has them on a flat screen.
   - Subtitles should sit low in the middle.
   - The activate icon should be in the middle when you look at something.
   - All of them are crisp, over any effects, inside any letterbox.
   - Open a menu: the HUD goes and the menus float in the world as before.
   - **Tell me** if anything is the wrong size or in the wrong place.
10. **Your own mix.** Every effect has its own switch and strengths below the looks.
    - Colour grades: teal and orange, bleach bypass, warm film, body cam, noir.
    - LUTs: put `.cube` files in `%LOCALAPPDATA%\RustVR\luts` and they show in the tab.

**Tell me** which looks you'd keep and what you'd change in each: they're my picks, tuned on the
test range, not in Oblivion.

## What I checked, and what I couldn't

**Checked** in the framework's test range, on the emulated headset, with the desktop window
captured:

- each look;
- a fast head turn, two snap turns (captured frame by frame through the glide);
- a walk and a sprint, a sword swung across the body, the rifle fired in bursts;
- the flat HUD, over Clean and over Cinematic.

The unit tests cover:

- what's a jump and what isn't: a snap, a crouch, a load, a game's hitch;
- the shake wearing off;
- the letterbox;
- the flat HUD's layout;
- the looks;
- reading a `.cube` LUT.

The three branches merged together (this one, the aim zoom, the view bob) build with no conflicts.

**Couldn't check:**

- **Oblivion itself**, above all:
  - Its HUD laid out flat. Its picture of the HUD is the eye's shape; the pieces are re-anchored
    and sized from its text.
  - Its depth for the focus and the blur.
  - How a real snap turn, a real recentre and a real horse look.
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

- The framework's `fxstudio` branch merges cleanly with `aimzoom` and `viewbob`, and the three
  together build. How the pieces fit is in `framework/docs/research/desktop-fx.md` section 3.
- **The Studio tab's Camera section** is where the aim zoom's and the view bob's settings belong
  once merged (they're on the Display tab on their branches).
- **`RUSTVR_DESKTOP_EVERY`** (seconds between saved desktop frames in tests) is written the same
  here as on the view bob's branch.
- **No protocol change.** The adapter is unchanged apart from the framework pointer.
