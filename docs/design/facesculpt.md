# Shaping the face by hand: research and design

In the race menu the player takes hold of their face with the controllers' grips and pulls its parts
into shape, instead of (or as well as) moving sliders. Every change goes into the face's own FaceGen
coefficients, the values the race menu's sliders set, so the face shaped is the one the game saves,
loads and shows its people. This is what Oblivion's FaceGen data and race menu are, how other tools
do hands-on sculpting, and the design built from them: the runtime's `sculpt.rs`, the avatar's
`sculpt.rs` (the solve) and the adapter's `facesculpt.rs`. The mirror it builds on is in
`docs/design/mirror.md`.

## FaceGen in Oblivion

**A face is a point in a statistical space.** The player's face is four coordinate vectors:

| Vector | Count | What it moves |
|---|---|---|
| symmetric geometry | 50 | both sides of the face alike |
| asymmetric geometry | 30 | one side against the other |
| symmetric texture | 50 | the skin's colour (complexion, beard, eyebrows, lips) |
| asymmetric texture | 0 | |

The face drawn is the race's coordinates plus the character's. Each head part's `.egm` holds a
morph per coordinate: every vertex's move at a coordinate of 1 (`FREGM002`: a scale and an `int16`
x, y, z per vertex). A vertex's place is its rest place plus `Σ coordinate · move`. The `.egt` does
the same for the head texture.

**`FaceGen\si.ctl` names directions in that space.** It's in `Oblivion - Misc.bsa` (`FRCTL001`):

- Four lists of named linear controls, each a coefficient vector over one of the coordinate sets
  and a name. All are unit vectors.
  - 56 over the symmetric geometry: "Brow Ridge - high / low" .. "Temples - thin / wide".
  - 26 over the asymmetric geometry: "Nose - tip transverse shift", "Face - coronal bend" ...
  - 33 over the symmetric texture: "Beard Light / Dark", "Skin Tint Red / Green" ...
  - None over the asymmetric texture.
- Then FaceGen's own linear controls, each 50 coefficients and an offset:
  1. age over the symmetric geometry (`age = Σ c·gs + 29.7151`);
  2. age over the symmetric texture (offset 32.12);
  3. gender over the symmetric geometry (offset −0.1956);
  4. gender over the symmetric texture (offset −0.58).

A named control's value is the coordinates' dot with its direction. Setting it moves the
coordinates along the direction by the difference. The named controls aren't square to one
another, so setting one moves the others' values.

**The race menu's sliders are those controls.** The race menu (`menus\chargen\race_sex_menu.xml`)
has these pages:

- Main: Race, Face, Hair and Eyes buttons, the Gender toggle, the Age and Complexion sliders.
- Face, then Shape and Tone.
  - Shape's pages hold one slider per symmetric geometry control, each control on exactly one page:

    | Page | Controls |
    |---|---|
    | General | 19..23 (Face ratios, heavy/light, round/gaunt, thin/wide) |
    | Brow | 0..2 |
    | Cheeks | 3..7 |
    | Chin | 8..14 |
    | Eyes | 15..18 |
    | Forehead | 24..26 |
    | Jaw | 27..30 |
    | Mouth | 31..40 |
    | Nose | 41..54 |

    "Temples - thin / wide" (55) has no slider.
  - Tone's pages hold the texture controls.
- The menu has no sliders for the asymmetric controls. The game still saves and draws them (the
  Construction Set sets them on its NPCs).

A geometry slider runs over ± the race's clamp: its face clamp (3 when the race sets none) while
the General page is on show, else its main clamp (5 when unset). A texture slider runs over ±10.
The Age slider is FaceGen's age control, 15 to 65. The Complexion slider is the gender control
less the race's default (±2), −2 to 2.

**How a slider changes the face.** Moving one sets its control on the whole face (race plus
character), and the change is taken off the race to give the character's new coordinates. For a
geometry slider, the player's two FaceGen heads (third and first person) are morphed by the
change, and the hair's length morph goes back on after. Then the page's other sliders are read
from the face again: they move, because the controls aren't square. A texture slider also
refreshes the age, the complexion and the face texture.

A page's sliders aren't read again when it comes up. They show the face as it was the last time one
of its own sliders moved.

**There's no undo or reset in the race menu.**

## What other tools and games do

For inspiration only.

- **Gravity Sketch.**
  - A translucent "grab sphere" on the dominant hand shows what a grab would take. An object it
    touches highlights before it's grabbed.
  - The sphere's size sets how much is taken.
  - Undo is a button on the drawing hand when it holds nothing. An object keeps its own editing
    history.
- **Adobe Substance 3D Modeler (Medium's successor).**
  - Undo and redo are fixed buttons on the controllers.
  - Symmetry mirrors each stroke across a plane, but only while it's on: strokes made with it off
    aren't mirrored later.
- **MetaHuman Creator.**
  - Its head sculpt tool puts markers on the face that you drag.
  - The drag moves the face within its statistical model: what's dragged is a point, what changes is
    the whole face's position in the model. That's this feature's approach exactly.
- **Character Creator (Reallusion), inZOI.**
  - You drag facial features directly in the 3D view, with sliders beside them for fine tuning.
- **Players** are split on drag-to-edit.
  - Some like it.
  - Others find sliders more exact. A Mordhau player found click-and-drag "clicks the wrong part of
    the face".

  So what will be taken must be shown before it's taken.

Sources: help.gravitysketch.com (Controllers Layout, Edit Objects), helpx.adobe.com/substance-3d-
modeler (VR shortcuts and controls, interface overview), dev.epicgames.com/documentation/metahuman
(head sculpt tool), manual.reallusion.com (Adjusting Facial Features), steamcommunity.com.

## The design

**Where the hands take hold: both, whichever is nearer.**

- **A copy of the head standing within reach**, like a sculptor's bust:
  - 42 cm ahead of the head, 24 cm to its right, 17 cm below the eyes;
  - 1.7 times life size;
  - turned to face the player;
  - placed when the race menu opens.

  It's the main one, because:
  - A headset sits where a real face's nose, eyes and cheekbones are, so on your own head the
    controllers can't reach them.
  - At 1.7 times its features are big enough to take one at a time.
  - It stands to the right (the race menu is at the mirror's left), so the hands on it don't hide
    the mirror. The mirror shows the face changing on your own body as you work.
- **Your own head**, where it is: reach up and pull your chin down or your ears out, and watch it
  happen in the mirror.
  - It's the funny one, and the chin, jaw, mouth and ears are within reach.
  - It follows the head as it moves, so the pull is the hand's move against the head.

Both work at once, so there's no setting to pick between them.

**Taking hold.**

- A palm (the grip pose, the middle of the palm) within 4.5 cm of the face (of the copy's larger
  face) shows what its grip would take: the vertices round the nearest point glow in the game's
  cursor gold (255, 203, 57, the menus' laser's colour), brightest at the middle and nearer the hand.
- The grip takes it.
- Nothing snaps:
  - the hand stays where it is;
  - its fingers close on the point taken (a 1.2 cm sphere there: the rig's grasp);
  - the face doesn't move until the hand does.

**Pulling: the face's own coordinates, solved.**

- The hand's move against the head it holds, in the head's model space, is the pull `d`.
- Each vertex `v` should move `w_v · d`, where `w_v` is a Gaussian of 2.2 cm round the point taken
  (the same weights that glow).
- **Why the rest of the face moves at all.** FaceGen's coordinates are a statistical model's modes:
  each moves the whole face a little. A change to one part alone is a mix of many of them whose
  moves cancel everywhere else, and they only cancel so far. So a pull always moves the rest of the
  face somewhat; the solve decides how much against how far the part held follows the hand.
- The change in the coordinates `Δ` minimises
  `Σ_v h_v |M_v Δ − w_v d|² + Σ_v s_v |M_v Δ|² + λ|Δ|²`:
  - over every vertex of the parts a hand takes (the head, ears, mouth, teeth, tongue, eyes; not
    the hair);
  - `M_v` is the morphs' moves of `v`;
  - `h_v = w_v / Σ w²`: the first term is the hold's mean miss;
  - `s_v` is how still `v` is held: 0 up to 4 cm from the point taken, rising (smoothstep) to all of
    it at 8 cm, over the face's total, times 10. So the second term is ten times the rest's mean
    move. A nose pulled long takes its bridge and nostrils with it, not the ears.
  - A symmetric pull frees the point's mirror image as much as the point: the symmetric morphs move
    both sides alike, so holding the other cheek still would only fight them. An asymmetric pull
    holds the other side still like the rest (that's what it's for).
  - `λ` is 1% of the matrix's mean diagonal: big mixes that barely cancel cost something.

  It's linear in `d`. So the 3-column gain `H = A⁻¹G` is solved once as the hand takes hold (1.3 to
  2.8 ms on the frame for both the symmetric and the asymmetric gains), and each frame is a multiply.
  The face's Gram matrix over all vertices is summed once per face, off the frame; a hold only adds
  the vertices near it. The age and gender are taken off the gain in the solve's own metric (so taking
  them off doesn't move the rest of the face either), and the Age and Complexion sliders don't move.
- **The first version** weighed the part held 250 times more than the rest staying put
  (`ρ_v = 1 + 250 w_v` over a target of `w_v d` everywhere), and took the age off the gain along the
  age's own direction. The first playtest found that a pull moved the whole head. On the real Nord
  head, a 1 cm pull at seven places (nose tip forward, chin down, cheekbone out, brow forward, jaw
  corner out, upper lip forward, ear out), measuring the face more than 6 cm from the point and from
  its mirror image:

  | | follows the hand | far face, rms | far face, most |
  |---|---|---|---|
  | first version | 0.69 cm | 0.22 cm | 0.88 cm |
  | now | 0.64 cm | 0.07 cm | 0.34 cm |

  The nose tip follows all of it either way. The chin follows a tenth (it was a quarter, by moving
  the whole jaw and forehead with it): FaceGen has no chin of its own, so the chin's sliders move
  the jaw and mouth too. Where the face can't follow, the hand is held back and feels it.
- **Keeping the face in range.**
  - The named controls stay within their sliders' ranges (or as far out as the face already was:
    nothing snaps back as a pull starts). The age and gender stay as they were.
  - Every coordinate stays within ±8 of the mean face (or as far as it already was). FaceGen's
    coordinates are in standard deviations of real faces. The sliders' ranges alone barely hold some
    mixes of them: the 56 controls' smallest singular value is 0.013, against a largest of 2.5. In the
    first playtest a few minutes of pulls took one coordinate to −28.6, a face no slider makes, where
    every part warps.
  - Whatever goes past is taken back in the solve's own metric, a step per limit, a few rounds, at
    most 0.16 ms a frame. So the rest of the face stays put and the part held stays as near the hand
    as it can. Forty random 1.5 cm strokes end with every coordinate within 8 and every control
    within 5. The first version reached 28.
- **The copy stands still.** It's placed by the middle of the head with no face on it (the parts'
  unmorphed vertices). The first version used the middle of the face as composed, so each time the
  body was put together with a new face, the copy jumped by half the face's change. A nose pulled
  2 cm moved the whole copy 1.7 cm.
- **The hand drawn.** It's drawn holding the point where it got to. When the face can't follow, the
  hand stays on it short of the controller (with ghost hands on, a ghost at the controller), as the
  physics hands are against walls. The pull is felt as a fine texture while it moves, and harder
  the further the face holds the hand back.
- **Both hands** can pull at once: the face is the stroke's start plus each holding hand's change.
- **The trigger** of a holding hand blends in the asymmetric morphs, eased over 0.12 s, so the face
  never jumps. Symmetric is the default: both sides alike, as the user asked.

**What the game does with it.**

- The runtime sends the whole face (symmetric and asymmetric geometry) whenever the hands change it
  (`FaceSculpt`, a new region), with a stroke number for each pull.
- The adapter does what the race menu does for a slider:
  1. the character's coordinates become the face less the race's, in place;
  2. the game's own two heads are morphed by the change (the game's own morph per part, the hair's
     length morph set again around it), with the change lent to it as a coordinate set of our own
     (the morph only reads it);
  3. the page on show has its sliders read again. Other pages are marked and read as they
     come up, because the General page's range is the face clamp only while it's the page on show.
- So the sliders move as the face is pulled. A slider moved afterwards moves from where the hands
  left it.
- **Drawn live.** The face is drawn every frame from the runtime's own solve:
  - On the body (and so in the mirror), each FaceGen part keeps its unmorphed vertices and moves, and
    its vertex buffer is rewritten when the face changes.
  - The copy is the face's parts on their own, drawn the same way.
  - The body is put together again (its eyes' turning points and joints exact) only once the face
    has been still for half a second.
  - A face the game changes (a slider, an undo) is drawn live too, as it comes back.

**Undo and reset: in the race menu itself.** The race menu has no control for either, so they're
buttons made from its own button template, in its own lists, worked by the laser like its other
buttons:

- **Main page:** "Undo", "Reset Face". Reset Face is the race's own face, at the age and complexion
  it has now.
- **Each Shape page:** "Undo", "Reset". Reset sets that page's controls to the race's values, with
  the least change to the rest and the age and complexion kept. These are the "features": Brow,
  Cheeks, Chin, Eyes, Forehead, Jaw, Mouth, Nose, and General.

Their clicks are taken before the menu's own click handling sees them. Undo goes back to the face
before the last pull or reset, as many times as there were. The menu closing forgets them.

**What stays on the game's sliders: hair and eye colour, complexion, age and the texture sliders.**

- They're colours and a whole-face age, with no place on the face to take hold of. A pinch can't
  say "redder" or "older".
- The pulls keep the age and complexion (the gender) as they are, so their sliders never move under
  the hands.
- The hair is excluded from the holds: its morphs follow the skull, but a hand on long hair is
  holding the hair, not the face.

**Sounds and feel.**

- Taking hold and letting go play the game's own organic collision sound (`COrganicSmall`, food
  set down: soft and wet, the game's nearest to clay). These are new pack events `face_take` and
  `face_let_go`.
- A tick in the hand as it takes hold and lets go.
- The texture and strain while pulling.

**Hands, laser and gestures.**

- A hand near the face or holding it:
  - has no laser (the other hand points);
  - its trigger is the pull's (asymmetry), not a click on the game's menu;
  - it doesn't count as empty for the mirror's curtain gesture.
- In the race menu the rig is paused (it takes nothing). The holds are the sculpt's alone.
- A free hand pushed into the copy is drawn on its surface.

## Booting into the character creator (playtests)

The playtest starts a new game, so the race menu is the one a player starting out meets, in the
prison cell, rather than `showracemenu` on a saved character.

- **The adapter's `NewGame` test line** answers the main menu's New ("Start new game?") with Yes,
  as the player would, once the main menu is one a player could click: shown, done fading in, and
  with no box of its own up. It reports whether the main menu went on to close.
  `tools/oblivion.ps1 -NewGame` and `tools/play.ps1 <feature> -NewGame` write it in place of the
  save to load.

  The first version answered as soon as the main menu was there, on its first frame. The game
  doesn't close a menu that isn't shown yet, so the quest started and the new game was flagged with
  the main menu still up (the second playtest). A save then loaded from that menu crashed in its
  loading screen, most likely because of the half-started new game.
- **The film before the cell** is `sOblivionIntro:General` (`OblivionIntro.bik`). Test boots empty it
  as they already empty the logo before the main menu (`sMainMenuMovieIntro`). The player's own
  value comes back as the game closes.
