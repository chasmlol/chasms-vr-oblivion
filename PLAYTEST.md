# Riding: playtest

Save: rustvr-horse

`tools\play.ps1 riding` boots this build into `rustvr-horse`: Chorrol, just outside its stables, at
eleven in the morning. Your own horse (Chorrol's "My Chestnut Horse", placed and made yours) stands
about two metres behind you, a little to the left. The two horses in the paddock are the stable's.

What it's built from (players' likes and hates, how other VR games ride, Oblivion's engine facts) and
why it works the way it does: `docs/research/riding.md`.

## Try it, in this order

1. **Get on by the saddle.** Turn round, walk up to your horse's left side and close a hand on the
   saddle: the horn at its front, or the seat. The view fades, the game's own mount animation plays
   behind the fade, and you're in the saddle facing the way the horse faces (wherever you were
   facing in your room). The trigger while looking at the horse still works too (the game's
   Activate).
2. **Look down and round.** Your body sits in the saddle, legs astride, round the horse's barrel.
   Look about freely: your head never steers it. The horse's head and neck stay in front of you.
3. **Take the reins.** They're at the base of the neck, just in front of the saddle horn, either side
   of the mane: close a hand there (either hand, or both). A tick in the hand says you have one.
4. **Spur on.** A flick of the reins (snap a rein hand down a few centimetres, quickly, and stop) or
   the trigger of a hand on a rein: walk. Again: gallop. A pulse in the rein hands each time.
5. **Steer.** Pull one rein back towards you (left rein: left), or move both hands to one side (a
   neck rein), or open one rein out to its side. Full turn at 12 cm. The view turns with the horse,
   smoothly; a vignette closes in while it turns hard or gallops.
6. **Slow and stop.** Pull both reins back (7 cm): down a gait. Keep holding: it walks, stands, and
   after a second backs up. Let the pull go and it stands.
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
    while getting on and off, how far the reins move for a full turn and for slowing, the flick,
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
- The runtime's log: the vignette closes in at a gallop and in a turn, and opens again.

## Not checked

- Anything on the real headset: how it feels, how comfortable the turning is, how natural the rein
  distances are (12 cm for a full turn, 7 cm to slow, 15 cm to take a rein, the flick's 1.1 m/s).
- Haptics (the rein tick, the spur, the stride beat): the emulator doesn't record them.
- The vignette and the fade as pictures: the emulator's captures don't show the comfort overlay
  (on foot either); the log shows the vignette's changes.
- The view turning in steps (the setting), and head bob on horseback.
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
  9 cm either side of the mane. Oblivion's horses have a bridle but **no reins** (its bridle is the
  size of the head), so the reins in the hands aren't drawn.
- **No fighting from the saddle**: the game's rule. The hands still swing, nothing lands; casting
  is off too.
- **The ridden horse isn't one of the hands' creatures**, so the hands don't push against it.
- **A jumps** now on foot too: the player rig holds Jump rather than counting a press, so A never
  reached the game's Jump before (a one-line fix in the input bridge).

## Not done

- Reins drawn in the hands (the game has none: drawing a pair from the bit would be ours, with the
  game's tack texture; your call).
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
