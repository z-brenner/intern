"""Sample RSS of processes by command name every 100 ms until SIGTERM/SIGINT.

python3 -I rsssampler.py OUT.json intern-worker,tesseract,llama-server,intern-bench
Writes per-name peak RSS (MiB, summed across same-name processes at one instant),
peak of the combined total, and a coarse timeline (1 s resolution).
"""
import json
import os
import signal
import sys
import time

out, names = sys.argv[1], sys.argv[2].split(",")
peaks = {n: 0.0 for n in names}
peak_total = 0.0
timeline = []
running = True


def handle(*_):
    global running
    running = False


signal.signal(signal.SIGTERM, handle)
signal.signal(signal.SIGINT, handle)
t0 = time.time()
last = -1
while running:
    now = {n: 0.0 for n in names}
    for pid in os.listdir("/proc"):
        if not pid.isdigit():
            continue
        try:
            with open(f"/proc/{pid}/comm") as f:
                comm = f.read().strip()
            match = next((n for n in names if comm.startswith(n[:15])), None)
            if not match:
                continue
            with open(f"/proc/{pid}/status") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        now[match] += int(line.split()[1]) / 1024.0
                        break
        except OSError:
            continue
    for n in names:
        peaks[n] = max(peaks[n], now[n])
    total = sum(now.values())
    peak_total = max(peak_total, total)
    sec = int(time.time() - t0)
    if sec != last:
        timeline.append([sec] + [round(now[n], 1) for n in names])
        last = sec
    time.sleep(0.1)
with open(out, "w") as f:
    json.dump({"names": names, "peak_mib": peaks, "peak_total_mib": peak_total,
               "timeline_cols": ["s"] + names, "timeline": timeline}, f)
