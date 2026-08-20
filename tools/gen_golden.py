#!/usr/bin/env python3
"""Generate golden test vectors for crates/ephemeris from Swiss Ephemeris' own
swetest CLI.

swetest is a separate program built from the same C library. Running the
reference through it rather than through our own bindings is what makes these
vectors an independent check: a mistake in our FFI layer, flag words, Ketu
derivation or sidereal setup shows up as a mismatch instead of being baked into
the expectation.

Usage:  python3 tools/gen_golden.py > crates/ephemeris/tests/golden.json
"""
import json
import math
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SWETEST = ROOT / "tools" / "swetest"
EPHE = ROOT / "src-tauri" / "resources" / "ephe"

# swetest planet selector -> our graha key. Ketu is absent by design: it has no
# body of its own and is asserted to be the node plus 180 degrees.
BODIES = [("0", "surya"), ("1", "chandra"), ("2", "budha"), ("3", "shukra"),
          ("4", "mangala"), ("5", "guru"), ("6", "shani")]

AYANAMSAS = {"lahiri": 1, "raman": 3, "krishnamurti": 5, "true_chitra": 27}

DATES = [
    (1900, 1, 1), (1950, 6, 15), (2000, 1, 1), (2024, 2, 29),
    (2026, 2, 28), (2026, 8, 20), (2027, 12, 31), (2100, 7, 4), (2399, 12, 31),
]

# Bengaluru, and Reykjavik for a high latitude where rise and set drift far apart.
PLACES = {
    "bengaluru": (77.5946, 12.9716, 920.0),
    "reykjavik": (-21.9426, 64.1466, 0.0),
}


def dms(text):
    """Parse swetest's ddd°mm\'ss.ssss" form into decimal degrees.

    swetest emits some columns as sexagesimal regardless of the decimal format
    letters, so this has to handle both."""
    text = text.strip()
    m = re.match(r"^(-?)\s*(\d+)\s*°\s*(\d+)\'\s*([\d.]+)", text)
    if not m:
        return float(text)
    sign = -1.0 if m.group(1) == "-" else 1.0
    return sign * (int(m.group(2)) + int(m.group(3)) / 60.0 + float(m.group(4)) / 3600.0)


def julian_day(y, m, d, hours):
    """Gregorian Julian Day, mirroring crates/ephemeris/src/julian.rs."""
    if m <= 2:
        y, m = y - 1, m + 12
    a = y // 100 if y >= 0 else -((-y + 99) // 100)
    a = math.floor(y / 100)
    b = 2 - a + math.floor(a / 4)
    return (math.floor(365.25 * (y + 4716)) + math.floor(30.6001 * (m + 1))
            + d + b - 1524.5 + hours / 24.0)


def run(args):
    out = subprocess.run([str(SWETEST), f"-edir{EPHE}", *args],
                         capture_output=True, text=True, check=True)
    return [ln for ln in out.stdout.splitlines() if ln.strip()]


def date_args(y, m, d):
    return [f"-b{d}.{m}.{y}", "-ut0:00"]


def positions():
    out = []
    for (y, m, d) in DATES:
        for name, mode in AYANAMSAS.items():
            for node_sel, node_key in (("t", "true"), ("m", "mean")):
                sel = "".join(p for p, _ in BODIES) + node_sel
                lines = run([*date_args(y, m, d), f"-p{sel}", f"-sid{mode}",
                             "-fJlbs", "-head", "-g,"])
                bodies = {}
                jd = None
                keys = [k for _, k in BODIES] + ["rahu"]
                for key, line in zip(keys, lines):
                    j, lon, lat, spd = (f.strip() for f in line.split(","))
                    jd = float(j)
                    bodies[key] = {"longitude": float(lon),
                                   "latitude": float(lat),
                                   "speed": float(spd)}
                out.append({"date": [y, m, d], "jd_ut": jd, "ayanamsa": name,
                            "node_type": node_key, "bodies": bodies})
    return out


def illumination():
    out = []
    for (y, m, d) in DATES:
        line = run([*date_args(y, m, d), "-p1", "-fJ+-*", "-head", "-g,"])[0]
        jd, angle, frac, elong = (dms(f) for f in line.split(","))
        out.append({"date": [y, m, d], "jd_ut": jd, "phase_angle": angle,
                    "fraction": frac, "elongation": elong})
    return out


def ayanamsa_values():
    """Ayanamsa as the difference between tropical and sidereal longitude.

    That difference is the definition of the ayanamsa and is exactly the
    quantity swe_calc_ut applies internally, so it is a stronger check than
    reading any single reporting function - it verifies the value the positions
    were actually computed with."""
    out = []
    for (y, m, d) in DATES:
        tropical = dms(run([*date_args(y, m, d), "-p0", "-fJl",
                            "-head", "-g,"])[0].split(",")[1])
        for name, mode in AYANAMSAS.items():
            line = run([*date_args(y, m, d), "-p0", f"-sid{mode}",
                        "-fJl", "-head", "-g,"])[0]
            jd, sidereal = (dms(f) for f in line.split(","))
            out.append({"date": [y, m, d], "jd_ut": jd, "ayanamsa": name,
                        "degrees": (tropical - sidereal) % 360.0})
    return out


RISE_LINE = re.compile(
    r"rise\s+(\S+)\s+([\d:.]+|-)\s+set\s+(\S+)\s+([\d:.]+|-)")


def clock_to_jd(datestr, timestr):
    """swetest prints rise and set as dd.mm.yyyy and hh:mm:ss.s in UT."""
    if datestr == "-" or timestr == "-":
        return None
    d, m, y = (int(v) for v in datestr.split("."))
    parts = [float(v) for v in timestr.split(":")]
    hours = parts[0] + parts[1] / 60.0 + (parts[2] if len(parts) > 2 else 0.0) / 3600.0
    return julian_day(y, m, d, hours)


def rise_set():
    """Rise and set falling inside one civil day, in UT.

    swetest pairs each set with a preceding rise and suppresses a rise that
    happens after that set (see call_rise_set in swetest.c). That is a display
    convention for a continuous stream, not an astronomical statement, and it
    does not match what a calendar needs. So the stream is read over several days
    with -n and every event is kept, then filtered to the target day. The values
    are still entirely swetest's."""
    out = []
    for place, (lon, lat, elev) in PLACES.items():
        for (y, m, d) in DATES:
            for sel, key in (("0", "surya"), ("1", "chandra")):
                lines = run([*date_args(y, m, d), f"-p{sel}", "-rise", "-n4",
                             f"-geopos{lon},{lat},{elev}"])
                start = julian_day(y, m, d, 0.0)
                rises, sets = [], []
                for ln in lines:
                    match = RISE_LINE.search(ln)
                    if not match:
                        continue
                    rd, rt, sd, st = match.groups()
                    for datestr, timestr, bucket in ((rd, rt, rises), (sd, st, sets)):
                        jd = clock_to_jd(datestr, timestr)
                        if jd is not None and start <= jd < start + 1.0:
                            bucket.append(jd)
                out.append({
                    "place": place, "date": [y, m, d], "graha": key,
                    "observer": {"longitude": lon, "latitude": lat, "elevation": elev},
                    "rise_jd": min(rises) if rises else None,
                    "set_jd": min(sets) if sets else None,
                })
    return out


def main():
    if not SWETEST.exists():
        sys.exit(f"swetest not built at {SWETEST}; see tools/README.md")
    doc = {
        "_note": "Generated by tools/gen_golden.py from swetest. Do not hand-edit.",
        "_library": "Swiss Ephemeris 2.10.03",
        "positions": positions(),
        "illumination": illumination(),
        "ayanamsa": ayanamsa_values(),
        "rise_set": rise_set(),
    }
    json.dump(doc, sys.stdout, indent=1)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
