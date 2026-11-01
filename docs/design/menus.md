# Floating menus and HUD: research and design

Every menu and every piece of the HUD floating in the world, made only of Oblivion's own UI, cut
into pieces, transparent, driven by the laser. This is the research it rests on and the design
built from it (the framework's `rustvr_protocol::hud` and the runtime's `hud.rs`; the adapter's
`hud.rs`, `tiles.rs`, `hud_info.rs`, `menus.rs` and `d3d9.rs`). The framework's earlier HUD research
(`framework/docs/design/vr-throwables-and-ui/ui.md`) covers HUD placement in general (Alyx,
Doom 3 BFG VR, UEVR, MRTK, text sizes); this note adds menus, looting, barter and dialogue, and
Oblivion's own menus.

Legend: **(snippet)** a search engine's summary, not a page read in full; **[unverified]** prior
knowledge, not confirmed by a source.

## 1. What players like and dislike

### Skyrim VR, the cautionary tale

- **The menus are the flat game's, hung in the air.** Road to VR: inventory as a floating text
  window "an ancillary task that just doesn't fit" the world
  ([review](https://roadtovr.com/skyrim-vr-review-pc-rift-vive-windows)); on PSVR "the antithesis of
  VR" ([review](https://roadtovr.com/skyrim-vr-review-playstation-vr-psvr/)). CGMagazine called the
  navigation "infuriating"
  ([review](https://www.cgmagonline.com/review/elder-scrolls-v-skyrim-vr-review-fresh-ideas-bad-controls)).
- **Menus and dialogue open inside things.** PCWorld: "a bad habit of appearing half-inside other
  objects, rendering them useless"
  ([review](https://www.pcworld.com/article/3268078/skyrim-vr-review-dragon-slaying-at-awe-inspiring-scale.html)).
  OpenMW VR lists the same ("menus keep getting stuck inside things") and gives a recentre key for it
  ([docs](https://openmw-vr.readthedocs.io/en/latest/manuals/openmw-vr/index.html)).
- **The HUD is hard to find.** The compass about 45 degrees down, the bars above it only in combat:
  "straining to find my health bar" (PSVR review above).
- **The same button means different things.** In barter the trigger takes; in a container it uses,
  and players used things by accident (snippet,
  [Nexus forum](https://forums.nexusmods.com/topic/10549028-skyrim-vr-lootingcontainersnpc-prevent-accidental-use-function)).
- **What players loved instead** were mods that take trips into menus away: VRIK holsters, HIGGS's
  physical looting ("a lot more fun than just pressing a button"), Spell Wheel VR's wheel of the
  inventory's own icons and its wrist bars
  ([Steam thread](https://steamcommunity.com/app/611670/discussions/0/3763353492968498409),
  [HIGGS](https://roadtovr.com/skyrim-vr-mod-higgsvr-hand-gravity-gloves/)). The SkyUI VR port of
  Dear Diary moves the inventory, favourites and barter menus to the side "easier to read in VR"
  ([Nexus](https://www.nexusmods.com/skyrimspecialedition/mods/44874)).

### Fallout 4 VR

- The Pip-Boy opens by looking at the wrist and "feels like just another part of the world"; looking
  away closes it ([UploadVR](https://uploadvr.com/preview-fallout-4-vr-needs-polish/)). Holding the
  arm up tires, and the projected mode exists for comfort.
- **Looting** is a small floating list at the corpse or container you point at, which closes when
  you walk or look away (same preview). The workshop menu, on one controller and pointed at with the
  other, was called its best-integrated menu ([The VR Grid](https://www.thevrgrid.com/?p=66519)).

### The native games

- **Half-Life: Alyx** has no HUD: health on the back of the left glove, ammo on the gun; the menu
  is made for the laser and works as a touchscreen too
  ([Road to VR](https://roadtovr.com/these-details-make-half-life-alyx-unlike-any-other-vr-game-inside-xr-design/)).
- **The Walking Dead: Saints & Sinners**: backpack over the shoulder, journal on the chest, a wrist
  watch for the bells ([Road to VR](https://www.roadtovr.com/cas-chary-present-walking-dead-saints-sinners-gameplay-overview/)).
- **Asgard's Wrath 1 and 2**: loot sorted by category with quick menus, which works only when the
  categories are right ([The VR Grid](https://thevrgrid.com/?p=72960)); AW2 is faulted for no
  sell-all and no proper pause menu
  ([GameCritics](https://gamecritics.com/elijah-beahm/asgards-wrath-2-review/)).
- **Lone Echo**'s design talk is the most specific about readable panels: light text on a
  medium-dark background (dark text looks thinner through the lenses, high contrast glares), text
  about 1.75 times print size, 3.5 rows shown so the cut row says "scroll", scrolling with a spring
  at the ends, and never text drawn through geometry
  ([Road to VR](https://roadtovr.com/designing-lone-echo-echo-arena-virtual-touchscreen-interfaces-robert-duncan/2/)).

### Oblivion in VR before

- **OBVR** (the 2006 game): menus either in the world in front of you or on a "cinema" screen, the
  HUD and the Tab menus on a wrist clicked with the other hand's laser, the crosshair at the
  target's distance ([GitHub](https://github.com/Naragorn/OBVR)).
- **Oblivion Remastered through UEVR**: the whole UI projected onto one sheet in front of you
  ([UEVR](https://github.com/praydog/UEVR/releases/tag/1.0)).

## 2. Looting, barter and dialogue in the big games

- **Physical looting wins when it's possible** (Alyx, Saints & Sinners, HIGGS). This adapter already
  has it for things lying in the world (`items.rs`): the gloves pull them, a body slot takes them.
  A container's contents aren't in the world, so a container needs a list; the list should be the
  container's, beside the container, not a screen.
- **Menu looting that works** is small, by the thing, and goes away when you leave: Fallout 4 VR's
  floating loot list. The Skyrim VR way (the whole flat window) is the one criticised.
- **Barter** wants the same buttons meaning the same things as looting, and bulk actions.
- **Dialogue** in Skyrim VR and Fallout 4 VR is a flat list picked with the stick: "not ideal".
  Guidance for dialogue UI: a short vertical list of choices, never over the speaker's face, the
  speaker's line near the speaker ([dialogue UI](https://www.abratabia.com/game-ui-design/dialogue-ui.php));
  a study of four VR dialogue interfaces found naturalness drove preference more than speed
  ([Conveyor](https://hci.cs.umanitoba.ca/Publications/details/conveyor-a-dual-task-paradigm-for-studying-vr-dialogue-interfaces)).

## 3. Readable and comfortable floating UI, in numbers

| What | Number | Source |
|---|---|---|
| Distance for laser-driven panels | 0.8 to 3 m; avoid 0.5 to 0.8 m (people try to touch it) | [Meta](https://developers.meta.com/vr/design/hands-ui-best-practices/) |
| Distance for reading | about 1 m comfortable, 0.75 m minimum; headsets focus at 1.25 to 2.5 m | Oculus guide via [Dummies](https://www.dummies.com/article/virtual-reality-design-principles-starting-up-user-attention-and-comfort-zones-256440); [Microsoft](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort) |
| Where to put it | resting gaze 10 to 20 degrees below the horizon; content 0 to 35 degrees below; neck turns under 45 degrees | [Microsoft](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort) |
| Text size | comfortable 1.2 to 1.4 degrees a line on today's headsets (0.75 to 0.9 minimum) | framework `ui.md` §5.2 |
| Hit targets | at least 2.5 to 3 degrees, at a constant angular size | [Meta](https://developers.meta.com/vr/design/hands-ui-best-practices/) |
| Backdrops | 50 % opacity behind text is enough for most; a busy world behind matters more than opacity | [Brown VR wiki](https://www.vrwiki.cs.brown.edu/applications-of-vr/reading-in-vr/reading-in-ar) |
| Following | world-lock menus once open; never head-lock; re-anchor on a big turn | [Microsoft](https://learn.microsoft.com/en-us/windows/mixed-reality/design/comfort), MRTK |
| Facing | turn panels to face the eye, pitch and yaw only, never roll | Meta, MRTK |
| Grouping | one coherent cluster per context, not scattered windows | [Meta](https://developers.meta.com/horizon/design/comfort/) |

## 4. Oblivion's menus

- **Menus are tile trees** read from `menus\*.xml` (in `Oblivion - Misc.bsa`), each tile with its
  values (`visible`, `locus`, `alpha`, `x`, `y`, `width`, `height`, `user0..`). A tile's `x` and
  `y` are from its nearest ancestor with `locus`; the menu root's `width` and `height` are the
  screen's, in menu units (960 high, or 1280 wide when the screen is taller than wide).
- **The frame draws the menus after the world**, then presents. So the UI can be drawn on its own,
  into a target of the adapter's (see Alpha below).
- **The menus draw a 3D scene of their own** before their tiles. The inventory's layout
  (`menus\main\inventory_menu.xml`) has no 3D in it, yet under the big four that scene holds a
  figure of the player, about 88 units across (logged in playtest 2; nothing under the pause
  menu): the flat game shows it beside the inventory. In VR it's culled while the UI is drawn
  under the big four (`menus::hide_menu_3d`), or it shows through the floating pages as a dark
  silhouette.
- **Behind most menus the world is a frozen picture.** On opening a menu (except the race menu,
  the main menu, the generic and message menus and dialogue, which keep the world live), and for
  sleep and wait, the game renders the world once into a texture and from then on draws that
  instead of the world. A cell's load takes the same picture as it starts (the picture behind the
  loading screen's fade), and so does a question box over the load's fade. In VR the world has to
  stay live (the head moves), so that picture isn't taken, and the game renders the world every
  frame.
- **The back buffer has no alpha**: the game asks for `D3DFMT_X8R8G8B8`. The UI is drawn with
  alpha blending over it.
- **Container layout** (`container_menu.xml`): one parchment box (`cont_background`, 840 x 854) with
  the list (`cont_window`, its headers `cont_headers_top`, its scroll bar `cont_scroll_bar`), the
  category tabs (`cont_tab_layout`), the money and weight (`cont_player_money_icon`,
  `cont_encumberance_icon`, the merchant's `cont_npc_money_icon`), the buttons (`cont_button_take_all`,
  `cont_button_haggle`, `cont_button_exit`) and the switch between the container's things and the
  player's (`cont_select_frame`). Barter is the same menu.
- **Dialogue** (`dialog\dialog_menu.xml`): the speaker's name (`dialog_name`), their line
  (`dialog_text_layout`), the topics (`dialog_topics`, its scroll pane and scroll bar) and the
  service buttons (`dialog_button_layout`: persuade, barter, spells, repair, training, recharge,
  goodbye).
- **HUD** (`main\hud_main_menu.xml`, `hud_reticle.xml`, `hud_info_menu.xml`,
  `hud_subtitle_menu.xml`): the bars (`hudmain_statusbars`), the weapon and spell icons, the compass
  (`hudmain_compass_layout`), the active effects (`magic_icons`), the crosshair, its enemy health
  (`hudreticle_enemy_health`) and the sneak eye (`hudreticle_sneak`), the card of what the crosshair
  is on (`HUDInfoMenu`), the subtitles (`hudsubtitle_text_layout`).

## 5. The design, as built

The flat screen's menus are never shown whole. Each menu is described by the adapter as its
pieces (rectangles of the game's own UI, from its tiles' places), and the runtime floats each piece
where it belongs in the world, transparent where the game's art is, with soft edges, the game still
drawing every pixel and running every click.

### Framework (`crates/protocol/src/hud.rs`, `crates/runtime/src/hud.rs`)

The HUD layout grew into this rather than a second system:

- **Groups.** A layout has up to eight groups (48 elements in all). A group is one menu's cluster
  of pieces, anchored:
  - `ANCHOR_WORLD`: beside a thing in the world (a container, a merchant, the speaker's head): its
    point and size, and which side (`SIDE_RIGHT`, `SIDE_LEFT`, `SIDE_BELOW`, `SIDE_FRONT`,
    `SIDE_ABOVE`). Placed at the thing's distance (clamped to 0.9 to 1.4 m, the laser's comfortable
    range), 10 degrees under the eyes, beside its silhouette wherever it would cover it (never more
    than 40 degrees round), facing them; below or above it for `SIDE_BELOW` and `SIDE_ABOVE`.
    World-locked once placed; placed again when its thing moves 30 cm or the head walks 80 cm from
    where it was placed, gliding over 0.3 s.
  - `ANCHOR_PLAYER`: the player's own menus (inventory, magic, map, journal, pause): 1.1 m ahead of
    where the player faces, 8 degrees under the eyes, world-locked, coming round after a turn of
    65 degrees (or a walk of 80 cm).
  - `ANCHOR_POPUP`: questions and counts (how many, are you sure): 0.8 m ahead of where the eyes
    look, nearer than the menus behind them, world-locked.
  - `ANCHOR_HELD`: on a thing in the hands (a book's pages, a map held like paper), placed with the
    hands by the runtime's `held_menus.rs`; no laser on it, the free hand's fingertip presses it.
  - `ANCHOR_VISTA`: over a view, as a title screen in the world (Oblivion's main menu,
    `worldmenu.rs`): 5 m out towards the view's subject, 6 degrees clear of it on its side, 6
    degrees under the horizon, fixed in the game's world.
- **Pieces.** An element in a group is a piece: its rectangle of the HUD texture, its place in the
  group (offset from the group's centre in the group's units, the HUD texture's height, plus how
  far it's lifted towards the eye), whether the laser points at it (`HUDE_POINT`), whether its
  edges fade (`HUDE_SOFT`), whether it's a page drawn solid wherever the game drew half or more
  (`HUDE_SOLID`), and whether its coverage is its colour's (`HUDE_OPAQUE`: art the game adds as
  light, such as the book and map pages). Pieces lie on a cylinder round the eye (each faces it),
  so a wide group keeps its text at one distance.
- **Size from the game's text.** The adapter says how tall its text is (`text_height`, a fraction
  of the texture's height); the runtime sizes each group so a line of it is `menu_text_deg`
  degrees (1.5 by default: Oblivion's serif face is harder to read than the sans-serif the
  1.2 to 1.4 figures were measured with) at the group's distance: the same size near or far.
- **Transparency.** The HUD texture's alpha is real when the layout says so (`HUDL_ALPHA`,
  premultiplied): the game's own art is as opaque as it drew it, everything else is clear. A
  piece's edges fade over a few texels (`HUDE_SOFT`) so a cut through the game's parchment reads as
  a torn edge, not a hard rectangle, and the groups can be a little transparent overall
  (`menu_opacity`, 1 by default).
- **Pointing.** The laser picks the nearest piece it hits that's `HUDE_POINT`; the point it hits
  in the piece is the point in the HUD texture (`Intents::cursor_uv`), and the adapter puts the
  game's own cursor there. The game does its own hover, selection, scrolling and clicks. The
  dominant stick scrolls (`Intents::scroll`, the game's mouse wheel). The trigger clicks; held, it
  drags (scroll bars).
- **Drawn in the world.** Menu pieces are drawn before the body, so the hands (closer than any
  piece) are never covered by one; the laser is drawn last.
- **The world stays.** With a game picture under a menu (the adapter keeps rendering the world in
  menus), the runtime draws the world, the body and the hands, and the pieces, never the void. The
  runtime's own settings sit beside the game's menu only when the layout asks (`HUDL_SETTINGS`:
  the pause menu) or outside the world (the main menu, loading).
- **A menu nobody described** still floats (one world-locked piece, its real alpha, over the live
  world) rather than going back to the void, after half a second without pieces.

### Oblivion (`crates/oblivion`)

- **The world live in menus**: the game's frozen picture of the world (on opening a menu, sleep and
  wait, a cell's load, the question box over a load's fade) isn't taken while in VR, so the game
  renders the world in stereo every frame it can; while it can't (a load), the runtime holds the
  last picture of it (`render.rs`).
- **Alpha**: the back buffer can't be given alpha (the game's renderer looks its pixel formats up
  by the back buffer's: `A8R8G8B8` crashed it). Instead, for the UI (after the eyes,
  until Present) the game's back buffer is swapped for the HUD copy's `A8R8G8B8` target wherever it
  sets it (`SetRenderTarget` hooked), cleared to nothing, and alpha is blended separately from
  colour so it builds up as coverage (`ONE, INVSRCALPHA` for the game's ordinary blending, nothing
  for its additive): the HUD texture is the UI premultiplied over clear, exactly as the game drew it
  (`d3d9.rs`).
- **Solid pages, see-through backgrounds as a setting**: a menu's pieces are `HUDE_SOLID`, so the
  parchment is never seen through. Each menu's background images (named `..._background`) can be
  made see-through with `menu_background` in `rustvr-oblivion.toml` (1 by default: solid), set as
  the game's own `alpha` of each, through the game's own tile update, whenever the game sets it
  afresh; the text and icons on them stay solid.
- **Pop-ups made room for**: a count or a question is drawn over the middle of the menu it's for,
  so cut out it would show twice; while it's up, the menu under it is moved to the screen's foot
  and the pop-up into the room above (their top tiles' `y`, through the same update), and put back
  when it goes.
- **B on a menu**: on a menu in the world (a container, dialogue, the big four) it presses Menu
  Mode, which closes it (Escape there would open the pause menu over it); on the pause menu and its
  own (options, load, save) it presses Escape; an open book shuts itself; the holsters' editor
  closes itself (`input.rs`).
- **No figure behind the big four**: the interface's own 3D scene (Menu3DRoot) is culled while the
  UI is drawn under them (`menus::hide_menu_3d`).
- **Pieces from the game's own tiles** (`tiles.rs`, generalised from `hud_info.rs`): each menu is a
  list of pieces named by its tiles in its XML (the list, the tabs, the buttons, the money and
  weight...), each piece's rectangle the union of what those tiles show now (so it follows the
  game: a hidden button leaves its piece). The pieces keep the game's arrangement, pulled apart a
  little so each floats on its own. Nothing is drawn by us.
- **Anchors from the game's own objects**: the container's or merchant's 3D bound, the speaker's
  head bone, the player.
- **Which menu**: the top menu, and the HUD's pieces in the world when there's no menu.

### Per menu (`hud.rs`)

| Menu | Anchor | Pieces |
|---|---|---|
| Container (loot) | the container, beside it | three strips of its box: the list (headers, scroll bar, gold, weight), the tabs, the buttons |
| Barter | the merchant, beside them | as the container, with the merchant's gold and haggle |
| Dialogue | the speaker | their name over their head; their line under their face; the topics and service buttons beside them |
| Inventory, magic, map, character | the player | the page or list, its tabs, the HUD's switcher between them (bars: character, weapon: inventory, spell: magic, compass: map), a description over the page |
| Quantity, messages, haggling | pop-up, where the eyes look; the menu under it stays where it was | the box itself |
| A service from dialogue (spells, training, repair, recharge, persuasion) | beside the speaker | the menu, cut from the screen |
| A thing's own (a lock, an altar's enchanting or spellmaking, a sigil stone) | beside the thing | the menu, cut from the screen |
| The main menu (`worldmenu.rs`) | over the view, beside its subject | the logo, then each word in a column under it |
| The loading screen | ahead of the player | its bar alone, with its word |
| The race menu | beside the mirror (`GS_MENU_IN_WORLD`) | none: the runtime shows the part of the picture it's in |
| A book or note being read, the map drawn from the belt | held (`ANCHOR_HELD`) | a book's pages, a note's scroll; the map's page and tabs |
| The holsters' editor (`slot_menu.rs`) | the player | three strips (slots, categories, buttons) and its title |
| Anything else | the player | the menu, cut from the screen |
| HUD bars, weapon, spell | over the inside of the off-hand wrist, only while its palm is turned up | the status bars with the weapon and spell icons |
| Compass, then the active effects | the off-hand wrist, stacked over the bars | the compass strip; the effect icons |
| Crosshair info, sneak eye | where the head looks (`HUD_GAZE`), eased after it, just in front of the thing looked at (lower for a person) | the reticle's pieces and the card of what it's on |
| Enemy health | over the enemy (`HUD_TARGET`) | the bar |
| Subtitles; notices and the region's name | the caption strip; over it | the subtitle text; the notices |
| Breath meter | where the game has it, at the top of the view | `breath_menu.xml`'s meter |

## 6. From the first real-headset playtest

- **The flash on opening a menu** had two causes, both in the hand-over between the adapter and the
  runtime:
  - `GS_MENU` is written as the frame starts (`link::game_frame`, in `prepare`). The layout of the
    menu's pieces is written only at that frame's Present. In between, the runtime saw a menu with
    no groups and showed the whole flat screen.
  - The layout and the picture were separate. The runtime read the layout early in its frame and
    took the picture late, so for a frame one could be a frame older than the other.
  - Fixes: each picture says the layout it was drawn with (`SlotInfo::hud_layout`). A menu over the
    world with no groups yet shows nothing for half a second before the flat screen stands in.
- **After a menu closes**, the HUD's tiles fade back in from alpha 0, so for about a quarter of a
  second the layout has no pieces (the log's "0 pieces in 0 groups"). An empty layout that hides
  what it doesn't cover (`HUDL_REST_HIDDEN`) now shows nothing, not the flat screen.
- **The laser's colours** are the game's own. Its menu cursor (`textures\menus\misc\cursor.dds`,
  64 x 64 A8R8G8B8) is a gold arrow:
  - gold (255, 203, 57) and (255, 207, 66);
  - a cream highlight (255, 247, 222);
  - a dark gold shade (173, 138, 44);
  - a black outline.
  The HUD's text over the world (`hud_main_menu.xml`: the region's name, the ammo count) is a cream
  (254, 244, 183); the text on the parchment is a dark brown (117, 59, 33). So the beam is a cream
  thread in a gold glow. Its dot is gold with a dark edge and a cream heart, like the cursor.
