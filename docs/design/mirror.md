# The mirror: research and design

The player's whole body in a mirror (their race, their face, their armour, lit as the game lights
them) in Oblivion's race menu and anywhere they call it up. This is what players like and dislike
in VR mirrors and character creators, how the reflection is drawn, Oblivion's heads and its race
menu, and the design built from them. It is for inspiration only:
nothing here is copied.

## What players say

**Why people stand at mirrors.** VRChat's "mirror dwellers" are the best-known case. A 2025 CHI
study of r/VRchat posts found people use mirrors to make up for what VR doesn't give them:

- they can't feel their body;
- the narrow field of view weakens their sense of the avatar as themselves.

In an IEEE VR 2022 study, users gathered at mirrors and said they felt relaxed there, but stayed
no longer than without one. People want to see themselves, and a mirror is how VR shows them.

**It has to move with you, exactly.**

- A mirror image that moves with you gives much more body ownership than one playing back other
  motion. People in the in-sync condition even dodged a falling object more often (González-Franco
  et al., IEEE VR 2010).
- Lag is noticed from about 60 ms, and clearly at 120-150 ms.
- Two engine bugs show how easily a reflection falls a frame behind:
  - Unreal's UE-181262: planar reflections trailed the headset because the capture ran before the
    late pose update.
  - Unity URP: `beginCameraRendering` read last frame's tracking.
- A VRChat update showed the head and shoulders in the mirror differently from the body, and
  players reported it straight away.

So the reflection must be drawn from this headset frame's poses, never the game's last.

**What breaks it.**

- **IK glitches and wrong height.** Meta's body guidance asks for:
  - a resting pose when tracking is lost;
  - the player's real height;
  - no simulated animation over tracked motion.
- **Eyes that drift off.** VRChat avatars whose eyes wander in the mirror are a long-running
  complaint. Meta's mirror sample gives its avatars gaze targets and natural glances away.
- **Legs that don't match.** Meta's plain-IK legs show only in the mirror; Bosworth called legs
  that don't match "disconcerting".

**Character creation in VR.**

- **Skyrim VR:**
  - Its race menu is the flat menu hung in front of you, with no stick zoom or rotation: you walk
    up to your own standing model, or grab it, to see your face.
  - After creation there is no mirror and no third person: `showracemenu` is the only way to see
    yourself again.
  - Its "VR Mirror" mod (about 4,000 downloads) lets you step back and walk round your image.
  - VRIK adds a selfie mode, but it is a third-person camera, not a reflection.
- **Fallout 4 VR:** it cut face editing at the start, leaving preset looks. Players minded: "not
  worth VR controls for something you never see" against "lazy". Its bathroom mirror is the one
  moment you see yourself.
- **What works well:**
  - Horizon puts the avatar editor button beside a mirror.
  - Meta's guidance: show the mirror before people enter a social space, so they "are not
    surprised by how they show up".
  - Rec Room's dorm mirror and Vacation Simulator's mirror selfie are mirror moments people
    remember.
- **Oblivion:** no prior art for its race menu in a mirror was found. OBVR (xOBSE plus DXVK) puts
  menus on the wrist and has no mirror.

**Where it stands.**

- A study of mirrors at 1, 2.5 and 4 m found no major effect on how people saw their avatar
  anywhere in that range. Judging body shape was harder at 4 m, and a farther mirror puts fewer
  pixels on you.
- VRChat's personal mirror can be anchored to the world, the head or the play space. Its
  complaints:
  - people grab it by accident;
  - its filters (other players, the world) are buried in the menu.
- No published standard size, height or tilt was found.
- Safety:
  - the mirror shouldn't fill the view;
  - it shouldn't stand inside a wall.

Sources (the research summary's): wiki.vrchat.com/wiki/Mirror,
creators.vrchat.com/worlds/components/vrc_mirrorreflection,
developers.meta.com/vr/documentation/unity/meta-avatars-best-practices,
dev.epicgames.com (Planar Reflections), issues.unrealengine.com/issue/UE-181262,
steamcommunity.com/app/611670/discussions/0/1696045708649378557 (Skyrim VR),
steamcommunity.com/app/611660/discussions/0/1500126447408130243 (Fallout 4 VR),
frontiersin.org/journals/virtual-reality/articles/10.3389/frvir.2022.1031093 (distance),
diposit.ub.edu/dspace/handle/2445/53296 (synchrony and ownership).

## How a mirror is drawn

- **A planar reflection is the scene drawn again from the eye reflected in the mirror's plane.**
  - Epic budgets about half a frame for one, and says even one per level "is often still too many".
  - Its draw calls don't fall with resolution.
  - VRChat's full mirror doubles the world's cost, and occlusion culling doesn't help inside it.
    So worlds offer cheaper mirrors:
    - an avatars-only mirror over the skybox;
    - mirrors that switch off when nobody stands at them.
- **Each eye needs its own reflection.** A single picture shared by both eyes looks painted on the
  glass. The usual fix is a render per eye (twice the cost again).
- **Clipping at the glass.** The reflected camera must clip at the mirror's plane, or what stands
  behind the mirror hides the reflection. Engines use an oblique near plane. A camera looking
  straight through the mirror, with its near plane on the glass and an off-axis frustum through
  the mirror's corners, does the same with an ordinary projection.
- **Screen-space reflections can't show a mirror.** They differ per eye, and they can't show what
  isn't on screen: your own face.

## Oblivion's heads and race menu

**The head is made of parts.** Each is the race's own model, with its texture:

| Index | Part |
|---|---|
| 0 | the head |
| 1, 2 | the ears (male, female) |
| 3 | the mouth |
| 4, 5 | the lower and upper teeth |
| 6 | the tongue |
| 7, 8 | the left and right eyes |

The race also has hair and the eyes' texture (without one the game uses
`Characters\Eyes\EyeDefault.dds`). The NPC also holds the hair's length (a morph on the hair) and
its colour.

**The models.**

- **The head is skinned.** `headhuman.nif` is skinned to `Bip01 Head`, `Bip01 Neck1`, both
  clavicles and `Bip01 Spine2`.
- **The other parts are rigid, attached to the head.**
  - The hair names its bone in a `Prn` string, `Bip01 Head`.
  - The ears, eyes, mouth, teeth and tongue are attached the same way.
  - Their vertices are in the model's own axes, about the head bone's place (the eyes at 5-7 units
    above it). So in the bind pose each sits at the head bone's position plus its own vertices.
- **The versions are older.** Many head parts are NIF 10.0.1.0 and 10.2.0.0, not 20.0.0.4:
  - up to 10.1.0.106 every block starts with a zero word;
  - the geometry has no group id or keep flags before 10.1;
  - the texture descriptors carry PS2 filter values;
  - the files carry an exported `NiDirectionalLight`.
- **The extractor reads them now.** All but four of `meshes\characters` read whole:
  - the first-person skeleton;
  - `emperor.nif`;
  - the two unused `imperial\*\hair01.nif`.

**FaceGen.**

- **The files.** Each head part has an `.egm` (geometry morphs, `FREGM002`), and the head and ears
  an `.egt` (texture morphs, `FREGT003`):
  - the human head's `.egm` has 50 symmetric and 30 asymmetric morphs over 1736 vertices: the
    NIF's 1275, then the 461 the `.tri`'s expressions use;
  - each morph is a float scale and an `int16` x, y, z per vertex;
  - the `.egt` has 50 symmetric morphs on a 256 x 256 image, each a scale and three `int8` planes.
- **A geometry morph** adds `coordinate · morph scale · delta` to every vertex. **The texture
  morphs** sum `coordinate · scale · delta` per texel, laid round mid-grey: the texture is
  `0.5 + Σ/510`.
- **The face's coordinates are the race's plus the NPC's**, in four sets: symmetric geometry (50),
  asymmetric geometry (30), symmetric texture (50), asymmetric texture (0).
- **How the head is shaded.** It's drawn by the skin shader's FaceGen pass ("facegen blend") with
  three textures:
  - `BaseMap`, the race's head texture (`headhuman.dds`, a 128² skin colour);
  - `FaceGenMap0`, the texture morphs' texture;
  - `FaceGenMap1`, an age map (`headhumanm30.dds`: grey, mean 0.25).

  The morphs move the skin colour, and the age map's detail (wrinkles, pores) is a 4× modulate.
  Then the skin lighting (its rim) as for the body.
- **The age** is FaceGen's age control from `FaceGen\si.ctl`. The control lists (56, 26 and 33
  named sliders) are followed by linear controls of 50 coefficients and an offset each. The first
  is age from the symmetric geometry: `age = Σ c·gs + 29.7151`.
- **The age map** is `Textures\<head texture>[M|F]<age>.dds`. The exact age is tried first, then
  down by tens (`headhumanm30.dds` for 34). Its normal map (`<age map>_n.dds`) replaces the head's.
- **Hair is drawn by its own shader**, tinted by the NPC's hair colour where the vertex colour's
  green masks it. Beside the diffuse it has:
  - a height map for parallax (`_hh`);
  - a layer map (`_hl`) blended by its alpha;
  - an anisotropic highlight looked up in `textures\effects\aniso.dds` by (N·L, N·H);
  - the skin's rim.

**The race menu** (opened by `showracemenu` or a new game) edits the player in place:

- race, sex, hair, eyes, hair colour and length;
- the face coordinates: its age slider sets FaceGen's age control, 15 to 65, and its complexion
  slider does the same, −2 to 2.

Opening it takes the helmet off (it's unequipped, and put back when it closes). Before this work
it was, in the headset, a panel on black in a void: the 3D preview it draws to the right of the
menu is the game's third-person camera, whose picture the runtime doesn't show in menus.

**Head-slot rules.** Armour and clothes cover biped slots: 0 head, 1 hair. The iron helmet covers
the hair slot alone, and the game draws no hair under what covers it.

**Another world render.** The adapter has the game render its world once per eye: the actors'
shadows once a frame, with the first eye; what the renderer sets up for a frame made again for
each render; the game's first-person arms hidden.

A third render with the camera put through the mirror renders the world from there with the game's
own lighting, fog, water and sky. Its back buffer and depth are copied out exactly as an eye's are.
The game's cost is mostly CPU (culling, draw calls), so a third render costs about half again an
eye pair. The frustum through the mirror's corners culls everything outside the glass, so in
practice it is less.

## The design

**The head (framework: `rustvr_asset::FaceMorphs`, `rustvr_avatar::facegen`).**

- **What the extractor writes.** It writes each head part attached to the head bone as the game
  attaches it, and its `.egm` as morphs in the cache's space (`morphs/<key>.rvf`). The head's and
  ears' `.egt` go into the same file.
- **What the adapter sends.**
  - The parts go into the avatar description like any other:
    - the race's head, ears and the rest;
    - the hair, unless something worn covers the hair slot;
    - the helmet.
  - The texture swaps put the race's textures and the chosen eyes on them.
  - A face description carries:
    - the coordinates (race plus NPC);
    - the age map, and the head texture it's laid over;
    - the hair colour.
- **What the runtime does.**
  - It moves every morphed part's vertices by the coordinates.
  - It makes the texture morphs' texture.
  - It shades the head with the game's FaceGen blend: `4 · age · (base + 2 · (morphs − 0.5))`,
    then the skin's lighting.
  - It draws the hair as the game's hair shader does: the tint where the vertex colour's green
    masks it, the layer map over the base, the rim, and each light's anisotropic highlight from
    the game's table. Its height-map parallax is left out.
  - The eyes get bones of their own at their centres (`LeftEye`, `RightEye`), which the rig turns
    to look at a point. In the mirror they look back at the wearer's reflected eyes, up to about
    30° off the head's forward: the eye contact that makes a reflection feel like you.
- **First person.** Everything on the head bone collapses onto the neck in first person, as the
  body's head does now, so the wearer never sees inside it.
- **Live changes.** A face change re-morphs the cached parts off the frame. The race menu's
  sliders show as they move.

**The mirror (framework: the runtime's `mirror`, the protocol's `MirrorView`).**

- **Where it stands.** A standing mirror in the world, kept in the game's world coordinates so it
  stays put while the player walks or turns (the runtime's `[mirror]` settings, `settings.rs`):
  - 0.95 m wide and 2.0 m tall by default, its foot 2 cm above the floor;
  - 1.2 m in front of the head when called. Your reflection is then 2.4 m away: the whole body in
    view, the face big enough to read.
  - Brought nearer when a wall is closer than that, and never nearer than 0.6 m.
  - Put away once the player is more than 25 m from it (walked away, or through a door).
- **How it's called up and put away.**
  - From the runtime's menu: the Body tab's Mirror section, "Stand a mirror in front of me" (then
    "Put the mirror away"), with sliders for its distance (0.6 to 3 m), width and height.
  - With a gesture: both grips held with empty hands close together (within 22 cm) in front of
    the chest, then pulled apart wide (75 cm) within 0.8 s, like opening curtains. The same
    gesture puts it away. It can be turned off in the same section.
  - With the game's Mirror spell (below).
- **What each eye shows.** Each headset frame, each eye sees the glass from where that eye is
  now: the camera reflected in the mirror's plane, looking straight through the glass with an
  off-axis frustum through its corners and its near plane on the glass.
- **What's drawn into it.** The runtime draws the player's whole body there:
  - from this frame's poses, so it moves exactly with them, with no lag;
  - with the head (not the first-person collapse);
  - lit by the game's lights as everywhere else (the specular from the reflected eye);
  - with what the hands hold.
- **The world behind the player.** The game renders it once more each frame, from the head's
  reflected camera with the same frustum construction. It comes back with its depth, which each
  eye's reflection moves into stereo with the runtime's warp (one game render, both eyes, the
  parallax right), and the body hides behind what's nearer. Without the game's picture the glass
  shows the body over a dark reflection; the setting "The game's world in the mirror" turns the
  extra render off for that. The game is asked for it only while some of the glass is within 77°
  of where the head looks, so turning away saves the extra render.
- **Hiding.** The glass itself is hidden behind the game's world where that is nearer, so a mirror
  half in a doorway shows half.

**The race menu (Oblivion).**

- **Opening it.** When `RaceSexMenu` opens:
  - the mirror stands in front of the player;
  - the world stays in the headset (the runtime keeps the stereo picture and the body for this
    menu);
  - the menu's panel stands beside the mirror, to its left (its edge at the glass's left edge,
    0.85 m tall, a little below the head and turned to face it), worked by the laser as the other
    menus are. Paused over it, the pause menu shows instead, as anywhere else.
  - The face can also be shaped by hand there ([facesculpt.md](facesculpt.md)).
- **Hiding the game's own player.** The game's third-person player, which the menu shows, is
  hidden from every render: the runtime draws the player.
- **Changes as they're made.** Every change in the menu changes the player's `TESNPC`, which the
  adapter reads every frame:
  - race, sex and hair are a new body;
  - the face sliders, eyes and hair colour are a new face description.
- **Closing it.** When the menu closes, the mirror goes too, unless the player had called it up.

**The mirror spell (Oblivion: the adapter's `mirror_spell`).** Oblivion has no mirror of its own,
as decoration or as an item (no model, no record: its only "mirrors" are a shield, a book and a
flipped arch), so the mirror comes into the game the way the game brings things in by magic.

- **The spell.** "Mirror", a lesser power every player has: cast as often as they like, for no
  magicka.
  - It's made as the spellmaking altar makes a player's spells: a created `SpellItem` (a 0xFF form
    id) in the game's created objects, which a save keeps. A game without it (a new one, or a save
    from before) is given it with the game's `AddSpell`.
  - Its one effect is a script effect called "Mirror" on the player, with no script (the game runs
    none), in Conjuration, with Bound Shield's look (`BASH`): a conjured thing's light in
    the hand, its sounds, its icon.
- **What it does.** Cast while looking at the mirror (within about 35° and 6 m), it sends it away;
  cast anywhere else, it summons one in front, where the gesture would stand it
  (`EV_MIRROR_CAST`).
- **Its arrival** is a summoned creature's. The adapter plays Summon Dremora's effect
  model (`summonDaedra.nif`, every Daedra's) as one of the game's temporary effects at the
  mirror's foot, facing the player, in its `SpecialIdle_SummonEffect` sequence. The glass appears
  at that sequence's "Hit" key, where the game brings a summoned creature in (`EV_MIRROR_HIT`),
  with the spell's hit sound at the glass. Sent away, it goes the same way. The game renders the
  mirror's world while it's being summoned, so the glass has it the moment it appears.

## What building it found

- **The race menu shows the player in third person.**
  - It puts the game's camera 0.62 m in front of the face, looking back.
  - The adapter used to switch any third-person view back to first person. Under the race menu
    that left the game without the player's third-person body when it rebuilt it for a new race or
    sex, and it crashed on the missing biped. The adapter now leaves the race
    menu's third person and the player's facing alone.
  - The tracking keeps the eye from before the menu opened, so the world doesn't jump 0.62 m as it
    opens and closes.
- **The menu's place on the picture** is the box of its tiles that draw something (images and
  text). The race menu has an invisible rect, `race_player_grab`, for dragging the player round in
  the flat game, and it reaches over the right half of the screen.
- **Live changes.** The face's coordinates change every frame while a slider is dragged. Putting
  the body together again from parts already loaded takes 2 to 13 ms, and putting it on the GPU
  2 to 4 ms once the avatar keeps its renderer (rebuilding it compiled its shaders, a second a
  time).
- **Cost.** The mirror's world render, a third world render a frame, kept the game at 140 to 300
  frames a second in the test saves' interiors (at 2064 x 2208 an eye, on the developer's PC).
