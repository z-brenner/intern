#!/usr/bin/env python3
"""Builds bench/assets/raster-fonts.json: anti-aliased glyph bitmaps that the
InternBench generator composites into scanned pages.

The generator has to draw text the way a scanner sees a printed page -
proportional glyphs from a real typeface - without a font rasteriser, so that
every machine produces the same bytes. This script is run once, by hand, on a
machine with ImageMagick and the Liberation fonts; its output is committed and
the generator only reads it.

The glyphs are rasterised from Liberation Sans, Liberation Sans Bold, and
Liberation Serif (SIL Open Font License 1.1). The atlas is a Modified Version
under that licence, so it carries none of the Reserved Font Names: it is
"InternBench Raster Sans" and "InternBench Raster Serif". See
bench/assets/OFL.txt.

Usage: python3 bench/tools/build-font-atlas.py [font-directory] > bench/assets/raster-fonts.json
"""
import base64
import json
import struct
import subprocess
import sys
import zlib

FONT_DIRECTORY = sys.argv[1] if len(sys.argv) > 1 else "/usr/share/fonts/truetype/liberation"
FACES = {
    "sans": "LiberationSans-Regular.ttf",
    "sans-bold": "LiberationSans-Bold.ttf",
    "serif": "LiberationSerif-Regular.ttf",
}
EM_PIXELS = 96
# Canvas: the glyph's origin sits MARGIN pixels in from the left, on a
# baseline BASELINE pixels down a HEIGHT-pixel cell.
MARGIN = 10
BASELINE = 90
HEIGHT = 116
CHARACTERS = [chr(code) for code in range(32, 127)] + list("§–—‘’“”•é©°½")


def tables(data):
    count = struct.unpack(">H", data[4:6])[0]
    found = {}
    for index in range(count):
        tag, _, offset, length = struct.unpack(">4sIII", data[12 + 16 * index: 28 + 16 * index])
        found[tag.decode("latin1")] = (offset, length)
    return found


def advances(path):
    """Advance width of every character, in font units, and units per em."""
    data = open(path, "rb").read()
    table = tables(data)
    head = table["head"][0]
    units_per_em = struct.unpack(">H", data[head + 18: head + 20])[0]
    hhea = table["hhea"][0]
    metrics_count = struct.unpack(">H", data[hhea + 34: hhea + 36])[0]
    hmtx = table["hmtx"][0]
    widths = [struct.unpack(">H", data[hmtx + 4 * i: hmtx + 4 * i + 2])[0] for i in range(metrics_count)]
    cmap = table["cmap"][0]
    subtables = struct.unpack(">H", data[cmap + 2: cmap + 4])[0]
    mapping = {}
    for index in range(subtables):
        platform, encoding, offset = struct.unpack(">HHI", data[cmap + 4 + 8 * index: cmap + 12 + 8 * index])
        start = cmap + offset
        if struct.unpack(">H", data[start: start + 2])[0] != 4 or (platform, encoding) not in ((3, 1), (0, 3)):
            continue
        segments = struct.unpack(">H", data[start + 6: start + 8])[0] // 2
        ends = struct.unpack(">%dH" % segments, data[start + 14: start + 14 + 2 * segments])
        base = start + 16 + 2 * segments
        starts = struct.unpack(">%dH" % segments, data[base: base + 2 * segments])
        deltas = struct.unpack(">%dh" % segments, data[base + 2 * segments: base + 4 * segments])
        range_base = base + 4 * segments
        offsets = struct.unpack(">%dH" % segments, data[range_base: range_base + 2 * segments])
        for segment in range(segments):
            for code in range(starts[segment], ends[segment] + 1):
                if code == 0xFFFF:
                    continue
                if offsets[segment] == 0:
                    glyph = (code + deltas[segment]) & 0xFFFF
                else:
                    address = range_base + 2 * segment + offsets[segment] + 2 * (code - starts[segment])
                    glyph = struct.unpack(">H", data[address: address + 2])[0]
                    if glyph:
                        glyph = (glyph + deltas[segment]) & 0xFFFF
                mapping.setdefault(code, glyph)
        break
    result = {}
    for character in CHARACTERS:
        glyph = mapping.get(ord(character), 0)
        result[character] = widths[min(glyph, len(widths) - 1)]
    return result, units_per_em


def raster(path, character, width):
    """The glyph as 8-bit coverage (0 blank, 255 ink) on a width x HEIGHT cell."""
    if character == " ":
        return bytes(width * HEIGHT)
    command = [
        "convert", "-size", "%dx%d" % (width, HEIGHT), "xc:white", "-font", path,
        "-pointsize", str(EM_PIXELS), "-density", "72", "-fill", "black",
        "-annotate", "+%d+%d" % (MARGIN, BASELINE), character, "-depth", "8", "gray:-",
    ]
    grey = subprocess.run(command, check=True, capture_output=True).stdout
    assert len(grey) == width * HEIGHT, (character, len(grey))
    return bytes(255 - value for value in grey)


def main():
    atlas = {
        "name": "InternBench raster fonts",
        "license": "SIL Open Font License 1.1 - see bench/assets/OFL.txt",
        "derived_from": "Liberation Sans, Liberation Sans Bold, Liberation Serif 2.x (Red Hat, Google)",
        "em_pixels": EM_PIXELS,
        "margin": MARGIN,
        "baseline": BASELINE,
        "height": HEIGHT,
        "faces": {},
    }
    for face, filename in FACES.items():
        path = "%s/%s" % (FONT_DIRECTORY, filename)
        widths, units = advances(path)
        glyphs = {}
        for character in CHARACTERS:
            advance = widths[character]
            cell = int(advance * EM_PIXELS / units) + 2 * MARGIN + 4
            coverage = raster(path, character, cell)
            # 4 bits of coverage is plenty for text that is about to be
            # downsampled, binarised, and scanned.
            packed = bytearray()
            for index in range(0, len(coverage), 2):
                high = coverage[index] >> 4
                low = coverage[index + 1] >> 4 if index + 1 < len(coverage) else 0
                packed.append((high << 4) | low)
            glyphs[character] = {
                "advance": advance,
                "cell": cell,
                "bitmap": base64.b64encode(zlib.compress(bytes(packed), 9)).decode("ascii"),
            }
        atlas["faces"][face] = {"units_per_em": units, "glyphs": glyphs}
    json.dump(atlas, sys.stdout, indent=1, sort_keys=True, ensure_ascii=False)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
