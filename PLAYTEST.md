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

## Since the first headset play (2026-10-30)

- **A book opens as soon as it's in your hand**: no grip with the other hand.
- **Its pages face you** however you turn the book: they float over it, upright, turned to your eyes,
  just in front of it.
- **Flick to turn**: a flick of your free hand turns a page. Right to left goes on, left to right
  goes back.
- **The map is at your right hip**, the side the sword isn't on: the belt pouch at the front was too
  small a place to find by feel.

## Since the second headset play (2026-10-30)

- **Flick anywhere**: a medium sideways flick of your empty free hand turns the page, wherever the
  hand is. It's the controller's own speed now, not the hand's against the pages: those turn with
  your head and move with the book, so pages turned by themselves and real flicks were missed.
- **The map stays in your hand** whatever you click on it: the journal's tabs used to send it to
  float ahead of you. Let go, and it goes straight back to the hip with nothing left floating (it
  used to show ahead of you for a moment as it shut).
- **The map is centred on your hand**, not held out to one side by its edge.

The flat game's crosshair is gone (taken from the menus branch, which has it from riding): where you
look shows only the icon of what you can activate, red when taking it is stealing.

## What to try, in order

1. **Read a book off a shelf, two ways.**
   - **By hand:** grip a book (or point at it and flick the gloves, as anything else). As soon as
     it's in your hand the game opens it as activating it does, and its open pages (the game's book
     art, both pages, tipped a little towards you about the spine) float over it, turned to your
     eyes. Turn the book any way: the pages stay facing you.
   - **By looking:** look at a book on a shelf and pull the trigger (the game's Activate). It flies
     into your **off hand** and opens there.
   - It's about as big as a large open book (38 by 28 cm): bring it nearer to read it bigger. Your
     other hand in front of the pages covers them, as a real hand would.
2. **Turn a page.** Flick your free hand sideways, right to left, a medium flick: the page turns
   onwards. Left to right turns back. Anywhere: the hand needn't be near the pages. Let the hand slow
   before the next flick; bringing it back after a flick doesn't turn the page back (the way back is
   shut for a little over a second after a turn). Or pull the trigger at a page's edge and pull it
   across. You feel a tick and hear the game's own page turn. The right stick turns pages too.
3. **Take it, or put it down.** The laser comes from your free hand while you hold a book:
   - point at **Take** and pull the trigger: the game takes it into your pack (it's the shop's: the
     owner reacts as the game has them react to a theft);
   - **Exit**, or **B**: it shuts, back to the closed book in your hand. It stays shut until you let
     go of it and take it again;
   - your other hand's grip on the book takes it as anything else (from hand to hand);
   - let go of the book while it's open: it shuts and falls.
4. **A skill book.** Take one in your hand (drop *Proper Lock Design* out of your pack and pick it
   up, or find one): opening it raises the skill exactly as the flat game does (the game's message shows
   once it's shut).
5. **From your pack.** Open your inventory (B on the left hand) and click the guide or the note:
   - it's held in your **off hand** like a sheet, centred on the hand, always turned to your eyes;
   - turn the guide's pages the same way; the note is the game's scroll, rollers and all, in one
     piece (a sweep or the stick scrolls it);
   - B shuts it, back to the inventory.
6. **The map.** Reach to your **right hip**, at the belt (you feel a tick as your hand gets there),
   and grip:
   - the game's map opens in your hand, centred on it like a sheet of paper held from behind,
     turned to your eyes: the **local map** indoors, the **world map** outside (step out of the
     shop's door for it);
   - its tabs are under it (local map, world map, and the journal's three pages): the journal's pages
     stay in your hand too;
   - grip **either side** with your other hand: it's held in both hands, where each took it; pull them
     apart to make it bigger, and it stays that big;
   - with one hand on it, grip its **face** with the other and move: the game drags its map (its own
     pan; the hand in front of the map covers it);
   - holding it in both hands, let go with the first: the other hand has it, where it took it;
   - point at a place with the laser (it comes from your free hand) and pull the trigger: the game's
     own fast travel, its question and its rules;
   - put it back at the hip, or just let go of it, anywhere: it goes back to the hip and the map
     shuts.

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

- **A book opens as it's taken** (the first build waited for the other hand's grip). It's read once
  while it's held: shut with Exit or B (or refused, as "you can't read that in combat"), it stays
  shut until it's let go of and taken again, so it doesn't spring open again at once. A **magic
  scroll** doesn't open: activating one makes the game take it into the pack (stealing it, in a
  shop), so it stays a thing in the hand.
- **The game reads it**: opening runs the book's own activation by the player (the reference's own
  `Activate`, as pressing Activate does), so a skill book's rise, a note's script, "you can't read
  that in combat" and Take's theft are the game's.
- **Pages over the closed book, facing you.** Oblivion has no open book models, so the game's own book
  art (an open book) floats over the closed book, centred on it, big enough to read, turned to your
  eyes from its middle (the first build laid it on the cover, where it turned with the wrist), just
  in front of the book and the fingers round it.
- **Turning by a flick**: the free hand's controller velocity, sideways as the eyes see it (level),
  1 m/s or more with most of its speed sideways, anywhere; the hand empty and not gripping; none in
  the 0.4 s after the pages appear. The first build's sweep had to start well out on a page and cross
  the spine within a few centimetres (in the first play the buttons turned the pages instead); the
  second's flick was the hand's speed against the pages, near them, and turned pages at random as
  the pages turned with the head and moved with the book. A trigger pinch pulled across still turns
  one. The hand has to slow before it flicks again, and the way back after a turn is shut for 1.2 s,
  so the hand coming back doesn't turn it back.
- **The game's own page turn**: a notch of its mouse wheel over the book (its wheel turns a book's
  pages, with its sound).
- **Drawn with the body.** Pages and the map are drawn in the body's depth: a hand in front of a page
  covers it, the fingers behind the map are hidden.
- **Solid paper.** Oblivion draws its book and map pages as light added over the screen, with no
  coverage (its UI starts on the world's last blending); in the hand they're as solid as what the
  game drew, and the black it leaves is clear.
- **A sheet faces you, centred on the hand.** The map, and a book read from the pack, are held at
  their middle (the hand behind them, hidden), always turned to your eyes, upright, a little smoothed.
  Spread between the hands, the points of it each hand took stay in them (a pinch to zoom: nothing
  jumps as the second hand takes it), and handed over, the second hand keeps it where it took it.
- **The map never leaves the hand but by letting go**: every page of the map menu, the journal's
  too, stays in the hand (only the map's pages are dragged by a grip). Let go, it's on its way back
  to the hip at once and nothing of the map is shown while the game shuts it.
- **The map lives at the right hip** (`map_slot` in `rustvr-oblivion.toml`: hip_right, hip_left,
  belt, chest_left, chest_right). The game's one-handed weapon is on the left hip, so the right is
  free here, and a hip is a 13 cm place where the hand drops by itself; the belt pouch is 8.5 cm in
  front of the belly with nothing to see there, and in the first play it wasn't found. The holsters
  branch keeps food at the right hip, so the two will need sharing out when they're merged.
- **The game is paused while a book or the map is open** (as in the flat game), but your hands aren't:
  you can move the book, hand it over, let it go, spread the map.

## Not done

- **The local map's picture** indoors. The game draws the local map by rendering the cell from above
  (`MapMenu`'s local map builder, run when its tab is turned to); in VR its window stays
  blank. Its markers (your arrow, doors) show. The world map, a picture the game loads, shows.
- **Pages on an upright book** sat a little low on it in the one test of a book brought to the hand
  standing up (they were centred on the book's collision box, and its model's bound gave the same
  middle, so the cause wasn't found). They're now over the book facing the eyes, centred on its
  model; whether they still sit low hasn't been seen.
- **Magic scrolls** aren't read in the hand: Oblivion takes a magic scroll when it's activated rather
  than reading it.
- **A book's own model when read from the pack**: none in the hand, just its pages.
- **Testing aids** left in: `RUSTVR_TEST_TAKE=<hand>:<hex id>` (an object into a hand),
  `RUSTVR_HELD_FLAT=1` (held pages in the world, not with the body), `RUSTVR_SHEETS_SOLID=1..3`
  (sheets in debug colours), `RUSTVR_OBLIVION_TRACE_UI=1` (the UI's blending), and the console's
  `ActivateRef <hex id>`.
