# Playtest: shaping your face by hand

Save: rustvr-base

## Playing it

From the repository's root, a new game:

```
powershell -File tools\oblivion.ps1 -NewGame
```

It builds main, installs it, and starts a new game as a player starting out has it: the main
menu's New, answered yes, its film skipped. You're in the prison cell, the race menu opens by
itself, the mirror stands in front of you and the race menu at its left.
`powershell -File tools\oblivion.ps1 -Close` puts it away and your settings back. The adapter's
log says whether the main menu began to close for New (`console: NewGame -> ran`, or `-> failed`).

The other way in is a save and the console's `showracemenu`:

```
powershell -File tools\oblivion.ps1 -Save rustvr-base -Commands "showracemenu"
```

`rustvr-base` (a Nord in iron armour) is the developer's own test save, not in the repository; any
save does, and one can be made from any save with `-At <cell>`, saved as `rustvr-base`
([DEVELOPING.md](../DEVELOPING.md), "Test saves").
`tools\oblivion.ps1` turns the desktop view off: set `$env:RUSTVR_DESKTOP_VIEW = "1"` first to see
the copy of the head on the monitor.

The research and the design: `docs/research/facesculpt.md`.

## What it does

The runtime's `framework/crates/runtime/src/sculpt.rs` and its solver
(`framework/crates/avatar/src/sculpt.rs`), and the adapter's `crates/oblivion/src/facesculpt.rs`.

- **A copy of your head** stands in front of you to the right, a bit under your eyes, 1.7 times
  life size, its face turned to you: your race's head with your face, hair and eye colour, lit by
  the room. It's centred on the head with no face on it, so it stays put as the face changes.
- **Taking hold.** A palm near the face shows what its grip would take: the vertices round the
  nearest point glow the game's cursor gold, most where they'd move most, brighter the closer the
  hand. The grip takes it. Nothing snaps: the hand stays where it is, its fingers closing on the
  point.
- **Pulling** is the hand's move against the head: the face is solved for it every frame, live on
  the copy and on you in the mirror. The part taken goes as far with the hand as the face can take
  it (its FaceGen coordinates, the sliders' ranges); the hand is drawn holding it where it got to,
  a fine buzz as it moves and a harder one when held back. The face more than 4 to 8 cm from the
  point is held still (and round its mirror image, for a symmetric pull), at ten times the weight
  of the part following the hand; every coordinate also stays within 8 of the mean face, what goes
  past taken back without moving the rest.
- **Symmetric by default:** one cheek pulled out widens both; a nose pulled sideways doesn't go.
  Squeeze that hand's trigger while pulling and the pull goes asymmetric (a crooked nose, one ear
  out); let go of the trigger and it eases back to symmetric. Both hands can pull at once.
- **Your own head** can be taken too, where the headset doesn't sit (chin, jaw, ears, mouth): it
  glows in the mirror where the grip would take.
- **The sliders follow.** The page on show is refreshed from the face as you pull, and a page shown
  later as it comes up; a slider moved moves the copy and the mirror. The sliders' range is the
  race's (the general page's sliders to the race's face clamp, 3 by default; the rest its main
  clamp, 5). Age and complexion don't move with a pull.
- **Undo and reset,** buttons made from the race menu's own button template: on the Main page,
  Undo and Reset Face (the race's own face, age and complexion kept); on each Shape page (General,
  Brow, Cheeks, Chin, Eyes, Forehead, Jaw, Mouth, Nose), Undo and Reset (that page's features back
  to the race's own). Undo goes back to the face before the last pull or reset, again and again;
  the menu closing forgets them.
- **The game keeps the face:** each face is put into the character as the sliders would (its own
  coordinates, the game's FaceGen heads morphed), so it's the one the game saves and its people
  see.
- **Sounds:** a squish taking hold and letting go (the sound pack's; `oblivion-extract sounds`
  makes the pack).

## What to try

1. **The copy:** the race menu opens by itself in the cell; the copy stands to the right, the
   mirror ahead, the race menu at its left.
2. **The nose:** bring a hand to it and watch the glow; squeeze; pull it out (longer), push it in
   (shorter), up and down (it tilts). Let go: it stays, and the copy doesn't jump. Pull it long: the
   ears, the back and the top of the skull stay where they were; the bridge, nostrils and upper lip
   go with it a little.
3. **The chin:** pull it down: it barely goes and your hand is held back with a buzz (FaceGen has
   no chin of its own: the Chin page's sliders move the jaw and mouth too). Then the chin forward,
   the cheekbones up or out, the jaw wider, the brow lower, the eyes apart, the lips fuller, the ears
   out.
4. **Symmetry:** one cheek out; the nose sideways (it won't); the nose sideways with the trigger
   squeezed (it bends), then the trigger let go while still holding.
5. **Your own head:** your chin down or your ears out, watching the mirror.
6. **The sliders:** Face, then Shape, then a page (Nose, Chin...) with the laser; pull that feature
   and watch the sliders; move one and pull again from there; open another page after pulling.
7. **Undo, Reset, Reset Face.**
8. **Done:** the copy goes. Look in a mirror (pull your gripped hands apart in front of your chest,
   or the Body tab): your face is the one you made. Save, load, and look again; talk to someone.
9. **The pause menu** in the race menu: the RustVR settings are beside it.

## Settings

| Where | Setting | Default | What it does |
|---|---|---|---|
| Hands tab | Ghost hands (a see-through hand where your real one is when it's held back), `interaction.ghost_hands` | off | Shows the controller while the hand holds the face where it got to. |
| Body tab | Pull your gripped hands apart in front of your chest to call it up or put it away, `mirror.gesture` | on | The mirror, to see your face after Done. |
| Body tab | The game's world in the mirror (off: you alone, on dark glass), `mirror.game_world` | on | |

## Decisions

- **On a copy of the head as well as your own:** every feature can be reached on it (the headset
  sits where your nose and eyes are), it's big enough to take a nostril or a lip, and the hands on
  it don't hide the mirror.
- **Through FaceGen's own coordinates,** so the face is one the game makes, saves and shows. Each
  coordinate moves the whole face, so a change to one part alone is only ever nearly one; the chin
  especially, since FaceGen has none of its own.
- **The rest of the face held still** against the part held, at ten times its weight: the balance
  (how still the rest, against how far the part follows) is one number, easy to move either way.
- **Coordinates kept within 8 of the mean face:** stroke after stroke took one to -28.6, where every
  part warps, and the sliders' ranges barely hold some mixes of them.
- **New game from the main menu as a player clicks it:** the boot waits until the main menu is
  shown, done fading in and not showing a box of its own. The game's close does nothing to a menu
  not yet shown, so New answered on its first frame left the menu up, half a new game started; a
  save then loaded from the menu crashed in its loading screen.
- **The race menu is the menu shown while it's open,** and the pause menu over it is the one shown
  then, with the RustVR settings beside it.

## Not done / known issues

- **Never checked on the headset since the second round's fixes:** whether the face now follows the
  hand enough (tell which way the balance should move); Undo, Reset and Reset Face; the trigger's
  asymmetric pull; your own head in the mirror; Done, then a save and load keeping the face (a
  crooked one too); the sounds; whether the new game's film is skipped.
- **Slow frames at high supersampling, once:** two runs with the runtime's Supersampling at 1.55
  (each eye 2.4 times the pixels) ran at 3 to 5 frames a second with the race menu open, the GPU
  full; a later run at the same size ran at 60, so the cause was likely outside the game. In the
  race menu the game draws the world a third time, at the eyes' size, for the mirror. If it comes
  back, the adapter's `timing:` line ends with `video memory <used> of <budget> MB` (and
  `(over its budget)` when it is) and times `the mirror's world` on its own, and the runtime's
  `pacing:` line ends with its own video memory: over budget means memory; under it, with the
  mirror's world large, the third render.

## History

Newest first.

- **Playtest 2:** the boot left the game at the main menu (New answered on its very first frame);
  two runs went at 3 to 5 frames a second at 1.55 supersampling; the desktop view showed no copy of
  the head; the menu button in the race menu paused the game with no RustVR settings; the squish of
  taking and letting go was loud. Now the boot waits for a main menu a player could click, the
  timing lines report video memory and the mirror's world, the desktop view draws the copy, the
  pause menu has the settings, and the squish is half as loud.
- **Playtest 1:** a pull moved the whole head, for three reasons: the solve let the rest of the
  face move as much as it took (now held still past 4 to 8 cm: on the game's Nord head a 1 cm pull
  moved the far face 0.07 cm rms, not 0.22), stroke after stroke took the face far past any the
  game makes (now kept within 8), and the copy jumped half a second after letting go (now centred
  on the head with no face). The boot became a new game at the race menu.
