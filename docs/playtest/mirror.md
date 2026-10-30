# Playtest: the mirror

Save: rustvr-base

`powershell -File tools\play.ps1 mirror` boots Oblivion into `rustvr-base`. That's Danwood, a Nord in
iron armour, in his house in Danwood. `powershell -File tools\play.ps1 -Close` puts the game away.
`play.ps1 mirror -Commands "showracemenu"` boots it with the race menu open (the real keyboard doesn't
reach the game in VR, so its console can't be typed into). `-Commands "SelectSpell Mirror"` boots it
with the Mirror power selected (the test console's `SelectSpell` takes a spell's name as well as its
id, since the game gives the spell its id).

The heads, hair, helmets and their FaceGen morphs are already extracted into the shared
`local_assets`. On another machine, run `cargo run --release -p oblivion-assets --bin
oblivion-extract -- export meshes/characters meshes/armor meshes/clothes textures/characters`.

The research and the design are in `docs/research/mirror.md`.

## What to try, in order

1. **Cast Mirror.**
   - Every character has a lesser power called "Mirror" (a Conjuration spell, free, castable any
     time). It's in the game's magic menu with your powers. The playtest .bats boot with it already
     selected, since the magic menu can't select spells in VR yet.
   - Cast it with the spell hand as any spell on yourself: hold the trigger with that hand empty,
     then let go.
   - The game's summoning effect, the swirl a summoned Daedra arrives in, rises 1.2 m in front of
     you. The mirror appears in it at its flash, as a summoned creature would, with Conjuration's
     hit sound at the glass.
   - Cast it again while looking at the mirror: the same effect, and it's gone.
   - Cast it while looking somewhere else, or when it's far behind you: it comes to stand in front
     of you instead.
   - Save, load the save again: you still have the spell (once, not twice). An older save of your
     own gets it the first second it's loaded.
2. **Call up a mirror without the spell.**
   - Hold both grips with empty hands, with your hands together in front of your chest (about
     30 cm in front, below your chin). Then pull them apart wide, quickly, like opening curtains.
   - A standing mirror appears 1.2 m in front of you, on the floor, facing you: 0.95 m wide and
     2 m tall.
   - You see yourself in it:
     - your race's head and face;
     - your hair, or your helmet if you wear one;
     - your armour;
     - the room behind you, as the game draws it, with its lights and fog.
   - Your reflection moves exactly with you: your head, both hands, your arms, crouching.
   - The same gesture puts it away.
   - Or use the RustVR menu: press the menu button so the game pauses and the RustVR menu shows,
     open the Body tab, and press "Stand a mirror in front of me" (the same button puts it away).
3. **Look at yourself.**
   - Step up to the glass and look at your face: the race's head, morphed to the save's own face,
     the age map's wrinkles, the eye colour, the hair colour.
   - Your reflection's eyes look back into yours (as far as eyes turn: about 30° from where the head
     faces).
   - Turn your head and see your profile.
   - Lean in close, then step back to see the whole body.
   - Walk round behind it: it's a single-sided mirror, and its back isn't drawn.
4. **The world in it.**
   - Behind you in the mirror is the game's own picture, drawn from where your eyes are reflected,
     each eye with its own parallax, so the reflection has depth.
   - Walk away and turn: the mirror stays where it stands in the world.
   - Put something between yourself and the mirror (a table): it hides you in the reflection.
   - A mirror called up in a doorway or against a wall is moved nearer (never nearer than 0.6 m).
   - The game's own walls hide the glass where they're nearer.
   - Walk more than 25 m away, or go through a door, and it's put away.
5. **The race menu.** Boot with `-Commands "showracemenu"` (or start a new game).
   - The world stays round you and a mirror stands in front of you. The game's own race menu
     stands at the mirror's left, turned to face you.
   - Point at the menu with the laser and pull the trigger to work it, as before.
   - Every change shows on your reflection as you make it:
     - drag the Age slider and the face ages, its age map changing at each decade;
     - Complexion changes the skin's colour;
     - open Race and use its arrows: race and gender change the whole body, head, ears and hair;
     - Hair and Eyes cycle the hair and the eye colour;
     - Face opens the face sliders.
   - Your helmet comes off while the menu is open (the game takes it off) and goes back on after.
   - Press Done and the mirror goes, unless you had called it up yourself.
6. **Settings** (the RustVR menu, Body tab, under Mirror):
   - Distance (0.6 to 3 m);
   - Width and Height;
   - the gesture on or off;
   - "The game's world in the mirror". Off, you see yourself alone on dark glass, and the game
     skips its extra render. (It's skipped anyway while you look away from the mirror.)

## What was checked in the game (emulated headset, captures)

- **Race menu, `rustvr-base`** (male Nord, iron armour):
  - the mirror stands at 1.2 m, with the room behind the player in it, rendered by the game;
  - the body in it is iron armour, with the Nord head and face, its hair, and the shield on the
    back;
  - the reflection follows head turns and a raised hand;
  - the menu stands beside the mirror, cropped to the menu itself;
  - the laser works it: the Age slider was dragged both ways;
  - the face changes live as the slider moves: its coordinates every frame, the age map at each
    decade, the body put together again in 2 to 13 ms and on the GPU in 2 to 4 ms;
  - the Race page under the laser: Nord to Khajiit, then male to female. The mirror showed each as
    it was made (the Khajiit head, ears and braids, the female armour), and the game didn't crash
    (it did before: see Decisions);
  - the face up close: the Nord head under the iron helmet, lit by a candle; the hair (Loose)
    without the helmet.
- **Called by the gesture** (`rustvr-weye`, made female with `player.sexchange`):
  - the mirror appears and goes again;
  - the female Nord shows with the iron helmet on (hair hidden under it) and her face lit by the
    room's candles;
  - looked at close up and from the side.
- **Frame rate:**
  - 140 to 300 game frames a second with the mirror up (a third world render a frame), in the test
    saves' interiors;
  - about 370 while turned away from it (its render skipped);
  - after turning back, its world is back the next frame.
- **In first person** the head, hair and helmet stay out of your view.

## What wasn't checked

- **The mirror spell, at all.** It was built after the emulated runs and has never been in the game:
  - the spell made and given to the player (and once only, after saving and loading);
  - how the magic menu shows it (its icon, and whether the game says it was added);
  - the cast, the summoning effect at the mirror's foot, the glass appearing at its flash;
  - the hit sound at the glass. The game may also play Conjuration's hit sound on you as the spell
    lands, so it could sound twice.

- **On a real headset.** Everything above is the emulated headset's captures. Watch for:
  - the reflection's depth and parallax;
  - whether the eyes' gaze looks right (at capture resolution the eyes are a few pixels);
  - comfort at 1.2 m.
- **Outdoors.** The test saves and `coc` all landed in interiors, so the mirror wasn't seen in
  daylight, at night, in fog or rain, nor its cost in a big outdoor scene (bigger scenes make the
  extra render dearer).
- **The other races** in the mirror: Argonians, the elves' ears, the Orcs. Also the hoods and
  helmets that cover only part of the head. Khajiit and Nord were checked.
- **The race menu's Hair, Eyes and Face sub-pages**, one by one. They go through the same paths
  as race, sex and age, which were checked.
- **The gesture** made by real hands.

## Decisions

- **The mirror spell:**
  - Oblivion has no mirror of its own to use: no model, no record, as decoration or an item. So it's
    conjured, the way the game brings things in by magic.
  - A lesser power, so anyone can cast it as often as they like, for nothing. It's a script effect
    with no script, in Conjuration, looking like Bound Shield (a conjured thing): its light in the
    hand, its sounds and icon.
  - It's made as the spellmaking altar makes a player's spells (a created spell, which the save
    keeps) and given with the game's own `AddSpell`, so every game has it: a new one, or a save from
    before.
  - It arrives as a summoned Daedra does: the game's own summoning effect (`summonDaedra.nif`), the
    glass appearing at the effect's "Hit" moment.
  - Cast while looking at the mirror (within about 35° and 6 m) it sends it away; anywhere else it
    summons one in front. So a mirror left behind comes to you with one cast.
  - The gesture and the menu button still call it up and put it away at once, with no effect.

- **One game render for both eyes.** The mirror's world is rendered once a frame from the head
  reflected in the glass, and moved to each eye by its depth (the runtime's warp). Two renders
  would be exact but cost a fourth world render a frame. The player in the reflection is drawn
  for each eye exactly.
- **Where it stands:**
  - 1.2 m: your reflection is 2.4 m away, the whole body in view and the face readable. Research
    found 1 to 4 m makes no difference to how people see themselves.
  - 0.95 x 2 m, foot on the floor: the whole body, with room for the arms.
  - It stays in the world (it doesn't follow you).
  - Defaults are in the RustVR menu.
- **The gesture:** both hands gripped together in front of the chest, pulled apart within 0.8 s.
  It doesn't clash with the hip and shoulder holsters, the chest and belt slots, or the gloves'
  flick back.
- **Eyes:** the reflection's eyes look back at your reflected eyes, up to about 30° off the head's
  forward. The Quest 3 has no eye tracking, so this is a guess. It gives the eye contact that
  makes a mirror feel like you.
- **Head-slot rules:**
  - what covers the hair slot (helmets, hoods) hides the hair;
  - the head itself is always drawn under it.
- **The race menu** keeps you where you stood: the game moves its camera in front of your face,
  and following it would jump you 0.6 m. The adapter no longer forces first person while it's open
  (forcing it crashed the game when the race changed).
- **The glass** has a thin dark frame, so it reads as a mirror and not a hole. Oblivion has no
  mirror model to use.

## Not done

- **The hair's height-map parallax** (its `_hh` map). The hair is otherwise the game's own:
  - its texture, layer map and colour;
  - its rim;
  - its anisotropic highlight from the game's table (`textures\effects\aniso.dds`).
- **The hair is alpha-tested, not blended,** so its edges are a little harder than in the game.
- **The hair's length** (the race menu's hair length morph) isn't applied.
- **Expressions and lip-sync** (the `.tri` files): the face is always neutral.
- **A second game render for exact stereo** in the mirror's world (see Decisions).
- **The desktop view** shows the mirror with the left eye's reflection.
