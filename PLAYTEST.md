# Playtest: dismemberment and gore

Save: rustvr-gore

`powershell -File tools\play.ps1 gore` boots it. `-Save rustvr-gore-undead` has a zombie and a
skeleton instead. `-Close` puts it away.

## The saves

* **rustvr-gore**: inside the house at Weye. Three bandits stand round the room; they don't
  attack until hit, and each has 20-25 health, so two or three good blows kill one. You carry a
  steel longsword (equipped, at the left hip), a Daedric claymore, a steel battle axe and an iron
  dagger. Your health is 1000, so you can take your time.
* **rustvr-gore-undead**: the same room with a zombie and a skeleton. They have their normal
  health and do attack.

## What to try, in order

1. **Hit a bandit.** Draw the sword and cut one. Blood should spray from where the blade met the
   body, as hard as you hit:
   * a slash fans out along the blade's way and out of the cut;
   * a stab jets back out of the hole;
   * a blunt blow spatters.

   Fine mist goes with each spray. The wound keeps bleeding for a few seconds, and it moves with
   the body part. The game's own hit blood (its spray and its decals) still happens too.
2. **Watch where it lands.** Drops fly with gravity and land on the floor and walls as the game's
   own blood art:
   * splats from its blood decal;
   * long splatters where a fast drop strikes at a slant;
   * stains that grow into pools where a lot lands;
   * runs that creep down walls and drop off edges.

   Blood darkens as it dries over a few minutes. Nothing vanishes after a few seconds; the oldest
   goes only once 640 decals are down. The game's own blood decals now last 10 minutes in VR
   instead of 10 seconds.
3. **Look at your sword and hands.** The blade is bloody along the part that went through:
   * after a cut, along the edge;
   * after a stab, back from the point.

   It drips from its lowest wet point while it's wet and dries over about 90 s. Drops that land
   on your hands, or on the blade in the air, stay on them.
4. **Take a head.** Swing across the neck, anywhere from its base up to the jaw, with the edge
   leading, as the killing blow. The head should come off **with its neck**, wearing the NPC's own
   face, hair and helmet.
   * It flies off a little along your swing and drops with physics.
   * The body falls with a capped stump in its collar.
   * The stump pumps blood at a heartbeat for about 14 s, weakening, then flows for about half a
     minute, then drips. With each strong pulse it also sprays the game's own blood effect.
   * Pools form under it.
   * The severed head's cut bleeds too, more weakly.

   A blow on the skull above the jaw doesn't take the head.
5. **Arms and legs.** A killing blow on a limb cuts it at the nearest joint:
   * an upper arm: the shoulder or the elbow;
   * a forearm: the elbow or the wrist;
   * a thigh: the hip or the knee;
   * a calf: the knee or the ankle.

   Heavier blades cut more easily. The claymore and the axe take legs; the iron dagger hardly takes
   anything.
6. **Corpses.** Hit a dead bandit's limbs or neck with a good blow: they come off (a corpse gives
   way a little more easily).
7. **Pick up the pieces.** A severed head or limb is a physical prop: grip it, or pull it with the
   gloves, and throw it. It stays until the body goes (the cell unloads or a save loads).
8. **rustvr-gore-undead.**
   * The zombie bleeds the game's red blood and has flesh stumps.
   * The skeleton bleeds no blood, only its bone-dust puffs, and its stumps are bone.

## The rule (the framework's `rustvr_protocol::severs`, with Oblivion's numbers)

A hit cuts a limb off when all of these hold:

* it's an edge: a slash, or an axe's chop;
* it's clean: not glancing, not a stab, not blunt;
* it kills, or the body was already dead (as Fallout does it);
* the cut's power is at least the limb's toughness.

**Power** = the hit's strength × (the weapon's damage ÷ 10) × the weapon's kind. Damage is the
game's own number for the weapon. Kinds:

* one-handed blade: 0.55-1.0 by reach (a dagger less);
* two-handed blade: 1.35;
* war axe: 1.15;
* battle axe: 1.45.

**Strength** comes from the swing: 1 is a full-strength hit, at about 7 m/s.

**Toughness:**

| Limb | Toughness |
|---|---|
| Wrist | 0.45 |
| Neck | 0.55 |
| Ankle | 0.55 |
| Elbow | 0.65 |
| Knee | 0.85 |
| Shoulder | 0.9 |
| Hip | 1.3 |

A corpse needs 0.8 of the toughness. So:

* a steel longsword swung moderately takes a head on a killing blow;
* a hip needs a heavy blade swung hard;
* a dagger doesn't get through a thigh at all.

## What was checked in the game (emulated headset, `tools/oblivion.ps1`)

* **A real swing took a head.** A steel longsword swept across a bandit's neck as the killing blow
  (health 5 → −2) gave power 1.47 against 0.55: "cut off". The head was cloned from the NPC's own
  3D: its face, hair and armour pieces.
* **A real swing took a forearm.** The same sweep at elbow height cut the bandit's left forearm at
  the elbow (power 1.47 against 0.65).
* **A severed head was pulled and thrown.** The gloves pulled it from the floor (point, grip, flick
  back) into the hand, where the NPC's face hangs from the fist, then threw it.
* **Cuts at every joint looked right.** The `GoreCut` test command took the head, a forearm, a
  calf, an upper arm and a thigh off a bandit, a zombie and a skeleton. The pieces fell and rolled
  on the floor, drawn with their own meshes.
* **The stumps looked right.**
  * The headless body standing (AI off) shows a closed collar with a stump.
  * The severed head lies on the carpet with its face.
  * The zombie's neck is capped with flesh.
  * The skeleton's arm is capped with bone and puffs dust.
* **Blood:**
  * sprays from wounds and stumps, landing as splats, splatters and pools;
  * pools under a decapitated body;
  * blood on the blade along the edge that cut, seen in captures;
  * red splats on the hand from drops.
* **A save loaded with severed bodies and pieces**: no crash; the pieces go with their bodies.
* **The game's own blood decals** are set to last 600 s (the log says so).
* **Frame rate**, below.
* **Blood dust from a skeleton**: the cut puffed grey dust.

## What wasn't checked

* **A real swing that cuts a leg, or cuts into a corpse.** These go through the same code as the
  neck and the forearm. The checks used `GoreCut` for legs.
* **Taking a piece by its hand grip** (not the gloves).
* **Blood running down a wall.** Runs started in the logs. None was seen in a capture.
* **The game's own blood spray at the stumps.** It's called every 1.6 s for 10 s. It wasn't picked
  out in a capture.
* **The real headset**, lighting outdoors, and fog.
* **Creatures without the people's skeleton.** Wolves, bears and rats can't lose limbs: their
  combat body is one capsule. Zombies, skeletons and other `Bip01`-limbed creatures can.

## Frame rate (same scene, your eye size 4147×4406, emulated headset at 120 Hz)

The scene: the bandit room, one bandit killed. **Before** with no gore. **After** with five limbs
cut off one bandit at once (head, a forearm, a calf, an upper arm, a thigh). That gives 640
decals, 157 of them pools, five stumps, five severed pieces, and up to 300 drops in the air while
it sprays.

| | Before | After (spraying, then at rest) |
|---|---|---|
| Game frames/s (unthrottled) | 207–224 | 176–203 |
| Game's two eye renders, ms/frame | 2.3–2.4 | 2.9–3.2 |
| Runtime frames (120 Hz) | 120.0/s, 0 late | 120.0/s, 0 late |
| Runtime CPU, ms/frame | 0.4 (95% under 0.5–0.6) | 0.5 (95% under 0.6–0.8) |

* **The game:** the extra 0.6 ms is the five pieces' clones. Each draws only the meshes that show
  its limb. Before that fix the five cost 1.4 ms.
* **The runtime:** blood costs about 0.1 ms of CPU, under 160 world ray casts a frame, and no
  dropped frames. Decals, caps and drops are a few instanced draws per kind of blood (one per texture).
* **The cap:** 3072 drops, 640 decals, 96 runs and 48 splats on the player, the oldest recycled.

## Decisions

* **The game decides the cut.** The adapter asks the framework's rule right after the game's own
  hit has landed. It needs the game's damage and whether the blow killed. Damage and power attacks
  stay the melee agent's.
* **Limbs come off only on a killing blow or a corpse.** This is Fallout's rule, and the one in
  your spec. Oblivion's AI can't fight on with a missing arm. Severing without killing (VR
  players love it; mods add it) isn't done.
* **A head goes with its neck.** It's cut at the neck's base (`Bip01 Neck1`). Oblivion's head mesh
  is skinned down to the clavicles, so a cut at the head bone left a cone of neck on the body.
  Creatures without `Neck1` are cut at the head.
* **How a limb is hidden:**
  * on the body, every node under the cut bone is shrunk to nothing at the joint, written before
    each eye's render (hands and feet are meshes of their own: they go whole);
  * the piece is a clone of the creature's whole 3D (`NiObject::Clone`, controllers and collision
    taken off for the copy), with every other bone shrunk away and only the meshes that show the
    limb drawn.

  So the piece wears exactly what the NPC wore, with its own face.
* **The pieces are simulated by the runtime's physics,** not Havok. It's the same world collision
  the game's Havok has, and it makes them grabbable for free, as Metro's props are.
  Havok-simulated pieces would need Havok bodies made at runtime; that isn't worked out
  yet.
* **The stump caps** are a dome the runtime draws, textured with the game's own neck stump from
  the Kvatch head pike (`clutter\kvatch\neck.dds`, the zombie's bone for dust). Oblivion has no
  gore-cap meshes.
* **The blood art is all the game's:**
  * splats: its blood decal atlas;
  * splatters: the dungeons' `bloodsplatter01`;
  * pools: `bloodstain01`;
  * runs: the arena's drip streaks;
  * drops: the spray's `blood01`;
  * mist: `blood06`.

  Each creature's kind comes from what the game says it bleeds: its decal and spray. The lich's
  is black, the will-o'-the-wisp's glows, the skeleton's is dust.
* **The torso's combat capsule now ends at the chest** (`Spine2`) instead of the neck. Its round
  end covered the whole neck, so no blade could reach it.
* **Body part ids are now the capsule table's index**, so a hit says which bones it struck. This
  touches `crates/oblivion/src/combat.rs`, which the melee agent also edits: merge with care.
* **Test command:** `GoreCut <bone> [standing]` in the console test file. It cuts a limb off the
  nearest creature: kills it first, or leaves it standing.

## Not done

* Severing without a kill, and a hand that drops its weapon (the game's disarm) when cut.
* Bisection, gibs from blunt weapons, and wounds painted onto NPCs' skin. The game's own
  hit decals on their meshes still appear.
* Wiping or washing blood off a blade. It only dries and fades.
* Severed pieces don't persist across loads or cell changes. They go with the body; the body's
  limbs come back when its 3D reloads.
* Blood doesn't collide with NPCs or props in the air (only the world's static collision and the
  player).
* Blood isn't drawn into the game's HDR or bloom: the runtime draws it as it draws the body.
