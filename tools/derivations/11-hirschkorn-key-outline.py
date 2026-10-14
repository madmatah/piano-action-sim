"""Hirschkorn's key (Boston GP-178, note 52), traced from his Figure A.1.

Sources: Hirschkorn 2004, MASc thesis. Figure A.1, 'Defined Points on Action
Components' (printed page 68): the positions of its markers and of the key's
outline in pixels of the page rendered at 600 dpi, with how they were read, in
data/params/hirschkorn-key-outline.json. His printed coordinates (Tables A.1
to A.3, A.9 and his regulation) from data/params/hirschkorn-boston-gp178-
note52.json, whose frame convention puts every point in its body's frame, the
key's origin at its pivot Pg1 on its underside. The bench's measuring point,
13 mm back from the covering's lip, from data/params/touchweight-protocol.json;
the length of the back rail cloth his profile lays at Pg5, and the borrowed
balance bearing, felts, cloth and papers his profile's rules stand on, from
data/profiles/hirschkorn.json. Each is read with json by a path relative to this
file: no value of his, no pixel and no number of the profile is typed here.

Prints:
  1. the similarity that carries the figure's pixels into his key's frame,
     fitted by least squares on the three markers whose printed coordinates
     are points: Pg1, Pk2 and Pk5;
  2. the residuals: at those three, and at the other markers, which are the
     trace's uncertainty: Pk4 and Pk6 across and along their printed lines,
     Pk3's marker, drawn on the capstan's dome, against the dome's top;
  3. every feature of the key's outline in millimetres in his key frame, and
     the numbers his profile binds from them, each beside the value the
     outline record gives; exit status 1 where the two differ at the 0.01 mm
     the record is written to, where the residuals differ from the ones it
     keeps, or where a number of section 7 differs from his profile's;
  4. two flags: the figure's CMk marker against Table A.1, which governs the
     mass, and how far Pk4's marker stands along its line from its printed
     point;
  5. the readings Table A.1, Table A.9 and the trace give in closed form at
     the bench's measuring point: front weight, key weight ratio, the key
     friction weight the bushings hold with the key level;
  6. his printed frame between its two stops, on his rounds of 5 mm and on
     the flat felts his profile draws with their tops where the rounds' tops
     stand: the attitudes, the dip at Pk2 and at the key front, and what the
     real felts change;
  7. the numbers his profile states by a rule from his frame, each beside the
     profile's: the front rail's and the back rail's heights, which put the
     felts' tops where his rounds' tops stand on the borrowed balance
     bearing, and the front mortise's place over Pg4; and the capstan's
     height above its seat, which the bench computes as it builds his key.

Heights are read from the drawn underside, a line through the chamfer's kink
and the key's front, so that the fit's small turn against the drawn key
(below 0.05 degree) moves none of them; lengths along the key are read in the
fitted frame.

Pure arithmetic on the standard library: math, json, sys and pathlib. Run with
python3 -I.
"""
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TRACE = json.loads((ROOT / "data" / "params" / "hirschkorn-key-outline.json").read_text(encoding="utf-8"))
HK = json.loads((ROOT / "data" / "params" / "hirschkorn-boston-gp178-note52.json").read_text(encoding="utf-8"))
PROTOCOL = json.loads((ROOT / "data" / "params" / "touchweight-protocol.json").read_text(encoding="utf-8"))
PROFILE = json.loads((ROOT / "data" / "profiles" / "hirschkorn.json").read_text(encoding="utf-8"))

G = 9.80665  # standard gravity, m/s^2


def mm(record):
    """A printed record's value in millimetres."""
    scale = {"m": 1000.0, "mm": 1.0}[record["unit"]]
    return record["value"] * scale


def point(body, name):
    """A printed point (x, y) in millimetres, in its body's frame."""
    p = HK["geometry"][body]["points"][name]
    return (mm(p["x"]), mm(p["y"]))


def normal(body, name):
    n = HK["geometry"][body]["points"][name]["normal"]
    return (n["x"]["value"], n["y"]["value"])


def radius(body, name):
    return mm(HK["geometry"][body]["points"][name]["radius"])


def named(records, name):
    return next(r for r in records if r["name"] == name)


# ---- 1. the similarity ------------------------------------------------------

FITTED_ON = ("Pg1", "Pk2", "Pk5")
PRINTED = {"Pg1": point("ground", "Pg1"), "Pk2": point("key", "Pk2"), "Pk5": point("key", "Pk5")}
MARKER = {name: tuple(m["px"]) for name, m in TRACE["markers"].items()}


def solve(a, b):
    """Gaussian elimination with partial pivoting on a square system."""
    n = len(b)
    m = [list(a[i]) + [b[i]] for i in range(n)]
    for col in range(n):
        piv = max(range(col, n), key=lambda i: abs(m[i][col]))
        m[col], m[piv] = m[piv], m[col]
        for i in range(n):
            if i != col:
                k = m[i][col] / m[col][col]
                for j in range(col, n + 1):
                    m[i][j] -= k * m[col][j]
    return [m[i][n] / m[i][i] for i in range(n)]


def fit():
    """X = a x + b y + c, Y = b x - a y + d: a turn, a scale and a shift, the
    page's y running down and the key's up. Least squares on FITTED_ON."""
    rows, rhs = [], []
    for name in FITTED_ON:
        (x, y), (X, Y) = MARKER[name], PRINTED[name]
        rows += [[x, y, 1.0, 0.0], [-y, x, 0.0, 1.0]]
        rhs += [X, Y]
    normal_eq = [[sum(r[i] * r[j] for r in rows) for j in range(4)] for i in range(4)]
    right = [sum(r[i] * h for r, h in zip(rows, rhs)) for i in range(4)]
    return solve(normal_eq, right)


A, B, C, D = fit()


def key(p):
    """A pixel position in his key frame, millimetres."""
    x, y = p
    return (A * x + B * y + C, B * x - A * y + D)


def sub(p, q):
    return (p[0] - q[0], p[1] - q[1])


def dot(p, q):
    return p[0] * q[0] + p[1] * q[1]


def unit(v):
    n = math.hypot(*v)
    return (v[0] / n, v[1] / n)


# ---- 3. the features ----------------------------------------------------------

FEATURE = {name: [key(tuple(p)) for p in f["px"]] for name, f in TRACE["features"].items()}
KINK, FRONT = FEATURE["underside"]
ALONG = unit(sub(FRONT, KINK))
UP = (-ALONG[1], ALONG[0])


def height(p):
    """Height above the drawn underside, millimetres."""
    return dot(sub(p, KINK), UP)


def mean_x(points):
    return sum(p[0] for p in points) / len(points)


def line_height(p, q, x):
    """The height above the drawn underside, at x, of the drawn line through p and q."""
    hp, hq = height(p), height(q)
    return hp + (hq - hp) * (x - p[0]) / (q[0] - p[0])


def derived():
    """The numbers his profile binds from the trace, millimetres, by name."""
    f = FEATURE
    pk2 = point("key", "Pk2")
    pk3, dome = point("key", "Pk3"), radius("key", "Pk3")
    front_edge = mean_x(f["key_front_face"])
    keystick_front = mean_x(f["keystick_front"])
    lip = mean_x(f["lip"])
    lip_top, lip_bottom = f["lip"]
    keytop_thickness = height(lip_top) - height(lip_bottom)
    step = mean_x(f["step"])
    step_top, step_foot = f["step"]
    wood_top = f["keystick_front"][1]
    tail_top = f["tail_end"][0]
    tail_end = mean_x(f["tail_end"])
    tail_height = (height(tail_top) + height(step_foot)) / 2
    button = f["key_button"]
    button_back, button_front = button[0][0], button[3][0]
    button_centre = (button_back + button_front) / 2
    button_height = (height(button[1]) + height(button[2])) / 2 - line_height(
        wood_top, step_top, button_centre
    )
    block = f["capstan_block"]
    block_back, block_front = block[0][0], block[2][0]
    block_height = (height(block[1]) + height(block[2])) / 2 - line_height(
        tail_top, step_foot, (block_back + block_front) / 2
    )
    check = f["back_check_block"]
    check_back, check_front = check[0][0], check[3][0]
    check_height = (height(check[1]) + height(check[2])) / 2 - line_height(
        tail_top, step_foot, (check_back + check_front) / 2
    )
    head = f["capstan_head"]
    head_diameter = ((head[2][0] - head[1][0]) + (head[3][0] - head[0][0])) / 2
    shank = f["capstan_shank"]
    shank_diameter = ((shank[2][0] - shank[1][0]) + (shank[3][0] - shank[0][0])) / 2
    head_bottom = (height(head[1]) + height(head[2])) / 2
    # The dome is Pk3's circle: its top on the axis at Pk3's height plus its
    # radius, and it meets the head's side its sagitta lower.
    sagitta = dome - math.sqrt(dome * dome - (head_diameter / 2) ** 2)
    head_height = pk3[1] + dome - sagitta - head_bottom
    return {
        "front_edge": front_edge,
        "front_board": front_edge - keystick_front,
        "keytop_overhang": lip - front_edge,
        "keytop_thickness": keytop_thickness,
        "keytop_length": lip - f["covering_back"][0][0],
        # The covering's top passes through Pk2, a printed point on it.
        "stick_height": pk2[1] - keytop_thickness,
        "capsule_length": button_front - button_back,
        "capsule_centre": button_centre,
        "capsule_height": button_height,
        "capstan_block_back": block_back,
        "capstan_block_length": block_front - block_back,
        "capstan_block_height": block_height,
        "back_check_block_back": check_back,
        "back_check_block_length": check_front - check_back,
        "back_check_block_height": check_height,
        "tail_end": tail_end,
        "tail_length": step - tail_end,
        "tail_height": tail_height,
        "capstan_diameter": shank_diameter,
        "capstan_head_diameter": head_diameter,
        "capstan_head_height": head_height,
    }, {
        "keystick front (the wood under the key front)": keystick_front,
        "covering's lip": lip,
        "step from the keystick down to the thinned tail": step,
        "keystick's height as drawn, at its front": height(wood_top),
        "keystick's height as drawn, at the step": height(step_top),
        "covering's top as drawn, at Pk2": line_height(wood_top, step_top, pk2[0]) + keytop_thickness,
        "capstan block's front": block_front,
        "capstan's seat, the block's top": tail_height + block_height,
        "capstan's head, its bottom": head_bottom,
        "capstan's dome, its top (Pk3's height plus its radius)": pk3[1] + dome,
        "chamfer's kink as drawn": KINK[0],
    }


# ---- 6. the key between its stops -------------------------------------------

def rot(t, v):
    c, s = math.cos(t), math.sin(t)
    return (c * v[0] - s * v[1], s * v[0] + c * v[1])


def bisect(f, lo, hi):
    f_lo = f(lo)
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        f_mid = f(mid)
        if (f_mid > 0) == (f_lo > 0):
            lo, f_lo = mid, f_mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def line_to_round(t, through, n, centre, r):
    """The gap between the key's line (through a point, outward normal n,
    both in the key frame, the key turned by t about its pivot) and a round,
    the normal taken as printed, as script 10 and the dynamics prototype
    take it: Pk5's is 0.99984 long, which moves the rest by 4 microradians."""
    p, m = rot(t, through), rot(t, n)
    return dot(sub(centre, p), m) - r


def stops(back_half_length):
    """Rest and bottom attitudes, radians nose-up, on his rounds and on flat
    felts with their tops at the rounds' tops."""
    pk4, n4 = point("key", "Pk4"), normal("key", "Pk4")
    pk5, n5 = point("key", "Pk5"), normal("key", "Pk5")
    pg4, r4 = point("ground", "Pg4"), radius("ground", "Pg4")
    pg5, r5 = point("ground", "Pg5"), radius("ground", "Pg5")
    rounds = (
        bisect(lambda t: line_to_round(t, pk5, n5, pg5, r5), 0.0, 0.1),
        bisect(lambda t: line_to_round(t, pk4, n4, pg4, r4), -0.1, 0.0),
    )
    # The chamfer meets the flat where Pk5's line crosses y = 0.
    kink = pk5[0] + n5[1] * pk5[1] / n5[0]
    back_top, front_top = pg5[1] + r5, pg4[1] + r4
    # The front punching is read at its centre, where the bench reads every
    # punching; the cloth's face is flat, and the key, nose-up, is lowest
    # over it at its kink where the strip reaches in front of the kink.
    strip_front = pg5[0] + back_half_length
    if strip_front < kink:
        raise SystemExit("the strip falls within the chamfer: the key bears on its edge, not on its kink")
    flats = (math.asin(back_top / kink), math.atan2(front_top, pg4[0]))
    return rounds, flats, kink, strip_front


def pg_top(name):
    """The top of a printed round of the ground's, millimetres from the
    balance bearing, Pg1."""
    return point("ground", name)[1] + radius("ground", name)


def main():
    ok = True
    print("=== 1. The similarity, fitted on Pg1, Pk2 and Pk5 ===")
    scale = math.hypot(A, B)
    print(f"  {scale * 1000:.4f} um per pixel at 600 dpi, the key drawn at 1:{scale * 600 / 25.4:.2f};"
          f" the drawn key turned {math.degrees(math.atan2(B, A)):+.3f} deg on the page")

    print("\n=== 2. Residuals, traced less printed, mm: the trace's uncertainty ===")
    residuals = {}
    for name in FITTED_ON:
        t = key(MARKER[name])
        r = sub(t, PRINTED[name])
        residuals[name] = math.hypot(*r)
        print(f"  {name}  traced ({t[0]:8.2f}, {t[1]:6.2f})  printed ({PRINTED[name][0]:8.2f}, {PRINTED[name][1]:6.2f})"
              f"  residual ({r[0]:+.2f}, {r[1]:+.2f}), {residuals[name]:.2f} mm   [fitted]")
    for body, name in (("key", "Pk4"), ("key", "Pk6")):
        t, p, n = key(MARKER[name]), point(body, name), unit(normal(body, name))
        r = sub(t, p)
        across, along = dot(r, n), dot(r, (-n[1], n[0]))
        residuals[name] = abs(across)
        print(f"  {name}  traced ({t[0]:8.2f}, {t[1]:6.2f})  printed line through ({p[0]:8.2f}, {p[1]:6.2f})"
              f"  across the line {across:+.2f}, along it {along:+.2f}")
    pk3, dome = point("key", "Pk3"), radius("key", "Pk3")
    t = key(MARKER["Pk3"])
    top = (pk3[0], pk3[1] + dome)
    r = sub(t, top)
    residuals["Pk3"] = math.hypot(*r)
    print(f"  Pk3  traced ({t[0]:8.2f}, {t[1]:6.2f})  the dome's top, Pk3's centre raised by its"
          f" {dome:.1f} mm radius, ({top[0]:8.2f}, {top[1]:6.2f})  residual ({r[0]:+.2f}, {r[1]:+.2f}), {residuals['Pk3']:.2f} mm")
    print("  as the outline record keeps them, mm:")
    for name, x in residuals.items():
        recorded = TRACE["residuals_mm"].get(name)
        same = recorded is not None and f"{x:.2f}" == f"{recorded:.2f}"
        ok &= same
        print(f"    {name}  {x:.2f}   record {recorded}{'' if same else '   DIFFERS'}")

    print("\n=== 3. The key's outline, mm in his key frame ===")
    values, drawn = derived()
    for name, x in drawn.items():
        print(f"  {name:58s} {x:8.2f}")
    print("  the numbers his profile binds, the trace's and the outline record's:")
    for name, x in values.items():
        try:
            recorded = named(TRACE["records"], name)["value"]
        except StopIteration:
            recorded = None
        same = recorded is not None and f"{x:.2f}" == f"{recorded:.2f}"
        ok &= same
        print(f"    {name:28s} {x:8.2f}   record {recorded if recorded is not None else 'missing':>8}"
              f"{'' if same else '   DIFFERS'}")

    print("\n=== 4. Flags ===")
    mass = HK["mass_properties"]["bodies"]["key"]
    m_g = mass["mass"]["value"] * 1000.0
    cm = (mm(mass["centre_of_mass"]["x"]), mm(mass["centre_of_mass"]["y"]))
    cmk = key(MARKER["CMk"])
    # From here on, the numbers the bench is built from: the record's, to its
    # 0.01 mm.
    rec = {r["name"]: r["value"] for r in TRACE["records"]}
    lip = rec["front_edge"] + rec["keytop_overhang"]
    from_lip = mm(named(PROTOCOL["records"], "measuring_point_from_lip"))
    arm = lip - from_lip
    print(f"  the figure's CMk marker stands at ({cmk[0]:.1f}, {cmk[1]:.1f}) mm, Table A.1's centre of mass at"
          f" ({cm[0]:.1f}, {cm[1]:.1f}) mm: {m_g * (cm[0] - cmk[0]) / arm:.1f} g of front weight apart at the"
          f" measuring point. Table A.1 governs the mass; the trace never corrects it.")
    pk4 = point("key", "Pk4")
    t4 = key(MARKER["Pk4"])
    print(f"  Pk4's marker stands {t4[0] - pk4[0]:+.2f} mm along its line from its printed x, {t4[1] - pk4[1]:+.2f} mm"
          f" across it: harmless for a line, whose printed point is any point on it.")
    print(f"  the key front stands {rec['front_edge'] - point('key', 'Pk2')[0]:.2f} mm in front of Pk2, where his"
          f" finger presses, and the covering's lip {lip - rec['front_edge']:.2f} mm in front of the key front.")

    print("\n=== 5. Closed forms at the bench's measuring point ===")
    print(f"  measuring point {from_lip:.1f} mm back from the lip at {lip:.2f} mm: {arm:.2f} mm in front of the pin,"
          f" {arm - point('key', 'Pk2')[0]:+.2f} mm from Pk2")
    print(f"  front weight, Table A.1's static moment over that arm, the key level: {m_g * cm[0] / arm:.4f} g")
    print(f"  key weight ratio, the dome's top on Pk3's axis over that arm: {-pk3[0] / arm:.4f}")
    friction = HK["joint_friction"]["rows"]["key"]["A"]["value"]
    print(f"  key friction weight the bushings hold, Table A.9's {friction} N.m at that arm, the key level:"
          f" {friction / (G * arm / 1000.0) * 1000.0:.4f} g")

    print("\n=== 6. His printed frame between its stops: his rounds, and the flat felts his profile draws ===")
    half = mm(next(v for v in PROFILE["values"] if v["param"] == "back_rail.half_length"))
    (rest_r, bottom_r), (rest_f, bottom_f), kink, strip_front = stops(half)
    pk2 = point("key", "Pk2")
    key_front = (rec["front_edge"], rec["stick_height"] + rec["keytop_thickness"])

    def dip(rest, bottom, p):
        return rot(rest, p)[1] - rot(bottom, p)[1]

    for label, rest, bottom in (("his rounds (Pg4, Pg5)", rest_r, bottom_r), ("the flat felts", rest_f, bottom_f)):
        print(f"  {label:24s} rest {math.degrees(rest):+.6f} deg ({rest:+.9f} rad),"
              f" bottom {math.degrees(bottom):+.6f} deg ({bottom:+.9f} rad),"
              f" dip at Pk2 {dip(rest, bottom, pk2):.4f} mm, at the key front {dip(rest, bottom, key_front):.4f} mm")
    print(f"  the cloth strip, {2 * half:.1f} mm long on Pg5, reaches {strip_front - kink:.2f} mm in front of the"
          f" chamfer's kink at {kink:.3f} mm, so the key rests on its kink;"
          f" the front punching is read at its centre, over Pg4")
    print(f"  the real felts change the rest by {math.degrees(rest_f - rest_r):+.6f} deg, the bottom by"
          f" {math.degrees(bottom_f - bottom_r):+.6f} deg, and the dip at Pk2 by"
          f" {dip(rest_f, bottom_f, pk2) - dip(rest_r, bottom_r, pk2):+.4f} mm")

    print("\n=== 7. The numbers his profile derives from his frame by a rule, and states ===")
    stated = {v["param"]: v for v in PROFILE["values"]}

    def value_of(param):
        v = stated[param]
        if "values" in v:
            return sum(v["values"])
        return mm(v) if "record" not in v else None

    bearing = value_of("balance_rail.height") + value_of("balance_punching.papers") + value_of("balance_punching.felt")
    front_top, back_top = pg_top("Pg4"), pg_top("Pg5")
    rules = {
        "front_rail.height": bearing + front_top - value_of("front_punching.felt") - value_of("front_punching.papers"),
        "back_rail.height": bearing + back_top - value_of("back_rail.cloth"),
        "front_mortise.from_front_edge": rec["front_edge"] - point("ground", "Pg4")[0],
    }
    print(f"  the balance bearing, borrowed: {bearing:.2f} mm above the keybed; his rounds' tops"
          f" {front_top:+.2f} mm (Pg4) and {back_top:+.2f} mm (Pg5) from it")
    for param, x in rules.items():
        recorded = mm(stated[param])
        same = f"{x:.2f}" == f"{recorded:.2f}"
        ok &= same
        print(f"    {param:30s} {x:8.2f}   profile {recorded:8.2f}{'' if same else '   DIFFERS'}")
    seat = rec["tail_height"] + rec["capstan_block_height"]
    print(f"  the capstan's height above its seat, which the bench computes as it builds his key:"
          f" Pk3's {pk3[1]:.1f} + {dome:.1f} - {seat:.2f} = {pk3[1] + dome - seat:.2f} mm")
    if not ok:
        print("\nthe outline record's or the profile's values differ from what they derive from",
              file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
