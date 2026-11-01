# Armour that moves like armour

Research written before the armour work: cuirasses, pauldrons, gauntlets and greaves bent like
cloth on the player's body, because every vertex of the game's models was linear-blend skinned
(LBS) with the weights its artist painted. This is what the best games do about it, what players
notice, what Oblivion's models are, and the design built from it. All of the design below is
built (the framework's `rustvr_avatar::cor`, `armour.rs` and `rustvr_asset::armour`, the
`rustvr-armour` preview, and this repository's `content/armour/oblivion.toml`).

## What players notice

* **The hands and forearms, above all.** In VR the player's own forearms and hands are in view
  most of the time, at arm's length, from every angle; the rest of the body is seen looking down,
  and then mostly the chest, the belt and the legs. Reviewers of the games best known for hands
  (Half-Life: Alyx) credit them with the presence, and the complaint about Alyx is that it has no
  arms at all ([PC Gamer](https://www.pcgamer.com/alyx-needs-arms/), [Road to VR](https://www.roadtovr.com/these-details-make-half-life-alyx-unlike-any-other-vr-game-inside-xr-design/)):
  arms are wanted, but only if they're done well.
* **Joints doing strange things break immersion.** VR melee players call joints doing strange
  movements an immersion breaker, and some would rather see only hands; the arm going through what
  it holds or bending at unnatural angles is the named problem. The other half: hands must stay
  1:1 with the player, so the body has to make the best of whatever pose the hands put it in.
* **Stretching and seeing inside the body** were the early complaints about Skyrim VR's VRIK (a
  forearm stretched instead of an elbow showing, the inside of the character visible)
  ([r/skyrimvr](https://reddit.birdcat.cafe/r/skyrimvr/comments/aqdex5/vrik_v01_beta_release)),
  and gauntlets clipping with the hands is a standing issue there (mods replace the VR hands with
  the gauntlets' own). The framework already fits each arm segment joint to joint so the arm
  never tears; what's left is the shape of the skin on it.
* **Wrists.** The wrist turns more than any other joint in VR (every grab, every pour, every look
  at the back of the hand). With LBS a turned wrist collapses into a twisted ribbon (the "candy
  wrapper") and a bent one pinches; a metal cuff bends with it like rubber. An Oblivion player's
  own report: the game's wrist irons clip "when the hands are flexed" because they're rigged to the
  hand bone ([Nexus forums](https://forums.nexusmods.com/topic/3341185-help-with-wrist-irons-please)).
* **Plates bending.** Even AAA games show rigid plates bending unrealistically now and then; it's
  the thing a player of a medieval game notices first on their own arm.

## How the AAA ones do it

* **Rigid plates are skinned rigidly.** Each plate is weighted wholly to one bone (or a helper
  joint), so it moves as one piece and can't bend; the cloth or leather under it keeps soft
  weights ([Polycount](https://polycount.com/discussion/comment/611018)). Plates overlap the next
  one (lames, a couter over the elbow, the cuff of a gauntlet over the vambrace) so the gap a
  joint opens shows the plate beneath, as real armour does.
* **Helper (corrective) joints** for what spans a joint: a pauldron, a couter, a poleyn, a tasset.
  The helper's rotation is a fraction of the joints either side of it (a parent constraint between
  two drivers, keyed or procedural), turning about the joint, so the plate stays over the joint
  instead of sinking into it; e.g. Skyrim's and WoW's shoulder helpers linked to the upper arm
  ([Polycount](https://polycount.com/discussion/comment/610569)). Engines drive them at run time
  from the pose: Unreal's Pose Driver (RBF interpolation of a bone's orientation driving
  corrective bones and blend shapes) is the general form
  ([Unreal docs](https://docs.unrealengine.com/4.27/AnimatingObjects/SkeletalMeshAnimation/AnimPose/PoseDriverNode/)).
* **Corrective blend shapes** sculpted per pose (elbow bent, arm raised), blended in by the pose
  driver. They need an artist per pose per mesh: not something the game's own models have.
* **Volume-preserving skinning.** LBS collapses joints (the elbow pinches, a twist collapses);
  dual quaternion skinning fixes the collapse but bulges at bent joints; Le and Hodgins's
  optimised centres of rotation (CoR, SIGGRAPH 2016) fixes both: each vertex turns by the blend of
  its bones' rotations about a centre precomputed from the rest pose and the weights, and that
  centre goes where LBS would put it, so nothing collapses or bulges
  ([Disney Research](https://studios.disneyresearch.com/?p=39548),
  [evaluation, Bittner 2019](https://www.graphics.tu-bs.de/publications/bittner2019evaluation)). It
  uses the same data as LBS plus one point per vertex, runs on the GPU at about LBS's cost, and
  needs nothing authored.
* **Layering.** Games with layered armour (Kingdom Come: Deliverance's cloth, mail and plate
  layers) keep each layer a little larger than the one beneath, hide what's fully covered and
  still have clipping as their weak point
  ([KCD forum](https://forum.kingdomcomerpg.com/t/layered-clothing-armor-what-is-so-special-about-it/8989/23?u=cerberus),
  [GDC 2025](https://gdcvault.com/play/1035519/Dressed-to-Quest-Achieving-Dynamic)).

## Oblivion's models

* **Skinning.** Gamebryo's hardware skinning: linear blend, at most four bones a vertex in each
  skin partition (`NiSkinPartition`); no dual quaternions. Every body part, armour and piece of
  clothing is skinned to the one skeleton (`characters\_male\skeleton.nif`, `skeletonbeast.nif`
  with a tail), which the extractor binds them to.
* **Helper bones.** The skeleton has `Bip01 * UpperArmTwist` (mid upper arm, under the upper arm),
  `Bip01 * ForearmTwist` (at the elbow, under the forearm), `Bip01 * Shoulder Helper` (at the
  shoulder, under the clavicle) and `Bip01 * Wrist Helper` (under the hand). The game's own
  animations key them; the framework's rig drives the twist bones along the forearm and the shin
  with a share of the hand's (the foot's) roll, by how far along the segment their skin sits, and
  keeps the others (the helpers, the upper arm's twist) rigid with their parents. The vanilla
  armour is weighted to the twist bones and clavicles, not the helpers.
* **How the meshes are cut.** Shapes are cut by material and skin partition, not by plate: a
  shape's name says the slot (`UpperBody`, `Arms`, `Hand`, `LowerBody`, `Foot`), and the left and
  right sides are in the same shape. Plates are its **islands** (triangles connected by shared
  corners, welded by position): the iron cuirass's pauldrons are two islands of shape 0 (78%
  clavicle, 22% upper arm twist), its couters two small islands at the elbows, its tassets four
  islands; the iron gauntlets are a cuff island (66% forearm twist, 33% hand) and a glove island
  per hand; the iron greaves' cuisses are islands (93% thigh) beside the leg's leather. A glove or
  a boot is one island, its plates and its leather together, so a plate is sometimes the part of
  an island on one bone.
* **Skin between pieces is the game's design.** An armour model carries the skin the game shows
  with it (the iron cuirass's forearms are the race's skin, `GamebryoSkin`), and gaps between
  pieces are in the game's own models; a gap is a bug only if the game doesn't have it in the same
  pose.
* **The slots** (biped mask, `crates/oblivion/src/body.rs`): an armour model replaces the race's
  skin for each slot it covers; what it doesn't cover is the race's skin.

## The design

### 1. Volume-preserving skinning for everything (framework, automatic)

Every soft vertex (any vertex on two or more bones) is skinned by optimised centres of rotation
instead of LBS. At load, off the frame (`rustvr_avatar::cor`):

* each triangle's weights are the mean of its corners', sampled finer where its corners' weights
  differ (split at the midpoints until they differ by under 0.1, to a depth limit), as the paper
  subdivides;
* each vertex's centre `p*` is the area-weighted mean of the triangle centres with weights like
  its own: `s(wp, wv) = Σ_{j≠k} wpj wpk wvj wvk exp(−(wpj wvk − wpk wvj)² / σ²)`, σ = 0.1, over
  every mesh of the body (the skin and the armour over it are the same limb);
* triangles are bucketed by the pairs of bones they carry, so a vertex only visits the triangles
  that share a pair with it.

Each frame the bones' palette carries, beside each bone's matrix, its rotation (a quaternion) and
its stretch (the symmetric rest), taken from its matrix (the rig's matrices aren't rigid: the
body's scale, the hands' own size, the arm segments' fit joint to joint). A soft vertex goes to
`R·S·(v − p*) + LBS(p*)`: R the normalised blend of its bones' quaternions (each turned to the
first's hemisphere), S the blend of their stretches, LBS(p*) where linear blending puts its centre.
A vertex on one bone is skinned exactly as before.

### 2. Plates (framework: the data and its application; per game: the marks)

A **plate** is a part of a mesh that moves as one rigid piece. It **rides** one or more bones with
shares: all its vertices get the same weights (the riders') and the same centre (the plate's
**pivot**), so the CoR formula turns the whole plate by one rotation about one point: rigid, with
no new bones, no CPU skinning and no new shader path. Riding two bones it turns by the share of
each about the joint between them: a couter turns half the elbow's bend about the elbow, a pauldron
part of the arm's raise about the shoulder, a tasset with the thigh about the hip; that's the AAA
helper joint, without the helper.

* **Riders**, by default, are the plate's own skin weights averaged (the artist's say of what it
  follows, made rigid): the game is the source of truth where it has the value. A mark may name
  them instead.
* **The pivot**, by default, is the joint between its two heaviest riders nearest the plate (the
  elbow, the shoulder, the knee, the hip; a twist bone at its parent's joint only rolls there and
  isn't one), else the plate's own centre.
* **Layers**: a plate may say it's over others (`layer`); where it meets a lower layer within a
  few millimetres, it's drawn over it (its depth taken 2.5 mm a layer towards the eye), so plates
  that overlap slide over one another instead of flickering through.

Found while building it, and added:

* **Plates are shaped at rest.** The models are bound with the arms out level; a pauldron the game
  drapes over the shoulder by bending it (its weights running from the clavicle to the upper arm)
  stood out like a wing, rigid in that shape, once the arms came down. Each plate now takes the
  shape the game's own skinning gives it standing with the arms at the sides
  (`rig::rest_pose`), and rides copies of its riders bound in that pose (bones that move exactly
  as the ones they copy). With that, the game's own weights are the right riders for a pauldron.
* **Firm corners.** A plate made of part of an island (a sleeve's upper-arm part, a cuff's
  forearm part) takes only the corners with most of their weight on its bone (`firm`), so the band
  blended across the joint stays soft and the joint bends smoothly between rigid plates.
* **Studs** (islands under 3 cm) within a centimetre of a plate move with it.
* **Two-sided plates.** Oblivion makes a plate seen from both sides (the iron pauldrons and
  tassets) as two surfaces in one place facing apart; drawn two-sided at once they fought and the
  dark back usually won. Front faces are now drawn first and back faces a little further off.

### 3. The marks (`content/armour/<game>.toml`)

Hand-written per piece, for modders to read and add to: which models a piece is, its plates
chosen by mesh, island and bone, what each rides and its pivot. Everything not marked keeps its
own weights and gets the volume-preserving skinning, so an unmarked mod's armour is still better
than today, and marking it is a few lines. `docs/ARMOUR.md` in the framework is the guide to the
format. The runtime reads the game's file from the adapter repository (`content/armour/
<game>.toml` beside its `local_assets/`), or `RUSTVR_ARMOUR` (a file) instead, to try an edit
without rebuilding. A modder's own marks for their own models go in
`local_assets/<game>/armour/*.toml`, read after the game's.

### 4. The preview (`rustvr-armour`)

A headless tool in the avatar crate that composes a body from a game's skeleton and pieces (or a
named set from the marks file), poses it through a sweep (rest, arms raised, arms crossed, elbows
fully bent, wrists turned both ways, wrists bent, reaching, holding a gun, a squat) and renders
each pose with the runtime's own renderer, from the front, the side, the back, the wearer's own
eyes, and close on the right forearm, shoulder, elbow (side on), wrist and the back of the hand,
into contact sheets (`captures/armour/<set>` by default). It renders the same pose skinned three
ways (the game's LBS, with volume-preserving joints, and with the marks), colours islands and
plates for marking, draws each marked model's textures with its plates filled in (for marks
chosen by texture coordinates), and lists every mesh's islands (their bones and bounds) and the
plates its marks make, so a modder can write a mark from the listing.

### 5. Oblivion

The vanilla sets marked, men's and women's: iron, steel, leather (the game's leather armour uses
the thief's models; the `leather` models too), chain mail, fur, elven, glass, dwarven, orcish,
ebony, daedric, mithril, the Legion's and the Blades' (each a `[[set]]` of
`content/armour/oblivion.toml`, the women's as `<set>_f`). Checked in the preview through the
poses and in the game (the iron of `rustvr-base`, the developer's own test save; steel, glass and
daedric given and equipped by console).

## Not done this way, and why

* **CPU-driven helper bones** (pose drivers, RBF): the rider shares and the pivot give the same
  rotation as a two-driver parent constraint, and an RBF needs authored poses per piece.
* **Corrective blend shapes**: none in the game's models, and an artist per pose per mesh.
* **Physics or collision between plates**: the plates' shapes and riders keep them apart in the
  swept poses; the layer bias covers what's left.
