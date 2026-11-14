# Format fixtures

Small documents in the legacy and open formats law and accounting offices
actually file, each carrying the same one-page engagement letter: a date
(March 3, 2025) and two parties (Harbor Lantern Accounting LLP and Juniper
Ridge Holdings Inc.), with a name that needs more than ASCII (Lena Müller).
`tests/formats.rs` reads every one through the reader the worker routes it to
and checks that all four facts come out.

| File | Written by LibreOffice as | From |
| --- | --- | --- |
| `letter.doc` | Word 97-2003 | `source/letter.fodt` |
| `letter.rtf` | Rich Text Format | `source/letter.fodt` |
| `letter.odt` | OpenDocument Text | `source/letter.fodt` |
| `letter.docm` | Word 2007 macro-enabled | `source/letter.fodt` |
| `deck.ppt` | PowerPoint 97-2003 | `source/deck.fodp` |
| `deck.odp` | OpenDocument Presentation | `source/deck.fodp` |
| `ledger.xls` | Excel 97-2003 | `source/ledger.fods` |
| `ledger.xlsm` | Excel 2007 macro-enabled | `source/ledger.fods` |
| `ledger.ods` | OpenDocument Spreadsheet | `source/ledger.fods` |
| `ledger.csv` | CSV: comma, double quotes, UTF-8 | `source/ledger.fods` |
| `ledger-encrypted.xls` | Excel 97-2003, password `intern` | `source/ledger.fods` |
| `dated.xls` | Excel 97-2003 | `source/dated.fods` |
| `dated.xlsm` | Excel 2007 macro-enabled | `source/dated.fods` |
| `dated.ods` | OpenDocument Spreadsheet | `source/dated.fods` |

Two fax pages stand apart from the letter: `fax-msb.tif` and `fax-lsb.tif`
are one Group 4 page stored most and least significant bit first (TIFF
FillOrder 1 and 2), as some scanning systems stored litigation productions.
libtiff writes both from one image, so they decode to the same pixels; a
decoder that ignores FillOrder reads the second backwards. `fax.py`
regenerates them, with Pillow built against libtiff; these were written by Pillow 12.3 with
libtiff 4.7.1.
`src/extract.rs` and `tests/image_files.rs` read them.

The ledger holds a real date cell (2025-04-14) and a formula
(`=B7+1500`), so the workbook readers show that a stored date comes out as
the date it means and a formula as the value it last calculated.

`dated.*` is another document: a quotation whose dates are worked out by
formulas that call `TODAY` or `NOW` (alone, with arithmetic, inside `IF`,
`TEXT`, `YEAR` and `EOMONTH`, and filled down three rows, which the binary
format stores as one shared formula), beside a typed date, formulas that
call neither, and text that only says "TODAY()". LibreOffice works the
formulas out as it converts, so each file holds the day it was generated
(2026-11-14 for the ones committed), which is the point: that day is the
file's and not the quotation's. `tests/formats.rs` checks which cells each
workbook reader marks as worked out by such a formula.

## Regenerating

The sources in `source/` are flat (single-file, uncompressed XML)
OpenDocument files, so they diff and review as text. Regenerate the binaries
from them with:

```sh
sh generate.sh
```

It needs LibreOffice (`soffice`) and, for the password-protected workbook,
LibreOffice's Python UNO bridge: `soffice --convert-to` cannot set a
password for the binary Office filters, so `save_with_password.py` drives
LibreOffice over UNO to store it. Each conversion runs with a private
LibreOffice profile, so a desktop LibreOffice that is already open neither
blocks it nor changes it. These fixtures were produced with LibreOffice
24.2.7.2.

## Why they are committed

The binaries are committed rather than generated at test time because
LibreOffice is not part of the build or CI toolchain, and because the point
of them is to be files a real office suite wrote, quirks included. The zip
containers embed timestamps, so regenerating changes their bytes even when
nothing else changes. `deck.ppt` is large on disk (about 450 KB) because
LibreOffice pads the compound file; it compresses to under 30 KB in git.
