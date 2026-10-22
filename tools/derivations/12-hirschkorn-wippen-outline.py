"""Hirschkorn's wippen assembly (Boston GP-178, note 52), traced from his Figure A.1.

Sources: Hirschkorn 2004, MASc thesis. Figure A.1, 'Defined Points on Action
Components' (printed page 68): the positions of the markers and of the
outlines of the wippen, the jack and the repetition lever in pixels of the page
rendered at 600 dpi, with how they were read, in
data/params/hirschkorn-wippen-outline.json. His printed coordinates (Tables
A.1, A.2, A.4 to A.6) from data/params/hirschkorn-boston-gp178-note52.json,
whose frame convention puts every point in its body's frame, the wippen's
origin at its pivot Pg2, the jack's at Pw2 and the lever's at Pw3, both pins
of the wippen. The borrowed balance bearing his profile's frame stands on, from
data/profiles/hirschkorn.json. His key between its stops on the real felts,
from tools/derivations/11-hirschkorn-key-outline.py, and his action assembled
at rest on his rounds, from tools/derivations/10-hirschkorn-assembled-at-rest.py,
both imported by their paths. Each file is read by a path relative to this
one: no value of his, no pixel and no number of the profile is typed here.

Prints:
  1. the three similarities that carry the figure's pixels into each body's
     frame, each fitted by least squares on the markers whose printed
     coordinates are points of that body: Pg2, Pw2 and Pw3 for the wippen;
     the jack's pin, the center of the arc that rounds the top of its back
     face (Pj2) and the center of the arc of its toe's end (Pj3); the lever's
     pin and the center of the arc along its top (Pr2);
  2. the residuals: at the fitted points, and at the other markers, which are
     the trace's uncertainty: Pw4, Pw6, Pj5 and Pr5 across their printed
     lines, the markers drawn on circles against their circles, the drawn arcs'
     radii against the printed ones; and the center-of-mass markers against
     Table A.1, which governs the masses;
  3. the outlines in each body's frame, as the bench draws them, and the
     numbers that place the heel and its cloth, each beside the value the
     outline record gives; exit status 1 where the two differ at the digits
     the record is written to, or where the residuals differ from the ones it
     keeps;
  4. the jack's and the repetition lever's rest angles on the wippen, closing
     contacts 5 and 6 on the printed points;
  5. the wippen's rest with the key on its back rail cloth, the key between
     the real felts of his profile, closing contact 1, against script 10's
     rest on his rounds;
  6. the body's long top edge, and the figure's drawn x axis, against the
     fitted frame's x axis;
  7. the assembly's mass and center at rest, Table A.1's three parts with the
     jack and the lever at the record's rest angles; the center pin's height
     above the keybed on the borrowed balance bearing, and the heel's height
     below the jack's pin, each against the value his profile states by its
     rule, exit status 1 where they differ at the 0.1 mm it states them to;
     and the wippen weight,
     the assembly's static moment over the arm of the capstan's mark, under
     each candidate level: (a) the body's long top edge horizontal, and (b)
     the center pin and the capstan's mark at one height.

A material outline runs counter-clockwise and its arcs with it, so an arc that
is concave from the material, a cove, runs clockwise: the record gives it as
the eight chords the bench draws an arc with, its ends and the points that
cut it into eight equal turns. The heel is given by numbers, since the bench
moves it as one piece: its sides straight from its lower corners up to the
body's bottom, the cove of its back side taken as its chord.

Pure arithmetic on the standard library: math, json, sys, pathlib and
importlib. Run with python3 -I.
"""
import importlib.util
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
TRACE = json.loads((ROOT / "data" / "params" / "hirschkorn-wippen-outline.json").read_text(encoding="utf-8"))
HK = json.loads((ROOT / "data" / "params" / "hirschkorn-boston-gp178-note52.json").read_text(encoding="utf-8"))
PROFILE = json.loads((ROOT / "data" / "profiles" / "hirschkorn.json").read_text(encoding="utf-8"))

CHORDS = 8  # the pieces the bench draws an arc in: two to its three halvings


def script(name, file):
    spec = importlib.util.spec_from_file_location(name, HERE / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


KEY = script("key_outline", "11-hirschkorn-key-outline.py")
ASSEMBLED = script("assembled_at_rest", "10-hirschkorn-assembled-at-rest.py")


def mm(record):
    """A printed record's value in millimeters."""
    scale = {"m": 1000.0, "mm": 1.0}[record["unit"]]
    return record["value"] * scale


def printed(body, name):
    """A printed point (x, y) in millimeters, in its body's frame."""
    point = HK["geometry"][body]["points"][name]
    return (mm(point["x"]), mm(point["y"]))


def radius(body, name):
    return mm(HK["geometry"][body]["points"][name]["radius"])


def unit(v):
    length = math.hypot(*v)
    return (v[0] / length, v[1] / length)


def normal(body, name):
    """A printed line's normal, made unit: the line is a direction, and the
    three decimals it is printed to leave it 15 ppm off unit length."""
    line = HK["geometry"][body]["points"][name]["normal"]
    return unit((line["x"]["value"], line["y"]["value"]))


def minus(p, q):
    return (p[0] - q[0], p[1] - q[1])


def add(p, q):
    return (p[0] + q[0], p[1] + q[1])


def dot(p, q):
    return p[0] * q[0] + p[1] * q[1]


def cross(p, q):
    return p[0] * q[1] - p[1] * q[0]


def rotated(t, v):
    c, s = math.cos(t), math.sin(t)
    return (c * v[0] - s * v[1], s * v[0] + c * v[1])


def circle(a, b, c):
    """The circle through three points: its center and its radius."""
    d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]))
    sa, sb, sc = dot(a, a), dot(b, b), dot(c, c)
    center = ((sa * (b[1] - c[1]) + sb * (c[1] - a[1]) + sc * (a[1] - b[1])) / d,
              (sa * (c[0] - b[0]) + sb * (a[0] - c[0]) + sc * (b[0] - a[0])) / d)
    return center, math.hypot(*minus(a, center))


def bisect(f, lo, hi):
    f_lo = f(lo)
    if (f(hi) > 0) == (f_lo > 0):
        raise SystemExit("no root in the bracket")
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        f_mid = f(mid)
        if (f_mid > 0) == (f_lo > 0):
            lo, f_lo = mid, f_mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def solve(a, b):
    """Gaussian elimination with partial pivoting on a square system."""
    size = len(b)
    rows = [list(a[i]) + [b[i]] for i in range(size)]
    for col in range(size):
        piv = max(range(col, size), key=lambda i: abs(rows[i][col]))
        rows[col], rows[piv] = rows[piv], rows[col]
        for i in range(size):
            if i != col:
                k = rows[i][col] / rows[col][col]
                for j in range(col, size + 1):
                    rows[i][j] -= k * rows[col][j]
    return [rows[i][size] / rows[i][i] for i in range(size)]


class Similarity:
    """X = a x + b y + c, Y = b x - a y + d: a turn, a scale and a shift, the
    page's y running down and the body's up. Least squares on its pairs of
    pixels and printed points."""

    def __init__(self, pairs):
        rows, rhs = [], []
        for (x, y), (big_x, big_y) in pairs:
            rows += [[x, y, 1.0, 0.0], [-y, x, 0.0, 1.0]]
            rhs += [big_x, big_y]
        normal_eq = [[sum(r[i] * r[j] for r in rows) for j in range(4)] for i in range(4)]
        right = [sum(r[i] * h for r, h in zip(rows, rhs)) for i in range(4)]
        self.a, self.b, self.c, self.d = solve(normal_eq, right)
        self.pairs = pairs

    def __call__(self, point):
        x, y = point
        return (self.a * x + self.b * y + self.c, self.b * x - self.a * y + self.d)

    def scale(self):
        return math.hypot(self.a, self.b)

    def turn_deg(self):
        return math.degrees(math.atan2(self.b, self.a))


def marker(body, name):
    return tuple(TRACE["bodies"][body]["markers"][name]["px"])


def path(body, outline):
    """An outline's path, as ('line', end) and ('arc', through, end) stretches
    after its start, in pixels."""
    raw = TRACE["bodies"][body]["outlines"][outline]["path"]
    start = tuple(raw[0])
    stretches = []
    for point in raw[1:]:
        if isinstance(point, dict):
            stretches.append(("arc", tuple(point["through"]), tuple(point["to"])))
        else:
            stretches.append(("line", tuple(point)))
    return start, stretches


def arc_circle(body, outline, index):
    """The circle of the index-th stretch of an outline's path, an arc, in
    pixels, with its start."""
    start, stretches = path(body, outline)
    before = start if index == 0 else stretches[index - 1][-1]
    _, through, end = stretches[index]
    return circle(before, through, end)


def toe_circle():
    """The jack toe's end: two arcs on one circle, read through the first's
    start, their joint and the second's end."""
    start, stretches = path("jack", "jack")
    return circle(start, stretches[0][2], stretches[1][2])


PG2, PW2, PW3 = (0.0, 0.0), printed("wippen", "Pw2"), printed("wippen", "Pw3")
WIPPEN = Similarity([(marker("wippen", "Pg2"), PG2), (marker("wippen", "Pw2"), PW2),
                     (marker("wippen", "Pw3"), PW3)])
PJ2, PJ3 = printed("jack", "Pj2"), printed("jack", "Pj3")
JACK_TOP = arc_circle("jack", "jack", 6)
JACK_TOE = toe_circle()
JACK = Similarity([(marker("jack", "pin"), (0.0, 0.0)), (JACK_TOP[0], PJ2), (JACK_TOE[0], PJ3)])
PR2 = printed("repetition_lever", "Pr2")
LEVER_TOP = arc_circle("repetition_lever", "repetition_lever", 1)
LEVER = Similarity([(marker("repetition_lever", "pin"), (0.0, 0.0)), (LEVER_TOP[0], PR2)])
FRAMES = {"wippen": WIPPEN, "jack": JACK, "repetition_lever": LEVER}


def outline_mm(body, outline):
    """An outline in its body's frame, millimeters, counter-clockwise: a list
    of stretches, ('line', start) or ('arc', start, center, radius), each
    ending where the next starts and the last where the first starts; a cove
    as its eight chords."""
    to = FRAMES[body]
    start, stretches = path(body, outline)
    points = [to(start)]
    pieces = []
    for s in stretches:
        if s[0] == "line":
            pieces.append(("line", points[-1], to(s[1])))
        else:
            pieces.append(("arc", points[-1], to(s[1]), to(s[2])))
        points.append(to(s[-1]))
    if math.hypot(*minus(points[-1], points[0])) > 1e-9:
        raise SystemExit(f"the {outline} does not end where it starts")
    twice_area = sum(cross(piece[1], piece[-1]) for piece in pieces)
    if twice_area < 0:
        pieces = [("line", piece[2], piece[1]) if piece[0] == "line" else ("arc", piece[3], piece[2], piece[1])
                  for piece in reversed(pieces)]
    out = []
    for piece in pieces:
        if piece[0] == "line":
            out.append(("line", piece[1]))
            continue
        _, a, through, b = piece
        c, r = circle(a, through, b)
        if cross(minus(a, c), minus(through, c)) > 0:
            out.append(("arc", a, c, r))
            continue
        # A cove: clockwise about its center, cut into equal turns.
        ua, ub = minus(a, c), minus(b, c)
        sweep = math.atan2(cross(ua, ub), dot(ua, ub))
        start_angle = math.atan2(ua[1], ua[0])
        for k in range(CHORDS):
            t = start_angle + sweep * k / CHORDS
            out.append(("line", a if k == 0 else (c[0] + r * math.cos(t), c[1] + r * math.sin(t))))
    return out


def as_record(stretches):
    """An outline as the record writes it, to 0.01 mm."""
    def r2(v):
        return round(v + 0.0, 2) + 0.0

    out = []
    for s in stretches:
        if s[0] == "line":
            out.append({"line": [r2(s[1][0]), r2(s[1][1])]})
        else:
            out.append({"arc": [r2(s[1][0]), r2(s[1][1])], "center": [r2(s[2][0]), r2(s[2][1])],
                        "radius": r2(s[3])})
    return out


def heel_point(name):
    point = TRACE["bodies"]["wippen"]["outlines"]["heel"]["points"][name]
    return WIPPEN(tuple(point["px"] if "px" in point else point["to"]))


PW5, R5 = printed("wippen", "Pw5"), radius("wippen", "Pw5")


def heel():
    """The numbers that place the heel and its cloth, millimeters, each read
    along the wippen's x from Pw5's center but the heel's top, a height."""
    cloth_back, cloth_front = heel_point("cloth_back_outer"), heel_point("cloth_front_outer")
    inner_back, inner_front = heel_point("cloth_back_inner"), heel_point("cloth_front_inner")
    thickness = ((math.hypot(*minus(cloth_back, PW5)) - math.hypot(*minus(inner_back, PW5)))
                 + (math.hypot(*minus(cloth_front, PW5)) - math.hypot(*minus(inner_front, PW5)))) / 2
    back_top, front_top = heel_point("back_top"), heel_point("front_top")
    return {
        "heel_top": (back_top[1] + front_top[1]) / 2,
        "heel_back_top": PW5[0] - back_top[0],
        "heel_front_top": front_top[0] - PW5[0],
        "heel_back": PW5[0] - heel_point("back_corner")[0],
        "heel_front": heel_point("front_corner")[0] - PW5[0],
        "heel_cloth_back": PW5[0] - cloth_back[0],
        "heel_cloth_front": cloth_front[0] - PW5[0],
        "heel_cloth_thickness": thickness,
    }


def on_heel_circle(reach):
    """The point of the heel cloth's outer face `reach` along x from Pw5's
    center, under it."""
    return (PW5[0] + reach, PW5[1] - math.sqrt(R5 * R5 - reach * reach))


def heel_lower_corners(h):
    """Where the heel's lower corners stand, on the line through the cloth's
    two ends on its outer face, as the figure draws its bottom."""
    back, front = on_heel_circle(-h["heel_cloth_back"]), on_heel_circle(h["heel_cloth_front"])
    slope = (front[1] - back[1]) / (front[0] - back[0])

    def at(x):
        return (x, back[1] + slope * (x - back[0]))

    return at(PW5[0] - h["heel_back"]), at(PW5[0] + h["heel_front"])


def center_pin_diameter():
    start, stretches = path("wippen", "center_pin")
    c, r = circle(WIPPEN(start), WIPPEN(stretches[0][1]), WIPPEN(stretches[0][2]))
    return 2.0 * r


def derived():
    """The outline record's E values, by name: outlines as the record writes
    them, the rest as numbers."""
    values = {
        "wippen_outline": as_record(outline_mm("wippen", "body")),
        "jack_outline": as_record(outline_mm("jack", "jack")),
        "repetition_lever_outline": as_record(outline_mm("repetition_lever", "repetition_lever")),
        "center_pin_diameter": center_pin_diameter(),
    }
    values.update(heel())
    values["jack_rest_angle"], values["repetition_lever_rest_angle"] = rest_angles()
    return values


def rest_angles():
    """The jack's and the lever's angles on the wippen, radians, counter-
    clockwise from its frame, that close contact 5 (Pw4's line on the wippen
    against Pj4's circle on the jack) and contact 6 (Pw6's line against Pr3's
    circle on the lever). Each has two roots; the one his action assembles at
    (script 10) is bracketed."""
    pw4, n4 = printed("wippen", "Pw4"), normal("wippen", "Pw4")
    pj4, r4 = printed("jack", "Pj4"), radius("jack", "Pj4")
    pw6, n6 = printed("wippen", "Pw6"), normal("wippen", "Pw6")
    pr3, r3 = printed("repetition_lever", "Pr3"), radius("repetition_lever", "Pr3")
    jack = bisect(lambda t: dot(minus(add(PW2, rotated(t, pj4)), pw4), n4) - r4, 0.0, 0.6)
    lever = bisect(lambda t: dot(minus(add(PW3, rotated(t, pr3)), pw6), n6) - r3, 0.0, 0.6)
    return jack, lever


def table_a1(body):
    r = HK["mass_properties"]["bodies"][body]
    cm = r["center_of_mass"]
    return r["mass"]["value"] * 1000.0, (mm(cm["x"]), mm(cm["y"]))


def assembly(jack_angle, lever_angle):
    """The assembly's mass, grams, and its center in the wippen's frame,
    millimeters: Table A.1's wippen, jack and repetition lever, the jack and
    the lever at their rest angles about their pins."""
    parts = [table_a1("wippen")]
    jack_mass, jack_center = table_a1("jack")
    parts.append((jack_mass, add(PW2, rotated(jack_angle, jack_center))))
    lever_mass, lever_center = table_a1("repetition_lever")
    parts.append((lever_mass, add(PW3, rotated(lever_angle, lever_center))))
    mass = sum(part for part, _ in parts)
    center = (sum(part * at[0] for part, at in parts) / mass, sum(part * at[1] for part, at in parts) / mass)
    return mass, center


def named(records, name):
    return next(r for r in records if r["name"] == name)


def same(x, recorded, digits):
    def text(v):
        return json.dumps(v, sort_keys=True) if isinstance(v, list) else f"{v:.{digits}f}"

    return recorded is not None and text(x) == text(recorded)


def trace_residuals():
    """The trace's residuals, mm, by body and by what each compares, and the
    lines that say them: at the fitted points, the distance between where the
    fit puts each and its printed point; at a printed line, the distance
    across it; at a marker drawn on a printed circle, its distance off the
    circle; at a drawn arc a printed circle stands for, the difference of
    their radii, or for the heel cloth's outer face the larger of that and
    the distance between their centers."""
    residuals = {body: {} for body in FRAMES}
    lines = []

    def fitted(body, labels):
        frame = FRAMES[body]
        for label, (px, point) in zip(labels, frame.pairs):
            t = frame(px)
            r = minus(t, point)
            residuals[body][label] = math.hypot(*r)
            lines.append(f"  {label:4s} traced ({t[0]:7.2f}, {t[1]:6.2f})  printed ({point[0]:7.2f}, {point[1]:6.2f})"
                         f"  residual ({r[0]:+.3f}, {r[1]:+.3f}), {residuals[body][label]:.2f}   [fitted]")

    def across(body, label, name, at):
        point, across_it = printed(body, name), normal(body, name)
        r = minus(at, point)
        residuals[body][label] = abs(dot(r, across_it))
        lines.append(f"  {label:4s} at ({at[0]:7.2f}, {at[1]:6.2f}), across {name}'s printed line {dot(r, across_it):+.2f},"
                     f" along it {dot(r, (-across_it[1], across_it[0])):+.2f}")

    def on_circle(body, label, name, at):
        c, r = printed(body, name), radius(body, name)
        residuals[body][label] = abs(math.hypot(*minus(at, c)) - r)
        lines.append(f"  {label:4s} at ({at[0]:7.2f}, {at[1]:6.2f}), {math.hypot(*minus(at, c)) - r:+.2f} off"
                     f" {name}'s printed circle of {r:.1f} mm about ({c[0]:.1f}, {c[1]:.1f})")

    def drawn_radius(body, label, name, r_px):
        r = r_px * FRAMES[body].scale()
        residuals[body][label] = abs(r - radius(body, name))
        lines.append(f"  {label:4s} the drawn arc's radius {r:.2f} mm, {name}'s printed {radius(body, name):.1f}")

    lines.append("  the wippen:")
    fitted("wippen", ("Pg2", "Pw2", "Pw3"))
    across("wippen", "Pw4", "Pw4", WIPPEN(marker("wippen", "Pw4")))
    across("wippen", "Pw6", "Pw6", WIPPEN(marker("wippen", "Pw6")))
    on_circle("wippen", "Pw5", "Pw5", WIPPEN(marker("wippen", "Pw5")))
    start, stretches = path("wippen", "heel_cloth_line")
    c, r = circle(WIPPEN(start), WIPPEN(stretches[0][1]), WIPPEN(stretches[0][2]))
    residuals["wippen"]["heel_cloth"] = max(math.hypot(*minus(c, PW5)), abs(r - R5))
    lines.append(f"  the heel cloth's drawn outer face: a circle of {r:.2f} mm about ({c[0]:.2f}, {c[1]:.2f}),"
                 f" Pw5's of {R5:.1f} about ({PW5[0]:.1f}, {PW5[1]:.1f})")
    lines.append("  the jack:")
    fitted("jack", ("pin", "Pj2", "Pj3"))
    drawn_radius("jack", "Pj2_radius", "Pj2", JACK_TOP[1])
    drawn_radius("jack", "Pj3_radius", "Pj3", JACK_TOE[1])
    start, stretches = path("jack", "jack")
    across("jack", "Pj5_face", "Pj5", JACK(stretches[4][1]))
    across("jack", "Pj5", "Pj5", JACK(marker("jack", "Pj5")))
    on_circle("jack", "Pj4", "Pj4", JACK(marker("jack", "Pj4")))
    lines.append("  the repetition lever:")
    fitted("repetition_lever", ("pin", "Pr2"))
    drawn_radius("repetition_lever", "Pr2_radius", "Pr2", LEVER_TOP[1])
    across("repetition_lever", "Pr5", "Pr5", LEVER(marker("repetition_lever", "Pr5")))
    on_circle("repetition_lever", "Pr3", "Pr3", LEVER(marker("repetition_lever", "Pr3")))
    on_circle("repetition_lever", "Pr4", "Pr4", LEVER(marker("repetition_lever", "Pr4")))
    return residuals, lines


def main():
    ok = True
    print("=== 1. The similarities ===")
    for body, frame in FRAMES.items():
        print(f"  {body:17s} {frame.scale() * 1000:.4f} um per pixel at 600 dpi, drawn at 1:"
              f"{frame.scale() * 600 / 25.4:.2f}; its frame turned {frame.turn_deg():+.3f} deg on the page")

    print("\n=== 2. Residuals, traced less printed, mm: the trace's uncertainty ===")
    residuals, lines = trace_residuals()
    for line in lines:
        print(line)
    print("  as the outline record keeps them, mm:")
    for body, found in residuals.items():
        kept = {k: v for k, v in TRACE["bodies"][body]["residuals_mm"].items() if k != "what"}
        for name in sorted(set(found) | set(kept), key=list(found).index):
            agree = name in found and same(found[name], kept.get(name), 2)
            ok &= agree
            shown = f"{found[name]:.2f}" if name in found else "missing"
            print(f"    {body:17s} {name:12s} {shown:>7s}   record {kept.get(name, 'missing')}"
                  f"{'' if agree else '   DIFFERS'}")

    print("  the center-of-mass markers against Table A.1, which governs the masses; the trace never"
          " corrects them:")
    for body, symbol in (("wippen", "CMw"), ("jack", "CMj"), ("repetition_lever", "CMr")):
        at, (_, center) = FRAMES[body](marker(body, symbol)), table_a1(body)
        print(f"    {symbol} at ({at[0]:.2f}, {at[1]:.2f}) mm, Table A.1's center at ({center[0]:.1f}, {center[1]:.1f}):"
              f" {math.hypot(*minus(at, center)):.1f} mm apart")

    print("\n=== 3. The outlines in each body's frame, and the numbers that place the heel ===")
    values = derived()
    for name, x in values.items():
        try:
            recorded = named(TRACE["records"], name)["value"]
        except StopIteration:
            recorded = None
        digits = 4 if name.endswith("_angle") else 2
        agree = same(x, recorded, digits)
        ok &= agree
        shown = f"{len(x)} stretches" if isinstance(x, list) else f"{x:.{digits}f}"
        print(f"    {name:30s} {shown:>14s}   record {'agrees' if agree else 'DIFFERS'}")
        if isinstance(x, list) and not agree:
            print(f"      derived {json.dumps(x)}")
    h = {k: v for k, v in values.items() if k.startswith("heel")}
    back, front = heel_lower_corners(h)
    for label, corner, name in (("back", back, "back_corner"), ("front", front, "front_corner")):
        traced = heel_point(name)
        print(f"  the heel's {label} lower corner, on the line through the cloth's ends: ({corner[0]:.2f},"
              f" {corner[1]:.2f}), {corner[1] - traced[1]:+.2f} mm from the one traced")
    cove_c, cove_r = circle(heel_point("back_corner"), WIPPEN(tuple(
        TRACE["bodies"]["wippen"]["outlines"]["heel"]["points"]["back_cove"]["through"])), heel_point("back_cove"))
    chord = math.hypot(*minus(heel_point("back_cove"), heel_point("back_corner")))
    print(f"  the cove of the heel's back side, taken as its chord: {cove_r - math.sqrt(cove_r ** 2 - chord ** 2 / 4):.2f}"
          f" mm deep at its middle, on a chord of {chord:.2f} mm")
    print(f"  the heel cloth's top, drawn straight between its ends, {h['heel_cloth_thickness']:.2f} mm in"
          f" from its outer face there and {R5 - h['heel_cloth_thickness'] - math.sqrt((R5 - h['heel_cloth_thickness']) ** 2 - ((h['heel_cloth_back'] + h['heel_cloth_front']) / 2) ** 2):.2f}"
          f" mm under its outer face's chord at its middle")

    print("\n=== 4. The jack and the repetition lever on their stops ===")
    jack, lever = values["jack_rest_angle"], values["repetition_lever_rest_angle"]
    found = rest_of_script_10()
    print(f"  the jack, contact 5 closed:             {jack:+.9f} rad ({math.degrees(jack):+.4f} deg);"
          f" script 10's rest {found[2]:+.9f}, Pw4's normal as printed")
    print(f"  the repetition lever, contact 6 closed: {lever:+.9f} rad ({math.degrees(lever):+.4f} deg);"
          f" script 10 closes the lever on the knuckle instead (contact 10): {found[3]:+.9f}")
    kept = {name: named(TRACE["records"], name)["value"] for name in ("jack_rest_angle", "repetition_lever_rest_angle")}
    gaps = rest_gaps(kept["jack_rest_angle"], kept["repetition_lever_rest_angle"])
    print(f"  at the record's angles, to 0.0001 rad: contact 5's gap {gaps[0] * 1000:+.3f} um,"
          f" contact 6's {gaps[1] * 1000:+.3f} um")

    print("\n=== 5. The wippen at rest, the key on its back rail cloth ===")
    edge_radius = mm(next(v for v in PROFILE["values"] if v["param"] == "back_rail.cloth_edge_radius"))
    (rest_round, _), (rest_felt, _), *_ = KEY.stops(edge_radius)
    for label, key_angle in (("his rounds (script 10's frame)", rest_round), ("the real felts", rest_felt)):
        print(f"  {label:32s} key {key_angle:+.9f} rad, wippen {wippen_on_capstan(key_angle):+.9f} rad")
    print(f"  script 10's rest:                key {found[0]:+.9f} rad, wippen {found[1]:+.9f} rad")

    print("\n=== 6. The body's long top edge and the drawn axis, against the frame's x axis ===")
    start, stretches = path("wippen", "body")
    edge = (WIPPEN(stretches[15][2]), WIPPEN(stretches[16][1]))
    edge_angle = math.atan2(edge[1][1] - edge[0][1], edge[1][0] - edge[0][0])
    axis = [WIPPEN(tuple(pixel)) for pixel in TRACE["bodies"]["wippen"]["axes"]["px"]]
    print(f"  the long top edge, ({edge[0][0]:.2f}, {edge[0][1]:.2f}) to ({edge[1][0]:.2f}, {edge[1][1]:.2f}):"
          f" {math.degrees(edge_angle):+.3f} deg")
    print(f"  the figure's x axis: {math.degrees(math.atan2(axis[1][1] - axis[0][1], axis[1][0] - axis[0][0])):+.3f} deg")

    print("\n=== 7. The assembly, its center pin and its wippen weight ===")
    mass, center = assembly(kept["jack_rest_angle"], kept["repetition_lever_rest_angle"])
    print(f"  Table A.1's wippen, jack and repetition lever, at the record's rest angles: {mass:.2f} g,"
          f" its center at ({center[0]:.2f}, {center[1]:.2f}) mm")
    stated = {v["param"]: v for v in PROFILE["values"]}
    bearing = (mm(stated["balance_rail.height"]) + sum(stated["balance_punching.papers"]["values"])
               + mm(stated["balance_punching.felt"]))
    pg2 = printed("ground", "Pg2")
    print(f"  the center pin: Pg2, {pg2[0]:.1f} mm from the balance pin and {pg2[1]:.1f} mm above its bearing,"
          f" on the borrowed balance bearing at {bearing:.1f} mm: {pg2[1] + bearing:.1f} mm above the keybed")
    heel_height = PW2[1] - PW5[1] + R5
    print(f"  the heel's height, its circle's lowest point below the jack's pin: Pw2's y {PW2[1]:.1f} less Pw5's"
          f" {PW5[1]:.1f}, plus its radius {R5:.1f}: {heel_height:.1f} mm")
    print("  as his profile states them by their rules, mm:")
    for param, x in (("wippen.center.y", pg2[1] + bearing), ("wippen_heel.height", heel_height)):
        recorded = stated[param]
        agree = recorded["unit"] == "mm" and same(x, recorded["value"], 1)
        ok &= agree
        print(f"    {param:20s} {x:6.1f}   profile {recorded['value']}{'' if agree else '   DIFFERS'}")
    wippen = wippen_on_capstan(rest_felt)
    mark = capstans_mark(rest_felt, wippen)
    print(f"  the capstan's mark, where its dome bears on the heel cloth with the key on its back rail cloth:"
          f" ({mark[0]:.2f}, {mark[1]:.2f}) mm")
    for label, level in (("(a) the long top edge horizontal", -edge_angle),
                         ("    the frame's x axis horizontal", 0.0),
                         ("(b) the center pin and the mark at one height", -math.atan2(mark[1], mark[0]))):
        arm_cm, arm_mark = rotated(level, center)[0], rotated(level, mark)[0]
        print(f"  {label:46s} the wippen turned {math.degrees(level):+7.3f} deg:"
              f" WW = {mass:.2f} x {arm_cm:.2f} / {arm_mark:.2f} = {mass * arm_cm / arm_mark:.2f} g")
    if not ok:
        print("\nthe outline record's values, or his profile's, differ from what they derive from", file=sys.stderr)
        sys.exit(1)


def rest_gaps(jack, lever):
    """Contacts 5 and 6's gaps at the given rest angles, mm."""
    pw4, n4 = printed("wippen", "Pw4"), normal("wippen", "Pw4")
    pj4, r4 = printed("jack", "Pj4"), radius("jack", "Pj4")
    pw6, n6 = printed("wippen", "Pw6"), normal("wippen", "Pw6")
    pr3, r3 = printed("repetition_lever", "Pr3"), radius("repetition_lever", "Pr3")
    return (dot(minus(add(PW2, rotated(jack, pj4)), pw4), n4) - r4,
            dot(minus(add(PW3, rotated(lever, pr3)), pw6), n6) - r3)


def capstan_center(key_angle):
    """The center of the capstan's dome, Pk3, in the ground's frame, mm."""
    return rotated(key_angle, printed("key", "Pk3"))


def wippen_on_capstan(key_angle):
    """The wippen's angle, radians, at which its heel cloth bears on the
    capstan's dome, contact 1, the key at key_angle: the root his action
    assembles at, bracketed about script 10's."""
    pg2, pk3_r = printed("ground", "Pg2"), radius("key", "Pk3")
    dome = capstan_center(key_angle)
    return bisect(lambda t: math.hypot(*minus(dome, add(pg2, rotated(t, PW5)))) - (pk3_r + R5), -0.3, 0.3)


def capstans_mark(key_angle, wippen_angle):
    """The point of the heel cloth's outer face the capstan's dome bears on,
    in the wippen's frame, mm."""
    dome = rotated(-wippen_angle, minus(capstan_center(key_angle), printed("ground", "Pg2")))
    u = unit(minus(dome, PW5))
    return (PW5[0] + R5 * u[0], PW5[1] + R5 * u[1])


def rest_of_script_10():
    """Script 10's rest, found as it finds it, in meters and radians: key,
    wippen, jack, lever, hammer."""
    for tw0 in (-0.2, -0.1, 0.0, 0.1):
        for tj0 in (-0.3, 0.0, 0.3):
            for th0 in (-0.3, 0.0, 0.3):
                try:
                    q = ASSEMBLED.close_rest_pose([0.0, tw0, tj0, 0.0, th0])
                except (ZeroDivisionError, OverflowError, ValueError):
                    continue
                if any(not math.isfinite(v) or abs(v) > 1.0 for v in q):
                    continue
                if max(abs(ASSEMBLED.gaps(q)[c]) for c in ASSEMBLED.CLOSED) > 1e-9:
                    continue
                return q
    raise SystemExit("script 10's rest was not found")


if __name__ == "__main__":
    main()
