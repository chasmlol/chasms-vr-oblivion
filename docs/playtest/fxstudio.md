# Playtest: the desktop view's effects studio, its bob as you walk, its zoom as you aim

Save: rustvr-archery

## Playing it

From the repository's root:

```
$env:RUSTVR_DESKTOP_VIEW = "1"
powershell -File tools\oblivion.ps1 -Save rustvr-archery
```

`tools\oblivion.ps1` turns the desktop view off unless `RUSTVR_DESKTOP_VIEW` says otherwise, so
set it first in the same PowerShell window. It builds main, installs it, and boots into Weye's
open ground on Lake Rumare's shore at 1 pm, an Iron Bow equipped with 168 Iron Arrows: room to
walk, run and jump, water to swim in, and a bow to aim. `powershell -File tools\oblivion.ps1
-Close` puts it away. Watch the monitor (or OBS) while you play.

`rustvr-archery` is the developer's own save, not in the repository (`docs/playtest/archery.md`
says how to make one); any outdoor save does.

How the studio's built and why: `framework/docs/research/desktop-fx.md`. The bob's research:
`docs/research/view-bob.md`.

## What it does

The desktop view (the window on the monitor, for recording and streaming) has effects only it
shows, all in the RustVR menu's **Studio** tab and saved with the settings
(`framework/crates/runtime/src/spectator_fx.rs`, `spectator.rs`, `view_bob.rs`). They'd be
sickening in a headset and they make footage look filmed. The headset's view never takes them; it
changed in two ways only, both below: it no longer takes the game's camera animation, and the fog
goes by distance.

**The looks,** picked at the top of the tab; change anything by hand and it becomes "custom".

| Look | What it is |
|---|---|
| **Clean** (the default) | The game's picture as it is: no effects. Snap turns still glide. |
| **Cinematic** | A film's: a 90 degree lens, 24 frames at a 180 degree shutter's motion blur, focus pulled to what you look at, aim at or hold up, warm glowing highlights, fine grain, teal shadows. |
| **Hyper-real** | A body camera's (Unrecord, Bodycam): a wide lens bent towards a fisheye, worn on the chest (it turns after your body, a little down, swaying), a slow shutter's smear, hard sharpening, fringes and dark corners, a cold flat colour. |
| **Action** | Every shot and blow felt, the view widening at a run, punchy colour. |
| **Vintage** | Old film: heavy grain, a strong glow, faded warm colour. |

Below the looks every effect has its own switch and strengths: motion blur, wider at speed, shake,
handheld sway, worn on the chest, depth of field, the lens (fisheye, colour fringes, vignette,
glow, grain, sharpen), and colour (exposure, contrast, saturation, temperature, tint, the grades
teal and orange, bleach bypass, warm film, body cam and noir, and `.cube` LUTs put in
`%LOCALAPPDATA%\RustVR\luts`). There's no letterbox.

**For viewers,** whatever the look:
- **Snap turns, snap crouches and recentres turn as a head would** instead of cutting: the turn
  gathers speed, swings through and lands with a little overshoot, at 160 degrees a second at its
  fastest (a 45 degree snap settles in about 0.6 s). Two quick snaps become one longer turn. A step
  to the side dissolves rather than pans.
- **No black at the edges:** where a fast turn makes the desktop view see past the game's picture,
  the picture's edge is stretched over the gap.
- **The game's HUD where a flat player has it** (off by default): the bars, the compass and the
  subtitles at the screen's edges, from the game's own picture of them; the same pieces then stop
  floating in the world on the desktop, and the headset keeps them where they float.

**The bob as you walk** ("Bobs as you walk (never the headset)"):
- **off:** the camera glides;
- **the runtime's:** the framework's own, the same for every game: about 2 steps a second walking
  and 3 running, a drop at each footfall and a little sway over each foot, a soft dip landing from
  a jump or a fall, smaller crouched, a slow float swimming;
- **the game's own** (the default): Oblivion's first-person bob, read from its first-person
  animations moving the camera (`crates/oblivion/src/camera.rs`), the runtime's where the game
  gives none (third person). Walking is 5 mm a stride, running 1.9 cm a step, landing a jump a
  20 cm dip, swimming a slow float of up to 5 cm; with a weapon or fists out, walking and running
  don't bob at all (Oblivion's own data).

Only the camera's movement off where it has sat for the last 1.5 s counts as bob: standing still,
Oblivion holds its camera 9.5 cm up and 3 cm to the side, its posture, not a bob. The bob turns the
view by as much as it would move what's 2.5 m away, so the world bobs and your hands stay with the
camera, as a flat game's arms do. The headset no longer takes the game's camera animation (a
landing used to dip your view 20 cm); the Comfort tab's "Camera bob / shake" brings it back.

**The zoom as you aim:** the desktop view eases down to the aiming field of view (65 degrees) and
back out when you stop. It's the same view narrowed, no scope picture or crop.
- Aiming is a gun with its sights lined up at your aiming eye (up to 75 cm ahead of it), or a bow
  drawn past half with the string hand at your cheek or the corner of your mouth and the bow arm
  out, in both cases where your head looks.
- Not a gun sweeping past your face, a gun at your eye with your head looking well away, a half
  draw, or a full draw at your chest.
- A burst keeps it zoomed through the magazine and for half a second after.
- Zoomed, the desktop camera is steadied more, as far as the view is narrowed. With a look's own
  lens (Cinematic's 90 degrees) it still narrows to exactly the aiming field of view; the speed
  widening fades and the bob is damped while you aim.

**The fog by distance** (headset and monitor, `crates/oblivion/src/fog.rs`). Oblivion's vertex
shaders fog by the length of the clip-space position, which grows towards the frustum's edges: on a
flat 75 degrees it barely shows, but in the headset's wide, off-centre, mirrored eyes the two eyes
fogged a far hill to the side differently, its haze changed as you turned, overscan thinned it,
and the land didn't meet the water (whose shader does use the distance). As the game makes its
vertex shaders, the adapter rewrites the ones that fog (112 of the 329 in `shaderpackage004.sdp`,
the 15 older `vs_1_1` ones among them: the trees' leaves and fronds) so they fog by the distance;
each camera it draws with (each eye, the mirror) gives them its frustum. The fog's colour, start and
end are the game's own; far things to the side are a little foggier than before, the same in both
eyes and at any overscan. The desktop view's own evening-out of the fog where the other eye's
picture fills in stays off unless the game's shaders keep their own fog (a shader the adapter
couldn't rewrite, or `RUSTVR_OBLIVION_GAME_FOG=1` to compare).

The right stick scrolls the RustVR menu, any tab, wherever the laser points.

## What to try

1. **Snap turns:** flick the right stick: the headset snaps, the desktop turns through it, no cut
   and no black edge. Flick twice quickly. Try the speed slider, and Clean against Cinematic (whose
   pan is motion-blurred).
2. **Recentre** (the menu's Recenter, or after a load): it glides.
3. **Turning your head fast** with motion blur on (Cinematic, Hyper-real, Action): a camera pan's
   smear, more at the edges; a stretched edge, never a black band.
4. **Walking, running, a horse** in Cinematic and Action: the view widens with speed (galloping
   most) and things passing close smear.
5. **A sword or a mace** swung fast: the blade and hand streak, the rest sharp.
6. **A bow:** loose an arrow: the camera kicks a little.
7. **Hard hits and a high jump:** a blow that lands shakes the camera (Action most); a hard landing
   dips it.
8. **Depth of field** (Cinematic, Vintage): near then far; a potion or a book held up close.
9. **The bob:** walk, run, sneak, jump and swim with each of its three choices. Nothing should get
   in the desktop camera's face; the world bobs, not your hands; your standing height in the
   headset is unchanged, and the headset never bobs.
10. **The zoom:** the bow off your back (left hand over the left shoulder), an arrow over the right
    shoulder, nock and draw to your cheek: the desktop narrows; loose, and it eases out. Draw
    without your hand at your face: no zoom. Again in Cinematic.
11. **The flat HUD:** the bars, compass and effects at the bottom at about the flat game's size,
    subtitles low in the middle, the activate icon in the middle, all crisp over any effects; open
    a menu and the HUD goes, the menus floating as before.
12. **Your own mix** and the LUTs.
13. **The menu:** push the right stick forward and back on any tab. The field of view slider moves
    the look's own lens where it has one (the look becomes custom); in Clean it's your own.
14. **The fog** on open ground with hills far off: one eye then the other, a far hill to the side
    as hazy in each; its haze staying put as you turn; the overscan changed and the fog the same;
    the land and the lake far off fogged alike; on the monitor, with overscan at 10 or more and the
    desktop field of view at 120 or more, the haze even across.

## Settings

The Studio tab (`framework/crates/runtime/src/spectator_fx.rs`'s `panel`; the keys in
`settings.toml`'s `[studio]`, `[spectator]`, `[desktop_bob]` and `[vr]`):

| Setting | Key | Default | Range |
|---|---|---|---|
| Look: Clean, Cinematic, Hyper-real, Action, Vintage | `studio.look` | Clean | |
| Snap turns, snap crouches and recentres turn as a head would, instead of cutting | `studio.smooth_snaps` | on | |
| Snap turn speed on the desktop view (its fastest) | `studio.snap_speed` | 160 deg/s | 60 to 500 |
| Stretch the picture's edges over the gaps a fast turn leaves (instead of black) | `studio.fill_edges` | on | |
| The game's HUD on the screen where a flat player has it | `studio.flat_hud` | off | |
| HUD size | `studio.flat_hud_size` | 1 | 0.5 to 2 |
| Desktop field of view (a look's own lens where it has one) | `spectator.fov` | 105 deg | 60 to 140 |
| Zooms in while you aim (a gun's sights at your eye, a bow drawn to your cheek) | `spectator.aim_zoom` | on | |
| Aiming field of view | `spectator.aim_fov` | 65 deg | 30 to 100 |
| Aiming zoom: in over | `spectator.aim_in` | 0.25 s | 0.05 to 1 |
| Aiming zoom: out over | `spectator.aim_out` | 0.33 s | 0.05 to 1 |
| Overscan: the game drawn past the headset's view (also on the Display tab; a change takes at the next start) | `vr.overscan` | 0 deg | 0 to 20 |
| Bobs as you walk (never the headset): off, the runtime's, the game's own | `desktop_bob.mode` | the game's own | |
| Bob strength | `desktop_bob.strength` | 1 | 0 to 3 |

Elsewhere: the Display tab's "Game picture from the left eye / right eye (your aiming eye)"
(`spectator.eye`, right), the desktop camera's smoothing and lag, a level horizon and fullscreen;
the Comfort tab's "Camera bob / shake" (`comfort.head_bob`, off), which gives the headset the game's
camera animation. Testing: `RUSTVR_OBLIVION_GAME_FOG=1` keeps the game's own fog.

## Decisions

- **Only the desktop view, never the headset:** the effects would make you sick. The fog by
  distance changes the headset's picture too, at the user's word after the third playtest.
- **No black bars** (asked after the first look).
- **Snap turns at 160 degrees a second:** twice that, the first playtest's, was quicker than the
  player liked.
- **The bob and the aim zoom live in the Studio tab's Camera section,** merged in from their own
  branches at the user's word; the Display tab points there.
- **The game's own bob by default,** what its flat game shows; its posture left out, and turned
  into a turn of the world's camera so nothing behind your head comes into view.
- **The zoom only for a real aim,** held through a burst: the zoom pumping in and out with each kick
  would be worse than its staying.
- **The fog's distance only:** its colour, start and end stay the game's.
- **The protocol** carries the view bob (`GameState::view_bob`) and, in `GameLighting::fog_fade`'s
  spare `z`, 1 when the game's shaders fog by distance.

## Not done / known issues

- **Taking damage doesn't shake the camera:** Oblivion doesn't tell the runtime when you're hit.
- **Never checked in Oblivion on the headset** (the studio was tuned on the test range): the flat
  HUD laid out from the game's picture of it; the game's depth for focus and blur; a real snap
  turn, recentre and horse; Oblivion's own bob in play (the adapter's log says every 5 s how far
  it went, `view bob: the game's own in the last 5 s: ...`, and the runtime's what the desktop
  showed, `desktop view: bobbed ...`); the headset no longer taking the game's bob; the fog by
  distance over the real hills, water, trees and distant land, and the game's other fog (its sky,
  interiors, water's reflections).
- **The headset's timing:** the effects run after the headset's frame is handed over, about a third
  of a millisecond on the GPU; the desktop view logs its GPU time in `rustvr.log` (`desktop view:
  ... ms on the GPU for its picture, ... ms for its effects and HUD`). Whether they ever cost the
  headset a frame is unchecked.

## History

Newest first.

- **Playtest 3:** less distant fog down the side of the desktop view with the overscan up and its
  field of view wide; the headset had the same quirk. The game's fog now goes by distance.
- **Playtest 2:** "something behind me getting stuck in the camera" on the desktop view. The merged
  view bob passed the game's standing posture on as bob, lifting the desktop camera off the eye
  (and sitting the headset 10 cm lower); moving the camera moved the body the runtime draws, the
  wrong way, not the world. Now only movement off the last 1.5 s counts, and the bob turns the
  world's camera.
- **Playtest 1:** the letterbox went, and the snap turn's speed was halved. The view bob and aim
  zoom branches were merged in.
