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

The ledger holds a real date cell (2025-04-14) and a formula
(`=B7+1500`), so the workbook readers show that a stored date comes out as
the date it means and a formula as the value it last calculated.

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
