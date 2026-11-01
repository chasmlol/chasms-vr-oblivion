# Hitboxes: the world the hands meet

Research and design for what the player's hands, arms and what they hold meet in Oblivion's world:
walls, furniture, shelves, rocks, statues, the land. Inspiration only: nothing below is copied.

## What players love and hate

From reviews, forum threads and developer talks about VR games with physical hands.

**Loved**

- **Hands that stop where things are.** Pressing a palm flat on a table, running fingers along a
  wall, resting a forearm on a counter: reviewers single out the moments a surface is exactly where
  it's drawn as the ones that sell the world.
- **Reaching into things.** Pulling something out from under a table, off a back shelf or from
  between a chair's legs; putting a hand through a gap because it's there.
- **Soft things staying soft.** Brushing through leaves, grass and hanging cloth without being
  stopped.

**Hated**

- **Invisible walls.** A hand stopped a few centimetres short of a wall, or by nothing between a
  chair's legs, is the most common complaint about ports of flat games, whose collision was made for
  walking, not touching.
- **Thin things that aren't there.** A hand going straight through a post, a bar or a statue's arm
  because its collision was left out.
- **Snagging on air.** Being stopped by a plant's leaves, a cobweb, a light shaft or a banner.
- **Hitches** when a new area's collision is built.

**What it adds up to:** match the drawn surface; keep soft and see-through things passable; build
it without being felt.

## The game's own systems

- **Its collision.** Every object that never moves is a fixed Havok body in the current world: the
  land, buildings, rocks, furniture. Its shapes are the model's own collision, made by hand for
  walking and things falling: boxes, convex hulls, capsules, simplified triangle meshes. A chair is a
  box or two, a bookshelf a solid block, a curved wall a coarse polygon a hand's width off in places.
  Many drawn parts have none at all (iron struts, bars, posts, statues' limbs).
- **Its models.** Each object's model in the game's archives holds what's drawn: its triangles, the
  textures and how they're blended (opaque, cut out, see-through, added on as a glow), which nodes
  are hidden, which the model's animations move, and its collision.
- **Trees** are grown from SpeedTree files, not drawn from models: their trunks' collision is the
  tree file's own round shapes.

## The design

- **The hands meet what's drawn.** Each object near the player whose model has collision of its
  own gets its drawn shape instead (`oblivion_assets::hitbox`): every drawn triangle, less what
  isn't solid. Not solid: hidden and editor-only parts, sprites, glows, flames and light shafts,
  water and lava, the sky, mist and painted shadows, decals, hanging cloth, cobwebs, cut-out or
  see-through leaves, grass and flowers, the parts the model's animations move, and the parts of a
  cut-out card (a grate, a railing) that the game's collision doesn't cover. What draws only soft
  things is passable; what draws nothing (a collision stand-in for something drawn elsewhere) keeps
  the game's shape. Models the game gave no collision stay passable, as in the game.
- **What the hands take and hold, too.** The things lying about (plates, cups, baskets, books,
  everything the player can take), the swinging parts the game simulates (a chain's links, a
  bucket on a rope) and the weapon, shield and torch in the hands are given the runtime as their
  drawn shapes, by the same rules (`COL_OBJECT_MESH` in the framework's protocol, each in its
  object's own space). The runtime makes each one's collision its convex parts (an approximate
  convex decomposition on a coarse voxel grid, finer for a long thin thing, and a part more for
  anything it would leave out; about a tenth of a second a model on a thread of its own, once for
  each model): a basket's wall and handle, a bowl's wall round its hollow, a sword's blade, guard
  and grip. Until then, and for what draws nothing solid, a thing keeps its
  own shape. The bow keeps its own (its string's side is the drawing hand's), and so do arrows
  (thin enough as they are) and the mortars and pestles (their pieces are the runtime's).
- **The game itself is unchanged.** Walking, the NPCs and things falling keep the game's collision;
  only the runtime's physics (the hands, the arms, what they hold, drop and throw) gets the drawn
  shapes. Arrows and spells in flight are the game's own projectiles, flying in the game's
  physics against its collision.
- **Worked out where they're needed.** The adapter finds the references in the loaded cells whose
  3D holds the world's fixed bodies, nearest first, reads each model from the game's archives on a
  thread of its own (a few milliseconds a model), and sends each reference's drawn shape to the
  runtime as its own collision mesh, placed where the game draws it, a frame's share at a time. Its
  triangles take the material of the game's nearest (what a blade bites into). The land, trees and
  the objects not read yet stay the game's shape.
- **Checked over the whole game.** `oblivion-extract hitboxes` works out every model's hitbox and
  holds it up to the game's own collision: by kind of model, what was kept and left out and why, how
  far the drawn shape reaches past the game's and how much of the game's has nothing drawn near it
  (`local_assets/oblivion/hitboxes`). `oblivion-extract hitbox <model> <png>` draws one model's
  hitbox above its game collision and, below, the convex parts the runtime makes of it; for the
  things the hands take, the report counts their parts and how long each took.
- **Seeing them in the headset:** `RUSTVR_SHOW_COLLISION=1` draws the world's collision within reach
  as lines over the game's picture.
