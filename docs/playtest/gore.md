# Playtest: dismemberment and gore

Save: rustvr-gore

## Playing it

From the repository's root:

```
powershell -File tools\oblivion.ps1 -Save rustvr-gore
powershell -File tools\oblivion.ps1 -Save rustvr-gore-undead
```

It builds main, installs it, and boots the save; `powershell -File tools\oblivion.ps1 -Close` puts
it away.

- **rustvr-gore:** inside the house at Weye. Three bandits stand round the room (`ff000c48`,
  `ff000c49`, `ff000c4c`); they don't attack until hit, and each has 20-25 health, so two or three
  good blows kill one. You carry a steel longsword (equipped, at the left hip), a Daedric claymore,
  a steel battle axe and an iron dagger, and have 1000 health to take your time.
- **rustvr-gore-undead:** the same room with a zombie and a skeleton, at their normal health, who
  do attack.

To cut a head off a bandit that stands still, hold all three with the game's own prisoner hold
(they don't move or fight back, but still die and fall) and put one 1 m in front of you on 5
health, so the first good blow across its neck takes the head:

```
powershell -File tools\oblivion.ps1 -Save rustvr-gore -Commands "ff000c48.SetRestrained 1|ff000c49.SetRestrained 1|ff000c4c.SetRestrained 1|PlaceInFront ff000c48 1|ff000c48.setav health 5"
```

Both saves are the developer's own, not in the repository; the bandits' ids are that save's. Any
save with people or creatures to fight will do, and one can be made with
`-At <cell> -Commands "..."`, saved as `rustvr-gore` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

The research and the design: `docs/design/gore.md`.

## What it does

The adapter's `crates/oblivion/src/gore.rs`, the framework's rule (`rustvr_protocol::severs`,
`framework/crates/protocol/src/gore.rs`) and the runtime's blood (`framework/crates/runtime/src/
gore.rs`).

- **Blood from the hit,** as hard as you hit: a slash fans out along the blade's way and out of
  the cut, a stab jets back out of the hole, a blunt blow spatters, fine mist with each. The wound
  keeps bleeding a few seconds and moves with its body part. The game's own hit blood (its spray
  and decals) still happens too.
- **Where it lands:** drops fly with gravity and land on the floor and walls as the game's own blood
  art: splats from its blood decal, long splatters where a fast drop strikes at a slant, stains
  growing into pools where a lot lands, runs creeping down walls and dropping off edges. Decals dry
  darker over a few minutes (pools longer) and never vanish on a timer: the oldest goes once 640
  are down. The game's own blood decals last 600 s in VR instead of its 10.
- **On the blade and hands:** a cut bloodies the edge in a band where it met the body, a stab back
  from the point, never past the blade's ends. Wet blood drips from the blade's lowest wet point.
  Where the blade's acceleration at the blood passes 45 m/s² (a shake or a flick; a hand held still
  is under 15) it flies off with the blade's own velocity, more the harder. Tilted more than about
  17° along its length, wet blood starts runs (about one every 1.25 s, at most 3 at once and 8 on a
  blade): thin streaks growing towards the lower end at up to 5 cm/s, pausing while the blade is
  level. Blood on the blade, the hands and the arms dries over 90 s and wears away 3 to 4 minutes
  after blood last got there. Drops that land on the hands, or on the blade in the air, stay; a
  drip never lands back on the blade it came from.
- **The cut.** Right after the game's own hit has landed, the adapter asks the framework's rule.
  A hit cuts a limb off when it's an edge (a slash, or an axe's chop), clean (not glancing, not a
  stab, not blunt), it kills or the body was already dead, and its power reaches the limb's
  toughness. It cuts at the nearest joint the blade crossed.
- **A head goes with its neck,** cut at the neck's base (`Bip01 Neck1`), anywhere from there up to
  the jaw, as the killing blow; a blow on the skull above the jaw doesn't take it. The head wears
  the NPC's own face, hair and helmet, flies off a little along your swing and drops with physics.
  The body falls with a capped stump in its collar.
- **The stump** pumps at a heartbeat for about 14 s, weakening, each beat a jet whose pressure rises
  and falls so it lands in a line out from the stump, with the game's own blood effect on the
  strong pulses (every 1.6 s for 10 s); then it flows for about half a minute, then drips. Pools
  form under it. The severed piece's cut bleeds too, shorter and weaker. The hand that struck feels
  a heavy thump as the limb goes.
- **Limbs:** an upper arm at the shoulder or the elbow, a forearm at the elbow or the wrist, a thigh
  at the hip or the knee, a calf at the knee or the ankle. A corpse gives way a little more easily.
- **The pieces** are the runtime's props: grip one or pull it with the gloves, and throw it. Each
  stays until its body goes (the cell unloads or a save loads).
- **Each creature bleeds what the game says it does:** the zombie the game's red blood with flesh
  stumps, the skeleton no blood, only its bone-dust puffs, and bone stumps; the lich's is black,
  the will-o'-the-wisp's glows.
- **The flat game's crosshair** is hidden in VR (`crates/oblivion/src/reticle.rs`); its activate
  icons stay.

**The rule's numbers.** Power = the hit's strength x (the weapon's damage / 10) x the weapon's kind.
The damage is the game's own for the weapon; strength 1 is a full-strength slash, at about 7 m/s.
Kinds: a one-handed blade 0.55 to 1.0 by reach (a dagger less), a two-handed blade 1.35, a war axe
1.15, a battle axe 1.45. A corpse needs 0.8 of the toughness.

| Joint | Toughness |
|---|---|
| Wrist | 0.45 |
| Neck | 0.55 |
| Ankle | 0.55 |
| Elbow | 0.65 |
| Knee | 0.85 |
| Shoulder | 0.9 |
| Hip | 1.3 |

So a steel longsword swung moderately takes a head on a killing blow, a hip needs a heavy blade
swung hard, and a dagger doesn't get through a thigh at all.

## What to try

1. **Hit a bandit:** draw the sword and cut one; watch the spray follow the blow and the wound go
   on bleeding.
2. **Where it lands:** splats, splatters, pools, runs down the walls; blood darkening as it dries.
3. **Your sword and hands:** the bloody band; drips; a shake or a flick; tilt it and watch the
   streaks run down and drip from the end.
4. **Take a head:** swing across the neck, edge leading, as the killing blow (the restrained boot
   above makes it the first good blow). Then the skull above the jaw: nothing comes off.
5. **Arms and legs** at each joint; the claymore and the axe take legs, the iron dagger hardly
   anything.
6. **Corpses:** a good blow on a dead bandit's limbs or neck.
7. **The pieces:** grip a head or a limb, pull one with the gloves, throw it.
8. **rustvr-gore-undead:** the zombie and the skeleton.

## Settings

None. For tests, `GoreCut <bone> [standing]` (a console test line,
`crates/oblivion/src/console.rs`) cuts a limb off the nearest creature as a killing blow would
(`standing`: leaves it alive, to look at the stump on a body that stands).

## Decisions

- **The game decides the cut:** the adapter asks the rule right after the game's own hit, which
  gives the damage and whether the blow killed. Damage and power attacks stay the game's.
- **Limbs come off only on a killing blow or a corpse:** Fallout's rule, and the one in the spec.
  Oblivion's AI can't fight on with a missing arm. Severing without killing (mods add it) isn't
  done.
- **A head goes with its neck:** Oblivion's head mesh is skinned down to the clavicles, so a cut at
  the head bone left a cone of neck on the body. Creatures without `Neck1` are cut at the head.
- **How a limb is hidden:** on the body, every node under the cut bone is shrunk to nothing at the
  joint, written before each eye's render (hands and feet are meshes of their own: they go whole).
  The piece is a clone of the creature's whole 3D (the game's own clone, controllers and collision
  taken off the copy), every other bone shrunk away and only the meshes that show the limb drawn. So
  it wears exactly what the NPC wore, with its own face.
- **The clone keeps the body's blood decals:** each decal on the body's decal geometries is copied
  onto the clone's through the game's own calls (and when the piece goes, taken off, let go of and
  freed), since the game's clone copies the decal shader's property without its list and the shader
  draws the first decal without checking.
- **The pieces are the runtime's physics,** not Havok: the same world collision the game's Havok
  has, and grabbable for free. Havok pieces would need Havok bodies made at run time, which the
  adapter doesn't do.
- **The stump caps** are a dome the runtime draws, textured with the game's own neck stump from the
  Kvatch head pike (`clutter\kvatch\neck.dds`; the zombie's bone for dust): Oblivion has no
  gore-cap meshes.
- **The blood art is all the game's:** splats its blood decal atlas, splatters the dungeons'
  `bloodsplatter01`, pools `bloodstain01`, runs the arena's drip streaks, drops the spray's
  `blood01`, mist `blood06`.
- **The torso's combat capsule ends at the chest** (`Spine2`) instead of the neck: its round end
  covered the whole neck, so no blade could reach it.
- **Body part ids are the capsule table's index,** so a hit says which bones it struck.
- **Wolves, bears and rats can't lose limbs:** their combat body is one capsule. Zombies, skeletons
  and other `Bip01`-limbed creatures can.
- **Caps:** 3072 drops, 640 decals, 96 runs, 20 splats on the body and 28 on each held thing, the
  oldest recycled. Blood costs the runtime about 0.1 ms of CPU a frame and at most 160 world ray
  casts; decals, caps and drops are a few instanced draws per kind of blood.

## Not done / known issues

- Severing without a kill, and a hand dropping its weapon (the game's disarm) when cut.
- Bisection, gibs from blunt weapons, and wounds painted onto NPCs' skin (the game's own hit decals
  on their meshes still appear).
- Wiping or washing blood off a blade: it only dries and fades.
- Severed pieces don't persist across loads or cell changes: they go with the body, and the body's
  limbs come back when its 3D reloads.
- Blood doesn't collide with NPCs or props in the air, only the world's static collision and the
  player.
- Blood isn't drawn into the game's HDR or bloom: the runtime draws it as it draws the body.
- **Never checked:** a real swing that cuts a leg or into a corpse (`GoreCut` did the legs); taking
  a piece by its hand grip; blood running down a wall in a capture; the game's own spray at the
  stumps in a capture; lighting outdoors and fog.

## History

Newest first.

- **Playtest 3:** a crash 16 s after a head came off, and the sword dripped only when shaken. The
  crash was in the adapter's own `combat::bones`, which walked into every child near a body's root,
  meshes too (the game's blood decals among them), and read a mesh's fields as a node's children; it
  now goes only into nodes, by each object's type (the check in `engine.rs`). A drip started under
  the blade's line and was caught by the blade at once; a drop no longer lands back on its own
  blade. Asked for: more blood thrown off with a shake, runs down a tilted blade, and blood on the
  blade and the player wearing away (it was kept 20 minutes).
- **Playtest 2:** heads came off without a crash; the sword dripped a little past its point, from
  nothing. The bloody band now stops at the blade's ends, and a drop landing on the blade leaves
  its blood on the blade's line.
- **Playtest 1** (the bandits restrained): the restraint held. A blow on top of the first bandit's
  skull took nothing, by the rule. The second bandit's head came off and the game crashed in its
  renderer (in the decal shader's pass): an earlier hit's blood decal was on its skin, and the clone
  had the decal shader without its list. The clone now copies the decals.
