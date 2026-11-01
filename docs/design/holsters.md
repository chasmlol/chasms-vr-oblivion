# Holsters and inventory: research and design

A lot of the game done by reaching for a holster. One slot holds what heals, most healing first,
so food comes out of a pocket and is eaten at the mouth (potions drunk the same way); one holds
arrows; the others hold whatever the player sets; and a popup made of Oblivion's own UI customises
every slot: where it is, which categories it draws from, and the order it gives them out. Built:
the framework's `rustvr_protocol::inventory` and `rig::pockets`, the adapter's `inventory.rs` and
`slot_menu.rs`.

Legend: **(snippet)** a search engine's summary, not a page read in full; **[unverified]** prior
knowledge, not confirmed by a source. Player opinion is from Steam discussions, reviews and mod
pages (Reddit couldn't be read).

## 1. What players like and dislike

### Half-Life: Alyx

- **Liked:** the ammo backpack over the shoulder. It's placed from the head, so it's always where
  the hand expects, and seated players don't hit their chair with it the way belt rigs do; it
  steals no grabs from tables and the floor
  ([Steam](https://steamcommunity.com/app/546560/discussions/0/2143091644396700611/)). A small,
  contextual inventory (ammo over the shoulder, two wrist pockets) keeps attention on the world
  ([AMA](https://venturebeat.com/business/half-life-alyx-ama-everything-we-learned)). State shown
  on the thing itself: each round's light on the magazine
  ([Road to VR](https://www.roadtovr.com/these-details-make-half-life-alyx-unlike-any-other-vr-game-inside-xr-design/)).
- **Disliked:** partial magazines refused by the backpack
  ([Steam](https://steamcommunity.com/app/546560/discussions/0/2143091389818821897)); unclear
  where to let go (magazines dropped on the floor); one thing per wrist (players carried crates
  instead, [UploadVR](https://uploadvr.com/valve-half-life-alyx-crates/)); a syringe injected by
  accident on its way to the wrist pocket, because using and stowing overlap
  ([Steam](https://steamcommunity.com/app/546560/discussions/0/3178981911781099101)).

### Boneworks and Bonelab

- **Liked:** the back slots, the most consistent of any game for some
  ([Steam](https://steamcommunity.com/app/823500/discussions/0/3963662449068210745/)); Bonelab's
  body fitting by height and build (snippet).
- **Disliked:** holstering is hard: heavy guns lag the hand so they must be let go slowly, the
  pistol holster sits beside the magazines so the wrong thing comes out, tall players holster into
  their armpits; the slots follow the body model, so crouching moves them (same thread). Bonelab's
  arm pockets fit too little to be useful
  ([Steam](https://steamcommunity.com/app/1592190/discussions/0/3594464430443109431/)).

### Skyrim VR's mods

- **Liked:** VRIK's holsters and HIGGS's physical looting (over the shoulder into the inventory)
  are called essential
  ([Steam](https://steamcommunity.com/app/611670/discussions/0/3763353492968498409),
  [Road to VR](https://roadtovr.com/skyrim-vr-mod-higgsvr-hand-gravity-gloves/)). VRIK has 14
  holster points (calves, thighs, hips, stomach, chest, shoulders, arms); a holster is moved by
  holding the grip on it for two seconds until a stronger pulse, then it follows the hand until
  let go; each holster takes only the weapon types allowed in it (snippet,
  [Nexus](https://www.nexusmods.com/skyrimspecialedition/mods/23416)). Spell Wheel VR adds
  potions, food, poisons and arrows as categories of its wheel (snippet). HIGGS consumes potions,
  food and ingredients let go at the mouth (snippet,
  [Nexus](https://www.nexusmods.com/skyrimspecialedition/mods/43930)).
- **Disliked:** a grip too far from a VRIK holster does the grip's own action instead (a jump, a
  shout) (snippet).

### The Walking Dead: Saints & Sinners

- **Liked:** the backpack: small things go in, big ones clip to its outside
  ([Game Developer](https://gamedeveloper.com/design/intuitive-inventory-systems-in-vr-games)).
- **Disliked:** holsters too eager: resting the arms near the waist draws the gun, picking things
  off the floor grabs it, one finger is enough on Index controllers; the zones are big and can't be
  resized ([Steam](https://steamcommunity.com/app/916840/discussions/0/2958292387835606638/));
  the backpack pulled by accident mid-fight; the journal hard to grab without the pistol
  ([Steam](https://steamcommunity.com/app/916840/discussions/0/3001052356449998709)).

### Into the Radius

- **Liked:** ITR2 making harnesses, vests and backpacks fully configurable
  ([Mixed](https://mixed-news.com/en/into-the-radius-2-massive-update-brings-new-city-zone-and-custom-gear/)).
- **Disliked:** too few pouches, and none where the player wants them
  ([Steam](https://steamcommunity.com/app/1012790/discussions/5/3823032780423413079)); physical
  backpack storage that loses things or lets them float
  ([Steam](https://steamcommunity.com/app/1012790/discussions/1/5575984402935932660)); slow
  sorting; players ask for stacks with grab-to-take
  ([Steam](https://steamcommunity.com/app/2307350/discussions/1/4632611275488127478)).

### Others

- **H3VR**'s QuickBelt: many preset belts of small, medium and large slots (snippet); placed from
  the head, so leaning moves them; slots out of view are "hunted"
  ([Steam](https://steamcommunity.com/app/450540/discussions/0/1354868867716435233/)). Its mods
  lock the belt's turn while looking down, and give each slot a handle to drag it to a new place
  ([Thunderstore](https://thunderstore.io/c/h3vr/p/quaternion/Hand_Quickbelts/)).
- **Contractors**: a vest placed from a guess at the body's direction wobbles and breaks muscle
  memory ([Steam](https://steamcommunity.com/app/963930/discussions/0/5792223132453011488)).
- **Metro Awakening**: the chest gives whatever goes with what you hold (a magazine with the
  pistol, a vial with the syringe), praised for how little thought it takes; but nothing shows
  where the volumes are
  ([Road to VR](https://roadtovr.com/metro-awakening-vr-review-quest-pc-vr-psvr-2/)).
- **Arizona Sunshine 2**: guns bound to a holster, forced swaps, a chest pouch that sucks a
  magazine back in as it passes: called cumbersome
  ([Road to VR](https://www.roadtovr.com/arizona-sunshine-2-review-quest-pc-vr-psvr-2/)).
- **Asgard's Wrath 2**: a belt of six on the stick; a quick press drinks a potion, and fires by
  accident while moving ([Mixed](https://mixed-news.com/en/asgards-wrath-2-review/)).
- **Fallout 4 VR's Virtual Holsters**: seven zones moved and resized in a configuration mode; the
  zone shows and pulses as the hand comes near (snippet). **RE2 VR**: every zone calibrated by
  standing still for a countdown, with distance sliders (snippet).

## 2. How the good ones are built

- **Anchored to the head, not to a guessed body.** Alyx's backpack, Metro's shoulder and HIGGS's
  shoulders are all from the head; following the head's yaw but not its pitch, and not turning
  while the player looks down, keeps hips put. Guessing the body's direction (Contractors) and
  following the drawn body (Boneworks, crouching) are both complained of. The framework's
  `BodyAnchor` already does the good thing (yaw after the head with a 0.5 s lag, frozen while a
  hand is in or near a zone, so the slots don't drift). The first real-headset playtest
  found it hung from the eyes, which swing about 10 cm forward and down as the head looks down at a
  hip: the slots were out in front of the body. It now hangs from the eyes as they'd be looking
  level over the IK body's neck pivot, and a zone reaches 12 cm in toward the body's middle from
  its own, so a hand pressed against the body at a slot is in it.
- **HIGGS's numbers** ([source](https://github.com/adamhynek/higgs), `config.h`, `hand.cpp`;
  Skyrim units, our conversion): the mouth a sphere about 14 cm in front of and 13 cm under the
  head node, 16 cm round; the shoulder stash 25 cm out, 7 cm back, 10 cm down; both only for a
  hand moving slower than 2 m/s; a soft buzz while the thing is in the zone, a pulse when it's
  used; checked on letting go, no hold; never while the hand is held back from its controller;
  poisons can't be drunk.
- **Feedback before it counts**: a tick on entering, a buzz while in, a highlight; and the
  count and the next thing shown (Metro is faulted for invisible volumes).
- **Never refuse what the player puts back** (Alyx's partial magazines); never force a swap
  (Arizona Sunshine 2); auto-return optional.
- **Customisation in the world beats sliders**: VRIK's hold-and-drag, H3VR's handles, Fallout 4's
  configuration mode; then a stand-still calibration, presets, per-slot type filters, and files.
  Players asked for movable slots in Contractors, Into the Radius and H3VR.
- **Logical stacks, not physical bags** (Into the Radius's lost things, Alyx's one-thing wrists).

## 3. Eating and drinking

- **Vivecraft** ([`EatingTracker.java`](https://github.com/Vivecraft/VivecraftMod)): the mouth is
  the head; food counts within 25 cm; a drink only counts tipped (the controller's up pointing
  down); a bite every 2.1 s while it's held there; a pattern of pulses for food, another for drink;
  off in seated play.
- **HIGGS**: let go at the mouth (see above). **Pouring into the mouth** is too small a target:
  players pour potions on their face and neck to make it count. **The Forest**: hold to eat, with a
  progress cue added after players asked
  ([UploadVR](https://uploadvr.com/forest-update-improves-vr-support-community-suggestions/)).
  **Asgard's Wrath 2**: a button, fast and accidental. **Alyx**: using and stowing near the body
  overlap, so things get used by accident.
- **Job Simulator** [unverified]: food shows bites, and the burrito that exits the game asks again
  after the first bite.

## 4. Oblivion's inventory, in play

- **Rows and categories.** The inventory menu shows a row per stack, and a thing that doesn't
  stack with the rest (worn, damaged, poisoned, souled...) gets its own row. Its categories are
  weapons (with arrows), apparel, magic and alchemy (apparatus, ingredients, potions) and misc (the
  rest). A thing may be stolen, enchanted, poisoned, broken, a quest item or equipped.
- **Potions** use their own magnitude and duration (the healing ones 20, 35 and 50 health at once).
  **An ingredient eaten** applies only its first effect, food at `fWortcraftFatigueMag` (5) for a
  second. Most food restores **fatigue**, not health (33 of 36 food ingredients start with Restore
  Fatigue: apple, cheese, bread, ham). Beer, mead and wine are potions flagged food.
- **Using a thing** is the inventory's click. A potion or drink is drunk (the potion limit: "You
  cannot drink any more potions right now." and nothing used; the swallow sound); an ingredient is
  eaten (quest items refused; the chew sound). No message on success. A poison asks with a message
  box, then coats the weapon ("<poison> used."); it refuses with no weapon, one already poisoned,
  or a staff, with their own messages. A scroll (a book with an enchantment) is cast and one used.
- **The game's menu art**, for a menu of our own: `prefabs\generic_background.xml` (a framed
  popup), the inventory's row (`inv_item_template`: an icon, a name, the equipped marker
  `inv_equiped_marker_1.dds`), its scroll bar, `button_short.xml` / `button_long.xml`
  (`shared_button_*_off/on.dds`), the tabs' icons
  (`inv_icon_tab_{weapons,apparel,alchemy,misc}.dds`), items' own icons under `Menus\Icons\` (a potion `Clutter\Potions\IconPotion01.dds`, a scroll
  `Clutter\iconscroll1.dds`...), the click sounds (1 OK, 3 previous/next, 5 tabs), the fonts
  (1 Kingthings, 2 its shadowed face, 3 Tahoma).

## 5. The design

### What the player does

- **Reach and grip.** A hand gripping in a slot takes the slot's first thing, the game's own model
  in the fist: a thing with a rigged grip in the hand where it's made to be held, the fingers round
  its handle; anything else held just where it came out. A tick on entering a slot says it's
  there; a weaker double tick says it's empty.
- **Eat and drink at the mouth.** Food and drinks are brought to the mouth (a generous zone just
  under the nose, from the head) and held there: bites (or gulps) in the hand for half a second,
  then the game's own use: its effects, its sound, one gone from the inventory. Moved away before
  then, nothing happens. While the hand is held back from its controller, never.
- **Read a scroll** by holding it up in front of the eyes: it's cast where you look, by the game.
- **Poison a blade** by touching the bottle to the weapon in the other hand: the game's own
  poisoning, without its yes/no box (the touch is the yes), with its own refusals.
- **Put it back** by letting go of it in any slot. **Drop it** by letting go anywhere else: it
  falls or flies, and where it comes to rest the game drops it into the world (its own drop),
  where the hands can take it again like anything lying about.
- **Hold the grip in a slot** still (within 8 cm) for a second and a half to open the slot editor
  on that slot (the VRIK hold, without having to find a menu; the time is the runtime's
  `[holsters] edit_hold`, 0 turns it off); the Holsters button in the RustVR menu beside the pause
  menu opens it too.
- **The editor** (Oblivion's own popup): the slots on the left; for the chosen one, the game's
  categories with their icons (on or off; Earlier and Later move one in the slot's order), the
  order inside them (most healing first, most valuable...), what an empty slot does, its size and
  reach, the equipped first or not, Move (reach to where you want it on your body and squeeze),
  New, Remove and Done. B closes it too.

### The framework (any game)

- **The game's inventory as categories** (`protocol::inventory`, adapter -> runtime): the game's
  categories (its names) and its things (up to 256 stacks): category, count, flags (equipped,
  stolen, quest, enchanted, eaten, drunk, read, put on a weapon, an arrow for the game's bows),
  what it restores (health, magicka,
  fatigue: the game's own magnitude times duration), value, weight, power, when it came, its name
  and its model (`<game>/<key>`, drawn by the runtime, with its box and mass). Also the game's
  own default slots, used until the player makes their own.
- **Slots** (`rig::pockets`): a place on the body (in the body anchor's frame, scaled with the
  player's height), a reach (the zone's radius), a size (what fits back in: small, medium,
  large), categories in order, an order inside them (`Rule`: most healing, most magicka, most
  fatigue, most valuable, heaviest, lightest, newest, by name, most of, strongest, or the
  player's own: what they last put in comes out first), whether the equipped thing comes first,
  and what reaching into it empty does (a tick, nothing, or the rule's best from anything). A
  slot's first thing is the first of its categories' things (in the categories' order), by its
  rule, less what's already out of it in a hand.
- **Drawn things are the runtime's**: a copy of the thing (`OS_INVENTORY`, its row in
  `parent`), held as it comes out, its model drawn by the runtime with the game's lighting, its
  box its collision. The game sees it used (`shots` up by one for each use) or let go (free, at
  rest), and says when it has taken it back (used, or dropped into the world: the inventory's
  `taken` list), and the runtime then lets it go.
- **The mouth** (`rig::pockets::MOUTH`): a point 7.5 cm under and 5 cm in front of the eyes; a
  thing that eats or drinks held with its surface within 9 cm of it for 0.5 s (bites or gulps
  felt every 0.16 s on the way) is used. **Reading**: its middle 12 to 55 cm in front of the eyes
  and within 23 degrees (0.4 rad) of where they look, for 0.8 s. **On a weapon**: touching what
  the other hand holds (within 3 cm) for 0.3 s. The three times are the runtime's settings
  (`[holsters] mouth_time`, `read_time`, `coat_time`; the menu's Hands tab has the first two).
  Never while the hand moves faster than 1.2 m/s or is held back; a use the game hasn't taken in
  1.5 s was refused, and the thing must leave and come back before it's tried again.
- **A weapon, a torch or arrows** the game equips from a copy in a hand: the game's own arrives in
  that hand by its grip (`ObjectDesc::replaces`), the copy gone. An arrow of the kind on the bow
  comes out of whichever slot holds it as one of the bow's own arrows (`INV_ARROW`, from the bow's
  `OBJ_ARROW` template), ready to nock.
- **While the editor is open** every slot shows as a sphere of light the size of its reach
  (`compositor::Bubble`, over the drawn body: a hip's slot is partly inside it), the picked one
  bright; and the picked slot's first thing shows still at its place on the body
  (`SlotEdits::editing`), a long one laid across the body.
- **Saved per game** in the runtime's settings (`[[holsters.games.<game>]]`), from the editor and
  the hand's Move; the adapter's defaults until then.
- **The editor's link** (`protocol::inventory::BodySlots`, runtime -> adapter; `SlotEdits`,
  adapter -> runtime): the slots as they are and what each gives now, a request to open the
  editor (and on which slot), a slot being placed by hand; and back, the slots as edited and a
  slot to place.

### Oblivion

- **Categories**: food (ingredients and drinks flagged food), potions (not food, not poisons),
  poisons, ingredients, scrolls, arrows, weapons, soul gems, keys, books, sigil stones,
  apparatus, misc, torches (a light in the inventory is a carried one). Armour and clothing aren't
  drawn from slots. A torch is a weapon-like thing: any slot can hold it, and drawn from one it's
  equipped and the burning torch takes the copy's place (`crate::torch`).
- **The inventory** read with the game's own rows, every half a second in the world and at once
  after a use (published again every 2 s for a runtime that has come back); values from the game's
  own functions; what restores from the effects (a potion's
  restore health, magicka and fatigue magnitude times its duration (at least a second); an
  ingredient's first effect, at the game's food magnitude for food); the model from the form's
  path (`meshes\clutter` exported), its box measured on the model.
- **Defaults**: Healing (potions and food, most healing first) at the belt in front of the left
  hip, for the off hand while the sword hand fights; Arrows (the equipped ones first) over the
  right shoulder, where Oblivion wears its quiver; Scrolls and poisons in the right chest pouch;
  Food (most fatigue first) at the right hip. Mirrored for a left-handed player.
- **Uses**: the game's own equip, as the inventory's click does, for food, drinks, potions and
  ingredients (the game's own drink and eat, with its potion limit and its refusals); the game's
  own poisoning steps without the box; the player's scroll use; a weapon, a torch or arrows drawn
  are equipped (the same equip; arrows the whole stack, as the inventory's click does) and the
  game's own object takes the copy's place. The inventory's own sounds as a thing comes out and
  goes back (up and down).
- **Drops**: the game's own drop of one (the player's `RemoveItem`, with a place),
  put where the runtime's copy came to rest: moving slower than 6 cm/s for 0.3 s, or after 4 s
  whatever.
- **The editor** (`slot_menu.rs`): menu type 1050, ours (the game's menu factory hooked to make it,
  its destructor, click handling and id ours), read from an XML of our own
  (written beside the runtime's settings at start) made of the game's prefabs
  (`generic_background.xml`, `button_long.xml`, `button_xtralong.xml`), textures, icons and fonts,
  cut by `hud.rs` into three strips (the slots, the categories, the buttons) and the title, floated
  ahead of the player as the player's own menus are; the laser drives it as every menu, B or Done
  closes it. Each change goes to the runtime at once (`SlotEdits`).

### Archery

The arrows are a slot like any other, and there's no quiver of its own: with the body's slots on,
the rig's quiver zones (`zones::quiver_zones`, over the shoulder or at the hip) are used only by a
game with no slots (the test range). An arrow of the kind on the bow, gripped from whichever slot
it's kept in, comes out as one of the bow's own (its `OBJ_ARROW` template), ready to nock; one of
another kind comes out as itself and the game puts it on the bow (`crate::archery::replacing`).
The playtest notes are in `docs/playtest/holsters.md`.
