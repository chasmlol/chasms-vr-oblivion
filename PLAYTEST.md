# Playtest: the player's body, measured against real people

Save: rustvr-base

The body the runtime draws for you (`framework/crates/ik`) was measured against real people's
motion capture, fed what a Quest would see of them (the eyes and both hands, with tracking noise),
and rebuilt where it was furthest from them. What the research found and why each change was made
is in `framework/docs/research/ik.md`.

`playtest.bat` boots Oblivion into rustvr-base with the Mirror lesser power selected. Cast it (or
hold both grips with your hands together in front of your chest and pull them apart) to stand a
mirror in front of you. `range.bat` starts the test range instead.

## What changed

1. **Your feet stay planted.** They step only when your balance needs it, your leg can't reach, or
   your hips have twisted 50° from them. Before, they shuffled under you whenever you leaned,
   reached or looked round (35 steps a minute while just standing), and slid along the floor at the
   end of every step. Now they never slide, and steps lift, travel and come straight down.
2. **Real proportions** (from ANSUR II, 6,068 adults): shoulders 0.38 m apart for a 1.82 m person
   (were 0.31), the neck's base and the hip joints where a real body has them, and the head pivoting
   on the neck where the skull does (4.6 cm below and 8.7 cm behind the eyes).
3. **The shoulder blades move.** As your arm rises past 30°, the shoulder joint rises (about 4 cm at
   the horizontal), and it moves forward round your ribs reaching across your body and back reaching
   behind. Every VR body so far, ours included, held the shoulder still until the arm was at full
   stretch.
4. **Bending right over no longer spins the body.** Picking something off the floor, the shoulders
   and hips used to swap sides from one frame to the next.
5. **No knee pops.** Standing, the knees used to flick back and forth with every bob of your head.
6. **Elbows don't fold in across your chest** (a rifle's stock hand had its elbow in front of the
   sternum).
7. **The chest turns and bends after your head like a mass on a spring**, with no jolt as it
   starts.
8. **A learned prior for the chest and elbows.** A small network trained on CMU's motion capture
   (free for any use; 1,850 recordings) guesses which way your chest and hips face and which way
   each elbow points; the solver blends that into its own estimates. The hands still land exactly
   on the controllers.
9. **Quest body tracking for the chest.** When Virtual Desktop forwards the Quest's body tracking,
   the chest now faces as your tracked shoulders do (as well as the elbows following it). The
   Body tab's checkbox is now "Elbows and chest from the headset's body tracking".

## The numbers

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
| Pelvis yaw (°) | 14.2 | **12.6** | 28.1 | **21.4** |
| Ankles (cm) | 19.6 | **18.2** | 42.7 | **38.3** |
| Planted feet sliding (cm/s) | 4.7 | **0.0** | 20.2 | **0.0** |
| Steps a minute (real people: 46 and 88) | 79 | 74 | 181 | 170 |
| Elbow acceleration, against the real one's | 2.3× | **1.9×** | 15.9× | **3.9×** |
| Knee acceleration, against the real one's | 6.0× | **3.9×** | 36× | **25×** |
| Elbow jerk, against the real one's | 2.9× | **2.3×** | 53× | **13×** |

With the Quest's tracked shoulders and elbows fed in (the real ones with 1.5 cm of noise, an
estimate; how close a real Quest comes isn't known): CMU elbows 7.3 cm, swivel 13°, chest yaw
7.8°; LaFAN1 elbows 6.0 cm, swivel 6.3°, chest yaw 11.5°.

## Pictures

In `<builds>\ik\pictures\final\`. Each row is a moment. Columns: the real body as its own eyes
see it looking down; then for the old body (`base`) and the new one, each from the eyes looking
down, from in front at an angle and from the side. Ours is orange; the real body is a blue ghost
over it with a blue outline.

- `aim_cmu_79_96.png`: shooting a gun. The new arms sit inside the ghost's; the old elbows hung.
- `drink_cmu_13_07.png`: drinking from a bottle, looking at the hands.
- `bent_over_cmu_86_12.png`: bent right over with a dustpan, the frames where the old body's
  shoulders swapped sides. (Both still squat where the real person folds at the hips.)
- `look_around_cmu_77_01.png`: looking round over the shoulder: the real feet don't move.
- `reach_cmu_15_06.png`: leaning and reaching forward.
- `aim_lafan.png`: LaFAN1 aiming, never trained on.

## What to look at in the headset

1. **Look down at yourself standing still, hands at your sides.** The chest and shoulders should be
   where yours are below your eyes: not sunk behind you, not pushed out in front. This is the one
   thing measured from anthropometry rather than motion capture (the two datasets disagree by 6 cm).
   If it's off, `BodyParams::head_pivot` and `skeleton::NECK_LEAN` move it.
2. **Raise one arm out to the side, then straight up.** Your shoulder should visibly rise with it
   from about halfway, and go forward when you reach across to the other shoulder. In the mirror
   the shoulder line should tilt as one arm goes up.
3. **Look round over your shoulder with your hands still, then turn back.** Your chest should
   barely follow and your feet shouldn't step. Turn your whole body further and the feet should
   step round after it.
4. **Lean on a table, sway, shift your weight.** No steps.
5. **Walk a few steps in your room and stop.** The feet should step, land and stay, never sliding.
   (They still step more often than you do: see below.)
6. **Hold a bow or a two-handed weapon, then aim, look along it, look down at your arms.** The
   elbows should hang naturally and never fold across your chest. With a two-handed hold the chest
   turns slightly towards the forward hand, as a real stance does.
7. **Pick something off the floor.** The body should bend and come back up without the shoulders
   jumping.
8. **Stand still in the mirror.** Your knees should be still (they used to flick).
9. If you have a Quest 3 with Virtual Desktop's body tracking on: the chest should turn with your
   real shoulders. Turn the Body tab's "Elbows and chest from the headset's body tracking" off and
   on to compare.

## What I couldn't check

- **Nothing in the headset or in Oblivion** (the rule for the night). The test range's emulated
  headset was busy with the other two sessions' runs most of the night; one capture run overlapped
  another session's by mistake and gave nothing, and the drawn body from the range is not in these
  results.
- **How close a real Quest's body tracking is** to the truth fed into the measurements.
- **Absolute chest placement from the eyes**: motion capture can't settle it (its markers sit on top
  of the shoulders), so it follows ANSUR; look at it first.
- **Steps are still too frequent when walking and turning** (about twice real people's cadence in
  the walking clips), and the legs as a whole are no better than before (knees 15 cm off). Bending
  far over still squats rather than folding at the hips.
- **The prior's weak spot**: holding a shouldered gun still and looking 60° to one side turns the
  chest about a third of the way, where your arms holding the gun would keep it more still.

## The evaluation tool (outside the repository)

Everything is under `<builds>\ik` (about 5 GB; delete the folder to drop it):

- `eval\`: `ikeval`, a Rust tool against both the old solver (`baseline\`, main at protocol v14)
  and the worktree's. Build with
  `cargo build --release` there (target `<builds>\ik\evaltarget`). Commands:
  - `ikeval metrics new|base [--set cmu|lafan] [--body]`: the numbers above, per clip and per kind
    of motion, into `results\`; `IKEVAL_SET="param=value,..."` overrides `BodyParams`;
  - `ikeval compare a.csv b.csv`;
  - `ikeval sheet <clip> --times 1,2 --views 1,2,3`: the pictures;
  - `ikeval trace|frames|spikes|slides <clip> new`: what the body did over time, its worst jolts,
    its sliding feet;
  - `ikeval export` / `prior-export`: rows for fitting and for training the prior.
- `eval\tools\`: the converters (`cmu2clip.py`, `lafan2clip.py`, `convert_all_cmu.py`) and the
  prior's trainer (`train_prior.py`, torch: run with `python`).
  Retraining: `ikeval prior-export fit\prior_train.bin mocap\train_clips cmu`, then
  `train_prior.py fit\prior_train.bin fit\prior_val_cmu.bin,fit\prior_val_lafan.bin out.bin`, then
  copy `out.bin` to `framework\crates\ik\src\prior.bin`.
- `mocap\`: CMU (the evaluation clips, and all of CMU for training) and LaFAN1 (evaluation only:
  CC BY-NC-ND), converted to `.clip`.
- `research\`: the three long research notes `ik.md` is drawn from.
- `range\body.toml`: an emulated-headset script that stands, looks down, aims, works with the hands,
  reaches and calls up the mirror, capturing each.
