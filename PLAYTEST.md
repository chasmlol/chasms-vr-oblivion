# Playtest: sneaking by leaning over

Save: rustvr-city

You go into Oblivion's own Sneak by leaning over a little, as you would to creep up on someone. Stand
up straight and you're out of it. Crouching down works too. It's the game's real sneak: its sneak
eye, its detection, its sneak attacks, its Sneak skill and its pickpocketing.

- **How far you have to go** is the RustVR menu's Comfort tab: **Stealth crouch amount**. It runs
  from "a slight lean" (the default) through "a lean" and "a crouch" to "a deep crouch". It replaces
  the old "Crouch depth".
- **"Duck to crouch"** still turns all of it on and off. **"Buzz going in and out of it"** (on)
  gives both hands a light tick as the game goes in or out of Sneak.
- **The right stick** still toggles Sneak by hand, as before. That's yours: standing up doesn't
  undo a sneak you started with the stick.
- **The view no longer drops when the game sneaks**, however it started. Oblivion's sneak lowers
  its own camera 35 cm (an animation). Before, that pulled your view down; now the view stays where
  your head is.

`playtest.bat` boots into the Imperial City's Market District at noon: people about, guards on
their rounds. `range.bat` starts the framework's test range, where leaning crouches the range's
own body (it walks at half speed).

How it reads your posture and why: `docs/research/stealth.md`.

## Try

1. **Stand normally, then lean over slightly** (about 20 degrees at the hips, as if creeping) and
   hold it. Within half a second you're sneaking: the sneak eye shows over what you look at, and
   your hands tick once.
2. **Stay leaning a minute or more.** Look about, look at the ground, creep forward a few steps.
   You stay sneaking the whole time.
3. **Stand up straight.** You're out of it in about half a second, and your hands tick.
4. **Things that shouldn't sneak** (stand normally for each):
   - look down at your hands;
   - glance at the ground and back up;
   - open a book or the map and read it standing;
   - work the alchemy bench;
   - pick something up off a table or a counter;
   - walk about the room.
5. **Crouch down deep, then stand.** It sneaks and un-sneaks as well.
6. **Sneak up on someone:**
   - the eye stays closed while they don't see you and opens as they notice you;
   - walk up behind someone while leaning, reach into a pocket (the pockets only exist while you
     sneak).
7. **Sneak through things that change it:** a menu (inventory, a book) while leaning, a jump, wading
   into water and out, a save loaded while leaning or standing. After each, if you're still leaning
   you should be sneaking again within a second; standing, you should be standing.
8. **Try the slider:** set "a crouch" or "a deep crouch". A lean no longer sneaks, only a crouch
   that deep.

## Look for

- **False sneaks:** anything in step 4 that sneaks (the eye shows when it shouldn't).
- **Missed sneaks:** a lean you'd call sneaking that doesn't sneak, or takes too long. Say how far
  you leaned and whether you were looking down.
- **Dropping out:** sneaking that stops while you're still leaning (looking around, at the ground,
  creeping).
- **The view:** it stays where your head really is. It mustn't dip when the game goes into Sneak
  (the game lowers its own camera), nor jump up.
- **Out of step:** the game sneaking while you stand, or standing while you lean, for more than a
  second (after a load, the water, a jump, a menu).
- **The body:** the drawn body leans because you lean; it shouldn't go into the game's sneak pose.

The runtime log (kept in `<builds>\stealth\playtest-<n>` after you close) says each time with
`stealth:`:

- when you lean in and stand up, and how deep the lean read;
- what the game did a second later: its eye height, how much the view was held up;
- when the game changed Sneak by itself.

## Not checked before this playtest

- **Never booted in Oblivion** (only the framework's test range on the emulated headset). The game's
  own sneak, its eye, its pockets and the view's height in the game are all for this playtest.
- **The posture's numbers** (the slight lean's 15 cm, the holds, the look-down discount) are from a
  model of a body, not a real one. They'll want tuning to how you actually lean.
