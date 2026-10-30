# Playtest: pickpocketing by hand

Save: rustvr-pocket

`playtest.bat` in this worktree (double-clicked in Explorer: not the app's terminal, where Steam
looks signed out) boots it with `tools\play.ps1 pickpocket`; `playtest-sneak100.bat` does the same
with the console's `player.setav sneak 100` (an 85% chance). It's the Merchants Inn in the Imperial
City's Market District, at noon, at the door. Bruce (a test NPC with no AI packages, so he stands still) is about a metre to
your **left**, his back to you. He carries 120 gold, a key, 3 lockpicks, a potion, two apples and a
note. The inn's own patrons walk about further in. The research and the design are in
`docs/research/pickpocket.md`.

## What to try, in order

1. **Sneak** (right stick click, or duck). Pockets only exist while you sneak, and only on people
   within 3 m who are alive and not fighting: whom the game would let you pickpocket.
2. **Find a pocket by touch.** Turn to Bruce and put a hand at his right hip, where a belt pouch is.
   You feel a soft brush going in, then a tick for the thing your fingers are on. Move the hand
   slowly inside: every 4 cm you're on the next thing, with another tick. Each kind of thing feels
   different: coins jingle (bright), a key clicks (sharp), glass rings (very short, high), paper
   rasps, cloth or food is a dull push, something hard knocks. An empty pocket is one dull brush.
   His pockets:
   - the right belt pouch: the potion and the apples;
   - the left belt pouch: the key and the lockpicks;
   - the purse, at the front of his belt: all his gold;
   - the inner pocket, at the left of his chest: the note;
   - a bag on his back, only when someone carries something big (weapons, armour, clothes).
3. **Take one.** Squeeze the grip: the thing (the game's own model) is in your hand, with the game's
   own card over it (its name, value and weight). It's still his while your hand is in the pocket:
   let go there and it's back, nothing tried.
4. **Pull it out.** The game decides as it leaves the pocket, with its own chance: 40 + 0.6 x (your
   Sneak - his), both with Luck, minus a tenth per gold of value taken, between 5 and 85%. This
   character's Sneak makes it about 40% against Bruce: expect to be caught (the console's
   `player.setav sneak 100` makes it 85%, the most the game allows).
   - **Success:** it's yours (marked stolen). Put it at a hip or a shoulder to keep it.
   - **Caught:** a hard jolt and the thing is gone from your hand. Then the game's own reaction:
     the pickpocketing crime (bounty 25), his disposition, his alarm or attack, and he can never be
     robbed again (his pockets feel shut, and the game says so).
5. **The purse** at the front of his belt is all his gold at once: 120 coins cost 12 points of
   chance. Gold is drawn as the game draws gold, one coin, so it hides in the fist; the card says
   how much ("Gold (120)").
6. **Plant something.** Hold something weightless you took (the purse, the key, a lockpick) and let
   go of it in one of his pockets: it's his. Something with weight (the potion) gets the game's own
   "You cannot place items in a container while pickpocketing." and stays yours. This is the game's
   own rule: Oblivion has no chance for planting, only that rule.
7. **Rummage and come away empty.** A hand in and out with nothing taken is the game's own "looked
   and took nothing": a use of Sneak, and the game's chance (with nothing taken) that he noticed. It
   happens once the hands have been away from his pockets for 3/4 s.
8. **Drop or throw a taken thing** away from your body and anyone's pockets: it's the game's own
   thing in the world where you let go, flying on as it left your hand (Havok), to pick up again.
9. **The patrons.** Walk in (sneaking) and try the people moving about. Their pockets hold what
   they carry but don't wear (often nothing: an empty pocket feels empty at once). Follow a walking
   one: the pockets move with them.
10. **Watch the sneak eye** (at your gaze; the flat game's crosshair is hidden in VR, its activate
    icons stay). While a hand is in the pocket of someone who sees you, you feel a slow heartbeat
    in that hand (a strong beat as they first notice you). Being seen doesn't change the per-item
    chance (the game's doesn't), but when you're done with someone, everyone who sees you then
    witnesses the theft (the game's own check): the crime and the guards.

The game's flat pickpocket menu is still there: the trigger on someone while sneaking opens it. A
hand in a pocket doesn't activate anything with its trigger.

## What was checked in the game

With the emulated headset, from the logs and captures:

- **Pockets:** an NPC's pockets from its bones, filled with exactly the game's pickpocketing list
  (gold in the purse, the key in the left pouch, potion and apples in the right, the note inside the
  coat); a patron with nothing unworn gets empty pockets; pockets appear as the player sneaks.
- **Taking:** into a pouch, a thing felt, the grip closing on it (the runtime's object in the hand, the
  game's model drawn: the potion, the key), pulled out, the game's chance against its roll; taken
  (the potion, the key, all 50 gold of a purse) and kept in a body slot.
- **Caught:** the game's chance 5 against its roll (Bruce's Sneak 100, the player's 0) and on a
  rummage: the bounty went 0 -> 25 and the NPC shut (never again), in a ruin and at the inn. Placed
  where it doesn't belong (someone's house, the fields at Weye) the same NPC is trespassing and
  doesn't react at all: that's the game's own rule (its handler passes over trespassers).
- **Planting:** gold planted back (taken); the potion refused (weight 0.5) with the game's message.
- **Rummaging:** in and out empty: the game's chance against its roll, no catch.
- **Dropping:** a taken key let go in the air is the game's own key lying on the floor (items.rs
  offers it again).
- **The game's card** over a thing out of a pocket (Apple; Gold (50): the game's own font and icons,
  with the count). The card's piece of the HUD now has room to its left: a name that grew after the
  card was measured was cut off ("ld (50)").
- **The game's older models** (10.2.0.0: the common key) and Havok triangle collision blocks are read
  now: the key and the lockpick are drawn; 2,117 of 2,216 models of things to take read whole (was
  1,334). Re-exported into `local_assets`: clutter, sigil stones, dungeon clutter, armour, clothes,
  weapons.
- **NPCs' bones:** creatures' capsules were only their arms (the bone walk ran into non-nodes);
  now all 10 (torso, head, legs too). This also fixes combat's capsules on NPCs.

Not checked:

- Anything in the real headset: how the ticks, the brush and the heartbeat feel; whether 4 cm per
  thing and the pocket sizes (9-11 cm) are right for a real hand; reaching around someone to their
  purse.
- The heartbeat itself (only the awareness values: 0, 0.5 noticed, 1 seen, read from the NPC's own
  detection list).
- The game's success message and sound (the call is the menu's own, with its message flag).
- The "Items Pickpocketed" statistic going up (it's written as the menu does).
- Sleeping, sitting and walking NPCs, and crowds (performance with many people near).

## Decisions

- **No chance of our own.** Oblivion's per-thing chance has no term for a steady hand or
  for being seen. Nothing was added: a careful hand matters by finding the right thing (each move
  ticks to the next) and by not rummaging (coming away empty is the game's own check); being seen
  matters at the close, where the game makes it matter (its witness check).
- **The purse is all the gold.** The game's menu asks how many; a hand can't say. Other stacks come
  out one at a time.
- **The refusal is a corner message, not a box.** The game shows `sInvalidPickpocket` in a message
  box; a box in the middle of a reach would take over the view.
- **The session is the menu.** Opened as a hand goes into someone's pockets (empty, or bringing
  something to plant), closed 0.75 s after the hands are away from them all.
- **Pockets are where clothes would have them**, the same on everyone (Oblivion's clothes have no
  pockets of their own): hips, belt front, inside the coat, a bag on the back only when there's
  something big. Shared out by kind (`pocket_for`).
- **Planting a thing lying about** (a thing to take, not yet yours) first takes it with the game's
  own pick-up, then plants it, if it's weightless; otherwise it falls out of the pocket.
- **Templates and a card reference.** Each kind of thing is a template object drawn from the game's
  model; its card is the game's own, given a reference of the game's own making that's never placed
  (as xOBSE's inventory references).

## Not done, known issues

- Gold is one coin (the game's only gold model): it hides in the fist.
- Older models in 10.0.1.x and 10.1.0.106 (about 30 of the clutter, candles and rugs mostly) still
  aren't read; a thing without a model is invisible in the hand (its card still shows).
- Creatures have no pockets (they're NPCs only, as the game's pickpocketing is in practice).
- Turning the head 90 degrees left puts the shield worn over the left shoulder in front of the eyes
  (not this branch's).

## Meeting the other branches

- **Grabbing NPCs (`overnight/npcs`):** pockets exist only while the player sneaks. A hand in one
  (`Rig::in_pocket(hand)`, its creature and pocket) takes from the pocket: the rig's hover order
  puts pockets before anything else near the hand, and the npcs branch's grab of a creature's part
  should stand aside while `in_pocket` is `Some` for that hand and creature. A hand in a pocket
  pushes nothing of its creature (`actor_pushes` skips it), so the npcs branch's body won't be
  shoved by a thief's hand. Not sneaking: no pockets, every grip on an NPC is a grab.
- **Floating menus (`overnight/menus`):** the sneak eye is a floating HUD piece there; it reads best
  over the hand that's in a pocket (`Rig::in_pocket` says which), or at the pocket's creature.
- **Holsters (`overnight/holsters`):** things drawn from the inventory are the player's, so they
  should plant like a taken thing: make the rig's `plantable` include them (it's `OBJF_LOOT` or a
  taken pocket copy now) and resolve their form in `pockets::plant` (it looks up taken copies, then
  things lying about).
- **The protocol:** this branch is version 12 (`Local\RustVR_v12`), as the others are: on merging,
  bump once past all of them. `OFF_POCKETS` is 0x51_8000, clear of npcs' 0x51_0000 region and the HUD
  layout at the end.

## Tools for checking

- The adapter's log: `pockets: N people near with pockets: <who>: <thing> xN in <pocket>`, every
  attempt with the game's chance and roll, the bounty as it changes, and every two seconds where each
  pocket is from the eye (right, up, forward).
- The runtime's log: hands into pockets, things closed on, pulled out and the game's answer, and
  every two seconds how far each hand is from the nearest pocket.
- Throwaway emulated-headset scripts were under `<builds>\pickpocket\scripts` (not committed).
