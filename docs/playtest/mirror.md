# Playtest: the mirror

Save: rustvr-base

## Playing it

From the repository root:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-base -Commands "SelectSpell Mirror"
```

It builds main, installs it beside the game and boots into the save with the Mirror power selected
(the test console's `SelectSpell` takes a spell's name as well as its id: the game gives this one
its id). `-Commands "showracemenu"` boots it with the race menu open instead, and `-NewGame` starts
a new game (the prison cell and the race menu). `powershell -File tools\oblivion.ps1 -Close` puts
the game away and the player's settings back. The fast boot, the test saves and the game lock are
in [DEVELOPING.md](../DEVELOPING.md).

`rustvr-base` is the developer's own test save, not in the repository: Danwood, a Nord in iron
armour, in his house in Danwood. Any save does; `-At <cell>`, saved as `rustvr-base`
([DEVELOPING.md](../DEVELOPING.md), "Test saves"), makes one. A woman to look at: `-Commands "player.sexchange"`.

The heads, hair, helmets and their FaceGen morphs come from the player's own install:

```bat
cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/characters meshes/armor meshes/clothes textures/characters
```

The research and the design are in `docs/research/mirror.md`.

## What it does

A standing mirror in the game's world (`framework/crates/runtime/src/mirror.rs`), in which you see
your whole body as the game would draw it, with the game's world behind you.

- **Calling it up.** Three ways, each also putting it away:
  - **the Mirror spell** (`crate::mirror_spell`): a lesser power every character has, from a new
    game or the first second an older save is loaded (once: a save keeps it). Cast it with the
    empty off hand, as any spell on yourself: pull the trigger, and let go once it's charged. The
    game's summoning effect, the swirl a summoned Daedra arrives in, rises 1.2 m in front of you,
    and the mirror appears at its flash with Conjuration's hit sound at the glass. Cast it while
    looking at the mirror (within about 35 degrees and 6 m) and it goes the same way; anywhere
    else, it comes to stand in front of you;
  - **the gesture:** hold both grips with empty hands together in front of your chest (within
    22 cm of each other), then pull them apart wide (75 cm) within 0.8 s, like opening curtains;
  - **the menu:** the Body tab's "Stand a mirror in front of me" (the same button puts it away),
    taking effect when you resume.

  The gesture and the menu are at once, with no effect.
- **Where it stands.** 1.2 m in front of where your head faced, on the floor, facing you, 0.95 m
  wide and 2 m tall. Nearer when a wall is closer (never nearer than 0.6 m). It stays where it
  stands in the world. More than 25 m away (walked off, or through a door) it's put away.
- **What you see.** Your race's head and face, morphed to the save's own face, with the age map's
  wrinkles, the eye colour and the hair colour; your hair, or your helmet if you wear one (what
  covers the hair slot hides the hair; the head is always drawn under it); your armour; the room
  behind you as the game draws it, with its lights and fog. The reflection moves with you: head,
  both hands, arms, crouching. Its eyes look back into yours, as far as eyes turn (about 30 degrees
  from where the head faces). It's single-sided: walk round behind it and its back isn't drawn.
- **The world in it** is the game's own picture, rendered once a frame from your head reflected in
  the glass, and moved to each eye by its depth (`crate::warp`), so each eye has its own parallax.
  Things between you and the glass hide you in it; the game's walls hide the glass where they're
  nearer. The extra render is skipped while the glass is out of view.
- **The race menu.** The world stays round you and a mirror stands in front of you, the game's race
  menu at the mirror's left edge, turned to face you; the laser works it. Every change shows on your
  reflection as you make it (age, complexion, race and sex, hair, eyes, the face sliders). The
  game takes your helmet off while it's open and puts it back after. Done puts the mirror away,
  unless you'd called it up yourself. The face can also be shaped by hand there
  ([facesculpt.md](facesculpt.md)).
- **In first person** the head, hair and helmet stay out of your own view.

## What to try

1. **Cast Mirror.** With the off hand empty, charge it and let go. Watch for the summoning effect,
   the glass at its flash, the hit sound. Cast again looking at it: it goes. Cast looking elsewhere
   or far from it: it comes to stand in front of you. Save, load: you still have the spell, once.
   The magic menu (B on the off hand, then the magic page) lists it with your powers.
2. **The gesture.** Hands together in front of your chest, about 30 cm out below your chin, both
   grips held; pull them apart quickly. Again to put it away.
3. **Look at yourself.** Step up to the glass: the face, the eyes looking back, your profile as you
   turn your head. Lean in, step back for the whole body. Walk round behind it.
4. **The world in it.** Walk away and turn: it stays. Put a table between you and it. Call one up in
   a doorway or against a wall: it stands nearer. Walk more than 25 m away, or through a door.
5. **The race menu.** Boot with `-Commands "showracemenu"` or `-NewGame`. Drag Age both ways, change
   Complexion, the race and sex with the Race page's arrows, the Hair, the Eyes, the Face sliders:
   each shows on your reflection. Your helmet comes off while it's open. Done.
6. **Outdoors**, by day and at night, in fog or rain, and the frame rate there.
7. **The settings** below, the game's world off in particular.

## Settings

The Body tab, under Mirror (`[mirror]` in `settings.toml`):

| Setting | Default (range) |
|---|---|
| Stand a mirror in front of me / Put the mirror away | (a button) |
| Pull your gripped hands apart in front of your chest to call it up or put it away (`mirror.gesture`) | on |
| The game's world in the mirror (`mirror.game_world`; off: you alone on dark glass, and the game skips its extra render) | on |
| Distance (`mirror.distance`) | 1.2 m (0.6 to 3) |
| Width (`mirror.width`) | 0.95 m (0.4 to 2) |
| Height (`mirror.height`) | 2 m (1 to 2.6) |
| `mirror.lift`: its foot above the floor (`settings.toml` only) | 0.02 m |

## Decisions

- **The mirror is conjured.** Oblivion has no mirror of its own: no model, no record, as decoration
  or an item. So it comes the way the game brings things in by magic.
- **A lesser power**, so anyone can cast it as often as they like, for nothing. A script effect with
  no script, in Conjuration, with Bound Shield's look (a conjured thing): its light in the hand, its
  sounds and its icon. It's made as the spellmaking altar makes a player's spells (a created spell,
  which the save keeps) and given with the game's own `AddSpell`, so every game has it.
- **It arrives as a summoned Daedra does:** Summon Dremora's effect (`summonDaedra.nif`, every
  Daedra's), the glass appearing at its "Hit" key.
- **One cast both ways:** looking at it sends it away; anywhere else summons it in front, so a mirror
  left behind comes to you with one cast.
- **One game render for both eyes.** Two would be exact but cost a fourth world render a frame. The
  player in the reflection is drawn for each eye exactly.
- **1.2 m away:** your reflection is 2.4 m off, the whole body in view and the face readable.
  Research found 1 to 4 m makes no difference to how people see themselves. 0.95 x 2 m fits the
  whole body with room for the arms.
- **The gesture** doesn't clash with the hip and shoulder holsters, the chest and belt slots, or the
  gloves' flick back.
- **The eyes meet yours.** The Quest 3 has no eye tracking, so it's a guess; it gives the eye
  contact that makes a mirror feel like you.
- **The race menu keeps you where you stood:** the game moves its camera in front of your face, and
  following it would jump you 0.6 m. First person isn't forced while it's open (forcing it crashed
  the game when the race changed).
- **The glass has a thin dark frame**, so it reads as a mirror and not a hole.

## Not done / known issues

- **The hair's height-map parallax** (its `_hh` map) is left out. Otherwise the hair is the game's
  own: its texture, layer map and colour, its rim, its anisotropic highlight from the game's table
  (`textures\effects\aniso.dds`).
- **The hair is alpha-tested, not blended**, so its edges are a little harder than in the game.
- **The hair's length** (the race menu's hair length morph) isn't applied to the drawn hair.
- **No expressions or lip-sync** (the `.tri` files): the face is always neutral.
- **No second game render for exact stereo** in the mirror's world.
- **The desktop view** shows the mirror with the left eye's reflection.
- **Not seen in a playtest record:** the Mirror spell at all (made and given, once after saving and
  loading; how the magic menu shows it; the cast, the effect, the glass at its flash; the hit sound,
  which the game may also play on you as the spell lands); a real headset (depth and parallax, the
  eyes' gaze, comfort at 1.2 m); outdoors; Argonians, the elves' ears, Orcs, hoods and partial
  helmets; the race menu's Hair, Eyes and Face pages one by one; the gesture by real hands.

## History

- **The spell** came last: the Mirror lesser power summoning it with the game's own effect, built
  after the emulated runs.
- **The first build**, checked on the emulated headset only. In the race menu (`rustvr-base`): the
  mirror at 1.2 m with the room in it, iron armour, the Nord head and hair, the shield on the back;
  the reflection following head turns and a raised hand; the laser dragging Age, the face changing
  live (the body put together again in 2 to 13 ms, on the GPU in 2 to 4 ms); Nord to Khajiit, male
  to female, without the crash forcing first person had caused. By the gesture (`rustvr-weye`,
  made female): a female Nord with the iron helmet, lit by candles. 140 to 300 game frames a second
  with the mirror up in the test saves' interiors, about 370 turned away from it.
