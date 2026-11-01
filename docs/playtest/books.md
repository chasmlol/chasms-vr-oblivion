# Playtest: books, notes and the map

Save: rustvr-books

## Playing it

From the repository's root:

```
powershell -File tools\oblivion.ps1 -Save rustvr-books -Settings "map_slot = 'hip_right'"
```

It builds main, installs it, and boots into First Edition, the bookshop in the Imperial City's
Market District, at two in the afternoon, by its door. Its shelves are full of the shop's books
(taking one is stealing, and the owner is there). In the pack: a skill book (*Proper Lock Design*,
Security), a note (*Undelivered Letter*) and *Guide to the Imperial City*. The `-Settings` line
puts the map on the body for this run (it isn't by default, see Settings).
`powershell -File tools\oblivion.ps1 -Close` puts it away and the settings back.

`rustvr-books` is the developer's own save, not in the repository. To make one, boot any save with
`-At <First Edition's cell> -Hour 14`, give the books by their form ids and save
it as `rustvr-books` ([DEVELOPING.md](../DEVELOPING.md), "Test saves"); any shop or house with books does.

The research and the design: `docs/design/books.md`.

## What it does

`crates/oblivion/src/books.rs` and `map.rs`, with the runtime's held menus
(`framework/crates/runtime/src/held_menus.rs`).

- **A book is held like anything else.** Take one in a hand and the game plays on (you walk, turn,
  fight, the HUD's there, time passes); the game's own book menu, laid out by the game (its
  parchment, fonts, page layout and pictures), floats over the book, turned to your eyes. It opens
  as soon as it's in your hand, once while it's held, and shuts when you let go (its pages gone at
  once, the book falling) or with its Exit (then it stays shut until you let go and take it
  again).
- **The game reads it:** opening runs the book's own activation by the player, so a skill book's
  rise, a note's script, "you can't read that in combat" and Take's theft are the game's. A magic
  scroll isn't read: the game takes an activated scroll into the pack.
- **A book where it lies,** looked at and activated with the trigger (the game's Activate), comes
  to the off hand and opens there, held without the grip (grip and let go to drop it).
- **Pages turn by the other hand's grip:** within 25 cm in front of or behind the pages, or half
  their size past their edges, the grip holds them (a tick); every 5 cm it then moves across them
  is a page, leftwards onwards, rightwards back, as many as it goes, with the game's page-turn
  sound. Near the pages that hand's grip takes nothing else.
- **A fingertip presses:** lift the free hand's index finger off the trigger to point it. Within
  6 cm in front of a button it's over it; within 1 cm, coming from in front, it presses until it's
  2.5 cm off again (a tick as it goes in). Take, Exit and the page arrows. No laser on what's held.
- **Steady pages:** pages in one hand follow it through a one-euro filter, smoothed hard while it's
  nearly still and less as it moves.
- **From the pack:** clicked in the inventory (B on the off hand), a book or note is held in the off
  hand like a sheet, centred on the hand and in front of it, always turned to your eyes. The guide
  turns by the grip as above; the note is the game's scroll, rollers and all (the grip dragged
  scrolls it). B shuts it, back to the inventory. This one is over a menu, so the game is paused.
- **The map,** with `map_slot` set: a folded map kept in that body slot. Grip there (a tick as your
  hand arrives) and the game's map menu opens in your hand, 8 cm in front of it and turned to your
  eyes: the local map indoors, the world map outside. Its tabs (the local map, the world map, the
  journal's three pages) are pressed with a fingertip and stay in the hand. The other hand's grip
  on either side holds it in both hands where each took it; pull them apart to make it bigger, and
  it stays that big. With one hand on it, grip its face with the other and move: the game drags its
  map. Touch a place you've found: the game's fast travel question floats ahead (answered with the
  laser). Let go anywhere and it goes back to its slot and shuts. The map is the game's menu, so
  the game pauses while it's open.
- **The flat game's crosshair is hidden** in VR (`crates/oblivion/src/reticle.rs`): where you look
  shows only the icon of what you can activate, red when taking it is stealing.

## What to try

1. **A book off a shelf by hand:** grip one (or point and flick the gloves). Turn it any way: the
   pages face you. Walk, turn with the stick, look at the HUD: the game doesn't stop. Bring it
   nearer to read bigger; your other hand in front of the pages covers them.
2. **By looking:** look at a book on a shelf and pull the trigger: it opens in your off hand.
3. **Turn pages:** the free hand's grip near the book, dragged left; then right to go back. Touch
   the page arrows too.
4. **Take or put down:** point and touch Take (the owner reacts to the theft) or Exit; hand the book
   to the other hand; let go of it open.
5. **A skill book:** drop *Proper Lock Design* from the pack and pick it up: the skill rises as in
   the flat game.
6. **From the pack:** the inventory, then the guide or the note; turn or scroll it; B shuts it.
7. **The map** at your right hip: open it indoors and outside; touch the tabs and the journal's
   pages; spread it between your hands; drag it; fast travel from it; let go.

## Settings

| Where | Setting | Default | What it does |
|---|---|---|---|
| `rustvr-oblivion.toml` | `map_slot` | empty | The body slot the map is kept in: `hip_right`, `hip_left`, `belt` (the belt pouch on the weapon hand's side), `chest_left` or `chest_right`. Empty: the map isn't on the body. |
| `settings.toml` `[ui]` | `held_distance` | 0.45 m | A held book or map is as big as a menu at this distance (bigger brought nearer). |
| `settings.toml` `[ui]` | `menu_text_deg` | 1.5 | How tall a line of the game's text looks in the floating menus, degrees. |

`map_slot` is empty by default since the holsters merged: the body's slots hand out the player's
own things, and the map isn't one of them. At the right hip it shares the place with the Food slot.

## Decisions

- **A book read in the hand isn't a menu** (round 8: "it's just the same as anything else except a
  HUD menu pops up"). Oblivion goes into its menus (the world stopped, the controls the menu's, the
  HUD gone) when a menu goes on the interface's menu stack (the first one pushed takes it out of
  play). A book opened with no menu up isn't pushed, so the game never leaves play; its close takes
  nothing off the stack (the menu's note of what it pushed is cleared). It's still the game's menu,
  built from `book_menu.xml`, laid out and drawn by the game, faded in by the interface's own list.
- **Its pages and buttons are the adapter's to press in play,** since the game's cursor and mouse
  are then its play (the left button swings, the wheel zooms the camera). A page turn is the step
  the book's own wheel takes: its page step (`user6`, 4020) by two, and its page-turn sound
  (`user8`) if its page (`user9`) moved; a scroll a notch. Take and Exit are its own clicks on those
  buttons (32 and 31). In a menu (the pack's book, the map) a press is the game's cursor and click.
- **The HUD under the book:** the game draws the book over its HUD in one picture, so the HUD's
  pieces under the book's area (the reticle, the card, the compass, subtitles) are left out until
  it shuts.
- **Pages over the closed book, facing you:** Oblivion has no open book models, so its book art
  floats over the closed book, centred on its model, just in front of the book and the fingers.
- **Turning by the other hand's grip** (round 9: "hold the grip anywhere near the book with second
  hand and flip the pages, nice and simple"). Flicks and swipes timed by speed missed.
- **Drawn with the body,** in its depth: a hand in front of a page covers it, the fingers behind
  the map are hidden.
- **Solid paper:** Oblivion draws its book and map pages as light added over the screen; in the
  hand they're as solid as what the game drew, and the black it leaves is clear.
- **Steadied, not exact:** a third of a second behind while nearly still, a fifteenth at half a
  metre a second, so a page can trail the book a centimetre or two. Spread between both hands, the
  map follows them exactly.
- **A sheet is held at its middle, a hand's depth in front of the hand,** so the fingers holding it
  stay behind it. Spread, the map keeps the points each hand took; handed over, the second hand
  keeps it where it took it.
- **The map never leaves the hand but by letting go:** every page of the map menu, the journal's
  too, stays in the hand.

## Not done / known issues

- **The local map's picture** indoors: the game draws it by rendering the cell from above (the map
  menu's local map builder, run when its tab is turned to); in VR its window shows the parchment and
  the markers (your arrow, doors) but not the room. The world map shows.
- **The map pauses the game,** as the game's map menu does on the flat screen.
- **Magic scrolls** aren't read in the hand.
- **A book read from the pack** has no model in the hand, just its pages.
- **Never checked on the headset:** fast travel from the held map; the page arrows and a note
  scrolled by fingertip and grip.
- **Testing aids left in:** `RUSTVR_TEST_TAKE=<hand>:<hex id>` (an object into a hand),
  `RUSTVR_HELD_FLAT=1` (held pages in the world, not with the body), `RUSTVR_SHEETS_SOLID=1..3`
  (sheets in debug colours), `RUSTVR_OBLIVION_TRACE_UI=1` (the UI's blending), and the console's
  `ActivateRef <hex id>`.

## History

Ten headset rounds, newest first.

10. The grip worked; a book let go of flashed its pages in the off hand as they faded. Now a book
    let go of is gone at once.
9. The book as a held thing worked; the fingertip swipe still often missed. Now the other hand's
   grip near the book, dragged across, turns the pages.
8. A book shouldn't be a menu at all, just a held thing with its pages on it; the flick was hard
   and flashed something big in the face (a page turn was a mouse-wheel notch, which the game under
   the book also took for its camera zoom). Now no menu, and fingertip swipes.
7. Walking while reading was asked for.
6. Picking a book up paused the game; flicking back hardly worked.
5. A let-go book leapt into the other hand; the flick missed most of the time.
4. No laser on what's held (fingers instead); the flick only near the pages; the map's fingers came
   through it (now a hand's depth in front).
3. A lighter flick; the pages shook with the hand (now steadied).
2. The flick against the pages was random; the map seemed to drop (a journal tab sent it ahead)
   and stood off to one side of the hand.
1. The map wasn't found in the belt pouch (moved to the right hip); a book should open by itself,
   face the eyes and turn by a flick.
