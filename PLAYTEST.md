# Playtest: hitboxes that match what you see

Save: rustvr-books

`playtest.bat` boots this build into **rustvr-books**: First Edition, the bookshop in the Imperial
City's Market District, with shelves, tables, chairs and a counter. `playtest-lines.bat` does the
same and also draws the hitboxes within reach as green lines over the game, so you can see what
your hands meet. When you're done, press a key in the playtest window (don't just close it): that
keeps the logs.

## What changed

Your hands, your arms and whatever you hold now meet each object's **drawn shape** instead of the
game's simplified collision: a chair is its seat, back, legs and rungs, not a box round it; a
shelf is its boards; a wall stops your hand where the wall is drawn. Each object near you gets its
own shape, worked out from its model in the game's files as you come near it.

- **What the game made solid stays solid, and what it didn't stays passable.** Only the shape
  changes. Leaves, flames, light shafts, cobwebs and water are never solid, and neither is
  anything the game gave no collision at all (most plants, hanging cloth).
- **The game itself is unchanged.** Your walking, the NPCs and things falling still use the game's
  own collision.
- **Still the game's shape:** the ground (it already matches), trees (the game grows them from a
  tree file, not a model: their trunks are the game's round shapes, made rounder), and anything
  whose drawn shape couldn't be worked out.

## What to try, in order

1. **Shelves.** Put a hand into a bookshelf, between two boards: it should go in up to the back
   panel and stop on the boards, not on an invisible front.
2. **Chairs and tables.** Reach between a chair's legs, under its seat, through the gap in its
   back. Slide a hand under a table and along its legs.
3. **Walls and the counter.** Run a flat hand along a wall, then along the counter's front and top:
   the hand should touch where it's drawn, not stop a few centimetres short or sink in.
4. **Small things.** Candlesticks, the fireplace, door frames, crates and barrels: hands go round
   their real outline.
5. **Outside.** Walk out of the shop's door into the Market District: barrels, carts, fences, the
   stone walls and the statues should all stop the hand where they're drawn.
6. **Things in your hand.** Hold a weapon or a book and press it against a shelf or a chair: it
   rests on the drawn shape too.

Say what feels wrong: a hand stopped by nothing you can see, a hand going into something it
shouldn't, or the game stuttering when you walk into a new area (the shapes are worked out as you
come near; that shouldn't be felt).

## What was checked, and what wasn't

Checked, not in the headset (the game was never started):

- the new reader goes through all of the game's models; the check tool
  (`oblivion-extract hitboxes`) worked out every model's hitbox and compared it with the game's
  own collision (the report is in `local_assets\oblivion\hitboxes\report.txt`);
- pictures of a sample of models, the drawn hitbox over the game's shape, looked at one by one.

Not checked: anything in the game itself. Where each shape lands in the world, how the hands feel
on them, and whether loading the shapes is felt as you move are all for this playtest. The log
says, for the first dozen objects, where each drawn shape landed next to the game's own shape (the
adapter's `rustvr-oblivion.log`, lines starting `collision:`).
