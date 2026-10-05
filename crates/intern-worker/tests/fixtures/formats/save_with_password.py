"""Saves a document through LibreOffice with a password to open.

`soffice --convert-to` has no way to set a document password for the binary
Office filters, so this drives LibreOffice over UNO instead:

    python3 save_with_password.py <profile-dir> <source> <target> <filter> <password>

Run by generate.sh; needs the python3-uno bridge LibreOffice ships with.
"""

import os
import subprocess
import sys
import time

import uno
from com.sun.star.beans import PropertyValue


def prop(name, value):
    result = PropertyValue()
    result.Name = name
    result.Value = value
    return result


def main():
    profile, source, target, filter_name, password = sys.argv[1:6]
    pipe = "intern_fixture_%d" % os.getpid()
    office = subprocess.Popen(
        [
            "soffice",
            "-env:UserInstallation=" + uno.systemPathToFileUrl(profile),
            "--headless",
            "--invisible",
            "--norestore",
            "--accept=pipe,name=%s;urp;" % pipe,
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        local = uno.getComponentContext()
        resolver = local.ServiceManager.createInstanceWithContext(
            "com.sun.star.bridge.UnoUrlResolver", local
        )
        context = None
        for _ in range(100):
            try:
                context = resolver.resolve(
                    "uno:pipe,name=%s;urp;StarOffice.ComponentContext" % pipe
                )
                break
            except Exception:
                time.sleep(0.2)
        if context is None:
            raise SystemExit("LibreOffice did not start")
        desktop = context.ServiceManager.createInstanceWithContext(
            "com.sun.star.frame.Desktop", context
        )
        document = desktop.loadComponentFromURL(
            uno.systemPathToFileUrl(os.path.abspath(source)),
            "_blank",
            0,
            (prop("Hidden", True),),
        )
        document.storeToURL(
            uno.systemPathToFileUrl(os.path.abspath(target)),
            (prop("FilterName", filter_name), prop("Password", password)),
        )
        document.close(True)
        try:
            desktop.terminate()
        except Exception:
            # Terminating drops the bridge this call travels over.
            pass
    finally:
        try:
            office.wait(timeout=30)
        except subprocess.TimeoutExpired:
            office.kill()


if __name__ == "__main__":
    main()
