# Playtest: pickpocketing by hand

Save: rustvr-pocket

## Playing it

From the repository root:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-pocket
```

With Sneak 100 (an 85% chance, the most the game allows):

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-pocket -Commands "player.setav sneak 100"
```

Each builds main, installs it beside the game and boots straight into the save for the headset.
`powershell -File tools\oblivion.ps1 -Close` puts the game away and the player's settings back.
The fast boot, the test saves and the game lock are in [DEVELOPING.md](../DEVELOPING.md).

`rustvr-pocket` is the developer's own test save, not in the repository: the Merchants Inn in the
Imperial City's Market District, at noon, at the door. Bruce (a test NPC with no AI packages, so he
stands still) is about a metre to your left, his back to you, carrying 120 gold, a key, 3
lockpicks, a potion, two apples and a note. The inn's own patrons walk about further in. Any
person does; to make a save there from any other:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-city -At ICMarketDistrictTheMerchantsInn -Hour 12
```

Then save it as `rustvr-pocket` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

The research and the design are in `docs/research/pickpocket.md`.

## What it does

While you sneak, the people near you wear pockets on their own skeletons holding their own things
(`crate::pockets`; the runtime's side is `rig::pickpocket`). A hand that pulls a thing out is the
game's own pickpocketing of it; one that lets a thing go in a pocket is the game's own planting.

- **Who.** The NPCs (not creatures) within 3 m, alive and not fighting: whom activating while
  sneaking would open the game's pickpocketing on. One who caught you before is shut: the hand
  finds the pockets so, and the game says why (`sNoPickPocketAgain`).
- **Where.** Their trouser pockets: one at the front and one at the back of each hip, on the body's
  17 cm round at the pelvis, 40 degrees round from each hip's side, each 12 cm round. Together they
  cover the hip from near the middle of the front, round its side, to near the middle of the back,
  so from in front, behind or the side there's a pocket under your fingers. They move with the
  person.
- **What.** Their inventory as the game's pickpocketing lists it (nothing worn, nothing unplayable
  or bound), each thing where it would be kept (`pocket_for`): keys, lockpicks and gems at the left
  front; potions, food and small things at the right front; gold at the right back; rings, amulets,
  soul gems, books, notes and anything bigger at the left back. Then spread out (`spread`): no
  pocket has more than a quarter of them, rounded up, while another has fewer.
- **Feeling.** A hand is in a pocket when its fingers or palm are (its collider, not its grip): it
  stops on the body, and touching there is being in the pocket. A soft brush going in, then a tick
  for the thing your fingers are on; every 4 cm the hand moves, the next thing, another tick. Each
  kind feels different: coins jingle, a key clicks, glass rings, paper rasps, cloth or food is a dull
  push, something hard knocks. An empty pocket is one dull brush.
- **The card.** The game's card of the thing under your fingers floats over your hand (its name,
  value and weight), changing as you move on. Over the name, in the game's stealing red, the
  reticle's red hand and the word **Pickpocket**. A thing of the world's in your hand that's
  someone's says **Steal**. Each hand has its own card.
- **Taking.** Squeeze: the thing (the game's own model) is in your hand, with its card. It's still
  theirs while your hand is in the pocket: let go there and it's back, nothing tried. Pull it out
  and the game decides as it leaves the pocket, with its own chance (40 + 0.6 times your Sneak less
  theirs, both with Luck, less a tenth per gold of value taken, between 5 and 85%) against its own
  roll.
  - **Taken:** it's yours, marked stolen, and the red line goes from its card. Put it in a slot or a
    holster on your body to keep it.
  - **Caught:** a hard jolt and it's gone from your hand. Then the game's own reaction: the
    pickpocketing crime (bounty 25), their disposition, their alarm or attack, and they can never be
    robbed again.
- **Gold** comes out all at once, a purse ("Gold (120)"), drawn as the game draws gold: one coin,
  hidden in the fist.
- **Planting.** Let go of something weightless you took from a pocket (the gold, a key, a lockpick),
  or a thing lying about you've picked up, in one of their pockets: it's theirs. Something with
  weight gets the game's own "You cannot place items in a container while pickpocketing." and stays
  yours. That's the game's rule: Oblivion has no chance for planting.
- **The session is the game's menu.** It opens once a hand has been in someone's pockets for a
  second (a quicker look is only feeling), or at once as a thing is taken or planted, and closes
  0.75 s after the hands are away from them all. Closed having tried nothing, it's the game's own
  rummaging check: a use of Sneak and its chance, with nothing taken, that they noticed. Having
  taken things, it's the game's own check for witnesses of the theft: everyone who sees you then
  witnesses it.
- **The sneak eye.** While a hand is in the pocket of someone who sees you, a slow heartbeat in that
  hand (a strong beat as they first notice you). Being seen doesn't change the per-thing chance (the
  game's doesn't); it matters at the close, in the witness check.
- **Close to people.** Every step on the ground, the stick's too, stops where your capsule would
  touch someone's, so nobody is shoved by walking into them. In VR your capsule and those of the
  people within 3 m are a torso's round (0.17 m), through the game's own `SetSize`; further off
  people have their own again, and out of VR you do too. People count only within a person's height
  up or down. Leaning in the room, your head goes on into their space, as far as room-scale lets it.
- **The game's flat pickpocketing is still there:** the trigger on someone while sneaking opens it.
  A hand in a pocket doesn't activate anything with its trigger.

## What to try

1. **Sneak** (the stick's crouch, or a slight crouch for real: [stealth.md](stealth.md)).
2. **Find a pocket by touch.** Lay your fingers on Bruce's hip: from behind (his back's to you) on
   the back of it, from in front on the front, from the side on its side. Feel the brush, then the
   ticks as you move slowly. Watch the card over your hand and the red **Pickpocket**. His pockets:
   the apples and the potion at the right front, the lockpicks and the key at the left front, all his
   gold at the right back, the note at the left back.
3. **Take one.** Squeeze; let go inside: it's back.
4. **Pull it out.** About 40% against Bruce with this character's Sneak: expect to be caught.
5. **His gold**, all at once: 120 coins cost 12 points of chance.
6. **Plant something.** The gold, the key or a lockpick back in a pocket: his. The potion: refused.
7. **Rummage and come away empty.** A quick touch is only feeling. Feel about longer and come away
   with nothing: the game's own rummaging check, three quarters of a second after your hands leave.
8. **Drop or throw a taken thing** away from anyone: the game's own thing in the world, flying on as
   it left your hand.
9. **The patrons.** Walk in sneaking and try the people moving about, and follow a walking one.
   Their pockets hold what they carry but don't wear, which is often nothing: one dull brush and no
   card.
10. **The sneak eye** and the heartbeat while someone sees you.
11. **Steal something lying about** (a plate, a bottle on a table): its card says **Steal**.
12. **Walk right up to someone**, with the stick and stepping in the room: chest to chest, nobody
    shoved.

## Settings

| Setting | Where | Default (range) |
|---|---|---|
| Duck to crouch, Stealth crouch amount (`input.physical_crouch`, `input.stealth_crouch_amount`) | Comfort tab | on, 8 cm (5 to 40) |
| Roomscale, Max distance from body (`comfort.roomscale`, `roomscale_limit`) | Comfort tab | on, 0.6 m (0.2 to 1.5) |
| Interaction sounds, Volume | Hands tab | on, 0.8 |

## Decisions

- **No chance of our own.** The game's per-thing chance has no term for a steady hand or for being
  seen. A careful hand matters by finding the right thing (each move ticks to the next) and by not
  rummaging; being seen matters at the close, where the game makes it matter.
- **Found by touch, chosen by closing, committed by pulling out** (the research: Thief VR's
  satchels that couldn't be picked were its players' top complaint; Alyx's lock on closing; Kingdom
  Come's way out). Time never stops: no menu, and the person goes on moving.
- **The purse is all the gold.** The game's menu asks how many; a hand can't say. Other stacks come
  out one at a time.
- **The refusal is a corner message, not a box.** The game shows `sInvalidPickpocket` in a message
  box; a box in the middle of a reach would take over the view.
- **The session opens after a second** (round 3: the game's rummaging check after every brush got
  the player arrested for looking).
- **Everything in the trouser pockets, the same on everyone** (round 6): Oblivion's clothes have no
  pockets of their own.
- **Templates and a card reference.** Each kind of thing is a template object drawn from the game's
  model; its card is the game's own, given a reference of the game's own making that's never placed
  (as xOBSE's inventory references).
- **"Pickpocket" and "Steal" in red, from the game's own UI.** Oblivion on the PC marks a theft one
  way only: the reticle's activate icon in its stealing red (`hud_reticle.xml`'s `_red_stealing`).
  Its card has an action line for the Xbox, a button and a word over the name
  (`hudinfo_action_icon`, `hudinfo_action_text`), and that's used: the reticle's own icon for the
  thing and the game's setting `sPickpocket` or `sSteal`, in the same red, only when the game's own
  test says taking it is a crime.
- **The card stands just over the thing** (round 2): its bottom edge 1.5 cm over the top of the
  thing's box as it's turned now.
- **The card keeps its size** (round 6): the red line is decided in the same frame as the card's
  thing, stays as the card fades, and the card is drawn at one scale, not stretched to a width.
- **A card over a hand only once it's the thing's** (round 7), and **a card for each hand** (round
  8): the game has one card, so each hand's is a picture of it (`HudLayout::held`,
  `held_cards.rs`), the adapter giving the game each hand's thing in turn till both have their
  picture. A hand's card shows about a quarter of a second after its thing.
- **As close as a body** (rounds 9 and 10): the game's capsules are rounder than the bodies they
  hold (each sized from its model's bound), so even stopped at the touch, chests were a hand or two
  apart. Near people, everyone's capsule is a torso's round.

## Not done / known issues

- **Gold is one coin** (the game's only gold model): it hides in the fist.
- **A thing without a model the extractor can read** is invisible in the hand (its card still
  shows).
- **Creatures have no pockets:** NPCs only, as the game's pickpocketing is in practice.
- **Things drawn from your own body slots can't be planted**, only things taken from a pocket or
  picked up from the world (`plantable` in the rig is `OBJF_LOOT` or a taken pocket copy).
- **Not seen in a playtest record:** the game's success message and sound; the "Items
  Pickpocketed" statistic going up; sleeping, sitting and walking NPCs, and crowds (performance
  with many people near).

## History

Newest first.

- **Round 10.** Still not close enough, "as close as my body physically is". The stick's steps now
  stop at the touch too, and near people every capsule is a torso's round.
- **Round 9.** The body couldn't get close; it shoved people away. Room-scale's steps now stop where
  the controllers would touch, the head going on alone.
- **Round 8.** Both hands could take at once, but only the latest had a card: a card for each hand.
- **Round 7.** As a hand went into a pocket, the card showed the person's name with the red line:
  it's now over a hand only once it's the thing's.
- **Round 6.** The card's size jumped as its red line came and went: one scale now. Everything went
  into the trouser pockets, front and back.
- **Rounds 3 to 5** tried belt pouches with a purse, a chest pocket and a bag; then two pockets
  wrapping each hip; then trouser, back and shirt pockets and a bag. Round 3 also moved the session's
  opening to a second in, after looking alone got the player arrested.
- **Round 2.** No pocket was ever found: the reach was measured from the hand's grip, a hand's
  length behind the fingers. Now from the fingers and palm. The card stood too far above the thing.
- **Round 1**, before playing: the card of the thing under the fingers, and "Pickpocket" and
  "Steal" in red.
- **The first build**, on the emulated headset: pockets from the bones, filled with exactly the
  game's list; taking (the potion, the key, a purse of 50) into a body slot; caught (the bounty 0 to
  25, the NPC shut); planting gold, the potion refused; rummaging; dropping a key. A trespassing
  player (someone's house, the fields at Weye) isn't reacted to at all: the game's own rule. The
  game's older models (10.2.0.0: the common key) and Havok triangle collision blocks were made
  readable, so keys and lockpicks are drawn; creatures' capsules were fixed to cover the torso, head
  and legs too, which fixed combat's on NPCs.

## Logs

- The adapter's (`rustvr-oblivion.log` in the game's folder): `pockets: N people near with pockets:
  ...`, every attempt with the game's chance and roll, the bounty as it changes; `pockets: the right
  hand's fingers on <thing> in <pocket>: its card`; `hud: the card says "Pickpocket" in the
  reticle's stealing red ...`; `movement: the player's capsule is a body's round ...`,
  `movement: 0x... near the player is a body's round ...`, `movement: the body stops at ...`.
- The runtime's (`%LOCALAPPDATA%\RustVR\rustvr.log`): hands into pockets, things closed on, pulled
  out and the game's answer, the pickpocketing opening and closing.
