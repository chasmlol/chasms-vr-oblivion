# Grabbing NPCs: playtest

Save: rustvr-npcs

`tools\play.ps1 npcs` boots into **rustvr-npcs**: a room in the Imperial City with a Legion soldier
standing a metre in front of you (friendly, AI on) and a dead bandit (fur armour) lying at your feet,
half a metre to your right. To get more people: `player.placeatme 0018B118 1 100 0` (another Legion
soldier), `player.placeatme 0003DC25 1 100 0` (a hostile bandit boss),
`player.placeatme 0006E912 1 100 0` (a wolf); `tgm` keeps you alive, `tai` stops everyone's AI.

## What to try, in order

1. **The dead bandit.** Reach down, put a hand on any part of him (an arm, a leg, the head, the
   torso) and squeeze the grip. The hand stays exactly where it touched him and the fingers close
   round the limb. Lift: that part comes up and the rest of him hangs from it. Lift by a leg and he
   hangs upside down; take a second hand on another part and he comes up between them. Walk away
   holding him: he's dragged along the floor (and stays where you leave him, even after a reload).
   Swing and let go: he flies, as fast as your swing, as far as your strength can throw his weight.
   Push him with an open hand: he moves where you push.
2. **The soldier's arm.** Take his forearm or wrist and lift it: it goes up over his head while the
   rest of him keeps standing (his animation goes on). Pull it out to the side: it follows to where
   his shoulder lets it. Let go: it eases back into his animation in a moment. Pull it along the
   ground and he's pushed after your hand, stumbling along; yank it away from him (55 cm past where
   his body lets it go) and he falls. Take a leg and pull: he trips sooner (35 cm).
3. **The throat.** Put a hand round his neck (the lower part of the head, or the top of the torso)
   and squeeze: the fingers close round the neck. Lift: once you've pulled his head up past where he
   can stand, his legs go and he hangs from your hand, his feet off the floor. Keep squeezing: his
   fatigue drains, faster while he's off his feet; his hands clutch at yours (his arms are his own
   while he's choked), and held up his legs kick, weaker as he tires, until the game knocks him
   out. Let go, or throw him: he lies there and gets up when
   the game says (a few seconds after a knockdown, longer after a choke-out). Choked while he's
   standing, he collapses out of your hand when the game knocks him out.
4. **The law.** Choking him, knocking him over, throwing him, or holding him for more than four
   seconds is an assault, the game's own: you get the game's bounty (40 gold) and his reaction (a
   Legion soldier comes to arrest you, or fights). One assault a hold.
5. **Shove.** Push the soldier in the chest with an open hand, fast (1.5 m/s or more): he's shoved
   back. Leaning on him slowly doesn't move him.
6. **Heavy and strong.** Holding a body that holds your hand back, you feel a low rumble in the
   controller, harder the further back it's held. Past an arm's reach (60 cm between your hand and
   the controller) your grip slips. Your strength is the game's: Strength 50 lifts a person
   one-handed by the throat; a weaker character sags more, a stronger one throws further. A goblin
   weighs nearly three people in Oblivion's own masses: one hand hardly moves it, two might. The
   Hands tab has "Strength on creatures" (1500 N a hand at Strength 50).
7. **A fight.** Place a bandit boss and grab him while he attacks: his held arm is yours (it goes
   where your hand takes it), the rest of him keeps fighting.

## What was checked in the game (emulated headset, captures and logs)

* A dead soldier taken by the foot: the foot follows the hand within 2 to 6 cm as it's lifted
  0.7 m, the leg hangs from it; thrown at 3 m/s.
* Two hands on a dead soldier (a thigh and a foot): lifted together, held within 3 cm, dragged as
  the player walked backwards, dropped.
* The playtest save's dead bandit taken by the forearm (lifted, hanging head down) and by the upper
  arm (dragged 2.7 m: the game moved his reference with his body).
* A standing soldier's forearm lifted over his head and pulled out to the side, the rest of him
  standing (the capture shows his arm out straight in the hand); let go, back to his animation.
* With his AI on, his arm pulled back towards the player: he was pushed 40 cm after it, then yanked
  over; the game got him up again.
* The throat: a standing soldier lifted by the neck is knocked down by the game and hangs about
  10 cm below the hand; the choke takes his fatigue from 280 to nothing; he struggles; thrown
  aside; 7 s later the game queues his get-up, he gets up and stands (knock states 1, 5, 6, 0).
  Choked standing, his arms come up at the hand, the game knocks him out (knock states 4, 3) and
  he sags from the hand (or falls out of it).
* With his AI on, the choke is an assault: the player's bounty went from 0 to 40.
* A shove at 1.7 to 2.8 m/s (his character controller took the push); a dead body pushed by an open
  hand moved.
* A wolf (37 kg by the game's masses) and a goblin (210 kg) are published with their bodies
  (grabbable); taking hold of them wasn't tried.
* The strain is felt (haptic pulses counted by the emulated headset).
* The adapter's creatures part of the frame stays at 0.4 ms.

**Not checked in the game:** grabbing creatures that aren't people (wolves, goblins: they're held by
their own skeleton's bodies, as people are), grabbing someone mid-attack, how far a shove carries a
person in the room (it moved him a little in the test room), a real headset's feel (only the
emulated one). The framework's test range now takes grabs too (its dummies), built but not run (the
range wasn't to be started tonight).

## Decisions

* **The game simulates the body; the runtime has the hand.** The hand is drawn on the part where
  the game has it (so its weight holds the hand back, the way HIGGS draws it);
  the game moves the body with its own Havok ragdoll, by its own grab (`bhkMouseSpringAction`), so
  the body has the game's joints, masses, gravity and collision.
* **Oblivion's ragdolls weigh about eight times a person** (a man is 605 of its units). The game's
  own scale is taken from the player's own ragdoll (a person, 75 kg) and every body weighs in
  proportion, so a hand's pull in newtons means the same in the game.
* **The spring is stiffer than the game's own grab** (elasticity 0.7 against its 0.2): the game's
  drags a body a bone at a time, measured against the bone alone, so a whole body hanging from one
  bone sank 30 cm below the hand; now it hangs a hand's breadth below.
* **A choke knocks out, it doesn't kill.** It drains fatigue as far below nothing as it had (the
  game's knockout lasts until it's back). Nothing in Oblivion says how hard a choke is, so the rate
  is ours: a full squeeze empties a person in 6 s, 4 s off his feet.
* **Lifting knocks down** when the hand pulls the head, neck or torso 12 cm above where the
  standing body has it for an eighth of a second; a yank is
  55 cm away from it (a leg 35 cm); a throw that knocks down is over 2.5 m/s. The struggle (claws
  and kicks) is ours too: Oblivion has no animation for it.
* **A standing actor pulled is pushed, not walked**: his character controller's push (the game's
  own knockback without a ragdoll) after the pull past 12 cm, up to 2.5 m/s. He slides rather than
  steps (Oblivion has no step-to-follow); PLANCK makes Skyrim's actors walk, which needs its
  behaviour graph.
* One assault a hold, whatever it comes to (the game's own counts each call).
* No new protocol version: the new regions sit in unused space and a push's new field in its
  padding, so older adapters and runtimes still agree.

## Not done

* Landing doesn't hurt (Oblivion's ragdolls take no damage from impacts; a throw into a wall is
  harmless).
* A pin is only the world's (a body pushed into a wall or the floor stays there while held); a
  blade through a body into a wall is the melee and gore features' (not this branch).
* Holding someone doesn't tire you (PLANCK drains stamina; Oblivion has no value for it).
* The gravity gloves don't pull bodies.

## Branches

* chasms-vr-oblivion: `overnight/npcs`
* chasms-vr-framework: `overnight/npcs`
