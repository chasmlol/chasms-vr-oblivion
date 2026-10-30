# Grabbing NPCs: playtest

Save: rustvr-npcs

`tools\play.ps1 npcs` boots into **rustvr-npcs**: a room in the Imperial City with a Legion soldier
standing a metre in front of you (friendly, AI on) and a dead bandit (fur armour) lying at your feet.
To get more people: `player.placeatme 0018B118 1 100 0` (another Legion soldier),
`player.placeatme 0003DC25 1 100 0` (a hostile bandit boss); `tgm` keeps you alive.

## What to try, in order

1. **The dead bandit.** Reach down, put a hand on any part of him (an arm, a leg, the head, the
   torso) and squeeze the grip. The hand stays exactly where it touched him and the fingers close
   round the limb. Lift: that part comes up and the rest of him hangs from it and drags. Lift by a
   leg and he hangs upside down; take a second hand on another part and he comes up between them.
   Walk backwards holding him: he's dragged along the floor. Swing and let go: he flies, as fast as
   your swing, as far as your strength can throw his weight.
2. **The soldier's arm.** Take his forearm or wrist and lift it: it goes up over his head while the
   rest of him keeps standing (his animation goes on: breathing, looking about). Pull it out to the
   side: it follows to where his shoulder lets it. Let go: it eases back into his animation in a
   moment. Pull hard along the ground and he's pushed after your hand, stumbling along; yank it
   right away from him (about half a metre past where his body lets it go) and he falls.
3. **The throat.** Put a hand round his neck (the lower part of the head, or the top of the torso)
   and squeeze. Lift: once you've pulled his head up past where he can stand, his legs go and he
   hangs from your hand, his feet off the floor. Keep squeezing: his fatigue drains (watch his
   struggles weaken: he claws at your hand and kicks), faster while he's off his feet, until the
   game knocks him out. Let go, or throw him: he lies there, and gets up when the game says
   (a few seconds, longer after a choke).
4. **The law.** Choking him, knocking him over, throwing him, or holding him for more than four
   seconds is an assault, the game's own: you get the game's bounty (40 gold) and his reaction
   (a Legion soldier comes to arrest you, or fights). One assault a hold.
5. **Heavy and strong.** Holding a body that holds your hand back, you feel a low rumble in the
   controller, harder the further back it's held. Past an arm's reach (60 cm between your hand and
   the controller) your grip slips. Your strength is the game's: Strength 50 lifts a person
   one-handed by the throat; a weaker character sags more, a stronger one throws further. The
   Hands tab has "Strength on creatures" (1500 N a hand; times your Strength / 50).
6. **A fight.** Place a bandit boss and grab him while he attacks: his held arm is yours (it goes
   where your hand takes it), the rest of him keeps fighting.

## What was checked in the game (emulated headset, captures and logs)

* A dead soldier taken by the foot: the foot follows the hand within 2 to 6 cm as it's lifted
  0.7 m, the leg hangs from it; thrown at 3 m/s.
* Two hands on a dead soldier (a thigh and a foot): lifted together, held within 3 cm, dragged a
  metre as the player walked backwards, dropped.
* A standing soldier's forearm lifted over his head and pulled out to the side, the rest of him
  standing (the capture shows his arm out straight in the hand); let go, back to his animation.
* The throat: a standing soldier lifted by the neck is knocked down by the game and hangs about
  10 cm below the hand; the choke takes his fatigue from 280 to the game's knockout; he struggles;
  thrown aside; 7 s later the game queues his get-up, he gets up and stands (knock states 1, 5, 6,
  0).
* With his AI on, the choke is an assault: the player's bounty went from 0 to 40.
* The strain is felt (haptic pulses counted by the emulated headset).
* The combat part of the adapter's frame stays at 0.4 ms.

**Not checked in the game:** the stumbling push (the soldier was standing with AI off when pulled
along; whether his character controller moves him then wasn't measured), grabbing creatures that
aren't people (wolves, rats, minotaurs: they're grabbed by their own skeleton's bodies, untried),
grabbing someone mid-attack, a real headset's feel (only the emulated one).

## Decisions

* **The game simulates the body; the runtime has the hand.** The hand is drawn on the part where
  the game has it (so its weight holds the hand back, the way HIGGS draws it);
  the game moves the body with its own Havok ragdoll, by its own grab (`bhkMouseSpringAction`), so
  the body has the game's joints, masses, gravity and collision.
* **Oblivion's ragdolls weigh about eight times a person** (a man is 605 of its units). The game's
  own scale is taken from the player's own ragdoll (a person, 75 kg) and every body weighs in
  proportion, so a hand's pull in newtons means the same in the game.
* **A choke knocks out, it doesn't kill.** It drains fatigue to a quarter below nothing (the game's
  knockout lasts a few seconds after you let go). Nothing in Oblivion says how hard a choke is, so
  the rate is ours: a full squeeze empties a person in 6 s, 4 s off his feet.
* **Lifting knocks down** when the hand pulls the head, neck or torso 12 cm above where the
  standing body has it for an eighth of a second; a yank is
  45 cm away from it; a throw that knocks down is over 2.5 m/s. The struggle (claws and kicks)
  is ours too: Oblivion has no animation for it.
* One assault a hold, whatever it comes to (the game's own counts each call).

## Not done

* Landing doesn't hurt (Oblivion's ragdolls take no damage from impacts; a throw into a wall is
  harmless).
* A pin is only the world's (a body pushed into a wall or the floor stays there while held); a
  blade through a body into a wall is the melee and gore features' (not this branch).
* An open hand shoving someone (the hands push creatures' capsules, but Oblivion's adapter doesn't
  take those pushes yet).

## Branches

* chasms-vr-oblivion: `overnight/npcs`
* chasms-vr-framework: `overnight/npcs`
