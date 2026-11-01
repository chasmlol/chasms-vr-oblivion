# Playtest: melee, in the test range

This is the framework's test range (`framework/crates/range`), not Oblivion, and needs no save.
How its melee works and why: `framework/docs/MELEE.md`.

On main, Oblivion's own melee is the runtime's combat landing the game's own hits
(`crates/oblivion/src/combat.rs`); the range's melee rules applied to Oblivion are on a separate
branch. Some of the framework's side reaches every game, Oblivion included: weapons going into the
hand by their grips.

## Playing it

Build the runtime and the range from the repository's root (`tools\oblivion.ps1` builds them too),
then start the range in one of its areas, the headset on:

```
cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-runtime -p rustvr-range
target\release\rustvr-play.exe --wait -- --arena arena
```

`--wait` waits for the headset; what follows `--` goes to the range. The range and the game share
the runtime's link, so put the game away first (`powershell -File tools\oblivion.ps1 -Close`), or
give the range a link and a folder of its own with `RUSTVR_LINK` and `RUSTVR_DIR` (its settings and
logs then live in that folder). `powershell -File tools\play.ps1 -Close` stops a range started with
`--wait`. A feature worktree's build: `powershell -File tools\play.ps1 <worktree> -Range` (the
range's usual start). See `framework/README.md`.

| `--arena` | Where you start |
|---|---|
| `range` (the default) | the range's usual start: training dummies ahead, the throwables' table at your left (an arming sword and a dagger on it), the melee rack behind it. Gravity gloves and the range's own buttons. |
| `arena` | the arena: fighters come at you once you're in it, a table at your left with a sword, a dagger, a longsword and a shield. When a wave is down for good (dead, out cold or gone), the next comes five seconds later, harder: an Easy sword and an Easy sword and shield; then Medium, one-handed, two swords and a shield-bearer in full plate; then Hard. |
| `rack` | facing the melee rack: dagger, mace, axe, longsword, 8 kg maul, shield, spear, staff. |
| `bench` | a step from three armoured dummies (leather left, chainmail ahead, plate right), two more one behind the other to run through, a wooden post; a mace, a sword and a dagger on a table at your left. |
| `climb` | in front of a 2 m stone block to climb. |

The `arena`, `rack`, `bench` and `climb` starts use the melee buttons (below). Every start has the gravity
gloves for pulling things in (telekinesis is off). The areas are all in one room, so you can walk
between them; the fighters only come once you step into the arena.

## What it does

The framework's melee (`framework/crates/melee`, the runtime's `combat.rs`, `powers.rs` and
`rig/melee.rs`, `rig/climbing.rs`, `rig/telekinesis.rs`; the range's people in `range/src/arena.rs`,
`ragdoll.rs`, `brain.rs`).

**Controls** (the melee buttons, in the melee areas):

- **Grip** near a weapon's handle takes it by that handle: the hand goes onto its grip (as the
  weapon's made to be held, or reversed if the hand comes at it the other way round) and the
  weapon's pulled into the hand. Grip anywhere else on it and it's held where you took it. Let go
  fast to throw.
- **Trigger** on a held weapon slides the hand along it (a light pull stiffly, a full pull freely);
  on the arming sword or the longsword, on over the guard onto the blade (half-swording) and back.
- **Empty hand:** hits (a punch). Grip on a creature takes hold of it where the hand is (the
  throat chokes, the mouth muffles); grip on an edge of the world holds it, to climb.
- **Movement stick click:** kicks (at what you look at within 2 m, with the foot on its side).
- **Dominant hand's B:** slows time (the focus meter on the HUD).
- **Dominant stick click:** jumps; hanging from a ledge, a climb jump off it.
- **The off hand, empty, trigger:** charges the spell, fire or shock, alternating each cast (the
  HUD says which). Held by a weapon's blade in the other hand, the charge fills the blade (it
  glows).
- **Gravity gloves:** point an empty hand at a loose thing, grip to lock on and flick back: it
  flies to you, and a weapon caught goes into the hand by its grip.

At the `range` start the off hand's stick click slows time, and jump and crouch are the range's
own.

**The rules, in short:** damage from the impact; armour by material (an edge glances off plate,
cuts leather; a mace hurts all); stabs and draw cuts that stick and can be rocked, twisted, pulled
out or run on through into a second body; easy dismemberment (an edge into an arm, a hand, a shin,
a foot or the neck takes it off at its joint; never a thigh or the torso; armour that stops the
edge stops the cut); knockouts (a mace on a bare head, 6 s, out cold on its feet); muscled ragdolls
that stand, stagger, fall and get up; blocks, parries and disarms; fighters with their own moves
(combos, lunges, shield pushes, dodges from thrusts, feints at Medium and Hard), one, two or three
at you at once by tier, fleeing when disarmed with nothing to take up; grappling, chokes and
tearing limbs off with both hands; throwing (the dagger and spear fly point first, the arming sword
tumbles); slowed time on the focus meter (30 at the start, 60 at most, about 6.7 s from 30);
imbuing (a fire-filled blade sets a body alight: dead in 4 s, charred through at 10 s alight, then
stiff and torn apart by a pull or a hard hit; shock may stun); breakables (bottles, boxes, crates,
the barrel) back whole after 8 s; climbing.

**Beyond melee:** the range's guns, grenades and knives on the creatures. A pistol round 15, a
rifle's 30 and on through, a shotgun's 45, before the armour (leather lets 0.9 through, chainmail
0.6, plate 0.4) and the part (a head x3, the limbs x0.5, the hands x0.25); a round through the arm
disarms. A frag does 120 at its centre, falling off over 6 m, floors everything within 4 m and
tears limbs off within 1.2 m; the room's tables and walls shield from it. A thrown knife hits as a
thrown dagger does. A spell cast at a creature lands at once (a fire bolt or lightning).

## What to try

**Holding**

1. Grips (`rack`): take each weapon by its handle with your hand turned any way: it turns into
   your hand at its proper grip (a sword's edge out from the knuckles, the axe's edge forward, the
   shield's face out from the back of your hand). Thumb towards the pommel for a reversed grip. A
   second hand on the longsword's or the spear's handle goes onto it the same way. The axe by its
   head is held there.
2. Weight: the dagger keeps up; the 8 kg maul sags, trails a fast swing and swings on past where
   you stop. Chop hard with the maul, the spear or the staff while turning your wrist: it stays in
   your hand.
3. Two hands on the longsword or the maul: much easier, the weapon leaning towards the second hand.
4. Sliding along the spear or staff; over a sword's guard onto its blade.
5. A stab left in something: walk away, and once your hand's further than you reach, it lets go.

**Hitting** (the dummies at `range`, the `bench`)

6. A lazy swing barely hurts, a hard one hurts; the dummies pale as they lose health.
7. Armour: the edge glances off the plate dummy and cuts the leather one; the mace hurts them all;
   a blow that does nothing scratches the armour.
8. Stabs that stick, by pressure or a draw cut; rock, twist and pull out (blood follows). Stab the
   first of the two dummies hard and push on into the second.
9. Dismemberment: an arm, a hand, a shin, a foot or the head with any edge.
10. Knockouts: the mace on the chainmail dummy's bare head.
11. Feedback: hits sound by what hit what, sparks off metal, the buzz of a stuck blade and of one
    drawn along the table or a body (every 2 cm).
12. Blood on the blade where it went in, wounds on the body, a cut neck bleeding.
13. Being struck (`arena`): a fighter's blow hurts you 5 to 15 (the head x1.5, the hands x0.5).

**Defence** (`arena`)

14. Block a blow; against the maul or the longsword your block can stagger you.
15. Parry: move into the blow (over 1 m/s): the swing turns back, or the fighter recoils.
16. Disarm: hit a fighter's hand hard, stab through its arm, or take its weapon and pull.

**Enemies** (`arena`)

17. They stand still and move smoothly, in slowed time too; shove one, knock it down, watch it get
    up once it's still. Falls and slams hurt.
18. A light hit flinches, a hard one staggers or floors; any stagger stops its swing.
19. Their moves, their spacing, dodges and feints.
20. Fear and voices: a disarmed fighter fetches a weapon or flees; grunts, shouts, a last line by
    how it died, a scream as it falls.

**Grappling and unarmed**

21. Grab a dummy or a fighter: steady in the hold. By the neck while it's unaware it goes down at
    once; by a shoulder or a wrist it struggles and may break free; the throat chokes it (5 a
    second, and it drops its weapons).
22. Take a dummy by both shoulders (or an arm and the head) and yank them apart hard.
23. Punches (only a hard one does anything) and kicks (a body that's down is shoved hard).

**Extras**

24. Throw the dagger, the arming sword and the spear.
25. The gloves: point at a weapon on the rack, grip and flick, catch it.
26. Slowed time: B on your dominant hand.
27. Imbuing: charge fire by your blade until it glows, then hit or stab a dummy. Cast once to
    switch to shock: a hit may stun.
28. Breaking: hit or throw the bottles, boxes, crates and the barrel at the range's start.
29. Climbing (`climb`): grip the block's top edge with both hands, pull down; click the dominant
    stick to jump up off it, or push down to your hips and let go to step up. An axe stuck in a
    wall holds you too.
30. Guns: the rifle and pistol on the table by the start, into a dummy or a fighter; a dead body
    jerks when shot.
31. Grenades (the chest pouch, or the throwables' table): a frag's blast hurts them, floors them
    and throws them.
32. Throwing knives into a dummy: one that sticks rides with it.
33. Spells: the fire bolt (knocks a dummy down) and the shock (jolts it, may stun).

## Settings

`[melee]` in `%LOCALAPPDATA%\RustVR\settings.toml` (`framework/crates/runtime/src/settings.rs`);
none of these is in the menu. The range's melee areas ask for the melee buttons; the rest is the
settings' unless a game says (`MeleeRules`).

| Key | Default | What it does |
|---|---|---|
| `easy_dismemberment` | true | An edge takes off what it cuts into at its joint (above). False: the strict cut (hard, square, within 4 cm of a joint, 12 m/s for the arming sword). |
| `telekinesis` | false | Telekinesis (on things and on bodies: catching, throwing, pulling a body apart with both hands) in place of the gravity gloves, everywhere. |
| `stamina` | false | Swinging costs stamina and a tired player's hands are weaker (the HUD shows it). |
| `stamina_time` | 12 | Seconds of hard swinging a full stamina lasts (3 tires fast). |
| `stamina_refill` | 6 | Seconds it takes to come back. |
| `slow_time` | true | Slowing time on the focus meter. |
| `climbing` | true | Climbing by the hands. |
| `strength` | 1 | The hands' strength on a weapon, times the standard player's. |

## Decisions

All in `framework/docs/MELEE.md`.

- **A weapon taken near a handle goes onto its grip** and is pulled into the hand (the user's rule:
  weapons snap to their rigged grips). Off its handles it's held where you took it.
- **Easy dismemberment on by default:** the strict cut needs a fast, square cut at the joint, which
  proved too hard to land in play.
- **Knockouts** come above set speeds and last 6 s.
- **Asked for:** a hand on the throat hurts (5 a second); a pulled weapon is wrenched away (by hand
  or telekinesis); fighters feint.
- **Always on in the range:** tearing limbs off by hand (and, with telekinesis on, its throw, its
  hold on bodies and its tearing).
- **The focus meter holds 60** (an earlier call said 30).
- **The melee buttons are the game's to ask for:** the range's melee areas ask, Oblivion keeps its
  own. Telekinesis is off unless the settings turn it on; the gravity gloves pull things in.
- **Each wave comes while you're in the arena.**
- **The runtime's gore stays its own** (wounds bleed, blood runs and pools), with blood on the blade
  and a cut neck bleeding.
- **Fire and shock alternate** as they're cast: the range has no spell wheel.
- **Guns and grenades are ours** (numbers above); a thrown knife hits as a thrown dagger does; a
  spell cast at a creature lands at once.

## Not done / known issues

- Thrown and telekinesis-held things are judged as weapons only if they're melee weapons or
  throwing knives: a thrown bottle breaks, but doesn't hurt.
- A body held at full stretch on a stuck blade drops it (the stuck rule: the grip's past your reach).
- **Never checked by feel in a headset:** the weight, the grips and a held body's steadiness above
  all; sounds by ear and the controllers' buzz; the pistol on a fighter; the incendiary; a knife
  riding a fighter as it moves; a spell at a fighter; the gloves' pull; a fighter's weapon taken
  by its handle and wrenched away; a heavy weapon's push on your guard; a body thrown into another;
  the mouth grab; a held arm following your hand; a stagger stopping a swing; rocking and twisting
  a shallow blade; a charred body tearing apart; one hand taking a sword's pommel while the other
  slides. The emulated headset checked the rest with scripts and captures.

## History

Newest first.

- **Creatures stopped shaking:** a standing dummy's upper arms twisted back and forth with nothing
  touching it, and a head held by the throat shook; now still. The range's guns, grenades, knives
  and spells now hit the creatures.
- **Easy Dismemberment, on by default:** the strict cut (about 19 m/s, square at the joint) was too
  hard to land.
- **Weapons snap to their grips; heavy holds steady; the gloves back in the range:** a chop with
  the wrist turning spun a maul about its haft and off the hand; telekinesis replaced the gloves
  in the melee areas and is now the settings' to turn on.
- **The melee audited and completed:** run-through stabs, telekinesis on bodies, tearing limbs off,
  a full char, each creature's own voice lines, holds the creature breaks free of, the push tables,
  the creature side (moves, guards, dodges, grappling), rubbing buzzes, waves only in the arena.
