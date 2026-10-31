# Playtest: archery

Save: rustvr-archery

## Playing it

From the repository's root:

```
powershell -File tools\oblivion.ps1 -Save rustvr-archery
```

It builds main, installs it, and boots into Weye's open ground by Lake Rumare at 1 pm, an Iron
Bow equipped with 168 Iron Arrows, and 20 Arrows of Flames in the pack.
`powershell -File tools\oblivion.ps1 -Close` puts it away. Other bows: `-Give "000229BF 1"` (an
Elven Bow) or `-Give "00035E7C 1"` (a Daedric Bow), equipped from the inventory.

`rustvr-archery` is the developer's own save, not in the repository. To make one, boot
`rustvr-weye` (or any save) with `-Hour 13`, give a bow and arrows by their form ids, equip them
(`player.equipitem <bow>`, and `EquipAll <arrows>`: the console's `EquipItem` equips a single
arrow), and save it as `rustvr-archery` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

The bows and arrows come from the player's own install:
`cargo run --release -p oblivion-assets --bin oblivion-extract -- export meshes/weapons
textures/menus/misc/hud_reticle.dds` writes the bows, their bent models (`#bent`), the arrows
(`#arrow`, without the quiver), the quivers (`#quiver0` to `#quiver5`), `bows.toml` and the
crosshair image into `local_assets/oblivion`.

The research and the design: `docs/research/archery.md`.

## What it does

The adapter's `crates/oblivion/src/archery.rs` and the runtime's bow
(`framework/crates/runtime/src/rig/bow.rs`, the crosshair in `crosshair.rs`).

- **The bow** is the game's model, measured on it: its grip, where the arrow rests, the string at
  rest and how far the game's own bend draws it. It hangs on the back over the left shoulder, where
  Oblivion wears it, and goes into the hand by its grip, upright, thumb up, the arrow resting on
  the back-of-hand side of the riser. Either hand can be the bow hand. With a bow the shield isn't
  offered: the game holds no shield with a bow.
- **Arrows** come from the body's Arrows slot (`crates/oblivion/src/inventory.rs`), over the right
  shoulder where Oblivion wears its quiver, the equipped arrows first; for a left-handed player the
  slots are mirrored, and the holsters' editor can move it. Arrows of the bow's kind come out as
  the bow's own: the equipped arrows' model, held by the nock between thumb and index finger,
  pointing where the controller points (its aim pose), so it lies level with the forearm level.
  An arrow let go gently in a slot goes back. The quiver worn on the back shows up to five of its
  arrows, fewer as they run out.
- **Nocking** is magnetic: from 15 cm the arrow turns about its nock to lie along the bow, and
  within 9 cm of the string's middle it goes on, with a soft click in both hands.
- **Drawing:** the string comes with the hand and the bow bends with it (the game's bend of its
  model). The bow turns so the arrow points from the string hand through the rest; the bow wrist
  sets the cant. A tick in both hands every centimetre, stronger as the draw grows. Full draw is
  the bow's own, or as far as the arm draws to the chin (92% of the arm) when that's shorter. At
  full draw a low strain pulses; past it the string still comes, stiffly, half the hand's further
  pull at first and less and less, up to 15% more (6 cm on the Iron Bow), the strain growing to a
  near-steady rumble. The game's `WPNBowDraw` plays as the draw starts.
- **Loosing** is letting go of the grip. The game fires its own `ArrowProjectile` along the drawn
  arrow at once, with the power the game gives a bow held drawn that long (no draw is its least,
  0.25; full draw its full charge): a half draw flies at about two thirds speed and drops much
  faster. Then the game's own after a shot: an arrow off the stack, the shot's fatigue,
  invisibility broken, the bow worn; `bowShoot` plays. The bow kicks in the bow hand, the string
  snaps in the other, and the limbs swing as the game's bow animation swings them (`bows.toml`),
  each swing felt. Under a fifth of the draw nothing flies: the arrow stays on the string (take it
  by its nock to draw again, anywhere else to take it off).
- **Fatigue** is the game's: holding drawn burns it as the game's own hold does, a shot costs its
  share, and the Marksman perk waives both.
- **Enchanted arrows** (Arrows of Flames): the arrows in the hand and the quiver become their own
  model, and the game's projectile carries the enchantment and uses it on a hit. An enchanted bow
  wears its shimmer as any equipped weapon does.
- **The crosshair** (off by default): the game's own crosshair image (`menus\misc\hud_reticle.dds`)
  20 m straight out along a drawn arrow, half the flat game's size in view and half seen through,
  fading in with the draw. It leaves out the drop and never sits on what's in the way.

## What to try

Right-handed; everything mirrors for the left hand.

1. **Take the bow** over your left shoulder, behind your head. Let go of it there and it goes back.
2. **Take an arrow** over your right shoulder: a tick as the hand gets there, an arrow by its nock.
   With none left, a short buzz.
3. **Nock it:** bring its back end to the string's middle; you don't need to line it up.
4. **Draw** towards your chin, then on past full draw, feeling it stiffen.
5. **Loose.** Watch the arrow leave along the drawn arrow, the limbs swing, the count go down and
   the quiver on your back thin.
6. **Short pulls:** let go under a fifth: nothing flies.
7. **Power:** compare a half draw with a full one.
8. **The crosshair:** Hands tab, "Bow crosshair". Draw: it sits out along the arrow, 20 m away.
9. **Enchanted arrows:** equip the Arrows of Flames from the inventory and shoot something.
10. **The other hand:** take the bow with your right hand (across, behind your head to the left
    shoulder) and draw with the left.
11. **Other bows:** an Elven Bow and a Daedric Bow, each its own model, grip and draw.

## Settings

The Hands tab (`framework/crates/runtime/src/menu.rs`; `[archery]` in `settings.toml`):

| Setting | Key | Default | What it does |
|---|---|---|---|
| Quiver: Over the shoulder, At the hip, Both | `quiver` | Both | Where arrows come from in a game without body slots. Oblivion hands its arrows out from its slots, so here it does nothing. |
| Bow crosshair (where a drawn arrow points) | `crosshair` | off | The crosshair above. |
| Fit a bow's full draw to my arm | `fit_draw` | on | Off: full draw is always the bow's own. |

## Decisions

- **The game fires its own arrow, at once,** made as the game makes it when its bow animation lets
  go. The attack animation isn't driven: it would add up to a second between letting go and the
  arrow leaving. The draw stands in for the time the game's bow is held drawn.
- **No hidden aim assist and no zoom while drawn** in the headset: the arrow goes where it points,
  and the game's `fArrowFOVZoom` would be sickening in a headset. (The desktop view's aim zoom is
  the studio's, `docs/playtest/fxstudio.md`.)
- **Both hands aim:** the bow turns to the string hand and the bow wrist sets the cant. Players
  praise it in In Death and The Lab; its absence is Skyrim VR's most common complaint.
- **The minimum draw is a fifth,** so a short pull is no shot: it stops "gatling" spam.
- **The release is the grip, not the trigger.**
- **The crosshair is the game's own image, off by default,** because the best VR bows have none.
  It's a guide, not a sight (playtest 1): at a set 20 m along the arrow it never jumps onto what's
  near, and since its size in view is the same at any distance, the 20 m only sets where the eyes
  meet on it (a few centimetres of error at 10 m, the eyes being above the arrow).
- **The arrow is held by its nock along the controller's aim pose** (playtest 1), so it lies level
  on any controller, as the Longbow holds it. It turns to the bow from 15 cm (the Longbow's) to the
  9 cm where it goes on, only about its nock: the nock stays in the fingers.
- **Past full draw the string stiffens instead of stopping** (playtest 1), the bend model drawn on
  past its full bend too. The shot stays the game's full charge (its power stops at 1): the extra
  draw is for the feel.
- **The arrows are where Oblivion wears its quiver.** With the holsters merged they're a body slot
  like the player's other things, so they can be moved; the runtime's own quiver zones (shoulder,
  hip) are for games that don't hand out their things from slots.
- **The arrow count is the equipped stack's:** what the game fires before it takes the arrows off.
  Tests equip the whole stack with the console's `EquipAll <id>`.

## Not done / known issues

- **A held enchanted arrow** doesn't wear the enchantment's shimmer; the projectile carries the
  enchantment in flight and on a hit.
- **Tracking loss near the face:** drawing with the hand by the face can lose a Quest controller,
  and nothing smooths it: the runtime has OpenXR's "valid", not "tracked".
- **A game that draws held things itself** (Metro's in-frame drawing) wouldn't bend a bow: the
  protocol's drawn objects carry no bend. Oblivion is drawn by the runtime.
- **Never checked on the headset:** the sounds; the haptics' strengths; the 9 cm nock reach;
  whether 92% of the arm is the right full draw; the grip's place on the bow; the quiver and the
  slung bow on the back (you can't see your own back); an arrow stuck in a creature; the
  enchantment's fire on an arrow in flight. The emulated headset checked the shots' speed and
  drop against the game's (21.7 m/s at full draw, 14.8 at 57%), the count going down, arrows
  sticking in wood and the ground and bouncing off stone, the bend, the Elven model for Arrows of
  Flames and the crosshair.

## History

Newest first.

- **Playtest 1 (2026-10-30):** the bow, nocking, drawing and loosing were fine. An arrow taken came
  out as the whole quiver with its five arrows: another branch's extract had overwritten the arrow
  alone in the shared `local_assets` with the whole model; the arrow alone is now its own key,
  `#arrow`. The crosshair stuck to what the arrow would hit and vanished aimed far off: now a guide
  straight out along the arrow, half size and half seen through. The arrow was held upright out of
  the fist like a dagger: now by its nock along the controller's aim, turning to the bow as it
  nears the string. Full draw felt like a wall: now the string gives up to 15% more, ever stiffer.
