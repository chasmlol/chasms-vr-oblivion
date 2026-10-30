# Playtest: books, notes and the map

Save: rustvr-books

`tools\play.ps1 books` boots this build into **rustvr-books**: First Edition, the bookshop in the
Imperial City's Market District, at two in the afternoon, by its door. Its shelves are full of books
(the shop's: taking one is stealing, and the owner is there). In your pack are a skill book (*Proper
Lock Design*, Security), a note (*Undelivered Letter*) and *Guide to the Imperial City*.
`tools\play.ps1 -Close` puts it all away.

Everything you read is the game's own book menu: its parchment, its fonts, its own page layout of
the book's text and pictures, its page-turn sound, its Take and Exit. It's laid on the book in your
hand instead of the flat screen.

## What to try, in order

1. **Read a book off a shelf, two ways.**
   - **By hand:** grip a book (or point at it and flick the gloves, as anything else). It's the
     game's own book, closed, in your hand. Then **grip it with your other hand** (the cover;
     anywhere within a few centimetres) and let go again: the game opens it as activating it does,
     and its open pages (the game's book art, both pages, tipped a little towards you about the
     spine) lie on the cover turned towards you.
   - **By looking:** look at a book on a shelf and pull the trigger (the game's Activate). It flies
     into your **off hand** and opens there.
   - It's about as big as a large open book (38 by 28 cm): bring it nearer to read it bigger. Your
     other hand in front of the pages covers them, as a real hand would.
2. **Turn a page.** Sweep your free hand across the pages, a few centimetres above them: from out on
   the right page across the spine to the left page turns onwards; the other way turns back. Or pull
   the trigger at a page's edge and pull it across. You feel a tick and hear the game's own page turn.
   The right stick turns pages too.
3. **Take it, or put it down.** The laser comes from your free hand while you hold a book:
   - point at **Take** and pull the trigger: the game takes it into your pack (it's the shop's: the
     owner reacts as the game has them react to a theft);
   - **Exit**, or grip the book again with your free hand, or **B**: it shuts, back to the closed book
     in your hand;
   - let go of the book while it's open: it shuts and falls.
4. **A skill book.** Open one in your hand (take *Proper Lock Design* out of your pack by dropping it,
   or find one): opening it raises the skill exactly as the flat game does (the game's message shows
   once it's shut).
5. **From your pack.** Open your inventory (B on the left hand) and click the guide or the note:
   - it's held in your **off hand** like a sheet, by its left edge, always turned to your eyes;
   - turn the guide's pages the same way; the note is the game's scroll, rollers and all, in one
     piece (a sweep or the stick scrolls it);
   - B shuts it, back to the inventory.
6. **The map.** Reach to the **belt pouch** (the front of your belt on your weapon hand's side) and
   grip:
   - the game's map opens in your hand, held by its edge like a sheet of paper, turned to your eyes:
     the **local map** indoors, the **world map** outside (step out of the shop's door for it);
   - its tabs are under it (local map, world map, and the journal's three pages);
   - grip its **far side** with your other hand: it's held in both hands; pull them apart to make it
     bigger, and it stays that big;
   - with one hand on it, grip its **face** with the other and move: the game drags its map (its own
     pan; the hand in front of the map covers it);
   - holding it in both hands, let go with the first: the other hand has it;
   - point at a place with the laser (it comes from your free hand) and pull the trigger: the game's
     own fast travel, its question and its rules;
   - put it back at the belt pouch, or just let go of it: it goes back to the belt and the map shuts.
   - A journal tab sends the journal to float ahead, as before.

## What was checked in the game (emulated headset, captures)

- **A book in the hand, opened by the other hand** (*The Real Barenziah*, *Proper Lock Design*): the
  game's book menu opened on the other hand's grip and lay on the book in the hand (both pages,
  tipped at the spine, solid, legible); the right hand in front of the page covered it; a sweep
  turned the page with the game's own turn; a second grip shut it.
- **The skill rise**: opening *Proper Lock Design* in the hand made the game raise Security (the book
  teaches nothing more afterwards, as the game marks it).
- **A book on a shelf activated where it lies** (*Darkest Darkness* in First Edition): it came to the
  off hand and opened on it; B shut it. **Take** with the laser took it into the pack, the book left
  the hand, and the shop's owner came to talk (the game's theft).
- **A book and a note read from the pack** (the console's `EquipItem`): held in the off hand, turned to
  the eyes, legible (the note's handwriting too); a sweep turned the book's page and scrolled the
  note; B shut them.
- **The map**: drawn from the belt pouch it opened the game's map (the local map in a house, the
  world map outside, "The Great Forest" with the Gold Road and the player's arrow), held in the right
  hand facing the eyes, its tabs under it. The left hand's grip on its far side spread it between the
  hands and pulling them apart made it bigger; the left hand's grip on its face dragged the world map
  (from the Imperial City across to Skingrad); letting go with the right hand left it in the left;
  letting go with that put it back on the belt and shut the map.

## What wasn't checked

- **The real headset**: the size of the pages (a line of the game's text 1.5 degrees at 45 cm), the
  feel of the sweep, how near the free hand has to be to open a book, comfort.
- **Fast travel** from the held map (its laser clicks, the game's question).
- **A book taken off a shelf by hand** (the gloves' pull and the grip are the existing ones; the tests
  put books in the hand from the console or brought them by activating them).
- **The local map's picture**: indoors the local map's window showed the parchment and your arrow
  but not the drawing of the room (see *Not done*).

## Decisions

- **Opening by the other hand's grip, not a pull.** It's unmistakable and works from any side. The
  same grip shuts it. A free hand within 8 cm of the book counts (the pages over the cover can stop
  the hand a little short of it).
- **The game reads it**: opening runs the book's own activation by the player (the reference's own
  `Activate`, as pressing Activate does), so a skill book's rise, a note's script, "you can't read
  that in combat" and Take's theft are the game's.
- **Pages on the closed book.** Oblivion has no open book models, so the game's own book art (an open
  book) lies on the closed book's cover, centred on it, big enough to read; the closed book is under
  it. Turn the book over and the pages are on its other cover.
- **Turning by a sweep** across the spine, or a trigger pinch pulled across. The way back after a turn
  is shut for a second, so the hand coming back doesn't turn it back.
- **The game's own page turn**: a notch of its mouse wheel over the book (its wheel turns a book's
  pages, with its sound).
- **Drawn with the body.** Pages and the map are drawn in the body's depth: a hand in front of a page
  covers it, the fingers behind the map are hidden.
- **Solid paper.** Oblivion draws its book and map pages as light added over the screen, with no
  coverage (its UI starts on the world's last blending); in the hand they're as solid as what the
  game drew, and the black it leaves is clear.
- **A sheet faces you.** The map, and a book read from the pack, are always turned to your eyes from
  their middle, upright, a little smoothed (a sheet held at its edge swings far with a little turn of
  the wrist). A book in your hand isn't: its pages lie on it and move with it.
- **The map lives in the belt pouch** (`map_slot` in `rustvr-oblivion.toml`: belt, hip_right,
  hip_left, chest_left, chest_right); the holsters branch can give it another place.
- **The game is paused while a book or the map is open** (as in the flat game), but your hands aren't:
  you can move the book, hand it over, let it go, spread the map.

## Not done

- **The local map's picture** indoors. The game draws the local map by rendering the cell from above
  (`MapMenu`'s local map builder, run when its tab is turned to); in VR its window stays
  blank. Its markers (your arrow, doors) show. The world map, a picture the game loads, shows.
- **Pages on an upright book** sat a little low on it in the one test of a book brought to the hand
  standing up (they're centred on the book's collision box, and its model's bound gave the same
  middle, so the cause wasn't found); lying in the hand they sit on it.
- **Magic scrolls**: Oblivion takes a magic scroll when it's activated rather than reading it, so
  opening one in the hand takes it into the pack.
- **A book's own model when read from the pack**: none in the hand, just its pages.
- **Testing aids** left in: `RUSTVR_TEST_TAKE=<hand>:<hex id>` (an object into a hand),
  `RUSTVR_HELD_FLAT=1` (held pages in the world, not with the body), `RUSTVR_SHEETS_SOLID=1..3`
  (sheets in debug colours), `RUSTVR_OBLIVION_TRACE_UI=1` (the UI's blending), and the console's
  `ActivateRef <hex id>`.
