# Playtest: talking to people out loud

Save: rustvr-voice

Look at someone a few steps away and talk. They turn to you and answer in their own voice, cloned
from the game's own lines for them, their lips moving with the words and the voice coming from
them.

This is the voice plugin, `rustvr-voice`: an optional plugin for power users who run its servers
(speech to text, the cloned voices, an LLM). It isn't in the default build, and
`tools\oblivion.ps1` doesn't build it. The simple voice commands (Windows' own speech recognition,
below) need none of it.

The research and the design: `docs/research/voice.md` (and `docs/research/npcs.md` for the looks).

## Setting the plugin up (once)

1. **Build the plugin** into the same folder as the runtime, so `rustvr-voice.exe` sits beside
   `rustvr-play.exe` (the runtime starts it from beside its own exe). From the repo root, with the
   target `tools\oblivion.ps1` builds into (`target`, or your `CARGO_TARGET_DIR`):

   ```bat
   cargo build --release --manifest-path framework\Cargo.toml --target-dir target -p rustvr-voice
   ```

   It builds in m8 (`chasmlol/m8`, branch `rustvr`, a git dependency of `framework/crates/voice`).
   Build it again after pulling main: a plugin built against an older protocol waits on an older
   mapping and never hears the game.
2. **The servers.** The plugin calls three, each a server of its own:
   - speech to text: m8's `stt-server` (built with m8) with the Parakeet TDT 0.6B v2 int8 model,
     which also gives each word's time (the mouth needs it);
   - the cloned voices: Breeze TTS 2's server (Breeze-TTS-2.cpp's `breeze-server`, or the Python
     one m8's `start-breeze.ps1` runs);
   - an LLM with an OpenAI-style `/v1/chat/completions`: a hosted API, or a local server.

   m8's README says where to get the models (Parakeet's, and Silero VAD's `silero_vad.onnx`).
3. **Where they are.** Copy `tools\dev.example.toml` to `%LOCALAPPDATA%\RustVR\dev.toml` and fill
   in its `[voice]` lines: `stt_server` and `stt_model` (the server and its model folder),
   `tts_server`, `tts_dir` and `tts_model` (Breeze's server, the folder it runs in, its model), and
   optionally `warm_up` (a clip of your voice, its text in a `.txt` beside it, that warms both
   servers up before the game's first line).
4. **`voice.toml`.** The plugin's first start writes `%LOCALAPPDATA%\RustVR\voice.toml` with every
   setting and what it does. Set its `[llm]`: the `url` and `model` (it starts as OpenAI's
   `gpt-4o-mini`; DeepSeek also wants `thinking = { type = "disabled" }` under `[llm.extra]`), and
   the servers' URLs if yours differ. Put Silero VAD's model where `[models] silero_vad` says
   (`silero_vad.onnx`, beside `voice.toml`, unless you give another path).
5. **The key** is yours: in `voice.toml`'s `api_key`, or in the environment variable its
   `api_key_env` names (`OPENAI_API_KEY` at first; `setx <name> <your key>`, then a new Explorer
   window). Never put a key in a repository. `clips\voice-servers.ps1` stops and says so while
   neither is set.

## Playing it

From the repo root, the servers first, then the game:

```bat
powershell -File clips\voice-servers.ps1
powershell -File tools\oblivion.ps1 -Save rustvr-voice -Commands "player.placeatme 000AED40 1 150 0"
```

`voice-servers.ps1` starts what isn't up (on the ports `voice.toml` calls them on: speech to text
on 8083 and Breeze on 7860 unless you changed them), waits for both (Breeze takes about 10 seconds),
warms them up if `warm_up` is set, and turns RustVR's Voice setting on (it stays on).
`oblivion.ps1` builds main (not the plugin), installs it beside the game and boots straight into
`rustvr-voice` for the headset, with the Adoring Fan put a couple of steps in front of you.
`powershell -File tools\oblivion.ps1 -Close` puts the game away and your settings back; the plugin
stops with the runtime, and the servers keep running. To keep the logs, close with
`powershell -File tools\play.ps1 -Close -KeepLogs voice -Logs "rustvr.log,voice.log"` instead: they
go to the next free `captures\voice-<n>`.

`rustvr-voice` is the developer's own test save, not in the repository (`docs/DEVELOPING.md` has
the test saves): The Merchants Inn in the Imperial City's Market District, people about, the
market out of its door. To make one from any save:

```bat
powershell -File tools\oblivion.ps1 -Save <any save> -At ICMarketDistrictTheMerchantsInn
```

Then save it as `rustvr-voice` ([DEVELOPING.md](../DEVELOPING.md), "Test saves").

## What it does

- **The plugin's process.** The runtime starts `rustvr-voice.exe` while Voice is on and a game is
  connected, and stops it when Voice goes off or the game has been gone a minute (not for a loading
  screen). A change to the mic, acting, streaming, talking over them or the pause starts it again;
  the volume it reads as it changes. The plugin quits on its own 5 s after the runtime goes. With
  no `rustvr-voice.exe` beside the runtime, the runtime's log says so once.
- **Who's listening.** When you start talking: the person your head looks at most directly, within
  5 m and 25 degrees of where it looks. Looking at no one keeps whoever was listening while they're
  within 8 m. They stay with you 12 s after the last word, theirs or yours, and are let go past
  8 m.
- **They turn to you** and look at you, as the game's own speakers do (its say-to look and its
  turn): not while on their way somewhere or busy with something, not when the game has them not
  turn, and only once you're past 0.8 of `iActorTurnDegree` off their facing. They turn to the
  headset, not to where the thumbstick put you.
- **Who they are**, from the game: name, race, sex, class, factions and ranks, how they feel about
  you (from their disposition), you, where they are, what they're doing, the time of day, who else
  is near, and the game's own lines for them (greetings, rumours, idle chatter, goodbyes, by the
  game's own conditions). Anyone with a UESP page (1,111 people, the Shivering Isles' too) also has
  a paragraph from `crates/oblivion/data/bios.toml`, written in our own words from those pages; a
  guard or a bandit has only what the game says.
- **Their voice** is cloned from those same lines' voice files, copied out of the game's voice
  archives into `%LOCALAPPDATA%\RustVR\voice\oblivion\voices`. Lines only this person says make a
  voice of their own; otherwise it's their race and sex's, as in the game. The first line to a new
  voice takes longer: its clip (10 to 18 s of their lines) is made then.
- **A line**: a sentence or two from the LLM, with how it's said in brackets ("[sneering]"). Breeze
  acts it out (the Voice tab's acting setting), the note sets their face's expression (the game's
  seven emotions, stronger for "very", milder for "slightly"), and each sentence brings its
  subtitle as the game shows its own. Each line is made as loud as the game's own voice for them,
  times the Voice tab's volume, under a limiter (turned down round a loud word, never clipped).
- **Streamed** (the default): they speak a long silent sound with the game's own `Actor::Speak` (so
  the voice comes from them in 3D, at the game's voice volume), and the voice is written into that
  sound as it's made; each frame their mouth is set to the mouth shapes at the voice where it plays.
  When Breeze makes the voice slower than they say it, the stream waits at a pause (a comma, a full
  stop) and goes on with a good stretch, not a word at a time. Not streamed: a clause at a time,
  cut at the voice's pauses, each a WAV with the game's own lip file (in the game's folder,
  `Data\Sound\Voice\RustVR`).
- **The mic** is deaf while any menu is open, the game is paused or loading. Talking over them (on
  by default) cuts them off and what you say is answered; off, the mic is muted while they speak.
- **Memory goes with the game's saves.** Each save keeps what everyone remembers then; loading it
  brings that back, and a save the plugin never saw starts everyone afresh. It's kept in
  `%LOCALAPPDATA%\RustVR\voice\the-elder-scrolls-iv-oblivion` (`saves.json`, the prompt, the logs).
- **The brain page**, http://127.0.0.1:8090 on the desktop: each prompt, every line and how long
  each step took; `/history` goes through the session; the characters' prompt can be edited there
  (it's `npc_prompt.txt` in the memory folder, and the edit lasts).
- **People look at your head**: every look at the player, anywhere in the game (idle head tracking,
  being spoken to, the dialogue menu, greetings, combat), aims at the headset, within the game's own
  head and eye limits. In the dialogue menu it's set each frame, since the game may not run it
  there. The game's own turns of their body (its dialogue, activating) are still to where the game
  has you.
- **The game's own dialogue** is untouched: activating someone opens it as ever. In it, the weapon
  hand's stick left and right turn you (Snap or Smooth, as the Comfort tab's turning is), the
  dialogue staying with the speaker; up and down still go through the topics. In any other menu
  the stick is the menu's.
- **A person's name card** floats under their chin (six lines of text lower than a thing's), not
  over the face you look at, with the talk icon over it.
- **Simple voice commands** (no plugin, servers or AI): Windows' own speech recognition, offline,
  on the mic whose name says Virtual Desktop, else the system's default.
  - In a menu, its short texts (48 characters, 7 words at most: a dialogue's topics, "Goodbye",
    "Barter", a button's word) are phrases, also with "tell me about", "what about", "what do you
    know about" or "any" before them. Said, the game's cursor goes to it and clicks.
  - Out of menus, looking at someone: asking what they sell ("what have you got", "show me your
    wares", "let's trade" and the like) activates them and clicks their dialogue's Barter; "can we
    talk", "let's talk", "I'd like to talk" or "hello there" activates them. Each is heard with
    "alright", "alright mate", "hey", "so" or "okay" before it and "mate", "then", "friend" or
    "there" after; anything else round it, it may not be.
  - What's heard counts at 0.8 of the recognizer's confidence. The adapter's log says each thing
    it heard and how sure (`voice commands: heard "..." (0.95)`, with `: not acted on` when not).

## What to try

1. **Someone at a stall.** Walk up to a merchant (two or three steps away), look at their face and
   say "Good day." (Not "Hello there" with Simple voice commands on: that opens their dialogue.)
   - As you start speaking they turn to face you (if they're standing; not if they're walking
     somewhere or busy at something) and look at you.
   - A couple of seconds after you stop they answer, a sentence or two, in their race's voice or
     their own, the subtitle at the bottom a sentence at a time, their lips moving with the words
     and their face showing how they say it.
   - If they open their mouth but nothing's heard, their mouth doesn't move, or it's out of step:
     turn **Their voice streams as it's made** off in the Voice tab and say which it was.
   - The voice is from them: turn your head and it moves round you; step back and it's quieter.
2. **Ask them about themselves and the city.** "What do you sell?", "Heard any news?", "What do you
   think of the Emperor?" They should know their name, work and guild, be as friendly as their
   disposition says, and talk about the game's rumours for them in their world's words. Someone with
   a bio knows their life and the people round them: ask about their neighbours, what they want.
   The Adoring Fan should gush; a guard has only what the game says.
3. **Someone of another race.** A Khajiit, an Argonian, a Nord, a woman after a man: each should
   sound as that kind of person sounds in the flat game.
4. **Change who you talk to.** Look at someone else nearby and talk: they turn to you and the first
   goes back to what they were doing. Look at no one and keep talking: the one you were talking to
   keeps listening.
5. **Walk away** more than about 8 m: they let you go.
6. **Talk over them** mid-answer: they stop, and what you said is answered.
7. **The game's dialogue.** Activate someone (the trigger, an empty hand): the game's dialogue opens
   as always, and the mic stops listening. Turn with the weapon hand's stick left and right; up and
   down go through the topics.
8. **Their name.** Look at someone: their card is under their chin, the talk icon just over it.
9. **They look at your head.** Stand in front of anyone looking at you (talking to you, the
   game's dialogue open, or just watching you pass) and move your real head: lean, crouch, stand on
   your toes, step aside in the room. Their head and eyes follow the headset.
10. **Memory.** Tell someone something ("My name is ..., I'm looking for my brother."), make a new
    save, talk a bit more, then load that save and ask them what you told them: they remember what
    was said before the save, not after.
11. **The brain page** (http://127.0.0.1:8090 on the desktop): the prompt each person got, their
    lines and the time each step took.
12. **Simple voice commands** (they work with Voice off too). In someone's dialogue, read a topic
    out ("Rumors", "any rumors", "tell me about the Imperial City"): it's picked as a click would.
    "Goodbye" closes it. Out of menus, look at a shopkeeper and ask what they sell as you'd say it
    ("Alright mate, what are you selling?"): their dialogue opens on Barter.

## Settings

RustVR's menu (the menu button), **Voice** tab (`[voice]` in `%LOCALAPPDATA%\RustVR\settings.toml`):

| Label | Key | Default | Range |
|---|---|---|---|
| Talk to the game's characters out loud | `enabled` | off | |
| Simple voice commands: say a menu's words to pick them, "what have you got for sale" to trade (Windows' own speech recognition, no AI) | `commands` | on | |
| Microphone | `mic` | the headset's own (Virtual Desktop), else the default | the system's inputs |
| Their voices' volume (1: as loud as the game's own) | `volume` | 2.5 | 0.25 to 4 |
| They act out how a line is said (a quarter more of the voice server's work) | `act_out` | on | |
| Their voice streams as it's made (off: a few words at a time, each made whole first) | `stream` | on | |
| Talk over them to cut them off | `barge_in` | on | |
| A pause this long ends what you're saying | `pause` | 0.3 s | 0.15 to 1 s |

Changing any of these but the volume and the commands starts the plugin again: everyone's memory
starts afresh until a save is loaded (what they knew at each save is kept).

The plugin's own `%LOCALAPPDATA%\RustVR\voice.toml` (written on its first start):

| Section | Keys (as written) |
|---|---|
| `[llm]` | `url` (OpenAI's chat completions), `model` (`gpt-4o-mini`), `api_key_env` (`OPENAI_API_KEY`) or `api_key`, `temperature` 1.0, `max_tokens` 300; `[llm.extra]`, anything else every request takes |
| `[stt]` | `url`, `http://127.0.0.1:8083/v1/audio/transcriptions` |
| `[voice]` | `server`, `http://127.0.0.1:7860` (Breeze); `seed` 42 |
| `[mic]` | `vad_threshold` 0.5 (Silero's sensitivity) |
| `[models]` | `silero_vad`, `silero_vad.onnx` (beside `voice.toml`) |
| `[kev]` | `enabled` false: m8's decision model, which would have people speak up at what happens; off, only what you say is answered |

## Decisions

- **A plugin, a process of its own, not in the default build.** Speech recognition and the network
  stay out of the runtime's frames, and only players who run the servers need it. The runtime
  starts it, so turning Voice on in the menu is all it takes once it's built.
- **The keys and the servers' places are the player's.** `voice.toml` and `dev.toml` live in
  `%LOCALAPPDATA%\RustVR`; nothing of them goes in a repository.
- **The game's dialogue is left as it is.** Talking out loud is beside it: activating someone still
  opens the game's menu, with its quests and trade.
- **The game decides who turns**, as its own speakers turn (`SayTopic`): someone on their way or
  busy keeps at it and only looks.
- **Their voice is the game's**: cloned from their own lines where they have them, else their race
  and sex's voice, as the game files its lines.
- **Memory goes with the saves**, so a loaded save never has people knowing what happened after it.
- **Streamed by default**, for the soonest first word; a voice made slower than it's said waits at a
  pause, where a speaker would, rather than stuttering word by word.
- **Louder than the game's own by default (2.5).** The dialogue menu speaks in 2D; in the world
  voices are 3D, a few dB quieter at the same loudness. A limiter keeps them under full scale.
- **Acting on by default** (the user's choice), at a quarter more of Breeze's work.
- **Talking over them is any speech** the mic hears while they talk. Telling their voice through
  the headset's speakers apart from yours was tried and taken out.
- **Simple voice commands need no AI**: offline, a menu's own words, and phrases said with the words
  people put round them, never a wildcard (which hears "wares" in "the weather's nice").
- **Kev is off**: every line you say to someone is answered, and nothing else starts one.

## Not done / known issues

- **"Hello there" with Simple voice commands on** activates the person you look at: their dialogue
  opens and the plugin's mic goes deaf. Greet them some other way, or turn the commands off.
- **Breeze is slower than real time in the headset**: the game's rendering has most of the GPU, so
  in a long answer they now and then wait at a pause for the rest. Acting isn't what slows it, nor
  would a second Breeze help (the GPU's time splits between them); less for the GPU to render (the
  resolution or the supersampling) would.
- **About 3 s** from the end of what you say to their first word in the last measured run: the
  mic's pause, speech to text, the LLM's first sentence, Breeze's first sound (longer for a voice it
  hasn't made since it started), then the first word or two of streamed voice.
- Someone walking somewhere keeps walking while they talk (they look at you; they don't stop).
- Their own greeting (the game's "Hello" as you pass) can talk over a line.
- The game's dialogue menu speaks in 2D, centred; voices in the world are 3D and a little quieter.
  If they're still quiet at 4, the game's own Voice slider at full is the rest of the room.
- If they stop by themselves mid-line, the mic may be hearing them through the headset's speakers
  (talking over them is any speech): turn talking over them off.
- The mic is open: no push-to-talk on a controller yet (VR players of Mantella asked for it), if
  it cuts your sentences short or fires on noise.
- Of the 1,111 bios, about thirty were read against their pages and all were searched for game
  terms (levels, quests, "the player"); the rest are as the LLM wrote them from their page.

## History

Newest first.

- **Merged into main** with stealth and the range's melee: protocol v17, a conversation's turning
  flag `GS_MENU_TURNS` bit 14.
- **Simple voice commands**: Windows' speech recognition, no AI; said with the words people put
  round a phrase; it no longer stops for the session when there's nothing to listen for. Faces
  showed neutral for most of the LLM's notes: many more words for the emotions, the first one that
  says one wins, and plainly enough to see.
- **Talking over them**: on by default. Telling their voice through the headset apart from yours
  was tried, then taken out: any speech again.
- **The stream waits at pauses**: streamed a word at a time, a voice made slower than it's said
  stuttered; the game now says how much it has played, and the voice goes to a pause.
- **People look at the headset**, and a line is streamed into the game's own sound as it's made,
  the mouth set each frame (before: a few words at a time, each made whole first).
- **Clauses joined at their real end, cut only at pauses** (a word came out in two halves); the talk
  icon drops with the name card. The latency was measured step by step from the run's logs.
- **Turning in a conversation; the name under the chin; louder voices** (they were quiet in the
  world): a limiter, not a bent waveform, which crackled.
- **A bio for everyone with a UESP page**, after the Adoring Fan's and five of the Imperial City's.
- **Voice settings in RustVR's menu**; the voices as loud as the game's, then louder; acting turned
  off for speed, then back on by default at the user's choice.
- **The save moved** to The Merchants Inn (`rustvr-city` is at Odiil Farm, outside the city). The
  plugin keeps running through loading screens, its first words sooner, long sentences a clause at
  a time.
- **First build**: talk to anyone out loud through the plugin, the user's m8.
