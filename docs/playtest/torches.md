# Playtest: torches, staffs and spells

Save: rustvr-torches

Boot it by double-clicking `playtest.bat` in this worktree (not committed): it runs `tools\play.ps1
torches` (this worktree's adapter and runtime, straight into the save), waits while you play, then
closes the game and keeps the runtime's log in `<builds>\torches\playtest-<n>`. The save is
Vilverin, at the top of the stairs down into the ruin. The console can't be typed in VR, so the .bat
puts a zombie 2 m in front (down the stairs) and switches the game's AI off (`player.placeatme
00025e6e 1 150 0`, `tai`), so it stands still for the touch spells. The player has 3 torches, one
equipped (at the right hip), the Longsword of Soul Chill equipped (at the left hip), the Staff of
Conflagration in the inventory (not equipped), Flare selected, and 200 more magicka. Added spells:
Absorb Health, Snowball, Spark, Shocking Touch, Cold Touch, Burning Touch and Protect (with the
character's own Heal Minor Wounds and Nordic Frost).

## What to try, in order

1. **The torch at the hip.** Look down: the torch hangs at the right hip, flame up and leaning a
   little forward and out, burning, lighting your body and the walls. It should crackle (the
   game's held-torch loop).
2. **Draw it.** Reach either hand to the right hip and grip: the hand takes the torch by its
   handle, flame up (it's only ever held that way up). Wave it about: the flame is the game's own,
   and its light (the torch's own radius, colour and flicker) moves with it over the walls, your
   arms and the stairs. Hold it up into the dark down the stairs.
3. **Put it away, drop it.** Let go of the grip at the right hip: it hangs there again. Let go
   anywhere else: it falls and lies burning where it lands. Walk away (more than 15 m) and it's back
   at the hip.
4. **Spells, left hand.** Keep the left hand empty (the torch in the right). Rest a finger on the
   trigger: Flare's own casting art (flames) appears small in the palm, with its light. Pull the
   trigger: the art grows over the casting time, the casting sound plays, the pulses quicken (a
   spike when it's ready). Let go once it's ready: the game's fireball goes from the palm along the
   line from your shoulder through the palm, bent towards a creature within 15 degrees. Let go
   before it's ready, or lift the finger off without pulling: the art goes away, nothing cast.
5. **Other spells.** Off hand's B opens Menu Mode; pick a spell in the magic menu.
   - Heal Minor Wounds (self): let go to cast; the game's healing glow goes over your hands and
     arms and fades as the game fades it.
   - Snowball, Spark (target): each with its own art in the palm (frost mist, sparks).
   - Shocking Touch, Cold Touch, Burning Touch, Absorb Health (touch): charge it until it's ready,
     then touch a creature with that hand: it goes off into it (letting go in the air casts it too,
     at nothing).
     The zombie the .bat put down the stairs is standing still for it.
   - Protect (self): its shield shader over the body (not checked, see below).
6. **The enchanted sword.** Draw the Longsword of Soul Chill from the left hip with the right hand:
   it shimmers with its enchantment's own effect shader (a purple rim glow, the game's colour,
   falloff and scrolling texture). Your arms stay their own colour.
7. **The staff.** Open the inventory and equip the Staff of Conflagration: it goes on the back.
   Reach over the right shoulder and grip to draw it. Pull the trigger: the enchantment's fire art
   at the tip, growing over half a second; let go: the game's own staff cast, a fireball from the
   tip along the staff, its charge used up (or the game's "not enough charge" when it's empty).
   The staff should shimmer too.

## What was checked in the game, and what wasn't

Checked once each in the game with the emulated headset (xrsim scripts, captures looked at):

- The torch: at the hip flame up and burning; drawn by the hand; held, waved and raised, its light
  following and lighting the walls, body and hands; dropped and lying burning; put back at the hip;
  the crackle loop started (from the log, not by ear).
- The palm art: Flare's flames readied (small) and charging (growing); the survey of art for
  absorb, healing, frost and shock; the art gone after the cast; the fireball from the palm.
- A touch spell: Shocking Touch charged beside the body, then the hand moved into a zombie: it went
  off on touching it (the game's shock shader on the zombie, its health 1.00 to 0.94).
- The staff: its fire art at the tip, then the fireball from the tip and its explosion.
- The sword's purple shimmer; the arms not tinted by it.
- Heal Minor Wounds' glow over the hands and arms, fading out.

Not checked:

- Anything in a real headset: how it feels, the haptics, the sizes and distances.
- Sounds by ear (the torch's crackle, casting sounds): only that the game was asked to play them.
- The staff's shimmer (only the sword's was looked at), and its charge going down in the
  inventory.
- Protect's shield shader on the body, other spells' shaders on the player (being burned and so on).
- Left-handed mode (the spell hand should become the right hand).
- The torch with a two-handed weapon or a shield drawn, and the game's own rules about that.

## Decisions made (nobody awake to ask)

- **The torch hangs at the right hip** (the sword has the left). It's one constant, `TORCH_SLOT`
  in `crates/oblivion/src/torch.rs`, so holster configuration (section 6) can move it.
- **A holstered torch still burns** and lights the player, like a torch hung on a belt.
- **The flat game's crosshair is gone** (riding's commit, as the menus branch took it): the
  activate icons stay.
- **Upright things** (`OBJF_UPRIGHT`, the torch) hang flame up, leaning forward and out, and are
  only held flame up (their grip isn't reversible).
- **Left behind**, more than 15 m away, the torch is back at the hip (it's still the game's equipped
  torch).
- **Readying a spell** is a finger resting on the trigger (the capacitive touch), or a pull past
  5%; pulling past 60% charges it over half a second (the adapter's casting time); letting go
  below 30% once it's fully charged casts (sooner, nothing).
- **The palm art** is the effect's own `SpecialIdle_Cast` (the game shows it as a short flare): its
  first quarter (at least 0.3 s) is played round and round while the spell is held, at 0.4 of the
  game's size readied, growing to full size as it charges. It floats 6 cm off the palm, clear of
  the fingers. The casting light is the game's own `MagicLight` (radius 500, the effect's colour),
  fading in as the game fades it.
- **Staffs charge over half a second** (the spell hand's casting time) before they cast.
- **Shimmers** (enchanted weapons) are drawn settled: the enchant shader's persistent alpha. Spell
  shaders on the body follow the game's fade in, full and fade out for the effect's age.
- **An enchanted weapon's shader on its wielder** (the game puts one on the player) isn't drawn
  over the body: the weapon's own shimmer draws it.

## Not done

- **The particle part of effect shaders** (`EFSH` particles: a fire shield's flames, for example):
  only the membrane is drawn over the body and held things. The enchant shaders have no particles.
- **The game doesn't know the torch is out**: its `IsTorchOut` stays false, so anything the game
  does with a lit torch (NPCs' reactions, a torch's burn time) doesn't happen.
- **The hands can cover game-drawn art in front of them**: the flame, the palm art and the
  projectiles are in the game's picture, which doesn't keep their depth, so a hand drawn over them
  hides them where they overlap, even when they're nearer.
- **Mage's Staff of Fire** (000E944B) in the inventory made `SaveGame` fail without a word when the
  save was made; it was left out of the save.
- The left hand holding the torch can't cast (the spell hand has to be empty): hold the torch in the
  right hand to cast.

## Branches

`overnight/torches` in chasms-vr-oblivion (this worktree) and in chasms-vr-framework (`framework/`),
both pushed. Research and design: `docs/research/torches.md`.
