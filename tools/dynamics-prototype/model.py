"""One note of Hirschkorn's grand piano action in motion: the model.

THROWAWAY PROTOTYPE (README.md). It exists to settle the hypotheses of
docs/FOUNDATIONS.md §4 (the time step, one dynamic solver, the felt law) and is
not the engine.

Source: Hirschkorn 2004, MASc thesis, Appendix A, as transcribed with locators
in data/params/hirschkorn-boston-gp178-note52.json ("the record"). The record is
read through tools/derivations/10-hirschkorn-assembled-at-rest.py ("script 10"),
loaded by its path: its reading of the record (every point with its body, kind
and shape; the masses; which two points each contact joins, and each cubic) and
its pivots (every frame's origin at its own pivot; Pw2 and Pw3 the pins of the
jack and of the repetition lever, carried by the wippen, never the wippen's
pivot) are used as they are, so this model and script 10 cannot disagree on
geometry. What script 10 does not read (damping factors, contact friction, axis
friction, springs) is read here from the same record, through script 10's parsed
copy of it. No value of Hirschkorn's is typed in this file's code; the
docstrings quote some.

Coordinates, Hirschkorn's (Eq. 2.1, Figure 2.3) and script 10's, in this order:

    q = (θk, θw, θj, θr, θh)

θk, θw, θh: key, wippen and hammer, counter-clockwise from ground about their
ground pins Pg1, Pg2, Pg3. θj, θr: jack and repetition lever, counter-clockwise
from the wippen about their pins Pw2, Pw3 (relative coordinates, FOUNDATIONS
§3.6). Every coordinate is a joint angle, so the joint speeds are exactly q̇.
SI throughout: metres, radians, kilograms, newtons, seconds.

Equations of motion:  M(q) q̈ + h(q, q̇) = Q(q, q̇, t) + (axis friction)

ASSUMPTIONS, each also stated where it is used:
  A1  Gravity 9.81 m/s² along -y of the ground frame. The record gives no g
      (not_given) and its ground frame has y upwards (Figure A.1).
  A2  Table A.1's moment of inertia is about the centre of mass (the record's
      reading, ambiguities). Inertia is an internal parameter: Table A.1 here,
      nothing measured.
  A3  Normal law, Eqs. 2.4-2.5 with the clamp:
          f_n = max(0, (a δ³ + b δ² + c δ)(1 + D δ̇))  for δ > 0, else 0,
      δ = -gap the penetration, δ̇ = -d(gap)/dt, positive while loading (p. 22).
  A4  Contact friction at contacts 1, 4, 7 and 10 only (pp. 25-26), Eq. 2.10
      with its bracket, μ = A (tanh(x) + B1 x / (1 + B2 x⁴)), x = s_t / v_t,
      f_t = μ f_n (Eq. 2.11), opposing the relative tangential speed s_t of the
      two bodies' material points at the contact point. v_t is not given: 1e-5
      m/s is ASSUMED, the value Hirschkorn's Figure 2.7 draws.
  A5  Contact point: midway between the two surfaces on the common normal,
      through the circle's centre. Any point on that normal gives the same
      lever arms; the tangential slip speed, used by friction only, depends on
      it through the relative rotation times the half-penetration.
  A6  Axis friction, Table A.9's A at each of the five joints, SET-VALUED:
      while a joint's speed is zero any torque within ±A holds it; while it
      slides, A opposes the sliding. Hirschkorn measured kinetic torques only
      and put no static friction in his joints (B1 = 0, §A.4), so the bound of
      the set is the kinetic torque. His threshold speed ω_t has no role here.
      The set is solved by the integrators (integrators.py), not here.
  A7  Repetition spring, Eq. A.3 as printed: T = -0.073 (θr - 0.832) N·m on θr.
      Jack spring, §A.4: a constant 0.0042 N·m on θj whose sign the thesis does
      not give. CHOSEN counter-clockwise (+): the let-off button turns the jack
      clockwise to escape (§1.2, p. 4), so the spring must turn it back
      counter-clockwise, which presses the jack against its rest (contact 5) and
      under the knuckle (contact 7); checks.py verifies both signs.
      A spring between a child and the wippen acts on the child's relative
      coordinate only (FOUNDATIONS §6.1, the lemma).
  A8  The finger acts at Pk2, the point on top of the key front where Figure
      A.1 draws its downward arrow; the model's one applied force is "an
      externally specified force applied to the key front" (§2.1, p. 15). Its
      direction is ASSUMED vertical, downward in the ground frame: a force F > 0
      pushes the key front down with the torque -F (x_Pk2 - x_Pg1) about Pg1. A
      force perpendicular to the key instead would act on Pk2's x in the key's
      own frame, an arm 0.4 % longer at rest.
  A9  The hybrid surface (contacts 7 and 10) is the boundary of the region
      W ⊕ disc(R): W the wedge with its apex at the arc's centre bounded by the
      two tangent lines' directions, R the arc's radius. The gap is the exact
      signed distance from the circle's centre to that region, minus the
      circle's radius: continuous everywhere and negative at every depth. Its
      normal is continuous across both ends of the arc and everywhere the
      circle's centre is outside W, that is at every penetration below R + r
      (5.4 mm at contact 7, 59.6 mm at contact 10). Deeper, inside W, the
      normal switches between the two lines on W's bisector: some switch is
      unavoidable, since a unit normal field turning once around a convex body
      cannot be continuous everywhere inside it.
  A10 Tables A.2-A.7 print their unit vectors to three decimals, and four are
      not of unit length: Pk5 0.99984, Pk6 0.99977, Pw4 0.99998, Pr5 0.99960.
      The gaps use them as printed, as script 10 does, so that the two agree
      on the closure and on every gap script 10 prints: normalised, they would
      move the hammer by 38 µrad at the closure and contacts 4, 11, 12 and 13
      across a printed hundredth of a millimetre. Those four lines' gaps are
      then the signed distance times that norm (0.04 % at most), and so are
      their gradients. Every contact reports a unit normal, and slip is
      measured along a unit tangent.
"""

from __future__ import annotations

import importlib.util
import math
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
SCRIPT_10 = HERE.parent / "derivations" / "10-hirschkorn-assembled-at-rest.py"


def _load_script_10():
    # Loading by path would cache bytecode beside script 10, in a directory
    # this prototype only reads: write none, anywhere.
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("hirschkorn_assembled_at_rest", SCRIPT_10)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


S10 = _load_script_10()
RECORD = S10.HK  # the record, as script 10 parsed it

# ---- coordinates ----
K, W, J, R, H = range(5)
N_Q = 5
COORDINATES = ("key", "wippen", "jack", "repetition_lever", "hammer")
BODY_CODE = ("k", "w", "j", "r", "h")  # script 10's letter for each coordinate's body
MOVING_BODIES = BODY_CODE

# ---- assumptions (module docstring) ----
GRAVITY_M_S2 = 9.81  # A1
FRICTION_THRESHOLD_SPEED_M_S = 1e-5  # A4: Hirschkorn's Figure 2.7, assumed
JACK_SPRING_SIGN = +1.0  # A7: counter-clockwise on θj, chosen
FINGER_DIRECTION = (0.0, -1.0)  # A8: vertical, downward in the ground frame

TAU = 2.0 * math.pi


def _value(rec):
    return rec["value"]


# ---- values from the record ----
_ROWS = RECORD["contacts"]["rows"]
CONTACTS = tuple(sorted(S10.CONTACT))  # 1 .. 13, Table A.8's order
NAME = S10.NAME  # script 10's short name of each contact
DAMPING_S_PER_M = {c: _value(_ROWS[str(c)]["damping_factor"]) for c in CONTACTS}
FRICTION_LAW = {}  # contact -> (A, B1, B2), Table A.8, the contacts with A > 0
for _c in CONTACTS:
    _f = _ROWS[str(_c)]["friction"]
    if _value(_f["A"]) > 0.0:
        FRICTION_LAW[_c] = (_value(_f["A"]), _value(_f["B1"]), _value(_f["B2"]))
AXIS_FRICTION_NM = np.array(
    [_value(RECORD["joint_friction"]["rows"][name]["A"]) for name in COORDINATES]
)  # Table A.9's A, N·m, one per coordinate's joint
JACK_SPRING_TORQUE_NM = JACK_SPRING_SIGN * _value(RECORD["springs"]["jack"]["torque"])
REPETITION_SPRING_COEFF_NM_PER_RAD = _value(RECORD["springs"]["repetition_lever"]["coefficient"])
REPETITION_SPRING_OFFSET_RAD = _value(RECORD["springs"]["repetition_lever"]["angle_offset"])

# body -> (mass kg, inertia about the centre of mass kg·m², centre of mass in its frame, m)
MASS = S10.MASS
O_K, O_W, O_H = S10.O_K, S10.O_W, S10.O_H  # ground pins of key, wippen, hammer
P_J, P_R = S10.P_J, S10.P_R  # the jack's and the lever's pins, in the wippen's frame

_body_pk2, _kind_pk2, FINGER_POINT_LOCAL = S10.POINT["Pk2"]  # A8, in the key's frame
assert (_body_pk2, _kind_pk2) == ("k", "pin")

if FRICTION_LAW and tuple(sorted(FRICTION_LAW)) != (1, 4, 7, 10):
    raise ValueError("Table A.8 should give contact friction at contacts 1, 4, 7 and 10 only")


# =============================================================================
# Primitives
# =============================================================================


def hybrid_signed_distance(dx, dy, radius, a0, a1):
    """Signed distance from a point to Hirschkorn's hybrid surface (A9).

    (dx, dy) is the point relative to the arc's centre, in the hybrid body's own
    frame. The arc runs counter-clockwise from a0 to a1; the tangent lines leave
    its ends, line 0 at a0 and line 1 at a1. Returns (distance, nx, ny, branch),
    (nx, ny) the outward unit normal in the body's frame, branch one of "arc",
    "line0", "line1".

    Outside W: in the arc's angular sector the nearest point is on the arc;
    elsewhere it is on one of the lines, and the larger of the two projections
    picks it. Inside W the same maximum is the distance to the nearer boundary.
    At an arc end the radial normal equals the line's normal, so the normal is
    continuous there.
    """
    rho = math.hypot(dx, dy)
    if rho > 0.0:
        psi = a0 + (math.atan2(dy, dx) - a0) % TAU
        if psi <= a1:
            return rho - radius, dx / rho, dy / rho, "arc"
    n0x, n0y = math.cos(a0), math.sin(a0)
    n1x, n1y = math.cos(a1), math.sin(a1)
    s0 = dx * n0x + dy * n0y
    s1 = dx * n1x + dy * n1y
    if s0 >= s1:
        return s0 - radius, n0x, n0y, "line0"
    return s1 - radius, n1x, n1y, "line1"


class ContactSpec:
    """One row of Table A.8: the two points it joins, first body A then B
    (record: contacts.points_order), their shapes, and the contact's law.
    gap_scale is the norm of a line's printed normal (A10), 1 otherwise."""

    def __init__(self, number):
        name_a, name_b = S10.CONTACT[number]
        body_a, kind_a, shape_a = S10.POINT[name_a]
        body_b, kind_b, shape_b = S10.POINT[name_b]
        self.number = number
        self.points = (name_a, name_b)
        self.a, self.b = body_a, body_b
        self.law = S10.LAW[number]  # (a, b, c): N/m³, N/m², N/m
        self.damping = DAMPING_S_PER_M[number]  # s/m
        self.friction = FRICTION_LAW.get(number)  # (A, B1, B2) or None
        self.gap_scale = 1.0
        kinds = (kind_a, kind_b)
        if kinds == ("circle", "circle"):
            self.kind = "cc"
            self.circle_a, self.radius_a = shape_a
            self.circle_b, self.radius_b = shape_b
        elif kinds == ("line", "circle"):
            self.kind = "lc"
            self.line_point, self.line_normal = shape_a
            self.circle, self.radius = shape_b
        elif kinds == ("circle", "line"):
            self.kind = "cl"
            self.circle, self.radius = shape_a
            self.line_point, self.line_normal = shape_b
        elif kinds == ("hybrid", "circle"):
            self.kind = "hc"
            self.arc_centre, self.arc_radius, (self.arc_start, self.arc_end) = shape_a
            self.circle, self.radius = shape_b
        else:
            raise ValueError(f"contact {number}: no primitive for a {kind_a} against a {kind_b}")
        if self.kind in ("lc", "cl"):
            self.gap_scale = math.hypot(*self.line_normal)


SPECS = {c: ContactSpec(c) for c in CONTACTS}


class Pose:
    """Every body's frame at q, and what turns each body.

    frame[b] = (ox, oy, cos φ, sin φ): origin and absolute angle φ of body b's frame.
    moves[b] = ((i, ox, oy), ...): each coordinate i that turns body b, with the
    centre it turns it about. The jack is turned by θw about Pg2 and by θj about
    its own pin, which the wippen carries; likewise the lever.
    """

    __slots__ = ("q", "frame", "moves")

    def __init__(self, q):
        tk, tw, tj, tr, th = (float(x) for x in q)
        cw, sw = math.cos(tw), math.sin(tw)
        ojx = O_W[0] + cw * P_J[0] - sw * P_J[1]
        ojy = O_W[1] + sw * P_J[0] + cw * P_J[1]
        orx = O_W[0] + cw * P_R[0] - sw * P_R[1]
        ory = O_W[1] + sw * P_R[0] + cw * P_R[1]
        self.q = (tk, tw, tj, tr, th)
        self.frame = {
            "g": (0.0, 0.0, 1.0, 0.0),
            "k": (O_K[0], O_K[1], math.cos(tk), math.sin(tk)),
            "w": (O_W[0], O_W[1], cw, sw),
            "j": (ojx, ojy, math.cos(tw + tj), math.sin(tw + tj)),
            "r": (orx, ory, math.cos(tw + tr), math.sin(tw + tr)),
            "h": (O_H[0], O_H[1], math.cos(th), math.sin(th)),
        }
        self.moves = {
            "g": (),
            "k": ((K, O_K[0], O_K[1]),),
            "w": ((W, O_W[0], O_W[1]),),
            "j": ((W, O_W[0], O_W[1]), (J, ojx, ojy)),
            "r": ((W, O_W[0], O_W[1]), (R, orx, ory)),
            "h": ((H, O_H[0], O_H[1]),),
        }

    def to_world(self, body, x, y):
        ox, oy, c, s = self.frame[body]
        return ox + c * x - s * y, oy + s * x + c * y

    def to_local(self, body, x, y):
        ox, oy, c, s = self.frame[body]
        dx, dy = x - ox, y - oy
        return c * dx + s * dy, -s * dx + c * dy


def contact_geometry(spec, pose):
    """Signed gap, unit normal n pointing from body B to body A, contact point
    P, and the hybrid's branch (None for the other primitives).

    Every gap is a signed distance: a line against a circle is the projection on
    the line's normal, a circle against a circle the distance between centres,
    the hybrid the exact signed distance to its region (A9). None takes an
    absolute value, so each keeps its sign through any penetration, deeper than
    the circle's radius included. A line's gap is projected on its printed
    normal, as script 10 does (A10): for four lines it is the signed distance
    times spec.gap_scale, which lever_arms carries into the gradient.
    """
    oxa, oya, ca, sa = pose.frame[spec.a]
    oxb, oyb, cb, sb = pose.frame[spec.b]
    kind = spec.kind
    if kind == "lc":  # line on A, circle on B
        (lx, ly), (lnx, lny) = spec.line_point, spec.line_normal
        cx, cy = spec.circle
        px, py = oxa + ca * lx - sa * ly, oya + sa * lx + ca * ly
        wnx, wny = ca * lnx - sa * lny, sa * lnx + ca * lny  # the line's normal: A towards B
        qx, qy = oxb + cb * cx - sb * cy, oyb + sb * cx + cb * cy
        gap = (qx - px) * wnx + (qy - py) * wny - spec.radius
        ux, uy = wnx / spec.gap_scale, wny / spec.gap_scale
        d = spec.radius + 0.5 * gap
        return gap, -ux, -uy, qx - ux * d, qy - uy * d, None
    if kind == "cl":  # circle on A, line on B
        (lx, ly), (lnx, lny) = spec.line_point, spec.line_normal
        cx, cy = spec.circle
        px, py = oxb + cb * lx - sb * ly, oyb + sb * lx + cb * ly
        wnx, wny = cb * lnx - sb * lny, sb * lnx + cb * lny  # the line's normal: B towards A
        qx, qy = oxa + ca * cx - sa * cy, oya + sa * cx + ca * cy
        gap = (qx - px) * wnx + (qy - py) * wny - spec.radius
        ux, uy = wnx / spec.gap_scale, wny / spec.gap_scale
        d = spec.radius + 0.5 * gap
        return gap, ux, uy, qx - ux * d, qy - uy * d, None
    if kind == "cc":  # circle on A, circle on B
        (cxa, cya), (cxb, cyb) = spec.circle_a, spec.circle_b
        ax, ay = oxa + ca * cxa - sa * cya, oya + sa * cxa + ca * cya
        bx, by = oxb + cb * cxb - sb * cyb, oyb + sb * cxb + cb * cyb
        dx, dy = ax - bx, ay - by
        dist = math.hypot(dx, dy)
        # Coincident centres need a penetration of r_a + r_b (35 mm at the
        # capstan): unreachable; the normal is then arbitrary.
        nx, ny = (dx / dist, dy / dist) if dist > 0.0 else (0.0, 1.0)
        gap = dist - spec.radius_a - spec.radius_b
        d = spec.radius_b + 0.5 * gap
        return gap, nx, ny, bx + nx * d, by + ny * d, None
    # "hc": hybrid on A, circle on B
    hx, hy = spec.arc_centre
    cx, cy = spec.circle
    ax, ay = oxa + ca * hx - sa * hy, oya + sa * hx + ca * hy
    qx, qy = oxb + cb * cx - sb * cy, oyb + sb * cx + cb * cy
    wx, wy = qx - ax, qy - ay
    dist, lnx, lny, branch = hybrid_signed_distance(
        ca * wx + sa * wy, -sa * wx + ca * wy, spec.arc_radius, spec.arc_start, spec.arc_end
    )
    wnx, wny = ca * lnx - sa * lny, sa * lnx + ca * lny  # the hybrid's outward normal: A towards B
    gap = dist - spec.radius
    d = spec.radius + 0.5 * gap
    return gap, -wnx, -wny, qx - wnx * d, qy - wny * d, branch


def lever_arms(spec, pose, nx, ny, px, py):
    """The gap's gradient over all five coordinates, by the lever-arm identity
    of FOUNDATIONS §6.2, and the Jacobian of the tangential slip speed.

    For each coordinate i turning body A about O: ∂g/∂q_i += ((P - O) × n)·ẑ;
    for body B the same with a minus sign. A coordinate that turns both bodies
    (θw, when the contact is inside the wippen assembly) cancels exactly. The
    jack's and the lever's points pick up θw about Pg2 and their own coordinate
    about their pin.

    The four lines of A10 project their gap on the printed normal, whose norm
    spec.gap_scale multiplies their arms.

    s_t = (v_A(P) - v_B(P))·t with t = ẑ × n; since (ẑ × r)·(ẑ × n) = r·n,
    ∂s_t/∂q̇_i = ±(P - O)·n, the projection of the arm on the normal.
    """
    grad = [0.0] * N_Q
    jt = [0.0] * N_Q
    scale = spec.gap_scale
    for i, ox, oy in pose.moves[spec.a]:
        rx, ry = px - ox, py - oy
        grad[i] += scale * (rx * ny - ry * nx)
        jt[i] += rx * nx + ry * ny
    for i, ox, oy in pose.moves[spec.b]:
        rx, ry = px - ox, py - oy
        grad[i] -= scale * (rx * ny - ry * nx)
        jt[i] -= rx * nx + ry * ny
    return grad, jt


def normal_force(spec, delta, delta_dot):
    """Eqs. 2.4-2.5 with the clamp (A3). Returns (f_n, df_n/dδ, df_n/dδ̇), the
    two derivatives zero wherever the clamp or separation holds f_n at zero."""
    if delta <= 0.0:
        return 0.0, 0.0, 0.0
    a, b, c = spec.law
    fit = ((a * delta + b) * delta + c) * delta
    fit_slope = (3.0 * a * delta + 2.0 * b) * delta + c
    damping = 1.0 + spec.damping * delta_dot
    f = fit * damping
    if f <= 0.0:
        return 0.0, 0.0, 0.0
    return f, fit_slope * damping, fit * spec.damping


def friction_coefficient(law, slip):
    """Eq. 2.10 with its bracket (A4): μ and dμ/ds_t."""
    a, b1, b2 = law
    x = slip / FRICTION_THRESHOLD_SPEED_M_S
    t = math.tanh(x)
    x4 = x * x * x * x
    den = 1.0 + b2 * x4
    mu = a * (t + b1 * x / den)
    dmu = a * (1.0 - t * t + b1 * (1.0 - 3.0 * b2 * x4) / (den * den)) / FRICTION_THRESHOLD_SPEED_M_S
    return mu, dmu


# =============================================================================
# The model
# =============================================================================


class Evaluation:
    """Everything one step needs at (q, q̇), from one pass over the bodies.

    M      mass matrix (5×5)
    h      Coriolis and centripetal terms: M q̈ + h = Q
    Q      generalized forces of gravity, springs and contacts (normal and
           friction); NOT axis friction, NOT the drive
    K, C   with jacobians=True: -∂Q/∂q and -∂Q/∂q̇ of the contact normal laws
           and the repetition spring, the part the implicit integrator treats
           implicitly (integrators.py states which terms are left out, and why
           the contact friction's slope is left out unless
           Model(implicit_friction_slope=True))
    gap, fn, ft, slip   per contact, Table A.8's order (index c - 1)
    """

    __slots__ = ("pose", "M", "h", "Q", "K", "C", "gap", "fn", "ft", "slip", "q_gravity",
                 "q_springs", "q_contacts")


class Model:
    """The one-note model. The flags switch a family of forces off, for the
    consistency checks; the defaults are the full model.
    implicit_friction_slope=True adds Eq. 2.10's slope to C for integrator (B),
    a variant kept only to rerun the comparison integrators.py cites."""

    def __init__(self, *, contacts=True, contact_friction=True, axis_friction=True,
                 springs=True, gravity=True, implicit_friction_slope=False):
        self.contacts = contacts
        self.contact_friction = contact_friction
        self.implicit_friction_slope = implicit_friction_slope
        self.springs = springs
        self.gravity = gravity
        self.axis_friction_bound_Nm = AXIS_FRICTION_NM.copy() if axis_friction else np.zeros(N_Q)

    # ---- kinematics ----

    @staticmethod
    def pose(q):
        return Pose(q)

    @staticmethod
    def centre_of_mass(body, pose):
        cx, cy = MASS[body][2]
        return pose.to_world(body, cx, cy)

    @staticmethod
    def body_angle(body, q):
        """Absolute angle of a body's frame (its angular velocity is the sum of
        the same coordinates' rates)."""
        tk, tw, tj, tr, th = q
        return {"k": tk, "w": tw, "j": tw + tj, "r": tw + tr, "h": th, "g": 0.0}[body]

    # ---- mass matrix and velocity terms ----

    @staticmethod
    def mass_matrix(pose):
        """M = Σ m Jvᵀ Jv + I Jωᵀ Jω, Jv the Jacobian of the centre of mass's
        velocity, Jω that of the angular velocity (1 for each coordinate turning
        the body), I about the centre of mass (A2)."""
        m_mat = [[0.0] * N_Q for _ in range(N_Q)]
        for body in MOVING_BODIES:
            m, inertia, (cx, cy) = MASS[body]
            xx, xy = pose.to_world(body, cx, cy)
            cols = [(i, oy - xy, xx - ox) for i, ox, oy in pose.moves[body]]  # ẑ × (x - O)
            for i, ax, ay in cols:
                row = m_mat[i]
                for j, bx, by in cols:
                    row[j] += m * (ax * bx + ay * by) + inertia
        return m_mat

    @staticmethod
    def velocity_terms(pose, v):
        """h(q, q̇), analytically: h = Σ m Jvᵀ γ with γ = J̇v q̇ the convective
        acceleration of each centre of mass. A body on a ground pin O turning at
        ω: γ = -ω² (x - O). The jack (the lever likewise), turned by θw about Pg2
        and by θj about its pin o_J, both moving: with a = x - Pg2, b = x - o_J,
            γ = -θ̇w² a - (2 θ̇w θ̇j + θ̇j²) b.
        Planar rotation adds no gyroscopic term, Jω being constant. checks.py
        tests h against Σ (∂M_ij/∂q_k - ½ ∂M_jk/∂q_i) q̇_j q̇_k by finite
        differences of M."""
        h = [0.0] * N_Q
        for body in MOVING_BODIES:
            m, _inertia, (cx, cy) = MASS[body]
            xx, xy = pose.to_world(body, cx, cy)
            moves = pose.moves[body]
            if len(moves) == 1:
                ((i, ox, oy),) = moves
                w2 = v[i] * v[i]
                gx, gy = -w2 * (xx - ox), -w2 * (xy - oy)
            else:
                (i0, ox0, oy0), (i1, ox1, oy1) = moves
                w0, w1 = v[i0], v[i1]
                k1 = 2.0 * w0 * w1 + w1 * w1
                gx = -w0 * w0 * (xx - ox0) - k1 * (xx - ox1)
                gy = -w0 * w0 * (xy - oy0) - k1 * (xy - oy1)
            for i, ox, oy in moves:
                h[i] += m * ((oy - xy) * gx + (xx - ox) * gy)
        return h

    # ---- applied forces ----

    def gravity_force(self, pose):
        """Q_i = -∂V/∂q_i with V = g Σ m y_cm (A1): -g Σ m (x_cm - O_i)_x."""
        qg = [0.0] * N_Q
        if not self.gravity:
            return qg
        for body in MOVING_BODIES:
            m, _inertia, (cx, cy) = MASS[body]
            xx, _xy = pose.to_world(body, cx, cy)
            for i, ox, _oy in pose.moves[body]:
                qg[i] -= GRAVITY_M_S2 * m * (xx - ox)
        return qg

    def spring_force(self, q):
        """A7: the repetition spring on θr, the jack's constant torque on θj."""
        qs = [0.0] * N_Q
        if self.springs:
            qs[R] = REPETITION_SPRING_COEFF_NM_PER_RAD * (q[R] - REPETITION_SPRING_OFFSET_RAD)
            qs[J] = JACK_SPRING_TORQUE_NM
        return qs

    @staticmethod
    def finger_arm(q):
        """Moment arm about Pg1 of the finger's force at Pk2 along
        FINGER_DIRECTION (A8), m: a force F > 0 gives the key the torque
        -F·arm, so arm = -((P - Pg1) × direction)·ẑ, the horizontal distance
        x_Pk2 - x_Pg1 for a vertical, downward force."""
        pose = Pose(q)
        px, py = pose.to_world("k", *FINGER_POINT_LOCAL)
        dx, dy = FINGER_DIRECTION
        return -((px - O_K[0]) * dy - (py - O_K[1]) * dx)

    # ---- contacts ----

    @staticmethod
    def contact_kinematics(q):
        """Per contact: (gap, normal B->A, point, gradient, slip Jacobian, branch)."""
        pose = Pose(q)
        out = {}
        for c in CONTACTS:
            spec = SPECS[c]
            gap, nx, ny, px, py, branch = contact_geometry(spec, pose)
            grad, jt = lever_arms(spec, pose, nx, ny, px, py)
            out[c] = (gap, (nx, ny), (px, py), grad, jt, branch)
        return out

    @staticmethod
    def gaps(q):
        pose = Pose(q)
        return np.array([contact_geometry(SPECS[c], pose)[0] for c in CONTACTS])

    # ---- everything at once ----

    def evaluate(self, q, v, jacobians=False):
        pose = Pose(q)
        v = [float(x) for x in v]
        ev = Evaluation()
        ev.pose = pose
        ev.M = np.array(self.mass_matrix(pose))
        ev.h = np.array(self.velocity_terms(pose, v))
        qg = self.gravity_force(pose)
        qs = self.spring_force(pose.q)
        qc = [0.0] * N_Q
        n_c = len(CONTACTS)
        gap_all = np.full(n_c, np.inf)
        fn_all = np.zeros(n_c)
        ft_all = np.zeros(n_c)
        slip_all = np.zeros(n_c)
        k_mat = [[0.0] * N_Q for _ in range(N_Q)] if jacobians else None
        c_mat = [[0.0] * N_Q for _ in range(N_Q)] if jacobians else None
        if jacobians and self.springs:
            k_mat[R][R] -= REPETITION_SPRING_COEFF_NM_PER_RAD  # -∂Q_r/∂θr = +0.073
        if self.contacts:
            for idx, c in enumerate(CONTACTS):
                spec = SPECS[c]
                gap, nx, ny, px, py, _branch = contact_geometry(spec, pose)
                gap_all[idx] = gap
                if gap >= 0.0:
                    continue
                grad, jt = lever_arms(spec, pose, nx, ny, px, py)
                gdot = sum(g * w for g, w in zip(grad, v))
                fn, dfn_ddelta, dfn_ddot = normal_force(spec, -gap, -gdot)
                fn_all[idx] = fn
                if fn == 0.0:
                    continue
                for i in range(N_Q):
                    qc[i] += fn * grad[i]
                if jacobians:
                    # f_n(δ, δ̇) with δ = -g, δ̇ = -∇g·q̇: K += k ∇g∇gᵀ, C += c ∇g∇gᵀ
                    for i in range(N_Q):
                        if grad[i] != 0.0:
                            ki, ci = dfn_ddelta * grad[i], dfn_ddot * grad[i]
                            for j in range(N_Q):
                                k_mat[i][j] += ki * grad[j]
                                c_mat[i][j] += ci * grad[j]
                if self.contact_friction and spec.friction is not None:
                    slip = sum(a * w for a, w in zip(jt, v))
                    mu, dmu = friction_coefficient(spec.friction, slip)
                    ft = mu * fn
                    slip_all[idx] = slip
                    ft_all[idx] = ft
                    for i in range(N_Q):
                        qc[i] -= ft * jt[i]
                    if jacobians and self.implicit_friction_slope and dmu > 0.0:
                        # OFF by default (integrators.py): the symmetric part,
                        # while μ rises with slip
                        for i in range(N_Q):
                            if jt[i] != 0.0:
                                ci = dmu * fn * jt[i]
                                for j in range(N_Q):
                                    c_mat[i][j] += ci * jt[j]
        ev.q_gravity = np.array(qg)
        ev.q_springs = np.array(qs)
        ev.q_contacts = np.array(qc)
        ev.Q = ev.q_gravity + ev.q_springs + ev.q_contacts
        ev.K = np.array(k_mat) if jacobians else None
        ev.C = np.array(c_mat) if jacobians else None
        ev.gap, ev.fn, ev.ft, ev.slip = gap_all, fn_all, ft_all, slip_all
        return ev

    # ---- energies ----

    def kinetic_energy(self, q, v):
        m_mat = np.array(self.mass_matrix(Pose(q)))
        v = np.asarray(v, dtype=float)
        return 0.5 * float(v @ m_mat @ v)

    def potential_energy(self, q):
        """Gravity, springs, and the elastic part of every contact law,
        ∫ f_fit dδ = a δ⁴/4 + b δ³/3 + c δ²/2."""
        pose = Pose(q)
        e = 0.0
        if self.gravity:
            for body in MOVING_BODIES:
                _xx, xy = self.centre_of_mass(body, pose)
                e += MASS[body][0] * GRAVITY_M_S2 * xy
        if self.springs:
            e -= 0.5 * REPETITION_SPRING_COEFF_NM_PER_RAD * (q[R] - REPETITION_SPRING_OFFSET_RAD) ** 2
            e -= JACK_SPRING_TORQUE_NM * q[J]
        if self.contacts:
            for c in CONTACTS:
                delta = -contact_geometry(SPECS[c], pose)[0]
                if delta > 0.0:
                    a, b, cc = SPECS[c].law
                    e += ((a / 4.0 * delta + b / 3.0) * delta + cc / 2.0) * delta * delta
        return e


# =============================================================================
# Poses
# =============================================================================

CLOSED_AT_REST = tuple(S10.CLOSED)  # script 10's closure: contacts 3, 1, 5, 7, 10


def closure_pose(q0, tol=1e-15, cap=100):
    """Script 10's closure solved with this model's gaps and analytic
    gradients: the gaps of contacts 1, 3, 5, 7 and 10 at zero, five equations in
    five unknowns, by Newton. A geometric pose, not an equilibrium (see
    static_equilibrium). Returns (q, iterations)."""
    q = np.array(q0, dtype=float)
    for it in range(1, cap + 1):
        kin = Model.contact_kinematics(q)
        r = np.array([kin[c][0] for c in CLOSED_AT_REST])
        jac = np.array([kin[c][3] for c in CLOSED_AT_REST])
        step = np.linalg.solve(jac, -r)
        q = q + step
        if np.max(np.abs(step)) < tol:
            return q, it
    return q, cap


def static_equilibrium(model, q0, tol_Nm=1e-12, cap=500):
    """Frictionless static equilibrium: Q(q, 0) = 0 over all five coordinates.

    At rest the contact dampers and contact friction exert nothing (δ̇ = 0,
    μ(0) = 0), so Q(q, 0) = -∇V with V = model.potential_energy, and the
    equilibrium is a minimum of V. It is found by Levenberg-Marquardt on V:
    the stiffness by central differences of Q, scaled by its diagonal; a step
    is taken if it lowers V (Armijo), or, once V's change is below its
    rounding, if it lowers |Q|. Newton on Q alone is not robust here: from
    script 10's closure the lever's 0.88 mm overlap with its stop throws the
    first step across several contact switches.

    With set-valued axis friction, any state at rest whose net torques lie
    within ±A holds; the frictionless equilibrium (zero net torque) is one of
    them, and the one the scenarios start from. Returns (q, iterations,
    residual N·m)."""
    q = np.array(q0, dtype=float)
    zero = np.zeros(N_Q)

    def force(x):
        return model.evaluate(x, zero).Q

    energy = model.potential_energy(q)
    f = force(q)
    damping = 1e-3
    for it in range(1, cap + 1):
        if np.max(np.abs(f)) < tol_Nm:
            return q, it - 1, float(np.max(np.abs(f)))
        stiff = np.zeros((N_Q, N_Q))
        for j in range(N_Q):
            e = np.zeros(N_Q)
            e[j] = 1e-9
            stiff[:, j] = -(force(q + e) - force(q - e)) / 2e-9
        stiff = 0.5 * (stiff + stiff.T)
        scale = np.maximum(np.abs(np.diag(stiff)), 1e-9)
        while True:
            step = np.linalg.solve(stiff + damping * np.diag(scale), f)
            trial = q + step
            e_trial = model.potential_energy(trial)
            f_trial = force(trial)
            decrease = float(f @ step)  # the first-order fall of V along the step
            lowers = e_trial <= energy - 1e-4 * decrease
            flat = abs(e_trial - energy) <= 1e-15 * (abs(energy) + 1.0)
            if lowers or (flat and np.max(np.abs(f_trial)) < np.max(np.abs(f))):
                q, energy, f = trial, e_trial, f_trial
                damping = max(damping / 3.0, 1e-12)
                break
            damping *= 4.0
            if damping > 1e12:
                return q, it, float(np.max(np.abs(f)))
    return q, cap, float(np.max(np.abs(f)))


def script_10_rest_pose():
    """Script 10's own rest pose, found as its __main__ finds it: Newton from its
    grid of starting points, first distinct converged pose."""
    for tw0 in (-0.2, -0.1, 0.0, 0.1):
        for tj0 in (-0.3, 0.0, 0.3):
            for th0 in (-0.3, 0.0, 0.3):
                start = [0.0, tw0, tj0, 0.0, th0]
                try:
                    q = S10.close_rest_pose(start)
                except (ZeroDivisionError, OverflowError, ValueError):
                    continue
                if any(not math.isfinite(x) or abs(x) > 1.0 for x in q):
                    continue
                if max(abs(S10.gaps(q)[c]) for c in S10.CLOSED) > 1e-9:
                    continue
                return np.array(q), start
    raise RuntimeError("script 10 found no rest pose")
