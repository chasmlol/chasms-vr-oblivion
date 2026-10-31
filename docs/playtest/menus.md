# Playtest: floating menus and HUD

Save: rustvr-menus

## Playing it

From the repository root:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-menus
```

It builds main, installs it beside the game and boots straight into the save for the headset.
`powershell -File tools\oblivion.ps1 -Close` puts the game away and the player's settings back.
The fast boot, the test saves and the game lock are in [DEVELOPING.md](../DEVELOPING.md).

`rustvr-menus` is the developer's own test save, not in the repository: Jensine's "Good as New"
Merchandise in the Imperial City's Market District at noon, a test chest full of things on the
floor in front of you and Jensine at her counter to your right. To make one from any save, with
the game's test container of every potion a metre ahead:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-city -At ICMarketDistrictJensinesGoodasNewMerchandise -Hour 12 -Commands "PlaceAt 00187B46 1 0 0"
```

Then save it as `rustvr-menus` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

## What it does

Every menu floats in the world as pieces of Oblivion's own UI: its parchment, fonts, icons and
buttons, cut out of what the game draws (`crate::hud`). The world is still drawn round and behind
it, your hands too, and the game still runs its own selection, scrolling and clicks. Point with the
laser in your weapon hand and pull the trigger to click; the stick scrolls; B closes.

- **The world stays live under menus.** On opening a menu Oblivion renders a frozen picture of the
  world to draw behind it. In VR your head keeps moving, so that picture is skipped and the world is
  rendered for both eyes every frame (`crate::render`). The game is still paused.
- **Real transparency, solid parchment.** The game draws its UI into an alpha render target of ours
  instead of its back buffer, alpha always written and blended as coverage
  (`crate::d3d9::ui_pass`), so each piece is exactly as opaque as the game's art and the rest is
  clear. A menu's pieces are pages (`HUDE_SOLID`): the runtime draws them opaque wherever the game
  drew half or more, so no blending of the game's leaves the parchment see-through. Torn edges and
  letters' edges stay soft.
- **Under the big four**, the game's own 3D scene (`Menu3DRoot`, a figure of the player) is culled
  while the UI is drawn, so no silhouette shows through the pages (`crate::menus`). Elsewhere (the
  race menu's face) it's drawn as the game draws it.
- **Size from the game's text.** Every piece is sized so a line of the game's text is 1.5 degrees
  tall wherever it floats. The research figure is 1.2 to 1.4; Oblivion's serif font reads harder.
- **Where things go.** Once placed, a menu stays put in the world; its pieces lie on an upright
  cylinder round the eye, each turned to face it.

  | Menu | Where |
  |---|---|
  | a container's or a merchant's | at the thing's distance (0.9 to 1.4 m), 10 degrees under the eyes; to its side only where it would cover it (never more than 40 degrees round) |
  | your own (inventory, magic, map, character) | 1.1 m ahead, 8 degrees under the eyes; it glides round after a turn of 65 degrees |
  | a count, a question, haggling | 0.8 m ahead, where you look, nearer than the menu under it |
  | dialogue | the speaker's name over their head, their line under their face, the topics beside their head |
  | a service from dialogue (spells, training, repair, recharge, persuasion) | beside the speaker |
  | a thing's own (a lock being picked, an altar's enchanting or spellmaking, a sigil stone) | beside the thing |
  | the race menu | beside the mirror ([mirror.md](mirror.md)) |
  | a book or note | over the book in the hand ([books.md](books.md)) |
  | the main menu | in its place in Cyrodiil ([worldmenu.md](worldmenu.md)) |
  | anything else | ahead, as one piece cut from the screen |

- **Pop-ups are moved apart on the screen.** In the flat game a count or a question sits over the
  middle of its menu, so cut out it would show twice. While it's up, the menu under it is moved
  down the game's screen and the pop-up into the room above, through the game's own tile update.
- **B closes menus with the game's Menu Mode (Tab):** in Oblivion, Escape opens the pause menu over
  an open menu instead of closing it. The pause menu and its own pages (options, load, save) close
  with Escape. B on the off hand opens Menu Mode (your own menus); held long, the Quick Menu.
- **Only the pause menu gets RustVR's settings beside it**, on its right.
- **The laser** is in the colours of the game's own cursor (`textures\menus\misc\cursor.dds`): a
  cream thread (255, 247, 222) in a soft gold glow (255, 203, 57), motes of light running out along
  it, fainter and fading along its length when it points at nothing; where it hits, the gold light
  it casts and a dot with a dark edge, gold, a cream heart, pressed in while the trigger is held.
  The adapter gives the colours (`HudLayout::pointer`), so another game keeps the runtime's blue.
  A light tick in the hand each time the game's highlight moves to another row, tab or button.
- **The HUD**, cut from the game's own HUD menus:
  - the health, magicka and fatigue bars with the weapon and spell icons, the compass above them
    and the active effects above that float over the inside of the off-hand wrist (the pieces at
    1.3 times their base size, the compass at 1.8, so its heading letters read on a raised wrist),
    shown only while its palm is turned up and you look at it: fully
    within about 37 degrees of straight up, gone by about 57, fading between. A left-handed player
    gets them on the right wrist;
  - the card of what you look at (a person's name, "Open Door", a container) floats just under the
    point you look at, just in front of what it's about (between 0.6 and 3 m), easing after your
    head over 0.18 s, the same size to the eye at any distance; a person's sits lower, under the
    chin;
  - a thing that can be taken shows nothing while you only look at it (no card, no hand icon, no
    fade as your gaze leaves it). Once a hand has it, its card floats over it in the hand;
  - the flat game's crosshair is hidden (`crate::reticle`); its activate icon stays, red when taking
    would be stealing;
  - a foe's health over the foe; spoken lines on the caption strip.

## What to try

1. **The HUD.** Turn your off-hand palm up and look at it: the bars, icons, compass and effects
   over the inside of the wrist. Turn it back and they fade. Look at Jensine: her card under where
   you look, just in front of her, easing after your head. Look at a thing on a shelf: nothing. Pick
   it up: its card (name, value, weight) over it in your hand, the red hand and "Steal" if it's
   hers.
2. **The chest.** Look at it and pull the trigger with an empty hand. Its list floats above it in
   three strips: the list (column heads, scroll bar, your gold and weight), the category tabs, the
   buttons (Take All, the switch to your own things, Exit). The parchment is solid, cut out round its
   edges. Point at rows (a tick as the highlight moves), pull to take, push the stick to scroll, drag
   the scroll bar, click a tab, Take All or the switch. B closes it.
3. **Counts and questions.** Switch to your own things and click the arrows: "How Many?" pops up
   where you look, nearer than the list, which stays behind it with no copy of the box in it.
4. **Your own menus.** B on the off hand. The page floats ahead with its tabs under it, and under
   them the HUD's switcher: the bars open the character page, the weapon the inventory, the spell
   magic, the compass the map. Point at something on the character page: its description pops up
   over it. Turn more than about 65 degrees: it glides round. Look for anything showing through the
   pages from behind.
5. **Dialogue.** Look at Jensine and pull the trigger: her name over her head, her line under her
   face, her topics and the service icons beside her head.
6. **Bartering.** Click the barter icon: her wares beside her in the same three strips, with her
   name, gold and Haggle. Haggle pops up over them. B or Exit goes back.
7. **Pause.** The menu button: the pause menu ahead, RustVR's settings on its right. Watch the
   laser.
8. **A thing's own menu**: a lock beside the locked door or chest, an altar's beside the altar, a
   sigil stone's beside it.
9. **Everything else** (sleep and wait, the level-up, repair, alchemy) floats ahead as one piece.

## Settings

| Setting | Where | Default (range) |
|---|---|---|
| `ui.menu_text_deg`: how tall a line of the game's text looks | `settings.toml` | 1.5 degrees |
| `ui.menu_opacity`: how opaque the pieces are as a whole | `settings.toml` | 1 |
| `ui.menu_scroll`: wheel notches a second at full stick | `settings.toml` | 10 |
| `ui.status_width`: the wrist pieces' width | `settings.toml` | 0.09 m |
| HUD in pieces (`ui.split_hud`) | Display tab | on |
| Subtitles: distance, below gaze, text size (`ui.caption_*`) | Display tab | 1.5 m (0.6 to 3), 13 degrees (0 to 30), 70 (40 to 120) |
| `menu_background`: how opaque the menus' background images are (the game's `..._background` tiles, through its own tile update; the text and icons on them stay solid) | `rustvr-oblivion.toml` (the adapter's; `tools\oblivion.ps1 -Settings "menu_background = 0.8"` for a run) | 1 |

## Decisions

- **The world stays live under menus,** so the head can move: the game's frozen picture is skipped.
- **Solid parchment** (playtest 1: it was 80% see-through). `menu_background` can still thin the
  background images, through the game's own tile update.
- **The card sits under where you look**, because it says what looking there does. It eases after
  the head and floats at the distance of the thing it's about (playtest 1).
- **No card or icon for a thing that can be taken** until a hand has it (playtests 1 and 2): the
  same test the gloves use for what they can pull. What can't be picked up (a person, a door, a
  container) keeps its card and icon, since there's nothing to hold. The red hand was the game's
  only warning before a theft; over a held thing it comes back as the game sets it for its target.
- **The flat game's crosshair is hidden.** The hands aim in VR, and a dot fixed in the middle of the
  view is only in the way. The game's own Crosshair setting isn't used: it hides the icons too.
- **The wrist HUD only with the palm turned up** (playtest 2), so a hand passing in front of the
  eyes doesn't bring it up. The palm is read from the controller's grip pose (OpenXR's grip: +X out
  of a left hand's palm).
- **The pointer.** The laser's dot is the pointer (the game's cursor goes where it points), and the
  game's own hover highlight shows what it's on.
- **Dialogue's speaker name over their head** (the flat game shows it in a corner).

## Not done / known issues

- **Most menus aren't cut into pieces.** Only the container and barter, dialogue, the big four,
  the holsters' editor, a book's pages and the main menu are; pop-ups, the pause menu and the rest
  each float as one piece. The map's journal pages are its page as a whole.
- **Typing** (naming a spell or a potion, `TextEditMenu`) needs a keyboard, and the real one doesn't
  reach the game in VR.
- **The dialogue's pieces can still let the world through:** in one run's log they were 0.22 to
  0.95 covered at their middles where the inventory's were 1.00. The runtime makes anything over
  half solid, so a piece under half shows the world. The log line "hud: piece N (group G) is
  covered X at its middle" says which.
- **Not seen in a playtest record:** sleep and wait, the magic page's spell descriptions, haggling,
  a foe's health bar, lockpicking, the level-up, spellmaking, enchanting, alchemy, a service menu
  beside a speaker, a thing's own menu beside it, a container tall enough for its list to go to its
  side, and whether the red hand shows over someone's thing in your hand.
- **Performance** in a big exterior with a menu open, now that the world is rendered under menus.
- **A crash once while loading a save** (in the game's load, before any menu), seen before this work
  too.
- **Cutting another menu.** Set `RUSTVR_OBLIVION_DUMP_TILES=1` and the adapter logs each menu's
  tiles with their boxes, once; each menu's pieces are a few lines in `crates/oblivion/src/hud.rs`.

## History

Newest first.

- **Playtest 2.** The wrist pieces came up whenever the hand passed: now only with the palm turned
  up, stacked up the inside of the wrist. An axe, then a dark silhouette of the character, showed
  behind the inventory: the game's own `Menu3DRoot` drawn into the UI, now culled under the big
  four. Item names still flashed as the gaze left them: the game was fading the card it had been
  given; now it's never given a thing only looked at. From the log, the inventory's pieces were
  solid and the dialogue's weren't.
- **Playtest 1.** The compass went onto the wrist with the bars, bigger. The menus' transparency was
  removed (`menu_background` and `menu_opacity` 1), then the UI pass made to keep its alpha and menus
  drawn as solid pages. A flash of the flat screen on opening: the runtime now shows nothing for up
  to half a second while a menu's pieces haven't come, waits for the picture that matches a layout,
  and shows nothing while the HUD's layout is empty as it fades back. The card was made to ease
  after the head, float in front of its thing and keep its size. The laser took the game's cursor
  colours. The crosshair was hidden; then the card and the hand icon for things that can be taken.
- **Before playtest 1**, on the emulated headset: the chest (hover, scroll, switch, B, How Many?),
  the big four and their switcher, a description, dialogue and barter with a copy of Jensine, a
  message box, the hover tick, the pause menu with the settings, the HUD.
