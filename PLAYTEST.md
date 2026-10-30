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
2. **Find a pocket by touch.** Everything anyone carries is in their trouser pockets: one at the
   front and one at the back of each hip. From in front, lay your fingers on the front of Bruce's
   hip; from behind (his back's to you), on the back of it; from the side, on its side (the front
   and the back pockets meet there). Your hand stops on his body; touching it there is being in the
   pocket. You feel a soft brush going in, then a tick for the thing your fingers are on. Move the
   hand slowly over it: every 4 cm you're on the next thing, with another tick. Each kind of thing
   feels different: coins jingle (bright), a key clicks (sharp), glass rings (very short, high),
   paper rasps, cloth or food is a dull push, something hard knocks. An empty pocket is one dull
   brush.
   **You see what you're on:** the game's card of the thing under your fingers floats over your
   hand (its name, value and weight), changing as you move on to the next thing. Over the name, in
   the game's stealing red, is the reticle's red hand and the word **Pickpocket**.
   His pockets (things go where they'd be kept, then are spread out: no pocket has more than a
   quarter of them while another has fewer):
   - the right front pocket: the apples and the potion;
   - the left front pocket: the lockpicks and the key;
   - the right back pocket: all his gold (a purse);
   - the left back pocket: the note.
3. **Take one.** Squeeze the grip: the thing (the game's own model) is in your hand, with the game's
   own card over it (its name, value and weight), still with the red hand and **Pickpocket**. It's
   still his while your hand is in the pocket: let go there and it's back, nothing tried.
4. **Pull it out.** The game decides as it leaves the pocket, with its own chance: 40 + 0.6 x (your
   Sneak - his), both with Luck, minus a tenth per gold of value taken, between 5 and 85%. This
   character's Sneak makes it about 40% against Bruce: expect to be caught (the console's
   `player.setav sneak 100` makes it 85%, the most the game allows).
   - **Success:** it's yours (marked stolen): the red **Pickpocket** goes from its card. Put it at
     a hip or a shoulder to keep it.
   - **Caught:** a hard jolt and the thing is gone from your hand. Then the game's own reaction:
     the pickpocketing crime (bounty 25), his disposition, his alarm or attack, and he can never be
     robbed again (his pockets feel shut, and the game says so).
5. **His gold** (in his right back pocket) comes out all at once, a purse: 120 coins cost 12 points of
   chance. Gold is drawn as the game draws gold, one coin, so it hides in the fist; the card says
   how much ("Gold (120)").
6. **Plant something.** Hold something weightless you took (the gold, the key, a lockpick) and let
   go of it in one of his pockets: it's his. Something with weight (the potion) gets the game's own
   "You cannot place items in a container while pickpocketing." and stays yours. This is the game's
   own rule: Oblivion has no chance for planting, only that rule.
7. **Rummage and come away empty.** A quick look (a hand in someone's pockets for less than a
   second) is only feeling: nothing happens. Feeling about longer opens his pickpocketing, as the
   game's menu opening; coming away with nothing then is the game's own "looked and took nothing": a
   use of Sneak, and the game's chance (with nothing taken) that he noticed, and if he did, the
   crime and the bounty. It happens once the hands have been away from his pockets for 3/4 s.
8. **Drop or throw a taken thing** away from your body and anyone's pockets: it's the game's own
   thing in the world where you let go, flying on as it left your hand (Havok), to pick up again.
9. **The patrons.** Walk in (sneaking) and try the people moving about. Their pockets hold what
   they carry but don't wear, as the game's pickpocketing lists it, and that's often nothing: an
   empty pocket is one dull brush and no card. In the round 3 run: Velus Hosidius had only 10 gold
   (his right back pocket); Nilphas Omellian nothing at all; Audens Avidius two keys, 31 gold, a
   silver warhammer and clothes, spread over all four. Follow a walking one: the pockets move with
   them.
10. **Watch the sneak eye** (at your gaze; the flat game's crosshair is hidden in VR, its activate
    icons stay). While a hand is in the pocket of someone who sees you, you feel a slow heartbeat
    in that hand (a strong beat as they first notice you). Being seen doesn't change the per-item
    chance (the game's doesn't), but when you're done with someone, everyone who sees you then
    witnesses the theft (the game's own check): the crime and the guards.
11. **Steal something lying about.** Pick up something of the inn's from a table or a shelf (a
    plate, a bottle): its card over your hand has the red hand and the word **Steal** over its name.
    Something that's nobody's (or yours) has no such line.
12. **Walk right up to someone** (rounds 9 and 10). Walk up to Bruce, with the stick or stepping
    in the room: you come chest to chest with him (your body and his each as round as a torso),
    and he isn't shoved. Leaning in the room, your head goes on into his space.

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
  thing is right for a real hand; the four trouser pockets (round 6) by hand; the card's steady
  size and its red line coming and going with it (round 6); a card on each hand (round 8); coming
  close to people (rounds 9 and 10: the game's own capsule radii, and how its controller takes a
  new size, are read from the code, not seen).
- The heartbeat itself (only the awareness values: 0, 0.5 noticed, 1 seen, read from the NPC's own
  detection list).
- The game's success message and sound (the call is the menu's own, with its message flag).
- The "Items Pickpocketed" statistic going up (it's written as the menu does).
- The card's red action line (round 2): how the icon and the word sit over the name, and that the
  game loads the reticle's icon sheet in place of the Xbox buttons'. None of it has been run.
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
- **The session is the menu.** Opened once a hand has been in someone's pockets for a second
  (round 3: the game's rummaging check after every brush got the player arrested for looking), or
  at once as a thing is taken out or planted; closed 0.75 s after the hands are away from them all.
  A quicker look is only feeling and the game has no part in it. Someone who caught the player
  before has no menu at all, only the game's refusal (`sNoPickPocketAgain`), so no second check.
- **Everything in the trouser pockets, front and back** (round 6; the rounds before tried belt
  pouches with a purse, a chest pocket and a bag; then two pockets wrapping each hip; then trouser,
  back and shirt pockets and a bag), the same on everyone (Oblivion's clothes have no pockets of
  their own). On the body's 17 cm capsule at the pelvis: a front pocket 40 degrees round from each
  hip's side towards the front, a back pocket 40 degrees round towards the back, each 12 cm round:
  together the hip's surface from near the middle of the front round its side to near the middle of
  the back, so from in front, behind or the side there's a pocket under the fingers. Things go where
  they'd be kept (`pocket_for`: keys, lockpicks and gems at the left front; potions, food and small
  things at the right front; gold at the right back; anything else at the left back), then are
  spread out (`spread`): no pocket has more than a quarter of them, rounded up, while another has
  fewer.
- **Planting a thing lying about** (a thing to take, not yet yours) first takes it with the game's
  own pick-up, then plants it, if it's weightless; otherwise it falls out of the pocket.
- **Templates and a card reference.** Each kind of thing is a template object drawn from the game's
  model; its card is the game's own, given a reference of the game's own making that's never placed
  (as xOBSE's inventory references).
- **The card of the thing under the fingers** (asked for in round 1, before playing). The runtime
  tells the game which thing each hand's fingers are on (`PE_FEEL`), and the game's card is that
  thing's while no hand holds one, over the hand in the pocket. A thing in a hand comes first.
- **"Pickpocket" and "Steal" in red, from the game's own UI** (asked for in round 1). Oblivion on
  the PC marks a theft one way only: the reticle's activate icon (a hand, mostly) in its stealing red
  (`hud_reticle.xml`'s `_red_stealing`: 208, 0, 0). It does that both for someone's thing and for a
  sneaking player looking at a living person, which is pickpocketing. It has no word for it on
  screen, but its card has an action line for the Xbox: a button and a word over the name
  (`hudinfo_action_icon`, `hudinfo_action_text`). That line is used here:
  - the icon is the reticle's own, from its activate icon sheet, the one the game would show for
    the thing, in the stealing red;
  - the word is the game's own setting, `sPickpocket` ("Pickpocket") or `sSteal` ("Steal"), in the
    same red;
  - it shows only when the game's own test (the reticle's) says taking it is a crime.

  A thing out of a pocket says "Pickpocket" until the game has made it yours. A thing of the
  world's in your hand says "Steal" while it's someone's.
- **A hand is in a pocket when its fingers or palm are** (round 2: no pocket was ever found). The
  runtime's log showed every reach for Bruce's hips stopped by his body ("blocked") with the hand's
  grip 13-24 cm from the pouches' middles: the grip is a hand's length behind the fingers that
  touch, and the hands came from behind while the pouches were at the very sides. Now the test is
  from the nearest of the hand (its collider: the palm and the fingers); the pouches hung 30 degrees
  behind the sides of the hips in round 3 (see the pockets' places above for now).
- **The card stands just over the thing** (round 2: too far above it). It stood 6 cm plus half its
  height over the ball round the thing (a plate held flat counted as 15 cm tall). Now its bottom
  edge is 1.5 cm over the top of the thing's box as it's turned now, the game's card as the
  runtime's.
- **The card keeps its size, and its red line comes and goes with it** (round 6: first "Gold"
  alone, then the red line; going, "Gold" big and cut off). The log had the card's part of the HUD
  31% of its width with the red line, 21% without it and 11% as it faded, each stretched to the
  same 20 cm: the text tripled as the line went, and the part lagged what the game drew. Now:
  - the red line is decided as the card's thing changes, in the same frame, not before the next
    picture;
  - it stays as the card fades out with nothing in a hand (it's on the card that's going);
  - the card's part is at least 34% of the HUD's width from its right edge (its lines are
    right-aligned), and only changes while a hand's thing is on it;
  - the runtime draws it at one scale (the whole HUD would be 60 cm), not stretched to a width.
- **The card over a hand only once it's the thing's** (round 7: as a hand first went into a pocket
  the card showed the person's name with the red "Pickpocket"). The game's card was still of what
  the crosshair was on (the person) while the game laid out the thing's name, about 0.1 s (the
  log's boxes); the red line was already set. Since round 8 the layout says what the card's of
  once the game has laid it out (`HudLayout::held`, 0.15 s after it's set to a thing), and only that
  is ever over a hand (below).
- **A card for each hand** (round 8: both hands could take at once, as the log showed, gold and a
  note within 6 ms, but only the latest hand had a card). The game has one card, of one thing at a
  time, so each hand's card is a picture of it: the runtime cuts the card's part of the HUD while
  the layout says it's of a thing on a hand (`HudLayout::held`: its id as the runtime knows its
  card, an object's own or a pocket thing's template) and draws each hand's card from its thing's
  picture, over the hand, at the card's scale, fading with it. With a thing on each hand, the
  adapter has the card of each in turn till both have their picture (0.15 s for the game to lay it
  out, 0.1 s more to cut it), a thing new to a hand first, then leaves it. A hand's card shows once
  its thing's picture is cut: a quarter of a second, a little more when the other hand's new too.
- **Close to people without shoving them** (round 9: the body couldn't get close, it pushed people
  away; the hands could). Room-scale walks the game's body after the head (the runtime's
  `TRK_FOLLOW`), and the game's character controllers push each other: walked into someone, the body
  shoved them. Now, while the body is walked after the head, its step stops where its controller
  would touch theirs (each controller's own radius, as the game's `SetSize` sets it), and
  the head goes on alone, as far as room-scale lets it from the body (60 cm): right up to them.
- **As close as a body** (round 10: still not close enough, "as close as my body physically is").
  The round-9 log had no stop at all: Bruce was walked into with the stick, which round 9 left to
  the game, and shoved again. And the game's capsules are rounder than the bodies they hold (each
  sized from its model's bound: the game's default bound is 25 to 33 cm round, a torso
  17), so even stopped at the touch, chests were a hand or two apart. Now:
  - Every step of the player's on the ground, the stick's too, stops where its capsule would touch
    someone's: nobody is shoved by walking into them any more (you go round them, or wait, where
    one stands in a doorway).
  - In VR, the player's capsule, and those of the people (NPCs, not creatures) within 3 m, are a
    torso's round (0.17 m, the one the hands meet on others), as the game's own `SetSize` makes
    them (the console's `SetSize`/`CSize`, "Shrink/Grow an actor": the controller's radius at
    the size asked, which it grows or shrinks to over its next updates). A metre further off, people have
    their own again; out of VR, the player too. People only count within a person's height (128
    units) up or down: not someone on the floor above.
  - The adapter's log says so: `movement: the player's capsule is a body's round ...`,
    `movement: 0x... near the player is a body's round ...` (each with the game's own radius), and
    where the body stopped (`movement: the body stops at ...`).

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
  layout at the end. `HudLayout` has a new `held` word (from its padding: the same size), and
  `PocketEvent` a new kind (`PE_FEEL`).
- **Floating menus (`overnight/menus`), the HUD:** the game's card over a hand (`HUD_HELD`) is now
  drawn from pictures of it (`held_cards.rs`), one per hand; the HUD's own `HUD_HELD` piece is drawn
  only when no hand has a card (at its place on the flat screen).

## Tools for checking

- The adapter's log: `pockets: N people near with pockets: <who>: <thing> xN in <pocket>`, every
  attempt with the game's chance and roll, the bounty as it changes, and every two seconds where each
  pocket is from the eye (right, up, forward). `pockets: the right hand's fingers on <thing> in
  <pocket>: its card` as they move on; `hud: the card says "Pickpocket" in the reticle's stealing
  red` as the action line first shows.
- The runtime's log: hands into pockets, things closed on, pulled out and the game's answer, and
  every two seconds how far each hand is from the nearest pocket.
- Throwaway emulated-headset scripts were under `<builds>\pickpocket\scripts` (not committed).
