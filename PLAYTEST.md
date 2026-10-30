# Playtest: books, notes and the map

Save: rustvr-books

`tools\play.ps1 books` boots this build into **rustvr-books**: First Edition, the bookshop in the
Imperial City's Market District, at two in the afternoon. Its shelves are full of books (the shop's:
taking one is stealing). In your pack are a skill book (*Proper Lock Design*, Security), a note (*An
Undelivered Letter*) and *A Guide to the Imperial City*. `tools\play.ps1 -Close` puts it all away.

Everything you read is the game's own book menu: its parchment, its fonts, its own page layout of the
book's text and pictures, its page-turn sound, its Take and Exit. It's laid on the book in your hand
instead of on the flat screen.

## What to try, in order

1. **Take a book off a shelf.** Grip it (or point at it and flick the gloves, as anything else). It's
   the game's own book, closed, in your hand.
2. **Open it with the other hand.** Grip the book with your free hand (the cover; anywhere on it
   works, within a few centimetres) and let go again.
   - The game opens it as it would if you'd activated it: the open book (the game's own book art,
     both pages) lies on the book in your hand, its two pages tipped a little towards you about the
     spine.
   - It's as big as a large open book (about 38 by 28 cm): bring it nearer to read it bigger.
   - Your hand holding it is under it; your other hand in front of it covers the page, as a real hand
     would.
3. **Turn a page.** Sweep your free hand across the pages, a few centimetres above them: from out
   on the right page across the spine to the left page turns onwards; the other way turns back. Or
   pull the trigger at a page's edge and pull it across. You feel a tick, and the game turns its
   page with its own sound. The right stick turns pages too.
4. **Take or put it back.** The laser comes from your free hand while you hold a book:
   - point at **Take** and pull the trigger: the game takes it (it's the shop's: that's stealing, with
     the game's own reaction if you're seen);
   - **Exit**, or grip the book again with your free hand, or **B**: it shuts, back to the closed book
     in your hand;
   - let go of the book while it's open: it shuts and falls.
5. **A skill book.** Do the same with a skill book lying about (or drop *Proper Lock Design* from your
   pack and pick it up): opening it raises the skill exactly as reading it in the flat game does
   (you'll see the game's message once the book shuts).
6. **From your pack.** Open your inventory (B on the left hand) and click a book or the note:
   - it opens held in your **off hand** like a sheet (by its left edge), always turned to your eyes;
   - turn its pages the same way with your other hand; B shuts it, back to the inventory.
   - The note is the game's scroll, rollers and all, in one piece; the stick scrolls it.
7. **The map.** Reach to the **belt pouch** (the front of your belt on your weapon hand's side) and
   grip:
   - the game's map opens in your hand, held by its edge like a sheet of paper, turned to your eyes:
     the **local map** indoors, the **world map** outside;
   - its tabs are under it (local map, world map, and the journal's three pages);
   - grip its **far side** with your other hand: it's held in both hands; pull them apart to make it
     bigger, and it stays that big;
   - with one hand on it, grip its **face** with the other and move: the game drags its map (its own
     pan);
   - point at a place with the laser (it comes from your free hand) and pull the trigger: the game's
     own fast travel, with its question and its rules;
   - put it back at the belt pouch, or just let go of it: it goes back to the belt and the map shuts.
   - Clicking a journal tab sends the journal to float ahead, as before.

## What was checked in the game (emulated headset, captures)

- **A book in the hand, opened by the other hand** (*The Real Barenziah*, *Proper Lock Design* in
  Weye): the game's book menu opened on the other hand's grip, lay on the book in the hand (both
  pages, tipped at the spine, solid), the right hand in front of the page covered it, a sweep turned
  the page with the game's turn (`page_next`), and a second grip shut it.
- **A book read from the inventory** (the console's `EquipItem`): held in the off hand like a sheet,
  legible, its pages turned by a sweep, B shut it.
- **The map from the belt** (indoors, the Weye house): drawn from the belt pouch it opened the game's
  map on the local map, held in the right hand facing the eyes with its tabs under it; the left hand's
  grip on its far side spread it between the hands and pulling them apart made it bigger; put back at
  the belt it shut.

## What wasn't checked

- **The real headset**: the size of the pages (a line of the game's text 1.5 degrees at 45 cm), the
  feel of the sweep, how far a grip on the cover reaches, comfort.
- **Take** from a held book (with its stealing), and a book taken from a real shelf: the tests put
  a book in the hand from the console.
- **The skill rise**: the skill book was read in the hand, but the message isn't in the captures (it
  shows on the HUD after the book shuts).
- **The map's drag** (its face gripped and dragged), **fast travel**, the **world map** outside.
- **A note (scroll)** in the hand or the off hand.
- **The local map's picture**: in every test the local map's window showed the parchment and the
  player's arrow but not the map of the room (a stale picture once). The game draws its local map by
  rendering the cell from above; that render doesn't seem to happen in VR. See *Not done*.

## Decisions

- **Opening by the other hand's grip, not a pull.** The grip on the cover opens it at once (a pull
  would be closer to a real book, but a grip is unmistakable and works from any side). The same grip
  shuts it.
- **The game reads it**: opening runs the book's own activation (`Activate player`), so a skill
  book's rise, a note's script, "you can't read that in combat" and Take's stealing are the game's.
- **Pages on the closed book.** Oblivion has no open book models, so the game's own book art (an
  open book) lies on the book's cover, centred on it, big enough to read. The closed book is under
  it.
- **Turning by a sweep** across the spine, or a trigger pinch pulled across. The way back after a turn
  is shut for a second, so the hand coming back after a turn doesn't turn it back.
- **The game's own page turn**: a notch of its mouse wheel over the book (its wheel turns a book's
  pages, with its sound).
- **Drawn with the body.** Pages and the map are drawn in the body's depth: a hand in front of a page
  covers it, the fingers behind the map are hidden.
- **Solid paper.** Oblivion draws its book and map pages as light added over the screen, with no
  coverage; in the hand they're as solid as what the game drew (the black it leaves is clear).
- **A sheet faces you.** The map, and a book read from the pack, are always turned to your eyes from
  their middle, upright, a little smoothed (a sheet held at its edge swings far with a little turn of
  the wrist).
- **The map lives in the belt pouch** (`map_slot` in `rustvr-oblivion.toml`: belt, hip_right,
  hip_left, chest_left, chest_right); the holsters branch can give it another place.
- **The game is paused while a book or the map is open** (as in the flat game), but your hands aren't:
  you can move the book, hand it over, let it go, spread the map.

## Not done

- **The local map's picture** (see above). The world map is a picture the game loads, so it should
  show; it wasn't seen.
- **Books read from a shelf by looking** (the trigger at a book on a shelf) open in the off hand like
  a sheet rather than coming into the hand.
- **Magic scrolls**: Oblivion takes a magic scroll when it's activated rather than reading it, so
  opening one in the hand takes it into the pack.
- **A book's own 3D model** when read from the pack: none in the hand (just its pages).
- **Handing the map over**: letting go of the map with the hand that drew it puts it back, even
  while the other hand holds its other side.
