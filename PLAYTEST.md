# Riding: playtest

Save: rustvr-horse

`tools\play.ps1 riding` boots this build into `rustvr-horse`: Chorrol, just outside its stables, at
eleven in the morning. Your own horse (Chorrol's "My Chestnut Horse", placed and made yours) stands
about two metres behind you, a little to the left. The two horses in the paddock are the stable's.

What it's built from (players' likes and hates, how other VR games ride, Oblivion's engine facts) and
why it works the way it does: `docs/research/riding.md`.

Run `playtest.bat` in this worktree (`<worktrees>\riding`): it installs this build and boots the save; press
a key in its window when you're done and it saves the ride's logs (into `<builds>\riding\logs`),
closes the game and puts your settings back.

## Round 8: subtle steering, and hands held close to the body

Built and unit-tested; not in the game yet. Your seventh ride's runtime log was lost: another
playtest started a few minutes after yours and replaced it (the game's log of what the horse was
told is in `<builds>\riding\playtest-7`). From now on `playtest.bat` saves both logs the moment
you press the key, before the game is closed.

- **"Way more subtle turns... make it so all the way out to the side is the fast turns."** A turn
  now grows as the square of how far the reins go past their give, and a full turn is 30 cm past
  it (was 10 cm, and half linear). A few centimetres turn it a degree or two a second; 10 cm, about
  7°/s; 15 cm, about 17°/s; 20 cm, about 30°/s; hands well out to a side, the game's full 67.5°/s.
  Changes are smoothed a little more (0.12 s).
- **"If I have my controller held a bit close to my body the steering doesn't really work
  properly."** Both hands to one side was measured from the horse's mane. But your body isn't always
  over the mane: the view centres on your head as it was when you got on, so if your head was off
  your body's middle then, your body sits a few centimetres to one side of the horse, and hands held
  close in front of it read as off-centre (a steady turn). Now it's measured from **your own
  middle**: where your head sits across the horse, followed slowly (over 10 s, so looking about
  doesn't move it). Hands centred in front of you go straight, held close or far out. One rein
  further back than the other is measured hand against hand, as before. If it still misbehaves
  close in, the log will show it: each hand across from your middle and ahead of the mane.

### Try this round

- Steer with tiny moves (a centimetre or two): a slow drift. Wider: harder. All the way out to a
  side: the fastest turn.
- Hold the controllers close to your body, centred in front of you: it should go straight. Move
  them a little to a side: a gentle turn.

## Round 7: hands centred on the horse go straight

Built and unit-tested; not in the game yet. Your sixth ride's logs are in
`<builds>\riding\playtest-6`.

- **"It just kept turning one direction whilst I had both my hands centred on the horse."** The
  reins measured steering from where each hand was when it took its rein (or when a spur last
  "gave" the reins, once with your hands 35 cm back). So hands you'd centred over the mane could
  read as 10 to 19 cm to one side: your log shows a full right turn held for two and a half
  seconds that way. Steering is now measured **against the horse itself**:
  - both hands to one side: from the **mane** (the middle between where a rider holds the
    reins); hands centred over it go straight, with 3 cm of give;
  - one rein back: **one hand against the other**; level hands go straight, with 2.5 cm of give;
  - a full turn 10 cm past that. Slowing is measured as before (it was right).

### Try this round

- Ride about, turn a lot, then put both hands centred over the mane and level: it should go
  straight, every time.
- Move both hands a little to one side of the mane: a gentle turn that way; further, harder.

## Round 6: a rider's grip on the reins

Built and unit-tested; not in the game yet.

- **"When I hold the grip on the reins... the pointy finger is lifted unless I hold the trigger as
  well."** A hand on a rein now closes into a rider's fist round it from the grip alone: all four
  fingers wrapped, the thumb pressed down on top. The trigger no longer matters to how it looks (a
  squeeze of it is still a spur).
- **"Rig the hands on the reins a bit nicer and realistic."** A rein now runs through the fist as a
  rider holds one: it comes up from the bit into the bottom of the fist at the little finger, and
  the loop between your hands comes out of the top, over the index finger under the thumb.

### Try this round

- Take the reins with just the grip: a closed fist, thumb on top. Look at where the rein goes in
  (under the little finger) and where the loop between your hands comes out (over the thumb).
- Turn your hands about: the rein should stay in the fist.

## Round 5: fine steering, a real walk, and spurs as hard as you do them

Built and unit-tested; not in the game yet. Your fourth ride's logs are in
`<builds>\riding\playtest-4`.

1. **"The turning is TERRIBLE... all or nothing, I have to fully reach over to turn... can you make
   it continuous so you can steer ever so slightly."** Three things made it so:
   - the reins did nothing for the first 5 cm and only reached a full turn at 17 cm;
   - the turn went to the game as a joystick, and Oblivion ramps a horse's turn up slowly (from
     nothing to full over half a second), so a small, short correction hardly turned it and a long
     one overshot;
   - to stop Oblivion halting the horse on a hard turn (round 3), the forward push was raised to
     match the turn, so turning sped the horse up.

   Now the reins steer from **1.5 cm** (a trembling hand's worth: drift under that is still taken
   up) and give a **full turn at 12 cm**, gently at first, smoothed against tremor. And the horse is
   **turned directly**: each game frame its heading is turned by the game's own heading turn (what
   Oblivion's horse turning itself calls), at exactly the share of the game's full turn (67.5° a
   second) the reins ask for, from a degree a second up, with no ramp and no effect on its speed. It
   turns while standing too.
2. **"The horse doesn't do the vanilla walk animation, just a very slowed sprint one."** I'd
   taken the game's "run speed" for the horse (4.1 m/s) as its gallop, but it gallops at 8.6 m/s
   (your log), so anything over 1.6 m/s was a gallop at 40-50% speed: that slowed sprint. Now
   slow speeds are **the game's walk** (up to 1.3 m/s, slower at a slower pace), and the gallop is
   never drawn slower than 60% of itself (5.2 m/s); in between it walks until half way, then
   gallops.
3. **"Doing a mini spur makes him walk slow as well."** A spur is now **as hard as you do it**: a
   flick by how fast your hand snaps down (a gentle flick to a sharp one), the trigger by how far
   you squeeze it (a light squeeze to a full one). A light spur from a stand walks it slowly; a
   harder one walks it faster or breaks into a gallop; the hardest surges to its full gallop. Each
   surge settles back to 70% of itself over about 8 s (so a light spur's walk slows a little, a
   gallop eases to a steady gallop), and spurs add up: keep spurring hard to stay flat out.

### Try this round

- **Steer** with the tiniest moves: one rein back a couple of centimetres, both hands a little to
  a side. The horse should drift gently that way; more, more; about 12 cm for its sharpest. Try it
  standing, walking and galloping: it should never stop or speed up to turn.
- From a stand, **a gentle flick or a light squeeze**: a slow walk. Another: a faster walk. A
  sharp flick: a gallop. Spur hard again and again: flat out.
- Pull the reins at a gallop: it slows through the gallop, then drops to a walk, then stands.
- Tell me how the steering feels (the 12 cm can go shorter or longer: the menu's "Rein travel for
  a full turn"), and how light a spur needs to be to walk.

## Round 4: a fluid speed, and reins that don't freak out near the horse

Built and unit-tested; not in the game yet. Your third ride's logs are in
`<builds>\riding\playtest-3`.

1. **"Doing the spur makes them go faster and then slowly fades down... to maintain full speed I
   have to keep spurring... lifting the reins should lower the speed... depending how much I pull...
   all fluid."** The reins no longer step between two gaits: they ask for a **speed**, any share of
   the horse's full gallop.
   - **A spur** (a flick, or a rein hand's trigger) **surges it to its full gallop** (it picks up
     over a second or two), and the surge **fades slowly to a cruise, a little over half its
     gallop** (most of the way in about 8 s, all of it in about 20). To hold it flat out, keep
     spurring every couple of seconds.
   - **Pull both reins back and it slows, as hard as you pull**: just past the dead zone, gently;
     15 cm back, hard (from a full gallop to a stand in about a second and a half). **Ease off and
     it keeps the speed you slowed it to**: a canter, a walk, a stand. A spur from there surges again
     and settles back at the cruise.
   - Backing up is unchanged: at a stand, ease the reins, pull them right back again and hold.
   - Let go of the reins: it keeps its speed 2.5 s, then eases to a stand over a few seconds.
   - How it's made of Oblivion's horse: Oblivion has a walk and a gallop, and a pace for each (the
     analog stick's). The speed you ask for is made of those with the game's own numbers for that
     horse (its walk and run speeds, from its Speed, as the game computes them), so Shadowmere is
     still faster than a paint horse. Up to about a quarter of its gallop it walks (at a slower
     walk if you ask for less); above, it gallops at a pace (a slowed gallop is the canter: never
     under 40% of the full gallop).
   - In the menu (Comfort, Riding): where a spur settles ("Spurred, it settles at", 0.55 of a
     gallop), how slowly the surge fades (8 s), and the pull for the hardest slowing (15 cm).
   - The stick rides as before (its push is the pace; click for a gallop).
2. **"The reins visual are still bugging out a lot... the moment they go near the horse they start
   freaking out and glitching around."** I tried this offline, sweeping a hand along, into and across
   the neck a millimetre at a time and measuring how far the reins moved: they jumped up to 90 cm in
   a single frame. They were rebuilt from scratch every frame, sagging through the neck and being
   pushed out of it, and near the neck's middle a rein would flip from one side to the other. Now each
   rein is **a rope**: a chain of points carried on from frame to frame, hanging under gravity
   (damped like leather, so it doesn't swing about), held at the bit and your hand, and kept out of
   the neck:
   - on the top of the neck, a point goes straight out from the neck's middle, so a rein reaching
     your other hand goes over the crest; lower down, straight out to the side it's on, which it only
     learns while it's clear of the neck, so it never flips across;
   - the neck's ends round off smoothly; near your hand the reins are kept out of the neck less and
     less, so a hand inside the horse keeps its rein in there (hidden), not fought over;
   - a hand further away than the rein reaches slides along it (as yours would), and the loop
     between your hands keeps its slack, instead of the rein snapping taut;
   - a rein point moves at most 1.5 m/s on its own, so a rein tipping over the crest slips over it.
   In the same sweeps no rein point now moves more than 13 mm in a frame, and still hands leave the
   reins still. They also hang with less slack than before (6%: a rider keeps light contact), and
   may sit 3 cm behind the horse's skin before the game's depth hides them, so they don't flicker
   against it.

### Try this round

- **Spur** from a stand: it should pick up to a full gallop, then ease back over ten or twenty
  seconds to a steady canter. **Spur every couple of seconds**: it should stay flat out.
- **Pull a little, then more**: it should slow gently, then harder; let go at a canter, a walk,
  or a stand, and it should keep it.
- **Hold the reins near the horse**: rest your hands on the withers, bring them together over the
  mane, move them along the neck, push a hand into it. The reins should follow calmly, lie along
  the neck, and never jump or flicker.
- Tell me the cruise speed (it can go up or down), how fast the surge fades, and how the pull feels.

## Round 3: from your second ride

Built and unit-tested; not in the game yet. Your second ride's logs are in
`<builds>\riding\playtest-2`.

1. **"The second time I tried to get back on... I was offset from the horse loads, floating in the
   sky to the side above it."** The view was anchored on the game's own camera. On the first mount
   of a session the game puts it at a seated rider's eye (0.85 m over the saddle), but after getting
   off and on again it stays at a standing player's eye on the horse's back (1.9 m over the saddle)
   and doesn't keep up with the horse as it moves: that's the floating, and the drifting to the
   side. Seated, the view is now anchored on the saddle itself: your eye 0.85 m over its seat (where
   the game's camera puts it when it gets it right), on the horse's spine as it moves. The game's
   log says each time how far the game's camera was off it.
2. **"The horse tended to stop whilst trying to turn while moving... turns like a tank."** That's
   Oblivion: it reads a rider's stick as a direction, and as soon as the turn is stronger than the
   forward push it stops treating it as forward at all, and the horse halts and turns on the spot.
   Every hard turn did it (your log: a gallop at 8.7 m/s down to 0.9 m/s each time the reins turned
   it past about two thirds). Now, while the horse is going, the forward push is always kept at
   least as strong as the turn (the game caps the speed, so it doesn't speed up): **it turns as
   hard as the game turns a horse (67.5° a second) and keeps its gait**. With the stick, pushing it
   to a side now keeps going too (it goes at how far you push it, in any direction but back); only
   pulling it back backs up. The right stick still turns on the spot at a stand.
3. **"The reins... can sometimes bug out slightly on the horse and glitch inside it."** Two causes:
   - the neck the reins lie on was fitted to the horse's model *through* its surface (about half of
     the model was outside it, by up to 4 cm). It's now fitted from outside, with the manes: every
     cross-section holds the model within 7 mm, and the reins keep 1 cm off it;
   - about a third of the game's pictures are moved on to where your head is by the time they're
     shown, and the horse moves with them, but the reins stayed where the game had last put the
     horse (13 cm off at a gallop). Their horse end now goes with the picture of the horse, and
     they stretch to reach your hands (as you said, a little stretch is fine).
4. Also from your log: taking the reins at the neck and drawing them in to your lap read as a 30 cm
   pull and stopped the horse straight away. **Taking up the reins** now: a rein follows your hand
   (and does nothing) until your hand comes to rest, up to 2.5 s; where it stops is where the rein
   is held from then on, and a soft tick says so. A trigger squeezed with the grip as you take a
   rein isn't a spur any more.

### Try this round

- Get on, ride, **get off and back on**, two or three times: the view should sit in the saddle each
  time, never above or beside the horse.
- **Turn hard at a gallop** with the reins (one rein right back, or both hands well to a side): it
  should keep galloping round the turn. Same with the stick pushed fully to a side.
- **Watch the reins** while riding, turning and galloping: they shouldn't go into the neck or head,
  or flicker into them.
- **Take the reins** at the neck and draw your hands in to where they're comfortable: nothing
  should happen until a soft tick, then the reins are live from there.
- If the turns still aren't sharp enough: 67.5°/s is the game's own horse (`iHorseTurnDegreesPerSecond`
  45, times the game's 1.5 for a full turn); say so and I'll look at raising it the game's way.

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
   the trigger of a hand on a rein, as hard as you want it to go: a light one walks it, the
   hardest surges it to its full gallop; each surge fades slowly to 70% of itself. Spur again to
   keep it going. A pulse in the rein hands each time.
5. **Steer.** Pull one rein back towards you (left rein: left), or move both hands to one side (a
   neck rein), or open one rein out to its side. From 1.5 cm, finely; full turn at 12 cm. The view
   turns with the horse, smoothly; a vignette closes in while it turns hard or gallops.
6. **Slow and stop.** Pull both reins back: it slows as hard as you pull (hardest at 15 cm), and
   where you let go it keeps that speed. To back up: at a stand, ease the reins, pull them right back
   again and hold 1.2 s. Let the pull go and it stands.
7. **Let go of the reins** at a gallop: it keeps going about 2.5 s, then eases to a stand over a
   few seconds.
8. **The sticks ride too.** Left stick ahead or to a side goes at its push and steers (click it:
   gallop), pulled back it backs up; let go and it stops. The right stick steers alone (without
   stopping it; at a stand, it turns on the spot).
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

- **A hand on a rein is a rider's fist** (round 6): `HoldShape::Rein` (all four fingers closed, the
  thumb pressed on top; chosen by eye), taken by a hand on a rein whatever its trigger does; the
  rein goes into the fist 3.5 cm down the grip's axis from the palm's middle (the little finger's
  side) and the bight comes out 3.5 cm up it (over the index, under the thumb).
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
- **Steering is measured against the rider and the horse** (rounds 7 and 8): the hands' middle
  across from the rider's middle (where the head sits across the horse, followed over 10 s, at
  most 15 cm from the mane) past 3 cm, one hand ahead of the other past 2.5 cm; the turn the square
  of the way past that, all of it 30 cm past. Measured from where the reins were taken (rounds 2 to
  6), "centred" depended on where the hands happened to be then, and a turn could stick; measured
  from the mane (round 7), a rider sitting off the horse's middle turned with hands held close.
- **Hands at rest do nothing** (round 2): a 5 cm dead zone, inside which a hand's rest follows it
  over 1.5 s (drift is taken up, a deliberate move of a few tenths of a second isn't); the turn
  comes in as half the share past the dead zone plus half its square, so small moves turn gently.
  A held pull slows only to a stand; backing is asked for on its own (a pull begun at a stand, held
  1.2 s), so it can't follow from slowing down. A spur always counts, and gives reins held back.
  The values are guesses for the headset (5, 17 and 10 cm): the menu has them all.
- **Speed is fluid, made of the game's gaits** (rounds 4 and 5): the reins ask for a share of the
  horse's full gallop; a spur surges it by as much as it's done (0.1 of a gallop for the lightest,
  all of it for the hardest: a flick's speed from 1.1 to 3.5 m/s, a trigger's squeeze from 30% to
  all of it), rising at 0.5 a second, fading over 8 s to 70% of the surge; a pull takes off up to
  0.7 a second, as hard as it's pulled, and the cruise follows it down. The adapter makes that
  speed of Oblivion's walk and gallop at a pace with the game's own walk speed for that horse
  (`GetWalkSpeed`) and its gallop, 2.1 times `GetRunSpeed` (4.1 m/s, where the
  horse gallops at 8.6 on the headset); the gallop is never asked for under 60% of its speed
  (slower, it looked like a slowed-down sprint), and below the speed half way from the walk to
  that, it walks. The turn rate is the game's own
  (`iHorseTurnDegreesPerSecond` 45, ramping at 80 per second, times 1.5 at full rein like the turn
  keys). Max speed is the game's full gallop for that horse (not raised).
- **The horse is turned straight** (round 5): the game takes a stick that points more to the side
  than ahead as a slide, which on a horse halts it and turns it on the spot (the player's update,
  the game's), and ramps a horse's turn up at 80°/s² (round 3 kept the forward axis at least the
  turn axis, which made turning speed it up). Now the stick's X is 0, and each game frame the
  horse's heading is turned by the game's own heading turn (`GetHeading` + an angle to
  `SetHeading`, which its horse turning calls) at the rider's share of `iHorseTurnDegreesPerSecond`
  (read live) times the turn keys' 1.5: continuous, no ramp, never stopping or speeding it.
- **The view sits on the saddle** (round 3): seated, its anchor is the rider's eye 0.85 m over the
  saddle's seat (on the horse's live spine), not the game's camera, which after a re-mount stays at
  a standing player's height on the horse's back. 0.85 m is where the game's camera was on every
  first mount of both rides; it's the one riding value read off the game in play rather than its
  data (nothing in its files says where a first-person rider's eye is).
- **Taking up the reins** (round 3): a rein follows its hand until the hand is still (under
  0.15 m/s for 0.25 s, at least 0.3 s after taking it; 2.5 s at most), so drawing the reins in from
  the neck doesn't pull them.
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
- **The reins are ropes** (round 4): a rein laid afresh every frame jumped as it went near the neck;
  as a rope carried from frame to frame (28 points a rein, 14 the bight; damped over 0.05 s, a
  point on the neck stops there; 12 passes a frame of lengths and the neck; sliding on its own at
  most 1.5 m/s) its shape only moves on from the last frame's. Measured offline by sweeping a hand
  along, into and across a neck shaped like the measured one, a millimetre a frame: at most 13 mm a
  frame for any rein point, where the laid curves jumped up to 90 cm.
- **The reins lie on the neck**: the exporter measures the horse's neck (its five neck bones, with
  the three manes) and head (four places along it) as cross-sections, an ellipse fitted to the top
  of each from outside the game's own models (round 3: a point left outside costs 200 times one
  inside, and the top may sit up to 4 units over a flat crest; every section holds the model to
  within 7 mm); the adapter sends them on the live bones; a rein that would pass through is put on
  the surface, 1 cm off it (below an ellipse's middle the neck is taken to go straight down). Each
  rein's length is its way from the bit along the neck to where it's held, plus 12% slack; one
  with further to go stretches. Their horse end is where each eye's picture shows the horse (a
  picture moved to the head's newer place moves the horse in it).
- **No fighting from the saddle**: the game's rule. The hands still swing, nothing lands; casting
  is off too.
- **The ridden horse isn't one of the hands' creatures**, so the hands don't push against it.
- **A jumps** now on foot too: the player rig holds Jump rather than counting a press, so A never
  reached the game's Jump before (a one-line fix in the input bridge).

## Not done

- Round 5 isn't checked in the game yet (built and unit-tested only). The horse turned on the spot
  turns without the game's turning animation (if it has one for horses); 2.1 for the gallop is
  measured on the chestnut horse only.
- Round 4 isn't checked in the game yet (built and unit-tested only). The slowed gallop as a canter
  depends on how Oblivion draws a gallop at a pace: if it looks wrong, the least gallop (40%) can go
  up.
- Round 3 isn't checked in the game yet (built and unit-tested only).
- The body's legs and seat are still where the game last put the horse, not where a moved picture
  shows it (as the reins were); at a gallop they can sit a few centimetres off the picture's horse.
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
