"""Writes fax-msb.tif and fax-lsb.tif: one page of Group 4 fax, stored most and least significant
bit first (TIFF FillOrder 1 and 2). libtiff encodes both, so the two decode to the same pixels;
a decoder that ignores FillOrder reads the second backwards. Needs Pillow built with libtiff.

    python3 fax.py
"""
import os

from PIL import Image, ImageDraw, TiffImagePlugin

HERE = os.path.dirname(os.path.abspath(__file__))
TiffImagePlugin.WRITE_LIBTIFF = True
page = Image.new("1", (320, 96), 1)
draw = ImageDraw.Draw(page)
draw.text((10, 10), "INVOICE 1882", fill=0)
draw.rectangle((10, 40, 300, 80), outline=0)
page.save(os.path.join(HERE, "fax-msb.tif"), compression="group4")
page.save(os.path.join(HERE, "fax-lsb.tif"), compression="group4", tiffinfo={266: 2})
