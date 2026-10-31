# Playtest: the framework's melee in Oblivion

Save: rustvr-gore

`playtest.bat` (in this worktree) puts away whatever's running, installs this build and boots
Oblivion into `rustvr-gore`: the house at Weye, three bandits round the room on about 20 health
who don't attack until hit (then they fight back), a steel longsword at your left hip, a Daedric
claymore, a steel battle axe and an iron dagger in your inventory, and you on 1000 health. Each
run's runtime log is kept in `<builds>\oblivion-melee\playtest-<n>`. `range.bat` starts the
framework's test range from this build (its melee areas: the rack, the bench of armoured dummies,
the arena's fighters, the block to climb), the same melee with the range's own creatures to compare.

This replaces Oblivion's older melee: the weapon in your hand is now held and judged as the
framework's melee holds and judges its weapons (`docs/MELEE.md`), and every hit is still the
game's own hit (its damage, skills, sneak attacks, enchantments, blocking, sounds, OnHit scripts,
stagger), as hard as the melee judged it. Our blood and dismemberment are unchanged: its judgement
just feeds them. How it fits together and every call made: `docs/design/oblivion-melee.md`.

## What's new, and what to compare against the old melee

**Holding** (any weapon; the inventory's Equip swaps them)

1. **Grips.** Take the sword by its handle with your hand turned any way: it turns into your hand at
   its proper grip, edge out from the knuckles. Come at it with your thumb towards the pommel for a
   reversed grip (the dagger especially). Grab it anywhere else (the blade) and it's held where you
   took it. *Old:* the grip snapped to one fixed hold.
2. **Weight.** The dagger keeps up with your hand; the Daedric claymore and the battle axe sag, trail
   a fast swing and swing on past where you stop (the game's weight: 0.06 kg each of its pounds, up
   to 8 kg). Their balance is where the model's metal is: an axe's or a mace's out at its head.
   *Old:* every weapon followed the hand exactly, weightless.
3. **Two hands and leverage.** Take the claymore or the axe in both hands: much steadier and faster
   than one, and the further apart your hands, the more it turns. *Old:* the second hand only aimed it.
4. **Sliding and half-swording.** Hold the trigger on a hand on the axe's haft and slide it along;
   on a sword, slide from the grip over the guard onto the blade, then grip there to half-sword.
5. **Throwing.** Let go mid-swing: it flies (the dagger point first, and it sticks). A thrown hit is
   the game's hit with that weapon (a thrown dagger hits three times as hard). Get it back by
   hand or with the gravity gloves.

**Hitting** (the bandits)

6. **How hard.** A lazy swing barely hurts, a full swing does the game's damage, a faster one more;
   a blow with the flat (a bad angle) is a light slap. Watch the health in the log: each hit says
   `combat: the player hits ... judged x<scale>`. *Old:* any clean hit was a whole hit, a fast one a
   power attack.
7. **Where.** By body part: the head x3, the throat and the trunk x1, a limb x0.5, a hand or a
   foot x0.25. *Old:* anywhere was the same.
8. **Armour.** Each part's armour is what the bandit wears there (a helmet the head, a cuirass the
   trunk and upper arms, gauntlets the forearms and hands, greaves the legs, boots the feet): the
   game's heavy armour is plate, its light armour leather. A sword's edge glances off
   plate unless it's fast (8 m/s), a mace still hurts through it, and a blow that does nothing
   scratches or scuffs. How much a blow that lands hurts is still the game's armour rating.
9. **Power attacks.** A blow that shoves the body by the push tables (a cut at 8 m/s, a mace at 7.5:
   the speeds the old melee used too) is the game's power attack in the way the blow went: a thrust
   or a blow driven ahead its forward one, a cut across its left or right one, a chop down its
   standing one. So its bonus by your skill and its perks are the game's: at Journeyman a sideways
   power attack may disarm, at Expert a backward one knock down, at Master a forward one paralyse.
   *Old:* a fast swing was a power attack that got no bonus (the game reads the attack's direction
   from its animation, which VR doesn't play).
10. **Stabs that stick.** Thrust a point in (or push it in slowly: a pressure stab). It stays, rocks
    while it's shallow, turns the body when you twist it, and comes out when you pull back past
    where it went in. Push on through a bandit and it comes out the far side. A stab deep in the head
    or the throat, or a cut throat, kills (deep wounds).
11. **Dismemberment and blood are ours, unchanged.** A killing cut (or a cut into a corpse) takes the
    limb or the head off at the joint the blade crossed, with the game's blood. Compare: the cut now
    has the swing's strength behind it, so a weak killing blow may hold where a strong one cuts.
12. **Knockouts.** A blunt blow hard on the head (a mace's, faster than the knockout speed for
    what the head wears) knocks the bandit out cold for a while: the game's own knockout, its
    Fatigue taken below nothing.

**Defence**

13. **Blocking and parrying.** Put the weapon or the shield in a bandit's swing: the game's block.
    Move into the blow for a parry.
14. **Disarming.** Hit a bandit's sword hand hard (push level 2; 3 in heavy gauntlets), or stab
    through its arm: it drops its weapon where its hand was, as the game's own disarm does it (it
    goes for a weapon again, or draws its next).

**Unarmed and grappling**

15. **Punches.** An empty hand punches, open or closed: the game's hand-to-hand hit
    (Hand to Hand's damage and its fatigue blow), even with a sword in the other hand. A shield's
    bash is one too.
16. **Grabbing and choking.** Grab a bandit (as before: the game's ragdoll); by the throat it chokes:
    its Fatigue drains until it's out, and now it hurts too: a tenth of its health a
    second squeezed fully, so a choke held long enough kills.

**The extras you chose**

17. **Stamina** is the game's Fatigue: hard swings spend it (a heavier weapon or a faster swing
    more), and a tired player's arms are weaker on the weapon. Watch the Fatigue bar fall as you
    flail and come back as you rest.
18. **Climbing.** With empty hands, grip a ledge or a wall's top edge and pull down: you climb, the
    game moving you by your arms with its gravity off while you hang (and no fall damage for the
    hang). Pull yourself up and over, then let go; let go to drop. (The climb jump is on the melee
    buttons, which stay the game's here.)

Slowed time, breakable things, the kick and telekinesis are off (the gravity gloves stay).

## Left alone

* **Magic and imbuing**: the spells chat (`<worktrees>\spells`) is rebuilding spellcasting; a hand's spell
  filling a blade isn't wired into Oblivion here.
* **Sneaking**: the stealth chat is reworking the physical crouch. Sneak attacks are still the
  game's own (its hit routine), as before.

## If something's off

The log (`<builds>\oblivion-melee\playtest-<n>\rustvr.log`, and the game's in its folder) says
each weapon's handles and blades as it's equipped (`weapons:`), each judged hit (`combat:`), each
hit the game made of it and how hard, each rule sent (`melee:`), and each cut (`gore:`).
