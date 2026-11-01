# Grabbing NPCs: research and design

NPCs as physical bodies: grab any part and move it, the body following softly; grab the throat
and lift them off the ground; throw them; pin them. This is what was found (October 2026), and
what's built from it (the framework's `rustvr_protocol::grab` and `rig/creatures.rs`, the
adapter's `grabs.rs`). Everything below is facts and numbers in our own words, for inspiration:
nothing is copied. They come from the players' own words and the mods' published source.

## What players love and hate

**VR melee with physical bodies** is what everyone measures this by. What players love: grabbing
anyone by anything, holding a body up by the neck one-handed, flinging people, using a body as a
shield. What they complain about, again and again:

* **Noodles.** Once grabbed or stabbed an NPC "just goes floppy", and players ask for ragdolls
  "slightly stiffer". The opposite complaint is bodies "far too resistant", like "a wood or stone
  training dummy". The target is narrow: too stiff is a mannequin, too soft is a noodle.
* **No weight.** Lifting a body doesn't feel heavier than lifting a sword, and bodies get launched
  across the room by accident; top-heavy bodies are a common complaint.
* **Choking is missing.** Players have asked for it for years; holding the neck does nothing. Mods
  add it: a squeeze to choke faster, two hands faster still, a time or force to knock out or break
  the neck.
* **Neck grabs spin** the dangling body.

**Boneworks and Bonelab.** Praised for weight: heavy things must be moved as if they're heavy, a
sledgehammer is almost impossible one-handed, and "in the real world you can only move your hands
so fast". The avatar's strength sets how hard the hands can move things.

**Half-Life: Alyx** has no grabbing of bodies on purpose: without force feedback, a physical
simulation does things that never reach the hand, so Valve didn't lean into it. What the hand feels
has to be shown instead.

**The Walking Dead: Saints & Sinners.** Walkers are physical in the upper body while they walk
(Unreal's physical animation), fully ragdolled when dead. Grabbing a head makes them passive,
which players call silly but necessary.

**Skyrim VR with HIGGS and PLANCK** (the same old Bethesda engine family as Oblivion): players grab
NPCs "as you would a dead body" and drag them, and they walk along; they hold them by the neck,
lift and throw them, and slap them, the heads reacting and coming back. The complaint is that it
can break the game (guards not caring, AI confused).

**What follows from it:**

1. Two layers: the animation keeps an NPC standing and alive, physics bends it where it's held. A
   body that goes limp the moment it's touched reads as a noodle.
2. A limb lifted carries the body after it softly (the parent follows where the child really is).
3. Weight must be felt without force feedback: the hand stays on what it holds and is held back,
   a ghost shows the controller, and strength is limited (one hand lifts a person slowly, two
   hands easily; a heavy creature can't be lifted).
4. Standing NPCs keep their feet when pulled: they step after the pull, and fall only when yanked
   or lifted.
5. Grabs never snap (the hand takes hold where it is), the hand never stretches (past arm's reach
   the grip slips), nothing explodes (velocities capped, the hand ignores the body it let go of
   until it's clear).
6. Choking: on the throat, harder with the squeeze, the body lifted off its feet, a reaction that
   sells it (kicking, the game's own knockout), and the game's social rules (it's an assault).

## How the others do it

**PLANCK and HIGGS** (Skyrim VR; their source is published): Skyrim's ragdoll driver runs on every
NPC near the player, mostly keyframe tracking with 5% joint motors. Its gains: hierarchy 0.6 (the
child follows where its parent actually is, not where the animation has it: Havok's default is
0.17), velocity 0.6, position 0.05, gravity off while it's tracking. The joint limits are eased to
the animated pose so they don't fight it, and tightened back over 0.7 s when it ragdolls. The grab
is a motor (2500 N at most for an actor, soft, and eased in over 0.1 s so it never snaps); past
0.7 m from the controller it lets go. A held NPC's character controller walks and turns towards
where its body is pulled (it starts past a threshold, stops below a lower one). An NPC goes fully
ragdoll only through Skyrim's own knockdown (`PushActorAway`, force 0): a foot yanked away, a hand
let go faster than 4 m/s. While its ragdoll is held it can't get up. Aggression builds with the
time held: complaints after half a second, angry at 4.5 s, the assault alarm at 9 s; followers and
friends are exempt. Holding drains the player's stamina (3/s, 10/s for a hostile NPC) and slows
them by the NPC's size.

**Everyone else**: PuppetMaster (inside Bonelab too) pins each muscle to its animated target, loses
its pins when knocked past a distance, regains them at a speed, and gets up; Unreal's physical
animation sets a strength per bone; Euphoria's characters stagger, re-step and fall from behaviours.
All of them: animation for balance, physics for contact, and a state where the body is only physics.

## Oblivion's bodies

**Its ragdoll is the skeleton.** Each bone of `skeleton.nif` that has a Havok body carries a blend
collision object (`bhkBlendCollisionObject`) with two gains, **heir** and **velocity**. At heir 1
the body follows the animated bone; at heir 0 it's dynamic and the bone is put where the body is,
a ragdoll; between, the bone is drawn partway from the body's pose to the animation's and the body
is driven towards it with the velocity gain. So a heir and a velocity gain below 1 are a spring that
overshoots: pushed, the body comes back past the animation and settles, a wobble. All of
`skeleton.nif`'s bodies have gains 1 and 1; the game changes them itself as a body goes physical
and back.

**The head tracking undoes a push on the head.** The game turns the head to what the actor looks
at from the head's drawn turn, so a push on the head is turned back within a few frames. For a
head the hands have, the adapter holds the look where it was and puts the push on top of it.

**What's drawn on an actor.** Its body, armour and clothes are skinned meshes under its 3D, each
vertex placed by its bones' world transforms. The head, its hair and eyes are rigid, under
`Bip01 Head`.

**Knockdown.** The game's knockdown (the one `PushActorAway` makes) turns every bone physical and
pushes the bones away from the pusher by the force. A knocked-down actor gets up once enough game
time has passed and its body has settled (the get-up animation, the bones blended back to it over
a second); a knocked-out one (fatigue below 0, or paralysed) stays down until that ends.
**Paralysis** freezes the joints, which is why paralysed bodies fall stiff. **Death** blends the
bones to ragdoll over a quarter of a second; a dead body is Havok's from then on.

**The game's own grab** (holding Activate on something) picks the body under the crosshair and
hangs it on a `bhkMouseSpringAction`: a point on the body pulled towards a point in the world, with
damping, elasticity and a most force relative to the body's mass. It refuses living actors (unless
`bAllowHavokGrabTheLiving`) and keyframed bodies, moves the target every frame and lets go 96
units off. Dead bodies are dragged by it, a bone at a time.

**The game's reactions.** Every melee hit ends with the victim's reaction to being attacked:
disposition, friendly-fire tolerance, fleeing or yielding, combat, and the assault alarm (the
bounty, the witnesses). Fatigue below 0 knocks an actor out by itself, until it comes back. An
actor without a ragdoll can still be pushed through its character controller, the game's own
knockback.

**Looking at the player.** Every look at someone (the idle pick, dialogue, combat, `Look`) aims the
head, the eyes and the upper body's twist at the target's head. The player's, in first person, is
the first-person camera's, which only the thumbstick moves: so `crate::look` makes it the
headset's. In the dialogue menu the game may not run its people's AI, so whoever's looking at the
player has their face's look set each frame too, the same way.

## The design

### The framework (any game)

* **Creatures' bodies.** The game already sends each creature near the player as capsules on its
  bones. It now also sends what grabbing needs (`ActorBodies`, parallel to `Actors`): each part's
  rotation (so a hand on a turning torso goes round with it), its velocity and mass, the whole
  body's mass, whether the body is physical now (a ragdoll: down, dead, lifted) and conscious, and
  how strong the player is (the game's own strength, 1 for an average person: in Oblivion,
  Strength 50).
* **Taking hold.** A hand whose grip closes with its palm or a finger on a creature's capsule (and
  nothing else to take nearer) takes hold of that part where it touched it: neither the hand nor the
  body moves, the fingers wrap the capsule from where they are. Anywhere: an arm, a leg, the head,
  the torso, a dead body. Two hands can hold one creature (different parts or the same).
* **Held.** The hand is drawn on the part where it took it, wherever the game has the part (the
  ghost hand shows the controller when they're apart). It never passes through: it's on the
  surface. Each frame the runtime sends the game the grab (`CreatureGrabs`, one per hand): the
  point held (in the part's own frame), where the hand would have that point now (the controller's
  pose carrying the hold), the rotation, the hand's velocity, the squeeze, and the force the hand
  can pull with (the runtime's "Strength on creatures" setting, 1500 N by default, x the game's
  strength; two hands each pull). The game moves its body with its own physics; the part follows
  as far as the body lets it, and the hand with it. That's the weight: a light arm follows closely,
  a body hangs and lags, a heavy creature barely moves. Held back past arm's reach (0.6 m between
  the hand and the controller, for 0.15 s) the grip slips. A hand held back more than 4 cm feels
  the body's weight as a strain in the controller, full at 30 cm.
* **The throat.** A hand on the neck (the lower 45% of the head's capsule, or the top 10 cm of the
  torso's) is on the throat (`GRAB_THROAT`): the squeeze past the grip's hold is the choke's
  strength, sent each frame.
* **Letting go.** The grab is sent once more as let go, with the throw: the hand's velocity at its
  peak before the release, from the runtime's throw estimator (no gain: a body isn't a grenade).
  The hand is a free hand again where it is, on the body's surface: held back by the body if
  its controller is in it, and catching up eased as a held-back hand does.
* **A hand that holds a creature** is busy: it doesn't punch or cast.
* Pins are the world's: a held body pushed into a wall or the floor stays there, the hand on it.

### Oblivion

* **Its own ragdoll.** A grab is the game's own grab: a `bhkMouseSpringAction` on the Havok body of
  the bone held, at the point held, pulled to where the runtime says, with the runtime's force (as
  the force relative to the body's mass the action takes). Dead bodies are already all Havok's: any
  part drags the whole body, as the game's own grab does. The spring is stiffer and livelier than
  the game's own (elasticity 0.7, damping 0.6, the body's own damping 0.85, against the game's 0.2,
  0.5 and 0.75, which drag a body a bone at a time), so a whole body hanging from one bone hangs a
  hand's breadth below a hand strong enough to lift it, and a swing carries.
* **A living NPC** keeps animating. The bones from the one held out to its end (an arm from the
  shoulder, a leg from the hip, the head) go physical (heir 0): the limb hangs from the body that's
  still animated, and the hand moves it. The spine goes partly physical (a heir between), so the
  body leans after a pulled arm, softly, and comes back (the spine's heir 0.5). Let go, the limb
  blends back to the animation over 0.4 s.
* **Off its feet.** Lifted (the hand holding a neck, head or torso 12 cm above where the standing
  body has it, for 0.12 s), yanked (an arm, the head or the torso pulled 55 cm from where the body
  has it, a leg 35 cm: a trip), or let go of faster than 2.5 m/s, the NPC goes down: the game's own
  knockdown (its knockback with no force, every bone physical), and then the whole body hangs
  from the hand. While it's held it doesn't get up (its knockdown time is kept fresh); let go, it
  lies there and gets up the game's way.
* **Pulled while standing**, more than 12 cm along the ground from where its body has the point
  held, it's pushed after the pull (its character controller's push, the game's knockback without
  a ragdoll: 6 m/s for each metre further, at most 2.5 m/s), so it stumbles along rather than
  standing there stretched.
* **Touched while standing**, it gives under the hand and springs back (Saints & Sinners' and
  PLANCK's physical bodies, without a grab): the bones about the touch go partly physical (the
  game's blend, a heir and a velocity gain below 1: a wobble), loose for what hangs (the arms, the
  head), firm for the spine, stiff for a leg it stands on, and the body nearest the push gets what
  its point there lacks of the push, as an impulse at that point (Havok's `applyPointImpulse`, by
  its mass and inertia, as the game's mouse spring does), so a push off its middle turns it: a
  cheek pushed turns the head away. Let be for a second and a half, they blend back to the animation. An open hand (or what it holds) coming in
  faster than 1.5 m/s shoves the actor instead, as fast as it's pushed up to 3 m/s, for a quarter
  of a second (a firm push moves a person about 40 cm). A body that's all physics is pushed
  however gently. (The pushes are the runtime's `GamePushes` on creatures' parts.)
* **Masses.** Oblivion's ragdolls weigh more than people do (a man is about 600 of its units), so
  the player's own ragdoll is taken as 75 kg and every body weighs in proportion.
* **Its body is as thick as what it wears.** The capsules the hands meet are fitted to the models
  drawn on it (the skinned body and armour, the head and hair): each vertex goes with the bone that
  moves it most, a part's thickness is where nine in ten of its vertices are, the torso's its depth
  (front to back), and its free ends (the crown, the fingertips, the heel and toes, the hips) reach
  its furthest vertices. Hands and feet are parts of their own.
* **Choking**: the throat squeezed drains the NPC's fatigue (the game's own damage to it: all of
  it in 6 s squeezed fully, 1.5 times as fast lifted off its feet), until its fatigue is gone and
  the game knocks it out itself. The choke takes it a whole fatigue's worth below nothing, so it
  stays out a while after it's let go. While it's still conscious it struggles, weaker as it
  tires: its arms go physical and its hands clutch at the hand at its throat, and held off its
  feet its legs kick every 0.25 to 0.6 s.
* **The reaction is the game's.** Choking, lifting, throwing or knocking down a living NPC, or
  holding one for 4 s, is an attack: the game's own reaction to an attack by the player (its
  disposition, tolerance, fleeing, combat and the assault crime with its bounty, as a blow would).
* **Throws**: let go, the bodies keep what the spring gave them, and the held one, and the rest of
  a lifted body with it, get the throw's velocity, as much of it as the hand's pull can give that
  mass over 0.3 s.

**Not built** (on main): a held NPC breaking free (escape tries; the protocol has
`ActorBody::let_go` for it, but the adapter never sets it), and holding costing the player stamina
or slowing them (PLANCK's).
