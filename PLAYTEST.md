# Riding: playtest

Save: rustvr-horse

`tools\play.ps1 riding` boots this build into `rustvr-horse`: Chorrol, just outside its stables, at
eleven in the morning. Your own horse (Chorrol's "My Chestnut Horse", placed and made yours) stands
about two metres behind you, a little to the left. The two horses in the paddock are the stable's.

What it's built from (players' likes and hates, how other VR games ride, Oblivion's engine facts) and
why it works the way it does: `docs/research/riding.md`.

Run `playtest.bat` in this worktree (`<worktrees>\riding`): it installs this build and boots the save; press
a key in its window when you're done and it closes the game and puts your settings back.

## Round 2: from your first ride on the headset

What you said, what was wrong, and what changed. None of this has been in the game yet: it's
built, and its unit tests pass.

1. **"The flatscreen crosshair is still visible."** Gone, everywhere (not just on the horse). It's
   the dot in the middle of Oblivion's own reticle; the reticle's other image, the icon of what you
   can activate (a hand, a door; red when taking it is stealing), still shows when you look at
   something, since it's the game's only warning of a theft. The game's own Crosshair option isn't
   used: it hides that icon too.
2. **"The controls feel a bit off when holding the reins... I kinda just walk backwards and start
   moving sideways."** Your ride's log showed three causes:
   - pulling the reins while the horse stood **backed it up at once** (a bug: the time it stopped
     was never set, so "held a second at a stand" was always true). Taking the left rein, your hand
     only had to move 7 cm back and it backed;
   - the dead zone was **2.5 cm**: an ordinary drift of a resting hand turned the horse, and moving
     both hands sideways together (a neck rein) turned it too;
   - **flicks did nothing while the reins counted as pulled** (four of your flicks in a row), so the
     horse kept backing.

   Now:
   - **hands resting on the reins do nothing**: a 5 cm dead zone, and while a hand stays inside it
     its rest quietly follows it (over 1.5 s), so drift never adds up to a turn or a pull;
   - **turning** starts past the dead zone, gently, and is full at 17 cm;
   - **slowing** takes a deliberate pull of both reins, 10 cm back; held, it slows a gait at a time
     to a stand **and stays standing**;
   - **backing up is its own request**: at a stand, ease the reins forward, pull them back again
     and hold for 1.2 s. Ease off and it stands;
   - **a spur is always heard** (a flick, or the trigger of a rein hand), even with the reins held
     back: they're given (where your hands are becomes their rest).
   - The menu's Comfort tab, Riding: a new "Reins do nothing until moved" (the dead zone), with the
     full turn and the pull to slow.
   - **The log** now says, twice a second while a rein is held, how far each hand is from its rest
     (across and back), the pull, and the gait, pace and steering sent; when the reins start, change
     or stop turning the horse; pulls, eases and spurs; and in the game's log the axes and Run the
     horse itself was given.
3. **"The reins visual is also bad. I can see it through the horse... I want it to be a real
   Oblivion texture."** The reins are now **leather straps cut from the horse's own tack texture**
   (`creatures\horse\tack.dds`: the bridle's plain black cheek strap, with its normal map and the
   saddle's material), 16 mm wide and 4 mm thick, **drawn with your body** as anything you hold is:
   the game's depth hides them behind the horse, and the game's sun, ambient and lamps light them as
   they light your arms. They **lie on the neck** instead of passing through it: the horse's neck
   and head are measured from its own model as cross-sections, and a rein that would go through
   them goes round. Let go, each hangs along its side of the neck back to the withers; held, it
   rises along the neck to your fist; with both hands, the bight hangs between them; with one hand,
   both reins come up their own side of the neck to the withers and then to that hand.

### Try this round

- Get on, take both reins and **just sit**: hands still and relaxed for a while. Nothing should
  happen. Then drift your hands about a little: still nothing.
- **Steer**: pull one rein back slowly; the turn should come in gently and grow. Then try moving
  both hands to one side.
- **Spur on** with a flick, ride, then **slow** with a firm pull of both reins: down a gait; hold:
  it stops and stays stopped.
- **Back up** on purpose: at a stand, ease forward, pull back again and hold a second. Then try to
  back it up by accident: it shouldn't.
- With the reins held back after stopping, **flick**: it should walk on.
- **Look at the reins**: from the bit, along the neck, into your fists. Lean over to see the far
  side: the far rein should be hidden by the neck where the neck is in front of it. Ride past a
  lamp or a fire at night, or into shade: they should darken and light like your gloves. The
  leather should be the bridle's black.
- **Look at the middle of your view** on foot and on the horse: no crosshair dot. Look at a door or
  an item: its icon shows (red on someone else's things).
- Tell me how the dead zone, the turn and the pull feel: they're in the menu if you want to try
  other values, and the log will show what your hands did.

## Try it, in this order

1. **Get on by the saddle.** Turn round, walk up to your horse's left side and close a hand on the
   saddle: the horn at its front, or the seat. The view fades, the game's own mount animation plays
   behind the fade, and you're in the saddle facing the way the horse faces (wherever you were
   facing in your room). The trigger while looking at the horse still works too (the game's
   Activate).
2. **Look down and round.** Your body sits in the saddle, legs astride, round the horse's barrel.
   Look about freely: your head never steers it. The horse's head and neck stay in front of you.
3. **Take the reins.** They lie along the neck from the bit back to its base, just in front of the
   saddle horn, either side of the mane: close a hand there (either hand, or both). A tick in the
   hand says you have one; the rein runs from the bit along the neck to your fist.
4. **Spur on.** A flick of the reins (snap a rein hand down a few centimetres, quickly, and stop) or
   the trigger of a hand on a rein: walk. Again: gallop. A pulse in the rein hands each time.
5. **Steer.** Pull one rein back towards you (left rein: left), or move both hands to one side (a
   neck rein), or open one rein out to its side. Nothing within 5 cm; full turn at 17 cm. The view
   turns with the horse, smoothly; a vignette closes in while it turns hard or gallops.
6. **Slow and stop.** Pull both reins back (10 cm): down a gait. Keep holding: it walks, then
   stands, and stays standing. To back up: at a stand, ease the reins, pull them back again and hold
   1.2 s. Let the pull go and it stands.
7. **Let go of the reins** at a gallop: it keeps going about 2.5 s, then slows a gait every 1.5 s to
   a stand.
8. **The sticks ride too.** Left stick forward walks at its push (click it: gallop), back backs up,
   sideways steers; let go and it stops. The right stick steers alone (without stopping it).
9. **A** jumps (galloping: the game's horse jump).
10. **Get off.** Hold the trigger of a hand that isn't on a rein (0.6 s), or hold the saddle horn
    and lean well out over one side (35 cm). The view fades; the game's dismount.
11. **Steal one.** The paddock's horses are the stable's: getting on one is stealing it, the game's
    crime (a watching guard arrests you).
12. **Comfort settings**: the runtime's menu (menu button), Comfort tab, "Riding": the view turns
    with the horse smoothly or in steps (and the step), the riding vignette's strength, the fade
    while getting on and off, how far the reins move before they do anything, for a full turn and
    for slowing, the flick,
    how long it keeps its pace with the reins let go, the stride in the hands. The Comfort tab's
    "Camera bob / shake" brings the gait's bob back on horseback too.

## Checked in the game (the emulated headset, `rustvr-horse`, the horse restrained side-on in front)

- Getting on by gripping the saddle: the game's own mount (`TESObjectREFR::Activate` as the
  Activate control calls it), the state through getting on, seated, getting off; the view facing the
  horse's way once seated. Grips that missed say how far off they were in the runtime's log.
- The reins taken by both hands; two flicks: walk, then gallop; the trigger of a rein hand: walk; the
  left rein pulled: a left turn, the view turning with the horse (the neck stays in front);
  both reins pulled and held: gallop, walk, stand, backing; let go: standing.
- Reins let go at a gallop: gallop for about 3 s, then walk, then stand.
- The left stick walks the horse; the right stick turns it in place (about 0.97 rad/s), the view with
  it.
- A reaches the game's Jump while galloping (the key goes down; the jump itself I couldn't see in a
  still picture).
- Getting off by holding the left trigger, and by holding the pommel and leaning 37 cm out: the
  game's dismount.
- Looking down: legs astride round the horse, hands on the reins at the base of the neck. Looking
  left from the saddle doesn't steer.
- Someone else's horse (made the stable's for the test): got on, and a Chorrol guard arrested the
  player for it.
- The reins drawn: from the bit over the neck to each hand holding one, lying on the neck when let
  go, both hands' bight between them.
- The runtime's log: the vignette closes in at a gallop and in a turn, and opens again; the fade
  comes on as the game gets the rider on, and off.
- The view turning with the horse in steps (a build with steps as the default, not committed): 30°
  steps every 0.45 s while the horse turns at the game's rate, the neck swinging off centre and the
  view catching up.

## Not checked

- Anything on the real headset: how it feels, how comfortable the turning is, how natural the rein
  distances are (12 cm for a full turn, 7 cm to slow, 15 cm to take a rein, the flick's 1.1 m/s).
- Haptics (the rein tick, the spur, the stride beat): the emulator doesn't record them.
- The vignette and the fade as pictures: the emulator's captures don't show the comfort overlay
  (on foot either); the log shows when they come and go.
- The steps setting through the menu (it's checked by the default only), and head bob on horseback.
- A real horse jump, riding into water or a swimming horse, fast travel with the horse, getting off
  beside walls, left-handed play, the horse's own steal prompt (the game arrested instead).

## Decisions

- **Getting on is the game's activation**, not an animation of our own: the game checks whose horse
  it is, makes stealing a crime and plays its mount. The view fades meanwhile (on by default), since
  the game's animation moves the view as the player doesn't, and research says a camera taken over
  by an animation sickens.
- **Getting off is deliberate**: a held trigger on a free hand, or the pommel held while leaning out.
  Never the plain trigger: Skyrim VR players hate getting off by accident. While mounted the trigger
  doesn't reach the game's Activate (which would get off).
- **Steering by the reins in the rider's own frame**, measured from where each hand took its rein:
  pull back one, or move both sideways (a neck rein) or open one out; both are what riders do, and
  an equestrian's opening rein not working was a complaint about another game. One hand alone holds
  both reins (western style).
- **Hands at rest do nothing** (round 2): a 5 cm dead zone, inside which a hand's rest follows it
  over 1.5 s (drift is taken up, a deliberate move of a few tenths of a second isn't); the turn
  comes in as half the share past the dead zone plus half its square, so small moves turn gently.
  A held pull slows only to a stand; backing is asked for on its own (a pull begun at a stand, held
  1.2 s), so it can't follow from slowing down. A spur always counts, and gives reins held back.
  The values are guesses for the headset (5, 17 and 10 cm): the menu has them all.
- **Gaits are the game's**: walk and gallop (Oblivion has no trot). A spur steps up one, a pull
  steps down; the speeds and the turn rate are the game's own (`iHorseTurnDegreesPerSecond` 45,
  ramping at 80 per second, times 1.5 at full rein like the turn keys).
- **The head never steers.** The view's yaw follows the horse (smoothly by default, steps of 30° as
  an option); its pitch and roll never do.
- **No bob by default**: the view is anchored on the rider's eye held steady on the horse (eased
  over 0.35 s); the Comfort tab's camera bob brings the game's bob back.
- **The riding vignette** is its own setting (0.5 by default, like walking's): it closes in when
  the horse turns faster than about a radian a second or gallops.
- **The seat and legs are the saddle's and the horse's body's**, measured on the game's models: in
  first person Oblivion doesn't pose its rider at all (the rider's tracks in the horse's animations
  are only played on the third-person body, which it leaves standing). The legs go round the horse's
  barrel at a rider's proportions (knee 0.43 m below the seat, ankle 0.89 m).
- **The reins are where a rider holds them**: half way from the saddle horn to the base of the neck,
  9 cm either side of the mane. Oblivion's horses have a bridle and bit but **no reins** (its bridle
  is the size of the head), so the runtime draws them (round 2): straps from the bit's rings
  (measured at the corners of the horse's mouth on its own head model) to the hands, **drawn with
  the body** as held things are (the avatar's held models, a "bent" one: each cross-section of the
  strap its own bone), so they're in the game's depth and lit by its lights. **Their leather is the
  tack's**: the saddle's material (`Tack.dds` and its normal map, its shine; the bridle wears the
  same texture, but its model is an older NIF the extractor can't read) and the bridle's plain
  cheek strap in that texture (texels 79-103 by 236-466: chosen by eye, the one plain straight
  stretch of strap in it), mirrored along the rein so it has no seams. Rope (the game has
  `clutter\rope.dds`) was the other choice; the bridle's own leather matches what the reins hang
  from.
- **The reins lie on the neck**: the exporter measures the horse's neck (its five neck bones) and
  head (four places along it) as cross-sections, an ellipse fitted to the top of each from the
  game's own models; the adapter sends them on the live bones; a rein that would pass through is
  put on the surface (below an ellipse's middle the neck is taken to go straight down: the throat
  and chest are wider than the ellipse). Each rein's length is its way from the bit along the neck
  to where it's held, plus 12% slack.
- **No fighting from the saddle**: the game's rule. The hands still swing, nothing lands; casting
  is off too.
- **The ridden horse isn't one of the hands' creatures**, so the hands don't push against it.
- **A jumps** now on foot too: the player rig holds Jump rather than counting a press, so A never
  reached the game's Jump before (a one-line fix in the input bridge).

## Not done

- Round 2 isn't checked in the game yet (built and unit-tested only): the reins' look, the
  crosshair, and how the controls feel.
- The reins don't swing or bounce on their own: their shape follows the bit and the hands each
  frame (no rope physics), and they don't touch your body or the saddle, only the horse's neck and
  head.
- Shadowmere's tack (`tack_db.dds`) isn't used for its reins: every horse's reins are the ordinary
  tack's leather.
- The HUD is still this branch's (the reticle's piece at gaze); the menus branch reworks it, and the
  crosshair is hidden in the game itself, so its reticle piece will show only the activate icon too.
- The game's own riding pose for the body (the game doesn't animate it in first person); the legs
  don't swing with the gait (the seat does, with the horse's spine).
- Mounted combat (the game's rule kept).
- The protocol version is 12 (riding's region, `TrackingControl::carry_step`): a branch that also
  bumps it needs a renumber when merging.
- Two of the IK crate's tests (`tpose_gives_the_reference_frames`,
  `elbows_point_out_and_down_for_gun_poses`) already failed before riding; not touched.

## Branches

- `overnight/riding` in chasms-vr-oblivion (this worktree), and `overnight/riding` in
  chasms-vr-framework (its `framework/` submodule).
