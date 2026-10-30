# Playtest: books, notes and the map

Save: rustvr-books

`tools\play.ps1 books` boots this build into **rustvr-books**: First Edition, the bookshop in the
Imperial City's Market District, at two in the afternoon, by its door. Its shelves are full of books
(the shop's: taking one is stealing, and the owner is there). In your pack are a skill book (*Proper
Lock Design*, Security), a note (*Undelivered Letter*) and *Guide to the Imperial City*.
`tools\play.ps1 -Close` puts it all away. When you're done, press a key in the playtest window
(don't just close it): that's what saves the runtime's log.

Everything you read is the game's own book menu: its parchment, its fonts, its own page layout of
the book's text and pictures, its page-turn sound, its Take and Exit. It floats on the book in your
hand instead of the flat screen.

## How it is now (after eight headset plays, 2026-10-30)

- **A book is held like anything else.** Take one in your hand and the game plays on as ever: you
  walk, turn, fight, the HUD's there, time passes. The only difference is the book's pages float
  over it, turned to your eyes.
- **It opens as soon as it's in your hand**, and shuts when you let go, or with its Exit.
- **Your fingertip works it**: lift your free hand's index finger off the trigger to point it and
  touch Take, Exit, or the page arrows. No laser.
- **A page turns with a swipe of your fingertip** across the pages, right to left onwards, left to
  right back: on them or a few centimetres off, barely a flick.
- **The pages are steady**: smoothed while you hold the book still, following closely as you move it.
- **The map** is at your right hip, centred on your hand a hand's depth in front of it, stays in your
  hand on every tab, and goes straight back to the hip when you let go. (The map is still the game's
  menu: the game pauses while it's open, as on the flat screen.)

The flat game's crosshair is gone (taken from the menus branch, which has it from riding): where you
look shows only the icon of what you can activate, red when taking it is stealing.

## What to try, in order

1. **Read a book off a shelf, two ways.**
   - **By hand:** grip a book (or point at it and flick the gloves, as anything else). As soon as
     it's in your hand its open pages (the game's book art, both pages, tipped a little towards you
     about the spine) float over it, turned to your eyes. Turn the book any way: the pages stay
     facing you. Walk about with it, turn with the stick, look at the HUD: the game doesn't stop.
   - **By looking:** look at a book on a shelf and pull the trigger (the game's Activate). It flies
     into your **off hand** and opens there.
   - It's about as big as a large open book (38 by 28 cm): bring it nearer to read it bigger. Your
     other hand in front of the pages covers them, as a real hand would.
2. **Turn a page.** Swipe your free hand's fingertip across the pages, right to left: the page turns
   onwards, with the game's own sound and a tick in your hand. Left to right turns back. It wants
   hardly any speed; let the finger slow (or leave the pages) before the next swipe, and pause a beat
   before swiping back the other way. Touching the page arrows turns them too.
3. **Take it, or put it down.** Point your free hand's index finger and touch a button (it presses
   as your fingertip goes in onto it and comes off still on it; a tick as it goes in):
   - **Take**: the game takes it into your pack (it's the shop's: the owner reacts as the game has
     them react to a theft);
   - **Exit**: it shuts, back to the closed book in your hand. It stays shut until you let go of it
     and take it again;
   - your other hand's grip on the book takes it as anything else (from hand to hand);
   - let go of the book while it's open: it shuts and falls.
4. **A skill book.** Take one in your hand (drop *Proper Lock Design* out of your pack and pick it
   up, or find one): opening it raises the skill exactly as the flat game does.
5. **From your pack.** Open your inventory (B on the left hand) and click the guide or the note:
   - it's held in your **off hand** like a sheet, centred on the hand, always turned to your eyes;
   - swipe the guide's pages the same way; the note is the game's scroll, rollers and all (a swipe
     scrolls it);
   - B shuts it, back to the inventory. (This one is over the inventory, a menu, so the game's paused
     as with any open menu.)
6. **The map.** Reach to your **right hip**, at the belt (you feel a tick as your hand gets there),
   and grip:
   - the game's map opens in your hand, centred on it, turned to your eyes: the **local map**
     indoors, the **world map** outside;
   - its tabs are under it (local map, world map, and the journal's three pages): touch them with
     your fingertip; the journal's pages stay in your hand too;
   - grip **either side** with your other hand: it's held in both hands, where each took it; pull
     them apart to make it bigger, and it stays that big;
   - with one hand on it, grip its **face** with the other and move: the game drags its map;
   - touch a place you've found: the game's fast travel question floats ahead (answer it with the
     laser, which comes back for it);
   - put it back at the hip, or just let go of it anywhere: it goes back to the hip and shuts.

## The rounds so far

1. The map wasn't found in the belt pouch (moved to the right hip); a book should open by itself,
   face the eyes, and turn by a flick.
2. The flick (against the pages) was random; the map seemed to drop (a journal tab sent it ahead)
   and stood off to one side of the hand.
3. A lighter flick; pages shook with the hand (now steadied).
4. No laser on what's held (fingers instead); the flick only near the pages; the map's fingers came
   through it (now a hand's depth in front).
5. A let-go book leapt into the other hand; the flick missed most of the time.
6. Picking a book up paused the game; flicking back hardly worked.
7. Walking while reading was asked for.
8. A book shouldn't be a menu at all, just a held thing with its pages floating on it; the flick was
   still hard and flashed something big in the face (a page turn was a mouse-wheel notch, which the
   game, running under the book, also took for its camera zoom). Now: no menu, fingertip swipes.

## What wasn't checked

- **A book read with the game in play** is new in round 9 and only worked out, never
  run: Oblivion never shows its book menu with no menu up on the flat screen. Things to look for:
  the pages not showing (or not changing as they turn), the HUD missing more than the middle, the
  game doing something odd when a book opens or shuts, a crash.
- **The page arrows and scrolling a note** by fingertip and swipe, now done on the menu's own
  values rather than the game's mouse.
- **Fast travel** from the held map (a fingertip's tap on a place, the game's question).
- **The local map's picture**: indoors the local map's window showed the parchment and your arrow
  but not the drawing of the room (see *Not done*).

## Decisions

- **A book read in the hand isn't a menu** (round 8: "it's just the same as anything else except a
  HUD menu pops up"). Oblivion goes into its menus (its world stopped, the player's controls the
  menu's, the HUD gone) when a menu goes on the interface's menu stack (its push: the first one
  sets the game's mode from play to menus). A book opened with no menu
  up (taken in the hand, or activated where it lies) isn't pushed, so the game never leaves play;
  its close then takes nothing off the stack (the type it pushed is cleared). Its
  menu is still the game's: built from `book_menu.xml`, laid out and drawn by the game, faded in by
  the interface's own list (which isn't the stack's).
- **Its pages and buttons are the adapter's to press** in play: the game's cursor and mouse are then
  its play (the left button swings, the wheel zooms the camera). A page turn is the step the
  book's own wheel takes: its page step (`user6`, 4020) by two, back to 0, and its
  page-turn sound (`user8`) if its page (`user9`) moved; a scroll scrolled a notch (`user9`). Take and
  Exit are its own `HandleClick(32)` and `(31)`; a fingertip press is on the button it went down on
  and came off on (`book_take`, `book_exit`, `book_prev`, `book_next` and a scroll's).
- **The HUD under the book**: the game draws the book over its HUD in the one picture of both, so the
  HUD's pieces under the book's area (the reticle, the card, the compass, subtitles) are left out
  until it shuts; the rest of the HUD is there.
- **A book opens as it's taken**, once while it's held: shut with Exit (or refused, as "you can't read
  that in combat"), it stays shut until it's let go of and taken again. A **magic scroll** doesn't
  open: activating one makes the game take it into the pack (stealing it, in a shop).
- **The game reads it**: opening runs the book's own activation by the player (the reference's own
  `Activate`, as pressing Activate does), so a skill book's rise, a note's script, "you can't read
  that in combat" and Take's theft are the game's.
- **Pages over the closed book, facing you.** Oblivion has no open book models, so the game's own book
  art floats over the closed book, centred on its model, turned to your eyes from its middle, just in
  front of the book and the fingers round it.
- **Turning by a swipe of the fingertip**: the free hand's index fingertip (the rig's, where the drawn
  finger is) moving across the pages at 0.15 m/s or more, more across them than in or out, within
  10 cm of them and a quarter of their size past their edges; the hand empty and not gripping; none
  in the 0.3 s after the pages appear. The next counts once it's slowed below half that or left the
  pages; the way back is shut for 0.45 s after the swipe's stroke ends (the finger coming straight
  back isn't a swipe back). A trigger pinch at a page's edge pulled across still turns one. A
  fingertip swipe fast enough that didn't count is logged with why (`rustvr.log`, "didn't count").
  (Before: a controller flick near the pages, which was hard to make and random.)
- **Pressing with a fingertip**: within 6 cm in front of a piece it's over it; within 1 cm, come from
  in front and going in more than across, it presses until it's 2.5 cm off again; a swipe that began
  as a press lets go on nothing. In a menu (the pack's book, the map) the press is the game's
  cursor and click; in play it's the adapter's (above).
- **Drawn with the body.** Pages and the map are drawn in the body's depth: a hand in front of a page
  covers it, the fingers behind the map are hidden.
- **Solid paper.** Oblivion draws its book and map pages as light added over the screen, with no
  coverage; in the hand they're as solid as what the game drew, and the black it leaves is clear.
- **Steadied, not exact.** Pages in one hand follow it through a one-euro filter (Casiez et al.,
  CHI 2012): a third of a second behind while nearly still, a fifteenth at half a metre a second.
  So text doesn't shake, but a page can trail the book a centimetre or two as it moves. Spread
  between both hands the map follows them exactly.
- **A sheet faces you, centred on the hand, in front of it.** The map, and a book read from the pack,
  are held at their middle, 8 cm in front of the hand (the fingers holding it stay behind it).
  Spread between the hands, the points each hand took stay in them; handed over, the second hand
  keeps it where it took it.
- **The map never leaves the hand but by letting go**: every page of the map menu, the journal's too,
  stays in the hand. Let go, it's on its way back to the hip at once, nothing of it shown as it shuts.
- **The map lives at the right hip** (`map_slot` in `rustvr-oblivion.toml`: hip_right, hip_left,
  belt, chest_left, chest_right): the game's one-handed weapon is on the left. The holsters branch
  keeps food at the right hip, so the two will need sharing out when they're merged.
- **A book read where it lies comes to the off hand held without the grip** (the shelf's look and
  trigger): grip and let go to drop it. A book a hand had open is never brought back (let go of, it
  shuts and falls).

## Not done

- **The local map's picture** indoors. The game draws the local map by rendering the cell from above
  (`MapMenu`'s local map builder, run when its tab is turned to); in VR its window stays
  blank. Its markers (your arrow, doors) show. The world map, a picture the game loads, shows.
- **The map pauses the game**, as the game's map menu does on the flat screen.
- **Magic scrolls** aren't read in the hand: Oblivion takes a magic scroll when it's activated rather
  than reading it.
- **A book's own model when read from the pack**: none in the hand, just its pages.
- **Testing aids** left in: `RUSTVR_TEST_TAKE=<hand>:<hex id>` (an object into a hand),
  `RUSTVR_HELD_FLAT=1` (held pages in the world, not with the body), `RUSTVR_SHEETS_SOLID=1..3`
  (sheets in debug colours), `RUSTVR_OBLIVION_TRACE_UI=1` (the UI's blending), and the console's
  `ActivateRef <hex id>`.
