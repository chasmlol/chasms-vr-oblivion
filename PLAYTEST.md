# Playtest: spells, remade after Skyrim VR

Save: rustvr-torches

Double-click `playtest.bat` in this worktree. It installs this worktree's build, boots straight into
the save, waits while you play, then closes the game and keeps the runtime's log in
`<builds>\spells\playtest-<n>`.

**The save.** Vilverin, at the top of the stairs down into the dark ruin. Flare is selected. The
character has:
* target, touch and self spells: Flare, Snowball, Spark, Absorb Health, Shocking Touch, Cold Touch,
  Burning Touch, Protect, Heal Minor Wounds;
* the Nord's power, Nordic Frost;
* the Staff of Conflagration in the inventory.

The console can't be typed in VR, so the `.bat` does the setting up for you:
* two zombies in front of you, one 2 m away for touch spells and one 10 m down the stairs for
  bolts, with the game's AI switched off so they stand still;
* **magicka 10000**, which comes back fast, so it's effectively unlimited;
* the six magic skills at 100, so even Master spells pass the game's own mastery check;
* **36 more of the game's own spells**, from every school, range and level. Among them are Fire
  Ball, Immolating Blast, Lightning Bolt, Frost Bolt, Ice Storm, Lightning Storm, Searing Grasp,
  Lightning Grasp, Fire and Lightning Shield, Heal Major Wounds, Convalescence, Shield, Open Easy
  Lock, Starlight, Illuminate, Paralyze, Frenzy, Chameleon, Shadow Shape, Summon Skeleton, Summon
  Dremora, Bound Dagger, Bound Sword, Turn Undead, Telekinesis and Soul Trap.

The launcher runs one console line every half second, so the extra spells arrive over the first
20 s or so after the save loads. That makes four pages on the wheel.

`range.bat` is the same on the framework's test range, with no game (see the end).

The design and its reasons are in `docs/design/spells.md`.

## How it works now

* **A spell in each hand**, as in Skyrim VR. The **left hand** starts with the game's selected
  spell (Flare). The **right hand** starts empty, so its trigger is still the game's Activate.
* **The spell wheel**, as in Spell Wheel VR. Hold the **right hand's B**: a ring of your spells
  appears ahead at chest height, the ones on quick keys first.
  * Point either hand at a spell and **pull that hand's trigger**: it goes into that hand.
  * Or **let go of B** while the right hand points at one: it goes into the right hand.
  * The **middle** empties the hand.
  * A spell lights up when you point anywhere along its slice from the middle, even a little past
    the ring's edge, and you feel a tick in that hand. The name in the middle is whatever either
    hand last pointed at.
  * There are twelve spells to a page. Point at the arrows at the foot and pull a trigger to turn
    the page.
* **Charge and release**, as in Skyrim VR with ISPVR's feel. **Hold the trigger**: the spell
  charges over the game's own casting time (self 0.37 s, touch 0.27 s, target 0.4 s). Its art
  appears in your palm and grows, its casting light fades in and its casting sound plays. After
  you let go the art stays in the palm for 30 s, then goes; a hand that's only holding a spell
  shows nothing. **The art is drawn apart from the game's picture** and laid over your hand where
  it's nearer: it sits in the hollow of your palm, just off the skin, your fingers cupped round it; turn
  your palm up and you see it, palm down and your hand hides it; a thumb in front of it hides it
  with a slightly soft edge, and both eyes agree where.
  (Not with the game's HDR on: then it stays in the game's picture, and your hand covers it.) In the hand you feel nothing
  at first, then pulses that speed up and strengthen sharply, then a spike when it's charged and a
  faint tick while you hold it. **Let go to cast**, and you feel a full burst. Let go before it's
  charged and nothing is cast; no magicka is spent.
* **Aiming with your hand.** A target spell leaves your palm where your hand points: **your arm and
  your wrist both steer it**. Bend your wrist down and it goes lower, up and it goes higher. It's
  the controller's own pointing ray (the menu laser's) turned 25° down, because a hand held out to
  cast rests with its wrist cocked up about that much (your casts at the zombies last time); raw,
  it went high. The aim is smoothed a touch, a little more for
  harder spells, to take out the jolt of the trigger. It's also helped gently onto a creature (10°
  up or down, 15° across).
* **Setting the aim yourself: the Spells tab.** Press the menu button for the pause menu; RustVR's
  own menu floats beside it. Open its **Spells** tab and tick **Show a line where a held spell
  would go**: an orange line runs from each palm holding a spell to where it would go, with a dot
  where it meets something (the help onto a creature isn't in the line). Hold your hand out at
  something with your wrist as it feels natural, and move the sliders until the line goes where
  you point:
  * **Down from where the controller points** (25°): more sends it lower, less higher.
  * **Out to the hand's side** (0°): plus turns each hand's spell out to its own side, minus
    across you.
  * **Towards the palm** (0°): palm casting; about 60 is the way some Skyrim VR lists cast.
  * **Starts this far ahead of the palm** (0 m): the bolt starts further out along its way.

  The line should move as you drag (if it doesn't show with the menu open, close the menu and it's
  there). Changes save at once; cast to try them. **Back to the defaults** puts them all back. When
  it feels right, tell me: I'll read what you set and make it everyone's default (each cast also
  writes the settings it went by to the runtime's log).

## What to try

1. **Look at your hands.** The left palm holds Flare's own flames, small and with no light. The
   right hand is empty.
2. **Fire a target spell (left hand, Flare).** Turn the left palm down the stairs towards the far
   zombie and hold the trigger. Watch the flames grow and their light fill the stairwell, listen
   for Flare's casting sound, and feel the ramp and the spike. Let go: the game's own Flare should
   leave your palm towards the zombie, with its own bolt art, light and sound, and hit it.
   * Try the palm at different angles, and at the near zombie.
   * Try letting go early (nothing), and casting again straight away. The hand waits out the rest
     of the game's cast first, about half a second.
3. **The wheel.** Hold the right B and look at the ring: each spell's own icon from the game, its
   name and cost in the middle when pointed at, a dot on the side of each hand that holds a spell,
   and spells you can't cast now dimmed.
   * Point the right hand at **Heal Minor Wounds** and let go of B: it's in your right hand.
   * Open it again, point the left hand at **Shocking Touch** and pull the left trigger: it's in
     your left hand.
4. **Self spell (right hand, Heal Minor Wounds).** Hold and let go anywhere: the game's healing
   on you, its glow over your hands and body.
5. **Touch spell (left hand, Shocking Touch).** Charge it, then put the hand on the near zombie:
   it goes off into it at once, without letting go. Also try charging and letting go with the hand
   just in front of the zombie: the game's own touch takes what's in reach along your palm.
6. **Activate with a spell in hand.** With spells in both hands, look at something you can use
   (a door, the zombie) and tap a trigger quickly: that's the game's Activate, not a cast.
7. **When the game says no.**
   * Cast **Nordic Frost** from the wheel twice. The second time nothing charges: you feel a long
     dull pulse, and get the game's own "only once per day" with the school's failure sound.
   * With 10000 magicka you won't run out. The "not enough Magicka" refusal works the same way, and
     you can feel it on the range with Inferno.
8. **A pick in the game's own menu.** Press the left B (Menu Mode), open Magic and click a spell:
   it goes into your **left** hand.
9. **The staff.** In the inventory, equip the Staff of Conflagration (it goes on your back) and
   take it in either hand. Pull the trigger: the enchantment's art charges at the tip. Let go: the
   game's own staff cast along the staff, using its charge.
10. **Holding something.** Take the torch or the sword in your left hand: the spell's art goes.
    Let go of it: the spell is back in that hand.
11. **Bound weapons.** Put **Bound Sword** (or Bound Dagger) in a hand from the wheel and cast it:
    the sword is made straight in that hand, by its grip, rather than at your hip. Squeeze the grip
    to take hold of it. Let go of it anywhere but a slot and it **goes back to your hip** (a bound
    bow to your back, a bound shield over your shoulder), since the game never drops a bound item.
    Draw it again from there as normal. When the spell runs out, or is dispelled, it's gone.
12. **The rest of the spells.** Use the wheel's pages to try the rest:
    * the big ones, whose aim should feel heavier (Immolating Blast, Lightning Storm);
    * area spells (Fire Ball, Ice Storm);
    * summons, which appear where you look;
    * Turn Undead on the zombies, Paralyze, Soul Trap, Open Easy Lock on a locked door, and the
      lights.
    * **Telekinesis by hand.** Cast Telekinesis (it's on you for a while). Point an open hand at
      something lying about within the spell's reach (about 9 m for the 30-foot one), grip to catch
      it, and it floats where your hand points. The trigger pulls it in, and a flick of the hand
      while letting go throws it. It moves things, not people, like the game's own. When the spell
      runs out, what you hold falls and the gravity gloves are back.
    * Chameleon and Shadow Shape (invisibility) don't make your own body see-through yet.

Things worth telling me about:
* Does it go where your hand points, with your wrist at its natural angle? If not, set it in the
  Spells tab (above) and tell me when it's right. Each cast writes to the runtime's log how far up
  it went, against the controller's ray and the head, and the settings it went by.
* Is the aim help too strong or too weak?
* Is the wheel in a good place and size?
* Can you feel the charge and the ready spike clearly?
* Do the spell's art and light in the palm look right? Palm up, is it over your palm; palm down,
  hidden by your hand? Does it keep up with your hand when you move it fast?

## The range (`range.bat`)

The framework's own test range, with no game. The left hand holds a fire bolt, and the training
dummies stand ahead and to the left. You have 1000 magicka, which comes back at 60 a second. The
right B opens the wheel, with eighteen spells on two pages:
* target spells: bolts of fire, frost and lightning, a fireball, an ice storm (Master, so its aim
  is heaviest), and a heal-other;
* touch spells: shocking, cold, searing, and paralyze;
* self spells: heal, shield, light, summon, bound sword;
* Lantern, a power;
* Inferno, which costs more than you'll ever have, so you can feel the refusal.

Bolts fly from the palm and knock the dummies, touches jolt them, a self spell flashes on you, and
the magicka is spent and comes back.

## Checked, and not

* **Checked:**
  * The aim: a hand held out level with its wrist at its rest (the controller 25° up) casts level,
    and its wrist bent 20° down from there casts 20° down (the runtime's tests). The headless
    `spell` scenario's hand held out at the dummy, the controller cocked 25° up: the bolt went 12°
    down and struck it, with the Spells tab's line on from the palm to the dummy; the outward turn
    goes to each hand's own side (the runtime's tests). Not seen: the Spells tab itself in the
    headset, and whether the line shows with the pause menu open.
  * A bound weapon on the range, on the emulated headset: Bound sword taken from the wheel's second
    page and cast, and the range's sword is made in that hand. Squeezed and let go in front, it went
    back to the left hip.
  * The framework part, on the range in the headless `spell` scenario: one charged bolt cast from
    a controller pointed at the dummy, the aim help puts it on the dummy, it strikes, and
    a tap isn't a cast.
  * The wheel, on the emulated headset: it opens, both hands pick from it, and casts go from either
    hand.
  * The unit tests for the charge curve, the palm aim, the cone and the wheel's slices.
  * The art laid over the hand, on the range in the headless `spell_art` scenario (the range draws
    its casting glow apart from its picture, as the game now does): a charged spell held up with
    the palm to the eye shows its glow over the palm; turned, the back of the hand hides it.
* **Not checked in the game**, since the game is never booted here:
  * the cast through the game's own checked cast;
  * the projectile carrying its casting light;
  * the touch pick into the touched creature;
  * the wheel's pick through the game's selection and its sound;
  * the failure message and sound;
  * the spell icons and font on the wheel;
  * the art drawn apart from the game's picture: the game's own draws of it sent to a picture of
    their own for each eye, its alpha kept, and the runtime laying that over the hand.

  This playtest is their first run.
