# Playtest: the desktop view zooms in while you aim

Save: rustvr-archery

The desktop view (the window on your monitor, for recording and streaming) keeps its wide field of
view (105 degrees by default) until you aim. Then it eases down to an **aiming field of view (65
degrees by default)** and eases back out when you stop. This happens with a gun's sights at your
eye, or a bow drawn to your cheek. It's the same view narrowed, nothing else: no scope picture, no
crop. The headset's own view never changes.

Two `.bat`s in this worktree (not committed):

- **`range.bat`**: the framework's test range from this build (`play.ps1 aimzoom -Range`).
  - Like every feature's range, it first puts away whatever range is up, the melee chat's too.
- **`playtest.bat`**: Oblivion booted into `rustvr-archery` (Weye by Lake Rumare at 1 pm, an Iron
  Bow equipped, 168 Iron Arrows), with the desktop view on.
  - `tools\oblivion.ps1` normally switches the desktop view off (`RUSTVR_DESKTOP_VIEW=0`), so the
    `.bat` sets it to 1 first.
  - It waits while you play, then closes the game and keeps the runtime's log in
    `<builds>\aimzoom\playtest-<n>`.

Keep an eye on the monitor (or record it) while you play. The settings are in the RustVR menu's
Display tab, under "Desktop field of view":

- "Desktop view zooms in while you aim": on by default;
- "Aiming field of view": 65 degrees;
- "Aiming zoom: in over" (0.25 s) and "out over" (0.33 s).

They're saved with the other settings (`[spectator]` in `settings.toml`: `aim_zoom`, `aim_fov`,
`aim_in`, `aim_out`).

## What to try in the range

1. **The rifle.** Take the range rifle from the near table at your right hand.
   - Hold it at the hip: the desktop view stays wide.
   - Raise it until the rear notch and front post line up in front of your right eye. About a
     sixth of a second later the view narrows over a quarter of a second, onto the sights.
   - Lower it: the view widens over a third of a second.
   - No jump either way.
2. **Not aiming.**
   - Sweep the gun across in front of your face at an ordinary speed: no zoom.
   - Hold it at your eye but look well away from where it points: no zoom.
3. **A burst.** Aim and hold the trigger down: the zoom stays through the whole magazine, even
   one-handed with the gun climbing. It stays for half a second after the trigger too, while the gun
   comes back.
4. **The pistol** (right hip). Hold it out at arm's length with its sights at your eye: that's aiming
   too (sights up to 75 cm ahead of the eye count).
5. **The bow** (new to the range).
   - It's on your back over the left shoulder: reach back with the left hand and grip.
   - With the bow in the left hand, reach over your right shoulder and grip: an arrow.
   - Nock it, and draw to the corner of your mouth or your cheek: the view narrows.
   - A half draw, or a full draw with the hand down at your chest, doesn't zoom.
   - Let the string down, or loose: it widens.
   - A loosed arrow flies nowhere in the range: there's no game to fly it.
6. **Settings, live.**
   - Change "Aiming field of view" while aiming (try 50 and 80).
   - Turn the zoom off: aiming does nothing.
   - The ease times are there too.
7. **Steadiness.** While zoomed, the desktop camera's smoothing and its lag limit grow as the view
   narrows (105/65: 1.6 times, 0.10 s to 0.16 s and 8 to 13 degrees). Small head shakes shouldn't
   jitter the zoomed view. A fast turn while aiming trails a little more than unzoomed.

## What to try in Oblivion

`playtest.bat`, then in the headset:

1. Take the bow off your back (left hand, over the left shoulder), an arrow over the right
   shoulder, nock, draw to your cheek: the desktop window narrows.
2. Hold, look round, loose: it eases back out after the shot.
3. Draw again but don't bring the hand to your face: no zoom.
4. Does 65 feel right for Oblivion's bows, or would you rather have more (or less)?

## What was checked, and what wasn't

Checked in the test range on the emulated headset. The range ran on its own link and settings
(`rustvr.exe --launch-range --mapping Local\RustVR_aimzoom`), so it couldn't share the melee chat's
link, and only while no other runtime, range or emulator was up. Scripts and captures are in
`<builds>\aimzoom\scripts` and `runs`.

- **Gun** (`gun1`, `gun2`; the range rifle taken with `RUSTVR_TEST_TAKE`):
  - at the hip: wide;
  - raised: the sight line passed 0.000 m from the right eye, and zoomed in 0.15 s after reaching
    it;
  - the zoomed capture shows the notch and post lined up on the far target;
  - lowered: back out;
  - a sweep across the eye at 0.5 m/s: no zoom;
  - a one-handed 3 s burst while aimed: it stays zoomed throughout. Before the fix it dropped out
    two seconds in: the sights went 5.5 cm off the eye and the muzzle climbed 44 degrees.
- **Bow** (`bow1`, `bow2`; the range's new bow):
  - taken off the back, an arrow from the quiver, nocked: wide;
  - half drawn (47%, the hand 22 cm from the anchor): wide;
  - drawn to the anchor: zoomed;
  - let down: back out.
- **Easing**, frame by frame (a temporary trace, since removed):
  - in over 28 frames (0.23 s at 120 Hz), out over 38 (0.32 s);
  - smooth at both ends, at most about 2 degrees a frame in the middle;
  - the smoothing and lag limit eased with it.
- **Unit tests:**
  - `rustvr-runtime --lib`: all 218 pass.
  - `rustvr-range`: 25 pass and one fails,
    `throwables::tests::the_layout_fills_the_pouches_and_the_table`. That failure isn't from this
    work: the test's list has no sword but `layout()` unwraps `builtin_sword` (since `eae28ea`,
    the melee merge). I left it for the melee chat.

Not checked:

- **Oblivion**: not started overnight, as agreed. Its bows are the runtime's own bow (`OBJ_BOW`,
  the same code the range's bow uses), so they should zoom the same.
- **The real headset**, and how 65 degrees looks recorded.
- **Metro's guns**: no Metro assets here. They go through the same gun pose as the range's
  (`MechOutput::weapon`, in rig mode and in legacy mode), so they should zoom the same.
- **The headset's view staying put**: not checked by eye. The change only touches the desktop
  view's own field of view and camera, never what's sent to the headset.

## The calls made (nobody was up to ask)

- **The aiming field of view: 65 degrees.**
  - That's about what a monitor fills of a viewer's sight at a desk (a 27 to 32 inch screen at arm's
    length is 55 to 65 degrees), so what you aim at shows near the size you see it in the headset.
  - It's twice as big in the middle as at 105, with the sights and what's round the target still
    in view.
  - Under 60 starts to feel like a scope; over 75 barely reads as a zoom from 105.
- **The aiming eye** is the desktop view's own eye setting ("Game picture from the right eye" by
  default). If you aim with your left eye, set it to the left.
- **A gun aims when:**
  - its line of sight passes within 2.5 cm of that eye (staying aimed: 4.5 cm);
  - its sights are no more than 75 cm ahead of the eye: a pistol at arm's length counts, a gun lined
    up by chance across the room doesn't;
  - it points within 25 degrees of where your head looks (staying: 37), as the runtime's own aiming
    down the sights has it.
- **A bow aims when:**
  - an arrow is drawn past half the full draw (staying: 40%);
  - the string hand is within 12 cm of a point 6 cm under the aiming eye (staying: 17 cm). That
    covers the cheekbone, the corner of the mouth, under the jaw and the ear;
  - the bow hand is at least 40 cm out (staying: 32 cm), within 25 degrees of where you look.
- **No flicker:**
  - it zooms in after 0.15 s of aiming and out after 0.12 s of not;
  - the staying tests are looser than the starting ones.
- **Steadier while zoomed:** the smoothing and lag limit grow with the ratio of the fields of view
  (1.6 times), not the true magnification in the middle (2 times). At 2 times the lag limit would
  be 16 degrees, nearly the zoomed view's half height. At 1.6 times it's 13 degrees, and what you
  aim at stays in view on a fast turn.
- **Firing:** a gun stays aimed through a burst and half a second after. The limits then are
  12 cm off the eye and 60 degrees off where you look.
- **Spells and staffs don't zoom**: they have no sights. Melee doesn't either.

## Every game

The zoom works from the runtime's own held weapons, so a game gets it for free if its weapons are
ones the runtime knows:

- **Guns:**
  - an object of kind `OBJ_WEAPON` with its `sight` on the line of sight (weapon +Z along it);
  - or, in legacy mode, the game's gun rig with its sight line (`RIG_HAS_SIGHT`).
  - Metro's guns are both.
- **Bows:** an `OBJ_BOW` with its string (`handle`, `handle_travel`). Oblivion's are.

A game whose held weapon is neither won't zoom: say, a crossbow drawn as a prop, or a gun the game
holds itself. Its adapter would need to register the weapon as an `OBJ_WEAPON` with its sight line
(a crossbow aims as a gun does). Nothing in Oblivion or Metro needed a hook.

## Also noticed

- At full draw the string passes a few centimetres in front of the aiming eye.
  - Anchored right under the eye, it shows as a pale band down the middle of the desktop view,
    zoomed or not, and the headset shows the same.
  - Anchored at the corner of the mouth, it's off to the side by the nose, as an archer sees it
    (`bow2`), and the target is clear.
  - If it's in the way in Oblivion, the runtime's draw of the bow could skip what's within a few
    centimetres of the desktop camera. I haven't done this.
- The emulator (`rustvr-xrsim run`) exits with a segmentation fault after every run, after the
  session has ended and everything has been saved. I haven't looked into it.
