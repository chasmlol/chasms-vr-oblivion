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
* 200 more magicka;
* the Staff of Conflagration in the inventory.

The console can't be typed in VR, so the `.bat` puts two zombies in front of you, one 2 m away for
touch spells and one 10 m down the stairs for bolts. It also switches the game's AI off, so they
stand still. `range.bat` is the same on the framework's test range, with no game (see the end).

The design and its reasons are in `docs/research/spells.md`.

## How it works now

* **A spell in each hand**, as in Skyrim VR. The **left hand** starts with the game's selected
  spell (Flare). The **right hand** starts empty, so its trigger is still the game's Activate.
* **The spell wheel**, as in Spell Wheel VR. Hold the **right hand's B**: a ring of your spells
  appears ahead at chest height, the ones on quick keys first.
  * Point either hand at a spell and **pull that hand's trigger**: it goes into that hand.
  * Or **let go of B** while the right hand points at one: it goes into the right hand.
  * The **middle** empties the hand.
  * A spell lights up when you point anywhere along its slice from the middle, even past the
    ring's edge, and you feel a tick in that hand.
  * The arrows at the foot turn the page when there are more than twelve spells.
* **Charge and release**, as in Skyrim VR with ISPVR's feel. **Hold the trigger**: the spell
  charges over the game's own casting time (self 0.37 s, touch 0.27 s, target 0.4 s). Its art grows
  in the palm, its casting light fades in and its casting sound plays. In the hand you feel nothing
  at first, then pulses that speed up and strengthen sharply, then a spike when it's charged and a
  faint tick while you hold it. **Let go to cast**, and you feel a full burst. Let go before it's
  charged and nothing is cast; no magicka is spent.
* **Aiming from the palm**, as in Magic Improvements' palm casting. A target spell leaves the palm
  60° from your fingers towards the palm's face, so **turn your palm towards what you're casting
  at**. The aim is smoothed, more for harder spells, and helped gently onto a creature (10° up or
  down, 15° across).

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
   * Cast until your magicka runs out, then pull: nothing charges, you feel a long dull pulse, and
     the game's own "You don't have enough Magicka" with the school's failure sound.
   * Cast **Nordic Frost** from the wheel twice: the second time it should be refused with the
     game's "only once per day".
8. **A pick in the game's own menu.** Press the left B (Menu Mode), open Magic and click a spell:
   it goes into your **left** hand.
9. **The staff.** In the inventory, equip the Staff of Conflagration (it goes on your back) and
   take it in either hand. Pull the trigger: the enchantment's art charges at the tip. Let go: the
   game's own staff cast along the staff, using its charge.
10. **Holding something.** Take the torch or the sword in your left hand: the spell's art goes.
    Let go of it: the spell is back in that hand.

Things worth telling me about:
* Does the 60° palm angle feel natural, or should it be more finger-pointing or more palm-out?
  `palm_angle` is in the RustVR settings (`[magic]`).
* Is the aim help too strong or too weak?
* Is the wheel in a good place and size?
* Can you feel the charge and the ready spike clearly?
* Do the spell's art and light in the palm look right?

## The range (`range.bat`)

The framework's own test range, with no game. The left hand holds a fire bolt, and the training
dummies stand ahead and to the left. The right B opens the wheel, with these spells:
* Heal minor wounds (self);
* Frost bolt (a harder spell, so its aim is heavier);
* Shocking touch;
* Lantern (a power);
* Inferno, which costs more than you have, so you can feel the refusal.

Bolts fly from the palm and knock the dummies, touches jolt them, a heal flashes on you, and the
magicka is spent and comes back.

## Checked, and not

* **Checked:**
  * The framework part, on the range in the headless `spell` scenario: one charged bolt cast from
    a palm turned 60° off the line to the dummy, the aim help puts it on the dummy, it strikes, and
    a tap isn't a cast.
  * The wheel, on the emulated headset: it opens, both hands pick from it, and casts go from either
    hand.
  * The unit tests for the charge curve, the palm aim, the cone and the wheel's slices.
* **Not checked in the game**, since the game is never booted here:
  * the cast through the game's own checked cast;
  * the projectile carrying its casting light;
  * the touch pick into the touched creature;
  * the wheel's pick through the game's selection and its sound;
  * the failure message and sound;
  * the spell icons and font on the wheel.

  All of these follow the engine facts in `docs/research/spells-engine-notes.md`, read from the
  game's code. This playtest is their first run.
