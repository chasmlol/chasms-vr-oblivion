# The physical world: doors, levers and everything the hands can work

Research for working the world by hand: Oblivion's doors that swing in place, its levers, pull
chains, buttons, wheels and gates, its lockpicking, and a pass over everything in `Oblivion.esm`
that could be a VR interaction. Inspiration only: nothing below is copied.

## 1. What players love and hate

From reviews, Steam threads, developer talks and the toolkits' own documentation (sources at the
end; Nexus and Reddit pages were only readable as search snippets).

**Loved**

- **Doors you can work any way.** Valve's door work for Half-Life: Alyx (Kerry Davis, DigiPen
  2019): grab the handle or push the door with either hand, it swings on its hinge; a latching door
  stops when it comes back into its frame, which reads as "shut"; a lever handle turns before the
  door moves; its creak and slam follow how far and how fast it moves. Resident Evil 4 VR rebuilt
  every door as physical: open one a crack to peek through.
- **Rummaging without prompts.** Alyx's reviewers singled out opening cupboards and drawers,
  taps, switches and toilet seats with no UI at all; players kept trying things Valve hadn't
  planned for.
- **Tools touch the world too.** In Alyx a held weapon can push a door open, the same as a hand.
- **Grab-and-pull activators in Skyrim VR.** Interactive Activators VR (pull chains, levers, pull
  bars, buttons, valves, keyholes) and Interactive Door Actions VR (push unlocked doors open by
  hand; touch a key to a door) are popular exactly because they replace point-and-click.
- **Lockpicking by feel.** Thief VR (2025): twist the picks and feel for the tumbler; with the
  helper circle off, purely by haptics, the reviewer called it a game-changer. Oblivion's own lock
  gives clearer feedback than Skyrim's (a click at the top of each tumbler, pitch by its speed).

**Hated**

- **Having to hit the exact handle** (the top complaint): Thief VR's single grab points, old VR
  doors that only opened by the knob, Paranormal Activity VR's levers that snapped the hand to them
  while the real hand was elsewhere, Into the Radius's fiddly cabinets.
- **Doors and the body fighting.** Boneworks doors catching the shoulders, bumped shut by
  accident, hard to step round; a held door pulled into the player's own collision goes erratic
  (Meta's guidance); Gone Home's low doors and bottom drawers launching the player. Fix in every
  case: a door never pushes the player, it stops on them.
- **Things flying off.** Into the Radius's objects shooting off; Skyrim VR levers coming off
  their hinges; pull-chain resistance changing with scale.
- **Scripted movement ignoring physics** (Alyx's garage door pushing a crate through the floor).
- **Hands stuck or sluggish** behind physics (Boneworks reviews). Valve's Jason Mitchell: there's
  no force feedback, so heavy rigid physics never reaches the hand; don't lean on it.
- **Tedious tugging** (Arizona Sunshine's slow handles, which reviewers skipped with a shortcut).
- **Lockpicking:** random split-second timing, tools hard to get out, loose input. Skyrim VR's
  vanilla lock on the sticks was called finicky; players had followers pick locks.

## 2. How the good ones make hinges and slides feel right

- **The hand holds the part where it took it, by a force-capped spring.** Torque is force times
  distance from the hinge, so leverage comes for free: pulling near the hinge is hard, at the edge
  easy. HurricaneVR, VRExpansion and HIGGS all cap the hand's force (HIGGS: 2000 N linear, a force
  to mass ratio of 500).
- **Mass, damping, friction.** Viscous damping for "heavy" (HurricaneVR's spring to damper about
  10:1), plus a small constant (Coulomb) friction so a let-go door or lever coasts to a stop
  (VRExpansion's levers 5 degrees/s, SteamVR's sliders a 5/s decay). Doors need real mass (10 kg+
  in HurricaneVR, or the joint stretches). The real-world reference: an interior door takes about
  22 N at its handle (ADA), most of it the latch and closer.
- **Latch.** A door is kinematic (or held) while latched, until its handle turns past a threshold
  (HurricaneVR) or the game unlocks it (Meta).
- **Breaking the hold.** Constrained grips let go much sooner than free ones: SteamVR's circular
  drive warns (jitter, haptics) 10 cm past its stop and lets go at 20 cm; Godot XR's handles at
  30 cm; HIGGS lets a free object go at 0.7 m (averaged, ignored for 0.2 s after the grab).
- **Levers** toggle at about 80% of their travel with a dead band so they don't flicker
  (VRExpansion, Unity XRI); spring-loaded ones return at about 50 degrees/s; momentum on release
  from the last few frames' speed.
- **Knobs and wheels**: within about 10 cm of the axle the wrist's twist turns them, further out
  the hand's place round the axle (Unity XRI's knob, and this framework's `rig::valve`).
- **Buttons** need part of their travel before they fire, and only hands press them (Meta).
- **Drawers**: a slide with hard limits; never zero mass (Leadwerks).
- **Haptics and sound**: contact pulses above 0.2 m/s (HIGGS); a tick at stops and latches, a
  burst when the hold breaks (SteamVR); creak and slam scaled with speed (Valve).

## 3. Oblivion's doors, activators and locks

### Doors

- **A door** (`TESObjectDOOR`) has its model, its sounds (open, close, loop) and flags (Oblivion
  gate, automatic, hidden, minimal use). A reference that leads elsewhere (`ExtraTeleport`) is a
  **load door**; one may have a lock (`ExtraLock`): its level, its key and whether it's locked.
- **Its open state** is saved with the game. `GetOpenState` returns 1 open, 2 opening, 3 closed,
  4 closing: "opening" and "closing" while the door's "Open" or "Close" sequence is playing.
- **Activating it** runs the reference's `OnActivate` script if it has one, or else the door's own
  activation: a lock without the key opens the lockpick menu when the player has picks, or says the
  door is locked and plays `DRSLocked`; the key opens it ("opened with"); a load door teleports; a
  door that swings toggles open or shut and plays its model's "Open" or "Close" sequence.
- **The door's own model says how it moves.** The "Open" sequence turns the leaf about its own
  node's origin: `CDoor02`'s leaf 100 degrees about +Z, the Anvil door 103 degrees, the prison
  cell's gate 105; a ruin's door slides 3.2 m down; double doors turn two leaves opposite ways.
  The same sequence turns the handles a few degrees as it unlatches (the Anvil door's knobs 2.5
  degrees).
- **Its sounds are in its animation**: text keys "Sound: DRSMetalOpen01" just after "Open"'s
  start, "Sound: DRSMetalClose01" where "Close" swings shut (1.43 s of 2.2), "Enum: StopSound" at
  the end. The game plays them as the sequence passes the keys.
- **Its collision** is a keyframed Havok body (motion type 6, layer 2 "animated static") on the
  leaf's node, which follows the node; it has no mass (0). Its Havok material (wood, metal, stone)
  is on its shape.
- **`Oblivion.esm`:** 501 door forms, 6949 references: 4397 load doors, **2552 that swing (or
  slide) in place**; 1319 locked. Of the swinging ones about 2000 have an "Open"/"Close" animation
  (153 forms); 121 have none.

### Activators

- An activator (`TESObjectACTI`) is a model and a script. Levers, buttons, switches, pull
  chains and wheels animate with "Forward" and "Backward" sequences their scripts start
  (`PlayGroup Forward 1`) in their `OnActivate` block; their sounds are text keys too
  ("Sound: DRSRootSwitch01").
- **Kinds found** (the moving node and its travel, from the models' own sequences): the fort's
  switch (`rfswitch01`: its handle 80 degrees about X), its turning switch (`rfswitch03`: 90
  degrees about Z), the castle's secret lever (slides 34 units), the ruins' face buttons (slide 4
  units in), pressure plates (slide 4 units down), the sewer's wheel (215 degrees, its chains
  slide with it), Kvatch's lever opener (handle 40 degrees, gears and chains with it), the fort's
  drawbridge (75 degrees about Y), the Ayleid gates (slide 187 units down: worked by a lever, not
  by hand), the Shivering Isles' Dementia and Mania levers (pulled and back within one "Forward").
- **`Oblivion.esm`:** 1252 activator forms, 12767 references, 4282 of them animated: 1236 with
  "Forward" and "Backward" (two-way levers, gates, buttons that come back), 1471 with "Forward"
  only (traps, plates, one-shot switches).

### Lockpicking

- The lock is `LockPickMenu` (`menus\lockpick_menu.xml`, 3D models `lockpicking\pick.nif`,
  `skeletonkeypick.nif`): five tumblers, each with its speed, set or not, and the tumbler the pick
  is under. A click on a tumbler sets it if it's at the top (the "UILockTumblerLock" sound), else
  it's an attempt that breaks the pick ("UILockPickBreak"). Auto-attempt is the other way to try.
  The game settings `fLockPickQualityBase/Mult`, `fLockPickBreakBase/Mult`, `fLockPickAutoBase/Offset/
  Difficulty` set the windows and the chances from the Security skill and the lock level.
- UESP: the window is 0.1 s to 1 s by the tumbler's speed; a mistimed click breaks a pick; the
  Security perks keep set tumblers up; tumbler speed depends on the frame rate (locks get harder at
  high fps), which matters at VR's.

### Everything else (`Oblivion.esm`, references)

| Kind | Forms | References | What a hand could do |
|---|---|---|---|
| Doors in place | 501 (all doors) | 2552 (of 6949) | grab anywhere, push, pull; locks; latch |
| Load doors | | 4397 | stay the game's Activate |
| Containers | 873 | 17899 (1547 locked) | none has a lid animation in vanilla (538 animated ones are hanging corpses' idles); a hand on one opens it (the game's container) |
| Furniture | 186 | 7356 | chairs, stools, benches, beds, bedrolls: sitting and sleeping |
| Flora (harvested plants) | 155 | 76769 | grab and pull to harvest (the game's own harvest, its chance of nothing, its regrowth) |
| Activators | 1252 | 12767 | levers, switches, buttons, wheels, pull chains; also fires, crystals, triggers |
| Books | 887 | 8791 | reading in the hand |
| Lights | 1625 | 24631 | carriable ones are already items; fixed torches and candles |
| Clutter, ingredients, potions... | | 60000+ | already items ([OBLIVION.md](../OBLIVION.md)) |

## 4. Design

### Framework: the jointed part, for any game

The rig already has `OBJ_VALVE` (`rig::valve`): a part turned on an axle fixed in the world, held
where the hand took it, cranked by the arm's force-capped spring at that point (`ARM_STIFFNESS`,
`ARM_FORCE`: the leverage) or twisted by the wrist, with its own inertia, friction, range and
clicks. It's simulated exactly (one degree of freedom), not as a Rapier joint, so it never comes off
its hinge or flies off. It grows into everything that moves on a hinge or a slide:

- **`JOINT_SLIDE`**: along its +Z instead of about it (drawers, buttons, pull chains, sliding
  doors, portcullises); its state's `slide` in metres.
- **`JOINT_PANEL`**: a part that swings or slides through the room, a door's leaf or a lid. Its
  collider is solid to the empty hands (they rest on it as on a wall), and a hand pressed on it
  pushes it: the arm's spring at the contact, pushing only. The player's body and the game's
  creatures stop it (it never pushes the player, as every good VR door does).
- **`JOINT_GRAVITY`**: its own weight turns it (lids, trap doors, drawbridges), from its
  collider's centre of mass.
- **Spring and damping** (`joint_spring`, `joint_damping`): what pulls it back to where it rests
  (a button, a pull chain, a spring-loaded lever) and what slows it as it moves (viscous), beside
  the friction that stops it.
- **Followed when the game moves it** (`OBJF_GAME_MOVED`: a door an NPC opens, a gate a script
  lets down): it takes the game's pose whenever no hand is turning it.
- **Taken anywhere on it** when the runtime doesn't draw it (a game's door): its collider is what
  the hand takes, the hold where the palm met it. Its inertia about the axle from the collider's
  own mass properties (a door is a slab turned about one edge, not a ring).
- **Let go when pulled away**: a hand held on it lets go once its controller is 30 cm off
  (`rig::valve::BREAK`; research: 20-30 cm for constrained grips), and glides back to the
  controller.
- **Feel**: a haptic tick when it hits a stop, scaled by its speed.
- The game changes its range and stiffness as it goes (a latch, a lock): the rig takes them from
  the table whenever it changes, keeping where it's turned to.

### Oblivion

- **`world.rs`** (its own owner in the object table, `Owner::World`): every door near the player
  that swings or slides in place (no `ExtraTeleport`, not an Oblivion gate), and every activator
  whose model has a part its "Forward" animation moves, is offered as jointed parts, one per part
  that moves with its own collision (a door's leaf; both leaves of a double door; a lever's
  handle).
- **The joint from the game's own model**: read from the game's archives at run time (the model
  the form names), its "Open" (doors) or "Forward" (activators) sequence: the part's node, the
  rotation's axis through its origin and the angle, or the slide; the sounds in its text keys.
- **Mass from the game's own data**: Oblivion gives doors no mass (keyframed), so the leaf's mass
  is its Havok shape's volume times its Havok material's density (wood, metal, stone); the hinge's
  friction is its pins' (the weight on them, a friction of 0.3 on a 1 cm pin). No hand-tuned feel.
- **The latch is the game's door.** While the game has it shut, the leaf moves only within a hair
  of its frame (a rattle). A hand tugging it to the latch's end is the game's Activate, as if the
  player had pressed Activate on it: its script, its lock and key, its open state. If the game opens
  it, the latch lets go and the hand swings it on from there; the game's own animation is stopped
  (the hand is the animation now), and its node, its handles (posed by the game's animation at the
  matching moment) and its keyframed Havok body are put where the runtime has the leaf every frame,
  so the player and the NPCs meet it where it's drawn. The door's own open sound (from its
  animation) plays through the game's sound system as it leaves the frame. Swung back into its
  frame, the game's Activate again (closing, its script), its close sound, latched.
- **Locked**: the latch holds; tugging is still the game's Activate: the key opens it, picks
  start the lock (below), otherwise the game's "locked" message and its rattle.
- **The game moving it** (the trigger's Activate at gaze, an NPC, a script): the game's animation
  plays and the runtime's leaf follows the game's node.
- **Activators**: the lever, button or chain moves by hand along its own "Forward" travel; at its
  end, the game's Activate (the script does what it does, `PlayGroup` and all; the adapter keeps
  the part where the hand has it). One whose "Forward" comes back to where it began (a pull chain,
  a button) springs back when let go; a two-way lever stays, and pushing it back to its start is
  the game's Activate again (its "Backward").
- **Lockpicking in the hand** (`lock.rs`): the game's own `LockPickMenu` runs and decides
  everything (its tumblers, their speeds from the lock level, the Security skill's windows, picks
  broken and used up, auto-attempt), and the game draws it. The hand works its pick: moved
  sideways from where it was when the lock opened, a tumbler every few centimetres (the game's
  cursor walked along the row until the game puts its pick under that one),
  moved up to push the tumbler up, the pick hand's trigger to set it as the game's click does. A
  tumbler riding up ticks in the hand, a set one clicks, a broken pick jolts. A is the menu's Auto
  Attempt, B closes it. Planned, not built: hiding the flat picture and drawing the pick at the
  keyhole.
- **Later** (written down, not built): harvesting plants by pulling (FLOR: the game's own harvest),
  sitting by lowering onto a chair, containers opened by a hand on their lid. Books are read in the
  hand now (`books.rs`).

## 5. What was made physical, and what's left

The full pass over `Oblivion.esm` (references counted in the ESM; models by
`oblivion-extract anims meshes`):

| Kind | References | Now | Left |
|---|---|---|---|
| Doors that swing in place (in buildings, ruins, caves, cells, gates, cupboards) | about 2000 with an "Open" animation (153 models) | worked by hand: latched, the game's Activate at the latch, swung by the hand, shut and latched again; locked ones open the game's lock | the 121 without an animation stay the game's Activate; two-leaf models whose leaves the reader can't find by name |
| Doors that slide (Ayleid and ruin stone doors, portcullises) | about 170 | a tug is the game's Activate; the game's animation moves it, followed | lifting one by hand (too heavy to mean anything) |
| Load doors | 4397 | the game's Activate, unchanged | |
| Levers, switches, wheels, buttons with a moving part and a script | the fort ruins' switches, the castles' secret levers, the sewer wheel, Kvatch's lever, the Shivering Isles' buttons and levers | worked by hand along their own Forward travel, the game's Activate at its end (its script's own animation in answer stopped); two-way levers hold at their ends, spring-back ones (buttons, chains, the Ayleid switch stones and wall switches) push in against a firm spring and come back at their animation's pace; a part with no Havok body of its own (the fort's turning switch) or whose body lies off its drawing (the Ayleid wall switch) is taken by its drawn shape | traps and their triggers (pressure plates, tripwires, trigger zones) stay the game's; a few models whose animations sit behind blocks the reader doesn't read |
| The lock | 1319 locked doors, 1547 locked containers | the game's lock worked by the hand (pick under a tumbler, pushed up, set by the trigger; `lock.rs`) | the pick drawn at the keyhole; containers' locks open the same lock (unchecked) |
| Containers | 17899 | the game's Activate (the floating menus' container list) | no vanilla container has a lid animation; a lid by hand needs a model of our own, which the rules forbid |
| Display cases | about 80 | their lids are doors on a level hinge: worked by hand, falling shut under their weight (unchecked) | |
| Flora (harvested plants) | 76769 | the game's Activate | grab and pull to harvest (the game's own harvest, its chance of nothing, its regrowth) |
| Furniture (chairs, benches, beds, bedrolls) | 7356 | the game's Activate | sitting by lowering onto it, sleeping by lying down |
| Books | 8791 | taken in the hand and read there, the game's book menu floating over it (`books.rs`) | |
| Things to take (clutter, food, potions, weapons, armour...) | 60000+ | taken in the hand, thrown, stowed (items) | |
| Loose bodies the game's physics swings (hanging chains and chain dolls, ropes, a bucket or skulls on a rope, a sprung trap's chain) | | solid to the hands, pushed and held with the game's own grab on the game's constraints (`props.rs`) | |
| Lights on walls, fires, crystals | 24631 / 753 / 1257 | | a torch taken off its bracket (not built) |

## Sources

- Valve's doors: https://uploadvr.com/valve-vr-game-doors/ ; Alyx's details:
  https://www.roadtovr.com/these-details-make-half-life-alyx-unlike-any-other-vr-game-inside-xr-design/2/ ;
  https://www.roadtovr.com/half-life-alyx-review/ ;
  https://gameinformer.com/interview/2020/03/23/valve-talks-half-life-alyx-and-why-arms-dont-work-in-vr ;
  https://uploadvr.com/half-life-alyx-boneworks-physics/
- Resident Evil 4 VR: https://developers.meta.com/horizon/blog/rebuilding-resident-evil-4-what-it-took-to-bring-capcoms-classic-to-oculus-quest-2/
- Skyrim VR: https://www.nexusmods.com/skyrimspecialedition/mods/161676 , /142338 , /184347 ,
  /196646 ; HIGGS: https://github.com/adamhynek/higgs
- Complaints: https://roadtovr.com/thief-vr-review-quest-3-psvr-2-steamvr/ ;
  https://gamesbeat.com/how-boneworks-turns-your-body-into-its-key-mechanic/ ;
  https://www.uploadvr.com/into-the-radius-quest-2-review ;
  https://steamcommunity.com/app/467660/discussions/0/1290691308573110401 ;
  https://amebouslabs.medium.com/the-design-of-everyday-vr-things-3e84371baaf2 ;
  https://fullbright.company/2012/07/02/code-judo ;
  https://www.leadwerks.com/community/blogs/entry/2880-interactive-multi-body-physics
- Toolkits: HurricaneVR https://github.com/cloudwalker2020/HurricaneVR-Docs ; VRExpansionPlugin
  https://github.com/mordentral/VRExpansionPlugin ; SteamVR Interaction System
  https://github.com/ValveSoftware/steamvr_unity_plugin ; Unity XRI examples
  https://github.com/Unity-Technologies/XR-Interaction-Toolkit-Examples ; Meta
  https://developers.meta.com/vr/blog/object-interaction-part-1-common-interaction-patterns/ ;
  Godot XR Tools https://github.com/GodotVR/godot-xr-tools
- Lockpicking: https://en.uesp.net/wiki/Oblivion:Security ;
  https://steamcommunity.com/app/2623190/discussions/0/604154245960204827 ;
  https://www.pcgamesn.com/skyrim/mod-skyblivion-lockpicking ;
  https://github.com/Stanislavsonder/SkyrimVR-PhysicalLockpicking ;
  https://gamingtrend.com/reviews/thief-vr-legacy-of-shadow-review-well-since-im-in-here-i-might-as-well-pick-up-something-for-myself/
- Real doors: https://AData.org/sites/adata.org/files/files/Opening%20Doors%20final2019.pdf
