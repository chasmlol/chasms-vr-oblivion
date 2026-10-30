# Playtest: doors, levers and the lock by hand (overnight feature 4)

Save: rustvr-doors

`tools\play.ps1 world` boots this branch's build into **rustvr-doors**: Bethesda's own test cell
`TestTraps`. A row of doors that swing in place is on your right: turn to face them and a big
planked cave door is right in front of you, with a double cave door, a barred prison cell gate and
an Imperial dungeon door along the row. The fort ruin's lever is about 16 m straight ahead (where you
face when the game loads), and there are 20 lockpicks in the pack. `-At WarehouseLock` (with
`-Give "0000000A 20"`) is Bethesda's lock test cell: rows of locked fence gates. Real places with
doors and levers that work by hand: the Imperial Dungeon (`-At ImperialDungeon01`, the cell gates),
Fort Ash (`-At FortAsh02`, its turning switches), Chorrol's Mages Guild (`-At ChorrolMagesGuild`, its
locked display cases: a lid on a hinge), and Bethesda's test halls `WarehouseFortRuinsHALLrooms` (a
portcullis, a turning switch) and `WarehouseAyleidRooms03` (Ayleid wall switches).

## What to try, in order

1. **Push the cave door open.** Walk up to the planked door ahead (it's big and heavy, about
   130 kg) and put your palm on it, then push. It gives a hair at first (its latch: it rattles in its
   frame), and pushed to the end of that, the game opens it (its own Activate: its open state, its
   script if it has one). From there your hand swings it. Shove it and let go: it swings on and slows
   to a stop over a couple of seconds.
2. **Take it by any part of it.** Grip anywhere on the leaf (no handle to hunt for): the fingers
   close where they are and nothing jumps. Pull and push it about; near the hinge it barely moves,
   at its edge it swings (the leverage is your arm's, capped at about 180 N). Walk into its swing: it
   stops on your body rather than pushing through you. Pull your hand more than half a metre from
   where you hold it and the hand lets go and glides back to the controller.
3. **Shut it.** Swing it back into its frame: at the frame the game closes it (its Activate
   again, its close sound) and it latches. Pull on it again to open it.
4. **The prison gate and the dungeon door** to your right behave the same way (the gate's two leaves
   each on their own hinge; the game gives the cell gates a mass of 100 kg each).
5. **Open a door the old way.** Look at a door and pull the empty hand's trigger (the game's
   Activate at gaze): the game plays its own opening and your hand meets the door where it's drawn as
   it swings.
6. **The lever.** Walk about 16 m ahead from where you started, past the trap doors: the fort ruin's
   floor lever stands by a rock. Grip its handle and push it over. It holds at each end (its two positions); at the end of its travel the game's
   Activate fires (its script; in this cell it works the secret stone door nearby).
7. **The lock.** `play.ps1 world -At WarehouseLock -Give "0000000A 20"`. Push or pull one of the
   locked gates: the game's own lock opens (the same menu as on a flat screen, as a panel). Your right
   hand is the pick:
   - move it left or right: the game's pick slides under the next tumbler (2.5 cm a tumbler);
   - move it up quickly: the pick pushes the tumbler up (you feel it riding up);
   - pull the trigger as it reaches the top: it sets (a click in the hand); too early or too late
     and the game breaks a pick (a jolt);
   - A is Auto Attempt, B closes the lock.
   Pick all five and the gate opens: the game swings it, and your hand meets it.
8. **Ayleid switch stones.** `play.ps1 world -Save rustvr-ruin -At Vilverin03`: in its halls, the
   switch stones stacked on pillars (glowing gems) push into the pillar with a palm, 25 cm; at the
   bottom of the push the game's Activate (its script, its stone's grinding), and they come back out
   as slowly as the game's own animation brings them. The Ayleid wall switches (the framed stone with
   a blue gem, in almost every Ayleid ruin) push into the wall the same way.
9. **A fort's turning switch.** `play.ps1 world -Save rustvr-ruin -At FortAsh02` (or the test hall
   `WarehouseFortRuinsHALLrooms`): the iron T-bar on a stone base. Take one end of the bar and walk it
   round a quarter turn (it's about 55 kg of iron); at the end the game's Activate (after its two
   seconds the switch's script works what it's wired to). It holds at either end; turn it back to
   work it again.
10. **A portcullis or a ruin's stone door**: take it in a hand and the game raises it (or drops it
   into the floor) with its own animation; the hand lets go as it goes.
11. **Chorrol's houses** (`-At ChorrolNorthernGoodsAndTrade`, `-At ChorrolFireAndSteel`) and the
   Gottshaw Inn (`-At GottshawInnTavern`): ordinary house doors, wood on iron hinges.
12. **A door the game's scripts keep shut, or a locked door without picks**, rattles in its frame and
   the game says why ("locked", the rattle sound), and opens nothing.

## What was checked in the game, and what wasn't

Checked, each once, with the emulated headset (the logs and captures are the evidence):

- The cave door (`TestTraps`): taken by hand, pushed open (the game's Activate: open state 3 -> 2,
  its animation stopped where it began), pulled shut (1 -> 4, latched); pressed open with an empty
  palm; looked at from a step back, open.
- The fort ruin's lever: taken by its handle, pushed over through 1.27 of its 1.40 rad, the game's
  Activate at 88% of its travel (its script ran; its animation stopped where the hand had it).
- The lock (`WarehouseLock`, Security 9): a locked gate pushed, its lock opened; the pick moved under
  each of the five tumblers in turn; tumblers pushed up; one set by the trigger; picks broken by
  mistimed ones; after Auto Attempt opened the lock, the gate swung open by the game with the
  runtime following it.
- The trigger at gaze on the cave door: the game's own Activate and animation, the runtime's leaf
  following it open (and saying when the game stopped moving it).
- Vilverin's Ayleid switch stones: one pushed in with a palm (0 to 0.23 of its 0.25 m), the game's
  Activate at 88% (its script ran, its sound played), springing back out when let go; seen pushed in
  on its pillar.
- A portcullis (`WarehouseFortRuinsHALLrooms`, the fort ruins' test hall): taken in a hand, the
  game's Activate (open state 3 -> 2), the game raised it its 2.86 m with the runtime's leaf following
  it, the hand letting go as it went up.
- A fort's turning switch (the same hall): taken by its bar's end, walked round 1.43 of its 1.57 rad,
  the game's Activate at 88%; the script's own turn, which it plays two seconds later, stopped (the
  bar stayed where the hand left it); seen turned.
- An Ayleid wall switch (`WarehouseAyleidRooms03`): pushed in with a palm (0.20 of its 0.22 m, at
  0.15 m/s), the game's Activate and its sound, sprung back out over about 2.5 s; seen pushed in.
- A display case's lid (`WarehouseUpperClass`, its unlocked cases): taken by its front edge and
  lifted, the game's Activate (open state 3 -> 2, its sound), lifted to 0.96 of its 1.34 rad, let go:
  it fell shut under its own weight and the game closed it at the frame (1 -> 4, its close sound).
  Not seen (that cell is too dark for the captures); its stay holding it fully open not tried.
- Cell changes (Chorrol's guilds, back and forth, then `TestTraps`): the world's parts dropped
  before the cell went and offered again in the new one, no crash; the double cave door's two leaves
  each on their own hinge.
- In the framework (a quick temporary check, not committed): a door's leaf taken and pushed, pressed
  with a palm, stopped by the player's body, latched and let go.

Not checked in the game:

- Following an NPC walking through a door (the same following as the trigger's).
- A trap door (a level hinge the same as the display case's lid).
- A ruin's stone door (it slides down into the floor, the same as the portcullis but the other way).
- The door sounds: they're the game's own (the animation's text keys, else the door form's sounds),
  played through the game's sound system; not listened to (the emulated headset has no ears).
- Haptics: sent, not felt.
- Any of it on the real headset.

## Decisions made

- **Doors are latched while the game has them shut**, and a tug (grip or palm) to the end of the
  latch's play (2 degrees) is the game's own Activate, as the player's Activate control would be.
  So locks, keys, scripts and the game's open state are always the game's. A palm push opens an
  unlocked door (players love push-to-open; real latches would want the handle turned).
- **The game's door animation is stopped** the moment the hand opens it (the hand is the animation),
  and the door's node, its handles (posed by the game's animation at the matching moment) and its
  keyframed Havok body are put where the runtime has the leaf every frame.
- **Mass:** the game gives most doors no mass (their bodies are keyframed). A leaf weighs its drawn
  area as a 4.5 cm plank of its Havok material (a house door about 65 kg, the big cave door 130),
  or, of metal, as heavy for its size as the game's own cell gates, whose 100 kg is the model's;
  levers and stones weigh their drawn shape's volume of their material; a lid (a level hinge) never
  more than half an arm lifts by its edge (a display case's about 19 kg). Hinge friction is its pins' (a friction of 0.3 on 1 cm pins under its weight). A
  swing dies away over about 2 s once let go, and a lever's two positions hold against 25 N at its end:
  those two are design values, not the game's.
- **Sliding doors are the game's to move:** a hand can't lift a stone slab, so a tug opens it with
  the game's own animation.
- **The lock is the game's lock menu, worked by the hand:** the floating-menus feature (another
  branch) owns how the menu's picture floats; here the hand drives the game's pick (its cursor and
  its mouse) instead of the laser, so every rule (tumbler speeds, Security windows, broken picks)
  stays the game's. The pick isn't drawn in the hand: the game's lock draws its own pick, under
  its tumblers, and a second pick in the hand, away from the lock it works, would touch nothing.
  It belongs with the lock drawn at the door's keyhole (after the floating menus land).
- **Activators** are levers, switches, buttons, chains and wheels whose model moves a part along its
  own "Forward" animation, with a script; traps and their triggers (models named or filed as traps or
  triggers, pressure plates) and big parts (more than 1.6 m, or slides past 0.9 m: gates and bridges
  levers work) are left alone. A part's shape for the hand is its Havok body's, or what's drawn of it
  where it has none (a turning switch's bar) or where its body lies off its drawing (an Ayleid wall
  switch's box sits half a metre out from the wall).
- **A script's own animation in answer to the hand's Activate is stopped** for 3 s after it (the
  fort switches' scripts play their turn two seconds later): the hand already moved it.
- **Buttons and switch stones** push in against a firm spring, 60 N all the way in, however heavy the
  stone (a wall switch's is about 460 kg of drawn stone), and come back no faster than the game's
  animation brings them: a design value, the game has none.
- The break distance for a hand on a hinge or slide is 50 cm (the rig's empty-hand leash is 70).

## Not done

- **The lock drawn at the door's keyhole, the pick in the hand in it** (the game's
  `lockpicking\*.nif` models posed by the menu's state), rather than as the menu's panel. It needs a
  way for the runtime to put a thing into a hand that didn't take it (the framework has none yet).
- A few older models whose animations go through blocks the model reader doesn't read yet (particle
  and material controllers ahead of the animation): they stay the game's Activate.
- **Drawers, chests' lids, buttons and pull chains** in vanilla: Oblivion's containers have no lid
  animations (the 538 animated containers are hanging corpses), and its buttons and chains are mostly
  traps' or the Shivering Isles'; buttons that slide in (the Shivering Isles' ruins) should work as
  slides with a spring back, unchecked.
- **Harvesting plants by pulling, sitting by lowering onto a chair, sleeping**: listed in
  `docs/research/world.md`, not built.
