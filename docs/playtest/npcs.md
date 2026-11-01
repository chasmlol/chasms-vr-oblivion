# Playtest: grabbing people and creatures

Save: rustvr-npcs

## Playing it

From the repository root, in god mode (`tgm`):

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-npcs -Commands "tgm"
```

For a fight, with a hostile bandit ringleader 4 m in front and a living wolf 4 m behind:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-npcs -Commands "tgm|player.placeatme 0003DC25 1 280 0|player.placeatme 0006E912 1 280 1"
```

Each builds main, installs it beside the game and boots straight into the save for the headset.
`powershell -File tools\oblivion.ps1 -Close` puts the game away and the player's settings back.
The real keyboard doesn't reach the game in VR, so console lines go in `-Commands`. The fast boot,
the test saves and the game lock are in [DEVELOPING.md](../DEVELOPING.md).

`rustvr-npcs` is the developer's own test save, not in the repository: a room in the Imperial City
with a Legion soldier standing a metre in front of you (friendly, AI on) and a dead bandit in fur
armour lying at your feet, half a metre to your right. To make one from any save, in a room:
`-At <cell> -Commands "player.placeatme 0018B118 1 70 0"` (a Legion soldier a
metre ahead), saved as `rustvr-npcs` ([DEVELOPING.md](../DEVELOPING.md), "Test saves"), with a body to drag of whoever you kill. More people: `player.placeatme 0018B118 1
100 0` (a Legion soldier), `0003DC25` (a bandit ringleader, hostile), `0006E912` (a wolf); `tai`
stops everyone's AI.

The framework's test range has dummies that take grabs too (`framework/crates/range`): after a
build, `target\release\rustvr-play.exe` is the runtime with the range (framework/README.md). Close
the game first: they share the runtime's link.

## What it does

A hand on any part of a person or creature (an arm, a leg, the head, the throat, the torso; a dead
body by anything) takes hold where it touched, and the game moves the body with its own Havok
ragdoll (`crate::grabs`; the runtime's side is `rig::creatures`).

- **The hand is the runtime's, the body the game's.** The hand is drawn on the part where it took
  hold, wherever the game has the part, so the body's weight holds it back and it never goes into
  the body. The game moves the body by its own grab (`bhkMouseSpringAction` on the Havok body of the
  bone held), with its own joints, masses, gravity and collision.
- **A dead or knocked-down body** is all Havok's: any part drags the whole body. Lift it by a leg and
  it hangs upside down; a second hand on another part holds it between them. Walk away holding it
  and it drags along the floor, staying where you leave it. Let go mid-swing and it flies with
  what the pull could give its mass.
- **A living actor keeps animating.** The bones from the one held out to the limb's end go physical,
  the spine partly: the limb follows the hand while the rest stands, and blends back to the
  animation over 0.4 s once let go.
- **Off its feet.** Pulling the head, neck or torso 12 cm above where the standing body has it for
  an eighth of a second knocks it down (the game's `KnockbackActor`), and the whole body hangs from
  the hand. So does a yank 55 cm away from where its body has the point (a leg: 35 cm), or a let-go
  faster than 2.5 m/s. While held it can't get up; let go, it gets up when the game says.
- **Pulled along the ground** while it stands, past 12 cm it's pushed after the hand by its
  character controller (the game's knockback without a ragdoll), up to 2.5 m/s, stumbling along.
- **Choking.** A hand squeezing the throat drains its fatigue: a full squeeze empties it in 6 s, 4 s
  off its feet, until the game knocks it out itself, and on below nothing so it stays out a while.
  Held up and still conscious, it struggles: its hands clutch at yours and its legs kick, weaker as
  it tires.
- **The law.** Choking, knocking down, throwing, or holding a living actor more than 4 s is an
  attack by the player (the game's `OnAttacked`): its disposition, fleeing or combat, and the
  assault crime with its bounty (40 gold). One assault a hold.
- **Touch and shove.** Without grabbing, a standing person gives under the hand and springs back,
  wobbling: the bones about the touch go partly physical through the game's own blend between
  animation and ragdoll. Arms and the head are loose, the spine firm, the legs stiff; the pelvis
  stays the animation's. A push is an impulse at the point pushed (Havok's `applyPointImpulse`), so
  off its middle it turns the part: a cheek pushed turns the head away, a forehead tips it back.
  Left alone 1.5 s, the bones blend back. An open hand coming in at 1.5 m/s or more shoves the
  person too, a firm push about 40 cm. Dead bodies are pushed the same way.
- **The head keeps looking at you.** While a hand has a person's head, the game's head tracking
  holds its look and the push is laid on top of it (hooks on its measure, its look and the bone's
  blend update), so the head gives under your hand and the face keeps its expression. About two
  seconds after the last touch it follows you again.
- **Hitboxes fit what they wear.** Each of a person's 14 parts (hands and feet their own) is fitted
  to the models the game draws on them (the skinned body, armour and clothes, the head, hair and
  helmet): as thick as nine in ten of its vertices reach, the torso as deep as its front and back,
  reaching to the crown, the fingertips, the heel and the toes (`crate::shapes`). Fitted again when
  what they wear changes.
- **Heavy and strong.** Holding a body that holds your hand back, a low rumble in the controller,
  harder the further back. Past 60 cm between the hand and its controller the grip slips. Your
  strength is the game's: at Strength 50 one hand lifts a person by the throat; weaker sags more,
  stronger throws further. Every body weighs in proportion to the player's own ragdoll taken as a
  75 kg person: a goblin weighs nearly three people.
- **Letting go.** The hand stays a free hand on the body where it let go and comes back to the
  controller from there, never through the body. A hand only resting on someone isn't holding them:
  its trigger uses what you look at as ever (look at them and pull it to talk).
- **While you sneak**, a hand at a person's hips is in their pockets, not on them
  ([pickpocket.md](pickpocket.md)).

## What to try

1. **The dead bandit.** Put a hand on any part of him and squeeze: the hand stays where it touched,
   the fingers round the limb. Lift by a leg (upside down), add a second hand, walk away dragging
   him, swing and let go, push him with an open hand.
2. **The soldier's arm.** Lift his forearm over his head while he keeps standing; pull it out to the
   side; let go and it eases back. Pull it along the ground: he's pushed after it, then over when you
   yank it 55 cm away. A leg trips him sooner.
3. **The throat.** Hand round his neck, squeeze, lift: his legs go and he hangs from your hand. Keep
   squeezing: his hands clutch at yours, his legs kick weaker, until the game knocks him out. Let go
   or throw him. Choked standing, he sags from your hand when he's out.
4. **The law.** Any of that, or holding him more than four seconds: the game's 40 gold bounty and
   his reaction.
5. **Touch and shove.** Push his face, cheek, forehead, an arm, his chest, a leg; slap an arm or his
   face; push fast. His head should give and he should keep looking at you.
6. **Heavy and strong.** Feel the rumble; pull past an arm's reach and the grip slips. Try a goblin
   or the wolf.
7. **A fight** (the second command). Grab the ringleader while he attacks: his held arm goes where
   your hand takes it, the rest of him fights on. Grab the wolf, alive.
8. **Letting go.** The hand comes back from his body, not through it. Rest a hand on his shoulder
   and pull the trigger to talk.

Look out for a hand stopping short of someone or sinking into them (and where), a part that flops
too much or too little, anything twitching while you hold someone.

## Settings

| Setting | Where | Default (range) |
|---|---|---|
| Strength on creatures (a hand's pull) (`interaction.creature_strength`, times the game's own strength: Strength 50 is 1) | Hands tab | 1500 N (400 to 4000) |
| Physics hands, Ghost hands (`interaction.physics_hands`, `ghost_hands`) | Hands tab | on, off |

## Decisions

- **The game simulates the body; the runtime has the hand.** The hand is drawn where the game has
  the part, so its weight holds the hand back (as HIGGS draws it).
- **Oblivion's ragdolls weigh about eight times a person** (a man about 600 of its units), so the
  player's own ragdoll is taken as a 75 kg person and every body weighs in proportion: a hand's
  pull in newtons means the same in the game.
- **The spring is stiffer than the game's own grab** (elasticity 0.7 against its 0.2). The game's
  drags a body a bone at a time, measured against the bone alone, so a whole body hanging from one
  bone sank 30 cm below the hand; now it hangs a hand's breadth below.
- **A touch is the game's blend, not a spring of ours.** The bones about a touch get a heir and a
  velocity gain below 1 (loose 0.2 and 0.1, firm 0.5 and 0.3, stiff 0.7 and 0.5): a spring that
  overshoots, as the game's blend has it. Oblivion has no value for how much a body gives, so these
  are ours. A held person isn't touched (the grab moves them).
- **The torso's thickness is its depth.** A capsule as wide as the shoulders would stop a hand in
  front of the chest; as deep as the chest, a hand on the waist's side sinks in a little (the arms
  hang there).
- **A choke knocks out, it doesn't kill.** Nothing in Oblivion says how hard a choke is, so the rate
  is ours.
- **The knockdown thresholds are ours** (12 cm of the head, neck or torso). The struggle is ours
  too: Oblivion has no animation for it.
- **A standing actor pulled is pushed, not walked:** he slides rather than steps. Oblivion has no
  step-to-follow; PLANCK makes Skyrim's actors walk, which needs its behaviour graph.
- **The head's look is held while a hand has it**, rather than the head tracking switched off, so
  he keeps looking at you (playtests 3 to 6).
- **One assault a hold**, whatever it comes to: the game's own counts each call.

## Not done / known issues

- **Landing doesn't hurt:** Oblivion's ragdolls take no damage from impacts; a throw into a wall is
  harmless.
- **A pin is only the world's:** a body pushed into a wall or the floor stays there while held. A
  blade through a body into a wall isn't this feature's.
- **Holding someone doesn't tire you** (PLANCK drains stamina; Oblivion has no value for it).
- **The gravity gloves don't pull bodies.**
- **Not seen in a playtest record:** a living creature that isn't a person (a wolf alive, a goblin),
  grabbing someone mid-attack, how far a shove carries a person, a body pinned against a wall.

## History

Newest first. Every round from playtest 2 on was about the head.

- **Playtest 6.** Still no head movement: the head tracking's look undoes the head bone's turn and
  puts its own in place, so nothing the physics does to the head shows while it's on. Now the head's
  animated turn is kept as the bone's blend begins, and the look is given the push on top.
- **Playtest 5.** The head didn't move: once a bone's body is dynamic its blend object writes the
  bone's own turn too, so the "animated" turn had the push in it. The head tracking's measure was
  then held for a head a hand has.
- **Playtest 4.** The head moved, but he should keep looking at you. Head tracking stayed on and was
  made to measure the head from its animation.
- **Playtest 3.** The head still didn't move: the game's head tracking turned it back within a few
  frames. It was switched off while a hand had the head (which stopped him looking at you).
- **Playtest 2.** The head couldn't be moved. Pushes became impulses at the point pushed, the head
  loose when touched; which bone a touch is on is logged.
- **Playtest 1.** It worked. Asked for soft bodies without grabbing (the touch through the game's
  blend), and hitboxes that cover armour (the fitted parts, hands and feet their own).
- **Before playtest 1**, on the emulated headset: a dead soldier taken by the foot and by two hands,
  lifted, dragged and thrown; the save's dead bandit lifted and dragged; a standing soldier's arm
  lifted and pulled, then him yanked over; the throat (knock states 1, 5, 6, 0 and 4, 3, fatigue
  280 to nothing, the bounty 0 to 40); a shove at 1.7 to 2.8 m/s; a dead wolf (37 kg) lifted; the
  strain felt; the adapter's creatures' part of the frame at 0.4 ms.

## Logs

The adapter's log (`rustvr-oblivion.log` in the game's folder): `shapes: ... fitted to N models`
(each part's thickness and reach), `grabs: ... touched on its bone N` (1 is the head),
`grabs: the heads the hands have: ...` and `grabs: the head tracking holds its look on the head
...`.
