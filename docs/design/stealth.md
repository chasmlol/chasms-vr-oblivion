# Sneaking by crouching

Crouch down a little in real life and Oblivion goes into its own Sneak, with everything the game
does with it. Stand back up and it stands. How far you crouch is the RustVR menu's Comfort tab:
**Stealth crouch amount** (8 cm by default, 5 to 40 cm), shown while **Duck to crouch** is on. It's
a wide slider without the number box beside it: a laser dragged that box to its end. Not in seated
mode.

## How it works

* **The framework's physical crouch** (`mechanics.rs`), as before, with these changes:
  * no crouch until your standing height is learned (still for 1.5 s at a head's height): putting
    the headset on doesn't sneak;
  * the default is a slight crouch, 8 cm under your standing height (it was a 35 cm duck);
  * your standing height stays your standing height:
    * held higher for a second, it rises to the lowest the head was in that second: standing up
      after the game started with you seated, it's your standing height in a few seconds (it
      used to stop 3 cm short, or not rise at all);
    * it settles down only for a slouch under half the threshold, at 2 mm/s (it used to settle at
      2 cm/s anywhere within 12 cm, so a slow or held slight crouch sank it with you);
    * not at all while crouched, so a crouch held for minutes stays one;
  * the height is measured where a nod turns the head (11 cm under the eyes, 8 cm behind), as if
    the head were held level. Looking down lowers the eyes 10 cm or more, but it isn't a crouch.
* **The game's toggle:** crouching presses the game's Crouch/Sneak once, and standing presses it
  again, unless the sneak was the stick's. Never on a horse or in the water.
* **Speed:** the game's own sneak speed, however you got into it.
* **The view:** Oblivion's sneak lowers its own first-person camera 35 cm.
  * A sneak your real crouch started: the adapter leaves that drop out of the headset's view
    (`camera::sneak_drop`, `hold_sneak_drop`), so the view stays where your head really is, until
    the game's camera is back up.
  * A sneak from the button is Oblivion's own crouch: the view goes down as the flat game's does.
    Crouching for real in it takes you lower still; it never jumps back up.
  * The runtime says when your crouch is real (`Intents::crouch`, how far: a spare float, no new
    layout).
* **A bug fixed on the way:** the old crouch raised the view by the game's crouch drop worked out
  from the game's eye height, and Oblivion never says its eye height. So it raised the view 1.8 m
  whenever you ducked into a sneak. It now does that only for a game that says.

## Oblivion's sneak

* **Crouch/Sneak is a toggle.** Each press flips it. There's no hold mode.
* **A press it can't take is lost:** mounted, in a menu, knocked down, paralysed, dead.
* **Swimming:** the sneak doesn't show while swimming, but a press still flips it. So the runtime
  doesn't press in the water.
* **What keeps or clears it:**
  * mounting clears it;
  * a save keeps it;
  * jumps, combat and doors don't touch it.
* **The camera.** Sneaking lowers the first-person eye 35 cm at a race height of 1, over 0.1 s.
* **The rest is the game's:**
  * the sneak eye (shown while sneaking, brighter once someone sees you);
  * sneak attacks (at the moment of the hit; one-handed 4x to 6x, bows 2x to 3x, two-handed
    none);
  * Sneak training (moving while sneaking near people who haven't seen you, one use a second; the
    runtime's walking goes through the game's joystick, so it counts);
  * pickpocketing (the pockets open while sneaking).

## Checked

* **The test range on the emulated headset:**
  * a 15 cm crouch sneaks as it goes down, and holds 61 s through looking 60° to the side, looking
    at the ground (50° down) and creeping 50 cm;
  * standing up ends it;
  * looking at the hands (55° down) and walking upright don't sneak;
  * the `crouch` and `seated` scenarios pass, and so do the mechanics unit tests.
* **Not checked by these:** Oblivion itself. That's the playtest (`docs/playtest/stealth.md`).
