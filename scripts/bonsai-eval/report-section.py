"""Replace the section starting at a '## '/'### ' heading (up to the next heading of the same
or higher level) in REPORT.md with stdin; append if absent. Usage: section.py REPORT HEADING < text"""
import sys, re
path, heading = sys.argv[1], sys.argv[2]
new = sys.stdin.read().rstrip("\n") + "\n\n"
s = open(path).read()
level = len(heading) - len(heading.lstrip("#"))
lines = s.splitlines(keepends=True)
start = next((i for i, l in enumerate(lines) if l.rstrip("\n") == heading), None)
if start is None:
    s = s.rstrip("\n") + "\n\n" + new
else:
    end = len(lines)
    for j in range(start + 1, len(lines)):
        m = re.match(r"^(#+) ", lines[j])
        if m and len(m.group(1)) <= level:
            end = j
            break
    s = "".join(lines[:start]) + new + "".join(lines[end:])
open(path, "w").write(s)
