# Playtest: doors, levers, hanging chains and the lock by hand

Save: rustvr-doors

## Playing it

From the repo root:

    powershell -File tools\oblivion.ps1 -Save rustvr-doors

It builds main, installs it beside the game and boots straight into the save for the headset;
`powershell -File tools\oblivion.ps1 -Close` puts the game away and your settings back.
`docs/DEVELOPING.md` has the fast boot, the test saves and the game lock.

`rustvr-doors` is the developer's own test save, not in the repository: Bethesda's test cell
`TestTraps`, with 20 lockpicks in the pack. Make it from any save (here `rustvr-base`), then
`-Close`:

    powershell -File tools\oblivion.ps1 -Save rustvr-base -At TestTraps -Give "0000000A 20" -Commands "SaveGame rustvr-doors"

In `TestTraps`, a row of doors that swing in place is on your right: a step ahead, a big planked
cave door, then a double cave door, a barred prison cell gate and an Imperial dungeon door. The
fort ruin's floor lever is about 16 m straight ahead (where you face as the game loads), past the
trap doors. The hanging roots, the two chain dolls, the rope with the bucket and the rope of skulls
are in the same cell.

Other places, each `powershell -File tools\oblivion.ps1 -Save rustvr-doors` plus:

| Add | What's there |
|---|---|
| `-At WarehouseLock -Give "0000000A 20"` | Bethesda's lock test cell: rows of locked fence gates |
| `-At WarehouseFortRuinsHALLrooms` | The fort ruins' test hall: a portcullis, a turning switch |
| `-At WarehouseAyleidRooms03` | Ayleid wall switches |
| `-At WarehouseUpperClass` | Unlocked display cases: a lid on a hinge |
| `-At ImperialDungeon01` | The Imperial Dungeon's cell gates |
| `-At FortAsh02` | Fort Ash's turning switches |
| `-At ChorrolMagesGuild` | Chorrol's Mages Guild: locked display cases |
| `-At ChorrolNorthernGoodsAndTrade`, `-At ChorrolFireAndSteel`, `-At GottshawInnTavern` | Ordinary house doors, wood on iron hinges |
| `-Save rustvr-ruin -At Vilverin03` (in place of `-Save rustvr-doors`) | Vilverin's halls: Ayleid switch stones on pillars |

The research and the design: `docs/research/world.md`. How it's wired into the game: the "Doors
and levers" and "The lock" rows of `docs/OBLIVION.md`.

## What it does

- **Doors that swing or slide in place** (not load doors, not Oblivion gates) and **scripted
  activators with a moving part** (levers, switches, buttons, chains, wheels) within 6 m of you, 24
  at most, are hinges or slides in the runtime's physics (`OBJ_VALVE`, `JOINT_PANEL`;
  `crates/oblivion/src/world.rs`). How each moves is its model's own "Open" or "Forward" animation,
  read from the game's archives; the other nodes that animation moves (handles, gears) go with it.
- **A shut door is latched**: it gives 2 degrees in its frame (it rattles). Pulled or palmed most
  of the way into that play, it's the game's own Activate, as the Activate control would be: its
  script, lock and key, open state. If the game opens it, its own animation is stopped where it
  began, its open sound plays, and its nodes and keyframed Havok body follow the hand every frame.
  Swung back within 0.7 degrees of its frame, slow, it's the game's Activate again: closed,
  latched, its close sound. The latch letting go or catching clicks in the hands on it.
- **A sliding door** (a portcullis, a ruin's stone door) is too heavy for a hand: taken in a hand
  it's tugged, the game's Activate, and the game's animation moves it.
- **An activator fires at the end of its travel**: a lever or a switch at 0.97 of it, a button or
  a switch stone that springs back at 0.88 (a palm bottoms out on the surround a little short). It
  is ready again once it's back within 0.35 of where it started (a two-way lever: of its other
  end). An animation the game starts on it within 3 s of that Activate (a fort switch's script plays
  its turn two seconds later) is stopped: the hand already moved it.
- **Left alone**: traps and their triggers (models named or filed as traps, triggers, pressure
  plates), and an activator's big parts (more than 1.6 m, or a slide past 0.9 m: the gates and
  bridges levers work).
- **What the hand meets** is the part's Havok shape, or its drawn shape where it has no body of its
  own or its body lies off its drawing (an Ayleid wall switch's box sits half a metre out from the
  wall).
- **Weight.** The model's mass where it has one; else a leaf (thin one way) weighs its area as a
  4.5 cm plank of its Havok material, or, of metal, as heavy for its size as the game's own cell
  gates (100 kg on 0.68 by 2.28 m); a lever or a stone weighs its drawn volume of its material. A
  lid is never heavier than half an arm (90 N) lifts by its edge. Hinge friction is its pins' (0.3
  on 1 cm pins, under its weight). A hinge more than half way to level (a lid, a trap door) turns
  under its own weight and props open at the end of its swing, like a stay.
- **The arm and the body** (`framework/crates/runtime/src/rig/valve.rs`): a hand pulls its point
  on the part up to 180 N over a 5 cm stretch; past that the body pulls, without limit, so the part
  stays within 5 cm of the controller along the way it moves. A palm pushes a leaf, never pulls it.
  The way it can't go (off its hinge, past its stop), the hand lets go 30 cm from it (an empty
  hand's leash is 70 cm). A door stops on the player's body or a creature rather than pushing
  through. Let go mid-swing, it slows to a stop over about 2 s. A lever's two positions hold
  against 25 N at its end; a button or switch stone pushes back 60 N pushed all the way in, and
  comes back no faster than the game's animation brings it.
- **A hand on a door or a lever is busy**: its trigger doesn't reach the game's Activate at gaze.
  An empty, free hand's trigger still does (the game's own opening, the runtime's leaf following
  it).
- **Hanging things** (`crates/oblivion/src/props.rs`): every Havok body within 3 m of your chest,
  48 at most, in a Havok world, neither fixed nor keyframed, not an item's, an NPC's or a
  creature's (chains, chain dolls, roots, ropes, a sprung trap's chain) is one of the game's props
  on the game's constraints (`OBJF_CONSTRAINED`). One offered stays so until it's 4 m off. It's
  solid to the hands: a hand rests on it and pushes it, as fast as its controller goes on into it,
  each push given to its body as an impulse at that point (Havok's `applyPointImpulse`), and the
  game's Havok swings it. Taken in a hand, it's held with the game's own grab (a
  `bhkMouseSpringAction` made as the game's grab makes it, its strength the game's), pulling its
  middle to where the hand would have it. The hand stays where the game has it, so a chain taut on
  its anchor holds the hand back, and pulled 30 cm off it the hand comes away. The gravity gloves
  don't pull them.
- **The lock** (`crates/oblivion/src/lock.rs`): while the game's lock menu is open, the right hand
  is its pick, through the game's cursor and mouse, so every rule is the game's (tumbler speeds, the
  Security windows, broken picks). Moved sideways from where it was when the lock opened, the pick
  goes under the next tumbler every 2.5 cm; moved up faster than 0.3 m/s, it pushes the tumbler up;
  the trigger past 0.75 sets it. A is Auto Attempt, B closes the lock. A tumbler riding up ticks in
  the hand, a set one clicks, a broken pick jolts.

## What to try

1. **Push the cave door open.** Put a palm on the big planked door ahead (about 130 kg) and push.
   It gives a hair (its latch, rattling), then the game opens it, and your hand swings it on. Shove
   it and let go: it swings on and stops over a couple of seconds.
2. **Take it by any part.** Grip anywhere on the leaf: the fingers close where they are. Push and
   pull it about: your drawn hand stays within 5 cm of the controller, the body bringing it past
   that. Hold it and walk backwards: it comes with you. Walk into its swing: it stops on you. Pull
   it 30 cm the way it can't go and the hand lets go.
3. **Shut it.** Swing it back into its frame: the game closes it, with its sound, and it latches.
   Pull on it to open it again.
4. **The prison gate and the dungeon door** along the row: the same, each gate leaf on its own
   hinge (the game's cell gates are 100 kg a leaf).
5. **The old way.** Look at a door and pull an empty hand's trigger: the game plays its own opening
   and your hand meets the door where it's drawn as it swings.
6. **The lever**, about 16 m ahead past the trap doors, by a rock. Grip its handle and push it all
   the way over: at its stop the game's Activate (in this cell it works the secret stone door
   nearby). It holds at either end. Pull it all the way back to work it again. The trigger squeezed
   with the grip does nothing more.
7. **The hanging things.** Push or swat the roots, the chain dolls, the bucket's rope, the rope of
   skulls: the hand doesn't go in, they swing off it and back. Grip one: it comes with the hand as
   far as its chain lets it, then the hand is held back and comes off. Let go mid-swing: it swings
   on. Tell me if a chain doll is too heavy to swing about (the grab's strength is the game's).
8. **The lock** (`-At WarehouseLock -Give "0000000A 20"`). Push or pull a locked gate: the game's
   lock menu opens. With the right hand, move left and right under a tumbler, flick up to push it,
   pull the trigger as it reaches the top. Too early or too late and a pick breaks. A is Auto
   Attempt, B closes it. Pick all five and the game swings the gate open, your hand meeting it.
9. **Ayleid switch stones** (`-Save rustvr-ruin -At Vilverin03`): push a glowing stone into its
   pillar with a palm, 25 cm. At the bottom the game's Activate and its grinding; it comes back out
   as slowly as the game's animation. The wall switches (`-At WarehouseAyleidRooms03`) push into
   the wall the same way.
10. **A fort's turning switch** (`-At FortAsh02` or `-At WarehouseFortRuinsHALLrooms`): take one end
    of the iron T-bar (about 55 kg) and walk it round a quarter turn. At its end the game's Activate;
    its script works what it's wired to. It holds at either end; turn it back to work it again.
11. **A portcullis or a ruin's stone door**: take it in a hand and the game raises it (or drops it
    into the floor) with its own animation.
12. **A display case's lid** (`-At WarehouseUpperClass`): lift it by its front edge. Let go short of
    fully open and it falls shut; the game closes it at the frame.
13. **House doors** in Chorrol and the Gottshaw Inn: ordinary doors on iron hinges.
14. **A door the game keeps shut** (a script's, or locked with no picks): it rattles, the game says
    why ("locked", its rattle sound) and nothing opens.

## Settings

The runtime's menu (the menu button), Hands tab:

- "Mechanisms' stiffness (valves, switches)": 0.25 to 2, default 1. Scales every hinge's friction
  and how hard a lever's positions hold.
- "Ghost hands (a see-through hand where your real one is when it's held back)": off by default
  (shown under "Physics hands", on by default). On, it shows the controller while a taut chain or
  a door's stop holds your hand back.
- "Grab transition (fast .. slow)": 0 to 0.4 s, default 0.15: how long a hand takes onto a part and
  back to the controller after letting go.

Nothing in `rustvr-oblivion.toml` shapes it.

## Decisions

- **The game's Activate decides**: a latched door tugged to the end of its play, a lever at its
  stop, are the game's own Activate, so locks, keys, scripts and open states stay the game's. A palm
  push opens an unlocked door: players love push-to-open, though a real latch would want the handle
  turned.
- **The hand is the door's animation**: once the hand has it open, the game's animation is stopped,
  and the door's node, its handles (posed by the game's animation at the matching moment) and its
  keyframed Havok body are put where the runtime has the leaf every frame, so the player and NPCs
  meet it where it's drawn. The game opening or closing it itself (an NPC, a script, the trigger at
  gaze) plays its animation and the runtime follows.
- **Sliding doors are the game's to move**: a hand can't lift a stone slab.
- **Mass from the model's own shape**: the game gives most doors and levers no mass (their bodies
  are keyframed), so a leaf is its area of plank or, of metal, as heavy as the game's cell gates; a
  lever or stone, its drawn volume. The 2 s swing and the lever's 25 N hold are design values, not
  the game's.
- **Buttons and switch stones push in against 60 N**, however heavy the stone (a wall switch's
  drawn stone is about 460 kg): a firm press. The game has no value for it.
- **An arm pulls 180 N over 5 cm, then the body** (round 1): a controller has no weight to feel,
  so a part that lags the hand reads as too heavy and a stretched arm. Its weight shows in that
  give and in how it swings on when let go. A wheel geared to a gate still turns no faster than the
  gate; a stiff wheel one arm can't turn, the body can.
- **30 cm to let go of a hinge or slide**, as the rig's foregrip (it was 50): along the way it
  moves, the body brings the part, so nothing is leaned on with a stretched arm.
- **A lever fires at its stop** (0.97, round 1); a spring-back button at 0.88, where the checks had
  a palm bottoming out on its surround.
- **A hand on a door or a lever is busy** (round 1): the trigger squeezed with the grip ran the
  lever's script again at every grab through the game's Activate at gaze.
- **Hanging things are solid and taken with the game's own grab** (round 2): letting a hand into a
  prop and asking the game to push it out barely moved a chain on its constraints. With the game's
  grab, its strength and damping are the game's. It pulls the piece's middle, not the exact point
  in the hand: a chain link is small.
- **The lock is the game's lock menu, worked by the hand**: the hand drives the game's pick (its
  cursor and mouse) instead of the laser, so every rule stays the game's. No pick is drawn in the
  hand: the game draws its own under its tumblers, and a second one in the hand, away from the lock
  it works, would touch nothing.

## Not done / known issues

- **The lock drawn at the door's keyhole**, with the pick in the hand in it (the game's
  `lockpicking\*.nif` models posed by the menu's state), rather than the menu's panel. (Lockpicks
  and keys drawn from the body are the `interact` branch's, not on main.)
- **The pick is always the right hand**: the lock doesn't follow the dominant-hand setting.
- **A few older models** have their animation after a block the model reader doesn't read yet
  (particle and material controllers); reading stops there, so they stay the game's Activate.
- **Drawers, chests' lids, buttons and pull chains** in vanilla: Oblivion's containers have no lid
  animations (its 538 animated containers are hanging corpses), and its buttons and chains are
  mostly traps' or the Shivering Isles'. Buttons that slide in (the Shivering Isles' ruins) should
  work as slides with a spring back; unchecked.
- **Not tried in the game**: following an NPC through a door, a trap door, a ruin's stone door, a
  display case's lid held fully open on its stay, the door sounds by ear, the haptics by hand.
- **Harvesting plants by pulling, sitting, sleeping**: in `docs/research/world.md`, not on main.

## History

- **Round 3 (30 October): a crash.** The hanging things worked, but the game crashed in the
  creatures' bone scan (`combat::bones`) as `TestTraps`' two goblins came to life: it read a mesh's
  fields as children and followed garbage into freed memory. The gore branch had the same crash;
  its fix, going only into nodes by the game's own type information, is on main.
- **Round 2 (30 October).** Doors, the lever and walking backwards were good; the hanging things
  still let the hands through and couldn't be taken. They became `OBJF_CONSTRAINED` props, solid
  and pushed by how deep the controller is, taken with the game's own grab, and kept offered to
  4 m.
- **Round 1 (30 October, the first headset run).** "Everything is insanely way too heavy": the arm
  was damped on the part's own speed, so a 126 kg door trailed the hand by a quarter of a metre;
  now 180 N over a 5 cm stretch, then the body. The hanging things went through the hands: every
  loose Havok body near you became a prop the hands push. The lever fired at 88% and the trigger at
  gaze ran its script at the grab: now it fires at its stop and a hand on it is busy. Walking away
  with a door stretched the arm: now the body pulls, and the hand lets go at 30 cm (was 50).
- **Before the headset**: the cave door, the fort ruin's lever, the lock in `WarehouseLock`, the
  trigger at gaze, Vilverin's switch stones, a portcullis, a fort switch, an Ayleid wall switch, a
  display case's lid and cell changes were each checked once with the emulated headset.
