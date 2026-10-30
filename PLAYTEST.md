# Playtest: archery

Save: rustvr-archery

`powershell -File tools\play.ps1 archery` boots Oblivion into the save `rustvr-archery`. The save
is on Weye's open ground by Lake Rumare at 1 pm. An Iron Bow is equipped with 168 Iron Arrows, and
20 Arrows of Flames are in the inventory. When you're done, `powershell -File tools\play.ps1
-Close` puts the game away.

The bows and arrows are already extracted into the shared `local_assets`. On another machine, run:
`cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/weapons
textures/menus/misc/hud_reticle.dds`. That writes the bows, their bent models, the arrows, the
quivers, `bows.toml` and the crosshair image.

The research and the design are in `docs/research/archery.md`.

## What to try, in order (right-handed; everything mirrors for the left hand)

1. **Take the bow.** Reach over your left shoulder, behind your head, and grip. The bow comes into
   the hand by its grip, upright, thumb up. Its arrow rests on the back-of-hand side of the riser.
   Let go of it there again and it goes back on your back.
2. **Take an arrow.**
   - Over your right shoulder: reach behind your head and grip. Oblivion wears its quiver there,
     with the arrows at the right shoulder.
   - Or at your right hip: grip there.
   - Either way, you feel a tick as the hand enters the quiver and an arrow comes out held by its
     back end between your thumb and index finger, pointing where the controller points. It
     points down into the quiver as you take it and lies level, out ahead of your hand, as you
     bring your forearm down.
   - With no arrows left, you feel a short buzz instead.
   - Let go of an arrow gently at the quiver and it goes back in.
3. **Nock it.** Bring the arrow's back end to the middle of the string. From about 15 cm it turns in
   your fingers to lie along the bow, and within about 9 cm it slides onto the string, with a soft
   click in both hands. You don't need to line it up.
4. **Draw.**
   - Pull back towards your chin. The string comes with your hand and the bow bends with it: the
     game's own bend of its model.
   - The bow turns so the arrow points from your string hand through the rest.
   - Twisting the bow wrist cants the bow.
   - You feel a tick in both hands every centimetre, stronger as the draw grows.
   - Full draw is the bow's own, or a little less if your arms are shorter: about 92% of your arm,
     drawn to the chin.
   - At full draw the string stops, and a low strain pulses now and then.
   - You hear the game's own `WPNBowDraw` as the draw starts.
5. **Loose.**
   - Let go of the grip. The arrow is the game's own: it leaves exactly along the drawn arrow, at
     the speed and with the drop the game gives that much draw.
   - The bow kicks in the bow hand and the string snaps in the other.
   - The limbs swing back and forth as the game's bow animation swings them, and you feel each
     swing.
   - You hear the game's `bowShoot`.
   - The arrow counts down in the inventory, and the quiver on your back shows up to five arrows,
     fewer as they run out.
6. **Short pulls.** Let go below about a fifth of the draw and nothing flies: the arrow stays on
   the string. Grip it by its back end to draw again, or anywhere else on it to take it off.
7. **Power.** A half draw flies at about two thirds speed and drops much faster than a full one.
   That's the game's own curve: your draw stands in for the time the game's bow is held drawn.
8. **Crosshair (off by default).** Open the runtime's menu, go to the Hands tab and turn on "Bow
   crosshair". While you draw, the game's own crosshair image sits straight out along the arrow,
   20 m away, like the flat game's crosshair but where the bow points. It leaves out the drop and
   never lands on buildings or things nearby. It's half the game's crosshair size and half
   see-through. The same tab has "Quiver" (shoulder, hip or both) and "Fit a bow's full draw to my
   arm".
9. **Enchanted arrows.** Equip the Arrows of Flames from the inventory. The quiver and the arrow in
   the hand become the game's Elven arrow model. The game's projectile carries the fire
   enchantment and uses it on a hit.
10. **Left-handed.** Take the bow with your right hand (reach across behind your head to the left
    shoulder). Your left hand then takes arrows over the left shoulder or at the left hip.
11. **Other bows.** `player.additem 000229BF 1` adds an Elven Bow and `00035E7C` a Daedric Bow.
    Equip one from the inventory: each is its own model, with its own measured grip and draw.

## What was checked in the game

All checks used the emulated headset with `tools/oblivion.ps1`, in the Imperial City save's inn and
at Weye.

- The bow was taken from the back by the left hand, and in another run by the right hand.
- An arrow was taken over the shoulder and at the hip, nocked, drawn to 100% and to 57%, and loosed.
- The game fired its own `ArrowProjectile` each time. Its flight was logged every tenth of a second:
  - At full draw it flew 21.7 m/s and fell at 2.1 m/s² (the game's 21.4 and 0.3 × 7).
  - At 57% draw, 14.8 m/s and about 5.4 m/s².
- The arrow count went down with each shot (168, 167, 166). With one arrow left, the game took the
  arrows off after the shot and the quiver left the body.
- Arrows stuck in wood and in the ground and stayed there. Arrows bounced off stone and the stone
  floor. That is the game's own choice.
- In captures:
  - the bow bent with the draw and the string came back with the hand;
  - the arrow lay on the bow, on the bow hand's side;
  - the Elven arrow model showed for Arrows of Flames;
  - the crosshair (then traced to where the arrow lands) sat exactly where a half-draw arrow then
    landed.
- The runtime wore the quiver part on the body, with the right count of arrows showing.
- Haptic pulses were counted: about 160 in one run (ticks, strain, nock, release, the limbs'
  swings).
- An arrow put back at the hip quiver was taken out of the hand.

## What wasn't checked

- **Anything felt or heard.** The emulated headset counts haptic pulses but can't feel them, and
  it plays no sound. `WPNBowDraw` and `bowShoot` are called through the game's own sound code (no
  error), but nobody has heard them.
- **How it feels on a real Quest 3:**
  - the tick and strain strengths;
  - the 9 cm nock reach;
  - whether 92% of the arm is the right full draw;
  - the grip's place on the bow (the fist just below the arrow).
- **The quiver on the back.** The part loads; it wasn't seen, because you can't see your own back.
  The extractor's preview shows it upright on the right of the back, as Oblivion hangs it. The bow
  slung on the back (limbs along the back, like a long gun) wasn't seen either.
- **An arrow stuck in a creature.** The test sheep wandered off the line of the shot. Hits are the
  game's own projectile, so it should behave as the game does.
- **The enchantment's fire on the arrow in flight.** It wasn't caught in a capture: it's the game's
  own effect on its own projectile.
- **Tracking loss.** Drawing with the hand near the face on Quest can lose the controller.
  Nothing smooths that yet (see below).

## Decisions

- **The game fires its own arrow, at once.**
  - It's made the way the game makes it when its bow animation lets go: its `ArrowProjectile`,
    then `UseAmmo`, fatigue, invisibility broken and the bow worn.
  - The game's attack animation isn't driven: it would add up to a second between letting go and
    the arrow leaving.
  - The power is the game's charge for a bow held drawn as long as the game takes to draw that far:
    no draw is the game's least (0.25), full draw its full charge.
- **The quiver is where Oblivion wears it.** It's over the right shoulder, plus the hip, because
  Quest loses controllers behind the head. Both are on by default; set either alone in the menu.
- **No hidden aim assist and no zoom while drawn.** The arrow goes exactly where it points. The
  game's `fArrowFOVZoom` would zoom the view, which is sickening in a headset.
- **Both hands aim.** The bow turns to the string hand, and the bow wrist sets the cant. This is
  what players praise in In Death and The Lab, and the absence of it is Skyrim VR's most common
  complaint.
- **The minimum draw is a fifth.** A smaller pull keeps the arrow on the string. This stops "gatling"
  spam.
- **The release is the grip, not the trigger.**
- **The crosshair is the game's own image** (`menus\misc\hud_reticle.dds`). It's off by default,
  because the best VR bows have none.
- **The crosshair is a guide, not a sight** (playtest 1). It sits straight out along the arrow, at
  a set 20 m, so it never jumps onto what's near. Its size in view is the same at any distance, so
  the 20 m only sets where your eyes meet on it. The aim error that leaves (the eyes sit a little
  above the arrow) is a few centimetres at 10 m.
- **The arrow is held by its nock, pointing where the controller points** (playtest 1). That's the
  controller's own aim pose (OpenXR's pointing ray), so it lies level with the forearm level on
  any controller. The Longbow holds its arrow the same way.
- **The arrow lines itself up with the bow as it nears the string**, from 15 cm (the Longbow's)
  to the 9 cm where it goes on. It only turns, about its nock: the nock stays in your fingers.
  The Longbow also slides it out of the hand onto the rest over that stretch; here the last
  centimetres slide on as it seats instead.
- **The fatigue is the game's.** Holding drawn burns fatigue as the game's own hold does (15 a
  second), and a shot costs 5. The Marksman perk waives both, as in the game.
- **No shield with a bow.** With a bow equipped the shield isn't offered: the game holds no shield
  with a bow.
- **The arrow count is the equipped stack's.** That's what the game fires before it takes the
  arrows off. The console's `EquipItem` equips a single arrow, so tests use the new `EquipAll <id>`
  console line, which equips the whole stack the way the inventory does.

## Not done

- **The enchantment's shimmer on a held enchanted arrow or bow.** Section 5 (torches, staffs and
  spells) brings the game's enchantment shimmer to held things. The game's own projectile already
  carries the enchantment in flight and on a hit.
- **The arrow count isn't shown anywhere beyond the quiver's five arrows** (no card beside the
  arrow).
- **The holsters' arrows slot (section 6).** The quiver is a set of zones arrows come from
  (`rig::zones::quiver_zones`), ready for a holster to be one more.
- **Smoothing tracking loss near the face.** The runtime has no "tracked" flag from OpenXR yet,
  only "valid".
- **Games that draw the player themselves (Metro's in-frame drawing) don't bend a bow yet.** The
  protocol's drawn objects carry no bend. Oblivion is drawn by the runtime, so it isn't affected.

## Playtests on the headset

### 1 (2026-10-30)

- **Fine:** the bow, nocking, drawing and loosing.
- **An arrow taken came out as the whole quiver, with its five arrows.** The arrow in the hand is
  the game's `arrow.nif` without its quiver. Every other branch's `oblivion-extract` writes that
  file's plain key whole, and `local_assets` is shared. Pickpocketing's extract this morning
  (06:03) overwrote the arrow alone with the whole model. Fixed: the arrow alone is now its own
  key, `#arrow`, which no other extractor writes, and the plain key is the whole model as everyone
  writes it. The arrows were re-extracted.
- **The crosshair stuck to what the arrow would hit, and vanished when aimed far off.** Asked for:
  a guide like the flat game's crosshair, where the bow points, never on buildings or things
  nearby, half the size and half see-through. Done (see Decisions).
- **The arrow out of the quiver was held awkwardly by its end, upright out of the fist like a
  dagger.** Asked for: held so it lies level as the forearm comes down, and nocking that looks
  nice, as other VR bows do. Done: held by its nock, along the controller's aim, and turned to
  lie along the bow as it nears the string (see Decisions).

## Branches

- chasms-vr-oblivion `overnight/archery`, with its framework submodule on chasms-vr-framework
  `overnight/archery`.
- The protocol is version 12: a new `Bows` table (`OFF_BOWS`), `OBJ_BOW`, `OBJ_ARROW` and
  `CE_LOOSE`.
