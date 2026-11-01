# Playtest: riding

Save: rustvr-horse

## Playing it

From the repo root:

    powershell -File tools\oblivion.ps1 -Save rustvr-horse

It builds main (the adapter and the runtime), installs it beside the game and boots straight into
the save for the headset. `powershell -File tools\oblivion.ps1 -Close` puts the game away and your
settings back. For an evening ride in clear weather add `-Commands "set gamehour to 19|fw 00038EEE"`
(as `clips/07-riding-evening.bat` does).

`rustvr-horse` is Chorrol, just outside its stables, at 11 am. Your own horse (Chorrol's "My
Chestnut Horse", made yours) stands about two metres behind you, a little to the left. The two
horses in the paddock are the stable's: getting on one is stealing it.

The save is the developer's own test save, not in the repository (`docs/DEVELOPING.md` has the test
saves). To make one from any save:

    powershell -File tools\oblivion.ps1 -Save <any save> -At ChorrolStables00 -Hour 11 -Commands "PlayerHorse 0004DE90 2"

Then save it as `rustvr-horse` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

`PlayerHorse <base form id> [metres]` is the adapter's test line (`crates/oblivion/src/console.rs`,
`riding::give_horse`): the game places a horse of that base that far from you (`placeatme`) and the
adapter makes it yours (`SetOwnership`). `0004DE90` is "My Chestnut Horse" in `Oblivion.esm`;
`0001F11D` a bay, `0001F11A` a black and `00032BF4` Shadowmere. Arriving at `ChorrolStables00`
brings up the main quest's "arrived in Chorrol" message.

The logs: the runtime's `%LOCALAPPDATA%\RustVR\rustvr.log` has `riding:` lines (twice a second while
a rein is held: each hand across from your middle and ahead of the mane, the pull, the gait, speed
and steering sent; every spur, pull, ease and gait change). The game folder's `rustvr-oblivion.log`
has the adapter's `riding:` lines (what the game's controls were given, the gait changes), and the
`d3d9:` and `mem:` lines that matter for the crashes in round 10.

Why riding works the way it does (what players like and hate, how other VR games ride):
`docs/design/riding.md`. How it's wired into the game: the Riding row of `docs/OBLIVION.md`.

## What it does

The game keeps its horse: its model and animations, its speeds and turning, whose it is, the crime
of stealing it, getting on and off and their sounds. The runtime has the rider
(`framework/crates/runtime/src/riding.rs`, `reins.rs`); the adapter tells it where the horses are and
gives the game the rider's controls as its own (`crates/oblivion/src/riding.rs`).

- **Getting on.** Every free horse the game processes within 6 m offers its saddle (measured on the
  game's own model, on the horse's live spine). A free hand closing anywhere along it, from the horn
  to the cantle, is the player's own activation of the horse: the game's check of whose horse it is,
  its "steal this horse?" and the crime, its mount animation. The trigger at gaze (the game's
  Activate) still gets on too. The view fades while the game's animation puts you in the saddle, and
  you're recentred facing the horse's way.
- **In the saddle.** The view's anchor is the rider's eye 0.85 m over the saddle's seat on the
  horse's live spine, eased over 0.35 s so the gait's bob goes. The horse's turn carries the view
  round with it (smoothly, or in steps); your head never steers, and you look round freely. Your body
  sits in the saddle, legs astride round the horse's barrel. The ridden horse isn't one of the
  creatures the hands meet, so they don't push against it.
- **The reins** are held half way from the saddle horn to the base of the neck, 9 cm either side of
  the mane. A free hand gripping within 15 cm takes the rein on its side (a tick in the hand); one
  hand alone holds both, western style. A hand on a rein closes into a rider's fist round it from
  the grip alone, the thumb on top (`HoldShape::Rein`), whatever its trigger does.
- **Taking them up.** A rein just taken follows its hand and does nothing until the hand comes to
  rest (slower than 0.15 m/s for 0.25 s, at least 0.3 s after taking it, 2.5 s at most): where it
  stops is where the rein is held from, and a soft tick says so. A trigger squeezed with the grip as
  the rein is taken isn't a spur.
- **Steering** is measured against you and the horse, in your frame (turned to the horse as it was
  when you took the reins):
  - both hands to one side: their middle across from your own middle (where your head sits across
    the horse, followed over 10 s and never more than 15 cm from the mane), past 3 cm;
  - one rein further back than the other: hand against hand, past 2.5 cm;
  - the turn is the square of the way past that, all of it 30 cm past ("Rein travel for a full
    turn"), smoothed over 0.12 s. 10 cm past turns it about 7°/s, 15 cm 17°/s, 20 cm 30°/s, hands
    well out to a side the game's full 67.5°/s;
  - the horse is turned directly: each game frame its heading is turned by the game's own heading
    turn (the one its horse turning calls) at your share of `iHorseTurnDegreesPerSecond` (45, read
    live) times the turn keys' 1.5. No ramp, no effect on its speed, and it turns standing too. The
    game's stick X stays 0.
- **Speed** is a share of the horse's full gallop, and fluid:
  - **a spur**, a flick of a rein hand (snapped down faster than 1.1 m/s, 4 cm or more, and stopped)
    or a squeeze of a rein hand's trigger (past 30%), is as hard as it's done: the flick's speed up to
    3.5 m/s, the squeeze's depth. It surges the speed by a tenth of a gallop for the lightest, all of
    it for the hardest, rising at half a gallop a second; the surge fades over 8 s to 70% of itself.
    Spurs add up: keep spurring hard to stay flat out. A spur is always heard, and gives reins held
    back (where the hands are becomes their rest);
  - **both reins pulled back** slow it as hard as they're pulled: nothing for the first 5 cm, hardest
    at 15 cm (0.7 of a gallop a second: from a full gallop to a stand in about a second and a half).
    Where you ease off, it keeps that speed;
  - the adapter makes the speed of the game's two gaits at a pace, with the game's own speeds for
    that horse (its walk speed, and its gallop as 2.1 times its run speed), so Shadowmere is still
    faster than a paint horse. Slow, it walks (up to the game's walk, about 1.3 m/s on the
    chestnut); past half way from the walk to 60% of its gallop it gallops, never asked for under
    60% of the gallop (slower looks like a slowed-down sprint).
- **Backing up** is its own request: at a stand, ease the reins, pull them right back again and hold
  1.2 s. Ease off and it stands. Slowing to a stand never backs it.
- **Let go of the reins** and it keeps its speed 2.5 s, then eases to a stand (a fifth of a gallop a
  second).
- **The sticks ride too.** The left stick ahead or to a side goes at its push (click it: the gallop)
  and steers with its sideways push; pulled back (within 60° of straight back) it backs up; let go,
  it stops. The turning stick steers alone, standing too.
- **A** is the game's Jump, on the horse and on foot.
- **Getting off** is deliberate: hold the trigger of a hand that isn't on a rein for 0.6 s, or hold
  the saddle horn and lean your head 35 cm out over one side (0.3 s). The view fades through the
  game's dismount. While mounted the trigger never reaches the game's Activate (which would get you
  off).
- **No fighting from the saddle**, the game's rule: the hands still swing, nothing lands, and casting
  is off.
- **Comfort.** A riding vignette closes in as the horse turns (fully at a radian a second, when the
  view turns smoothly) or gallops. Haptics: a tick taking a rein and taking it up, a pulse in the rein
  hands at each spur, a softer one as the reins start slowing it and when the gait drops, and the
  stride's beat in the rein hands.
- **The reins drawn.** Oblivion's horses have a bridle and bit but no reins, so the runtime draws
  them: leather straps, 16 mm wide and 4 mm thick, cut from the horse's own tack texture (a plain
  stretch of the bridle's cheek strap in it, with its normal map and the saddle's material), drawn
  with your body as held things are, so the game's depth hides them behind the horse and its lights
  light them. Each runs from a ring of the bit to the fist holding it, in at the little finger; let
  go, it lies along its side of the neck. Each is a rope carried from frame to frame, hanging under
  gravity with 6% slack, held at the bit and the hand, and kept 1 cm off the horse's neck and head.
  With both hands on the reins the bight between them comes out over the index fingers and just
  hangs between the fists.
- **The crosshair is gone** (on foot too): the dot in the middle of Oblivion's reticle is hidden
  (`crates/oblivion/src/reticle.rs`); the icon of what you can activate (red when taking it is
  stealing) still shows.

## What to try

1. **Get on by the saddle.** Turn round, walk up to your horse and close a hand on the saddle (its
   horn or its seat). The view fades, the game's mount plays behind the fade, and you're in the
   saddle facing the way the horse faces.
2. **Look down and round.** Legs astride, hands free. Look about: the horse never turns with your
   head, and its head and neck stay in front of you.
3. **Take the reins** where they lie along the neck, just in front of the saddle horn, with either
   hand or both. Draw your hands in to where they're comfortable: nothing happens until a soft tick,
   then the reins are live from there. Look at the fists: closed round the rein, thumb on top, the
   rein going in under the little finger.
4. **Sit still** with your hands relaxed for a while: nothing should happen.
5. **Spur on.** A gentle flick or a light squeeze from a stand: a slow walk. Another: faster. A sharp
   flick: a gallop. Spur hard again and again: flat out. Leave it: it eases back to a steady pace over
   ten or twenty seconds.
6. **Steer** with tiny moves (a centimetre or two past centre): a slow drift. Wider: harder. Hands
   well out to a side: the fastest turn. Try it standing, walking and galloping: it should never stop
   or speed up to turn. Hold your hands close to your body, centred in front of you: it goes
   straight.
7. **Slow and stop.** Pull both reins a little, then more: it slows gently, then harder. Let go of the
   pull at a canter, a walk or a stand: it keeps it.
8. **Back up** on purpose: at a stand, ease the reins, pull them right back again and hold a second.
   Then try to back it up by accident: it shouldn't.
9. **Let go of the reins** at a gallop: it keeps going about 2.5 s, then eases to a stand.
10. **The sticks**: left stick to ride and steer (click: gallop), back to back up; the right stick to
    turn.
11. **A** to jump.
12. **Get off and back on**, two or three times: the view sits in the saddle each time, never above
    or beside the horse. Get off both ways: a held trigger on a free hand, and the horn held while
    leaning well out.
13. **Watch the reins** while riding, turning and galloping, and with your hands on the withers,
    along the neck or pushed into it: they follow calmly, lie along the neck and never jump or
    flicker. Ride past a lamp or into shade at night: they darken and light like your gloves.
14. **Ride for a good while**: gallop across cell borders, along water and past the paddock's horses.
    If the picture freezes for half a second (a lost device), carry on: it should come straight back.
15. **Steal one**: get on one of the paddock's horses. It's the game's crime (a watching guard
    arrests you).

## Settings

The runtime's menu (the menu button), Comfort tab, under "Riding" (`[riding]` in
`%LOCALAPPDATA%\RustVR\settings.toml`):

| Menu | Key | Default | Range |
|---|---|---|---|
| The view turns with the mount: Smoothly / In steps | `turn` | `smooth` | `smooth`, `snap` |
| Step (shown with In steps) | `turn_step` | 30° | 20 to 90° |
| Vignette while turning or galloping | `vignette` | 0.5 | 0 (off) to 1 |
| Fade out while getting on and off | `fade_on_off` | on | |
| Reins pulled before they slow it | `rein_dead` | 0.05 m | 0.01 to 0.12 m |
| Rein travel for a full turn | `rein_travel` | 0.30 m | 0.05 to 0.50 m |
| Rein pull to slow hardest | `rein_pull` | 0.15 m | 0.08 to 0.30 m |
| Spurred, it settles at (of the surge) | `cruise` | 0.7 | 0.2 to 1 |
| A spur's surge fades over | `spur_fade` | 8 s | 2 to 30 s |
| Flick the reins to spur on | `flick` | on | |
| Reins let go: keeps its pace for | `coast` | 2.5 s | 0 to 10 s |
| Feel the stride in the reins | `stride_haptics` | on | |

The Comfort tab's "Camera bob / shake" (`[comfort] head_bob`, off) brings the gait's bob back on
horseback too. The other numbers above are constants: the hands' (the steering's give, the flick,
the spur, the brake, getting off) at the top of `framework/crates/runtime/src/riding.rs`, the
horse's (the eye over the seat, the gallop's least and its 2.1) in `crates/oblivion/src/riding.rs`.
The Help tab lists the riding controls.

## Decisions

- **Getting on is the game's activation**, not an animation of our own: the game checks whose horse
  it is, makes stealing a crime and plays its mount. The view fades meanwhile (on by default): the
  game's animation moves the view as the player doesn't, and a camera taken over by an animation
  sickens.
- **Getting off is deliberate**: a held trigger on a free hand, or the horn held while leaning out.
  Never the plain trigger: Skyrim VR players hate getting off by accident.
- **Steering by the reins as riders do**: one rein back, both hands to a side (a neck rein), or a rein
  opened out (an equestrian's opening rein not working was a complaint about another game). One hand
  can hold both.
- **Steering is measured against the rider and the horse** (rounds 7 and 8). Measured from where the
  reins were taken, "centred" depended on where the hands happened to be then and a turn could stick;
  measured from the mane, a rider sitting off the horse's middle turned with hands held close.
- **Fine turns, the hardest well out** (round 8): the square of the way past the give, all of it 30 cm
  past, so a few centimetres turn it a degree or two a second.
- **The horse is turned straight** (round 5). The game reads a rider's stick as a direction (in the
  player's update): once the turn is stronger than the forward push it's a slide, which on a horse
  halts it and turns it on the spot, and it ramps a horse's turn up at 80°/s². Turning the heading
  directly at the game's own rate is continuous and never stops or speeds the horse.
- **Speed is fluid, made of the game's gaits** (rounds 4 and 5): the reins ask for a share of the
  gallop and the adapter makes it of the game's walk and gallop at a pace with the game's speeds for
  that horse, so every horse goes as fast as the game makes it. The top speed is the game's full
  gallop, not raised. The gallop's 2.1 times `GetRunSpeed` was measured on the headset (8.6 m/s where
  `GetRunSpeed` says 4.1).
- **Hands at rest do nothing** (round 2): the pull has a 5 cm give, and a hand within 1.5 cm of its
  rein's rest carries the rest along over 1.5 s, so drift never adds up to a pull. A held pull slows
  only to a stand; backing is asked for on its own, so it can't follow from slowing down.
- **The view sits on the saddle** (round 3): after getting off and on again the game's own camera
  stays at a standing player's eye on the horse's back (1.9 m over the seat) and lags the horse.
  0.85 m over the seat is where the game's camera was on every first mount: it's the one riding value
  read off the game in play rather than its data.
- **The head never steers.** The view's yaw follows the horse; its pitch and roll never do.
- **No bob by default**: the rider's eye is held steady on the horse; the Comfort tab's camera bob
  brings the game's bob back.
- **The riding vignette** is its own setting (0.5, like walking's): on a horse the stick rides it, so
  the horse's turning and galloping close the vignette instead.
- **The seat and legs are the saddle's and the horse's body's**, measured on the game's models: in
  first person Oblivion doesn't pose its rider (the rider's tracks in the horse's animations play
  only on the third-person body). The legs go round the barrel at a rider's proportions (knee 0.43 m
  below the seat).
- **The reins are the runtime's**, since Oblivion's horses have none (its bridle is the size of the
  head): from the bit's rings (measured at the corners of the horse's mouth on its head model), in
  the tack's own leather (the bridle's plain cheek strap, chosen by eye, mirrored along the rein so it
  has no seams). Rope (the game has `clutter\rope.dds`) was the other choice; the bridle's leather
  matches what the reins hang from.
- **The reins are ropes** (round 4): a rein laid afresh every frame jumped up to 90 cm a frame near the
  neck; carried from frame to frame (28 points a rein, sliding on their own at most 1.5 m/s), no rein
  point moved more than 13 mm a frame in the same offline sweeps.
- **The reins lie on the neck**: `oblivion-extract` measures the horse's neck and head as
  cross-sections, each an ellipse fitted to the model from outside (every section holds the model to
  within 7 mm), and the adapter sends them on the live bones. They may sit 3 cm behind the horse's
  skin before the game's depth hides them, so they don't flicker against it. Their horse end goes
  with each eye's picture of the horse when a picture is moved on to where the head is.
- **The bight just hangs** between the fists (round 9): laid on the neck like the reins, it was
  pushed up over the neck's rounded end behind the withers, where there's no neck.
- **A rider's fist** (round 6): the hand closes round a rein from the grip alone, the curls chosen by
  eye; the rein goes in 3.5 cm down the grip's axis from the palm's middle and the bight comes out
  3.5 cm up it.
- **No fighting from the saddle**: the game's rule.
- **A jumps**: the player rig holds Jump while A is down, so the input bridge jumps on its press
  rather than counting presses (before, A never reached the game's Jump, on foot either).
- **A read that faults is unreadable, not the end of the game** (round 10): riding brings cells in
  and out quickly, and the game's loading thread can free memory between the page check and the read
  (`crates/oblivion/src/mem.rs`).
- **A device wanting a reset inside the game's render is reported as lost** (round 10): the game's
  renderer resets as soon as it hears it, and a reset in the middle of drawing the water's reflection
  freed the reflection's camera under it. Reported as lost, the game draws nothing more that frame
  and its main loop resets the device before the next, as it already does whenever it finds it lost
  there. Nothing of the mod's is made on a lost device (it kept the reset from succeeding)
  (`crates/oblivion/src/d3d9.rs`).

## Not done / known issues

- No ride on the headset is recorded since round 10's crash fixes. The third crash (inside NVIDIA's
  driver, five seconds after a device reset) may have followed from the mid-frame resets; if it comes
  back, the adapter log's last `d3d9:` lines before it show whether a reset came first.
- The gallop's 2.1 times the game's run speed was measured on the chestnut horse only.
- The horse turned on the spot turns without a turning animation (if the game has one for horses).
- The body's seat and legs are where the game last put the horse, not where a moved picture shows it
  (as the reins are): at a gallop they can sit a few centimetres off the picture's horse.
- The game's own riding pose for the body (the game doesn't animate it in first person); the legs
  don't swing with the gait (the seat does, with the horse's spine).
- The reins touch only the horse's neck and head, not your body or the saddle.
- Shadowmere's own tack isn't used for its reins: every horse's are the ordinary tack's leather.
- Mounted combat (the game's rule is kept).
- The Help tab's riding rows are older than the controls ("slow down a gait", "spur on: walk, then
  gallop").
- Not tried: a real horse jump seen in the game, riding into water or a swimming horse, fast travel
  with the horse, getting off beside walls, left-handed play, the game's own steal prompt (the game
  arrested instead).

## History

Newest first.

- **Round 10: three crashes.** The rides on round 9 crashed three times: a read of a skeleton the
  game's loading thread had just freed (now skipped as unreadable, `mem: a read at ... faulted`); the
  game resetting its device in the middle of drawing (now reported as lost there, reset between
  frames); and one in NVIDIA's driver after a reset.
- **Round 9.** "It worked perfect." The loop between the hands floated up and misbehaved: it now just
  hangs between the fists by its slack.
- **Round 8.** "Way more subtle turns, all the way out to the side the fast turns": the turn is now the
  square of the way past the give, full at 30 cm (was 10 cm, half linear). Hands held close to the
  body steered: now measured from the rider's own middle, not the mane.
- **Round 7.** It kept turning with both hands centred on the horse: steering was measured from where
  each hand took its rein; now against the horse (the mane, and one hand against the other).
- **Round 6.** The index finger lifted unless the trigger was held, and the hands on the reins wanted
  rigging: a rider's fist from the grip alone, the rein running through it.
- **Round 5.** Turning was all or nothing (a 5 cm give, full at 17 cm, the game's slow ramp, and the
  forward push raised to match): now fine from small moves, and the heading turned directly. The walk
  was a slowed sprint (the game's run speed taken for the gallop): now the game's walk, and the gallop
  never under 60%. A mini spur walked it slowly: spurs became as hard as they're done.
- **Round 4.** Spurs and pulls asked for a fluid speed (a surge that fades to a cruise, a pull that
  slows as hard as it's pulled and keeps the speed): done. The reins glitched near the horse: they
  became ropes carried from frame to frame, with less slack.
- **Round 3.** After getting back on, the view floated above and beside the horse: it's now anchored
  on the saddle. The horse stopped in hard turns (the game's slide), first fixed by keeping the forward
  push up, replaced in round 5. The reins went into the neck: the neck was refitted from outside and
  the reins move with the moved picture. Drawing the reins in from the neck read as a pull: taking up
  the reins.
- **Round 2: the first ride on the headset.** The flat-screen crosshair showed: hidden. The horse
  backed up and drifted sideways (a backing bug, a 2.5 cm give, flicks ignored with the reins back):
  a wider give, backing asked for on its own, a spur always heard, and the log of the hands. The reins
  showed through the horse and weren't the game's look: leather straps of the tack's texture, drawn
  with the body, lying on the neck.
- **Round 1.** The first build, checked with the emulated headset on `rustvr-horse`: getting on by the
  saddle, the reins, flicks and pulls through the gaits, the sticks, A reaching the game's Jump,
  getting off both ways, and a Chorrol guard arresting the player on someone else's horse.
