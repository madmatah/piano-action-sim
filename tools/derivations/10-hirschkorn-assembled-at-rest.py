"""Hirschkorn's Boston GP-178, note 52, assembled at rest.

Source: Hirschkorn 2004, MASc thesis, Appendix A, read from its transcription
data/params/hirschkorn-boston-gp178-note52.json: Table A.1 (masses), Tables
A.2-A.7 (geometry), Table A.8 (which two points each contact joins, and its
law), and the frame convention stated at the top of that file. No value of his
is typed in this script, so the record and the script cannot drift apart.

Every body frame has its origin at the body's own pivot, as his Figure A.1
draws them: the key turns on Pg1, the wippen on Pg2 and the hammer on Pg3, all
fixed to ground. In the wippen's own frame, Pw2 is the jack's pin and Pw3 the
repetition lever's pin: both ride on the wippen. The pivots are read from the
record, which must name for each body a pin carried by the body it turns on.

Do not take Pw2 for the wippen's pivot: every lever arm, reflected mass and
time step built on that reading is wrong.

Do not compute an effective mass as I_pivot / r^2 with r the distance from a
pivot to a contact point. The arm that counts is the moment arm of the contact
normal, ((P - O) x n).z, coordinate by coordinate, and the jack and lever pins
move with the wippen. The effective mass along a contact normal is
1 / (grad g . M^-1 grad g) with M the assembled mass matrix.

Coordinates, in this order: key, wippen, jack relative to the wippen,
repetition lever relative to the wippen, hammer. All angles in radians.

Prints:
  1. the rest pose closed on contacts 1, 3, 5, 7 and 10, and every gap there;
  2. the mass matrix at rest;
  3. the gap gradients of contacts 7 and 10 (their lever arms);
  4. the effective mass along every contact normal;
  5. the reflected masses on the two sides of the jack/knuckle contact;
  6. the highest linearised mode with chosen contacts loaded at assumed forces.

Pure arithmetic on the standard library: math, and json to read the record by
its path relative to this file. Run with python3 -I.
"""
import json
import math
from pathlib import Path

RECORD = Path(__file__).resolve().parents[2] / "data" / "params" / "hirschkorn-boston-gp178-note52.json"
HK = json.loads(RECORD.read_text(encoding="utf-8"))
# The record's bodies, and the letter each goes by below.
BODY = {"ground": "g", "key": "k", "wippen": "w", "jack": "j", "repetition_lever": "r", "hammer": "h"}
KIND = {"Pin Location": "pin", "Circle Contact": "circle", "Line Contact": "line",
        "Hybrid Contact": "hybrid"}


def value(record):
    return record["value"]


def shape(p):
    """A point as the gap functions take it: a pin (x, y); a circle ((x, y), r);
    a line ((x, y), (nx, ny)); a hybrid ((x, y), r, (start, end))."""
    xy = (value(p["x"]), value(p["y"]))
    kind = KIND[p["description"]]
    if kind == "pin":
        return xy
    if kind == "circle":
        return (xy, value(p["radius"]))
    if kind == "line":
        return (xy, (value(p["normal"]["x"]), value(p["normal"]["y"])))
    return (xy, value(p["radius"]), (value(p["arc"]["start"]), value(p["arc"]["end"])))


# ---- Tables A.2-A.7: every point, in the frame of the body that carries it (metres) ----
POINT = {name: (code, KIND[p["description"]], shape(p))
         for body, code in BODY.items()
         for name, p in HK["geometry"][body]["points"].items()}


def pivot(body, carrier):
    """The pin `body` turns on, which frames() needs carried by `carrier`."""
    entry = next(p for p in HK["frame_convention"]["pivots"] if p["body"] == body)
    code, kind, xy = POINT[entry["turns_on"]]
    if (code, kind) != (BODY[carrier], "pin") or entry["pin_carried_by"] != carrier:
        raise ValueError(f"the {body} must turn on a pin carried by the {carrier}")
    return xy


# The key, the wippen and the hammer turn on ground pins; the jack and the lever on wippen pins.
O_K, O_W, O_H = pivot("key", "ground"), pivot("wippen", "ground"), pivot("hammer", "ground")
P_J, P_R = pivot("jack", "wippen"), pivot("repetition_lever", "wippen")


def inertia(body):
    r = HK["mass_properties"]["bodies"][body]
    cm = r["centre_of_mass"]
    return value(r["mass"]), value(r["moment_of_inertia"]), (value(cm["x"]), value(cm["y"]))


# ---- Table A.1: mass (kg), inertia about the centre of mass (kg m^2), centre of mass ----
MASS = {BODY[b]: inertia(b) for b in ("key", "wippen", "jack", "repetition_lever", "hammer")}
# ---- Table A.8: the two points each contact joins, and the elastic branch
# f = a d^3 + b d^2 + c d of its law ----
ROWS = HK["contacts"]["rows"]
CONTACT = {int(n): tuple(r["points"]) for n, r in ROWS.items()}
LAW = {int(n): (value(r["a"]), value(r["b"]), value(r["c"])) for n, r in ROWS.items()}
NAME = {1: "capstan / wippen heel", 2: "key front / front rail punching", 3: "key back / back rail cloth",
        4: "back check / hammer tail", 5: "wippen / jack rest", 6: "wippen / lever stop",
        7: "jack / knuckle", 8: "jack toe / let-off button", 9: "jack / lever window",
        10: "lever / knuckle", 11: "lever / drop screw", 12: "hammer / string", 13: "shank / rest rail"}
# [S] Assumed peak contact forces at fortissimo, newtons. Assumptions, not
# measurements: they decide the stiffness each contact is linearised at.
F_ASSUMED = {1: 60, 2: 50, 3: 5, 4: 15, 5: 3, 6: 3, 7: 100, 8: 25, 9: 3, 10: 25, 11: 15, 12: 800, 13: 2}
N = 5  # key, wippen, jack (relative), lever (relative), hammer


def rot(t, v):
    c, s = math.cos(t), math.sin(t)
    return (c * v[0] - s * v[1], s * v[0] + c * v[1])


def add(a, b):
    return (a[0] + b[0], a[1] + b[1])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1])


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1]


def frames(q):
    """Origin and absolute angle of every body's frame."""
    tk, tw, tj, tr, th = q
    o_w = O_W
    return {"k": (O_K, tk), "w": (o_w, tw),
            "j": (add(o_w, rot(tw, P_J)), tw + tj),
            "r": (add(o_w, rot(tw, P_R)), tw + tr),
            "h": (O_H, th), "g": ((0.0, 0.0), 0.0)}


def point(f, body, p):
    o, t = f[body]
    return add(o, rot(t, p))


def vector(f, body, v):
    return rot(f[body][1], v)


def gap_circles(f, b1, c1, r1, b2, c2, r2):
    d = sub(point(f, b1, c1), point(f, b2, c2))
    return math.hypot(d[0], d[1]) - (r1 + r2)


def gap_line_circle(f, bl, p, n, bc, c, r):
    return dot(sub(point(f, bc, c), point(f, bl, p)), vector(f, bl, n)) - r


def gap_hybrid_circle(f, bh, centre, radius, arc, bc, c, r):
    """An arc joined to two tangent lines (Hirschkorn's hybrid), against a circle."""
    o, t = f[bh]
    d = rot(-t, sub(point(f, bc, c), point(f, bh, centre)))
    phi = math.atan2(d[1], d[0])
    if phi < -math.pi / 2:
        phi += 2 * math.pi
    a0, a1 = arc
    if a0 <= phi <= a1:
        return math.hypot(d[0], d[1]) - radius - r
    a = a0 if phi < a0 else a1
    return dot(d, (math.cos(a), math.sin(a))) - radius - r


def gap(f, c):
    """Contact c's gap, from the shapes of the two points Table A.8 says it joins."""
    (b1, k1, s1), (b2, k2, s2) = (POINT[p] for p in CONTACT[c])
    if (k1, k2) == ("circle", "circle"):
        return gap_circles(f, b1, *s1, b2, *s2)
    if (k1, k2) == ("line", "circle"):
        return gap_line_circle(f, b1, *s1, b2, *s2)
    if (k1, k2) == ("circle", "line"):
        return gap_line_circle(f, b2, *s2, b1, *s1)
    if (k1, k2) == ("hybrid", "circle"):
        return gap_hybrid_circle(f, b1, *s1, b2, *s2)
    raise ValueError(f"contact {c}: no gap function for a {k1} against a {k2}")


def gaps(q):
    f = frames(q)
    return {c: gap(f, c) for c in sorted(CONTACT)}


def solve_linear(a, b):
    """Gaussian elimination with partial pivoting on a square system."""
    n = len(b)
    m = [list(a[i]) + [b[i]] for i in range(n)]
    for col in range(n):
        piv = max(range(col, n), key=lambda i: abs(m[i][col]))
        if abs(m[piv][col]) < 1e-300:
            raise ZeroDivisionError("singular")
        m[col], m[piv] = m[piv], m[col]
        for i in range(col + 1, n):
            k = m[i][col] / m[col][col]
            for j in range(col, n + 1):
                m[i][j] -= k * m[col][j]
    x = [0.0] * n
    for i in reversed(range(n)):
        x[i] = (m[i][n] - sum(m[i][j] * x[j] for j in range(i + 1, n))) / m[i][i]
    return x


def inverse(a):
    n = len(a)
    cols = [solve_linear(a, [1.0 if i == j else 0.0 for i in range(n)]) for j in range(n)]
    return [[cols[j][i] for j in range(n)] for i in range(n)]


def gradient(c, q, h=1e-8):
    g0 = gaps(q)[c]
    out = []
    for i in range(N):
        qp = list(q)
        qp[i] += h
        out.append((gaps(qp)[c] - g0) / h)
    return out


CLOSED = [3, 1, 5, 7, 10]


def close_rest_pose(q0):
    q = list(q0)
    for _ in range(100):
        g = gaps(q)
        r = [g[c] for c in CLOSED]
        jac = [gradient(c, q, 1e-7) for c in CLOSED]
        step = solve_linear(jac, [-x for x in r])
        q = [q[i] + step[i] for i in range(N)]
        if max(abs(s) for s in step) < 1e-13:
            break
    return q


def mass_matrix(q, h=1e-7):
    omega = {"k": [1, 0, 0, 0, 0], "w": [0, 1, 0, 0, 0], "j": [0, 1, 1, 0, 0],
             "r": [0, 1, 0, 1, 0], "h": [0, 0, 0, 0, 1]}
    m_mat = [[0.0] * N for _ in range(N)]
    for b, (m, i_cm, cm) in MASS.items():
        p0 = point(frames(q), b, cm)
        jv = []
        for i in range(N):
            qp = list(q)
            qp[i] += h
            p = point(frames(qp), b, cm)
            jv.append(((p[0] - p0[0]) / h, (p[1] - p0[1]) / h))
        for i in range(N):
            for j in range(N):
                m_mat[i][j] += m * dot(jv[i], jv[j]) + i_cm * omega[b][i] * omega[b][j]
    return m_mat


def quad(v, a, w):
    return sum(v[i] * a[i][j] * w[j] for i in range(len(v)) for j in range(len(w)))


def stiffness_at(c, force):
    a, b, cc = LAW[c]
    lo, hi = 0.0, 0.02
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if a * mid**3 + b * mid**2 + cc * mid < force:
            lo = mid
        else:
            hi = mid
    d = 0.5 * (lo + hi)
    return 3 * a * d * d + 2 * b * d + cc


def top_mode_hz(m_mat, m_inv, contacts, q):
    """Largest generalized eigenvalue of (K, M) by power iteration."""
    k_mat = [[0.0] * N for _ in range(N)]
    for c in contacts:
        g = gradient(c, q)
        k = stiffness_at(c, F_ASSUMED[c])
        for i in range(N):
            for j in range(N):
                k_mat[i][j] += k * g[i] * g[j]
    x = [1.0] * N
    lam = 0.0
    for _ in range(3000):
        kx = [sum(k_mat[i][j] * x[j] for j in range(N)) for i in range(N)]
        y = [sum(m_inv[i][j] * kx[j] for j in range(N)) for i in range(N)]
        norm = math.sqrt(sum(v * v for v in y))
        x = [v / norm for v in y]
        lam = quad(x, k_mat, x) / quad(x, m_mat, x)
    return math.sqrt(lam) / (2 * math.pi)


if __name__ == "__main__":
    found = []
    for tw0 in (-0.2, -0.1, 0.0, 0.1):
        for tj0 in (-0.3, 0.0, 0.3):
            for th0 in (-0.3, 0.0, 0.3):
                try:
                    q = close_rest_pose([0.0, tw0, tj0, 0.0, th0])
                except (ZeroDivisionError, OverflowError, ValueError):
                    continue
                if any(not math.isfinite(v) or abs(v) > 1.0 for v in q):
                    continue
                if max(abs(gaps(q)[c]) for c in CLOSED) > 1e-9:
                    continue
                key = tuple(round(v, 4) for v in q)
                if key not in [k for k, _ in found]:
                    found.append((key, q))
    print("=== 1. Rest pose closed on contacts 1, 3, 5, 7 and 10 ===")
    print(f"  distinct rest poses found from 36 starting points: {len(found)}")
    for key, q in found:
        g = gaps(q)
        print("  q (rad), key / wippen / jack rel. / lever rel. / hammer:", key)
        for c in range(1, 14):
            note = "  closed" if c in CLOSED else ("  INTERPENETRATION" if g[c] < -1e-9 else "")
            print(f"    #{c:2d} {NAME[c]:32s} gap {g[c] * 1e3:+7.2f} mm{note}")
    q0 = found[0][1]

    m_mat = mass_matrix(q0)
    m_inv = inverse(m_mat)
    print("\n=== 2. Mass matrix at rest, kg m^2 (key, wippen, jack rel., lever rel., hammer) ===")
    for row in m_mat:
        print("   " + " ".join(f"{v:+.3e}" for v in row))

    print("\n=== 3. Lever arms at rest: d(gap)/dq, mm/rad ===")
    for c in (7, 10):
        print(f"  #{c:2d} {NAME[c]:16s} " + "  ".join(f"{v * 1e3:+7.2f}" for v in gradient(c, q0)))
    print("  (the radial distance from the jack's pin to its contact arc is"
          f" {math.hypot(*POINT['Pj2'][2][0]) * 1e3:.1f} mm;")
    print("   the jack's own arm on its flat top is a few millimetres)")

    print("\n=== 4. Effective mass along each contact normal, 1/(grad g . M^-1 grad g) ===")
    for c in range(1, 14):
        g = gradient(c, q0)
        m_eff = 1.0 / quad(g, m_inv, g)
        print(f"  #{c:2d} {NAME[c]:32s} {m_eff * 1e3:8.2f} g")

    print("\n=== 5. Jack/knuckle (#7): reflected mass on each side at rest ===")
    g = gradient(7, q0)
    driver_inv = inverse([row[:4] for row in m_mat[:4]])
    driver = 1.0 / quad(g[:4], driver_inv, g[:4])
    hammer = m_mat[4][4] / g[4] ** 2
    print(f"  driver side (key, wippen, jack, lever; hammer held): {driver * 1e3:6.1f} g")
    print(f"  hammer side (hammer alone):                          {hammer * 1e3:6.1f} g")
    print(f"  ratio {hammer / driver:.0f} : 1")

    print("\n=== 6. Highest linearised mode at rest, contacts at their ASSUMED fff forces ===")
    for label, cs in (("key down, jack driving (1, 2, 7)", [1, 2, 7]),
                      ("let-off region (1, 2, 7, 10, 11)", [1, 2, 7, 10, 11]),
                      ("all thirteen at once, an upper bound", list(range(1, 14)))):
        f = top_mode_hz(m_mat, m_inv, cs, q0)
        print(f"  {label:40s} {f:6.0f} Hz")
