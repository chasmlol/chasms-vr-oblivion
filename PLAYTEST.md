# Playtest: floating menus and HUD

Save: rustvr-menus

`playtest.bat` in this worktree (not committed: double-click it in Explorer) puts away any game left
running, then `tools\play.ps1 menus` boots this build into **rustvr-menus**: Jensine's Good as New
Merchandise in the Imperial City Market District at noon. A test chest full of everything is on the
floor in front of you, and Jensine is at her counter to your right. A key in the .bat's window closes
the game (`tools\play.ps1 -Close`) and keeps the runtime's log in `<builds>\menus\playtest-<n>`.

Every menu now floats in the world as pieces of Oblivion's own UI (its parchment, fonts, icons and
buttons). The world is still drawn around and behind it, your hands too, and the game still runs
its own selection, scrolling and clicks. Point with the laser in your right hand and pull the
trigger to click. The right stick scrolls, and B closes.

## What to try, in order

1. **Look around (the HUD).**
   - Turn your left palm up and look at it: the health, magicka and fatigue bars and your weapon
     and spell icons float over the inside of your wrist, with the compass above them and any
     active effects above the compass. Turn the palm back to the side and they fade away.
   - Look at Jensine: the game's card of what you're looking at (her name) floats just under the
     point you're looking at, just in front of her. It eases after your head instead of being
     fixed to it. It's the same card that says "Open Door".
   - Look at a thing you can take (in the chest's room, anything on a shelf): nothing comes up, no
     card and no hand icon. Pick it up and its card (name, value, weight) floats over it in your
     hand.
   - The flat game's crosshair is gone. Where the head looks, only the game's icon of what can be
     activated shows (a hand, a door; red when taking it would be stealing).
2. **The chest (looting).** Look at the chest and pull the trigger with an empty right hand.
   - Its list floats above the chest, at about the chest's distance and a little under your eye
     line.
   - It comes as three strips: the list (with its column heads, scroll bar, your gold and weight
     under it), the category tabs, and the buttons (Take All, the switch to your own things, Exit).
   - The parchment is solid, cut out round its edges.
   - Point at a row and the game highlights it, with a light tick in your hand each time the
     highlight moves to another row, tab or button. Pull the trigger to take it.
   - Push the right stick to scroll. Dragging the scroll bar with the trigger held should work too.
   - Click a tab, Take All, or the switch icon to put your own things in.
   - B closes the chest, as Exit does.
3. **Counts and questions.**
   - Click the switch icon to see your own things, then click the arrows: the game's "How Many?"
     pops up where you're looking, nearer than the list. The list stays behind it.
   - Any yes/no question pops up the same way.
4. **Your own menus (the big four).** Press B on your left hand.
   - The page floats about a metre ahead, 8 degrees under your eyes, with its tabs under it.
   - Under the tabs is the HUD's own switcher, which works as in the flat game:
     - the bars open the character page;
     - the weapon opens the inventory;
     - the spell opens magic;
     - the compass opens the map and journal.
   - Point at something on the character page: its description pops up over the page.
   - It all stays where it was opened. Turn more than about 65 degrees and it glides round to
     where you face.
   - B closes it.
5. **Talking (dialogue).** Look at Jensine and pull the trigger.
   - Her name floats over her head, and while she speaks her line floats under her face.
   - Then her topics float beside her head on the right, with the service icons under them
     (persuade, barter, goodbye).
6. **Bartering.** In the dialogue, click the barter icon (the second one).
   - Her wares float beside her, in the same three strips as the chest, now with her name and gold
     and the Haggle button.
   - Haggle pops up over them like a count does.
   - B or Exit goes back.
   - Other services from dialogue (spells, training, repair, recharge, persuasion) float beside
     the person offering them.
7. **Pause.** Press the menu button. The game's pause menu floats ahead, with RustVR's own
   settings beside it on the right.
   - The laser is in the colours of the game's own cursor: a cream thread in a gold glow, with
     light running along it, and a gold dot with a dark edge where it points. The dot presses in
     while the trigger is held.
8. **A thing's own menu** floats beside the thing: picking a lock beside the locked door or chest,
   an altar's enchanting or spellmaking beside the altar, a sigil stone beside it.
9. **Everything else** floats ahead of you as one piece, cut out of the flat screen: sleep and
   wait, books, the level-up, alchemy, repair with a hammer and the race menu.

## Playtest 1 (30 October, real headset)

What you said, and what changed:

- **"Put the compass on my hand with the health and stuff, a good size."** The compass is now on
  the left wrist, just up the forearm from the bars, weapon and spell. It's drawn 1.8 times the
  game's size there (the bars are 1.3), so with the wrist raised about 40 cm from the eyes its
  heading letters look as big as when it floated in front.
- **"Remove the transparency from all the menus. I like how they're cut out."** The parchment is
  drawn as the game draws it (`menu_background` is now 1), and pieces are no longer see-through as
  a whole (the runtime's `menu_opacity` is now 1). The cut edges still fade over a few texels.
- **"A flash of the flat-screen version for a millisecond when opening things."** Two causes, both
  fixed in the runtime:
  - The game says it's in a menu as the frame that opens it starts. What the menu's pieces are
    comes only when that frame is drawn. In between, the runtime showed the whole flat screen.
    Now a menu over the world whose pieces haven't come shows nothing for up to half a second.
  - The layout of the pieces and the picture they're cut from came separately, so for a frame the
    menu's pieces could be cut from the picture before it (or the HUD's from the menu). Each
    picture now carries the number of the layout it was drawn with, and the runtime waits for the
    picture before using a newer layout.
  - Also, for a quarter of a second after a menu closes, the HUD's layout is empty while it fades
    back in (log: "0 pieces in 0 groups"). The runtime showed the whole HUD flat then; now nothing.
- **"The names of the NPCs closer to them, stabilised and smoothed, at the same viewing angle."**
  I took this as the card that comes up when you look at someone (their name), not the name over
  a speaker's head in dialogue, which is unchanged. The card (and the activate icon) now:
  - eases after your head (0.18 s) instead of being fixed to it, so it doesn't shake;
  - floats just in front of what it's about (a person's head, a thing's bound), between 0.6 and
    3 m, instead of always 1.6 m from your face;
  - stays the same size to the eye at any distance.
- **"An Oblivion-themed laser."** The colours are the game's own cursor's
  (`textures\menus\misc\cursor.dds`, a gold arrow with a cream highlight and a dark outline):
  - a cream thread (255, 247, 222) in a soft gold glow (255, 203, 57);
  - motes of light running out along it;
  - fainter, fading out along its length, when it points at nothing;
  - where it hits, the gold light it casts and a dot like the cursor (a dark edge, gold, a cream
    heart), pressed in while the trigger is held.
  - The game's HUD text over the world is nearly the same cream (254, 244, 183).
  - The adapter gives the colours (`HudLayout::pointer`), so another game keeps the runtime's blue.
  - The main menu and loading screens get them too.
- **The flat game's crosshair** was still showing in this branch's HUD. It's gone now, the same way
  as on the riding branch.

Then: **"I don't like the HUD popup for every single item I look at. It should only say it when I
pick it up."** The card no longer comes up for a thing that can be taken (the same test the gloves
use for what they can pull). It still shows over the thing once a hand has it. I kept it for what
can't be picked up, since there's nothing to hold: a person's name, "Open Door", a container. The
hand icon stays for things, because it turns red when taking one would be stealing.

Then: **"The menus are still see-through."** Your run at 13:41 had the new build (the log says
`menu_background: 1.0`, and the runtime's `menu_opacity` is 1 with nothing in your settings to change
it), so turning those down wasn't all of it. What I checked:
- The game's own art is solid. The parchment of every menu (the chest's `cont_box_background_1`, the
  big four's and the dialogue's `shared_main_background_1`) is opaque inside its torn edge, and its
  tiles' `alpha` is 255.
- So it's in how the UI's alpha reaches the runtime. The runtime draws a piece exactly as opaque as
  that alpha.
- I couldn't see your run's alpha (the game can't be run here, and the runtime's log was replaced by
  the next game's), so the fix is in two places, and a third tells us which one did it:
  - **The UI pass keeps its alpha:** while the game draws its UI into our copy, alpha is always
    written and blended as coverage, even if the game masks alpha writes or switches the separate
    alpha blending off between draws. Its back buffer has no alpha, so it never had to keep that
    right.
  - **Menus are pages (`HUDE_SOLID`):** the runtime draws a menu's pieces opaque wherever the game
    drew half or more, in their own colour. Torn edges and letters' edges stay soft. This makes the
    parchment solid even if something else in the game's blending thins it.
  - **The log says:** half a second after a menu opens, the adapter's log gets "hud: piece N
    (group G) is covered X at its middle". Over parchment that should be 1.00. If it isn't, the
    runtime's solid drawing is what's holding it, and I'll find the game's blend that thins it.

Then: **"The steal red hand icon still pops up over items."** A thing you can take now shows
nothing while you only look at it, the hand icon included (the same test as the card). A person, a
door or a container still has its icon and card. The red hand was the game's only warning before a
theft. While a hand has a thing, the game's card is aimed at it (that's how the card shows over your
hand), and the icon comes back under your gaze, as the game sets it for its target. Whether it turns
red there for someone else's thing hasn't been seen: worth a look on a shelf in Jensine's shop.

## Playtest 2 (30 October, real headset)

- **"Make the left-hand HUD things only show when I twist my wrist up, palm facing almost
  upwards."** The wrist pieces (the bars with the weapon and spell, the compass, the active
  effects) now show only while the off hand's palm is turned up and you look at it. They're fully
  there within about 40 degrees of straight up and gone by about 57, fading between, so a hand
  passing in front of the eyes doesn't bring them up. They float over the inside of the wrist and
  stack upwards as you see them (bars nearest the wrist), instead of along the back of the forearm,
  which faces down once the palm is up. The palm is read from the controller's grip pose (OpenXR's
  grip: its +X is out of a left hand's palm), so a left-handed player gets it on the right wrist.
- **"When I opened my inventory I can see an axe in the back, bleeding in behind the menu."** Then:
  "a dark silhouette of my character behind it, the exact model that pops up in the flat game's
  inventory." Found and fixed. The game's menus draw a 3D scene of their own before the pages
  (`Menu3DRoot`). Under the big four it holds a figure of the player, about 88 game units across in
  your 16:41 run's log (nothing there under the pause menu). It's drawn with the menus' own camera
  into the UI, so in VR it landed in the HUD texture and showed through the floating pages. That
  scene is now culled while the UI is drawn under the big four, and only there: the race menu's
  face, at the start of a new game, is drawn as the game draws it. Not seen in the headset yet.
  What I ruled out on the way: the inventory's own layout has no 3D, the game's first-person arms
  are hidden while each eye is drawn, the axe in your hand is drawn over the menu, and your
  character's body in the world stays hidden (the log says so).
- **From the 15:50 run's log (see-through):** the inventory's pieces came out solid (covered 1.00
  at their middles). The dialogue's didn't: its two pieces were 0.22 and 0.57 at their middles the
  first time, 0.77 and 0.95 the second, and 0.51 and 0.00 the third. The runtime makes anything
  over half solid, so the dialogue's pieces can still show the world through them.

## What was checked in the game (emulated headset, captures)

- **The chest.**
  - Its list floats above it, with the world live behind it and the parchment see-through.
  - The laser hovers rows with the game's own highlight.
  - The right stick scrolls the list (the game's own wheel).
  - The switch icon showed the player's own things.
  - B closes it.
  - Clicking the arrows brought up "How Many?", which floated in front of the list with no copy
    of it left in the list.
- **The big four.**
  - The character page, its tabs and the HUD switcher float ahead.
  - The switcher's weapon icon opened the inventory, and its compass opened the map.
  - Pointing at the birthsign brought up its description over the page.
- **Dialogue** with a copy of Jensine.
  - Her name was over her head, her greeting under her face, and the topics and service icons
    beside her.
  - The barter icon opened her wares beside her. The copy has no stock, so the list was empty;
    the real Jensine in the save has stock.
- **A message box** (the console's `messagebox`) popped up ahead, and the laser hovered "Yes".
- **The hover tick**: 20 haptic pulses in a run of pointing at and scrolling the chest's list.
- **The pause menu**, with the RustVR settings beside it.
- **The HUD outside menus**: the compass low in front, the bars and icons on the left wrist, and
  the card of what's looked at under the gaze point.

## What wasn't checked

- **The real headset.** Text size (a line of the game's text is 1.5 degrees), distances, the feel
  of the laser, and comfort over a long session.
- **Never seen:**
  - sleep and wait;
  - the magic page and its spell descriptions;
  - haggling;
  - a foe's health bar over the foe;
  - lockpicking, the level-up and the race menu;
  - spellmaking, enchanting and alchemy;
  - books;
  - a service menu beside a speaker;
  - a lock's, an altar's or a sigil stone's menu beside it (the console can't name the test's
    locked chest, so no lock was picked);
  - the real Jensine's stock;
  - a container that's tall enough for its list to go to its side (the side placement was only
    seen with a merchant).
- **A crash once while loading a save** (in the game's own load, before any menu): it also
  happened on 28 October before this work, so it isn't from it, but it's there.
- **Performance** in a big exterior with a menu open. The world is now rendered every frame under
  menus, where it used to be a frozen picture.

## Decisions

- **The world stays live under menus.** On opening a menu, Oblivion renders a frozen picture of the
  world to draw behind it. In VR your head keeps moving, so that picture is skipped and the world
  is rendered for both eyes every frame. The game is still paused.
- **Real transparency.** The game draws its UI into an alpha render target of ours instead of its
  back buffer, so each piece is exactly as opaque as the game's own art and the rest is clear. Each
  piece's cut edges fade over a few texels.
- **Solid parchment** (playtest 1: it was 80% see-through). `menu_background` in
  `rustvr-oblivion.toml` can still make the menus' background images (the game's own
  `..._background` tiles) see-through, through the game's own tile update, with the text and icons
  on them solid. It's 1 by default.
- **Size from the game's text.** Every piece is sized so a line of the game's text is 1.5 degrees
  tall wherever it floats. The research figure is 1.2 to 1.4; Oblivion's serif font reads harder,
  hence 1.5.
- **Runtime settings** live in `%LOCALAPPDATA%\RustVR\settings.toml` under `[ui]`:
  - `menu_text_deg` (1.5);
  - `menu_opacity` (1, the whole piece; it was 0.94 before playtest 1);
  - `menu_scroll` (wheel notches a second at full stick).
- **Where things go.** Everything stays put in the world once placed.

  | Menu | Where |
  |---|---|
  | a container's or merchant's | at the thing's distance (0.9 to 1.4 m), 10 degrees under the eyes; to the thing's side only if it would cover it (a chest on the floor never does) |
  | your own | 1.1 m ahead |
  | pop-ups | 0.8 m ahead, where you're looking |
- **Pop-ups are moved apart on the screen.** In the flat game, a count or a question sits over the
  middle of the menu it's for, so cut out, it would show twice. While it's up, the menu under it is
  moved down the game's screen and the pop-up into the room above, through the game's own tile
  update. Both still float where they belong in the world.
- **B closes menus with the game's Menu Mode (Tab).** In Oblivion, Escape opens the pause menu over
  an open menu instead of closing it. The pause menu and its options, load and save pages still
  close with Escape.
- **Dialogue.** The speaker's name floats over their head (the flat game shows it in a corner).
- **The crosshair card** sits under where you look, because it says what looking there does. It
  eases after the head and floats at the distance of the thing it's about (playtest 1).
- **The flat game's crosshair is hidden** (`reticle.rs`, the same commit as the riding branch's, so
  the two merge cleanly). The hands aim in VR, and a dot fixed in the middle of the view is only in
  the way. The reticle's activate icon stays, as it's the game's only warning of a theft. The game's
  own Crosshair setting isn't used, because it hides the icons too.
- **Only the pause menu gets RustVR's settings beside it.** They used to open next to every game menu,
  a chest's list included.
- **The pointer.** The laser's dot is the pointer (the game's cursor goes where it points), and the
  game's own hover highlight shows what it's on. In the captures the game's own arrow didn't show.

## Not done

- **Most menus aren't cut into pieces.** Everything except the container, barter, dialogue, the
  big four, pop-ups and pause floats as one piece cut out of the screen. The map's journal pages are
  its page as a whole.
- **Typing** (naming a spell or a potion, `TextEditMenu`) still needs the keyboard: there's no
  keyboard in VR yet.
- **The race menu** shows the player's face in the flat game; in VR the world is live and nothing
  looks at your face (that's the mirror, overnight feature 15).
- **Loading screens and the main menu** are as before: a panel in the runtime's room, since there's
  no world to float them in.
- **Tile dumps.** To cut another menu, set `RUSTVR_OBLIVION_DUMP_TILES=1` and the adapter logs its
  tiles with their boxes. Each menu's pieces are a few lines in `crates/oblivion/src/hud.rs`.
