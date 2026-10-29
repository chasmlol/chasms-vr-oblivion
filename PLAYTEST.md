# Playtest: floating menus and HUD

Save: rustvr-menus

`tools\play.ps1 menus` boots this build into **rustvr-menus**: Jensine's Good as New Merchandise in
the Imperial City Market District at noon. A test chest full of everything is on the floor in front
of you, and Jensine is at her counter to your right. `tools\play.ps1 -Close` puts it all away.

Every menu now floats in the world as pieces of Oblivion's own UI (its parchment, fonts, icons and
buttons), with the world still drawn around and behind it, and the game still running its own
selection, scrolling and clicks. You point with the laser in your right hand and pull the trigger to
click. The right stick scrolls, and B closes.

## What to try, in order

1. **Look around (the HUD).**
   - Glance down: the compass sits low in front of you and turns with your body.
   - Raise your left hand and look at the back of your wrist: the health, magicka and fatigue bars
     and your weapon and spell icons are there, like a watch. Any active effects stack above them.
   - Look at Jensine: her name (the game's own card) floats just under the point you're looking at.
     That card shows whatever you're looking at ("Open Door", an item's numbers).
2. **The chest (looting).** Look at the chest and pull the trigger with an empty right hand.
   - Its list floats above the chest at about the chest's distance, a little under your eye line.
     It comes as three strips: the list (with its column heads, scroll bar, your gold and weight
     under it), the category tabs, and the buttons (Take All, the switch to your own things, Exit).
   - Point at a row and the game highlights it. Pull the trigger to take it.
   - Push the right stick to scroll. Dragging the scroll bar with the trigger held works too.
   - Click a tab, Take All, or the switch icon to put your own things in.
   - B closes it, as Exit does.
3. **Your own menus (the big four).** Press B on your left hand.
   - The page floats about a metre ahead, 8 degrees under your eyes, with its tabs under it.
   - Under that is the HUD's own switcher, which works the way it does in the flat game: the
     bars open the character page, the weapon the inventory, the spell magic, and the compass the
     map.
   - Everything stays where it was opened. Turn more than about 65 degrees and it glides round
     to where you face.
   - B closes it.
4. **Talking (dialogue).** Look at Jensine and pull the trigger.
   - Her name floats over her head, and while she speaks her line floats under her face.
   - Then her topics float beside her head on the right, with the service icons under them
     (persuade, barter, goodbye).
5. **Bartering.** In the dialogue, click the barter icon (the second one).
   - Her wares float beside her, in the same three strips as the chest, now with her name and gold
     and the Haggle button.
   - B or Exit goes back.
6. **Pause.** Press the menu button. The game's pause menu floats ahead, with RustVR's own
   settings beside it on the right.
7. **Questions and counts.**
   - Take a stack from the chest: the game's "how many" pops up where you're looking, nearer
     than the list.
   - Any yes/no question pops up the same way.
8. **Everything else** (sleep and wait, books, the level-up, lockpicking, spellmaking,
   enchanting, alchemy, the race menu, training, repair) floats ahead of you as one piece, cut out
   of the flat screen.

## What was checked in the game (emulated headset, captures)

- The container list beside the chest, with the world live behind it. The laser hovers rows
  with the game's own highlight, the right stick scrolls the list (the game's own wheel),
  and B closes the container.
- The big four: the character page, its tabs and the HUD switcher. Clicking the switcher's weapon
  icon opened the inventory list, tabs and switcher.
- Dialogue with a copy of Jensine: her name over her head, her greeting under her face, and the
  topics and service icons beside her. Clicking the barter icon opened her wares beside her.
  The copy has no stock of its own, so the list was empty; the real Jensine in the save has stock.
- A message box (the console's `messagebox`) popped up ahead, and the laser hovered "Yes".
- The pause menu with the RustVR settings beside it.
- The HUD outside menus: the compass low in front, the bars and icons on the left wrist, and the
  card of what's looked at under the gaze point.

## What wasn't checked

- **The real headset.** Text size (a line of the game's text is 1.5 degrees), distances, the
  feel of the laser, and comfort over a long session.
- **These menus and pieces were never seen:**
  - the quantity ("how many") pop-up;
  - sleep and wait;
  - the map and magic pages;
  - a foe's health bar over the foe;
  - lockpicking, the level-up and the race menu;
  - spellmaking, enchanting and alchemy;
  - books.
- **Performance** in a big exterior with a menu open. The world is now rendered every frame
  under menus, where it used to be a frozen picture.

## Decisions

- **The world stays live under menus.** On opening a menu, Oblivion renders a frozen picture of
  the world to draw behind it. In VR your head keeps moving, so that picture is skipped and the
  world is rendered for both eyes every frame (the game is still paused). The hands and the laser
  are drawn too.
- **Real transparency.** The game draws its UI into an alpha render target of ours instead of
  its back buffer, so each piece is exactly as opaque as the game's own art and the rest is clear.
  Each piece's cut edges fade over a few texels. Overall opacity is a setting (`menu_opacity`,
  0.94).
- **Size from the game's text.** Every piece is sized so a line of the game's text is
  `menu_text_deg` (1.5) degrees tall wherever it floats. Settings live in
  `%LOCALAPPDATA%\RustVR\settings.toml` under `[ui]`: `menu_text_deg`, `menu_opacity` and
  `menu_scroll` (wheel notches a second).
- **Where things go:**
  - A container's or merchant's menu: at the thing's distance (0.9 to 1.4 m), 10 degrees under
    the eyes. It moves to the thing's side only if it would cover it, which a chest on the floor
    never does.
  - Your own menus: 1.1 m ahead.
  - Questions: 0.8 m ahead, where you're looking.
  - All of these stay put in the world once placed.
- **B closes menus with the game's Menu Mode (Tab).** In Oblivion, Escape opens the pause menu
  over an open menu instead of closing it. The pause menu and its options, load and save pages
  still close with Escape.
- **Dialogue:** the speaker's name floats over their head (the game shows it in a corner).
- **The crosshair card under where you look** is fixed to your gaze, because it says what looking
  there does.
- **Only the pause menu gets RustVR's settings beside it.** They used to open next to every game
  menu, including a chest's list.

## Not done

- **Most menus aren't cut into pieces yet.** Everything past the container, barter, dialogue,
  the big four, questions and pause floats as one piece cut out of the screen. That includes
  the journal's pages, the map's pages and the magic page beyond its list.
- **Loading screens and the main menu** are as before: a panel in the runtime's room, since there's
  no world to float them in.
- **Transparency is one setting for the whole piece.** The parchment can't be made more see-through
  than the text on it.
