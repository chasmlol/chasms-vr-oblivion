# Playtest: hitboxes that match what you see

Save: rustvr-books

`playtest.bat` boots this build into **rustvr-books**: First Edition, the bookshop in the Imperial
City's Market District, with shelves, tables, chairs and a counter. `playtest-lines.bat` does the
same and also draws the world's hitboxes within reach as green lines over the game, so you can see
what your hands meet. When you're done, press a key in the playtest window (don't just close it):
that keeps the logs.

## What changed

Your hands, your arms and whatever you hold now meet each thing's **drawn shape** instead of the
game's simplified collision.

- **The world** (first round, played 2026-11-01: it works): a chair is its seat, back, legs and
  rungs, not a box round it; a shelf is its boards; a wall stops your hand where the wall is drawn.
- **Things you pick up and move** (new): plates, cups, mugs, bowls, baskets, buckets, books,
  bottles, everything lying about that you can take. Each one's collision is its drawn shape, in a
  handful of solid pieces: a basket is its wall and its handle with the inside open, a bowl or a
  bucket stays hollow, a tankard's handle has a gap your fingers go through. They used to be one
  solid blob round the game's shape.
- **What's in your hands** (new): your weapon (a sword is its blade, guard, grip and pommel, not a
  box), your shield and your torch (its flame isn't solid).
- **Swinging things** (new): chain links, buckets on ropes and the like, each part as drawn.
- **Still as before:** the bow (its string's side is your drawing hand's), arrows (thin sticks
  already), the mortar and pestle (their own pieces), and the ground and trees in the world.
- **The game itself is unchanged.** Your walking, the NPCs, things falling and resting where you
  let them go, and arrows and spells in flight still use the game's own collision.

Each thing's shape is worked out from its model as it comes near (a few hundredths of a second a
model, once for each model); until then it keeps its old shape.

## What to try, in order

1. **Shelves, chairs, tables, walls** (as before): reach into a bookshelf, between a chair's
   legs, under a table; run a hand along a wall and the counter.
2. **A basket or a bowl.** Pick one up and put your other hand inside it: it should go in, not
   stop at the top. Put a book or an apple in it and carry it.
3. **A mug or tankard.** Hook a finger through its handle; set it down and push it by its rim.
4. **Plates and cups on a table.** Slide your hand under a plate's edge, tip a cup by its lip.
5. **Your weapon.** Draw it and lay the blade on a table edge, slide it between a chair's rungs,
   rest it against a shelf: the blade, not a box round the sword, meets things.
6. **A torch** (if you have one): the handle and head are solid, the flame isn't.

Say what feels wrong: a thing your hand stops short of, a hand going into something it shouldn't,
a thing that jitters or flies off when you set it down or pick it up, or a stutter as you walk
into a new area.

## What was checked, and what wasn't

Checked, not in the headset (the game was never started):

- the reader goes through all of the game's models; the check tool (`oblivion-extract hitboxes`)
  worked out every model's hitbox and compared it with the game's own collision, and split all
  2,313 models of things you can take into the pieces the hands meet (5 to 16 pieces for most,
  50 ms on average, at most a third of a second); the report is in
  `local_assets\oblivion\hitboxes\report.txt`;
- pictures of chairs, tables, shelves and walls, and of baskets, bowls, buckets, a tankard and a
  sword split into their pieces, looked at one by one; that bowls and buckets stay hollow was
  measured (open down the middle nearly as far as the drawn one);
- the runtime's side on its own: a bucket's mesh sent to it came back as its pieces in a tenth of a
  second and became the object's collision, its middle empty.

Not checked: anything in the game itself. Where each shape lands, how things feel in the hand and
on the table, and whether working out the shapes is felt as you move are all for this playtest.
The log says, for the first dozen world objects, where each drawn shape landed next to the game's
own shape, and names each thing given its drawn shape (the adapter's `rustvr-oblivion.log`, lines
starting `collision:`).
