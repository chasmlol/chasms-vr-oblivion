# Playtest: the player's body, measured against real people

Save: rustvr-base

## Playing it

From the repository root:

```bat
powershell -File tools\oblivion.ps1 -Save rustvr-base -Commands "SelectSpell Mirror"
```

It builds main, installs it beside the game and boots into the save with the Mirror lesser power
selected: cast it, or pull your gripped hands apart in front of your chest, to stand a mirror in
front of you ([mirror.md](mirror.md)). `powershell -File tools\oblivion.ps1 -Close` puts the game
away and the player's settings back. `rustvr-base` (a Nord in iron armour, in his house in Danwood)
is the developer's own test save, not in the repository; any save does, and
`-At <cell>`, saved as `rustvr-base`, makes one. The fast boot, the test saves and the game
lock are in [DEVELOPING.md](../DEVELOPING.md).

The framework's test range has the same body on its mannequin (framework/README.md): after a build,
`target\release\rustvr-play.exe` is the runtime with the range. Close the game first: they share
the runtime's link. Scripted poses on the emulated headset are in framework/docs/XRSIM.md; set
`RUSTVR_LINK` and `RUSTVR_DIR` to give a test run its own link and settings folder, apart from the
game's.

## What it does

The body the runtime draws for you (`framework/crates/ik`) was measured against real people's
motion capture, fed what a Quest would see of them (the eyes and both hands, with tracking noise),
and rebuilt where it was furthest from them. What the research found and why each change was made
is in `framework/docs/design/ik.md`.

- **It stands on the game's ground at the character's size.** The adapter reports how high the
  game's eye is over the player's feet (`link.rs`, logged once as "link: the game's eye is 1.60 m
  over the player's feet: the body stands on the game's ground"). While the game sneaks, the body
  keeps its standing size and crouches.
- **Your feet stay planted.** They step only when your balance needs it, a leg can't reach, or your
  hips have twisted 50 degrees from them. They never slide; steps lift, travel and come straight
  down. Walking about your room walks (the stick's gait, a stride at a time); when you stop, the
  other foot closes up.
- **Real proportions** (ANSUR II, 6,068 adults): shoulders 0.38 m apart for a 1.82 m person, the
  neck's base and the hip joints where a real body has them, and the head pivoting on the neck where
  the skull does (4.6 cm below and 8.7 cm behind the eyes: `BodyParams::head_pivot`).
- **The shoulder blades move.** As your arm rises past 30 degrees the shoulder joint rises (about
  4 cm at the horizontal), and it moves forward round your ribs reaching across your body and back
  reaching behind.
- **Bending right over doesn't spin the body**, and the knees don't pop while you stand.
- **Elbows don't fold in across your chest.**
- **The chest turns and bends after your head like a mass on a spring**, with no jolt as it starts.
- **The forearm rolls as a real one does:** none of the wrist's roll at the elbow, all of it at the
  wrist. On Oblivion's body, with its forearm twist bones, the skin at the elbow stays still.
- **A learned prior for the chest and elbows** (`ik::prior`, its weights `prior.bin`): a small
  network trained on CMU's motion capture (free for any use) guesses which way your chest and hips
  face and which way each elbow points, and the solver blends that into its own estimates. The
  hands still land exactly on the controllers.
- **Quest body tracking.** When the headset's runtime gives it (Virtual Desktop forwarding a Quest
  3's or Pro's), the elbows follow it and the chest faces as your tracked shoulders do.
- **Size.** You play at the character's height unless the Body tab's "Play at my real height" is
  on. Your height sizes the hands. Seated, the body is drawn standing.

### What was measured

Mean errors against the real body, every frame, with the Quest's tracking noise, before (main at
protocol v14) and after. CMU: 42 clips of 16 people the prior never trained on (reaching, picking
up, aiming, swinging, crouching, turning, walking, close handwork, chores). LaFAN1: 14 stretches of
40 s by 5 people from another lab, never trained or tuned on.

| | CMU before | CMU after | LaFAN1 before | LaFAN1 after |
|---|---|---|---|---|
| Elbow position (cm) | 10.9 | **8.9** | 9.6 | **8.5** |
| Elbow, the worst 5% of frames (cm) | 17.5 | **14.6** | 19.6 | **15.5** |
| Elbow swivel (°) | 23.7 | **18.4** | 19.2 | **13.3** |
| Upper arm direction (°) | 17.9 | **13.7** | 16.0 | **12.1** |
| Forearm direction (°) | 30.7 | **26.0** | 22.4 | **19.5** |
| Shoulder position (cm) | 10.7 | **8.7** | 11.2 | **8.7** |
| Shoulder width, ours minus real (cm) | −7.8 | **−1.7** | −3.9 | +2.4 |
| The shoulders' middle as the eyes see it (cm) | 9.2 | **7.8** | 8.6 | **7.6** |
| Chest orientation (°) | 15.8 | **14.4** | 30.7 | **20.3** |
| Chest yaw (°) | 12.2 | **9.3** | 23.3 | **16.2** |
| Chest forward lean (°) | 10.5 | 10.7 | 12.2 | 12.6 |
| Pelvis yaw (°) | 14.2 | **12.6** | 28.1 | **21.2** |
| Ankles (cm) | 19.6 | **18.5** | 42.7 | 43.1 |
| Knees (cm) | 15.1 | 15.0 | 30.3 | 33.1 |
| Planted feet sliding (cm/s) | 4.7 | **0.0** | 20.2 | **0.0** |
| Steps a minute (real people: 46 and 88) | 79 | **66** | 181 | **114** |
| Elbow acceleration, against the real one's | 2.3× | **1.9×** | 15.9× | **4.0×** |
| Knee acceleration, against the real one's | 6.0× | **3.4×** | 36× | **25×** |
| Elbow jerk, against the real one's | 2.9× | **2.3×** | 53× | **13×** |

LaFAN1's knees and ankles are further off than before only in its fights and obstacle runs, at
running speeds: the walking gait sets the feet a stride ahead there. In its walking they're nearer.

With the Quest's tracked shoulders and elbows fed in (the real ones with 1.5 cm of noise, an
estimate; how close a real Quest comes isn't known): CMU elbows 7.3 cm, swivel 13°, chest yaw
7.8°; LaFAN1 elbows 6.0 cm, swivel 6.3°, chest yaw 11.5°.

### The evaluation tool

The numbers come from `ikeval`, a Rust tool kept outside the repository with the motion capture it
reads (CMU's, for evaluation and training, and LaFAN1's, for evaluation only: CC BY-NC-ND),
converted to its own clip format. It runs the old solver and the new one on the same clips and
gives:

- the metrics above, per clip and per kind of motion, with `BodyParams` overridable per run, and a
  comparison of two runs;
- picture sheets: the real body as a blue ghost over ours, from the eyes looking down, from in front
  at an angle and from the side (aiming, drinking, bent over with a dustpan, looking round over the
  shoulder, reaching, LaFAN1's aiming);
- traces of a clip over time, its worst jolts and its sliding feet;
- the test range's two-handed hold through both solvers, its elbows against level;
- the rows the prior is trained from. Retrained, the weights replace
  `framework/crates/ik/src/prior.bin`.

An emulated-headset script kept with it stands, looks down, aims, works with the hands, reaches and
calls up the mirror in the test range, capturing each. Looking down, the belly and both legs show
below the hanging arms, each arm runs from the shoulder to its controller, and the mirror shows the
same pose each time. A two-handed hold with the grips tilted 45 degrees up, as a gun's sit in a
hand, has the support elbow 10 degrees below level and the trigger elbow level; the old body held
both higher at every tilt.

## What to try

1. **Your feet on the ground**, looking down and in the mirror, sitting or standing, and the mirror
   standing on the ground too. Sneak with the button: the body crouches rather than shrinking.
2. **Look down at yourself standing still, hands at your sides.** The chest and shoulders should be
   where yours are below your eyes: not sunk behind you, not pushed out in front. This is the one
   thing set from anthropometry rather than motion capture (the two datasets disagree by 6 cm). If
   it's off, `BodyParams::head_pivot` and `skeleton::NECK_LEAN` move it.
3. **Raise one arm out to the side, then straight up.** The shoulder should visibly rise with it
   from about halfway, and go forward when you reach across to the other shoulder. In the mirror the
   shoulder line tilts as one arm goes up.
4. **Look round over your shoulder with your hands still, then turn back.** The chest should barely
   follow and the feet shouldn't step. Turn your whole body further and the feet step round after
   it.
5. **Lean on a table, sway, shift your weight.** No steps.
6. **Walk a few steps in your room and stop.** A stride at a time, landing and staying, never
   sliding, closing up when you stop.
7. **Hold a bow or a two-handed weapon, aim, look along it, look down at your arms.** The elbows
   hang naturally and never fold across your chest. In a two-handed hold the chest turns slightly
   towards the forward hand.
8. **Pick something off the floor.** The body bends and comes back up without the shoulders
   jumping.
9. **Roll your wrist with the arm bent.** The forearm twists from the wrist; the elbow's skin stays
   still.
10. **Stand still in the mirror.** The knees should be still.
11. **With a Quest 3 and Virtual Desktop's body tracking on:** the chest turns with your real
    shoulders. Turn the Body tab's checkbox off and on to compare.

Seated, the body is drawn standing whatever you do, so 5, 6 and 8 need you on your feet.

## Settings

All in the Body tab (`[body]` and `[comfort]` in `settings.toml`):

| Setting | Default (range) |
|---|---|
| Elbows and chest from the headset's body tracking (Quest 3 / Pro) (`body.tracked_elbows`) | on |
| Show: Hands, Arms, Full body (`body.visibility`) | Full body |
| Your height (`body.height`; 0: measured as you stand) | 0 (0 to 2.2 m) |
| Arm span the body is built for (`body.arm_span`; 0: automatic, growing as your arms reach further) | 0 (0 to 2.4 m) |
| Hand size (`body.hand_size`; 1: a real hand for your height) | 1 (0.8 to 1.3) |
| Light the body with the game's lighting (`body.game_lighting`) | on |
| Seated mode (`comfort.seated`) | off |
| Play at my real height (`comfort.real_height`) | off |
| Height offset (`comfort.height_offset`) | 0 (−0.5 to 0.5 m) |

## Decisions

- **Measured against real people.** The body was rebuilt where motion capture showed it furthest
  from them, and judged on clips the prior never trained on, from two labs.
- **Proportions from anthropometry.** Motion capture can't settle where the chest sits under the
  eyes (its markers sit on top of the shoulders), so that follows ANSUR II.
- **A prior for what the head and hands leave open, the limbs solved exactly.** The network only
  guesses the chest's facing and each elbow's way out; the solver then puts the hands exactly on
  the controllers (MANIKIN's split).
- **Training data free for any use.** The prior trains on CMU only; LaFAN1's licence allows
  evaluation, not training.
- **The body stands on the game's ground.** The test range always reported its eye's height over
  the feet; Oblivion now does too.

## Not done / known issues

- **Steps are still a little too frequent** (66 a minute against real people's 46 across the CMU
  clips; turning in place most), and the legs as a whole are no nearer real people's than before
  (knees 15 cm off). Bending far over squats rather than folding at the hips.
- **The prior's weak spot:** holding a shouldered gun still and looking 60 degrees to one side
  turns the chest about a third of the way, where arms holding the gun would keep it stiller.
- **How close a real Quest's body tracking is** to the truth fed into the measurements isn't known.

## History

- **After the first look: the body stood on your room's floor.** Oblivion didn't tell the runtime
  how high its eye is over the player's feet, so the body stood on the real floor and was sized to
  the head's height over it: sitting, a child's size floating about 35 cm over the ground (the
  mirror with it); standing, sunk a few centimetres. Every build had this. The adapter now reports
  the eye's height, and a real crouch while sneaking no longer throws the view up.
- **The rebuild,** measured against motion capture (the table above), checked in the test range on
  the emulated headset, not in Oblivion or a real headset.
