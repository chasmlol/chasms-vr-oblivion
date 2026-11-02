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

- **The world** (played 2026-11-01: it works): a chair is its seat, back, legs and rungs, not a box
  round it; a shelf is its boards; a wall stops your hand where the wall is drawn.
- **Things you pick up and move** (played: they work in the hand): plates, cups, mugs, bowls,
  baskets, buckets, books, bottles, everything lying about that you can take. Each one's collision
  is its drawn shape, in a handful of solid pieces: a basket is its wall and its handle with the
  inside open, a bowl or a bucket stays hollow, a tankard's handle has a gap your fingers go
  through.
- **Knocking things about** (new): a hand, or what it holds (a sword, a torch, a bottle), swept
  into something lying about knocks it: the game's own physics gives it the push. Before, only
  swinging chains and the like moved; things lying about only moved once picked up.
- **Fingers stop on things** (new): with your hand at something you could take, your fingers curl
  with the grip but stop on its surface instead of going through it; when you take hold, they
  close onto it from there, not from a fist.
- **Arrows through gaps** (new): arrows in flight (yours and anyone's) go through the gaps the
  game's shapes used to fill (between a chair's legs, through a fence, past a table's corner) and
  hit what's drawn: stuck in the drawn leg or board, broken on stone, with the game's own damage
  and sounds.
- **What's in your hands:** your weapon (a sword is its blade, guard, grip and pommel, not a box),
  your shield and your torch (its flame isn't solid).
- **Swinging things:** chain links, buckets on ropes and the like, each part as drawn.
- **Spells in flight** (new): a fireball or a bolt flies through the same gaps arrows do, and
  bursts on the drawn surface it meets (a chair's leg, a gate's bar), not on the box round it.
- **Arrows from further off** (new): drawn shapes now meet arrows and spells out to about 48 m from
  you, not just within 12 m, so an archer across a courtyard shoots through the gaps too.
- **Your hands behind see-through things** (new): your hands used to vanish behind anything the
  game draws see-through (the grime sheet before a cell gate's bars, fog cards in doorways, smoke,
  water) as if it were solid. Now only solid things hide them: a hand through a gate's bars stays
  in sight.
- **Portcullises** (new): the ones a lever raises (fort ruins, the Arena, Kvatch) are their drawn
  bars to your hands and to arrows, where the game has them as they rise. Before your hand went
  through them and arrows hit a flat slab.
- **Cut-off heads and limbs** (new): their collision is what's drawn of them (the head with its
  hair and helmet, the arm's upper arm, forearm and hand), not a box.
- **Statues and decorations** (new): things the game gave no collision at all (statues, mounted
  trophies, armour displays, candles on walls) are solid to your hands, as drawn. Before, your hand
  went straight through them.
- **Doors and gates** (new): a door's or a gate's moving part is its drawn shape too, open or shut:
  a cell gate is its bars, your hand goes between them, and arrows fly through the gaps. Before it
  was one flat slab.
- **Furniture and anything made of separate bars, legs or boards** (new): each separate piece that's
  a plain solid (a bar, a leg, a slat) is its exact shape now, so the gaps between them stay open.
- **Still as before:** the bow (its string's side is your drawing hand's), the arrow on the string,
  the mortar and pestle (their own pieces), and the ground and trees.
- **The game itself is unchanged.** Your walking, the NPCs, and things falling and resting where
  you let them go still use the game's own collision.

Each thing's shape is worked out from its model as it comes near (about a tenth of a second a
model, in the background, once for each model); until then it keeps its old shape.

## What to try, in order

1. **Knock things over.** Swat a cup or a plate off a table with an open hand, then with your
   sword or a bottle in your hand: it should fly the way you hit it, harder the faster you swing.
   A slow push should slide it. Things you can't pick up (a barrel, a crate) stay put as before.
2. **Fingers on a thing.** Hold your hand over a cup, a bottle or a book and slowly squeeze the
   grip: the fingers should curl until they touch it and stop there, not sink in. Take it: they
   close onto it without flicking through first.
3. **Arrows through gaps.** Shoot an arrow between a chair's legs, under a table, through a
   fence or a window grate: it should fly through. Shoot a chair leg or a table top: it should
   stick where it's drawn. Shoot a stone wall: it breaks as before.
4. **Shelves, chairs, tables, walls** (as before): reach into a bookshelf, between a chair's
   legs, under a table; run a hand along a wall and the counter.
5. **A basket or a bowl.** Pick one up and put your other hand inside it: it should go in, not
   stop at the top. Put a book or an apple in it and carry it.
6. **A mug or tankard.** Hook a finger through its handle; set it down and push it by its rim.
7. **Plates and cups on a table.** Slide your hand under a plate's edge, tip a cup by its lip.
8. **Your weapon.** Draw it and lay the blade on a table edge, slide it between a chair's rungs,
   rest it against a shelf: the blade, not a box round the sword, meets things.
9. **A torch** (if you have one): the handle and head are solid, the flame isn't.
10. **A cell gate** (`playtest-prison.bat`: the prison cell the game starts in, a bow, 500 arrows
    and a dagger): put your hand between the gate's bars (it should stay in sight, through them
    and out the other side), swing it by a bar, and shoot arrows through the gaps and into a bar.
    Put a hand into water, smoke or a fog card in a doorway: it should still show.
11. **A spell through a gap**: cast a fire bolt through a chair's legs or a gate's bars: it should
    fly through and burst behind; cast at a leg or a bar: it bursts there.
12. **A statue or a trophy**: touch a statue, a mounted head or an armour display: your hand stops on
    it, as drawn.
13. **A portcullis** (in a fort ruin): put your hand between its bars, shoot an arrow through them;
    pull the lever and watch it rise, still solid where it's drawn.
14. **A cut-off head or arm**: cut one off, pick it up, set it on a table and roll it: it should rest
    and roll on its drawn shape, not tip like a box.

Say what feels wrong: a thing your hand stops short of, a hand going into something it shouldn't,
a thing that jitters or flies off when you set it down or pick it up, a stutter as you walk into
a new area, an arrow that stops in mid-air, passes through something solid, or sticks a hand's
width off what it hit.

## What was checked, and what wasn't

Checked, not in the headset (the game was never started):

- the reader goes through all of the game's models; the check tool (`oblivion-extract hitboxes`)
  worked out every model's hitbox and compared it with the game's own collision, and split all
  2,313 models of things you can take into the pieces the hands meet (5 to 16 pieces for most,
  a tenth of a second on average, under a second for the slowest); the report is in
  `local_assets\oblivion\hitboxes\report.txt`;
- pictures of chairs, tables, shelves and walls, and of baskets, bowls, buckets, a tankard and a
  sword split into their pieces, looked at one by one; that bowls and buckets stay hollow was
  measured (open down the middle nearly as far as the drawn one);
- the runtime's side on its own: a bucket's mesh sent to it came back as its pieces in a tenth of a
  second and became the object's collision, its middle empty; a hand over a bottle, not gripping,
  gives the drawn fingers the bottle to stop on, and the grip closes in from there; a rifle swept
  through one of the game's hanging props asks the game to push it (the hands' and the runtime's
  tests all pass);
- the arrows: read from how the game's own arrow works (its flight is a Havok character proxy
  whose listener decides every impact), and built on that; nothing of it was seen working.

Not checked: anything in the game itself. Where each shape lands, how things feel in the hand and
on the table, whether knocks are as strong as they should be, whether arrows fly through and stick
right, and whether working out the shapes is felt as you move are all for this playtest. The
arrows log each drawn-shape hit (`arrows:` lines).
The log says, for the first dozen world objects, where each drawn shape landed next to the game's
own shape, and names each thing given its drawn shape (the adapter's `rustvr-oblivion.log`, lines
starting `collision:`).
