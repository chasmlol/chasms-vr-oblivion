# Archery: research and design

How the best VR bows work, what players love and hate about them, how Oblivion does its own
archery, and the design built from all three: a general bow in the framework, Oblivion's own bows,
arrows, sounds and projectiles through its adapter. What other games do is inspiration only; nothing
here is copied.

## What players say

Sources: Valve's Longbow (The Lab; its code ships in the SteamVR Unity plugin, the best source of
hard numbers there is), In Death and In Death: Unchained, QuiVr, Skyrim VR and its archery mods,
Elven Assassin, Holopoint, Twisted Arrow, Asgard's Wrath, Thief VR, with their
Steam forums, reviews and developer posts.

**Loved**

- **An arrow without fuss.** The best-reviewed bows hand the player an arrow at the string (In
  Death, the Longbow) or keep a forgiving quiver over the shoulder (QuiVr, Holopoint). Nobody praises
  fumbling for an arrow; reaching over the shoulder is loved when it always works.
- **Magnetic nocking.** The arrow seats itself on the string once it's within about 10 cm; the player
  never lines it up (the Longbow blends it in from 15 cm and seats it at 8 cm).
- **Both hands aim.** The arrow points from the string hand through the bow: the bow turns to the
  string hand, and the bow wrist's twist sets the cant. Skyrim VR's default, where only the bow hand
  aims, is the most complained-about.
- **A draw you feel.** Ticks in both hands as the string comes back, stronger the further it goes, and
  an unmistakable full draw (the Longbow's random strain pulses; In Death's growing rumble). QuiVr's
  players praised the string's audible tension.
- **The arrow goes exactly where it points.** In Death has no crosshair and is called the best bow in
  VR, because the arrow leaves exactly along the drawn arrow. Holopoint's bow snapping a few degrees
  as the arrow left was its most hated bug.
- **Power you can see.** A short pull is a weak, dropping shot and a full draw a fast flat one, in a
  way the player can read. QuiVr's first, front-loaded curve made players think the draw didn't
  matter, and its developer changed it to a real recurve's.

**Hated**

- A full draw that needs the hand behind the head, or a draw the player's arms are too short for
  (In Death, on shorter players).
- Tracking lost when the drawing hand is beside or behind the head (Quest and Rift S cameras),
  giving weak or wild shots, and quivers that can't be reached reliably behind the head.
- The bow ignoring the wrist's roll, or held at a fixed odd angle (Skyrim VR, Thief VR).
- Arrows not leaving where the drawn arrow was (Holopoint).
- Weak bows with heavy drop, or arrows that fly forever (Skyrim VR).
- No left-handed bow, or the arrow nocked on one side only.
- Exploits from instant arrows and a tiny minimum draw (Elven Assassin's "gatling", Skyrim's spam).
- Harsh, metallic nock clicks that wear in rapid fire (QuiVr's brief Longbow sounds).
- Crosshairs: the acclaimed games have none. Skyrim's appears only with a perk. Where there is one,
  it's an aid players choose.

**Numbers worth having** (the Longbow's unless said)

| What | Number |
|---|---|
| Nocking | within 10 cm of the nock point; seated in an eased blend |
| Minimum draw | 5 cm (Longbow); Skyrim VR about 25 cm. Below it, letting go doesn't shoot |
| Full draw | 50 cm (Longbow); the bow's own here, short of it for short arms |
| Speed | linear in the draw (the stored energy of a spring goes with the draw squared, so speed is linear) |
| Bow turning to the string hand | eased over 0.15 s after nocking |
| Draw haptics | a tick every centimetre, both hands, 100 µs at rest rising to 500 µs at full draw |
| Full-draw strain | 400 µs pulses 0.1 to 0.5 s apart |
| Release | a fading burst on the bow hand: 1500, 800, 500, 300 µs, 50 ms apart |
| Nock | one 500 µs pulse |

## How the AAA ones do it

- **The Lab's Longbow**: no quiver; an arrow appears in the empty string hand 0.5 s after a shot,
  its nock at the hand and its shaft out ahead of the fist (`ArrowHand.cs`). From 15 cm to 8 cm
  from the string's nock point it blends onto the bow's rest, turning and moving
  (`rotationLerpThreshold`, `positionLerpThreshold`, `lerpCompleteDistance`), with a 500 µs pulse
  as it's lined up; within 10 cm a pinch nocks it. The bow pivots about its grip to look at the
  string hand, its up from the bow hand. A hidden assist
  pulls the aiming point up to 6 cm towards a point under the eye as the draw grows. A sphere cast
  along the arrow before the release stops shots from inside a wall.
- **In Death**: the trigger at the string nocks an arrow; no crosshair; strict hitboxes; damage grows
  with the draw.
- **QuiVr**: over-the-shoulder quiver, the draw curve of a real recurve, a hand-position filter.
- **Skyrim VR**: arrows appear in the hand; `fArrowLerpDistance` (18 units, about 26 cm) and
  `fArrowSnapDistance` (about 18 cm) take the arrow from the hand onto the bow as it nears, and
  players pitch the held arrow 40 to 45 degrees (`fArrowHoldRotationPitch`) to lay it along the
  index finger; `fArrowDistanceToFire` stops weak releases; mods add a
  real over-the-shoulder quiver (SRAVR) and remove the second press (Seamless Arrow Nocking).

## Oblivion's archery

- **The bow** is one model whose limbs and string bend by a morph, its target the full draw. The
  player's bow animation plays the draw and loose sounds (`WPNBowDraw`, `bowShoot`), brings the
  morph up to full at the hold and, after the release, swings it a few times: the limbs' vibration,
  the game's own. The bow hangs on the back, over the left shoulder.
- **The arrows' model** holds the arrow alone and the quiver, which shows up to five arrows. The
  quiver hangs upright on the right of the back, its opening at the right shoulder.
- **A shot's power** grows with how long the bow is held drawn (`fArrowBowTimerBase`,
  `fArrowBowTimerMult`), and the arrow's speed and drop go with it. A shot costs fatigue and a held
  draw burns it, unless the Marksman perk waives it; a shot dispels invisibility and wears the bow.
  Drawn, the flat game zooms (`fArrowFOVZoom`).

## The design

Built as designed, with the changes the playtests asked for: the framework's `rig/bow.rs`,
`crosshair.rs` and `rustvr_protocol::archery`, and the adapter's `archery.rs`. In Oblivion the
arrows come from the body's slots rather than the quiver (below).

### Framework: a general bow

**Objects.** A bow is `OBJ_BOW`, in the guns' object space (+Z where the arrow flies, +Y up its
upper limb, X across it): its grip a handle (`grip_line`) the fist takes, thumb up; `muzzle` the
arrow rest; `handle` the string's nock point at rest; `handle_travel` the string's travel at the
bow's full draw. An arrow is `OBJ_ARROW` (+Z its point, the tip at `muzzle`, the nock at
`mag_well`), registered as a template with the bow's `ammo_type`; with `OBJF_GAME_AMMO` its
`rounds` are the game's count. A `Bows` table says what the object table can't: the bow's bent
model (the morph target, for drawing it bent), its release vibration (the game's own curve), and how
the game's arrow flies at no draw and at full draw (speed and gravity, the speed told to the game
with each shot), and the game's crosshair texture.

**The rig** (`rig/bow.rs`):

- **Holding**: the bow by its grip, in either hand: the hand holding it is the bow hand and the
  arrow rests on that hand's side of it, so a left-handed archer is the mirror image.
- **The quiver**: while the other hand holds a bow, gripping over the shoulder (behind the head) or
  at the hip on that hand's side (the Hands tab says which, both by default) takes an arrow, held
  by its nock in the pinch, pointing where the controller points (playtest 1: along the fist it
  stood up like a dagger), ready to nock; with the game's count, none when the game has none left
  (a fail cue). The quiver is a set of zones the arrows come from (22 cm round the shoulder slot,
  15 cm round the hip's). For a game that hands out its own things from the body's slots (Oblivion
  does), there is no quiver: the arrows come from whichever slot they're kept in.
- **Nocking**: from 15 cm (the Longbow's) an arrow nearing the string turns about its nock, still in
  the fingers, to lie along the bow, all the way by 9 cm. Within 9 cm of the string's nock point,
  pointing anywhere forward of the string, it seats itself on the string (eased in over 0.12 s, a
  click and a pulse in both hands). The hand holding it now holds the string.
- **Drawing**: the string comes back with the hand, along the bow's arrow line. The bow turns about
  its grip so the arrow points from the string hand through the rest (eased in over 0.15 s as it's
  nocked, then exact), its roll the bow wrist's own. Past full draw the string stiffens rather than
  stopping (playtest 1): drawn further it comes back at first half as far as the hand, less and
  less after, up to 15% more, with the strain growing; the hand stays on the string and the ghost
  hand shows the controller. The shot is the game's full charge from full draw on. Full draw is
  the bow's own, or less for a player whose arms are shorter (92% of the arm's length from the
  grip, less the string's brace, the draw an archer anchors at the chin; never under half the
  bow's own; the Hands tab's "Fit a bow's full draw to my arm", on by default). The bow bends with
  the string: its morph's weight is how far the string is drawn of the bow's own travel, so the
  string is where the hand is.
- **Feel**: a tick in both hands every centimetre of draw, stronger as it goes; at full draw a low
  strain now and then; a click as it nocks.
- **Release**: letting the string hand's grip go. Past the minimum draw (a fifth of full) the arrow
  is loosed: a `CE_LOOSE` combat event gives the game the arrow's tip, its way along the bow and the
  draw (0..1 of full), and the runtime's arrow is gone (the game flies its own). The bow then plays
  the release vibration (the game's own curve of its limbs), felt as a fading burst in the bow hand
  timed to it, and a snap in the string hand. Short of the minimum the arrow stays on the string,
  resting there, to be drawn again or taken off.
- **Taking it off**: a hand gripping a nocked arrow away from the string takes it off the bow.

**The crosshair** (Hands tab, off by default): while the string is drawn, the game's own crosshair
texture straight out along the arrow, 20 m from its tip: where the bow points, as the flat game's
crosshair shows where the view points, not where the arrow will land. It leaves out the drop and
is never put on what's in the way (the player's first playtest asked for a guide, not a cheat).
Facing the eye, half the size in view of the game's own and half seen through, at any distance.

### Oblivion

- **The bow**: the equipped bow's own model and its bent morph (`oblivion-extract` writes the bow and
  `#bent`, the morph at full), registered as the runtime's `OBJ_BOW` on the back (where Oblivion
  wears it), measured on the model: the grip, the rest, the string and its travel.
- **The arrows**: the equipped arrows' own model (`#arrow`, the arrow alone) as the template, their
  count the game's (the equipped stack). They're drawn from the body's Arrows slot
  (`crate::inventory`: over the right shoulder, where Oblivion wears its quiver, the equipped
  ones first) or any slot the player keeps arrows in. Arrows of another kind drawn while a bow is
  on are equipped by the game, and one of the bow's own takes the drawn copy's place in the hand.
  Not built: the arrows' name and enchantment on a card.
- **The limbs' spring** after a shot is the game's own (`bows.toml`, written by `oblivion-extract`
  from `bowattack.kf`'s morph keys), and the runtime is told how the game's arrow flies at no draw
  and at full draw (its speed and gravity, from the game's settings) and the game's crosshair
  (`textures\menus\misc\hud_reticle.dds`).
- **Release**: the game's own shot, made as its release makes it, at once: the arrow from the VR
  arrow's tip along its line (the projectile's heading and pitch), the power the game's own for the
  draw: the draw stands in for the hold timer, so a full draw is a full charge and no draw the
  game's least (`fArrowBowTimerBase`). Then the game's own `UseAmmo`, fatigue, invisibility and wear, and its
  `bowShoot` at the bow. `WPNBowDraw` plays as the draw starts, and the fatigue burns a second while
  drawn, as the game's own hold does.
- Everything else is the game's: the arrow's flight, its hit, damage, enchantment, sneak attack,
  crime and aggression, sticking where it lands, and being picked up again.

### Decisions

- **The game's quiver place first.** Oblivion wears its quiver on the back with its arrows at the
  right shoulder, so over the shoulder is the game's own; the framework's quiver offers the hip
  too, for headsets that lose the controller behind the head (both on by default). In Oblivion
  that place is the Arrows slot's, which the player can move like any slot.
- **No zoom while drawn** (`fArrowFOVZoom`): zooming the view in a headset is sickening.
- **No hidden aim assist.** The arrow leaves exactly along the drawn arrow, as players ask.
- **The game's power curve, not a new one**: the draw replaces the game's hold time, so the game's
  own numbers decide speed, drop and damage.
