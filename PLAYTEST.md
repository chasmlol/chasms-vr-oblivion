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
   - Glance down: the compass sits low in front of you and turns with your body.
   - Raise your left hand and look at the back of your wrist: the health, magicka and fatigue bars
     and your weapon and spell icons are there, like a watch. Any active effects stack above them.
   - Look at Jensine: the game's card of what you're looking at (her name) floats just under the
     point you're looking at. It's the same card that says "Open Door" or gives an item's numbers.
   - The flat game's crosshair is gone. Where the head looks, only the game's icon of what can be
     activated shows (a hand, a door; red when taking it would be stealing).
2. **The chest (looting).** Look at the chest and pull the trigger with an empty right hand.
   - Its list floats above the chest, at about the chest's distance and a little under your eye
     line.
   - It comes as three strips: the list (with its column heads, scroll bar, your gold and weight
     under it), the category tabs, and the buttons (Take All, the switch to your own things, Exit).
   - The parchment is a little see-through; the text on it isn't.
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
8. **A thing's own menu** floats beside the thing: picking a lock beside the locked door or chest,
   an altar's enchanting or spellmaking beside the altar, a sigil stone beside it.
9. **Everything else** floats ahead of you as one piece, cut out of the flat screen: sleep and
   wait, books, the level-up, alchemy, repair with a hammer and the race menu.

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
- **See-through parchment.** The menus' background images (the game's own `..._background`
  tiles) are drawn at 80% of their alpha, through the game's own tile update, so the text and icons
  stay solid. Pop-ups stay solid. This is `menu_background` in `rustvr-oblivion.toml`.
- **Size from the game's text.** Every piece is sized so a line of the game's text is 1.5 degrees
  tall wherever it floats. The research figure is 1.2 to 1.4; Oblivion's serif font reads harder,
  hence 1.5.
- **Runtime settings** live in `%LOCALAPPDATA%\RustVR\settings.toml` under `[ui]`:
  - `menu_text_deg` (1.5);
  - `menu_opacity` (0.94, the whole piece);
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
- **The crosshair card** under where you look is fixed to your gaze, because it says what looking
  there does.
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
