# Playtest: the duel, a second melee mode, in the test range

This is the framework's test range (`framework/crates/range`), not Oblivion, and needs no save. The
duel is the framework's second melee mode beside the physics sandbox: slow, weighty swings, a parry
window, ripostes, staggers. How it works, the research behind it and every number:
`framework/docs/DUEL.md` (the numbers are all in `framework/crates/melee/src/duel.rs`).

## Playing it

`duel.bat` at this worktree's root (not committed) puts away whatever range or game is up and starts
the range from this build (`<builds>\duel`) in the duel area, its duelist left out for now (the
two-handed sword first), the headset on:

```
<builds>\duel\release\rustvr-play.exe --wait -- --arena duel --no-duelist
```

A table at your left hand has a sword and a longsword. Started there, the range plays the duel's
rules across the whole room (the HUD's melee line says `duel`). The menu's Controls tab has
**Melee** (*The game's*, *Physics sandbox*, *Duel*), **Sword angle in the hand** (take it again to
apply) and **Strength on a weapon** (live: less is heavier).

## The two-handed sword (now)

The hold is plain physics, with no code for a swing: the sword's own mass and inertia on the arms'
spring and damper, no harder than the hands are strong and no faster than the arms go (a top turn
rate, slower for a heavier blade), its weight borne by the arms. Whatever your hands do, it goes
their way at its own pace.

1. **Take the longsword in both hands** and hold it still at any angle, up or out: it shouldn't
   droop.
2. **Put your hands sideways.** Bring your lower hand up beside the other (or roll your wrists):
   the handle lies along the line between your hands, so the blade turns sideways with them.
3. **Fast slash, as fast as you can, and stop.** The sword can't go faster than about 620 degrees a
   second (a half turn in 0.3 s), so a slash faster than that comes after your hands along the
   same way; it carries on past where they stopped (about 27 degrees after a quick chop) and
   settles there about half a second later. The drawn hands stay on its grip, so they follow your
   real hands.
4. **Slow chop, stop your hands dead**: it follows closely (8 degrees behind), carries on about 8
   degrees and settles.
5. **Lay it on your shoulder**, blade back: it rests there instead of passing through you (your
   head and torso too).
6. **Strength on a weapon**: lower it for a heavier sword (slower to turn, more carry), raise it
   for a lighter one. Tell me where it feels right.

## The duel's combat (later: start without `--no-duelist`)

1. **The walk.** Walk with the stick, sword in hand, then without: slower with it.
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

The sword, checked headless (`framework/tools/xrsim/duel_chop.toml`, `duel_sideways.toml`,
`duel_shoulder.toml`, with `RUSTVR_DEBUG_MELEE=1`): chopped from overhead in 0.25 s it falls 34
degrees behind the hands, carries on 27 past their stop and is with them 0.45 s after it; chopped
in 0.6 s it trails 8 and carries on 8; the hands put sideways over a second, its handle turns with
them (11 degrees behind, then on their line); the blade laid back through the shoulder rests on it
(held 9 cm up off its hold); held still, it stays where the hands put it. Not checked: how heavy it
feels in the hand, and the drawn hands away from your real hands while it comes after them (up to
about 14 cm in a fast slash).

The combat, checked headless on the emulated headset before the hold became plain physics (to be
checked again; `framework/tools/xrsim/duel_parry.toml`, `duel_guard.toml`, twice each, the same
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
