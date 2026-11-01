# Playtest: the duel, a second melee mode, in the test range

This is the framework's test range (`framework/crates/range`), not Oblivion, and needs no save. The
duel is the framework's second melee mode beside the physics sandbox: slow, weighty swings, a parry
window, ripostes, staggers. How it works, the research behind it and every number:
`framework/docs/DUEL.md` (the numbers are all in `framework/crates/melee/src/duel.rs`).

## Playing it

`duel.bat` at this worktree's root (not committed) puts away whatever range or game is up and starts
the range from this build (`<builds>\duel`) in the duel ring, the headset on:

```
<builds>\duel\release\rustvr-play.exe --wait -- --arena duel
```

You start in the ring, facing the duelist across it (Oblivion's male body in iron armour, from your
own `local_assets`). A table at your left hand has a sword and a longsword. Take one: the duelist
comes in and fights as soon as you're in the ring. Outside the ring the range plays the physics
sandbox again; in it, the runtime plays the duel (the HUD's melee line says `duel`, and `DUEL:
staggered` while you're staggered). The menu's Controls tab has **Melee**: *The game's* (the
default: the duel in the ring), *Physics sandbox*, *Duel*.

## What to try, in order

1. **The weight.** Swing the sword about: it should lag a fast swing and swing on past a stop more
   than in the sandbox, and sag a little. Walk with the stick, sword in hand, then without: slower
   with it.
2. **Read its swings.** It winds up slowly (0.8 s) from its right, its left, over its head or from
   low, stepping in, then swings. Don't parry: let one land (you lose 5 to 15 of 100 health; you
   get up again at 0).
3. **Parry.** As its blade comes, push your blade fast into it, towards the blow. Inside the
   window (0.3 s from when your push gets going) the blades clash, your blade bounces back off
   theirs, and the duelist reels back (staggered 0.85 s). Push too early, too slow, or just hold
   your blade in the way: it's knocked aside and the blow lands anyway.
4. **Counter.** Straight after a parry, swing back (any side: your bounce gives you the way, and
   the sword is lighter for a moment). It can't swing while staggered and can't parry for its
   first 0.3 s: a quick counter lands.
5. **Get parried.** Attack it first with a readable swing: it parries (it sees your blade about a
   quarter second late). Your sword goes heavy and harmless (0.85 s, a long buzz) and its riposte
   comes fast: try swinging anyway (it does nothing, and you're hit), then try parrying the
   riposte instead (you can once 0.3 s have passed).
6. **Drags and accels.** Swing at it, and halfway turn your body away so the blade comes late: its
   parry goes up early and your blade lands after. Or start slow and turn into the swing at the
   end so it comes sooner than it can react.
7. Kill it (it bleeds; its limbs stay on): another stands in its place 4 s later.

## What was checked, and what wasn't

Checked headless on the emulated headset (`framework/tools/xrsim/duel_*.toml`, twice each, the same
each time): a parry inside the window staggers the duelist and the counter lands; a push 0.6 s
early is beaten aside and the player struck; parried by the duelist, the player's next swing does
nothing and its riposte strikes; a dragged swing makes it parry early and then lands. The duelist
fights on its own (it walks in and swings from all four sides, each blow landing once), and its
figure draws and holds its guard. A unit check: a dragged blow lands after its parry's window.

Not checked: how any of it feels in the hand (the weight, the stagger's heaviness, the ricochet,
the buzzes), the slower walk with a sword drawn, the right hand on the sword (the checks used the
left), the longsword, and parrying its over and under swings (the checks used its swing from the
right). Tell me the timings that feel wrong: they're all in `framework/crates/melee/src/duel.rs`.

## Not yet

Stamina, feints, chambers and kicks in the duel; Oblivion's own sword in its hand; more than one
duelist; the duel in Oblivion itself (after this has been played).
