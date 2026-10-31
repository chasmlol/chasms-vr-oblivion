"""Writes a bio for each of Oblivion's people (crates/oblivion/data/bios.toml), for the voice: who they
are, a paragraph, written by an LLM from their UESP page in the style of the bios already there.

    python tools/oblivion_bios.py [--game <Oblivion folder>] [--limit N] [--only "Name,Name"] [--jobs 12]

The people are the game's own: every NPC with a name in its plugins (Oblivion.esm, the Shivering
Isles, Knights of the Nine, the downloads), but not a name more than two share ("Imperial Watch",
"Bandit"), and only those with a page of their own on UESP (`Oblivion:<Name>` or
`Shivering:<Name>`, or the person a page of that name that "may refer to" several points to), read
through its API fifty at a time. The pages and the bios are kept in --cache: a bio once written is
kept, and a run writes only the ones still missing. The LLM is the voice plugin's own, voice.toml's
[llm] (its key from the config, or the variable it names; never shown). bios.toml's bios written by
hand stay as they are, and nobody they name gets another; the rest of the file, after its WRITTEN
line, is remade each run. UESP's text is CC BY-SA 2.5: the bios are written from it, and the file
says so.
"""
import argparse
import concurrent.futures
import json
import os
import re
import struct
import sys
import textwrap
import time
import tomllib
import urllib.parse
import urllib.request
import zlib

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIOS = os.path.join(REPO, "crates", "oblivion", "data", "bios.toml")
# (the line in bios.toml after which every bio is this tool's, remade each time it runs)
WRITTEN = "# Written by tools/oblivion_bios.py from here to the end (run it again to remake this part):"
DEFAULT_GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion"
DEFAULT_CACHE = r"<builds>\voice\uesp"
VOICE_TOML = os.path.join(os.environ.get("LOCALAPPDATA", ""), "RustVR", "voice.toml")
# The game's plugins with people in them, and the wiki namespace each one's people are in.
PLUGINS = [
    ("Oblivion.esm", "Oblivion"),
    ("DLCShiveringIsles.esp", "Shivering"),
    ("Knights.esp", "Oblivion"),
    ("DLCBattlehornCastle.esp", "Oblivion"),
    ("DLCFrostcrag.esp", "Oblivion"),
    ("DLCThievesDen.esp", "Oblivion"),
    ("DLCVileLair.esp", "Oblivion"),
    ("DLCOrrery.esp", "Oblivion"),
    ("DLCMehrunesRazor.esp", "Oblivion"),
]
USER_AGENT = "RustVR-Oblivion-bios/0.1 (a personal VR mod's tool, about one request a second)"

SYSTEM = """You write the bios that Oblivion's people speak from when a player talks to them out loud: one
paragraph each, 120 to 160 words and never more than 170, plain English prose, present tense, third
person, about the character as they are in the world of Cyrodiil. Who they are, where they live and
work, how they spend their days, the people round them and what they make of them, what they want,
their personality and quirks, how they talk (one short line of their own, if they have a telling
one). In the world's own terms only: never game mechanics (levels, stats, bugs, patches, the
Remastered edition, merchants' gold, "services", "quests", "the player"); someone who matters to
them is named, a stranger is a stranger.
Describe them as they are on the day a stranger first meets them, before anything a hero does
changes their lives: nothing that happens later, nothing that waits for someone to come and help,
and none of their lines from those later days. Only what they themselves know: a secret they keep
is theirs, so say they keep it; a truth about them they don't know (a parent, a fate) is left out.
Only what the page says: no names, places, events, fears or feelings it doesn't support, and no
flourish of your own at the end. Write the paragraph alone, nothing before or after it. These are
bios already written, in the style to keep:"""


def records(path):
    """Every NPC_ record of a plugin: (its masters, form id, editor id, name)."""
    data = open(path, "rb").read()
    masters, out = [], []

    def fields(body):
        at, got = 0, {}
        while at + 6 <= len(body):
            kind, size = body[at:at + 4], struct.unpack_from("<H", body, at + 4)[0]
            got.setdefault(kind, body[at + 6:at + 6 + size])
            if kind == b"MAST":
                masters.append(body[at + 6:at + 6 + size].rstrip(b"\0").decode("latin-1"))
            at += 6 + size
        return got

    def walk(at, end):
        while at < end:
            kind = data[at:at + 4]
            size = struct.unpack_from("<I", data, at + 4)[0]
            if kind == b"GRUP":
                label, group = data[at + 8:at + 12], struct.unpack_from("<i", data, at + 12)[0]
                if group != 0 or label in (b"NPC_",):
                    walk(at + 20, at + size)
                at += size
                continue
            flags, form = struct.unpack_from("<II", data, at + 8)
            body = data[at + 20:at + 20 + size]
            if kind in (b"TES4", b"NPC_"):
                if flags & 0x00040000:
                    body = zlib.decompress(body[4:])
                got = fields(body)
                if kind == b"NPC_":
                    name = got.get(b"FULL", b"").rstrip(b"\0").decode("latin-1").strip()
                    out.append((list(masters), form, got.get(b"EDID", b"").rstrip(b"\0").decode("latin-1"), name))
            at += 20 + size

    walk(0, len(data))
    return out


def people(game):
    """The game's named people, by name: the wiki namespaces their page may be in, likelier first.
    A name more than two NPCs share (by who they are, an override counting as the NPC it
    overrides; the developers' test NPCs not counted) is a kind of person, not a person. (The
    Isles' people may be in Oblivion.esm itself, the Isles' plugin a stub: theirs are the editor
    ids that start "SE".)"""
    named = {}
    for plugin, namespace in PLUGINS:
        path = os.path.join(game, "Data", plugin)
        if not os.path.exists(path):
            continue
        for masters, form, editor, name in records(path):
            if not re.search(r"[A-Za-z]", name) or editor.lower().startswith("test"):
                continue
            index = form >> 24
            owner = masters[index] if index < len(masters) else plugin
            entry = named.setdefault(name, {"isles": False, "npcs": set()})
            entry["isles"] |= namespace == "Shivering" or editor.startswith("SE")
            entry["npcs"].add((owner, form & 0xFFFFFF))
    return {name: ["Shivering", "Oblivion"] if e["isles"] else ["Oblivion", "Shivering"]
            for name, e in named.items() if len(e["npcs"]) <= 2}


def pages(titles, cache):
    """UESP's pages (title -> wikitext; None for one it hasn't), fifty a request, kept in `cache`."""
    path = os.path.join(cache, "pages.json")
    got = json.load(open(path, encoding="utf-8")) if os.path.exists(path) else {}
    wanted = [t for t in titles if t not in got]
    for i in range(0, len(wanted), 50):
        batch = wanted[i:i + 50]
        query = urllib.parse.urlencode({"action": "query", "prop": "revisions", "rvprop": "content", "rvslots": "main",
                                        "format": "json", "redirects": 1, "maxlag": 5, "titles": "|".join(batch)})
        request = urllib.request.Request("https://en.uesp.net/w/api.php?" + query, headers={"User-Agent": USER_AGENT})
        for attempt in range(5):
            try:
                reply = json.load(urllib.request.urlopen(request, timeout=60))
                break
            except Exception as e:
                print(f"  (UESP: {e}; again in {5 * (attempt + 1)} s)")
                time.sleep(5 * (attempt + 1))
        else:
            sys.exit("UESP isn't answering")
        # (a page asked for under another name, or a redirect's, is the page that name leads to)
        leads = {n["from"]: n["to"] for n in reply["query"].get("normalized", [])}
        leads.update({r["from"]: r["to"] for r in reply["query"].get("redirects", [])})
        text = {p["title"]: p["revisions"][0]["slots"]["main"]["*"]
                for p in reply["query"]["pages"].values() if "missing" not in p and p.get("revisions")}
        for title in batch:
            final = title
            for _ in range(5):
                final = leads.get(final, final)
            got[title] = text.get(final)
        os.makedirs(cache, exist_ok=True)
        json.dump(got, open(path, "w", encoding="utf-8"))
        print(f"  pages: {min(i + 50, len(wanted))} of {len(wanted)} asked")
        time.sleep(1.0)
    return got


def readable(wikitext):
    """A page's text as the LLM reads it best: its markup and the parts about the game (bugs, the
    Remastered edition, galleries) taken out, kept to the first 12,000 characters."""
    text = re.sub(r"(?s)<!--.*?-->|<ref[^>]*?/>|<ref.*?</ref>|\{\{\s*cleanup[^{}]*\}\}", "", wikitext, flags=re.I)
    text = re.sub(r"(?m)^\|\s*(level|health|magicka|resp|aggress|refid|baseid|image\w*|img\w*)\s*=.*\n", "", text)
    text = re.sub(r"(?ms)^==\s*(Bugs|Gallery|Oblivion Remastered|Notes)\s*==.*?(?=^==[^=]|\Z)", "", text)
    text = re.sub(r"\[\[(?:File|Image):[^\]]*\]\]", "", text)
    text = re.sub(r"\[\[(?:[^|\]]*\|)?([^\]]*)\]\]", r"\1", text)
    text = re.sub(r"'{2,}", "", text)
    return text[:12000]


def is_person(wikitext):
    return wikitext is not None and re.search(r"\{\{\s*(NPC Summary|Creature Summary|Oblivion NPC)", wikitext, re.I) is not None


def pointed_to(wikitext):
    """The pages a page that "may refer to" several things points to, in its order ("Umbra":
    the sword, the person)."""
    if wikitext is None or not re.search(r"(may|can) refer to", wikitext[:600]):
        return []
    return re.findall(r"\[\[((?:Oblivion|Shivering):[^|\]]+)", wikitext)


def llm():
    """The voice plugin's LLM: its URL, model, extras and key (from voice.toml)."""
    cfg = tomllib.load(open(VOICE_TOML, "rb"))["llm"]
    key = (cfg.get("api_key") or os.environ.get(cfg.get("api_key_env") or "", "")).strip()
    if not key:
        sys.exit(f"no LLM key: voice.toml's api_key, or {cfg.get('api_key_env')}")
    return cfg, key


def write_bio(name, namespace, wikitext, examples, cfg, key):
    shown = "\n\n".join(f"{n}:\n{b}" for n, b in examples)
    body = {
        "model": cfg.get("model"),
        "messages": [
            {"role": "system", "content": f"{SYSTEM}\n\n{shown}"},
            {"role": "user", "content": f"UESP's page on {name} ({namespace}):\n\n{readable(wikitext)}\n\nWrite {name}'s bio."},
        ],
        "temperature": 0.6,
        "max_tokens": 500,
        **cfg.get("extra", {}),
    }
    request = urllib.request.Request(cfg["url"], json.dumps(body).encode(), {"Content-Type": "application/json", "Authorization": f"Bearer {key}"})
    for attempt in range(4):
        try:
            reply = json.load(urllib.request.urlopen(request, timeout=120))
            text = reply["choices"][0]["message"]["content"].strip()
            return " ".join(text.split())
        except Exception as e:
            time.sleep(3 * (attempt + 1))
            error = e
    print(f"  {name}: the LLM failed ({error})")
    return None


def toml_entry(name, bio):
    wrapped = "\n".join(textwrap.wrap(bio.replace("'''", "' ' '"), 100))
    return f"\n[{json.dumps(name)}]\nbio = '''\n{wrapped}\n'''\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--game", default=DEFAULT_GAME)
    ap.add_argument("--cache", default=DEFAULT_CACHE)
    ap.add_argument("--limit", type=int, default=0, help="write at most this many (0: everyone)")
    ap.add_argument("--only", default="", help="these people only (names, comma-separated)")
    ap.add_argument("--jobs", type=int, default=12)
    args = ap.parse_args()

    done_path = os.path.join(args.cache, "bios.json")
    done = json.load(open(done_path, encoding="utf-8")) if os.path.exists(done_path) else {}
    have = tomllib.load(open(BIOS, "rb"))
    written = {n.lower() for n in have} | {a.lower() for e in have.values() for a in e.get("names", [])}
    written |= {n.lower() for n in done}
    examples = [(n, " ".join(e["bio"].split())) for n, e in have.items() if n not in done][:6]
    everyone = people(args.game)
    print(f"{len(everyone)} named people in the game's plugins ({len(written)} with a bio already)")
    if args.only:
        wanted = {n.strip() for n in args.only.split(",")}
        everyone = {n: ns for n, ns in everyone.items() if n in wanted}
    todo = sorted(n for n in everyone if n.lower() not in written)
    got = pages([f"{ns}:{n}" for n in todo for ns in everyone[n]], args.cache)
    # (their page: the first of their namespaces with a person's page in it, or the first person
    # a page of theirs that may refer to several things points to)
    choices = {n: [f"{ns}:{n}" for ns in everyone[n]] for n in todo}
    choices = {n: [t for c in titles for t in [c, *pointed_to(got.get(c))]] for n, titles in choices.items()}
    got = pages([t for titles in choices.values() for t in titles], args.cache)
    page = {n: next((t for t in titles if is_person(got.get(t))), None) for n, titles in choices.items()}
    todo = [n for n in todo if page[n]]
    if args.limit:
        todo = todo[:args.limit]
    print(f"{len(todo)} to write (people with a page of their own)")

    cfg, key = llm()
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        jobs = {pool.submit(write_bio, n, page[n].split(":")[0], got[page[n]], examples, cfg, key): n for n in todo if n not in done}
        for i, job in enumerate(concurrent.futures.as_completed(jobs), 1):
            name = jobs[job]
            if bio := job.result():
                done[name] = bio
            if i % 25 == 0 or i == len(jobs):
                json.dump(done, open(done_path, "w", encoding="utf-8"), ensure_ascii=False, indent=0)
                print(f"  bios: {i} of {len(jobs)}")
    json.dump(done, open(done_path, "w", encoding="utf-8"), ensure_ascii=False, indent=0)

    # (the file: its header and the bios written by hand as they are, then every bio written here,
    # by name)
    text = open(BIOS, encoding="utf-8").read()
    blocks = re.split(r'(?m)^(?=\["[^\n]*"\]\n)', text)
    kept = [b for b in blocks[1:] if json.loads(b.split("\n", 1)[0][1:-1]) not in done]
    kept = [b.split(WRITTEN)[0].rstrip("\n") + "\n" for b in kept]
    with open(BIOS, "w", encoding="utf-8", newline="\n") as f:
        f.write(blocks[0].split(WRITTEN)[0] + "\n".join(kept) + "\n" + WRITTEN + "\n")
        for name in sorted(done, key=str.lower):
            f.write(toml_entry(name, done[name]))
    tomllib.load(open(BIOS, "rb"))
    print(f"{len(done)} bios from {done_path} in {BIOS}")


if __name__ == "__main__":
    main()
