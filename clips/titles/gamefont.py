"""Oblivion's own UI pieces, read-only from the install's BSAs: a font (`fonts\\<name>.fnt` and its
atlas `<name>_0_lod_a.tex`) and menu textures. BSA v103: header, folder records (hash, count,
offset), then per folder its name (a bstring) and file records (hash, size, offset), then the file
names; a file is zlib (its size first) when the archive's compressed flag and the size's 0x40000000
bit differ."""
import struct
import zlib
from pathlib import Path

DATA = Path(r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data")


def bsa_files(path):
    """{lower-case path: (offset, size, compressed)} for one archive."""
    b = path.read_bytes()
    magic, ver, off, flags, nfolders, nfiles, _, _, _ = struct.unpack_from("<4sIIIIIIII", b, 0)
    assert magic == b"BSA\0" and ver == 103, (path, magic, ver)
    folders = [struct.unpack_from("<QII", b, off + 16 * i) for i in range(nfolders)]
    at = off + 16 * nfolders
    entries = []
    for _, count, _ in folders:
        n = b[at]
        name = b[at + 1:at + n].decode("latin1")
        at += 1 + n
        for k in range(count):
            _, size, foff = struct.unpack_from("<QII", b, at + 16 * k)
            entries.append((name, size, foff))
        at += 16 * count
    names = b[at:].split(b"\0")
    out = {}
    for (folder, size, foff), fname in zip(entries, names):
        comp = bool(flags & 4) != bool(size & 0x40000000)
        out[f"{folder}\\{fname.decode('latin1')}".lower()] = (foff, size & 0x3FFFFFFF, comp)
    return b, out


_ARCHIVES = None


def read(path):
    """A file's bytes from whichever archive has it."""
    global _ARCHIVES
    if _ARCHIVES is None:
        _ARCHIVES = [bsa_files(DATA / n) for n in ("Oblivion - Misc.bsa", "Oblivion - Textures - Compressed.bsa")]
    key = path.lower().replace("/", "\\")
    for b, files in _ARCHIVES:
        if key in files:
            off, size, comp = files[key]
            raw = b[off:off + size]
            return zlib.decompress(raw[4:]) if comp else raw
    raise KeyError(path)


def font(name):
    """(line height, glyphs[256] as dicts, atlas RGBA bytes, atlas w, h) of `fonts\\<name>.fnt`."""
    data = read(f"fonts\\{name}.fnt")
    height = struct.unpack_from("<f", data, 0)[0]
    glyphs = []
    for c in range(256):
        g = struct.unpack_from("<14f", data, 0x128 + 56 * c)
        glyphs.append({"uv": (g[1], g[2], g[7], g[8]), "width": g[9], "height": g[10], "before": g[11], "after": g[12], "top": g[13]})
    tex = read(f"fonts\\{name}_0_lod_a.tex")
    w, h = struct.unpack_from("<II", tex, 0)
    return height, glyphs, tex[8:8 + w * h * 4], w, h
