# Talking to NPCs

Research for the voice feature: the player talks out loud, in the headset, to any NPC in the world,
and the NPC answers in its own voice (cloned from the game's voice actors), its mouth moving with the
game's own lip sync and its voice coming from it in the world. The brain is the user's
[m8](https://github.com/chasmlol/m8) (made for a Halo CE companion; the voice crate uses its
`rustvr` branch), made a plugin of the framework for any game, with a hosted LLM. What the modding
scene does is for inspiration only: nothing is copied.

**An optional plugin, for power users.** It isn't in the default build, and `tools/oblivion.ps1`
doesn't build it. You build `rustvr-voice` yourself (`-p rustvr-voice` with the framework's other
crates); the runtime starts `rustvr-voice.exe` from beside its own exe while the Voice setting is on
and a game is connected. Its servers are yours to run: m8's stt-server (speech to text) and a Breeze
TTS server (the cloned voices); `clips/voice-servers.ps1` starts them from where `dev.toml`'s
`[voice]` lines say they are. The LLM's key is yours too, in `%LOCALAPPDATA%\RustVR\voice.toml`
(`api_key`) or the environment variable it names (`api_key_env`), never in a repository.
Simple voice commands (below) need none of this.

## How the modding scene does it

Gathered from Mantella's and CHIM's source and docs, their Nexus pages and issues, and Reddit and
press through search excerpts (Reddit and Nexus block fetching). Researched 2026-10-30.

### Mantella (Skyrim, Fallout 4)

* **Pipeline**: Silero VAD (a 0.25 s pause ends a turn; "proactive mode" re-transcribes every
  0.3 s while you talk) into Moonshine or faster-whisper; OpenRouter by default or any
  OpenAI-style LLM; Piper, xVASynth (a model per voice type) or XTTS (a clip per voice type).
  ([Mantella](https://github.com/art-from-the-machine/Mantella) `src/stt`, `src/config`)
* **Playing a line through the engine**: for each sentence it overwrites the WAV and `.lip` of
  a placeholder topic and has the actor `Say()` it, so the game gives lip sync, 3D voice, head
  look and subtitles. Two placeholder topics alternate, because repeating one "can cause the
  engine to replay the previous (cached) voiceline". `.lip` files come from the Creation Kit's
  `LipGenerator.exe` (the WAV and the text). ([Mantella-Spell](https://github.com/art-from-the-machine/Mantella-Spell)
  `MantellaConversation.psc`; Mantella `src/tts/ttsable.py`)
* **Who's talked to**: a spell or hotkey on the crosshair NPC, or up to five nearby; "goodbye"
  ends it; an MCM toggle has them stop to talk.
* **Persona and memory**: a 41k-line CSV of bios per NPC, the player's gear, location, time and
  weather, events pushed from Papyrus; a one-paragraph summary per NPC at a conversation's end,
  for those who heard it.
* **Latency**: sentences streamed (at least three words, four sentences at most); the
  "real-time NPCs" guide aims at an LLM answering in under 0.5 s and skips the first sentence's
  `.lip` to save 0.5 s. In 2023 a reply took about 15 s ([Road to VR](https://www.roadtovr.com/chatgpt-skyrim-vr-ai-mod/)).

### Herika, AI-FF, now CHIM (Skyrim)

* An SKSE DLL and a PHP server in a WSL image ([HerikaServer](https://github.com/Dwemer-Dynamics/HerikaServer)).
  Typed game events are either only logged or start a turn; idle NPCs may speak up by chance.
  With several possible answerers, a cheap model picks one within 2 s and falls back to the
  nearest; a crosshair target or a spoken name skips that.
* Memory per NPC per playthrough, rewound by loading an older save; diaries; lore scoped to what
  each NPC would know. Parakeet is one of its recommended speech-to-text options.
* **Not the game's lips or sound**: it plays WAVs itself (XAudio2, 3D emitters) and writes
  FaceGen phoneme keyframes from the text spread over the clip ([CHIM](https://github.com/Dwemer-Dynamics/CHIM)
  `Plugin/SpeakManager.cpp`). Its New Vegas sibling, Dialectic, clones from the user's own
  `Data/Sound/Voice` and silences vanilla speech by swapping in a silent voice type.

### Oblivion

* No live voice-LLM mod was found for classic Oblivion or Remastered (one GitHub repo claiming to
  be one has no code and links to outside installers: a lure).
* Oblivion's `.lip` files come only from the Construction Set 1.0.303's generator (broken in
  1.2.404, fixed by the CS Extender), which needs the line's MP3 and text
  ([UESP: Lip Synch](https://cs.uesp.net/wiki/Lip_Synch)); no public spec of the format was found.
  A line without one plays with a still face.

### What players love and hate

* **Love**: characters that stay in character, remember earlier talks, know the hour and notice
  what you carry ([UploadVR](https://www.uploadvr.com/playing-skyrim-vr-with-chatgpt-powering-npc-conversations/)).
* **Latency**: free tiers lag badly; today's bar is the first sentence in a second or two.
  Players want to cut a slow or long reply short.
* **Monologues** are the top complaint: "long winded responses that can't be interrupted".
* **Too clever, too modern**: NPCs "way too proper", present-day references, assistant-speak.
* **Who's addressed**: crowds stall or the wrong NPC answers in another's voice; tagging NPCs
  with a gadget first was too much trouble.
* **Voices**: robotic to some; others object that the actors never consented to cloning.
* **Walking off** mid-conversation, and vanilla greetings talking over AI lines.
* **VR**: silence-based turn ending failed for one player every time and push-to-talk on a
  controller was asked for ([Mantella #12](https://github.com/art-from-the-machine/Mantella/issues/12));
  an open mic in a headset hears the NPC through its speakers.

### What this design takes from it

* Speak through the engine (Mantella's lesson), with every line its own file name, so no cached
  sound is replayed: here the file is ours and the game's `Actor::Speak` plays it directly, no
  placeholder topic.
* Never a line without a `.lip`: here the plugin makes the mouth shapes from when each word is
  said (speech to text on the voice itself, the times CHIM gets from the text alone) and the
  adapter writes the game's own lip file.
* Short replies (the prompt asks for a sentence or two; 300 tokens at most), streamed sentence by
  sentence; who's addressed is where the head looks (no gadget), shown by them turning to listen
  as soon as speech starts.
* Voices from the user's own install, by the game's own choice of lines for that NPC: their own
  actor where they have lines of their own, else their race and sex.
* Personas from the game's records (no CSV), the NPC's own lines as their way of speaking, and
  nothing they couldn't know; memory per NPC per save.
* A headset mic beside its speakers hears the NPC too. Barge-in is the Voice tab's "Talk over them
  to cut them off", on by default (their voice through the speakers is told apart from the
  player's); off, the mic is deaf while they talk. Push-to-talk on a controller isn't built (the
  next step if the open mic misfires in play).

## How Oblivion plays a line

A dialogue line is a `TESTopicInfo`'s response; the game plays it through one function on the
actor (`Actor::Speak`), and everything we need hangs off that.

### The voice file and its path

* A response's voice is `Data\Sound\Voice\<plugin>\<voice race>\<M|F>\<file>.mp3`. The plugin is
  the info's file (`Oblivion.esm`, or the last plugin that changed it); the voice race is the
  speaker's race's voice for its sex (a race can borrow another's voice); the sex is `M` or `F`.
* The file's name is `<quest editor id>_<topic editor id>_<info form id %08X>_<response number>`,
  e.g. `bladesmartinconvsystem_goodbye_00068900_1.mp3`. The game looks for it loose or in a BSA.
* `Oblivion - Voices1.bsa` and `Voices2.bsa` hold 38,898 lines, an `.mp3` (MPEG-1 layer III, 64
  kbit/s, 44.1 kHz, mono) and a `.lip` for each, filed by race and sex only: Oblivion has no voice
  types. A generic Imperial man says every generic Imperial man's lines, and the unique characters'
  lines (Martin, Jauffre, Baurus) are in their race's folder too, under infos only they can say
  (`GetIsID` conditions). The Shivering Isles add `DLCShiveringIsles - Voices.bsa`.

### Speaking a line

* Only within `iSpeakSoundLipDistance` of the player does the face move at all.
* **The lip file**: the voice's path with `.lip` for its extension (loaded in the background with
  `bBackgroundLoadLipFiles`). It drives the face's 16 phoneme morphs (FaceGen's: Aah, BigAah, BMP,
  ChJSh, DST, Eee, Eh, FV, I, K, N, Oh, OohQ, R, Th, W) and 17 modifiers (the blinks, brows, looks,
  squints and the head's pitch, roll and yaw), 30 frames a second. With no `.lip` the face isn't
  animated at all.
* **The expression**: with FaceGen expressions on, the line's emotion (neutral, anger, disgust,
  fear, sad, happy, surprise) at its value (0 to 100) goes to the face.
* **The sound**: 2D for the player, else 3D at the actor and attached to them, so it moves with
  them, with the dialogue's distances. The actor counts as talking for the sound's length (with a
  lip, plus the lip's lead-in and `fSpeechDelay`; with none, the text's length times
  `fAIDefaultSpeechMult`, 0.05 s a character).
* **With no `.mp3`** of the voice's name, the game plays a `.wav` of it. So a `.wav` and a `.lip` of
  our own, under `Data\Sound\Voice\`, play exactly as the game's lines do. The voice is
  DirectSound 3D, which Windows since Vista renders in software as panning and distance only.

### A line the game is playing: its sound, its face

What streaming a line into the game rests on (Speak a long silent WAV, write the voice into its
sound as it comes, set the mouth each frame):

* **One buffer, filled once.** A WAV voice is one sound buffer the size of the file's data, filled
  once and never touched again, and nothing in the game rewinds a playing voice. So a voice written
  into it just ahead of where it plays is heard.
* **Starting.** The game starts the voice after the lip's lead-in and `fSpeechDelay`. Started more
  than 125 ms late, it skips ahead in the sound by the lateness.
* **What stops it.** Menus pause every sound and resume it where it was. Past the sound's distance
  (`fDialogAttenuationMax`, 35 m) a voice is stopped. Activating someone for the dialogue menu, load
  screens, sleeping and waiting, and a few menus stop every voice.
* **The face** works through a lip's frames on a clock of its own (each frame's real time), not the
  sound's. So a streamed mouth is set each frame instead: the face's keys to come dropped and one
  key of the 16 phonemes at once, the shapes at the voice where the sound plays. The modifiers (the
  blinks, the looks) stay the game's. The mouth shuts with a blend to rest, as a lip ends.
* **The expression** is the emotion's by the game's table: anger and disgust 0, fear 1, happy 2,
  sad 3, surprise 4.
* **The subtitle** shows for the sound's length, at least 2 s, then while it plays; another
  replaces it, and stopping the voice clears it.
* **The end.** While they talk the game counts their talk time down, and at 0 with the voice done
  it ends the line its own way (talking off, the look at the listener cleared). A streamed line keeps
  the talk time at what's left to play, at least half a second; at its end the voice is stopped, the
  talk time set to 0, the mouth sent to rest, and the game ends the talking.

### The `.lip` format

The adapter writes the game's own lip format: a 12-byte header, then the data, run-length encoded
(runs of zero bytes counted); decoded, the frame count (30 a second), the lead-in (negative, in
frames: -1 to -9 in the game's files), then per frame 16 phoneme weights and 17 modifier weights
(0 to 1). A 4.1 s line is 123 frames, 16 KB decoded, 4.4 KB on disk. Mid-line frames in the game's
files are mostly one phoneme near 1 with a neighbour or two blended in (`Eh` 0.97 and `N` 0.04;
`Aah` 0.59, `BMP` 0.26, `Oh` 0.15).

So a line we generate gets a `.lip` the game reads as its own, and the game's talking head plays it.

### Script commands built on it

* **`SpeakSound "<file>" [emotion] [value]`**: the actor speaks a sound file as all dialogue does,
  and counts as talking for the sound's length. This is everything a line of ours needs.
* **`Say <topic>`** and **`SayTo <target> <topic>`**: lip sync at any distance for the line, the
  speaker's head turned to whoever they say it to.
* **`Look <target>`**: the head tracks the target, until **`StopLook`** (see [npcs.md](npcs.md)).
* **`StartConversation <target> <topic>`**: the AI's own walking up to someone to talk.

### Saying a topic to someone

How an NPC says a line to someone in the world, and the model for ours:

* The game picks the info the speaker says to the target by its conditions (random infos pick again
  each time). A response has its text, emotion and value, and its voice file for this speaker.
* The speaker's previous voice is stopped, the response is spoken, they count as talking, and the
  subtitle shows for as long as the sound plays (with `bGeneralSubtitles`).
* The speaker's head looks at the target, and, past 0.8 of `iActorTurnDegree` (90 degrees; 0.8 of
  `iActorKeepTurnDegree`, 10, while already turning), the body turns to them a step a frame at its
  turn speed with its turning animation. Not while it's travelling or using something.
* The game's own topics every speaker has lines for: GREETING, HELLO, GOODBYE, Idle Chatter, Rumors
  (INFOGENERAL). An info may be random, and may be conditioned `GetIsID` on one speaker.

### Talking to the player

The player activating an actor checks that they can talk (asleep, in combat, yielding), stops
both, dips the music and sounds and opens the dialogue menu on the speaker. The voice feature
leaves all of that alone: activating still opens the game's own dialogue menu.

## The design

### Where things live

| Part | Owns | Where |
|---|---|---|
| m8 (the user's, public) | The brain for any game, now a library: the feed and memory per speaker, the LLM (local or hosted), Kev (optional, off), the mic (Silero VAD, barge-in), speech to text, the cloned voice, the mouth shapes of what's said, a brain page (http://127.0.0.1:8090: every prompt and line, the characters' prompt edited there). Its own `m8` binary is still the Halo CE companion, on its socket. | github chasmlol/m8, branch `rustvr` |
| `rustvr-voice` (framework) | The plugin: a process the runtime starts while the player has Voice on and a game is connected. It runs m8 against the shared mapping: reads who's around and who's listening, writes each line (a WAV, its mouth shapes, or a stream) and tells the adapter to play it. Its config and keys are the player's, outside any repo. Not in the default build. | `framework/crates/voice` |
| The protocol | The voice regions: the scene and the game's events (adapter to plugin), the lines and the streamed voice (plugin to adapter). | `framework/crates/protocol/src/voice.rs` |
| The runtime | The Voice tab's settings, starting `rustvr-voice.exe` from beside its own exe (again when the mic or how it listens changes, at most every 10 s if it stops), stopping it when Voice goes off or the game has been gone a minute; its output in `voice.log` beside the runtime's log. | `framework/crates/runtime/src/voice.rs` |
| The Oblivion adapter | Who's around and who's addressed, who each of them is (from the game's data), their voice (the game's own clips), saying a line (the game's `.lip` and `Actor::Speak`, or a streamed line into a playing sound), turning to listen, saves and loads; people looking at the player's real head (`look.rs`); the simple voice commands (`commands.rs`). | `crates/oblivion/src/voice.rs` |

The plugin is its own process rather than part of the runtime: speech recognition, the HTTP calls
and the native VAD stay out of the VR runtime (a stall or a crash there can't drop a headset frame),
and the runtime builds without them.

### A turn

1. The player starts talking (Silero in the plugin hears speech). The plugin tells the adapter
   *listen*; the adapter picks who's addressed then (below) and has them turn to the player and look
   at them, as the game's NPCs do when spoken to, before the sentence is finished.
2. The sentence ends (a short pause), speech to text (Parakeet) gives its words, and the line goes
   into the addressed NPC's feed. Without Kev every line said to an NPC is answered, and nothing
   else starts one.
3. The LLM (any OpenAI-style chat API: the written config's default is OpenAI's `gpt-4o-mini`; the
   developer uses DeepSeek, with `thinking = { type = "disabled" }` under `[llm.extra]`) writes the
   answer, streamed; each sentence goes to the voice (Breeze, cloning the NPC's voice clip) as soon
   as it's complete.
4. Streamed (the Voice tab's setting, on by default): the voice goes to the game as Breeze makes
   it. Every quarter second of new voice, Parakeet gives the word times of the last second sent
   and what's new, and the voice goes as far as the gap after the last word it's sure of (ending
   0.15 s before what's come, with another word heard after it: the last heard may be the start
   of a longer one, "a" of "Azura"), with FaceGen's 16 phonemes for it, 30 a second, a frame for
   each 1/30 s of voice (`VoiceStream`'s rings). A sentence's end sends the rest; each sentence's
   words go as a subtitle at the sample it starts on. Breeze slower than real time (its speed
   measured as its voice comes), and the game with under 0.6 s left to say, the voice goes only as
   far as a pause (a gap of 0.15 s, or 0.05 s after a comma or a stop; at a word's end after 2.5 s
   with none), and with the game waiting, only with 0.8 s of it or the sentence's end: a word at a
   time it stutters. A line's first voice goes at its first pause, or a second in at a word's end.
   Simulated on Breeze's own voice and Parakeet's word times (a 6 s line), 1.6 times slower than
   real time: the first word 2 s after Breeze's first sound and two waits (word by word: five);
   1.25 times slower, one; faster than real time, none.
5. The adapter has the NPC speak a long silent WAV (`stream-<rate>.wav`, 30 s, no lip) as the
   game speaks a response: into their voice slot, talking, their head on the player, the voice
   from them in the world with the game's distance and volume. Each frame
   it writes what's come into that sound's buffer, just past where the sound card reads it (a run
   that ran out goes on past where it plays, after the gap), sets the mouth to the phonemes at the
   voice where it plays (30 ms ahead: the sound's way to the ears), and as a sentence is reached,
   its subtitle and their expression (the line's own "[angrily]"). A line longer than the silent
   WAV goes on in another as it ends.

Not streamed, a sentence's voice is gathered whole, a clause at a time: the plugin writes `<n>.wav`
and `<n>.lips` (the mouth from the clause's word times), the adapter writes `<n>.lip` in the game's
format and has the NPC speak `<n>.wav`, the face playing the lip file; the next follows as one ends.
A long sentence goes to the game a clause at a time, so its first words don't wait for the voice
of all of it. Each cut is in the middle of a pause of at least 0.12 s: a shorter quiet is mostly a
"p" or a "t" closing inside a word, and a cut there splits the word in two. Each clause starts
when the one before ends, by its own length (the game's is 0.1 s longer, the lip file's padded
end), so the pause joins up as it was. When Breeze is slower than real time (the GPU shared with
the game's rendering), they wait for the next clause at a pause, where a speaker would; streamed,
at a gap between words.

Breeze's speed, measured on the developer's machine (one high-end GPU shared with the game): with
the game not running, warm, the Python server makes a second of voice in 0.40 s plain and 0.51 s
acted (guidance: a quarter more work, not twice); in the headset with the game rendering, about 1.2
to 1.3 s acted. A second Breeze beside it doesn't help: two at once make no more voice a second
than one (the GPU's time splits between them).

Breeze-TTS-2.cpp (HoppouAI, on ggml, the same `/v1/audio/speech` and `/health` as Breeze's own
server, so m8 needs nothing but its address), its Q8_0 model (3.6 GB): built for Vulkan, slower
(0.82 s acted, 0.47 to 0.52 s plain); built for CUDA (`-DBREEZE_CUDA=ON`, for the GPU's
architecture), 0.34 s acted, 0.24 s plain, the first sound in 0.13 to 0.19 s (the Python one: 0.25
to 0.48 s). Either works: `voice.toml`'s `[voice] server` names the one the plugin calls (the
written default is port 7860, which `clips/voice-servers.ps1` also reads).

### Who's listening

* The candidates are the actors the game has near the player (its high-process actors), alive,
  conscious, not hostile or fighting, and people (NPCs, not creatures).
* The addressed one is the one the head looks at most directly (within 25 degrees of where the
  head looks, at their head), within talking distance (5 m: the game starts dialogue by
  activation, at arm's reach, and has no distance for talking across a few steps), when the
  player starts talking. Looking at no one keeps whoever was listening, while they're within 8 m.
* They turn to the player and look at them while listening and while answering (the game's own
  look target), and go back to what they were doing after a while with nothing said.
* Activating an NPC still opens the game's dialogue menu, untouched.
* Their look is at the player's real head, the headset, not the game's own eye (`look.rs`: the
  player's look-at point is the headset's), for every look at the player in the game.
* In the game's menus the plugin hears nothing and cuts no one off (`VS_DEAF`).

### Who they are

From the game: their name, race, sex, class, factions and ranks, their disposition towards the
player (the game's own), where they are (the cell, the region), what they're doing (their AI
package), the date and time (the game's calendar), the player's name, race and class, and what the
game's dialogue gives them to say (their greeting, their rumours and topics: the infos whose
conditions pass for this speaker, by the game's own condition check). A generic prompt (the
framework's, overridable) wraps it: who they are, speak as them in their world, a line or two as
people talk, no lists, nothing they couldn't know.

And a paragraph on who they are as a person, their days, the people round them, their quirks and
how they talk, from `crates/oblivion/data/bios.toml` by the name the game gives them. A few are
written by hand; the rest, one for each of the game's named people with a page of their own on
UESP (about 1,100), are written by `tools/oblivion_bios.py`: it reads the names from the game's
plugins, the pages through UESP's API, and has the voice's own LLM write each paragraph from its
page in the style of the hand-written ones, as the person is the day a stranger meets them (nothing
from later in their story, nothing about them they don't know). UESP's text is CC BY-SA 2.5; the
file says so. Someone without a page (a guard, a bandit) has only what the game says.

### Their voice

Each NPC's reference clip is the game's own voice for them: the lines the game has that NPC say
to the player for its own topics (GREETING, HELLO, Rumors, Idle Chatter, GOODBYE, by its own
conditions, each response's voice file built for this speaker), with the response text as the
transcript. Lines only this NPC says (an info conditioned `GetIsID` on them) are their own actor,
and make a voice of their own (`npc\<base form id>`); otherwise the voice is their race and sex's
folder, shared by everyone the game voices with it (`oblivion.esm\imperial\m`), as in the flat
game. The adapter copies those MP3s out of the archives (loose files first, as the game reads
them) into the voice's folder; the plugin joins lines into one clip of 10 to 18 s. Breeze (with
m8's patch) encodes a clip once, by its contents.

### Their mouth and their sound

The game's own, end to end: its lip format, its talking head, its expression, its 3D voice attached
to the actor. The runtime's spatialiser (interaural time and level, head shadow, a rear low-pass) is
better than DirectSound 3D's pan and distance, but the game's sound has the game's voice volume, its
pausing in menus, its distance falloff for dialogue, and its "this actor is talking" state, all from
the same file as the lips. So the game plays it. (Feeding the game's own 3D sounds through the
runtime's spatialiser would improve every sound in the game, the voices with them: a follow-up.)

How loud: the game plays a WAV of ours as it plays its own MP3s (both end in the same volume, the
master times the voice slider), with no falloff within `fDialogAttenuationMin` (7.75 m;
`fDialogAttenuationMax` 35 m, `fDSoundRolloffFactor` 4 past it). But the dialogue menu speaks in
2D (at `fDBVoiceAttenuationIn2D`, 0 dB, centred), and everywhere else, our lines too, it's 3D,
panned, a few dB quieter: so the player hears ours as quiet next to the menu's, though the files
are louder than the game's (its clips have room to spare: speech at about -17 dB, peaks at full
scale; a line of ours is made the game voice's loudness times the player's volume, 2.5 by
default). Turned up, a line's peaks are kept under full scale by a limiter, the line turned down
around a loud word (5 ms ahead of it, back up over 80 ms), never its waveform bent, which
crackles.

### Memory

m8's: each NPC's feed (what was said with them, and what they said), folded into a summary past
16k tokens. A save stores every NPC's memory as it was, and loading that save brings it back (by the
save's file name). Kept in the plugin's folder per game.

### Simple voice commands

Without the AI, a few things are said to the game itself, recognized by Windows' own speech
recognition (SAPI's in-process recognizer and the Windows language's desktop engine, offline, which
the 32-bit game can load; on the headset's mic, the audio input named Virtual Desktop's, else the
default), in the adapter (`crates/oblivion/src/commands.rs`), on when the runtime says
(`RT_VOICE_COMMANDS`, the Voice tab's "Simple voice commands", on by default). It needs neither the
plugin nor its servers. The recognizer runs on a thread of its own and has one rule, the phrases
that mean something now, set from the game's frame as what shows changes:

* In a menu: every short text of the menu on top that shows (a text tile's `string` trait), up to
  48 characters and 7 words. Heard, the game's cursor goes to the text and the left button is
  pressed and let go, as the laser clicks it.
* Looking at someone (the HUD's target, a person near the player): ways to ask what they sell (the
  Activate key tapped, then their dialogue's "Barter" clicked once it's up) and to ask to talk (the
  Activate key).

Each phrase is also listened for with the words people put round it (before: "alright", "alright
mate", "hey", "so", "okay"; after: "mate", "then", "friend", "there"; before a menu's word: "tell me
about", "what about", "what do you know about", "any"), not with SAPI's wildcard (`...`): with it,
"I think the weather is nice today" was heard as "... wares ..." at 0.82. What's heard counts at
0.8 of the engine's confidence. Tried on Windows' own voice saying them (its synthesizer to a WAV,
the recognizer's input from it): "what have you got mate", "alright mate, what are you selling",
"hey, what do you sell then", "tell me about the imperial city", "any rumours" all 0.97 to 0.98;
"what have you been up to mate" heard as "what have you got mate" at 0.68, "what is the weather
like" at 0.26, two sentences that aren't anything as nothing.

### Talking over them

On by default: any speech the mic hears while they talk cuts them off (m8's barge-in). The game
says it's still saying a line (`Game::still_speaking`) until it has played all of it: m8's own
count ends when it has sent the voice, earlier than the game says it when the game waited on
Breeze.

### Settings and keys

The plugin's config is the player's (`%LOCALAPPDATA%\RustVR\voice.toml`, written with the defaults
on the first run), outside every repo:

* `[llm]`: the chat API's `url` and `model`, its key in `api_key` or the environment variable
  `api_key_env` names (the default names `OPENAI_API_KEY`), `temperature`, `max_tokens` (300), and
  `[llm.extra]` for anything else the API takes;
* `[stt]`: speech to text's `url` (m8's stt-server, Parakeet, with word times; 8083 by default);
* `[voice]`: the Breeze server (`server`) and its noise `seed`;
* `[mic]`: Silero's `vad_threshold`; `[models]`: `silero_vad`, the VAD model's file;
* `[kev]`: off by default.

The rest is RustVR's own Voice tab (the runtime's `settings.toml`, `[voice]`): Voice on or off (off
by default), "Simple voice commands" (on), the microphone (empty: the headset's own, Virtual
Desktop's, else the default), the voices' volume (2.5), acting lines out (on), streaming (on),
talking over them (on) and the pause that ends what you say (0.3 s). A change to the mic, acting,
streaming, talking over or the pause starts the plugin again; the volume it reads as it changes.

Each game's memory, its logs and the characters' prompt (`npc_prompt.txt`, the framework's to
start with) are in `%LOCALAPPDATA%\RustVR\voice\<game>`.
