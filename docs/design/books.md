# Books, notes and the map: research and design

Research written before building books and the map in the hands: a book or note held open in the
hand, drawn in the game's own book art with the game's own page layout, its pages turned by the
other hand; skill books and the like doing what the game does when read; the map unfolded from the
belt into the hands, the game's own world and local map art, markers and fast travel. It builds on
the floating menus ([menus.md](menus.md)): the book menu and the map menu are the game's own menus,
drawn by the game into the HUD texture, cut into pieces and placed by the runtime, now on a thing in
the hands. Section 4 is what was built (the framework's `held_menus.rs` and `hud.rs` protocol, the
adapter's `books.rs`, `map.rs` and `hud.rs`), as the headset plays changed it.

Legend: **(snippet)** a search engine's summary, not a page read in full; **[unverified]** prior
knowledge or arithmetic, not confirmed by a source. Inspiration only; nothing copied.

## 1. What players like and dislike

### Books and notes

- **Skyrim VR's books** open as the flat book menu in front of the face; players asked for books held
  in the hands with pages turned by hand
  ([Nexus forum](https://forums.nexusmods.com/topic/9483463-pov-book-reading-animation-with-hands/),
  snippet). The "Immersive Reading" mod is the answer they made: the book drops from the inventory
  into the hand, pages turn by a gesture of the free hand whose size is a setting, and the tall
  book models are shrunk because they look too big held close
  ([Nexus](https://www.nexusmods.com/skyrimspecialedition/mods/173931), snippet).
- **Riven VR** (the most direct feedback on turning pages): hover the free hand at the page's edge
  until it highlights, grip and swipe. Complaints: the page feels tethered "like moving a bar of soap
  through another bar of soap", it snaps back, the book shakes with the hand (players asked for
  smoothing), the grip on the book is lost too easily, some couldn't find how to turn at all; Myst
  VR's handling felt easier ([Steam](https://steamcommunity.com/app/1712350/discussions/0/4524512379487209415)).
- **Myst VR**: books feel like objects; the complaint is things snapping back to their place when
  handed from hand to hand ([Road to VR](https://www.roadtovr.com/myst-vr-quest-review/)).
- **Half-Life: Alyx**: players remember picking papers up and studying them close (snippet,
  [vr.org](https://vr.org/articles/half-life-alyx-still-the-gold-standard)). Valve's own commentary:
  information in one readable place beats information split about
  ([commentary](https://combineoverwiki.net/wiki/Developer_commentary/Half-Life:_Alyx)).
- **Lone Echo**'s tablet is a physics object the player brings to their own reading distance; text
  about 1.75 times print size, bold weights, light on medium-dark reads best (dark ink thins through
  the lenses), never text through geometry; controls easy to use on purpose and hard by accident
  ([Road to VR](https://roadtovr.com/designing-lone-echo-echo-arena-virtual-touchscreen-interfaces-robert-duncan/2/)).
- **How people hold a real book** (a study of 30 for a VR library): most keep it in the hand that
  took it, balanced in the palm by the spine; most overshoot when leafing and turn back. Their VR
  advice: the free hand turns by moving left or right across the book, controlling the direction
  ([Waikato](https://researchcommons.waikato.ac.nz/server/api/core/bitstreams/a853bc91-1234-4f11-a96f-60ea73568e40/content)).
  Moving the pages beat moving the camera for comfort and reading speed (Dingler et al., OzCHI 2020,
  [AAU](https://vbn.aau.dk/en/publications/page-turning-techniques-for-reading-interfaces-in-virtual-environ), snippet).

### Maps

- **Skyrim VR**: the world map you fly over was liked
  ([UploadVR](https://uploadvr.com/preview-skyrim-vr-without-teleportation/)); no zoom, and a map so
  close you look under it, were not ([Steam](https://steamcommunity.com/app/611670/discussions/0/1634166237662619767/)).
  Players' fix is NavigateVR: physical maps drawn from a holster, the right region picked
  automatically, with add-ons for quest markers and the player's place and many map packs (snippet,
  [Nexus](https://www.nexusmods.com/skyrimspecialedition/mods/47174)). A map that has to be held up
  or it fades was disliked (Equip-able Maps, snippet).
- **Fallout 4 VR**: raising the wrist to open the Pip-Boy felt natural, holding the arm up tires,
  and the map moved by the sticks was "effectively unusable"
  ([UploadVR](https://uploadvr.com/fallout-4-vr-review/)).
- **The Walking Dead: Saints & Sinners**: the journal and map from a chest holster; the player's
  place updating live while walking with it open was praised
  ([COGconnected](https://cogconnected.com/feature/ten-things-love-saints-sinners-psvr/)).
- **Into the Radius**: a paper map in the hand with a magnifier for detail; complaints are about
  missing landmarks, not the handling ([Steam](https://steamcommunity.com/app/1012790/discussions/0/3181232619052061498/)).
- **Pan and zoom by hand**: there's no standard (Satriadi et al.,
  [ICA](https://ica-abs.copernicus.org/articles/1/147/2019/)); grab-to-pan is intuitive but tires the
  arm ([iLab](https://ilab.ucalgary.ca/publications/vr-2019-satriadi/), snippet). Two hands apart to
  zoom, a hand dragging to pan is what map apps do (Google Earth VR: grab and drag, two hands to
  scale) [unverified].
- **Earlier Elder Scrolls VR**: OpenMW VR's menus sit at fixed places and end up inside things
  ([docs](https://openmw-vr.readthedocs.io/en/latest/manuals/openmw-vr/common-problems.html)); OBVR
  (the old Oblivion mod) has the Tab menus on a wrist, clicked by the other hand's laser, and says
  nothing of books or the map ([GitHub](https://github.com/Naragorn/OBVR)).

### What to take from it

- The book stays in the hand that took it; the free hand turns pages by moving across them, either
  way, with a clear cue (a tick, the game's page sound) and no snapping back; a turn is easy to
  undo (the other way).
- No tethered page dragging, no hold-it-up-or-it-fades, nothing that closes by accident: a book
  closes only when the hands close it, it's let go, or B.
- The map from a holster on the body, held like paper, zoomable, panned by hand not stick, the
  player's own arrow on it.
- Size the page for the headset (text about 1.2 to 1.5 degrees a line), not for a real book.

## 2. Legibility in numbers

| What | Number | Source |
|---|---|---|
| Quest 3 | 2064 x 2208 an eye, about 25 pixels a degree at the lens centre (about 20 average) | [UploadVR](https://uploadvr.com/quest-3-specs) |
| Comfortable text | 1.2 to 1.3 degrees a character's height (0.8 to 0.9 minimum; 1.375 Google; 1.8 Dingler) | Solum 2019 et al., via [Frontiers](https://frontiersin.org/journals/virtual-reality/articles/10.3389/frvir.2023.1243387/full) |
| Text people chose for 20 to 50 words | 0.9 to 1.0 degrees | Kojić et al. 2020, [arXiv](https://arxiv.org/pdf/2004.01545) |
| Near content | at least 0.5 m; fade at 40 cm, clip at 30; a quarter of the time at most on content nearer than 1 m | [Meta](https://developers.meta.com/horizon/design/display/), [Microsoft](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort) |
| Page angle | turned up to about 55 degrees costs little reading time; pitch hurts more than yaw | [Larson et al.](https://www.dgp.toronto.edu/public_user/ravin/papers/chi2007_volumetrictextorientation.pdf) |
| Contrast | 7 to 12 : 1 chosen; the brightest white close to the eye glares | Kojić et al. |
| Lines | about 40 characters by 7 lines preferred for a text box | Dingler 2018, via the [Brown VR wiki](https://www.vrwiki.cs.brown.edu/applications-of-vr/reading-in-vr/text-colors-and-fonts) |

So at the 45 cm a book is held at, a line of the game's book text (32 of its 960 units) at about
1.4 degrees is 11 mm: the game's book art (1152 x 856 units) is then about 38 x 28 cm, a big open
book, and its 18 lines a page are 250 pixels of the eye's picture [unverified arithmetic]. Oblivion's
parchment with its dark brown ink is about the contrast asked for.

## 3. Oblivion's books and map

- **A book** has its model, its text, perhaps an enchantment and perhaps a skill it teaches. Its
  models are closed only (`meshes\clutter\books\octavo01..04`, `quarto`, `folio`, `note01`,
  `scroll01..08`, ...): no open book models. A **note** or a **scroll** is one long page on the
  scroll art. A **magic scroll** (a scroll with an enchantment) is taken when activated in the
  world, not read, and cast when used from the inventory. There are no spell tomes in Oblivion;
  spells come from people.
- **Reading** is activating it: in combat the game says so and does nothing; otherwise it opens
  the book menu with a book's or a scroll's sound. A skill book with its skill under 100 raises it
  by one, once, with the game's message. So the skill rise, the stats and a note's script all come
  from activating it: the adapter reads a held book with the game's own
  `<ref>.Activate player 1`, exactly as activating it would.
- **The book menu** (`menus\book_menu.xml`): a book is `book_background`
  (`Menus\Book\book_background.dds`, 1152 x 856, centred on the screen), an open book's two pages
  of the same text, cut into pages by the game's own HTML layout, with the page numbers and the
  buttons Take (when it can be taken), Prev, Next and Exit. A scroll is `book_background_scroll`
  (`scroll_background.dds`, 990 x 740) with its roller, one page of text that scrolls, Take and
  Exit. The mouse wheel over a book turns two pages with the page's sound; over a scroll it
  scrolls. Take picks the book up (with its stealing) and closes the menu.
- **The map menu** (`menus\main\map_menu.xml`): five pages by tabs: the local map, the world map,
  and three of the journal (the active quest, the current ones, the completed ones), each on the
  parchment. The world map is the world space's map art with its markers (discovered places,
  quest markers, the player's arrow) in a frame; the local map is the cell's own map with its
  doors and the player. Dragging pans either map. **There's no zoom**: the wheel scrolls the
  journal. Clicking a discovered place asks "travel there?", with the game's own rules (not from
  an interior, not with enemies near, not while in combat...).

## 4. The design

### Framework (general)

- **A menu on a held thing** (`ANCHOR_HELD`, `crates/protocol/src/hud.rs`): a group whose pieces lie
  on a thing in the hands instead of in the world. Its `HudHeld` says what: an object of the table
  (`ObjectDesc::id`; the runtime has its pose every frame) or a hand (a sheet with nothing under
  it), and how:
  - `HELD_FACE`: on the thing's broad face, the one turned to the eyes when it opens (a book's
    cover), its middle there, upright along the face's longer side, a few millimetres out from it.
  - `HELD_SHEET`: held like paper: the hand at its middle (first at the middle of its near edge,
    which stood it off to one side of the hand), turned to the eyes; when the other hand takes it
    near either side it spreads between the two hands, the points each took staying in them
    (pulling them apart enlarges it).
  - Its size: so a line of the game's text is `menu_text_deg` at a reading distance
    (`held_distance`, 0.45 m by default), kept as it moves (a book doesn't change size).
  It follows the hand every frame (the rig's pose of the thing, or the hand's drawn pose), turned
  only as the thing turns.
- **Drawn with the body** (`crates/avatar`): pieces on held things are drawn in the body's depth,
  after what the hands hold: a hand in front of a page covers it, the fingers behind a sheet are
  hidden by it, the book's model under the pages is covered by them, and the game's world in front
  of all of it still hides it.
- **Facing the eyes on a thing** (`HELD_FACING`, after the first headset play: pages lying on the
  cover turned with the wrist and were read edge on): centred on the thing, turned to the eyes from
  its middle, upright, a little smoothed, out past its box as the eyes see it.
- **Pages turned by hand** (`HELD_PAGES`): the free hand's grip near the pages holds them, and every
  5 cm it then moves across them the runtime presses `Action::PageNext` (leftwards) or
  `Action::PagePrev` once, with a tick in that hand; near them that hand's grip takes nothing else.
  The game turns its page with its own sound.
  (The first build's sweep had to start well out on a page and cross the spine within a few
  centimetres of it, and in the first headset play the buttons turned the pages instead; the
  second's flick, the hand's speed against the pages, turned them at random as the pages turned
  with the head; the third's, at 1 m/s, wanted too hard a flick; the fourth's counted anywhere,
  and the player wanted it on the pages or near them; by the eighth the player wanted it "insanely
  easy" with the finger: the fingertip's swipe, which still missed; in the ninth the player asked for
  the plainest thing: the other hand's grip near the book, dragged.)
- **Steadied pages**: in one hand, where pages are and how they're turned go through a one-euro
  filter (Casiez et al., CHI 2012), heavy while the hand's nearly still, light as it moves: in the
  third headset play the pages shook with the hand and were hard to read (Riven VR's players asked
  for the same smoothing, section 1).
- **Opening and shutting by hand** (`OBJF_OPENS`): the other hand's grip on a held book (its cover)
  counts an open (the object's state `shots`, one more each time) instead of taking it; the game
  opens what's inside it, or shuts it if it's open. Letting go of it shuts it too (the game's side).
  It's in the framework; Oblivion's books no longer use it (they open as soon as they're held).
- **Kept things** (`OBJF_KEPT`): let go anywhere, it glides back to the slot it came from (a map to
  its place on the belt), as a holstered gun does.
- **A sheet's map pan** (`HELD_DRAG`): the free hand's grip on a sheet's face drags it: the game's
  cursor goes where the hand is on the sheet, its button down, so the game drags its own map; let
  go, it's let go.
- **Pressed by the fingers**: there's no laser on a held menu (the player asked for it gone); the
  free hand's index fingertip presses its buttons (the game's cursor follows it near a piece, and
  its button is down while the fingertip's on it). The laser, from the free hand, is only for what
  floats (a question over the map).
- **The hands go on while a held menu is open**: the rig isn't paused for it, so the book can be
  moved, handed over, let go.
- **Held with no menu open**: a game may hold a group in the hands as it plays on (a book read in the
  hand): the runtime runs and draws held menus over the game's stereo picture in play too, and a
  fingertip on them reaches the game as the cursor intents (`INT_CURSOR`), which the game's adapter
  turns into the menu's own presses (its mouse being its play).

### A book isn't a menu (the eighth headset play)

The player: a book held should be "just the same as anything else except a HUD menu pops up". The
sixth and seventh rounds had kept the game's book menu a menu and let the world (and then the
player's walking) go on under it; the player found that the wrong way round. So:

- Oblivion goes into its menus when a menu goes on the interface's menu stack: the first one pushed
  takes the interface from play into menu mode, and every system that stops for menus asks that.
- The book menu, once built from `book_menu.xml`, registers itself (which pushes it on that stack)
  and fades in by the interface's own fade list (advanced by real time, not by the stack). Its
  close takes it off the stack only if it was registered as on it.
- So a book opened with no menu up (taken in the hand, or activated where it lies) isn't pushed (the
  adapter's detour of the game's menu push), and it isn't marked as on the stack: the game never
  leaves play, and the game's own book menu is laid out, drawn and faded as ever.
- In play the game's cursor and mouse are the player's play (the left button swings; the wheel zooms
  the camera, which the sixth round's page turns had also done: "a big flash in the face"). So the
  page turn is done on the menu's own values, as its wheel does: a book's page step by ±2 then 0,
  and its page sound (`ITMBookPageTurn`) if its page moved; a scroll's by ±1 (its notches) then 0.
  Take and Exit are the menu's own buttons (`book_take`, `book_exit`, a scroll's
  `book_scroll_take`, `book_scroll_exit`; `book_prev` and `book_next` have no ids of their own,
  their clicks feed the page step in the XML).
- The game draws the book over its HUD in the one picture of both, so the HUD pieces under the
  book's area are left out of the layout while it's open.

### Oblivion (`crates/oblivion`)

- **Books lying about** (`books.rs`; the hands' things are `items.rs`'s) are read as soon as a
  hand has one (the first build waited for
  the other hand's grip, `OBJF_OPENS`; the player asked for it to open by itself): the reference's
  own activation by the player, the game's own reading (its menu, the skill rise, the note's script,
  "you can't read in combat"), once while it's held. Let go, Exit, or B: the game's own close
  (its Exit); it stays shut until it's let go of and taken again. A magic scroll isn't read
  (activated, the game takes it).
- **The book menu over the held book** (`hud.rs`): two pages for a book (the art's left and right
  halves, each tipped a few degrees about the spine so the spread reads as an open book), one for a
  scroll (its parchment and roller); the buttons (Take, Exit) on the pages; `HELD_FACING`,
  `HELD_PAGES`, centred on the book's model rather than its collision box (`HELD_CENTRE`). Its
  parchment stays solid (a book isn't see-through).
- **Pages**: `PageNext` / `PagePrev` turn them on the menu's own values, as its wheel does (the
  game's own turn and sound): a book's page step by two, a scroll a notch. The fingertip on Prev
  and Next (a scroll's arrows) does the same. In play the stick doesn't turn them; a book read
  from the pack is in the inventory's menus, where the stick's scroll is the game's wheel.
- **Take** (the game's button, pressed with the free hand's finger) takes it, with its stealing.
- **A book activated where it lies** (on a shelf, by looking at it and pulling the trigger) comes
  to the off hand as the gloves would bring it (`HELD_BRING`) and opens there, on the book.
- **A book read from the inventory** has no book in a hand: its pages are held in the off hand as a
  sheet (`HELD_SHEET`, at its middle).
- **Solid pages.** Oblivion's UI pass starts on whatever blending the world's HDR left (light
  added), and its book and map pages don't set it again, so they arrive in the HUD texture with
  colour but no coverage. Held, they take their coverage from their colour (`HUDE_OPAQUE`): what
  the game drew is solid, the black it left is clear.
- **The map** is an object of the adapter's (`map.rs`): a folded map in a body slot,
  `OBJF_HOLSTERABLE | OBJF_KEPT`, drawn by nothing (the game's art is the map). Which slot is
  `map_slot` in `rustvr-oblivion.toml` (`hip_right`, `hip_left`, `belt`, `chest_left`,
  `chest_right`; first the belt pouch, too small a place to find by feel, then the right hip).
  **It's empty by default now, so the map isn't on the body**: the body's slots are the holsters'
  to hand the game's things out from, and the map isn't one of them. Without it the map is the
  game's map menu floating as the big four do. Drawn from its slot it opens the game's map menu on
  the local map in an interior, the world map outside (pressing its own tabs); its
  page, frame and tabs are a `HELD_SHEET` in that hand (`HELD_DRAG` on a map page); the other hand
  spreads it (zoom), grips its face to drag (the game's pan), and its fingertip presses the tabs
  and places (fast travel with the game's own question and rules). The sheet lies a hand's depth in
  front of the hands holding it (the fourth headset play: the fingers came through it). Put back,
  or let go, it folds away (the menu closed, nothing of it shown meanwhile). The journal's pages,
  turned to from the map in the hand, stay in the hand (after the second headset play: they
  floated ahead, which looked as if the map had dropped).
