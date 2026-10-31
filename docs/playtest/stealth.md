# Playtest: sneaking by crouching

Save: rustvr-city

Crouch down a little in real life and you're in Oblivion's own Sneak: its sneak eye, its detection,
sneak attacks, Sneak training, pickpocketing. Stand back up and you're out of it.

- **How far:** the RustVR menu's Comfort tab, **Stealth crouch amount**: 8 cm by default, 5 to
  40 cm (it was a 35 cm duck, called "Crouch depth"). Drag along the slider itself; the value is in
  its label. "Duck to crouch" still turns it on and off.
- **Crouched for real, the view stays where your head is.** Oblivion's sneak lowers its own camera
  35 cm; that no longer pulls your view down on top of your own crouch (and a deep duck no longer
  throws the view 1.8 m up).
- **The button's crouch is Oblivion's own**, as before: the view goes down with the game's.
  Crouching for real while in it takes you lower; it doesn't pop you back up any more.
- **Speed** is the game's normal sneak speed, however you're sneaking.

`playtest.bat` boots into the Imperial City's Market District at noon: people about, guards on
their rounds. `range.bat` starts the framework's test range.

## Try

1. Crouch down a little and hold it: the sneak eye shows.
2. Stay crouched a minute: look about, look at the ground, creep. You stay sneaking.
3. Stand up: you're out of it.
4. Look down at your hands, read a book: that shouldn't sneak.
5. Sneak up behind someone, reach into a pocket.
6. Press the stick: Oblivion's own crouch, the view going down with it, as before. Then crouch for
   real as well: you go lower, and stay sneaking.
7. Try the slider if 8 cm is too little or too much.

## Look for

- A crouch that doesn't sneak, or sneaking you didn't mean.
- The view dipping or jumping when the game goes in or out of Sneak.

The runtime log (kept in `<builds>\stealth\playtest-<n>` after you close) says
`physical crouch: crouched` / `stood up`, with how far under your standing height your head was.

Not checked before this: anything in Oblivion itself (only the test range).

## On main (merged with the view bob and the body)

`playtest-main-stealth.bat` in `<checkout>` plays main's build. There the game's own
camera bob is held out of the headset too, and Oblivion tells the runtime its eye height for the
body. The sneak drop is held out once: the bob leaves it out (it's the sneak's posture, not a
bob), and the runtime lifts the view for a game's crouch only when the real crouch made it. Look
for the view sitting high, or sinking a moment later, after a real crouch; and for the body's legs
not bending with your crouch in the mirror.
