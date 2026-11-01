# Riding in VR: what players want, how it's done, and Oblivion's horses

Research for riding Oblivion's horses in VR, gathered 2026-10-30, and the design built from it.
Players' opinions are paraphrased with their source; the sources are linked so the exact words can
be read there. Nothing here is copied from another game or mod: it's the reasoning the design rests
on.

## 1. What players love and hate

**Love**

- Holding the controllers like reins and steering with them: simple and natural (Skyrim VR players,
  [Steam](https://steamcommunity.com/app/611670/discussions/0/1696043806553872544)).
- Turning the head and the reins together, and looking over a shoulder at the others while riding
  on (Rival Stars Horse Racing VR, [Steam reviews](https://steamcommunity.com/app/2937440/reviews/?browsefilter=toprated&filterLanguage=english)).
- A mount picking up its pace, wind in the face (Asgard's Wrath 2, [The VR Critic](https://www.thevrcritic.com/reviews/asgards-wrath-2/)).
- Physical reins as an idea: Skyrim VR's "Horse Reins" mod (grab the real reins, move a hand to
  steer, whip to change pace) was welcomed the moment it appeared ([Nexus 196305](https://www.nexusmods.com/skyrimspecialedition/mods/196305)).

**Hate**

- Having to choose between looking around and steering. In Skyrim VR before patch 1.4, steering by
  the head broke the horse and steering by the hand broke the bow: "Gotta pick your poison - working
  horse or head look." (JoshG, [Steam](https://steamcommunity.com/app/611670/discussions/0/1696043806553872544)).
- A view that doesn't turn with the horse: turn the horse round and you still face the old way
  unless you turn yourself ([Steam](https://steamcommunity.com/app/611670/discussions/0/1696043806574805761/)).
- Gestures that don't fire, or fire when they shouldn't; arm fatigue; jittery, over-sensitive hands
  (Rival Stars: [UploadVR review](https://www.uploadvr.com/rival-stars-horse-racing-vr-edition-review/), Steam reviews).
- Rein input that doesn't match real riding: an equestrian's instinct to open a rein out to the side
  wasn't understood ([Mane Quest](https://themanequest.com/blog/2024/8/28/preview-i-tried-the-vr-edition-of-rival-stars-horse-racing-and-its-looking-pretty-good)).
- Getting off by accident: in Skyrim VR, Activate both loots and dismounts (Steeds of Ultima's notes,
  [Nexus 81220](https://www.nexusmods.com/skyrimspecialedition/mods/81220)).
- The camera taken over by a mount animation (paired animations teleport the VR camera:
  [Convenient Horses VR](https://www.nexusmods.com/skyrimspecialedition/mods/165369)), and a
  view locked to an animated body (VRIK Actions warns it sickens: [Nexus 69282](https://www.nexusmods.com/skyrimspecialedition/mods/69282)).
- Comfort options that are missing, or can't be turned off (Rival Stars' vignette).

## 2. How the best do it

| | Steering | Pace | Getting on |
|---|---|---|---|
| Skyrim VR (vanilla, 1.4+) | the hand points the way (or the head) | the stick | Activate and the game's animation |
| Steeds of Ultima VR | turning tied to the headset (frees both hands to fight) | the stick | faster mount and dismount; a separate dismount |
| DovaVR Locomotion | the controller points | two quick down-up whips speed up, each further whip a step; both arms down and back slows, held 1.5 s stops; touching the stick stops at once | the game's |
| Horse Reins (Skyrim) | grab the real reins, move a hand left or right | whip to toggle walk and run, drop them to stop | the game's |
| Rival Stars Horse Racing VR | pull one rein back (from a calibrated neutral) | rhythmic up-and-down of both hands builds energy | placed in the saddle |
| Dragon Wrangler VR | pull the reins left, right, up, down | up then quickly down speeds up, pull slows | - |
| Witcher 3 "Alternate Horse Controls" (flat) | the stick is the reins | four gaits, each tap a step up, kept with nothing held; pulling back brakes by how far | - |
| Mount & Blade II (flat) | the rider steers by weight while the hands fight | - | - |

Nobody ships getting on by grabbing the saddle (one itch.io comment suggests it): it's new ground,
built here and tested rather than copied.

## 3. Comfort, from the research and the developers

- **Acceleration and rotation sicken, steady speed doesn't.** Meta: the inner ear feels changes in
  motion; keep accelerations short and stops soft ([Meta](https://developers.meta.com/horizon/design/locomotion-comfort-usability/)).
  VirZOOM, who made cycling and riding games, ranked rotation the worst of all motions and turned the
  world only when the rider's head turned too ([VirZOOM](https://virzoom.com/vr-that-moves-you/)).
  Snapping the rotation cut sickness about 40% in one study (Farmani & Teather, [Carleton](https://www.csit.carleton.ca/~rteather/pdfs/theses/thesis-yasinfarmani-final.pdf)).
- **A vignette narrowing the view in motion** reduced sickness without most people noticing it
  (Fernandes & Feiner, IEEE 3DUI 2016); Meta's SDK closes one on turning and on acceleration
  ([Meta ISDK](https://developers.meta.com/vr/documentation/unity/unity-isdk-locomotion-comfort/)).
  Players want to be able to turn it off.
- **Something fixed in view:** the horse's head and neck (Rival Stars keeps the head in view, Eagle
  Flight shows the eagle's beak, Purdue's "virtual nose" cut sickness ~13%).
- **No oscillation:** visual bob makes sickness worse (a 2025 meta-analysis in *Multisensory
  Research*); the horizon stays level, the camera height steady.
- **The head free:** steering by the hands, looking by the head. Oblivion itself works so: mouse look
  doesn't steer its horse ([UESP](https://en.uesp.net/wiki/Oblivion:Horses)).

## 4. Oblivion's horses

As the game plays them, measured on the headset.

**Getting on and off.**

- Activate on a horse is the game's whole mount: its owner check, "Do you want to steal this horse?"
  and the crime (`iCrimeGoldStealHorse`), the mount animation (`horsemount.kf`, from the
  `EntryPoint` node on the horse's left) and the sounds. Mounted, Activate on the horse ridden is
  the dismount.
- A script can't have the player activate something, so the adapter activates the horse as the
  player's Activate control does.

**A mounted player's input.**

- The joystick's forward and back set the horse's pace; its sides turn it. Only while the stick
  points more ahead or back than to the side does the horse go forward or back; past that it turns
  on the spot with no forward motion: a hard turn stops it. Either axis pushed nearly all the way
  presses Run too.
- The horse's turn ramps up at `iHorseTurnDegreesRampUpPerSecond` (80) to
  `iHorseTurnDegreesPerSecond` (45). The turn keys turn it 1.5 times that: 67.5°/s.
- Run gallops; Always Run inverts it. Jump with forward makes the horse jump.
- The mouse turns only the camera (held to ±90°): looking doesn't steer.
- A horse walks at its walk speed times the pace (1.3 m/s at full pace, the chestnut), but gallops
  at 2.1 times its run speed (8.6 m/s where its run speed is 4.1), times the pace.
- The first-person camera on a horse: on the first mount of a session it's a seated rider's eye,
  0.85 m over the saddle's seat (measured on both rides); after getting off and on again it stays at
  a standing player's eye on the horse's back (1.9 m over the seat) and doesn't keep up with the
  horse as it moves.
- The horse's speeds are its own (its movement speed from its Speed, walking or running, times the
  pace). Its gaits: walk (`forward.kf`) and gallop (`fastforward.kf`); there's no trot.

**The horse's model.**

- Skeleton `creatures\horse\skeleton.nif`: `Bip02 ...` bones; `ActorParent` under `Bip02 Spine2` is
  where the rider hangs; `EntryPoint` (-69, 0, 0) is the mounting side (the left).
- `saddle.nif` is skinned to Spine1-3 and the clavicles, made in the skeleton's bind pose: a horn
  (pommel) at the front, a seat, a cantle, skirts down the sides and the girth under the belly. It
  has **no stirrups**.
- `bridle.nif` (NIF 10.1.0.106, which the extractor doesn't read) is rigid on `Bip02 Head` with a
  morph for the bit (`BridleBitOpen`): the reins go with the head.
- The horse's animations carry **the rider's `Bip01` tracks**: the game animates its rider with the
  horse. But only in third person: there's no first-person riding animation. So in first person
  nothing in the game poses the rider; the seat and legs are measured on the models instead (below).
- Measured by `oblivion-extract export meshes/creatures/horse` (`oblivion_assets::horse`, into
  `local_assets/oblivion/horse.toml`), in `Bip02 Spine2`'s frame: the pommel (the highest point on
  the saddle's middle), the seat (the lowest of its top between the pommel and the cantle), the
  cantle, and where a rider's legs go round the horse's body (`horse.nif`): how wide the horse is
  beside the seat at a rider's knee height (0.43 m below the seat: 0.37 m each side) and below its
  belly at the ankle.

## 5. The design built

**The framework (any game):**

- **Protocol** (`rustvr_protocol::ride`): `Ride` (game -> runtime, every game frame, tracking space):
  the mounts near the player with their saddle (a segment and a reach), and the mount ridden: getting
  on, seated, getting off; its gait and how many it has, its speed; where the rider sits (moving with
  the gait as the game animates the mount), where the knees and feet go, where the reins are, the
  pommel. `RtRide` (runtime -> game): the gait asked for, how much of its pace, the steering, getting
  on (which mount) and getting off. `TrackingMapper::carry_turn` turns the view with what carries the
  player, smoothly or in steps (`TrackingControl::carry_step`).
- **The rider** (`rustvr_runtime::riding`):
  - Getting on: a free hand closing on a saddle.
  - The reins are taken by the grip where the game has them (within 15 cm); where a hand took its
    rein up is its rest. Measured in the rider's frame (tracking space turned to the mount when the
    reins were taken), so a view turning in steps never moves them.
  - Hands at rest do nothing. A hand within 1.5 cm of its rest carries the rest along slowly (over
    1.5 s), so a resting hand's drift never adds up; braking starts only 5 cm back (`rein_dead`).
    (The first ride on the headset, with a 2.5 cm dead zone and no following, turned and backed the
    horse without meaning to.)
  - Steering: one rein back further than the other, or both moved to one side (a neck rein, or an
    opening rein: both are what riders do). It's measured against the rider and the mount, not
    from where the reins were taken: the hands' middle across from the rider's middle past 3 cm
    (the rider's middle follows where the head sits over 10 s, so a glance round doesn't move it),
    or one hand ahead of the other past 2.5 cm; as the square of the way past that, the hardest turn
    at 30 cm (`rein_travel`), smoothed over 0.12 s. One hand alone holds both reins (western
    style): sideways steers, back slows.
  - Speed is fluid: a share of the mount's full gallop. A spur surges it by as much as it's done
    (the lightest a slow walk from a stand, the hardest a full gallop), fading slowly (over 8 s,
    `spur_fade`) to a cruise (0.7 of the surge, `cruise`); both reins pulled back slow it as hard as
    they're pulled (hardest at 15 cm, `rein_pull`), and where it's slowed to it stays. Backing is
    asked for on its own, as a rider asks for a rein-back from a halt: at a stand, the reins eased
    and pulled right back again, held 1.2 s. Let go of the pull, backing stops.
  - Spurring on: a flick (a rein hand snapped down faster than 1.1 m/s and stopped within 0.3 s, 4 cm
    or more; as hard as it's fast, the hardest at 3.5 m/s), or the trigger of a rein hand (a squeeze
    of the heels: no legs are tracked; as hard as it's squeezed). Always heard, even with the reins
    held back (which are then given).
  - The log says what the hands did (each one's move from its rest, the pull) and what the mount
    was told, twice a second while a rein is held.
  - Reins let go: the speed holds 2.5 s, then eases off to a stand.
  - The stick rides too: ahead or to a side goes at its push and steers, with sprint gallops; pulled
    back (within 60° of straight back) it backs; let go, it stops (as the game's controls). The
    turning stick alone only steers.
  - Taking up the reins: a rein follows its hand until the hand is still (2.5 s at most): drawing
    the reins in from the neck isn't a pull. A trigger with the grip as a rein is taken isn't a
    spur.
  - Getting off: a free hand's trigger held 0.6 s, or a hand on the pommel while the head leans
    35 cm out over a side for 0.3 s (swinging a leg off). Never the plain trigger: that's the
    accidental dismount players hate.
  - Haptics: a tick taking a rein, a pulse spurring on, a softer one slowing, a beat with each stride
    in the hands on the reins.
  - Settings (the runtime's `[riding]`): `turn` (smooth, or snap in steps of `turn_step`, 30°),
    `vignette` (0.5), `fade_on_off`, `rein_dead`, `rein_travel`, `rein_pull`, `cruise`,
    `spur_fade`, `flick`, `coast`, `stride_haptics`.
- **The body in the saddle** (`BodySolver::ride`): the hip joints a sitting body's height over the
  seat, the spine bowing from them to the head wherever it is (or lifting the hips, standing in the
  stirrups), the chest turning with the head up to 50°, the legs to where the game puts the knees and
  feet.
- **Comfort:** while riding the body isn't walked after the head (the mount moves it), the turning
  stick steers instead of turning, and ducking isn't sneaking. The view turns with the mount (the
  game's own turn, ramped by the game) smoothly or in steps; a vignette while it turns or gallops
  (its own strength); a fade while the game's animation gets the rider on or off; the view's anchor
  is the rider's eye held steady on the mount (the gait's bob comes back with the head bob setting).

- **The reins drawn** for a game whose mount has none (`RIDE_BITS`, `rustvr_runtime::reins`): from
  the bit's rings to the hands, hanging by their slack, as straps of the game's own leather
  (`RIDE_LEATHER`: a model's material and a stretch of its texture) drawn with the body (a held
  model that bends, each cross-section a bone), so the game's depth hides them and its lights light
  them; laid on the mount's neck and head where they'd pass through (`RIDE_NECK`: cross-sections,
  an ellipse round the top of each). Without the leather, flat ribbons of its colour.

**Oblivion:** the adapter's `riding.rs` gives the framework the horses near the player (within 6 m)
and the one ridden (as above), gets on and off with the game's own activation, and gives the rider's
speed to the game as its own joystick and Run: the game's walk at a pace up to its own, or its
gallop at a pace, never under 0.6 of the full gallop (slower, a gallop's stride looks like a
slowed-down sprint), with a little hysteresis between the two. The steering doesn't go through the
stick (a hard turn there stops the horse, and it ramps up slowly): each frame the horse's heading is
turned by the game's own turn at the rider's share of the game's full turn rate. Jump is
the game's own horse jump. No fighting from the saddle: the game's rule, kept (blows and casts land
nothing while mounted). The bit is measured on the horse's head model (`horsehead.nif`: the corners
of its mouth, 6 units behind the muzzle's tip, 70% down it, the rings 1.5 units out), the reins'
colour on its tack texture (its smallest mip). The reins' leather is the saddle's material
(`Tack.dds`, its normal map and shine; the bridle wears the same texture, in an older NIF the
extractor can't read), cut from the bridle's plain cheek strap in it; the neck is its five neck
bones' cross-sections of the body model, the head four along the head model. The game's crosshair
(the dot of its reticle, `hudreticle_reticle_crosshair`) is hidden in VR through the game's own tile
update; the reticle's activate icon stays (red: stealing).
