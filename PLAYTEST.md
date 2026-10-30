# Grabbing NPCs: playtest

Save: rustvr-npcs

`tools\play.ps1 npcs` boots into **rustvr-npcs**: a room in the Imperial City with a Legion soldier
standing a metre in front of you (friendly, AI on) and a dead bandit (fur armour) lying at your feet,
half a metre to your right. To get more people: `player.placeatme 0018B118 1 100 0` (another Legion
soldier), `player.placeatme 0003DC25 1 100 0` (a hostile bandit boss),
`player.placeatme 0006E912 1 100 0` (a wolf); `tgm` keeps you alive, `tai` stops everyone's AI.

On the real headset the keyboard doesn't reach the game, so two .bat files in the worktree (not
committed) boot it with the console lines already run, each in god mode (`tgm`): `playtest.bat` is the
save as it is (steps 1 to 6 and 8), `playtest-fight.bat` adds a hostile bandit boss 4 m in front and a
living wolf 4 m behind (step 7, and a living creature that isn't a person). Each keeps the run's
runtime log in `<builds>\npcs\playtest-<n>\` once the game is closed.

The flat game's crosshair is hidden in VR (from the riding branch): only the icon of what you can
activate shows where you look.

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
   out. Let go, or throw him: he lies there and gets up when the game says (a few seconds after a
   knockdown, longer after a choke-out). Choked while he's standing, he sags from your hand (or out
   of it) when the game knocks him out.
4. **The law.** Choking him, knocking him over, throwing him, or holding him for more than four
   seconds is an assault, the game's own: you get the game's bounty (40 gold) and his reaction (a
   Legion soldier comes to arrest you, or fights). One assault a hold.
5. **Touch and shove.** Without grabbing, push the soldier's face, an arm, his chest: the part gives
   under your hand and springs back with a little wobble once you let it be. His head turns on his
   neck, loosely: push his cheek and he turns his face away, push his forehead and his head tips
   back; an arm swings from the shoulder, loosely; pushed in the chest, his head and arms swing
   with it; a leg hardly gives (he's standing on it). Slap an arm or his face and it swings away
   and back. Push him fast with an open hand (1.5 m/s or more) and he's shoved back
   as well.
6. **Heavy and strong.** Holding a body that holds your hand back, you feel a low rumble in the
   controller, harder the further back it's held. Past an arm's reach (60 cm between your hand and
   the controller) your grip slips. Your strength is the game's: Strength 50 lifts a person
   one-handed by the throat; a weaker character sags more, a stronger one throws further. A goblin
   weighs nearly three people in Oblivion's own masses: one hand hardly moves it, two might. The
   Hands tab has "Strength on creatures" (1500 N a hand at Strength 50).
7. **A fight.** Place a bandit boss and grab him while he attacks: his held arm is yours (it goes
   where your hand takes it), the rest of him keeps fighting.
8. **Letting go.** Your hand stays a free hand on his body where it let go, and comes back to the
   controller from there (it doesn't pass through him). A hand resting on someone still has the
   trigger: touch a shoulder and pull it to talk.

## Playtest 1 (real headset)

It works. Asked for:

* **Soft bodies without grabbing:** push a face, an arm, a chest and it should give and wobble.
  Now a standing person's bones about a touch go partly physical, through the game's own blend
  between animation and ragdoll (a heir and a velocity gain below 1: a spring that overshoots, read
  from its blend code). The arms hang loose (a lean moves them most of the way, and they wobble
  back over about a second), the spine and the neck are firm (a third of the way, back in a
  quarter of a second), the legs stiff. Left alone for 1.5 s, the bones blend back to the animation
  in 0.4 s. (Step 5.)
* **The hitboxes didn't cover armour:** they were fixed-size capsules of a bare body. Now each part
  is fitted to the models the game draws on that person (the skinned body, armour and clothes, the
  head, hair and helmet): as thick as nine in ten of its vertices reach, the torso as deep as its
  front and back, and reaching to the crown, the fingertips, the heel and the toes. Hands and feet
  are hitboxes of their own now (14 parts a person). Fitted again whenever what they wear changes.
  The adapter log has a line per person fitted (`shapes: ... fitted to N models`): each part's
  thickness and how far its ends reach.

Look out for: a hand stopping short of someone or sinking into them (and where), a part that
flops too much or too little when pushed, the face and arms coming back too slowly or too fast,
anything twitching while you hold someone.

## Playtest 2 (real headset)

The touches and the hitboxes worked, except the head: it couldn't be moved. The log shows each
person fitted (the soldier's head 0.15 m thick with his helmet, reaching 0.10 m past the head bone)
and touches arriving. The head was firm (half its body's turn drawn, and that a third of a lean),
and a push moved its body without turning it, while the game draws only the head bone's turn (at
the animated place). Now:

* **A push is an impulse at the point pushed** (Havok's own `applyPointImpulse`, as the game's grab
  pulls), as much as the point lacks of the push by the body's own mass and inertia. Off its middle
  it turns the body as well as moving it: a cheek pushed turns the head away, a forehead tips it
  back. Dead bodies are pushed the same way.
* **The head is loose** when it's touched (as the arms): leant on, it gives most of the way.
* Which bone a touch is on is logged (`grabs: ... touched on its bone N`: 1 is the head).

Found while looking (not changed): Oblivion turns the upper spine itself (its look-at twist, a
`bhkBlendCollisionObject` of its own kind on Spine, Spine1, Spine2) and draws those bones from the
animation, so a push on the chest doesn't bend the chest itself; the head and arms swinging with it
are what show. The game's head tracking only adds its look on top of the head's own turn, so it
doesn't undo a push.

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
* A dead wolf (37 kg by the game's masses) taken by its leg and lifted, splayed; a goblin is
  published at 210 kg.
* The strain is felt (haptic pulses counted by the emulated headset).
* The adapter's creatures part of the frame stays at 0.4 ms.

**Not checked in the game:** a living creature that isn't a person (a wolf alive, a goblin), grabbing
someone mid-attack (the test bandit kept out of reach), how far a shove carries a person (it moved
him a little in the test room), a body pinned against a wall, a real headset's feel (only the
emulated one). The framework's test range now takes grabs too (its dummies), built but not run (the
range wasn't to be started tonight).

## Decisions

* **A touch is the game's blend, not a spring of ours.** The bones about a touch get a heir and a
  velocity gain below 1 (loose 0.2 and 0.1, firm 0.5 and 0.3, stiff 0.7 and 0.5); Oblivion has no
  value for how much a body gives, so these are ours. The head touched is loose, the spine above
  it stiff. The pelvis stays the animation's: it holds him up. A held person isn't touched (the
  grab moves him).
* **The torso's thickness is its depth.** A capsule as wide as the shoulders would stop a hand
  in front of the chest; as deep as the chest, a hand on the waist's side sinks in a little (the
  arms hang there).

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
