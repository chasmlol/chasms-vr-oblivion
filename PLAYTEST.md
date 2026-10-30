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

## Since the third headset play (2026-10-30)

- **A lighter flick**: 0.6 m/s turns a page (it was 1; your flicks that turned one were 1.0 to 1.3).
- **Steady pages**: a book's pages (and the map, and a sheet from the pack) no longer shake with your
  hand. While you hold it nearly still they're smoothed hard, so your hand's tremor all but goes;
  as you move it they follow more and more closely, so they keep up with a deliberate move.

## Since the fourth headset play (2026-10-30)

- **No laser on a book or the map**: press their buttons with your free hand's finger. Lift your
  index finger off the trigger to point it, then touch the button: Take, Exit, Next and Prev on a
  book, the tabs and the places on the map. As the fingertip comes near, the game lights what's
  under it; touch it and you feel a tick. The laser still comes for anything that floats ahead,
  like the map's fast travel question.
- **Flick on the pages, lighter**: a flick turns a page only with your hand over the pages or near
  them (a forearm's length in front, or a little behind or beside them), and a lighter one does it
  (0.45 m/s).
- **The map is in front of your hand**: it lies a hand's depth in front of the hand holding it (or
  both, spread), so your fingers don't come through it.

## Since the fifth headset play (2026-10-30)

- **A let-go book stays let go**: an open book you let go of used to leap into your other hand and
  stay held there without the grip (for a moment the game's book was open on a book no hand had,
  which is how a book you look at on a shelf is brought to the hand). It now just shuts and falls.
- **A more forgiving flick** (the same speed):
  - flicking again and again works: the next flick counts as soon as the hand slows to come back,
    where before it had to all but stop;
  - your fingertip brushing the pages as a flick starts no longer counts as pressing them (that
    stopped the flick): a press is the fingertip going into the page, not across it;
  - a flick that arcs (half sideways is enough), and a hand through the pages or out past their
    edges, count.

## Since the sixth headset play (2026-10-30)

- **The game goes on while you read a book in your hand**: time passes, people walk about and talk,
  the weather and the physics and the sounds go on, as with no menu open. You don't walk while it's
  open, and the hand on the pages doesn't activate anything (your controls are the book's until you
  shut it). The map, and a book read from your pack (the inventory's open under it), still pause.
- **Flicking back works**: the hand coming straight back after a flick is still ignored, but now
  only for a moment after it slows. Pause a beat after a forward flick, then flick back.
- When you're done, press a key in the playtest window (don't just close it): that's what saves the
  runtime's log for me. The last run's was lost when another feature's game started.

## Since the seventh headset play (2026-10-30)

- **Walk while reading**: with a book open in your hand, the off-hand stick walks you as it does
  without a book (the same deadzone, towards where your head looks, and the game body follows you
  when you step about the room); the comfort vignette is as when walking. The right stick still
  turns the pages, and B still shuts the book. Turning with the stick isn't there while reading:
  turn your body.

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
2. **Turn a page.** Flick your free hand sideways over the pages or near them, right to left, a
   light flick: the page turns onwards. Left to right turns back. Let the hand slow
   before the next flick; bringing it back after a flick doesn't turn the page back (the way back is
   shut for a little over a second after a turn). Or pull the trigger at a page's edge and pull it
   across. You feel a tick and hear the game's own page turn. The right stick turns pages too.
3. **Take it, or put it down.** There's no laser on a book: point your free hand's index finger
   (lift it off the trigger) and touch its buttons. The game lights what's under your fingertip as
   it comes near; touching it gives a tick:
   - touch **Take**: the game takes it into your pack (it's the shop's: the owner reacts as the game
     has them react to a theft);
   - touch **Exit**, or press **B**: it shuts, back to the closed book in your hand. It stays shut
     until you let go of it and take it again;
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
   - touch a tab, or a place on the map, with your free hand's fingertip: the game's own click (a
     place you've found: its fast travel question, which floats ahead of you; answer it with the
     laser, which comes back for it);
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
- **Fast travel** from the held map (a fingertip's tap on a place, the game's question).
- **The world going on under a book** was only worked out, never run: the game's
  world never updates with a menu open in the flat game, so something may not expect it (an NPC
  talking to you, combat starting, a script). If anything odd happens while reading, say what.
- **Walking while reading**: that the game takes the player's step while its book menu is up (its
  own walking controls may be off in menus; the step is the adapter's, in `Actor::Move`, which the
  player's update should still call).
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
  0.45 m/s or more with half its speed sideways at least, the hand over the pages or near them (30
  cm in front, 25 cm behind, half their size past their sides, top and bottom: "on the UI or near
  it"); the hand empty and not gripping; none in the 0.4 s after the pages appear. The next counts
  once its sideways speed is under half a flick's (the fifth play's waited for the whole hand to be
  under 0.18 m/s, which a hand flicking again and again never was). A flick fast enough that
  didn't count is logged with why (`rustvr.log`, "didn't count"). The first build's sweep had to start well out on a page and cross
  the spine within a few centimetres (in the first play the buttons turned the pages instead); the
  second's flick was the hand's speed against the pages, near them, and turned pages at random as
  the pages turned with the head and moved with the book. A trigger pinch pulled across still turns
  one. The hand has to slow before it flicks again, and the way back is shut for 0.45 s from when the
  stroke that turned the page (or the hand's return after it) slows: the hand coming straight back
  doesn't turn it back, a flick back after a beat's pause does (the sixth play's 1.2 s from the turn
  dropped nearly every flick back).
- **The game's own page turn**: a notch of its mouse wheel over the book (its wheel turns a book's
  pages, with its sound).
- **Drawn with the body.** Pages and the map are drawn in the body's depth: a hand in front of a page
  covers it, the fingers behind the map are hidden.
- **Solid paper.** Oblivion draws its book and map pages as light added over the screen, with no
  coverage (its UI starts on the world's last blending); in the hand they're as solid as what the
  game drew, and the black it leaves is clear.
- **Steadied, not exact.** Pages in one hand follow it through a one-euro filter (Casiez et al.,
  CHI 2012): a third of a second behind while nearly still, a fifteenth at half a metre a second.
  So text doesn't shake, but a page can trail the book a centimetre or two as it moves. Spread
  between both hands the map follows them exactly (a pinch steadied would slide under them).
- **Fingers, not a laser, on what's held.** A book's and the map's buttons are pressed by the free
  hand's index fingertip (the rig's, where the drawn finger is: straight when it's off the
  trigger). Within 6 cm in front of a piece the game's cursor follows the fingertip (the game lights
  what's under it); the fingertip within 1 cm of it, come from in front, holds the game's button
  down there until it's 2.5 cm off again (a click where it's let go, so sliding off a button before
  lifting doesn't press it). A fingertip sweeping across faster than 0.35 m/s, or moving more across
  the page than into it, isn't pressing, so a flick that brushes the pages doesn't press Take (nor
  stop the flick). What floats (a question over the map) still has
  the laser.
- **A sheet faces you, centred on the hand, in front of it.** The map, and a book read from the pack,
  are held at their middle, 8 cm in front of the hand towards your eyes (a hand's depth: the
  fingers holding it stay behind it, hidden), always turned to your eyes, upright, steadied.
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
- **A book read where it lies comes to the off hand held without the grip** (the shelf's look and
  trigger): grip and let go to drop it. A book a hand had open is never brought back (let go of, it
  shuts and falls).
- **The game goes on under a book read in the hand** (the player asked: it mustn't pause). Oblivion
  stops its world while any menu is open: its frame asks `IsMenuMode` before
  each part of the world's update, and the world's seconds for a frame are 0 while one
  is. Those 15 calls ask the adapter instead, which says no while the book being read is in a hand
  and its menu is on top; everything else still sees the menu (the cursor and clicks are the
  book's, the player's Activate is off, scripts run their menu-mode blocks). The map, and a book
  read from the pack, pause as the flat game does.
- **Walking while reading** (the player asked): the adapter says the book's menu is one the game goes
  on under (`GS_MENU_WALKS`), and the runtime then walks with the off-hand stick as without a menu
  (its move setting, deadzone, roomscale following, the comfort vignette); the menu's buttons and
  the right stick stay the book's (no stick turning while reading). The step is the adapter's own
  (`Actor::Move`, `crate::movement`), taken in the player's update, which goes on with the world. Your hands are never paused: you can move the book, hand it over,
  let it go, spread the map.

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
