# Playtest: sneaking by crouching

Save: rustvr-city

## Playing it

From the repository root:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-city -Hour 12
```

It builds main, installs it beside the game and boots into the Imperial City's Market District at
noon: people about, guards on their rounds. `powershell -File tools\oblivion.ps1 -Close` puts the
game away and the player's settings back. `rustvr-city` is the developer's own test save, not in
the repository; any save does, and `-At ICMarketDistrict01`, saved as `rustvr-city`, makes
one. The fast boot, the test saves and the game lock are in [DEVELOPING.md](../DEVELOPING.md).

The framework's test range crouches the same way (framework/README.md): after a build,
`target\release\rustvr-play.exe` is the runtime with the range. Close the game first: they share
the runtime's link.

## What it does

Crouch down a little in real life and you're in Oblivion's own Sneak: its sneak eye, its
detection, sneak attacks, Sneak training, pickpocketing ([pickpocket.md](pickpocket.md)). Stand back
up and you're out of it (`framework/crates/runtime/src/mechanics.rs`).

- **How far.** Your head this far under your upright height: 8 cm by default. Upright is measured
  from the head as if held level (at the top of the neck), so looking down, at your hands or a book,
  isn't a crouch. It's learnt once the headset has been still at a likely height a moment (worn, not
  on the desk), rises when you stand up after sitting, and settles slowly into a slouch of under
  half a crouch.
- **It's the game's own sneak.** The runtime presses the game's Crouch/Sneak to match, so the speed
  is the game's normal sneak speed and everything the game does with sneaking follows.
- **Crouched for real, the view stays where your head is.** Oblivion's sneak lowers its own camera
  35 cm; that's held out of the headset for a crouch your own head made (`crate::camera`'s
  `hold_sneak_drop`), so the view isn't pulled down on top of your crouch. The game's camera bob is
  held out too; its sneak drop is a posture, not a bob, and held out only this once.
- **The button's crouch is Oblivion's own**, as ever: the view goes down with the game's camera.
  Crouching for real while in it takes you lower, and doesn't pop you back up.
- **Not while** seated, riding (leaning on a mount's neck isn't sneaking) or swimming (a head ducked
  is a face put into the water).
- **The body crouches** with you, keeping its standing size, standing on the game's ground
  ([ik.md](ik.md)).

## What to try

1. Crouch down a little and hold it: the sneak eye shows.
2. Stay crouched a minute: look about, look at the ground, creep. You stay sneaking.
3. Stand up: you're out of it.
4. Look down at your hands, read a book: that shouldn't sneak.
5. Sneak up behind someone and reach into a pocket.
6. Press the stick: Oblivion's own crouch, the view going down with it. Then crouch for real as
   well: you go lower, and stay sneaking.
7. Try the slider if 8 cm is too little or too much.
8. Watch for the view dipping or jumping as the game goes in or out of Sneak, sitting high or
   sinking a moment later after a real crouch, and, in a mirror ([mirror.md](mirror.md)), the legs
   not bending with your crouch.

The runtime's log (`%LOCALAPPDATA%\RustVR\rustvr.log`) says `physical crouch: crouched` or
`physical crouch: stood up`, with how far under upright the head was and the upright height.

## Settings

The Comfort tab:

| Setting | Default (range) |
|---|---|
| Duck to crouch (`input.physical_crouch`) | on |
| Stealth crouch amount (`input.stealth_crouch_amount`): drag along the slider itself; the value is in its label | 8 cm (5 to 40) |
| Seated play (`comfort.seated`): ducking doesn't crouch | off |

## Decisions

- **A slight crouch by default** (8 cm; it was a 35 cm duck, "Crouch depth"), the slider for more.
- **Upright measured where a nod turns the head**, as if the head were held level: looking down
  lowers the eyes 10 cm or more, and that isn't a crouch.
- **Upright stays upright:** it doesn't settle while you're crouched, and only slowly (2 mm/s) for a
  slouch under half the amount, so a crouch held for minutes stays one. No crouch until it's learnt,
  so putting the headset on doesn't sneak.
- **The game's own sneak,** so its detection, training and speed are the game's.
- **The view is your head's.** The game's camera drop is held out only for a crouch your head made;
  a crouch from the button is the game's, its camera with it.
- **The slider has no number box:** a laser drags a number box to its end, and clicked it wants
  typing.

## Not done / known issues

- **Not seen in a playtest record in Oblivion itself:** the first build was checked in the test
  range only, then merged with the game's view bob and the body on main.

## History

- **On main** it met the view bob (the game's camera bob held out of the headset) and the body
  (Oblivion telling the runtime its eye height). The sneak drop is held out once: the bob leaves
  it out as a posture, and the view lifts for a game's crouch only when the real crouch made it.
- **The first build** replaced the 35 cm duck with a slight crouch, stopped the game's sneak from
  pulling the view down on top of a real crouch (and a deep duck from throwing it 1.8 m up), and
  stopped a real crouch inside the button's from popping the view back up.
