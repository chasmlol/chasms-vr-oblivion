# Playtest: torches, staffs and spells

Save: rustvr-torches

The research and the design: `docs/research/torches.md`. How it's wired into the game: the Torch
row of `docs/OBLIVION.md`.

## Playing it

From the repository's root:

    powershell -File tools\oblivion.ps1 -Save rustvr-torches -Commands "player.placeatme 00025E6E 1 150 0|tai"

It builds main, installs it beside the game and boots straight into the save for the headset;
`powershell -File tools\oblivion.ps1 -Close` puts the game away and your settings back. The two
console lines put a zombie 2 m in front of you, down the stairs, and switch the game's AI off
(`tai`), so it stands still for the touch spells (the console can't be typed in VR).

`rustvr-torches` is the developer's own test save, not in the repository (`docs/DEVELOPING.md` has
the test saves): Vilverin, at the top of the stairs down into the ruin. The player has 3 torches,
one equipped, the Longsword of Soul Chill equipped, the Staff of Conflagration in the pack (not
equipped), Flare selected and 200 more magicka, and these spells besides the character's own Heal
Minor Wounds and Nordic Frost: Absorb Health, Snowball, Spark, Shocking Touch, Cold Touch, Burning
Touch and Protect.

To make one from any save (here `rustvr-ruin`, Vilverin), with `Torch02` (`0002CF9F`) and the Staff
of Conflagration (`000914C5`):

    powershell -File tools\oblivion.ps1 -Save rustvr-ruin -Give "0002CF9F 3,000914C5 1" -Commands "player.equipitem 0002CF9F|player.modav magicka 200|player.addspell <spell id>|SaveGame rustvr-torches"

with a `player.addspell` line for each spell, and an enchanted weapon of your own equipped. Leave
the Mage's Staff of Fire (`000E944B`) out: with it in the pack, `SaveGame` failed without a word.

## What it does

**The torch is a thing like a weapon.** The equipped torch is an object on the body as the
equipped weapon is (`crates/oblivion/src/torch.rs`):

- Equipped from the game's own menu (or the console), it starts at the right hip, as an equipped
  sword starts at the left. From there either hand draws it, waves it about, holsters it in any
  holster it fits (the hips and the back; the chest and belt pouches take only small things) and
  drops it.
- Torches are a category of the body's slots (the holsters' editor: "Torches", the inventory's
  carried lights). A torch drawn from a slot is equipped by the game, and the burning torch takes
  its copy's place in the hand, as a weapon drawn from a slot does. The equipped torch isn't one of
  the slots' things.
- Let go in a holster (gently), it hangs there; at a hip it hangs flame up, leaning forward and out.
  Let go anywhere else, it falls and lies burning where it lands, as a sword let go lies. It
  doesn't come back on its own; the gravity gloves pull it like anything else.
- It's held only flame up (its grip isn't reversible: `OBJF_UPRIGHT`), by its handle, from its foot
  to a hand's width under the flame.
- It burns wherever it is, holstered too: the game's own model, its flame (`FlameNode`) and its
  light, made from the torch's form (its radius, colour and attenuation) among the world's lights
  and flickered every frame as the game flickers a torch. It lights the world, the body and the
  hands.
- It crackles with the game's held-torch loop (`ITMTorchHeldLP`), started again once a second if
  the game lets it go. Grabbed and holstered, it sounds as the inventory sounds a torch.
- Unequipped in the inventory, it's gone.

**Spells, in the spell hand.** The spell hand is the off hand (the left, unless the weapon hand is
set to the left), empty, while the game has a spell selected (`framework/crates/runtime/src/combat.rs`,
the adapter's `combat.rs` and `magic.rs`):

- A finger resting on the trigger, or a pull past 5%, readies the spell: the game's own casting art
  for its main effect (the effect model's `SpecialIdle_Cast`: a fire spell's flames, frost mist,
  sparks, healing sparkles) floats 6 cm off the palm at 0.4 of the game's size, its first quarter
  (at least 0.3 s) played round and round, with the effect's own casting light (`MagicLight`)
  fading in as the game fades it.
- Pulled past 60%, it charges over the casting time (0.5 s): the art grows to the game's full size,
  the effect's casting sound plays, and the hand feels pulses, faint and slow at first, then a sharp
  ramp, a spike when it's ready and a soft tick every 0.3 s while it's held ready.
- Let go below 30% once it's ready, it casts; sooner, nothing. The game's own cast takes over (its
  projectile and its art, its sounds, its impact, its magicka).
- A target spell goes along the line from the shoulder (18 cm to the hand's side of the head) through
  the palm, bent onto the creature nearest that line within 15 degrees and 25 m. A hand moving
  faster than 1 m/s as the trigger goes throws it, the line bent towards the throw.
- A spell on yourself is cast wherever it's let go.
- A touch spell, charged, goes off into the first creature the hand touches (within 8 cm of its
  body), or wherever it's let go.
- Spells' shaders on the player (Heal Minor Wounds' glow, a shield) are drawn over the body and
  hands, through the game's own curve: fading in, full, fading out to their persistent strength.

**Staffs.** An enchanted staff (two-handed, it starts over the right shoulder) casts in whichever
hand holds it: the trigger charges its enchantment at the tip over the same 0.5 s, its art growing
there; let go, the game's own staff cast fires from the tip along the staff, its charge used up, or
the game's "not enough charge".

**Shimmer.** An enchanted weapon wears its enchantment's own effect shader, settled: its membrane's
colour, falloff and scrolling texture. The same shader the game puts on its wielder isn't drawn over
the body.

## What to try

1. **The torch at the hip.** Look down: it hangs at the right hip, flame up, leaning a little
   forward and out, burning and lighting your body and the walls. Listen for its crackle.
2. **Draw it.** Reach either hand to the right hip and grip: the hand takes it by the handle,
   flame up. Wave it about and hold it into the dark down the stairs: its light moves with it over
   the walls, your arms and the stairs.
3. **Holster it anywhere.** Let go of it at the other hip, or behind a shoulder: it stays there,
   burning. Draw it from there.
4. **Drop it.** Let go of it away from your holsters: it falls and lies burning where it lands.
   Pull it back with the gravity gloves (point, grip, flick), or pick it up.
5. **A torch from a slot.** Open the holsters' editor (grip held in a slot for 1.5 s, or Holsters in
   RustVR's menu), give a slot the Torches category, and draw from it: the game equips that torch
   and it burns in your hand.
6. **Flare, in the spell hand.** Keep the left hand empty (the torch in the right). Rest a finger on
   the trigger: Flare's flames, small, in the palm. Pull: they grow, the casting sound plays, the
   pulses quicken to a spike. Let go once it's ready: the game's fireball from the palm along the
   line from your shoulder through it. Let go too soon, or lift the finger off without pulling: the
   art goes, nothing is cast.
7. **Other spells.** The off hand's B is the game's Menu Mode: pick a spell in its magic menu.
   - Heal Minor Wounds: let go anywhere; the game's healing glow goes over your hands and arms and
     fades as the game fades it.
   - Snowball, Spark: their own art in the palm (frost mist, sparks).
   - Shocking Touch, Cold Touch, Burning Touch, Absorb Health: charge it, then touch the zombie
     with that hand: it goes off into it.
   - Protect: its shield over the body.
8. **The enchanted sword.** Draw the Longsword of Soul Chill from the left hip: it shimmers with its
   enchantment's own shader. Your arms stay their own colour.
9. **The staff.** Equip the Staff of Conflagration in the inventory: it goes over the right
   shoulder. Reach over and grip to draw it. Pull the trigger: the fire art at the tip, growing over
   half a second; let go: a fireball from the tip along the staff, its charge used up (or the
   game's "not enough charge" when it's empty).

## Settings

| Where | Setting | Default | What it does here |
|---|---|---|---|
| Controls tab | Weapon hand: Left / Right | Right | The spell hand is the other one |
| Hands tab | Holsters' editor: grip held in a slot (0: off) | 1.5 s (0 to 3) | How long a grip in a slot opens the editor that sets a slot's categories (Torches among them); RustVR's Holsters button opens it too |
| Hands tab | Gravity gloves (point, grip, flick to pull) | on | Pulls a dropped torch back; Glove range 10 m (1 to 10) |
| Hands tab | Grab reach | 0.07 m (0.03 to 0.15) | How near a hand takes the torch |
| Hands tab | Throw strength | 1 (0.6 to 2) | How far a thrown torch goes |

The casting thresholds (5%, 60%, 30%), the casting time (0.5 s) and the aim assist (15 degrees,
25 m) are constants in the runtime's `combat.rs` and the adapter's `combat.rs`, not settings.

## Decisions

- **The torch is a thing like a weapon**: no slot of its own, any holster it fits, from any slot
  given the Torches category. The fixed right hip (and its coming back there when left 15 m behind)
  was a test setup; it now only starts there when the game's menu equips it, across from the sword.
- **Holstered, it still burns** and lights the player, like a torch hung on a belt.
- **Upright things** (`OBJF_UPRIGHT`) are only held their one way up, and hang at a hip flame up,
  leaning forward and out, clear of the body and the face.
- **The game draws it**: the game's own equipped torch belongs to the first-person viewmodel and
  never shows in VR, so the hand's torch is a copy of its model from the game's model cache, given
  the game's flame and a light made as the game makes a torch's. So it lights the world and the
  body as the game's torches do.
- **The flat game's crosshair is gone** (`crates/oblivion/src/reticle.rs`): the activate icons
  stay, as they're the game's only warning of a theft.
- **Readying is a finger on the trigger** (its capacitive touch) or a little pull, so the art shows
  before the charge; letting go too soon casts nothing, as the game's own cast needs its animation.
- **The palm art is the game's own `SpecialIdle_Cast`**, which the game plays once as a short flare:
  its first quarter held round and round keeps it burning in the palm. It floats 6 cm off the palm,
  clear of the fingers.
- **Staffs charge over the spell hand's casting time** before they cast, so a staff and a spell
  feel the same.
- **Shimmers are drawn settled** (the shader's persistent strength); spells' shaders on the body
  follow the game's fade in, full and fade out for the effect's age.
- **A weapon's enchant shader on its wielder isn't drawn over the body**: the weapon's own shimmer
  draws it.

## Not done / known issues

- **The particle part of effect shaders** (`EFSH` particles, a fire shield's flames) isn't drawn:
  only the membrane, over the body and held things. The enchant shaders have no particles.
- **The game doesn't know the torch is out**: nothing sets its `IsTorchOut`, so what the game does
  with a lit torch (NPCs' reactions, a torch's burn time) doesn't happen.
- **The hands can cover game-drawn art in front of them**: the flame, the palm art and projectiles
  are blended into the game's picture without depth, so a hand drawn over them hides them where
  they overlap, even when they're nearer.
- **A hand holding the torch can't cast**: the spell hand has to be empty. Hold the torch in the
  weapon hand to cast.
- Not checked in the game: the staff's shimmer and its charge going down in the inventory;
  Protect's shield and other spells' shaders on the player; the left-handed spell hand; the torch
  with a two-handed weapon or a shield drawn (the game's own rules about that); drawing a spare
  torch of the same kind as the one equipped from a Torches slot; the sounds by ear.

## History

- **30 October, merging books and holsters**: the books feature keeps the map at the right hip,
  which clashed with the torch's fixed slot. The torch became a thing like a weapon: a Torches
  category of the slots, any holster, lying where it falls instead of coming back to the hip, the
  inventory's torch sounds.
- **The overnight build** (`overnight/torches`), checked once each on the emulated headset: the
  torch at the hip burning, drawn, waved, dropped and holstered, its light on the walls, body and
  hands, its crackle asked for; Flare's art readied and charging and the fireball from the palm;
  Shocking Touch going off into a zombie on touching it (its shock shader, its health 1.00 to
  0.94); the staff's art at its tip and its fireball; the sword's purple shimmer, the arms not
  tinted; Heal Minor Wounds' glow fading out. Not yet felt or heard in a real headset.
