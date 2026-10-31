# Playtest: talking to people out loud

Save: rustvr-voice

`playtest.bat` in this worktree (not committed: double-click it in Explorer) puts away any game left
running, starts the voice servers if they aren't up (speech to text and Breeze, waiting for both:
Breeze's first start takes about 4 minutes), turns RustVR's Voice setting on, then boots this build
into **rustvr-voice**: The Merchants Inn in the Imperial City's Market District, with people about
(a copy of the pickpocket feature's `rustvr-pocket`; `rustvr-city` is at Odiil Farm, outside the
city). The market is out of the inn's door. A key in the .bat's
window closes the game and keeps the runtime's and the voice plugin's logs in
`<builds>\voice\playtest-<n>`.

Look at someone a few steps away and talk. They turn to you as you start, and answer in their own
voice: cloned from the game's own lines for them, streamed into the game as it's made, the mouth
moving with the words, the voice coming from them. Your first line to someone new takes a moment
longer: their voice is being made from the game's clips.

## Before the first run (once)

1. **The DeepSeek key.** In a command prompt: `setx DEEPSEEK_API_KEY <your key>`, then open a new
   Explorer window (the .bat stops and says so if it isn't set). The key stays in your environment:
   it's in no file and no repository.
2. Nothing else: the plugin's config is already written, `%LOCALAPPDATA%\RustVR\voice.toml`
   (DeepSeek's `deepseek-flash`, thinking off; Parakeet on 8083; Breeze on 7860; the Virtual
   Desktop mic; Kev off). Your m8 setup is used where it is, `<voice models>` (Parakeet's and
   Silero's models, Breeze in WSL with your patch). The voices made from the game's clips go in
   `%LOCALAPPDATA%\RustVR\voice\oblivion\voices`, each character's memory in `...\oblivion\saves.json`.

## Settings

RustVR's menu (the menu button) has a **Voice** tab: Voice on or off, the microphone (the headset's
own, Virtual Desktop's, unless you pick another), their voices' volume (2.5 times the game's own
by default, up to 4: a limiter keeps them under full scale, turned down around a loud word rather
than clipped), acting lines out (on: their voice follows the line's "[sneering]"; a quarter more of
the voice server's work), their voice streaming as it's made (on: the soonest; off: the way it
was before, a few words at a time, each made whole first), talking over them to cut them off (on:
their own voice coming back through the headset is told apart from yours), and how long a pause
ends what you're saying. Changing any of these but the volume
restarts the voice plugin (what people were told since your last save goes with it).

## What to try, in order

1. **Someone at a stall.** Walk up to a merchant (two or three steps away), look at their face and
   say "Hello there."
   - As you start speaking they turn to face you (if they're standing; not if they're walking
     somewhere or busy at something, as in the flat game) and look at you.
   - A couple of seconds after you stop: they answer, a sentence or two, in an Imperial (or their
     race's) voice, the subtitle at the bottom as the game shows its own lines (a sentence at a
     time, as each is reached), their lips moving with the words and their face showing how they
     say it (angry, happy, sad...). The voice runs on as it's made, with no breaks inside a word.
   - If they open their mouth but nothing's heard, or their mouth doesn't move, or it's out of
     step with the voice: turn **Their voice streams as it's made** off in the Voice tab (the
     way it was) and tell me which it was.
   - The voice is from them: turn your head and it moves round you; step back and it's quieter.
2. **Ask them about themselves and the city.** "What do you sell?", "Heard any news?", "What do you
   think of the Emperor?" They should know their own name, work and guild, be as friendly as their
   disposition towards you says, and talk about the rumours the game gives them, in their world's
   words. Anyone with a name of their own (about 1,100 people, the Isles' too) also knows who they
   are from their UESP page: their days, the people round them, their quirks, how they talk. Ask
   about their life, their neighbours, what they want. The Adoring Fan (the .bat puts him in
   front of you) should gush; a guard or a bandit has only what the game says.
3. **Someone of another race.** A Khajiit, an Argonian, a Nord, a woman after a man: each should
   sound like that kind of person sounds in the flat game.
4. **Change who you talk to.** Look at someone else nearby and talk: they turn to you and the first
   goes back to what they were doing. Look at no one and keep talking: the one you were talking to
   keeps listening.
5. **Walk away** more than about 8 m: they let you go.
6. **The game's own dialogue.** Activate someone (trigger, empty hand): the game's dialogue menu
   opens as always. While any menu is open the mic doesn't listen. Flick the right stick left or
   right in it: you turn (a step, or smoothly, as your turning setting is), the dialogue staying
   with the speaker; up and down still go through the topics. In any other menu the stick is the
   menu's, as before.
7. **Their name.** Look at someone: the card with their name is under their chin now, not over
   their face, and the talk icon sits just over the card, where it was before.
8. **They look at your head.** Stand in front of anyone who's looking at you (talking to you, the
   game's dialogue open, or just watching you pass) and move your real head: lean left and right,
   crouch, stand on your toes, step aside in the room. Their head and eyes follow your headset,
   not the spot the thumbstick puts you at. Do it in the game's dialogue menu too. (Their body
   still turns by where you stand, as the game has it; the head and eyes do the rest, within the
   game's own limits: about 45 degrees either side, 30 up or down.)
9. **Memory.** Tell someone something ("My name is ..., I'm looking for my brother."), save (a new
   save), talk a bit more, then load that save and ask them what you told them: they remember what
   was said before the save, not after.
10. **The brain page.** On the desktop, http://127.0.0.1:8090 shows the exact prompt each person
   gets, every line and how long each step took (/history keeps the session).

## What I checked, and what I couldn't

Checked (without the game):

- m8 as a library: its tests (the mouth's shapes, old saves loading, the sentence splitter), the
  Halo binary and `stt-server` build.
- Parakeet's word times on a game line ("Very well. If there's anything else I can do for you, let
  me know.", every word timed, 159 ms), and m8's mouth shapes for it against the game's own `.lip`
  for that line: in step within a frame.
- The plugin's tests (its config, DeepSeek's extras, joining clips into one voice); the protocol's
  and the runtime's settings tests; the adapter's (the emotions, the game's lip format, read back).
- Everything builds: the adapter, the runtime, the range, the emulated headset, the plugin.
- The `.lip` format and every engine function used were checked in play and
  against the game's own files (`docs/research/voice.md`).

Also checked, for streaming and the looks (without the game): Breeze's speed (one server and two
at once, plain and acted), Parakeet's word times on a growing line as the stream asks for them
(50 to 90 ms a time, every word in place a quarter second behind the voice), the plugin's cut
between words, the protocol's rings; the game's sound, face and look functions read off the
game's own behaviour in play (`docs/design/voice.md`, `npcs.md`).

Not checked (needs you and the game):

- Anything in the game: the hooks into its dialogue (speaking, the lip file, the subtitle, turning
  and looking), the personas built from its data, the clips copied out of its archives.
- The streamed line: the voice written into the game's playing sound, the mouth set each frame,
  the subtitles and expressions as each sentence is reached, its end. And the look-at hook on the
  headset (and in the dialogue menu).
- Every one of the 1,100 bios: I read about thirty against their pages and searched them all for
  game terms (levels, quests, "the player"); the rest are as the LLM wrote them from their page.
- The Virtual Desktop mic and how the open mic copes with the headset's own speakers.

Known for now:

- The game's dialogue menu speaks in 2D, centred; in the world (theirs, and the game's own greetings)
  voices are 3D, a few dB quieter at the same loudness. If they're still quiet at 4, the game's
  own Voice slider at full (and its effects or music a little lower) is the rest of the room.
- In the headset Breeze makes their voice a little slower than they speak it: the game's rendering
  has most of the GPU (a second of acted voice took 1.2 to 1.3 s in your last run; 0.5 s with the
  game closed). So in a long answer they now and then wait for the rest: at a pause (a comma, a
  full stop), and then go on with a good stretch of it, not a word at a time. Acting
  isn't what slows it (a quarter more work), nor would a second Breeze help (I tried: the GPU's
  time splits between them). Less for the GPU to render (the game's resolution or supersampling)
  is what would let it keep up.
- From the end of what you say to their first word was about 3 s in your last run (2.8 to 4.9 s):
  the mic's pause 0.3 s, speech to text 0.05 to 0.15 s, DeepSeek's first sentence 0.5 to 0.8 s,
  Breeze's first sound 0.5 to 0.8 s (1.4 to 2.4 s for a voice it hasn't made since it started),
  and the first piece of voice made whole, a second or so, before the game could play it.
  Streamed, that last is the first word or two and the start of the next (Parakeet has to hear
  where the word ends): about half a second sooner.
- Someone walking somewhere keeps walking while they talk (they look at you, they don't stop).
- Their own greeting ("Hello", as the game has people greet you when you pass) can talk over a line.
- Talking over them (on now, in your settings too): the mic stays open while they talk, and their
  own voice coming back through the Quest's speakers is told apart from yours by how loud it is
  (it learns how loud their voice comes back, in their first second or two of talking; you need
  to be about twice as loud as that). As you start speaking they stop, and what you say is
  answered. If they cut themselves off, or don't stop when you talk, the plugin's log
  (`voice.log`) says how loud their voice came back (`[mic] ... x0.12 of its level`): tell me what
  it says.
- If the open mic cuts your sentences short or fires on noise, say so: push-to-talk on a controller
  is the next step (VR players of Mantella asked for it).

## If something's wrong, the logs

- The adapter: `C:\Program Files (x86)\Steam\steamapps\common\Oblivion\rustvr-oblivion.log`
  (from the last "adapter log started"): lines starting `voice:` (the plugin running, who you're
  talking to, each line said, its emotion and length, the clips copied).
- The plugin: `%LOCALAPPDATA%\RustVR\voice.log` (the mic it opened, what it heard, the LLM's lines,
  the voice's timings), copied to `<builds>\voice\playtest-<n>` by the .bat.
- The runtime: `%LOCALAPPDATA%\RustVR\rustvr.log` (`voice: started the plugin`), copied there too.
