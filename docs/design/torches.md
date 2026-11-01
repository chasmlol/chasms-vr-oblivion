# Torches, staffs and spells

Research for torches, staffs and spells in the hand: what the best VR games do with held lights,
staffs and spells, what their players love and hate, how Oblivion itself does each of them, and
the design built from it. The games are for inspiration only: nothing is copied.

## What players love and hate

Gathered from Nexus mod pages, Steam threads and announcements, reviews (UploadVR, Road to VR) and
developer talks. Reddit was read through search summaries only.

### Held lights

* **Skyrim VR**: the vanilla torch is a left-hand item with a flat light, and it takes the spell hand.
  The best-loved mods are about putting it somewhere: VRIK's 14 body holsters (hover until a haptic
  pulse, grip to holster, grip and pull to draw; a lit torch can be holstered), MageVR's shoulder
  slots, Interactive Torches (light a torch from a fire spell, a brazier or a candle; rain and water
  put it out; a dropped lit torch sets off oil traps; either hand). Torches that keep burning in
  flight when thrown are a favourite.
* **Half-Life: Alyx**: the light is on the off-hand glove, not the head and not a held flashlight, so
  it can be aimed and still leave both hands free (Valve: a held light blocks reloading). Players
  miss it while reloading and miss a toggle.
* **Saints & Sinners, Into the Radius, FO4 VR mods**: a light clipped to the chest or a pocket that
  stays on is loved for its hands-free light, hated when it doesn't follow where you look.
* **Lessons**: hands-free light matters (holster it lit); the light must come from the thing held, not
  the head; a torch should be a physical thing (wave it into corners, drop it to mark a room, throw
  it); never let it lock the spell hand.

### Staffs

* **Skyrim VR**: one-handed, cast along the controller with a fixed pitch offset (60 degrees) rather
  than from the tip: shots go "off the top of the wrist". Mods let players zero the angles (aim like
  a pistol) or cast from the palm. Players want the staff to aim where it points.
* **Dungeons of Eternity**: "feel like a wizard, not a staff pointer".
* **Lessons**: cast from the tip along the staff; keep the trigger as the cast (no gestures to fail);
  show the charge at the tip; feedback (haptics, light, sound) on the cast.

### Spells in the palm

* **Skyrim VR**: trigger to charge, release to fire, from the hand. The spell's hand art shows while
  it's equipped (players like seeing their magic). Complaints: aim pitch, dual casting with no
  visible gain, slow spell switching. ISPVR (open the hand to cast) logs the most exact haptics found:
  the charge's pulses shorten from 100 ms to 20 ms as it builds, strength as charge^6 (silent until
  late, then a sharp ramp); a faint tick every 50 ms while held charged; ten full pulses 10 ms apart
  on release.
* **The Wizards, Waltz of the Wizard, Hellsweeper**: gesture spells feel magical but misreads in combat
  are the top complaint ("shield sometimes cast fire").
* **Aim help**: Skyrim's Spell Auto-Aim VR uses ±10 degrees vertical, ±15 horizontal for bolts and
  is praised for being "not cheaty". Gentle cones are welcome; hard locks aren't.
* **Too bright**: curated Skyrim VR lists dim enchantment glow to 60% and drop hand-effect lights,
  because up close they overexpose.

### Enchantments

* **Skyrim VR**: the stock enchantment shimmer reads too strong up close in VR.

### What it adds up to

Loved: lights that are physical and hands-free when you want; the spell visible in the hand with a
clear build-up and a "ready" spike; casting from where the hand or staff points; gentle aim help;
the world's own effects. Hated: a light stuck to the head, a torch that blocks the spell hand,
staffs that don't cast from the tip, gesture misreads, accidental casts, effects that blind.

## Oblivion's own torches, spells and staffs

As the game plays them and as `Oblivion.esm` and its models have them.

### The player's torch

* The game draws an equipped torch on the body: its model on the skeleton's `Torch` node (in the
  left hand; the model's `Prn` extra data says `Torch`), with its light.
* A light's record has its radius, colour, flags (dynamic, can carry, negative, flicker, off by
  default, flicker slow, pulse) and fade. The game makes its light as a point light under the
  model's `AttachLight` node (or its root), flickering every frame where the record says so.
* A torch's flame isn't in its model: the model has `FlameNode1` (and a `FlameCap`), and the game
  puts its flame there.
* `Torch02` (`Lights\Torch02.NIF`): radius 768, colour (184, 123, 69), flicker, carried, 1000 s of
  burning. Its model is 0.68 m long along +Y, the handle at the origin, `AttachLight` at 35 units
  (0.5 m) up, the flame at 27.
* The held torch crackles with its loop sound (`ITMTorchHeldLP`).
* Checked in the game: with the torch equipped through the console, neither the first-person nor
  the third-person `Torch` node had a model under it after nine seconds, and no light was near the
  player: in VR the game never shows its own equipped torch.

### Spells and their art

* A magic effect (MGEF) has a model, a light (a LIGH) and a casting sound. Every effect's model in
  `meshes\magiceffects\` (`fireball.nif`, `frostcloud.nif`, `shockbolt.nif`, `restoration.nif`...)
  holds three controller sequences: `SpecialIdle_Cast` (the art in the caster's hands while casting:
  `PCloud Casting Flame` for fire), `SpecialIdle_Projectile` (the bolt in flight) and
  `SpecialIdle_AreaEffect` (the burst).
* A spell's art is its main effect's (the costliest): while the cast animation plays, that effect's
  model plays its `SpecialIdle_Cast` at the caster's `magicNode` (in front of the chest; in first
  person the first-person skeleton's), with a casting light of the effect's LIGH colour fading in.
  The effect's casting sound plays with the cast animation.
* The cast sends the projectile with the effect's model playing `SpecialIdle_Projectile`, its bolt
  sound, and on hit the effect's hit shader and area art.

### Staffs

* A staff is a weapon whose enchantment casts as it attacks. With charge enough for the
  enchantment, the charge is used up and the enchantment cast (the projectile, its sounds); with too
  little the player is told ("The enchanted item does not have enough charge", once).

### Enchantment shimmer

* Every MGEF names an enchant effect shader (an `EFSH`): the membrane the game draws over an
  enchanted weapon (a fill texture scrolled over the mesh, its colour and alpha pulsing, a brighter
  edge where the surface turns away), and the same for the hit shader on a creature struck. `EFSH`
  keeps its fill texture (`ICON`), particle texture (`ICO2`) and its numbers (`DATA`: blend modes,
  fill colour, alpha fade and pulse, the fill texture's U and V speeds, the edge falloff and
  colour).
* An enchant effect settles at its edge's persistent ratio (0.3 to 0.4, falloff 2 to 3, the fill's
  persistent 0): a rim glow in the school's colour.
* An equipped enchanted weapon's shimmer is on what the actor holds, not its body.

### The casting art, seen in the game

`SpecialIdle_Cast` is a flare at its start (fire's burst of flame, absorb's red column, healing's
sparkles) that has died away by about a third of the sequence; the game shows it once, as its cast
animation lets go. Fire's is 1.67 s long, frost's 0.67 s, restoration's 2.83 s. Every spell
effect's casting light is radius 500 (its colour the effect's LIGH's).

## The design

### Framework (any game)

1. **Casting, for the game to draw** (`rustvr_protocol::Casters`, in the protocol's `magic.rs`;
   runtime to game every frame, from the runtime's `combat.rs`): each hand's casting state
   (readied, charging, ready), what it casts (a spell, or the object it holds), how charged, and
   where the art goes (the palm, or a held object's tip) in tracking space with the aim. A game
   that draws its own casting art says so (`SPELLS_GAME_ART` in its spell table) and the runtime
   draws none of its own.
2. **Readied**: the spell hand's art comes up when its finger rests on the trigger (or the trigger
   is pulled a little), charges as it's pulled, is ready with a haptic spike, and goes off on
   release. The charge's haptics follow ISPVR's curve: faint and slow early, a sharp ramp, a tick
   while held ready.
3. **Things that cast** (`OBJF_CASTS`): a held object that casts its own magic (a staff): its
   trigger charges it over its own cast time, and release casts from its tip (`muzzle`) along its
   length, with the spell hand's aim assist. The cast event carries the object.
4. **Effect overlays** (`rustvr_protocol::Overlays`, game to runtime when they change): the game's
   effect shader on something the runtime draws (an enchanted weapon's shimmer): its fill texture,
   colours, alpha and pulse, the texture's scroll, the edge and its falloff, drawn as an additive
   pass over the model.

### Oblivion

1. **The torch** (`torch.rs`): the equipped torch is an object on the body as the equipped weapon
   is, and either hand draws it, waves it about, holsters it (in any holster it fits) and drops it.
   One equipped from the game's own menu starts at the right hip (`TORCH_SLOT`), as a sword starts
   at the left; one drawn from a body slot is equipped by the game and takes the copy's place in
   the hand. The game draws it: its own model through its model cache, its own flame
   (`FlameNode`), its own light (radius, colour, attenuation) with its own flicker every
   frame, put where the runtime has the torch, and it crackles with the game's held-torch loop.
   Holstered, it still burns (the hands-free light players love); let go anywhere else, it falls
   and lies burning where it lands, as a sword does. Unequipped in the inventory, it's gone. The
   runtime's body and hands are lit by its light as by any of the game's.
2. **The spell in the palm** (`magic.rs`): the selected spell's main effect's own model, its
   `SpecialIdle_Cast` art, in the palm, and its own casting light fading in as the game's does; the
   art small while readied, growing as it charges, the effect's casting sound as the charge starts.
   The cast stays the game's immediate cast from the palm (its projectile, bolt, sounds, impacts).
3. **Staffs**: held and aimed by the hand, the enchantment's art at the tip while charging, and
   released as the game's own staff cast (the charge used up, its projectile from the tip along
   the staff, its "not enough charge").
4. **The shimmer**: an enchanted weapon or staff in the hand carries its enchantment's effect
   shader as an overlay.

### As built

* The palm art is the effect's own `SpecialIdle_Cast`, its first quarter (at least 0.3 s) played
  round and round by holding the sequence's offset, so the flare burns on while the spell is held;
  0.4 of the game's size readied, growing to its size as it charges. Its casting light fades in as
  the game's does; the casting sound plays as the charge starts.
* The torch is the torch's own model with the game's flame and the torch form's own light (radius
  768 for `Torch02`, flickering), hung in the world's lit scene (not where the game hangs its
  projectiles, where particles are never made), run on the world time (`scene.rs`). It hangs in a slot flame up
  and is held only flame up (`OBJF_UPRIGHT`).
* Not built: a torch coming back to the hip when it's left behind; a torch taken off a wall
  bracket.
* Staffs charge over the spell hand's casting time (the adapter's 0.5 s, its spell table's
  `cast_time`) at the tip, with the enchantment's art there, and cast as the game's staff cast
  (`combat.rs`).
* Overlays draw the game's membranes over the held weapon (settled) and over the body (through the
  shader's curve for the effect's age); the particle part of an `EFSH` isn't drawn (the enchant
  effects have none: their flag 0x08).
