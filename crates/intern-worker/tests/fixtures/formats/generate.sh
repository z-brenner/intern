#!/bin/sh
# Regenerates the committed format fixtures from the flat-ODF sources in
# source/ with LibreOffice. See README.md for why these are committed rather
# than generated at test time.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# A private profile, so a desktop LibreOffice that is already running - or
# another conversion in parallel - neither blocks this one nor changes it.
convert() {
    soffice -env:UserInstallation="file://$work/profile" --headless \
        --convert-to "$1" --outdir "$work/out" "$2" >/dev/null
}

convert 'doc:MS Word 97' "$here/source/letter.fodt"
convert 'rtf:Rich Text Format' "$here/source/letter.fodt"
convert 'odt:writer8' "$here/source/letter.fodt"
convert 'docm:MS Word 2007 XML VBA' "$here/source/letter.fodt"
convert 'ppt:MS PowerPoint 97' "$here/source/deck.fodp"
convert 'odp:impress8' "$here/source/deck.fodp"
convert 'xls:MS Excel 97' "$here/source/ledger.fods"
convert 'xlsm:Calc MS Excel 2007 VBA XML' "$here/source/ledger.fods"
convert 'ods:calc8' "$here/source/ledger.fods"
# Field separator 44 (comma), text delimiter 34 (double quote), UTF-8 (76).
convert 'csv:Text - txt - csv (StarCalc):44,34,76' "$here/source/ledger.fods"
for file in letter.doc letter.rtf letter.odt letter.docm deck.ppt deck.odp \
    ledger.xls ledger.xlsm ledger.ods ledger.csv; do
    cp "$work/out/$file" "$here/$file"
done

# The same workbook saved with a password to open. Legacy Excel encryption
# lives inside the Workbook stream (a FilePass record), not in the
# EncryptionInfo streams an encrypted OOXML package carries, and
# --convert-to cannot set a password, so this one goes through UNO.
python3 "$here/save_with_password.py" "$work/profile-uno" \
    "$here/source/ledger.fods" "$here/ledger-encrypted.xls" 'MS Excel 97' intern
