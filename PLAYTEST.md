# Playtest: shaping your face by hand

Save: rustvr-base

`playtest.bat` in this worktree (not committed: double-click it in Explorer) puts away any game left
running, then boots this build into **rustvr-base** (Danwood, a Nord in iron armour, in his house)
with the race menu open (`showracemenu`). The mirror stands in front of you and the race menu at its
left, as before. A key in the .bat's window closes the game and puts your settings back.

The research and the design are in `docs/research/facesculpt.md`.

## What to try, in order

1. **The copy of your head.**
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
   - Let go of the grip: it stays as you left it.
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

- **Builds and unit tests:**
  - the adapter (32-bit release);
  - the runtime, the range and xrsim (release);
  - the solver's, the face controls' and the protocol's tests, and the audio and runtime crates'.
- **FaceGen's controls**, read from the game's `si.ctl` into the asset cache (`facegen.toml`): the
  56 named controls, the age and the gender. The named controls are unit vectors and match the race
  menu's slider names and pages, as the game's menu has them.
- **The solver on the game's real head** (the Nord head from the cache, from the mean face, a 1 cm
  pull with the sliders' range at ±3):
  - the chin and a cheekbone follow 0.9 cm;
  - the nose tip 0.66 cm (its sliders reach their end);
  - the far side of the head moves up to 0.4 cm;
  - the age stays exactly where it was;
  - a nose pulled sideways doesn't go symmetric, and goes 1.05 cm asymmetric.

## What I couldn't check

- **Nothing of it has run in the game**, nor on any headset (emulated or real). The game can't be
  started from here. So all of these are untested:
  - the copy of the head being drawn;
  - its place and size;
  - the glow;
  - the live face;
  - the hands holding on;
  - the haptics;
  - the sounds.
- **The race menu's new buttons.** They're made from the menu's own button template, through the
  game's own calls, but never yet in the game. Watch for where they land in the lists, what they
  look like, whether clicking them works (the log says "the race menu's button N clicked"), and the
  game crashing as the race menu opens.
- **The game's own head morphed by a pull** (as its sliders do). It isn't seen in VR. If the game
  crashes as you first pull, this is the first suspect (the log says "the game's own heads morphed
  with the face" once).
- **The saved face**: whether the save keeps the pulled face as it keeps a slider's (it's the same
  data in the same place, so it should).
- **The asymmetric shape after loading.** The race menu has no sliders for it. The game draws and
  saves it, but check that a crooked nose survives a save and load.
- **Feel and tuning:**
  - how far the face follows the hand;
  - how wide a grab takes (2.2 cm on your face, 3.7 cm on the copy);
  - the copy's size and place;
  - the strength of the glow and the buzz.

  These are first guesses: tell me what feels off.
- **The sounds** (`COrganicSmall`, the game's sound of food set down) have never been heard here.
