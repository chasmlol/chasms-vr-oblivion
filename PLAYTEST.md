# Playtest: shaping your face by hand

Save: rustvr-base

`playtest.bat` in this worktree (not committed: double-click it in Explorer) puts away any game left
running, then starts a **new game** with this build: the main menu's New, answered yes, its film
skipped. You're in the prison cell, and the race menu opens by itself, as it does for a player
starting out. The mirror stands in front of you and the race menu at its left. A key in the .bat's
window closes the game and puts your settings back. The runtime's log is kept in
`<builds>\facesculpt\playtest-logs` as the game closes, however the window is closed (last time
another test replaced it before it was kept).

The save above (rustvr-base, a Nord in iron armour, with `showracemenu`) is the other way in:
`tools\play.ps1 facesculpt -Commands showracemenu`.

The research and the design are in `docs/research/facesculpt.md`.

## What changed since the second playtest

- **The .bat left you at the main menu.** It answered New on the menu's very first frame, while the
  game was still fading the menu in, and the game's close does nothing to a menu not yet shown. So
  the menu stayed, with the rest of the answer done: the character generation quest started and a
  new game's loading flagged. That half-started game is likely what crashed the save you then loaded
  from the menu (the crash was in its loading screen). Now the .bat waits until the menu is shown,
  done fading in and not showing a box of its own, as it is when a player clicks it, and the log
  says whether the menu began to close (`console: NewGame -> ran`, or `-> failed`). In the third
  run it did: the main menu closed and the race menu opened in the prison cell 8 s later.
- **The third run went at 4 to 5 frames a second.** That wasn't this feature: the runtime's
  Supersampling was 1.55 (saved in `%LOCALAPPDATA%\RustVR\settings.toml` at 13:52, during another
  feature's test), so the game drew each eye at 7237x7791 rather than 4669x5026, 2.4 times the
  pixels, and the GPU ran out. Handing each eye's picture over took about 70 ms and its depth about
  55 ms, where both had taken under 1 ms. The first playtest had the race menu, the mirror and the
  copy of the head up at 1.0 and ran at 70 to 95 frames a second. Play at 1.0 (the in-headset
  settings' Supersampling).

## What changed since the first playtest

- **A pull moved the whole head.** That had three causes, all fixed:
  - **The solve let the rest of the face move as much as it took** for the part held to follow. Now
    the face more than 4 to 8 cm from the point is held still, as is the area round its mirror image
    for a symmetric pull, at ten times the weight of the part held following the hand. On the game's
    Nord head, a 1 cm pull:

    | | the part held follows | the far face moves (rms) | the far face moves (most) |
    |---|---|---|---|
    | before | 0.69 cm | 0.22 cm | 0.88 cm |
    | now | 0.64 cm | 0.07 cm | 0.34 cm |

  - **Stroke after stroke took the face far past any the game makes.** In your run one of its
    coordinates reached −28.6, where every part warps. The sliders' ranges barely hold some mixes of
    the coordinates. Now every coordinate also stays within ±8 of the mean face, and what goes past
    is taken back without moving the rest.
  - **The copy of the head jumped half a second after you let go**, when the body was put together
    with the new face: it was centred on the face as shaped. Now it's centred on the head with no
    face on it, so it stays put.
- **Some of it is FaceGen itself.** Its coordinates each move the whole face, so a change to one part
  alone is only ever nearly one. The chin especially: FaceGen has no chin of its own (the chin
  sliders move the jaw and mouth too). So a chin pulled down now mostly holds your hand back, where
  before it stretched the whole face.
- **The .bat starts a new game**, at the race menu in the prison cell.

## What to try, in order

1. **The copy of your head.**
   - The race menu opens by itself in the prison cell (with your new character's race and face).
   - A copy of your own head stands in front of you to the right, a bit under your eyes, 1.7 times
     life size, its face turned to you. It's your race's head with your face, hair and eye colour,
     lit by the room.
   - The mirror still shows you, and the race menu still stands at its left.
2. **Grab the nose and pull.**
   - Bring a hand to the copy's nose. Before you squeeze, the part your grip would take glows a soft
     gold (the game's cursor colour), brightest at the middle, brighter the closer your hand is.
   - Squeeze the grip. Nothing jumps: your hand stays where it is, and its fingers close on the point
     you took.
   - Pull it out towards you: the nose gets longer. Push it in: shorter. Up and down tilts it.
   - The face changes live, on the copy and on you in the mirror.
   - You feel a fine buzz as it moves, and a harder one as you pull past what the face can do. Your
     hand stays on the nose where it got to (with ghost hands on in the RustVR menu, a see-through
     one shows where your controller is).
   - Let go of the grip: it stays as you left it. The copy stays where it stands (it used to jump half
     a second later).
   - Pull the nose long: the ears, the back of the head and the top of the skull stay where they
     were. Round the nose (its bridge, the nostrils, the upper lip) the face goes with it a little.
   - Pull your chin down: it barely goes, and your hand is held back with a buzz. That's FaceGen
     having no chin of its own (the Chin page's sliders move the jaw and mouth too).
   - Try more:
     - the chin forward or down;
     - the cheekbones up or out;
     - the jaw wider;
     - the brow lower;
     - the eyes apart (take an eye or beside it and pull outward);
     - the lips fuller;
     - the ears out.
3. **Symmetric versus asymmetric.**
   - Pull one cheek outward: both cheeks widen together (symmetric, the default).
   - Pull the nose sideways: it doesn't go. A symmetric face can't have a crooked nose.
   - Now take the nose again and squeeze the **trigger** of that hand while you pull it sideways. It
     bends to that side. Let the trigger go while still holding and it eases back to symmetric.
   - Both hands can pull at once.
4. **Your own head.**
   - Reach up to your own chin, jaw, ears or mouth (the headset is where your nose and eyes are, so
     those you do on the copy). It glows in the mirror where your grip would take.
   - Squeeze and pull your chin down or your ears out, and watch it in the mirror.
5. **The sliders follow.**
   - Open Face, then Shape, then a page (Nose, Chin...) with the laser.
   - Pull that feature on the copy: the page's sliders move as you pull.
   - Move one of those sliders: the copy and the mirror follow it. Then pull again from there.
   - Open another page after pulling: its sliders show the new face.
   - The Age and Complexion sliders (Main page) don't move when you pull.
6. **Undo and reset.**
   - New buttons in the race menu, made like its own:
     - on the Main page, at the bottom of the list: **Undo** and **Reset Face**;
     - on each Shape page (General, Brow, Cheeks, Chin, Eyes, Forehead, Jaw, Mouth, Nose), at the
       bottom: **Undo** and **Reset**.
   - **Undo** puts the face back as it was before your last pull (or reset). Click it again to go
     back further.
   - **Reset** on a page puts that page's features back to your race's own (e.g. the Nose page's
     Reset gives you your race's nose and leaves the rest).
   - **Reset Face** puts the whole face back to your race's own, keeping your age and complexion.
7. **Done, and the saved face.**
   - Press the race menu's Done.
   - The copy goes. Look in a mirror (the hands-apart gesture, or the Body tab): your face is the one
     you made.
   - Save, load that save, and look again: it should be the same face.
   - Talk to someone close (or wait): your face should be the same from the game's side too.

## What was checked

- **Your first run (the adapter's log):** the race menu's new buttons were made on all ten pages, the
  game's own head was morphed with the face (9 parts), and nothing crashed. There's no click on Undo
  or Reset in the log, so those are still untried. The runtime's log of that run was replaced by
  another test's before it was kept.
- **Builds and unit tests:**
  - the adapter (32-bit release);
  - the runtime, the range and xrsim (release);
  - the solver's tests (with a new one: a face pulled past its slider and the coordinates' range
    comes back within them, its age kept, the far end where it was), and the adapter's.
- **The solver on the game's real Nord head** (all its face parts, from the mean face): the table
  above. Seven places were pulled: nose tip forward, chin down, cheekbone out, brow forward, jaw
  corner out, upper lip forward, ear out. The far face is everything more than 6 cm from the point
  and from its mirror image.
  - A hold is taken in 1.3 to 2.8 ms.
  - Forty random 1.5 cm strokes, one after another: every coordinate stays within 8, every slider
    within 5, the age exactly where it was. Keeping it there takes at most 0.16 ms a frame. The first
    version's coordinates reached 28 in the same strokes.
- **The new game:** the game's own steps (the main menu's New, its "Start new game?" box,
  which button is Yes, and when a menu fading in can be closed). The second playtest's log shows
  the first try ran on the menu's first frame and left it up; the third shows the fixed one start
  the new game, the race menu open in the cell and the face ready to be shaped by hand.

## What I couldn't check

- **Nothing of it has run in the game since the fixes.** The game can't be started from here.
- **The new game's film:** whether it was skipped (the race menu came 8 s after the main menu
  closed, loading included).
- **Feel:** whether the face now follows the hand enough for you, now that the rest stays still. The
  balance (how still the rest is held against how far the part follows) is one number, easy to move
  either way: tell me which way.
- **Still untested from the first round:**
  - Undo, Reset and Reset Face;
  - the trigger's asymmetric pull;
  - your own head in the mirror;
  - Done, then a save and load keeping the face (a crooked one too);
  - the sounds.
