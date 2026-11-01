# Pickpocketing by hand

Research for picking pockets by hand: reaching into an NPC's pocket while sneaking and taking
something out, with Oblivion's own chance, detection and reaction, and the reverse (planting).
First what players say about the VR games that do this, then Oblivion's own pickpocketing, then
the design built from both.

## 1. What players say

Searches of reviews, Steam threads and mod pages (October 2026; Reddit and Nexus pages couldn't be
read directly, so mods are known from their listings' summaries). Inspiration only.

### Thief VR: Legacy of Shadow (2025): the one shipped game where it's physical

- Guards carry purses, satchels and keys on the belt or the back. You sneak up and lift them off
  with your own hand, without brushing the guard (touching him alerts him); loot is pulled to the
  chest into your pack. A small white dot on a thing says it can be grabbed (UploadVR called it a
  stand-in for touch). The developers' GDC talk singled out its "haptic-heavy" pickpocketing;
  lockpicking can be done by rumble alone with the interface off.
- **Loved:** stretching out an arm, lifting a coin purse or a key off a belt unnoticed and melting
  away ("incredibly rewarding", Gaming Trend); the interface can be switched off; physics mishaps
  (a bottle knocked over reaching for a coin) add tension rather than annoyance; directional sound.
- **Hated:** not knowing who can be robbed (every guard wore a satchel, only some could be picked:
  the preview asked for unpickable guards to show no bag); fumbly, tiny grab points ("pawing" at
  things, Road to VR); awkward reaches into sleeping guards in chairs; loot that is only a score;
  guards that swing between oblivious and hyper-aware, and reset after ten seconds of hiding, so
  failure costs nothing; a lit/unlit crystal on the hand that hides how close you are to being
  seen.

### Assassin's Creed Nexus VR (2023)

- Pickpocketing a guard's key is a faster, riskier route than picking a lock. A detection meter
  fills over each guard; your hood frames the view while sneaking; being spotted gives a moment to
  hide.
- **Hated:** things vanish into an invisible inventory and reappear on a grip; handing things to
  people feels like grabbing at air; bodies grabbed only at fixed points; stealth without suspense
  (recharging health: hide and retry); interface that can't be turned off.

### Skyrim VR and its mods

- Vanilla pickpocketing is the flat menu floating in space, things "sliding into your inventory
  magically". Looting on PSVR meant putting your head to the floor.
- HIGGS (hands, gravity gloves, over-the-shoulder stashing) and PLANCK (physical contact with NPCs)
  are loved for removing the menu looting. No true physical pickpocketing mod turned up; the nearest
  are *Gift by Hand VR* (drop a thing into a follower's palm to give it: planting's shape), *Immersive
  Pickup Only* (no theft by a stray grab), *QuickPocket* (no inventory screen), and flat mods that
  add a risky timing step (*PickPocketRedone*) or conditions like being out of the target's sight
  (*Peekpocket*).

### Hitman VR, The Walking Dead: Saints & Sinners, Sniper Elite VR

- Hitman: early VR was "made for a gamepad" (magnetic snaps, menu selection); PS VR2's version is
  praised for keys and keycards held and inspected up close, criticised for "press A" interactions.
- Saints & Sinners: body slots and an over-the-shoulder backpack are loved, with small confirmations
  that a thing really went in; the backpack's grid and finicky zones aren't, and players keep
  dropping valuables.
- Sniper Elite VR: picking ammo off the floor "very finicky"; its eye icon and meters over heads
  "aren't exactly complementary" to VR.

### Detection, readably

- **Works:** sound (guards' barks, directional footsteps; Alyx's Jeff is stealth by sound alone);
  cues on your own body (Thief's hand crystal, Espire's visor, Metro's watch, AC's hood); an
  escalating per-NPC cue before full alert; a short beat to react when spotted.
- **Fails:** screen-space notifications and HUD that can't be turned off; binary lit/unlit cues;
  colour-only cues; detection that feels random (seen in the dark); AI that's too dumb to matter.

### What to take from it

1. Pockets are real places on the NPC's own body that move with it.
2. Show who carries what: an empty pocket or a closed one is felt at once, never a mystery.
3. Feel before choosing: each kind of thing its own touch.
4. Lock the choice when the fingers close; pulling it out commits, and only then does the game roll.
5. Backing out is safe until you commit (Kingdom Come's "back out of the door").
6. Time keeps running: no menu, no pause.
7. Awareness from the game's own channels, readable without a flat HUD.
8. Caught: the game's own reaction at once, with a clear jolt.
9. Stolen things stay physical: in your hand, then a body slot. Never vanish.
10. Planting uses the same verbs in reverse: hand in, open the hand.
11. Avoid: tiny grab points, finicky zones, "press A to steal", accidental theft from a stray grab,
    things snapping into the hand.

## 2. Oblivion's own pickpocketing

### How it opens

Activating an NPC while sneaking skips dialogue and opens its inventory in the container menu, in
pickpocket mode (a dead one's is just its container). Not when:

- it's in combat: the activation goes to yielding instead;
- it has caught the player before: the game says "<name> " + `sNoPickPocketAgain` instead.

Taking from a living NPC is taking what it owns, so what moves is marked stolen (owned by the NPC).

### What it lists

The NPC's inventory, one entry a stack:

- not worn: equipped things can't be taken;
- not non-playable armour or clothing;
- not bound;
- gold is listed, as a stack;
- quest items and keys are listed like anything else (their scripts and the stealing rules do the
  rest).

### The chance, rolled per thing as it's taken

Each side's skill is its Sneak plus its Luck times `fActorLuckSkillMult` plus
`iActorLuckSkillBase`, held to 0..100. The chance is three terms added and rounded: the player's
skill times `fPickPocketActorSkillMult` plus `fPickPocketActorSkillBase`; the target's skill times
`fPickPocketTargetSkillMult` plus `fPickPocketTargetSkillBase`; and what's taken (its value times
the count) times `fPickPocketAmountMult` plus `fPickPocketAmountBase`. It's held between
`fPickPocketMinChance` and `fPickPocketMaxChance`, and the take works when the game's roll of 0 to
99 is under it.

| Setting | Oblivion.esm |
|---|---|
| `fPickPocketActorSkillBase` | 40 |
| `fPickPocketActorSkillMult` | 0.6 |
| `fPickPocketTargetSkillBase` | 0 |
| `fPickPocketTargetSkillMult` | -0.6 |
| `fPickPocketAmountBase` | 0 |
| `fPickPocketAmountMult` | -0.1 |
| `fPickPocketMinChance` | 5 |
| `fPickPocketMaxChance` | 85 |
| `iActorLuckSkillBase` | -20 |
| `fActorLuckSkillMult` | 0.4 |

So with the shipped data: 40 + 0.6 x (your Sneak - theirs), less a tenth of a point per gold of
what's taken (value times count), between 5% and 85%. Sneak 50 against 25, Luck 50 both: 55% for
something worthless, 50% for a 50-gold ring, 43% for a purse of 120 gold. **Weight doesn't count,
and neither does whether the NPC sees you**: the per-thing roll is skill against skill against
value, nothing else. (The game's values are read at runtime from those settings, so a mod that
changes them changes ours.)

### Success, failure, closing

- **Success:** the thing moves to the player marked stolen, with the game's message and sound. Not
  gold: the "Items Pickpocketed" statistic goes up. Its value and count are added to the stolen
  total for the close.
- **Failure** (and the NPC isn't knocked out): the menu closes, then the game's pickpocketing crime
  (`iCrimeGoldPickpocket` 25 bounty, `fCrimeDispPickpocket` -25 disposition), the victim's own
  reaction (it reports it, and guards come; or attacks; or "did not care"), and the NPC remembers
  catching the player: no pickpocketing it again. The thing stays.
- **Closing having taken nothing at all:** the player uses Sneak (a skill use) and the game rolls
  the chance with nothing taken (amount 0): failing it, the NPC noticed the rummaging and it's the
  same crime and reaction with no thing. (That's the old Sneak-training trick: open, close, repeat.)
- **Closing having taken things:** the stealing check for the stolen total: every actor who detects
  the player now witnesses a theft and reacts: the crime, the guards. This is where awareness
  matters: being seen while you rob someone, by them or anyone, is reported once you're done.

### Planting

The menu's other tab gives from the player. To a living NPC, a thing that weighs anything is
refused with `sInvalidPickpocket` ("You cannot place items in a container while pickpocketing."); a
weightless one (gold, keys, lockpicks, some clutter) goes over at once, **with no roll**: Oblivion
has no reverse-pickpocket chance, only that rule.

### Detection

Each actor keeps who it detects and how well: lost, unseen, noticed or seen. The HUD's sneak eye
(above the crosshair while sneaking) opens when the player is detected. In VR the eye is part of
the reticle's HUD piece, placed where the head looks (`hud.rs`), so it shows where the player
looks: at the pocket.

## 3. The design

### Framework (general, any game)

`rustvr_protocol::pockets`:

- **`Pockets`** (game -> runtime, every frame): each pocket on a creature: the creature
  (`Actor::id`) and its part, where its opening is (world), how big, which way it faces, how aware
  its creature is of the player (the game's detection, 0..1), whether the game has it shut, and
  what's in it: each thing's game id, its template object (`OBJF_TEMPLATE`, drawn by the runtime
  from the game's model: the shape and weight in the hand), how it feels (`FEEL_*`: coins, a key,
  glass, paper, cloth, something hard) and how many come out together. Also the game's answers to
  the attempts (`PocketOutcome`).
- **`PocketEvents`** (runtime -> game, a ring): a hand going into a creature's pockets (`PE_OPEN`)
  and away again (`PE_CLOSE`: the game's pickpocketing opened and closed), a thing pulled out
  (`PE_TAKE`: the attempt), a thing let go in a pocket (`PE_PLANT`), a taken thing let go in the
  world (`PE_DROP`) or put in a body slot (`PE_STOW`), and which thing a hand's fingers are on
  (`PE_FEEL`, as it changes: the game shows its card of it).

The rig (`rig/pickpocket.rs`, wired into `rig/mod.rs`; `rig/pockets.rs` is the player's own body
slots):

- **Reaching in.** An empty hand whose fingers or palm are in a pocket (the nearest of the hand, not
  its grip, which stays a hand's length outside a body the fingers are on; and that isn't on
  anything nearer) is in it: a soft brush on the way in, and the thing it's on is felt: one tick in
  that thing's feel. Each 4 cm the hand moves in the pocket it's on the next thing, another tick (a
  fast rummage skips past things; a slow hand finds what it wants). An empty pocket is one dull
  brush and nothing more; a shut one a refusal. The game's card of the thing under the fingers
  floats over the hand, with the game's sign that taking it is a crime: the reticle's red hand and
  its word `sPickpocket`, on the card's own action line (`crate::hud_info`).
- **Taking.** The grip closes on the thing it's on: a copy of its template is in the hand (the
  game's model, its card beside it), the fingers closing round it. Still in the pocket it's the
  NPC's: let go there and it's back where it was, nothing tried. Pulled out (the palm out past the
  pocket's edge), the attempt goes to the game (`PE_TAKE`) and the game's answer comes back: taken,
  it's the player's (a body slot keeps it, let go anywhere else it's the game's own thing in the
  world, falling where it was let go); caught, the copy goes from the hand with a hard jolt, and the
  game does the rest.
- **Planting.** Anything to take (`OBJF_LOOT`) or a taken thing let go in a pocket is offered to the
  game (`PE_PLANT`); planted, it's gone into the pocket; refused, it falls out.
- **Sessions.** A creature's pickpocketing opens once a hand has been feeling in its pockets for 1 s
  (`OPEN_AFTER`; a quicker brush past them is only feeling, so it can't fail the game's rummaging
  roll), or at once as a hand takes or plants a thing, or finds them shut. It closes once no hand
  has been within 12 cm of any of them for 0.75 s (nothing pending): the game's menu opening and
  closing, so its close rules run.
- **Awareness.** While a hand is in a pocket of a creature that sees the player (awareness over
  0.66), a slow heartbeat in that hand, a strong one as it first notices: the game's own detection,
  felt without a HUD.
- **A hand in a pocket is light:** it doesn't push the creature's body (no `GamePush` from it).

### Oblivion (`crates/oblivion/src/pockets.rs`)

- **Who has pockets:** NPCs (not creatures) the game is fully processing within 3 m, alive, not in
  combat, while the player sneaks: exactly whom activation would open pickpocketing on. One that has
  caught the player is published shut (its hand gets the refusal and the game's
  `sNoPickPocketAgain`).
- **Where (its own skeleton's bones, every frame):** its trouser pockets, at the front and the back
  of each hip (`Bip01 Pelvis`, the thighs' line for its sides): from in front the front ones, from
  behind the back ones, from the side either. (The playtests went from belt pouches, a purse, a
  chest pocket and a bag, to two hip pockets, to trouser, shirt and back pockets, to these.)
- **What (its inventory, as the pickpocket screen lists it, every half second):** the game's own
  entries and its own filter, where they'd be kept: keys, lockpicks and gems at the
  left front; potions, food and small things at the right front; gold at the right back (all of it
  together: it comes out whole as a purse, so its value counts whole, as a full stack does in the
  menu); rings, amulets, soul gems, books, notes and anything bigger at the left back. Then spread
  out: no pocket has more than a quarter of them while another has fewer (12 at most a pocket).
  Other stacks come out one at a time.
- **The take is the game's:** its own chance with its own skills, value and count, its own roll,
  then its own `RemoveItem` (stolen) and statistic, or its own caught crime and reaction, exactly
  as the menu does them.
- **Sessions are the menu's:** opening clears nothing; closing runs the game's close: nothing tried
  → the Sneak use and the rummaging roll; things taken → the stealing check with the stolen total.
- **Planting is the game's rule:** weightless things go over (`RemoveItem` to the NPC), anything
  else gets the game's `sInvalidPickpocket` message and falls out of the pocket. A thing lying in the
  world that's planted is first the player's by the game's own pick-up (its stealing, if it's
  someone's).
- **Dropping:** a taken thing let go in the world is the game's own drop from the player's
  inventory at that place (`RemoveItem` with its drop), so it's a thing in the world again, for the
  hands to take.
- **Awareness:** the NPC's own detection of the player (its detection list: noticed is 0.5, seen
  1).

### Decisions

- **No chance of our own.** Oblivion's per-thing chance has no term for a steady hand or for being
  seen, so none is added: a careful hand matters by finding the right thing (each move ticks to the
  next) and by not rummaging (every time the hands come away empty is the game's own rummaging
  roll); being seen matters where the game makes it matter, at the close.
- **The purse is all the gold.** The game's menu asks how many; a hand can't say. Coins come out as
  the purse, and the game prices the whole of it.
- **The refusal is a message, not a box.** The game shows `sInvalidPickpocket` in a message box; in
  VR a box mid-reach would take over the view, so the same text goes to the game's corner message.
- **No contact rule.** Thief VR's "touching alerts" isn't Oblivion's: the hand is held off the body
  by its capsules and pushes nothing, and detection stays the game's.

### With the other features

- **Grabbing people** (`grabs.rs`): a hand in a pocket of a creature (sneaking, its pockets
  published) takes from the pocket; that comes before a grab of the creature's body. The rig's hover
  order says so (a creature's pocket before the holsters, the body slots and anything of the
  creature's body), and `Rig::in_pocket(hand)` says which hand is in one, for the grab code to stand
  aside. Not sneaking, no pockets: every grip on a person is the grab.
- **The floating HUD:** the sneak eye is part of the reticle's piece, where the head looks.
- **The body slots:** what can be planted is a thing lying in the world, taken in the hand
  (`OBJF_LOOT`), or a thing taken from a pocket.

**Not built:** the sneak eye riding on the pocket hand's wrist; planting a thing drawn from one of
the player's body slots (the rig's copy of it isn't offered to the pocket, so it falls).
