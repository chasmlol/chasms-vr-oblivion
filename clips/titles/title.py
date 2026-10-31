"""A title in Oblivion's own lettering, as a transparent PNG for a video.

    python -I title.py "Character Customizer" out.png [--scale 6] [--native]
    python -I title.py "Automatic Zoom on Aim|for Recording" out.png     (two lines, centred)

The game's `Kingthings_Shadowed` font (what it writes over the world with: white letters in a soft
dark halo, baked into its atlas), laid out as the game lays text out (each glyph's space before
it, the glyph, its width and the space after; the baseline at the font's tallest ascent). Upscaled
for video: the letters and the halo are pulled apart from the atlas (its white is the letter, its
dark the halo), each enlarged smoothly, the letters' edges sharpened back to crisp (a smoothstep
round half coverage) and the halo left soft, then put back together.
"""
import argparse
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
import gamefont  # noqa: E402


def layout(glyphs, text):
    """Each glyph's (char, x) along the line, and the line's width (font pixels)."""
    x, out = 0.0, []
    for ch in text:
        g = glyphs[ord(ch) & 0xFF]
        x += g["before"]
        out.append((ch, x))
        x += g["width"] + g["after"]
    return out, x


def render(text, scale, native):
    height, glyphs, px, aw, ah = gamefont.font("Kingthings_Shadowed")
    atlas = np.frombuffer(px, np.uint8).reshape(ah, aw, 4).astype(np.float32) / 255.0
    ascent = max(g["top"] for g in glyphs)
    # (Lines split at "|", each centred, the game's line height apart.)
    lines = [layout(glyphs, line) for line in text.split("|")]
    width = max(w for _, w in lines)
    placed = [(ch, x + (width - w) / 2, i * height) for i, (line, w) in enumerate(lines) for ch, x in line]
    s = 1 if native else scale
    pad = 8
    W = int(np.ceil((width + 2 * pad) * s))
    H = int(np.ceil((height * len(lines) + 2 * pad) * s))
    letter = np.zeros((H, W), np.float32)
    halo = np.zeros((H, W), np.float32)
    for ch, x, y in placed:
        g = glyphs[ord(ch) & 0xFF]
        if g["width"] <= 0 or g["height"] <= 0:
            continue
        u0, v0, u1, v1 = g["uv"]
        r = atlas[int(round(v0 * ah)):int(round(v1 * ah)), int(round(u0 * aw)):int(round(u1 * aw))]
        lum = r[..., :3].mean(-1)
        a = r[..., 3]
        parts = [a * lum, a * (1.0 - lum)]
        gw, gh = int(round(g["width"] * s)), int(round(g["height"] * s))
        ox = int(round((pad + x) * s))
        oy = int(round((pad + y + ascent - g["top"]) * s))
        for k, (part, into) in enumerate(zip(parts, (letter, halo))):
            img = Image.fromarray(part.astype(np.float32), mode="F")
            if s != 1 or img.size != (gw, gh):
                img = img.resize((gw, gh), Image.Resampling.BICUBIC)
            p = np.clip(np.asarray(img, np.float32), 0.0, 1.0)
            if k == 0 and s > 1:
                # The letter's edge crisp again: coverage round a half pushed to its side.
                t = np.clip((p - 0.32) / 0.36, 0.0, 1.0)
                p = t * t * (3.0 - 2.0 * t)
            sl = into[oy:oy + gh, ox:ox + gw]
            np.maximum(sl, p[:sl.shape[0], :sl.shape[1]], out=sl)
    # White letters over the dark halo (straight alpha).
    alpha = letter + halo * (1.0 - letter)
    rgb = np.where(alpha[..., None] > 1e-4, (letter / np.maximum(alpha, 1e-4))[..., None], 0.0) * np.ones(3, np.float32)
    out = np.dstack([rgb, alpha])
    # Trimmed to what's drawn, with a margin.
    ys, xs = np.nonzero(alpha > 0.002)
    m = int(4 * s)
    out = out[max(ys.min() - m, 0):ys.max() + m + 1, max(xs.min() - m, 0):xs.max() + m + 1]
    return Image.fromarray((np.clip(out, 0.0, 1.0) * 255.0 + 0.5).astype(np.uint8), mode="RGBA")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("text")
    ap.add_argument("out")
    ap.add_argument("--scale", type=float, default=6.0)
    ap.add_argument("--native", action="store_true")
    a = ap.parse_args()
    img = render(a.text, a.scale, a.native)
    img.save(a.out)
    print(f"{a.out}: {img.size[0]}x{img.size[1]}")


if __name__ == "__main__":
    main()
