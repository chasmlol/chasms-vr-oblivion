# Playtest: sneaking by crouching

Save: rustvr-city

Crouch down a little in real life and you're in Oblivion's own Sneak: its sneak eye, its detection,
sneak attacks, Sneak training, pickpocketing. Stand back up and you're out of it.

- **How far:** the RustVR menu's Comfort tab, **Stealth crouch amount**: 12 cm by default (it was a
  35 cm duck, called "Crouch depth"). "Duck to crouch" still turns it on and off.
- **The view stays where your head is.** Oblivion's sneak lowers its own camera 35 cm. Before,
  that pulled your view down on top of your own crouch (and a deep duck threw the view 1.8 m up);
  now it doesn't.
- **The less you crouch, the faster you sneak.** Crouched as low as the game's own sneak camera
  (35 cm), you walk at the game's sneak pace (60% of walking). Higher than that you're faster:
  12 cm down is about 86% of walking.
- **The right stick** still toggles Sneak by hand (at the game's own sneak pace).

`playtest.bat` boots into the Imperial City's Market District at noon: people about, guards on
their rounds. `range.bat` starts the framework's test range.

## Try

1. Crouch down a little and hold it: the sneak eye shows.
2. Stay crouched a minute: look about, look at the ground, creep. You stay sneaking.
3. Walk with the stick while crouched a little, then lower: the lower you go, the slower you walk,
   down to the game's sneak pace about 35 cm down.
4. Stand up: you're out of it.
5. Look down at your hands, read a book: that shouldn't sneak.
6. Sneak up behind someone, reach into a pocket.
7. Try the slider if 12 cm is too little or too much.

## Look for

- A crouch that doesn't sneak, or sneaking you didn't mean.
- The pace: too fast or too slow for how low you are.
- The view dipping or jumping when the game goes in or out of Sneak.

The runtime log (kept in `<builds>\stealth\playtest-<n>` after you close) says
`physical crouch: crouched` / `stood up`, with how far under your standing height your head was.

Not checked before this: anything in Oblivion itself (only the test range).
