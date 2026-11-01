# Dismemberment and gore: research and design

What players love and hate in the games that do it, how the AAA games keep blood cheap, what
Oblivion has (its actors' meshes, its skeleton and ragdoll, its blood art, its decals), and the
design built from that. Inspiration only: nothing is copied. The design is built: the framework's
`rustvr_protocol::gore`, the runtime's `gore.rs`, the avatar's `gore.rs` and the adapter's
`crates/oblivion/src/gore.rs`.

## What players love and hate

**VR melee with physical bodies** is what everyone measures against.

* Loved: limbs and heads come off where the blade really crosses a joint, at a right angle, with a
  hard swing; the severed parts stay physical and can be picked up; blood stays on the weapon and
  on bodies; wounds on bodies (cuts, exit wounds); a stump bleeds.
* Hated: dismemberment that feels random (too stingy: a one-handed sword that can't cut a throat;
  too easy in Half Sword, where a tap pops a limb); flesh that feels like a wooden dummy; blood
  that is too little, or decals that fade; frame drops from heavy bleeding mods; corpses that look
  silly. Players mod in what's missing: blood pools under stumps, drips along weapons, stump
  bleeds that last, and severing without killing.

**Others.** GORN is loved for cartoon excess (limbs torn off by hand, enemies that keep fighting
limbless). The Walking Dead: Saints & Sinners for resistance: a blade stuck in a skull to pull
out. Bonelab and its mods for blood that stays (stains on the ground, splatter on walls).
Players want blood on blades that runs and drips, and to flick it off. The pattern:
**dismemberment that depends on the joint, the angle and the force, so it can be learned; blood
that stays; stylised excess over clinical realism; and never a frame drop.**

**Flat games.** Fallout 3/NV dismember only on a killing blow (or a corpse), at fixed body parts
(`BGSBodyPartData`: severable, explodable, a gore effect, a limb replacement model), the body's
skin split into partitions with meat caps authored in pairs (one on the body, one on the limb).
Skyrim's decapitations (killmoves, and the SKSE mods after them) hide the head on the body, cap
the neck, and spawn the head; their users complain when the head doesn't match the NPC's face or
hair. Killing Floor 2 swaps to a gore mesh, breaks the limb's physics constraint, rigidly skins the
limb, and plays three effects at once: a jet from the stump, a spray from the flying limb, a
splash at the cut. Dying Light 2 hides the limb with a shader, covers the wound with a patch mesh
and spawns the piece. Oblivion's own mods (Unnecessary Violence, Deadly Reflex) decapitate on
skill-gated power attacks; vanilla Oblivion has none.

## How the AAA games keep blood cheap

* **Projected (deferred) decals**: a box drawn over the scene, each pixel's position rebuilt from the
  depth buffer and textured in the box's space. Cost is screen area only; no z-fighting; on the
  real visible surface. They float on skinned (moving) meshes, so not for bodies.
* **Budgets**: a fixed pool, the oldest recycled first (Shadow Warrior: 100 decals FIFO).
* **One atlas, one pass** (Doom 2016's clustered decals).
* **Cast once, don't simulate**: Source places blood decals with one trace from the wound; particle
  systems roll a chance per drop to make a decal. GPU particles that collide with depth miss what's
  off screen.
* **Masks on characters and weapons**: sphere masks in rest-pose space (constant cost, a handful of
  hits), or painted UV masks (The Last of Us). Wounds that grow and dry by animating a
  threshold.
* **Drips and runs**: a scrolled gradient through a drip mask, or a gravity map on the surface; the
  cheap way is a moving point that leaves a streak.

## What Oblivion has

**Actors' meshes, per slot.** An actor is its skeleton (`skeleton.nif`, `Bip01 ...`) with the worn
meshes attached under the root, each skinned to the skeleton's bones: the upper body
(`upperbody.nif` or the cuirass: the torso and the arms down to the wrists, skinned to the
clavicles, upper arms, forearms and their twists), the lower body (to the ankles), the hands
(`hand.nif` or gauntlets: skinned to the forearms, their twists, the hands and fingers), the feet
(`foot.nif` or boots: the calves, feet and toes), the head (`headhuman.nif`, FaceGen-morphed per
NPC: skinned to `Bip01 Head`, `Neck1`, both clavicles and `Spine2`), hair, eyes, mouth and the
helmet. So a hand or a foot is a mesh of its own; an elbow or a knee is the middle of a mesh; the
neck is shared by the head and the body.

**The skeleton's ragdoll.** Havok rigid bodies (layer 8, biped) on `NonAccum`, the three spine
bones, `Head`, both `UpperArm`, `Forearm`, `Hand`, `Thigh`, `Calf` and `Foot`, each held to its
parent's by a `bhkRagdollConstraint` or a `bhkMalleableConstraint` (the elbow, knee and ankle).
Their masses (head 30, upper arm 40, forearm 30, hand 15, thigh 45, calf 30, foot 15) are the
game's.

**Its blood, per creature.** A hit that does enough damage bleeds: a decal on the world behind the
victim, the creature's blood spray as a short-lived particle effect, and decals on the victim's own
skinned meshes. Creatures name their own decal texture and spray (their CREA record); NPCs take the
game settings' defaults: `sBloodParticleDefault` (`Effects\BloodSpray.nif`),
`sBloodTextureDefault` (`Effects\blooddecal.dds`), and the lich's, the skeleton's
(`skeletonBloodSpray.nif`: bone dust) and the will-o'-the-wisp's extras.

**Its decals** are cast a short way along the hit onto what they meet, one fixed size, under the
game settings `fDecalLifetime` (10 s by default: the "blood disappears" complaint) and
`iMaxDecalsPerFrame` (10). Fixed size and short-lived: no good for drips, runs or pools.

**Its gore art.** `textures\effects\blooddecal.dds` (people), `lichblooddecal.dds`,
`willothewispblooddecal.dds`; the spray's particles `blood01..06.dds`; the dungeons'
`bloodsplatter01`, `bloodstain01/02` and `drip01` (`textures\dungeons\misc`); Kvatch's severed
head on a pike (`meshes\clutter\kvatch\headpike.nif`) with its neck stump (`neck.dds`), brain and
bloody eye; the zombie's flesh and bone (`textures\creatures\zombie`). No gore cap meshes on the
bodies, no limb meshes: the head pike's neck stump is the game's only cap.

## The design, as built

The game is the world; the runtime is the player and what the player makes happen. The game draws
its bodies and its own blood as it always did (its spray, its decals); the gore system adds the
rest, from the game's own data and art.

### Framework (any game)

* **Protocol `gore`.** The adapter writes, every frame, its **gore bodies**: each creature's blood
  (an index into the **blood table**: the game's decal, splatter, stain, drip and particle textures;
  liquid, dust or glowing) and its **stumps** (where a severed limb was, on the body: the joint, the
  way out of the stump, the limb's radius). Its **severs** go to the runtime in a ring (a creature
  lost a limb: which, where, the piece's object, the cap's way out in the piece's space, the blade's
  push). The rule for when an edge cuts (`severs`) is the same for every game: a slash (`DMG_SLASH`,
  not glancing: never a stab or a blunt blow) on a limb, on a killing blow or a corpse, with the
  cut's power (the hit's strength times the weapon's damage against a reference blade's 10, times
  its kind) at least the limb's toughness (thigh 1.3, upper arm 0.9, calf 0.85, forearm 0.65, head
  and foot 0.55, hand 0.45; a corpse needs 0.8 of it).
* **The severed part** is one of the runtime's props (an `ObjectDesc` the adapter offers,
  grabbable and pullable): a box round the limb's bones and its reach past the last one (a hand's
  fingers, a head's crown), simulated by the runtime's physics against the world's collision the
  game gave it; the game draws it where the runtime has it. It leaves the body at half the body
  part's speed, pushed off the blade at 3 m/s.
* **Blood** (the runtime's `gore.rs`): a pool of droplets (3072 at most, the oldest recycled), each
  a point with gravity and air drag, collided against the world's static collision by a budget of
  160 ray casts a frame (each live drop is probed in turn, from where it was last probed to where it
  is), and against the player's hands, held things and body by distance. A fine mist with no
  collision goes with every spray. Spawned by:
  * **wounds**: a clean hit sprays from the cut along the blade's way and out of the surface (a
    slash fans, a stab jets out of the entry, a blunt hit spatters), as much as the hit's strength;
    the wound then bleeds a few seconds on the body part (it follows the part);
  * **stumps**: arterial pulses at a heartbeat (0.78 s), each a jet out of the stump: pumping for
    14 s, weakening into a flow until 45 s, then drips until two minutes; the severed piece bleeds
    from its own cut, shorter and weaker;
  * **blades**: blood the weapon carries drips from its lowest bloody point while it's wet, and runs
    down a blade held upright.

  A drop that lands becomes a **decal** where it lands (a splat, streaked along its way when it hits
  at a slant), or adds to a **pool** on a floor (pools grow from what lands in them); on anything
  steep it may become a **runner**: a point that slides down the surface, probed every few frames,
  leaving a streak, dropping off edges as a drop again. Decals are pooled (640 at most; the oldest
  go first, pools last) and dry over minutes (150 s, a pool 400 s), darkening; they don't fade.
* **Blood on the player**: splats on the held weapons (model space: along the blade that went in, as
  deep as it went) and on the body and hands (bind space, on the nearest bone), from hits and from
  drops that meet them; wet, then dry (90 s), then wearing away three minutes after blood last got
  there. Held things drip while wet, and a hard flick or shake throws the wet blood off.
* **Drawing** (the avatar's `gore.rs`), over the game's picture with its depth: the decals as
  projected boxes (the game's depth gives the surface, the box's texture the shape), drawn before
  the body; the caps on stumps (a dome of the limb's size over the cut, the game's flesh texture on
  it); the drops as sprites stretched along their flight, after the body; the splats on the
  player's body and what it holds go to the body's own shader. One instanced draw per kind of
  blood. All lit with the game's lights the body is lit by, fogged as the game fogs, and wet (a
  specular highlight while fresh); a glowing blood is drawn unlit, and dust is never wet.

### Oblivion

* **Blood**: each creature's own blood from the game (its base form's decal texture and spray
  model, as the hit asks for it), mapped to the game's art for it: people's red (the `blooddecal`
  atlas, the dungeons' `bloodsplatter01` and `bloodstain01`, the arena's `arenablooddecal01` for
  runs, `blood01` drops and `blood06` mist, the Kvatch head pike's `neck.dds` for the caps' flesh);
  a lich's own decal and drops; a will-o'-the-wisp's, glowing; a skeleton's bone dust
  (`bonedust.dds`, with the zombie's bone texture for its flesh). A creature with a decal of its own
  gets its own splats, the rest from its colour.
* **The cut**: right after the game's own hit lands (its damage, its death: the adapter's
  `combat.rs` calls `gore::after_hit`), the adapter asks the rule with the game's numbers: the
  weapon's damage, its kind (a one-handed blade 0.55 + 0.45 x its reach, so a dagger less; a
  two-handed blade 1.35; the blunt kinds, which hold Oblivion's axes, 1.15 one-handed and 1.45
  two-handed), the hit's strength, and the creature's health before and after. A cut is at the joint
  nearest where the blade struck the body part: the neck (people are cut at its base, `Bip01 Neck1`,
  so the head's mesh closes up under the collar; a creature's skeleton without one at `Bip01
  Head`), the shoulder (`UpperArm`), the elbow (`Forearm`), the wrist (`Hand`), the hip (`Thigh`),
  the knee (`Calf`) or the ankle (`Foot`). An upper arm or thigh struck in its upper half is cut at
  its top joint, lower down at the elbow or knee; a forearm or calf in its upper 40% at the elbow or
  knee, lower down at the wrist or ankle; the neck struck anywhere up to the jaw takes the head. A
  part cut off takes no more hits.
* **On the body** every node under the cut bone is drawn shrunk to nothing at the joint (its world
  transform written once a frame before the eyes, after the game's animation and ragdoll), so the
  meshes skinned to it close up there: a hand or foot mesh goes whole, an arm or leg tapers to the
  joint.
* **The severed part** is a clone of the actor's 3D (`NiObject::Clone`, without its
  controllers and collision: its meshes, its face, its hair, what it wears) attached beside the
  body, with every bone but the cut ones shrunk to nothing at the joint and every mesh that doesn't
  show the piece hidden, so only the limb shows, as it was at the cut; moved where the runtime has
  the piece. Its mass is the game's ragdoll bodies under the cut, scaled so a whole body weighs
  75 kg (the game's add up to about 630). The game's blood decals on the body's skin are copied onto
  the clone: a clone of a decal geometry comes without its list of decals, and the game's decal
  shader draws the first one without asking whether there is one (the crash when a head that had
  been hit came off). It stays while the body does.
* **The stumps** go to the runtime every frame (it pumps blood from them) and, for their first
  10 s, spray the game's own blood effect every 1.6 s (a temporary particle effect, as its hit
  makes one: `Effects\BloodSpray.nif` for people).
* **The game's own decals** last as the runtime's do: in VR `fDecalLifetime` is set to 600 s and
  `iMaxDecalsPerFrame` to 20, in memory (the player's `Oblivion.ini` is left as it is).
* **Testing**: the console line `GoreCut <bone>` kills the nearest creature and cuts that limb off
  it as a full-strength blow would (`GoreCut <bone> standing` leaves it alive).

**Not built:** caps made from the game's own neck stump mesh (the caps are the runtime's domes with
the game's flesh texture).
