# Playtest: the desktop view's effects studio, its bob as you walk, its zoom as you aim

Save: rustvr-archery

The desktop view (the window on the monitor, for recording and streaming) gets a studio of
effects only it shows. They'd make you sick in a headset, and they make footage look filmed. The
headset's view never changes.

Everything is in the RustVR menu's new **Studio** tab, and it's saved with the settings.

- **No black bars:** the letterbox is gone, as you asked after the first look.
- **The view bob branch is merged in here:** the desktop view bobs as you walk (below).
- **The aim zoom branch is merged in here too:** the desktop view zooms in while you aim a bow or a
  gun (below).
- **The desktop camera's own settings** sit at the top of the Studio tab's Camera section:
  - its field of view;
  - its zoom while you aim;
  - the overscan;
  - the bob.
- **The right stick scrolls the RustVR menu** (any tab, wherever the laser points), as it already
  scrolls the game's menus.
- **Fixed after the third playtest:** the thinner distant fog down the side of a wide desktop view
  with the overscan up (below).

The save is Weye's open ground on Lake Rumare's shore at 1 pm, with an Iron Bow equipped and 168
Iron Arrows. There's room to walk, run and jump, water to swim in, and a bow to aim: for the bob,
the zoom and the effects.

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
  - It turns at 160 degrees a second at its fastest: half the speed it had at first, so a 45 degree
    snap settles in about 0.7 s.
  - "Snap turn speed on the desktop view" in the Studio tab sets it (60 to 500).
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

**Fixed after the second playtest** ("something behind me getting stuck in the camera" on the
desktop view):

- **Standing still, Oblivion holds its camera 9.5 cm up and 3 cm to the side** of where the bob was
  measured from. That's its posture, not a bob.
  - The merged branch passed it on as one, which lifted the desktop camera off your eye. From there
    it could see into what the runtime draws behind your head.
  - It also sat the headset 10 cm lower than before.
  - Now only the camera's movement off where it has sat for the last second and a half counts as
    bob: the steps, a landing's dip and a swim's float.
- **The bob now moves the world, not your body.**
  - Before, moving the desktop camera only moved the body the runtime draws, the wrong way. The
    game's picture is looked up by direction, so the world never moved with the camera.
  - Now the bob turns the view by as much as it would move what's 2.5 m away, so the world bobs
    and your hands stay with the camera, as a flat game's arms do.
  - A landing's dip no longer drops the camera into your chest.

## The zoom as you aim (merged from the aim zoom branch)

The desktop view keeps its field of view until you aim. Then it eases down to the **aiming field of
view (65 degrees by default)**, and eases back out when you stop. It's the same view narrowed: no
scope picture, no crop. The headset's view never changes.

- **What counts as aiming:**
  - a gun with its sights lined up at your aiming eye (up to 75 cm ahead of it);
  - a bow drawn past half, with the string hand at your cheek or the corner of your mouth and the
    bow arm out.
- **It doesn't zoom:**
  - when a gun sweeps past your face;
  - when you hold a gun at your eye but look well away from it;
  - for a half draw, or a full draw at your chest.
- **A burst keeps it zoomed** through the whole magazine, and for half a second after.
- **While zoomed**, the desktop camera is steadied more, as far as the view is narrowed.
- **In the Studio tab**, under Camera: "Zooms in while you aim" (on), "Aiming field of view" (65),
  "Aiming zoom: in over" (0.25 s) and "out over" (0.33 s).
- **With a look's own lens** (Cinematic's 90 degrees), aiming still narrows to exactly the aiming
  field of view. The speed widening fades while you aim, and the bob is damped.

## The fog at the sides (fixed after the third playtest)

You saw less distant fog at the side of the desktop view, with the overscan up and its field of
view wide. You were right, on the left side.

- **Why:** Oblivion's shaders don't fog a thing by how far away it is.
  - They use the length of its clip-space position, which grows towards the edges of the eye's
    frustum. The same hill is foggier near the edge of an eye's picture than in its middle.
  - The desktop view is the right eye's picture. Past the right eye's left edge, it fills in from
    the left eye's. With your settings that edge is 51 degrees out: 11 degrees of overscan and a
    128 degree desktop view.
  - In the left eye's picture that hill sits nearer the middle of the frustum, so the game fogged
    it about a fifth less. That made a step in the fog at the join, and thinner fog past it.
- **Now:** where the left eye's picture fills in, the desktop view evens out the fog.
  - From the game's depth, it works out how much fog the game gave each pixel, and how much the
    right eye's picture would have given it.
  - The fog carries on past the join without a step.
  - **The headset doesn't change:** each eye only ever shows its own picture.
  - It's for Oblivion's fog only (Gamebryo's). Metro fogs by real distance.
  - The sky has no depth, and keeps its colour.
- **The headset has the same quirk, unchanged:**
  - your two eyes see a far hill off to the side fogged a little differently;
  - more overscan thins the fog a little.
  - Fixing those would mean changing Oblivion's shaders, and so the headset's picture.

## What to try

Watch the monitor (or OBS) while you play.

1. **Snap turns.** Flick the right stick.
   - The headset snaps.
   - The desktop view should turn through it as your head would: it gathers speed, swings, and
     settles with a slight overshoot. The world turns round your body, with no cut and no black
     edge.
   - Flick twice quickly: it should become one longer turn that keeps its speed.
   - The slider "Snap turn speed on the desktop view" sets how fast it turns. **Tell me** the
     speed you like and I'll make it the default.
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
   - Nothing should get in the desktop camera's face any more, standing or moving.
   - The world should bob, not your hands.
   - Your height in the headset standing should be as it was before the bob was merged.
   - Watch the recording, and check the headset never bobs.
   - With the game's own, walking is barely there; running, landing and swimming show it.
   - **Tell me** how each looks, and whether the strength should be different by default.
10. **The zoom.**
    - Take the bow off your back (left hand over the left shoulder) and an arrow over the right
      shoulder. Nock it and draw to your cheek: the desktop view narrows.
    - Hold it, look round, and loose: the view eases back out after the shot.
    - Draw without bringing your hand to your face: no zoom.
    - Try it in Cinematic too: it zooms to the same aiming field of view.
    - **Tell me** whether 65 feels right for Oblivion's bows, or you'd rather have more or less.
11. **The flat HUD.** Studio tab, "The game's HUD on the screen where a flat player has it".
   - The health bars, the compass and the effects should sit at the bottom of the desktop view, at
     about the size Oblivion has them on a flat screen.
   - Subtitles should sit low in the middle.
   - The activate icon should be in the middle when you look at something.
   - All of them are crisp, over any effects.
   - Open a menu: the HUD goes and the menus float in the world as before.
   - **Tell me** if anything is the wrong size or in the wrong place.
12. **Your own mix.** Every effect has its own switch and strengths below the looks.
    - Colour grades: teal and orange, bleach bypass, warm film, body cam, noir.
    - LUTs: put `.cube` files in `%LOCALAPPDATA%\RustVR\luts` and they show in the tab.

13. **The menu.** Open the RustVR menu and push the right stick forward and back: the tab
    scrolls, wherever the laser points.
    - The Studio tab's Camera section has the desktop field of view, the aim zoom, the overscan
      and the bob.
14. **The fog at the sides.**
    - Set the overscan to 10 or more and the desktop field of view to 120 or more.
    - On open ground with hills far off, the haze should be even across the desktop view.
    - There should be no edge in it down the left side.
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
- reading a `.cube` LUT;
- the fog where the other eye's picture fills in (on the GPU).
  - It uses two eye pictures fogged as Oblivion's shaders fog them, with your playtest's frustums
    and a 128 degree desktop view.
  - Before the fix, the fog past the join was 0.10 off the right eye's: a tenth of the way from
    clear to full fog.
  - After the fix, it's 0.006 off at worst, which is 8-bit rounding. That holds for far hills (found
    by direction) and near ones (followed through the depth).

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
  - Its fog evened out at the desktop view's side, over its real hills, water and distant land.
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
- **The aim zoom is merged in too** (framework and adapter), at your word.
  - Its settings and the desktop field of view moved from the Display tab to the Studio tab's
    Camera section.
  - With a look's lens, aiming still reaches the aiming field of view.
  - Its range has the bow on your back.
- **The protocol** has the view bob's `GameState::view_bob`, and nothing of the studio's.
