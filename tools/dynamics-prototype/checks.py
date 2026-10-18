"""Level-0 consistency checks of the one-note model and its two integrators.

THROWAWAY PROTOTYPE (README.md).

Every check here is a CONSISTENCY CHECK in the sense of `docs/FOUNDATIONS.md` > What a test proves; drive in position, compare forces: it shows that the model agrees with itself, with mathematics, or with
tools/derivations/10-hirschkorn-assembled-at-rest.py ("script 10"). It catches
coding errors (a sign, a lever arm, a frame, a unit, a solver that does not
solve what it says); a wrong model passes every one of them. None of them is a
validation, which needs measurements independent of the model.

Sections 1 to 6 hold the six level-0 checks the prototype is specified with
(README.md): the mass matrix, the contact gradients, the hybrid normal, energy
in free flight, script 10's rest pose, and axis friction holding a body still,
each with the companions it needs. The checks after them test what those six
rely on. The last section re-derives the model from the record with code it
shares with neither model.py nor script 10, checks the directions and signs a
technician would assert (which way each normal points, where each contact acts,
which way gravity and the repetition spring turn each body), and runs the
assembled action: axis friction holding every joint near its bound at once,
energy through contact closures, and a slow key stroke through let-off.

Run:  python3 -I checks.py     exit status 1 if any check fails.
Plain asserts: do not run with -O, which strips them.
"""

from __future__ import annotations

import cmath
import json
import math
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # python3 -I drops it
sys.dont_write_bytecode = True  # no __pycache__ beside the prototype

import numpy as np  # noqa: E402

import integrators as itg  # noqa: E402
import model as mdl  # noqa: E402
import scenarios as sc  # noqa: E402

if not __debug__:
    raise SystemExit("checks.py relies on assert: run it without -O")

SEED = 20041  # fixed, so every run draws the same poses
CHECKS = []


def check(fn):
    CHECKS.append(fn)
    return fn


def closure_pose():
    q, _start = mdl.script_10_rest_pose()
    return q


def random_poses(rng, n, spread=(0.05, 0.1, 0.4, 0.4, 0.4)):
    """Poses scattered around script 10's closure, radians per coordinate."""
    q0 = closure_pose()
    return [q0 + rng.uniform(-1.0, 1.0, mdl.N_Q) * np.array(spread) for _ in range(n)]


# =============================================================================
# 1. The mass matrix against central finite differences of the kinetic energy
# =============================================================================


def kinetic_energy_from_points(q, v, h=1e-6):
    """T from the bodies' motion, independently of model.mass_matrix: each
    center of mass's velocity by a central difference of its position along
    q̇, each body's angular velocity from its absolute angle (linear in q)."""
    q = np.asarray(q, dtype=float)
    v = np.asarray(v, dtype=float)
    plus, minus = mdl.Pose(q + h * v), mdl.Pose(q - h * v)
    energy = 0.0
    for body in mdl.MOVING_BODIES:
        m, inertia, _cm = mdl.MASS[body]
        xp = np.array(mdl.Model.center_of_mass(body, plus))
        xm = np.array(mdl.Model.center_of_mass(body, minus))
        vel = (xp - xm) / (2.0 * h)
        omega = (mdl.Model.body_angle(body, q + h * v) - mdl.Model.body_angle(body, q - h * v)) / (2 * h)
        energy += 0.5 * m * float(vel @ vel) + 0.5 * inertia * omega * omega
    return energy


@check
def mass_matrix_against_kinetic_energy():
    """M_ij = ∂²T/∂q̇_i∂q̇_j, T computed from the bodies' point motion and
    differentiated by central second differences in q̇ (exact for a quadratic
    form up to rounding), at script 10's closure and 40 random poses. Also: M
    against script 10's own matrix at its rest pose, and the wippen block
    against `docs/FOUNDATIONS.md` > Relative coordinates for the jack and the repetition lever's closed forms."""
    rng = np.random.default_rng(SEED)
    poses = [closure_pose()] + random_poses(rng, 40, spread=(1.0, 1.0, 1.0, 1.0, 1.0))
    worst = 0.0
    eps = 1.0
    for q in poses:
        m_mat = np.array(mdl.Model.mass_matrix(mdl.Pose(q)))
        for i in range(mdl.N_Q):
            for j in range(mdl.N_Q):
                ei, ej = np.eye(mdl.N_Q)[i] * eps, np.eye(mdl.N_Q)[j] * eps
                fd = (kinetic_energy_from_points(q, ei + ej) - kinetic_energy_from_points(q, ei - ej)
                      - kinetic_energy_from_points(q, -ei + ej)
                      + kinetic_energy_from_points(q, -ei - ej)) / (4 * eps * eps)
                scale = math.sqrt(m_mat[i, i] * m_mat[j, j])
                err = abs(fd - m_mat[i, j]) / scale
                worst = max(worst, err)
                assert err < 1e-8, f"M[{i}][{j}] at {q}: {m_mat[i, j]:.6e} vs FD {fd:.6e}"
        assert np.allclose(m_mat, m_mat.T, rtol=0, atol=0), "M not exactly symmetric"
        assert np.all(np.linalg.eigvalsh(m_mat) > 0), "M not positive definite"
    # script 10's matrix (forward differences, h = 1e-7) at its rest pose
    q10 = closure_pose()
    m10 = np.array(mdl.S10.mass_matrix(list(q10)))
    m_mat = np.array(mdl.Model.mass_matrix(mdl.Pose(q10)))
    rel10 = float(np.max(np.abs(m_mat - m10)) / np.max(np.abs(m10)))
    assert rel10 < 1e-6, f"script 10's mass matrix differs by {rel10:.2e}"
    # `docs/FOUNDATIONS.md` > Relative coordinates for the jack and the repetition lever: a child pinned at r on the wippen, center of mass at u
    # from its pin: coupling I_cm + m(|u|² + r·u), share of the wippen's
    # diagonal I_cm + m|r + u|². Key and hammer are decoupled, jack and lever
    # siblings uncoupled, exactly.
    worst36 = 0.0
    for q in poses:
        m_mat = np.array(mdl.Model.mass_matrix(mdl.Pose(q)))
        mw, iw, (cwx, cwy) = mdl.MASS["w"]
        diag_w = iw + mw * (cwx * cwx + cwy * cwy)
        for child, idx, pin in (("j", mdl.J, mdl.P_J), ("r", mdl.R, mdl.P_R)):
            m, inertia, (cx, cy) = mdl.MASS[child]
            a = q[idx]
            ux, uy = math.cos(a) * cx - math.sin(a) * cy, math.sin(a) * cx + math.cos(a) * cy
            rx, ry = pin
            coupling = inertia + m * (ux * ux + uy * uy + rx * ux + ry * uy)
            diag_w += inertia + m * ((rx + ux) ** 2 + (ry + uy) ** 2)
            worst36 = max(worst36, abs(m_mat[mdl.W, idx] - coupling) / coupling)
        worst36 = max(worst36, abs(m_mat[mdl.W, mdl.W] - diag_w) / diag_w)
        for i, j in ((mdl.K, mdl.W), (mdl.K, mdl.J), (mdl.K, mdl.R), (mdl.K, mdl.H), (mdl.W, mdl.H),
                     (mdl.J, mdl.H), (mdl.R, mdl.H), (mdl.J, mdl.R)):
            assert m_mat[i, j] == 0.0, f"M[{i}][{j}] should be exactly zero"
    assert worst36 < 1e-12, f"the closed forms of `docs/FOUNDATIONS.md` > Relative coordinates for the jack and the repetition lever differ by {worst36:.2e}"
    return (f"41 poses, worst |ΔM_ij|/sqrt(M_ii M_jj) = {worst:.1e} (bound 1e-8); script 10's"
            f" matrix within {rel10:.1e}; the relative coordinates' closed forms within {worst36:.1e}; the eight"
            f" structural zeros exact")


# =============================================================================
# 2. Every contact gradient against central finite differences of its gap
# =============================================================================


@check
def contact_gradients_against_finite_differences():
    """∂g/∂q by the lever-arm identity (`docs/FOUNDATIONS.md` > The lever arm, and the gradient of a gap, the parent's
    coordinate included for the jack's and the lever's points) against a
    central difference of the gap (step 1e-6 rad), all thirteen contacts, all
    five coordinates, at script 10's closure and 300 random poses. A stencil
    across a hybrid's two lines inside its wedge (a kink, deeper than R + r) is
    skipped and counted. Also: every coordinate is acted on by some contact."""
    rng = np.random.default_rng(SEED + 1)
    poses = [closure_pose()] + random_poses(rng, 300)
    h = 1e-6  # rounding and truncation balance near here
    worst, compared, skipped = 0.0, 0, 0
    for q in poses:
        kin = mdl.Model.contact_kinematics(q)
        for i in range(mdl.N_Q):
            e = np.zeros(mdl.N_Q)
            e[i] = h
            kp, km = mdl.Model.contact_kinematics(q + e), mdl.Model.contact_kinematics(q - e)
            for c in mdl.CONTACTS:
                branches = {kp[c][5], km[c][5]}
                if branches == {"line0", "line1"}:
                    skipped += 1
                    continue
                fd = (kp[c][0] - km[c][0]) / (2 * h)
                an = kin[c][3][i]
                err = abs(fd - an)
                worst = max(worst, err / (1e-3 + abs(an)))
                compared += 1
                assert err < 1e-8 + 1e-6 * abs(an), (
                    f"contact {c}, coordinate {i}, q={q}: analytic {an:.9e}, FD {fd:.9e}")
    kin = mdl.Model.contact_kinematics(closure_pose())
    for i in range(mdl.N_Q):
        assert any(abs(kin[c][3][i]) > 1e-3 for c in mdl.CONTACTS), f"coordinate {i} untouched"
    return (f"{compared} partial derivatives compared, worst error {worst:.1e} relative to"
            f" (1 mm/rad + |∂g/∂q|); {skipped} stencils across a wedge's bisector skipped;"
            f" every coordinate has a lever arm of at least 1 mm/rad at rest")


@check
def slip_and_normal_speeds_against_material_points():
    """The relative velocity of the two bodies' material points at the contact
    point, by central differences of their positions, projected on the unit
    normal (times the printed normal's norm, model.py A10, it must equal ∂g/∂q:
    the lever-arm identity read as a speed) and on the unit tangent (it must
    equal the slip Jacobian contact friction uses)."""
    rng = np.random.default_rng(SEED + 2)
    poses = [closure_pose()] + random_poses(rng, 100)
    h = 1e-7
    worst = 0.0
    for q in poses:
        pose = mdl.Pose(q)
        kin = mdl.Model.contact_kinematics(q)
        for c in mdl.CONTACTS:
            spec = mdl.SPECS[c]
            _gap, (nx, ny), (px, py), grad, jt, _branch = kin[c]
            local_a = pose.to_local(spec.a, px, py)
            local_b = pose.to_local(spec.b, px, py)
            for i in range(mdl.N_Q):
                e = np.zeros(mdl.N_Q)
                e[i] = h
                pp, pm = mdl.Pose(q + e), mdl.Pose(q - e)
                va = (np.array(pp.to_world(spec.a, *local_a)) - np.array(pm.to_world(spec.a, *local_a))) / (2 * h)
                vb = (np.array(pp.to_world(spec.b, *local_b)) - np.array(pm.to_world(spec.b, *local_b))) / (2 * h)
                rel = va - vb
                # the gap of a line is projected on its printed normal (A10)
                normal = (rel[0] * nx + rel[1] * ny) * spec.gap_scale
                tangent = -rel[0] * ny + rel[1] * nx
                e_n, e_t = abs(normal - grad[i]), abs(tangent - jt[i])
                worst = max(worst, e_n, e_t)
                assert e_n < 1e-8 and e_t < 1e-8, (
                    f"contact {c}, coordinate {i}: normal {normal:.9e} vs {grad[i]:.9e},"
                    f" tangent {tangent:.9e} vs {jt[i]:.9e}")
    return f"101 poses × 13 contacts × 5 coordinates, worst error {worst:.1e} m/rad"


# =============================================================================
# 3. The hybrid normal, continuous across the arc's ends
# =============================================================================


@check
def hybrid_normal_continuous_at_arc_ends():
    """Contacts 7 (jack) and 10 (lever).
    (a) The primitive: the circle's center just either side of each arc end, at
    radii from inside the arc's circle to well outside it: gap and normal differ
    by O(ε) across the end.
    (b) The assembled contact (contact_geometry and the lever arms, frames
    included): the knuckle's frame placed so that its center sits 0.5 mm into
    the surface just either side of each end of each arc, ε = 1e-9 rad: world
    normal and gradient equal on both sides to 1e-8. Then along real motions:
    turning θj from script 10's closure until the knuckle leaves the jack's arc,
    both ways (onto the flat top, line 0, and onto the side, line 1), and
    turning θr until it leaves the lever's arc, both ways (it rests by the arc's
    a1 end, which both directions cross); each crossing found by bisection to
    1e-13 rad, normal and gradient equal on both sides to 1e-8.
    (c) A scan of 20,001 angles of θj and of θr, ±0.4 rad about the closure,
    wherever the knuckle's center is outside the wedge W (penetration below
    R + r): no step in the normal larger than 50 × the step in angle."""
    worst_prim = 0.0
    for c in (7, 10):
        spec = mdl.SPECS[c]
        big_r, rk = spec.arc_radius, spec.radius
        for end in (spec.arc_start, spec.arc_end):
            for rho in (0.3 * big_r, big_r, big_r + rk - 1e-4, big_r + rk, big_r + rk + 1e-3,
                        4 * (big_r + rk)):
                for eps in (1e-6, 1e-9):
                    sides = [mdl.hybrid_signed_distance(rho * math.cos(a), rho * math.sin(a), big_r,
                                                        spec.arc_start, spec.arc_end)
                             for a in (end - eps, end + eps)]
                    assert {sides[0][3], sides[1][3]} & {"arc"} and sides[0][3] != sides[1][3], (
                        f"contact {c}: the end at {end} is not between the arc and a line")
                    dn = math.hypot(sides[0][1] - sides[1][1], sides[0][2] - sides[1][2])
                    ds = abs(sides[0][0] - sides[1][0])
                    worst_prim = max(worst_prim, dn / eps)
                    assert dn <= 1.5 * eps + 1e-15 and ds <= 1.5 * rho * eps + 1e-15, (
                        f"contact {c}, end {end}, rho {rho}: jump {dn:.2e} in n, {ds:.2e} in gap")
    # (b) through the assembled contact: placements at both ends of both arcs
    q0 = closure_pose()
    worst_placed = 0.0
    for c in (7, 10):
        spec = mdl.SPECS[c]
        rho = spec.arc_radius + spec.radius - 5e-4
        for end in (spec.arc_start, spec.arc_end):
            sides = []
            for a in (end - 1e-9, end + 1e-9):
                pose = mdl.Pose(q0)
                cx, cy = pose.to_world(spec.a, spec.arc_center[0] + rho * math.cos(a),
                                       spec.arc_center[1] + rho * math.sin(a))
                kx, ky = pose.to_world(spec.b, *spec.circle)
                ox, oy, cs, sn = pose.frame[spec.b]
                pose.frame[spec.b] = (ox + cx - kx, oy + cy - ky, cs, sn)
                gap, nx, ny, px, py, branch = mdl.contact_geometry(spec, pose)
                grad, _jt = mdl.lever_arms(spec, pose, nx, ny, px, py)
                assert abs(gap + 5e-4) < 1e-12, f"contact {c}: placement gap {gap}"
                sides.append((np.array([nx, ny]), np.array(grad), branch))
            assert sides[0][2] != sides[1][2] and "arc" in (sides[0][2], sides[1][2])
            dn = float(np.max(np.abs(sides[0][0] - sides[1][0])))
            dg = float(np.max(np.abs(sides[0][1] - sides[1][1])))
            worst_placed = max(worst_placed, dn, dg)
            assert dn < 1e-8 and dg < 1e-8, f"contact {c}, end {end}: jump {dn:.1e} in n, {dg:.1e} in ∇g"
    # (b) through the assembled contact, along real motions
    crossings = []
    for c, coord in ((7, mdl.J), (10, mdl.R)):
        start_branch = mdl.Model.contact_kinematics(q0)[c][5]
        assert start_branch == "arc", f"contact {c} is not on its arc at script 10's closure"
        for direction in (+1.0, -1.0):
            q = q0.copy()
            lo = 0.0
            hi = None
            for k in range(1, 2001):
                q[coord] = q0[coord] + direction * k * 1e-3
                if mdl.Model.contact_kinematics(q)[c][5] != "arc":
                    hi = k * 1e-3
                    break
                lo = k * 1e-3
            assert hi is not None, f"contact {c}: no arc end within 2 rad of θ{coord}"
            for _ in range(60):
                mid = 0.5 * (lo + hi)
                q[coord] = q0[coord] + direction * mid
                if mdl.Model.contact_kinematics(q)[c][5] == "arc":
                    lo = mid
                else:
                    hi = mid
            qa, qb = q0.copy(), q0.copy()
            qa[coord] += direction * lo
            qb[coord] += direction * hi
            ka, kb = mdl.Model.contact_kinematics(qa)[c], mdl.Model.contact_kinematics(qb)[c]
            dn = float(np.max(np.abs(np.array(ka[1]) - np.array(kb[1]))))
            dg = float(np.max(np.abs(np.array(ka[3]) - np.array(kb[3]))))
            assert dn < 1e-8 and dg < 1e-8, f"contact {c}: jump {dn:.1e} in n, {dg:.1e} in ∇g at its end"
            crossings.append(f"#{c} {ka[5]}->{kb[5]} at Δθ={direction * hi:+.4f} rad"
                             f" (Δn {dn:.0e}, Δ∇g {dg:.0e})")
    # (c) scans
    worst_scan, scanned = 0.0, 0
    for c, coord in ((7, mdl.J), (10, mdl.R)):
        spec = mdl.SPECS[c]
        angles = np.linspace(-0.4, 0.4, 20001)
        step = angles[1] - angles[0]
        prev = None
        for a in angles:
            q = q0.copy()
            q[coord] += a
            gap, n, _p, _g, _jt, _branch = mdl.Model.contact_kinematics(q)[c]
            if gap <= -(spec.arc_radius + spec.radius):  # inside the wedge W
                prev = None
                continue
            n = np.array(n)
            if prev is not None:
                jump = float(np.linalg.norm(n - prev))
                worst_scan = max(worst_scan, jump / step)
                assert jump <= 50 * step, f"contact {c}: normal jumps by {jump:.2e} at Δθ={a:+.5f}"
            prev = n
            scanned += 1
    return (f"primitive: worst |Δn|/ε = {worst_prim:.2f} (bound 1.5); placed at both ends of"
            f" both arcs: worst jump {worst_placed:.0e}; along θj and θr: "
            + "; ".join(crossings)
            + f"; scan: {scanned} poses, worst |Δn|/Δθ = {worst_scan:.1f} (bound 50)")


@check
def signed_gap_keeps_its_sign():
    """Every contact, through penetrations deeper than its circle's radius:
    from script 10's closure, body B's frame is translated along n (which
    points from B to A), into A, by up to three times the circle's radius (for
    the hybrids, up to R + r and then 3 mm past it, into the wedge). The gap
    must fall exactly as the depth grows while the nearest feature stays the
    same (at the rate |n|, which is 1 except for the four printed vectors of
    model.py A10, whose norms this also checks), and stay negative to the end
    (`docs/FOUNDATIONS.md` > Seventeen contacts per note: a compliant law needs a signed gap)."""
    q0 = closure_pose()
    lines = []
    norms = {}
    for c in mdl.CONTACTS:
        spec = mdl.SPECS[c]
        pose = mdl.Pose(q0)
        gap0, nx, ny, _px, _py, _branch = mdl.contact_geometry(spec, pose)
        assert abs(math.hypot(nx, ny) - 1.0) < 1e-15, f"contact {c}: normal not unit"
        norm = spec.gap_scale
        if spec.kind in ("lc", "cl"):
            assert norm == math.hypot(*spec.line_normal) and 0.9995 < norm <= 1.0
            norms[spec.points[0] if spec.kind == "lc" else spec.points[1]] = norm
        else:
            assert norm == 1.0
        radius = min(spec.radius_a, spec.radius_b) if spec.kind == "cc" else spec.radius
        linear_to = (spec.arc_radius + spec.radius) if spec.kind == "hc" else 3.0 * radius
        deepest = linear_to + (3e-3 if spec.kind == "hc" else 0.0)
        worst = 0.0
        for depth in np.linspace(0.0, deepest, 601):
            moved = mdl.Pose(q0)
            ox, oy, cs, sn = moved.frame[spec.b]
            moved.frame[spec.b] = (ox + depth * nx, oy + depth * ny, cs, sn)
            gap = mdl.contact_geometry(spec, moved)[0]
            if depth > max(gap0, 0.0):
                assert gap < 0.0, f"contact {c}: gap {gap:.3e} >= 0 at depth {depth:.4e}"
            if depth <= linear_to:
                worst = max(worst, abs(gap - (gap0 - norm * depth)))
        assert worst < 1e-12, f"contact {c}: gap departs from linear by {worst:.1e}"
        lines.append(f"#{c} to {deepest * 1e3:.1f} mm (r {radius * 1e3:.1f})")
    short = {p: n for p, n in norms.items() if n < 1.0}
    assert sorted(short) == ["Pk5", "Pk6", "Pr5", "Pw4"], short
    return ("negative, and linear in depth, for all 13: " + ", ".join(lines)
            + "; printed normals of norm below 1: "
            + ", ".join(f"{p} {n:.5f}" for p, n in sorted(short.items())))


# =============================================================================
# 4. Energy bounded in free flight
# =============================================================================


@check
def energy_bounded_in_free_flight():
    """Contacts, contact friction, axis friction, springs and gravity off; all
    five bodies spinning (the wippen block's mass matrix changes with θj and
    θr). Both integrators, 0.2 s, at dt = 10 and 5 µs. The kinetic energy must
    stay within 1e-3 of its start, and its worst departure must fall at least
    1.6 times when dt halves (first order, as semi-implicit Euler on a
    configuration-dependent mass matrix gives, `docs/FOUNDATIONS.md` > Levels level 2). The
    key and the hammer, on constant inertia with nothing acting, must keep
    their speeds to rounding."""
    model = mdl.Model(contacts=False, contact_friction=False, axis_friction=False,
                      springs=False, gravity=False)
    v0 = np.array([3.0, -5.0, 40.0, -30.0, 8.0])
    out = []
    for name in ("A", "B"):
        errors = []
        for dt in (1e-5, 5e-6):
            state = itg.State(0.0, closure_pose(), v0)
            e0 = model.kinetic_energy(state.q, state.v)
            worst = [0.0]

            def watch(_before, after, _info, e0=e0, worst=worst):
                worst[0] = max(worst[0], abs(model.kinetic_energy(after.q, after.v) - e0) / e0)

            final, steps, finite = itg.simulate(model, state, dt, int(round(0.2 / dt)),
                                                itg.FreeDrive(), name, observe=watch)
            assert finite and steps == int(round(0.2 / dt))
            assert abs(final.v[mdl.K] - v0[mdl.K]) < 1e-12 and abs(final.v[mdl.H] - v0[mdl.H]) < 1e-12
            errors.append(worst[0])
        assert errors[0] < 1e-3, f"{name}: energy departs by {errors[0]:.2e} at 10 µs"
        assert errors[0] / errors[1] > 1.6, f"{name}: departure not first order ({errors})"
        out.append(f"{name}: {errors[0]:.2e} at 10 µs, {errors[1]:.2e} at 5 µs"
                   f" (ratio {errors[0] / errors[1]:.2f})")
    return "relative kinetic-energy departure, worst over 0.2 s: " + "; ".join(out)


# =============================================================================
# 5. The rest pose equal to script 10's
# =============================================================================


@check
def rest_pose_equals_script_10():
    """Script 10's closure (gaps of contacts 1, 3, 5, 7 and 10 at zero) solved
    with this model's gaps and analytic gradients, from the starting point
    script 10's own search converges from: the same pose, every gap the same,
    and every number script 10 prints the same to its printed precision (q to
    1e-4 rad, gaps to 0.01 mm), among them the three quoted for its rest pose:
    the hammer 46.99 mm from the string, the shank 0.93 mm above its rail, and
    0.88 mm of overlap at contact 6."""
    q10, start = mdl.script_10_rest_pose()
    q, iterations = mdl.closure_pose(start)
    dq = float(np.max(np.abs(q - q10)))
    assert dq < 1e-12, f"closure differs from script 10's by {dq:.1e} rad"
    mine = mdl.Model.gaps(q)
    theirs = mdl.S10.gaps(list(q10))
    dg = max(abs(mine[c - 1] - theirs[c]) for c in mdl.CONTACTS)
    assert dg < 1e-12, f"gaps differ by {dg:.1e} m"
    for c in mdl.CONTACTS:
        assert round(mine[c - 1] * 1e3, 2) == round(theirs[c] * 1e3, 2), f"contact {c} printed"
    assert all(round(a, 4) == round(b, 4) for a, b in zip(q, q10))
    printed = {12: 46.99, 13: 0.93, 6: -0.88}
    for c, mm in printed.items():
        assert round(mine[c - 1] * 1e3, 2) == mm, f"contact {c}: {mine[c - 1] * 1e3:.2f} mm"
    for c in mdl.CLOSED_AT_REST:
        assert abs(mine[c - 1]) < 1e-12
    return (f"Newton in {iterations} iterations; |Δq| = {dq:.1e} rad, worst |Δgap| = {dg:.1e} m;"
            f" q = ({', '.join(f'{x:.4f}' for x in q)}); hammer {mine[11] * 1e3:.2f} mm from the"
            f" string, shank {mine[12] * 1e3:.2f} mm above its rail, {mine[5] * 1e3:.2f} mm at"
            f" contact 6")


# =============================================================================
# 6. Axis friction holds a body exactly still below its bound
# =============================================================================


@check
def axis_friction_holds_exactly():
    """(a) Contacts, gravity and springs off: a constant torque of ±0.9 A on one
    joint at a time, then on all five at once, from rest, 1000 steps of 10 µs:
    every coordinate keeps its exact starting value and every speed is exactly
    zero, in both integrators. Counter-test: 1.1 A on the key (decoupled) slides
    it with exactly (1.1 A - A)/M_kk of acceleration. (b) The whole action, all
    forces on, at its frictionless equilibrium: a finger force at Pk2 of 0.8 A_k
    over the arm leaves all five coordinates bit-identical over 1000 steps in
    both integrators; 1.2 times the same moves the key."""
    dt, n = 1e-5, 1000
    bare = mdl.Model(contacts=False, contact_friction=False, springs=False, gravity=False)
    q0 = closure_pose()
    cases = []
    for i in range(mdl.N_Q):
        for sign in (+1.0, -1.0):
            tau = np.zeros(mdl.N_Q)
            tau[i] = sign * 0.9 * mdl.AXIS_FRICTION_NM[i]
            cases.append(tau)
    cases.append(0.9 * mdl.AXIS_FRICTION_NM * np.array([1, -1, 1, -1, 1]))
    for name in ("A", "B"):
        for tau in cases:
            state = itg.State(0.0, q0, np.zeros(mdl.N_Q))
            final, steps, finite = itg.simulate(bare, state, dt, n, itg.TorqueDrive(tau), name)
            assert finite and steps == n
            assert np.array_equal(final.q, q0), f"{name}, torque {tau}: moved by {final.q - q0}"
            assert np.array_equal(final.v, np.zeros(mdl.N_Q)), f"{name}: speeds {final.v}"
        tau = np.zeros(mdl.N_Q)
        tau[mdl.K] = 1.1 * mdl.AXIS_FRICTION_NM[mdl.K]
        final, _steps, _finite = itg.simulate(bare, itg.State(0.0, q0, np.zeros(mdl.N_Q)), dt, n,
                                              itg.TorqueDrive(tau), name)
        m_kk = mdl.Model.mass_matrix(mdl.Pose(q0))[mdl.K][mdl.K]
        expected = n * dt * 0.1 * mdl.AXIS_FRICTION_NM[mdl.K] / m_kk
        assert abs(final.v[mdl.K] - expected) < 1e-12 * expected + 1e-15, (
            f"{name}: key slides at {final.v[mdl.K]:.9e}, expected {expected:.9e}")
    # (b) the assembled action
    full = mdl.Model()
    state0, _iterations, residual = sc.rest_state(full)
    arm = full.finger_arm(state0.q)
    threshold_N = mdl.AXIS_FRICTION_NM[mdl.K] / arm
    for name in ("A", "B"):
        held = itg.ForceDrive(lambda t: 0.8 * threshold_N)
        final, steps, finite = itg.simulate(full, state0.copy(), dt, n, held, name)
        assert finite and steps == n
        assert np.array_equal(final.q, state0.q), f"{name}: the action moved by {final.q - state0.q}"
        assert np.array_equal(final.v, np.zeros(mdl.N_Q))
        moving = itg.ForceDrive(lambda t: 1.2 * threshold_N)
        final, _steps, _finite = itg.simulate(full, state0.copy(), dt, 200, moving, name)
        assert final.q[mdl.K] < state0.q[mdl.K], f"{name}: 1.2 x the threshold did not move the key"
    return (f"(a) {len(cases)} torques x 2 integrators held bit-identical, the key sliding at"
            f" exactly the predicted rate above its bound; (b) the action at its equilibrium"
            f" (residual {residual:.0e} N·m) held bit-identical under {0.8 * threshold_N * 1e3:.1f} mN"
            f" ({0.8 * threshold_N / mdl.GRAVITY_M_S2 * 1e3:.2f} g) at Pk2 in A and B, moved"
            f" by {1.2 * threshold_N * 1e3:.1f} mN")


# =============================================================================
# What the six rely on
# =============================================================================


@check
def velocity_terms_against_lagrange():
    """h(q, q̇), derived analytically from the convective accelerations (model.py),
    against Lagrange's Σ (∂M_ij/∂q_k - ½ ∂M_jk/∂q_i) q̇_j q̇_k, ∂M/∂q by central
    differences of M, at 40 random states."""
    rng = np.random.default_rng(SEED + 3)
    worst = 0.0
    hq = 1e-4  # M is trigonometric in q: truncation ~1e-15 here, rounding ~1e-9 relative
    for q in random_poses(rng, 40, spread=(1.0, 1.0, 1.0, 1.0, 1.0)):
        v = rng.uniform(-20, 20, mdl.N_Q)
        dm = []
        for k in range(mdl.N_Q):
            e = np.zeros(mdl.N_Q)
            e[k] = hq
            dm.append((np.array(mdl.Model.mass_matrix(mdl.Pose(q + e)))
                       - np.array(mdl.Model.mass_matrix(mdl.Pose(q - e)))) / (2 * hq))
        lagrange = np.zeros(mdl.N_Q)
        for i in range(mdl.N_Q):
            for j in range(mdl.N_Q):
                for k in range(mdl.N_Q):
                    lagrange[i] += (dm[k][i, j] - 0.5 * dm[i][j, k]) * v[j] * v[k]
        analytic = np.array(mdl.Model.velocity_terms(mdl.Pose(q), v))
        scale = float(np.max(np.abs(lagrange))) + 1e-12
        err = float(np.max(np.abs(analytic - lagrange))) / scale
        worst = max(worst, err)
        assert err < 1e-7, f"h differs by {err:.1e} at q={q}, v={v}"
    return f"40 states, worst |Δh| / max|h| = {worst:.1e} (bound 1e-7)"


@check
def conservative_forces_against_potential():
    """Gravity, the springs and every contact's elastic force at rest against
    -∇V, V = model.potential_energy, by central differences, at 60 poses near
    the frictionless equilibrium (contacts loaded); poses with a gap within
    1 µm of zero, where V's curvature jumps, are redrawn."""
    rng = np.random.default_rng(SEED + 4)
    model = mdl.Model()
    q_eq = sc.rest_state(model)[0].q
    worst, drawn = 0.0, 0
    h = 1e-7
    poses = 0
    while poses < 60:
        q = q_eq + rng.uniform(-1.0, 1.0, mdl.N_Q) * np.array([2e-3, 2e-3, 1e-2, 1e-2, 1e-2])
        drawn += 1
        if np.min(np.abs(mdl.Model.gaps(q))) < 1e-6:
            continue
        poses += 1
        force = model.evaluate(q, np.zeros(mdl.N_Q)).Q
        fd = np.zeros(mdl.N_Q)
        for i in range(mdl.N_Q):
            e = np.zeros(mdl.N_Q)
            e[i] = h
            fd[i] = -(model.potential_energy(q + e) - model.potential_energy(q - e)) / (2 * h)
        err = float(np.max(np.abs(force - fd))) / (float(np.max(np.abs(force))) + 1e-6)
        worst = max(worst, err)
        assert err < 1e-5, f"Q differs from -∇V by {err:.1e} at {q}: {force} vs {fd}"
    return f"60 poses ({drawn - 60} redrawn), worst |Q + ∇V| / max|Q| = {worst:.1e} (bound 1e-5)"


@check
def contact_laws_as_printed():
    """Eqs. 2.4-2.5 with the clamp, and Eq. 2.10 with its bracket, as the record
    prints them: the normal force at a few penetrations and speeds; zero below
    the separation speed -1/D, never negative; and the friction law's
    static peak over its kinetic coefficient, 1.69, 1.72 and 1.13 at contacts 4,
    7 and 10 and none at contact 1, as `docs/FOUNDATIONS.md` > Friction [E, 07] gives them."""
    for c in mdl.CONTACTS:
        spec = mdl.SPECS[c]
        a, b, cc = spec.law
        d_s_m = spec.damping
        for delta in (1e-5, 1e-4, 5e-4):
            for rate in (-0.5 / d_s_m, 0.0, 0.3):
                f, _k, _c = mdl.normal_force(spec, delta, rate)
                expected = max(0.0, (a * delta ** 3 + b * delta ** 2 + cc * delta) * (1 + d_s_m * rate))
                assert abs(f - expected) <= 1e-12 * abs(expected), f"contact {c}"
            for rate in (-1.0001 / d_s_m, -2.0 / d_s_m):  # -1/D itself rounds either way
                assert mdl.normal_force(spec, delta, rate)[0] == 0.0, f"contact {c}: no clamp"
        assert mdl.normal_force(spec, -1e-4, 1.0)[0] == 0.0
    peaks = {}
    x = np.linspace(1e-4, 10.0, 200001)
    for c, law in mdl.FRICTION_LAW.items():
        mu = np.array([mdl.friction_coefficient(law, s * mdl.FRICTION_THRESHOLD_SPEED_M_S)[0] for s in x])
        peaks[c] = float(np.max(mu)) / law[0]
        dmu_fd = (mdl.friction_coefficient(law, 1.3e-5 + 1e-12)[0]
                  - mdl.friction_coefficient(law, 1.3e-5 - 1e-12)[0]) / 2e-12
        assert abs(dmu_fd - mdl.friction_coefficient(law, 1.3e-5)[1]) < 1e-4 * abs(dmu_fd)
        for s in (-3e-5, 3e-5):  # odd, so friction always opposes slip
            assert mdl.friction_coefficient(law, s)[0] * s > 0
    expected = {1: 1.00, 4: 1.69, 7: 1.72, 10: 1.13}
    for c, ratio in expected.items():
        assert round(peaks[c], 2) == ratio, f"contact {c}: peak ratio {peaks[c]:.3f}"
    return ("normal law and clamp exact at 13 x 9 points; static peak / A: "
            + ", ".join(f"#{c} {peaks[c]:.3f}" for c in sorted(peaks)))


@check
def jack_spring_sign_holds_the_jack():
    """The chosen sign of the jack spring (model.py A7), counter-clockwise on θj:
    at script 10's closure a positive θj closes the jack's rest (contact 5) and
    the jack/knuckle (contact 7), the let-off button's force turns the jack the
    other way (contact 8), and at the frictionless equilibrium the spring holds
    the jack loaded against both its rest and the knuckle."""
    kin = mdl.Model.contact_kinematics(closure_pose())
    g5, g7, g8 = kin[5][3][mdl.J], kin[7][3][mdl.J], kin[8][3][mdl.J]
    assert mdl.JACK_SPRING_TORQUE_NM > 0.0
    assert g5 < 0.0 and g7 < 0.0, "a counter-clockwise jack would not close its rest and knuckle"
    assert g8 < 0.0, "the button would not turn the jack clockwise"
    ev = mdl.Model().evaluate(sc.rest_state()[0].q, np.zeros(mdl.N_Q))
    assert ev.fn[5 - 1] > 0.0 and ev.fn[7 - 1] > 0.0
    return (f"∂g/∂θj: rest {g5 * 1e3:+.2f}, knuckle {g7 * 1e3:+.2f}, button {g8 * 1e3:+.2f} mm/rad;"
            f" at equilibrium the rest carries {ev.fn[4]:.3f} N and the knuckle {ev.fn[6]:.3f} N")


@check
def box_solver_against_enumeration():
    """The axis-friction solver (PGS, then the finishing pass) against the exact
    solution found by enumerating all 3⁵ stick/slide patterns, on 300 random
    box problems with a mass-matrix-like spread of scales; some joints
    frictionless."""
    rng = np.random.default_rng(SEED + 5)
    worst, sweeps = 0.0, 0
    for _ in range(300):
        n = mdl.N_Q
        basis = rng.normal(size=(n, n))
        w_mat = basis @ basis.T + 0.05 * np.eye(n)
        scale = 10.0 ** rng.uniform(-2, 2, n)
        w_mat = w_mat * np.outer(scale, scale)
        v_free = rng.normal(size=n) * scale
        bound = np.abs(rng.normal(size=n)) / scale
        bound[rng.uniform(size=n) < 0.15] = 0.0
        lam, v, report = itg.solve_axis_friction(w_mat, v_free, bound, np.zeros(n), 1e-13, 2000)
        sweeps = max(sweeps, report[0])
        exact = None
        for pattern in np.ndindex(*(3,) * n):
            fixed = {}
            free = []
            for i, p in enumerate(pattern):
                if bound[i] == 0.0:
                    if p:
                        break
                    fixed[i] = 0.0
                elif p == 0:
                    free.append(i)
                else:
                    fixed[i] = -bound[i] if p == 1 else bound[i]
            else:
                trial = np.zeros(n)
                for i, val in fixed.items():
                    trial[i] = val
                if free:
                    rest = [i for i in range(n) if i not in free]
                    rhs = -(v_free[free] + w_mat[np.ix_(free, rest)] @ trial[rest])
                    trial[free] = np.linalg.solve(w_mat[np.ix_(free, free)], rhs)
                speeds = v_free + w_mat @ trial
                ok = all(abs(trial[i]) <= bound[i] * (1 + 1e-12) for i in free)
                for i, p in enumerate(pattern):
                    if bound[i] > 0 and p == 1:
                        ok = ok and speeds[i] >= 0.0
                    if bound[i] > 0 and p == 2:
                        ok = ok and speeds[i] <= 0.0
                if ok:
                    exact = trial
                    break
        assert exact is not None
        err = float(np.max(np.abs(lam - exact) / np.maximum(bound, 1e-300)) if np.any(bound) else 0)
        worst = max(worst, err)
        assert err < 1e-8, f"impulses differ by {err:.1e} of their bound"
        assert report[2] < 1e-9, f"residual {report[2]:.1e}"
    return f"300 problems, worst |Δλ| / bound = {worst:.1e}, most PGS sweeps {sweeps}"


@check
def integrators_agree_on_a_forte_blow():
    """Both integrators on the forte-like force drive (scenarios.py), up to the
    strike, at dt = 10 and 5 µs: the hammer head's speed at the strike and the
    hammer-to-string distance at let-off agree between A and B to 0.1 % and
    0.02 mm, and move by less than that when dt halves."""
    results = {}
    for name in ("A", "B"):
        for dt in (1e-5, 5e-6):
            let_off, strike = sc.strike_on_forte(name, dt)
            assert let_off is not None and strike is not None, f"{name} at {dt}: no let-off or strike"
            results[name, dt] = (let_off[1], strike[1])
    for dt in (1e-5, 5e-6):
        (ga, va), (gb, vb) = results["A", dt], results["B", dt]
        assert abs(va - vb) < 1e-3 * va and abs(ga - gb) < 2e-5, f"A and B differ at {dt}: {results}"
    for name in ("A", "B"):
        (g1, v1), (g2, v2) = results[name, 1e-5], results[name, 5e-6]
        assert abs(v1 - v2) < 1e-3 * v2 and abs(g1 - g2) < 2e-5, f"{name} not converged: {results}"
    return "; ".join(f"{name} {dt * 1e6:.0f} µs: let-off {g * 1e3:.3f} mm, strike {v:.4f} m/s"
                     for (name, dt), (g, v) in results.items())


# =============================================================================
# The model re-derived from the record; directions and signs; the action in motion
# =============================================================================

RECORD_PATH = (Path(__file__).resolve().parents[2] / "data" / "params"
               / "hirschkorn-boston-gp178-note52.json")


class RecordKinematics:
    """The record read afresh and the action assembled with complex numbers,
    sharing no code with model.py or script 10: the frames from the record's
    own list of pivots; every gap from the two points Table A.8 joins and their
    shapes; the hybrid's signed distance by projecting the knuckle's center on
    the wedge W (model.py A9) instead of choosing a branch; the kinetic energy
    from the motion of each center of mass; gravity's potential from its
    height. A line's gap is projected on its printed normal, as script 10 and
    model.py do (model.py A10)."""

    BODIES = {"ground": "g", "key": "k", "wippen": "w", "jack": "j",
              "repetition_lever": "r", "hammer": "h"}

    def __init__(self):
        rec = json.loads(RECORD_PATH.read_text(encoding="utf-8"))

        def v(r):
            return r["value"]

        self.points = {}
        for body, code in self.BODIES.items():
            for name, p in rec["geometry"][body]["points"].items():
                d = {"body": code, "kind": p["description"], "c": complex(v(p["x"]), v(p["y"]))}
                if "radius" in p:
                    d["r"] = v(p["radius"])
                if "normal" in p:
                    d["n"] = complex(v(p["normal"]["x"]), v(p["normal"]["y"]))
                if "arc" in p:
                    d["arc"] = (v(p["arc"]["start"]), v(p["arc"]["end"]))
                self.points[name] = d
        self.mass = {}
        for body, code in self.BODIES.items():
            if code != "g":
                b = rec["mass_properties"]["bodies"][body]
                cm = b["center_of_mass"]
                self.mass[code] = (v(b["mass"]), v(b["moment_of_inertia"]),
                                   complex(v(cm["x"]), v(cm["y"])))
        self.pins = {self.BODIES[p["body"]]: self.points[p["turns_on"]]["c"]
                     for p in rec["frame_convention"]["pivots"]}
        self.rows = {int(n): r["points"] for n, r in rec["contacts"]["rows"].items()}

    def frames(self, q):
        """Origin and absolute angle of each body's frame; the jack's and the
        lever's pins ride on the wippen."""
        tk, tw, tj, tr, th = (float(x) for x in q)
        ew = cmath.exp(1j * tw)
        o_w = self.pins["w"]
        return {"g": (0j, 0.0), "k": (self.pins["k"], tk), "w": (o_w, tw),
                "j": (o_w + ew * self.pins["j"], tw + tj),
                "r": (o_w + ew * self.pins["r"], tw + tr), "h": (self.pins["h"], th)}

    @staticmethod
    def world(f, body, z):
        o, a = f[body]
        return o + cmath.exp(1j * a) * z

    @staticmethod
    def dot(a, b):
        return a.real * b.real + a.imag * b.imag

    @classmethod
    def hybrid_signed_distance(cls, x, radius, a0, a1):
        """Signed distance from x (relative to the arc's center, hybrid frame)
        to W ⊕ disc(radius), W = {y: y·n0 <= 0, y·n1 <= 0}. Outside W: the
        distance to the nearest point of W, among the apex and the feet of x
        on W's two edge rays. Inside: minus the distance to W's boundary,
        which for an intersection of half-planes is the least distance to
        their lines."""
        n0, n1 = cmath.exp(1j * a0), cmath.exp(1j * a1)
        e0 = -1j * n0  # W's edge on line 0's side: perpendicular to n0, in n1's half-plane
        if cls.dot(e0, n1) > 0:
            e0 = -e0
        e1 = 1j * n1
        if cls.dot(e1, n0) > 0:
            e1 = -e1
        if cls.dot(x, n0) <= 0.0 and cls.dot(x, n1) <= 0.0:
            return -min(abs(cls.dot(x, n0)), abs(cls.dot(x, n1))) - radius
        nearest = [0j] + [cls.dot(x, e) * e for e in (e0, e1) if cls.dot(x, e) > 0.0]
        return min(abs(x - p) for p in nearest) - radius

    def gap(self, q, c):
        f = self.frames(q)
        pa, pb = (self.points[name] for name in self.rows[c])
        kinds = (pa["kind"], pb["kind"])
        if kinds == ("Circle Contact", "Circle Contact"):
            centers = self.world(f, pa["body"], pa["c"]) - self.world(f, pb["body"], pb["c"])
            return abs(centers) - pa["r"] - pb["r"]
        if kinds == ("Hybrid Contact", "Circle Contact"):
            x = self.world(f, pb["body"], pb["c"]) - self.world(f, pa["body"], pa["c"])
            x *= cmath.exp(-1j * f[pa["body"]][1])
            return self.hybrid_signed_distance(x, pa["r"], *pa["arc"]) - pb["r"]
        line, circle = (pa, pb) if kinds == ("Line Contact", "Circle Contact") else (pb, pa)
        assert {line["kind"], circle["kind"]} == {"Line Contact", "Circle Contact"}
        normal = cmath.exp(1j * f[line["body"]][1]) * line["n"]
        offset = self.world(f, circle["body"], circle["c"]) - self.world(f, line["body"], line["c"])
        return self.dot(offset, normal) - circle["r"]

    def gaps(self, q):
        return np.array([self.gap(q, c) for c in sorted(self.rows)])

    def kinetic_energy(self, q, v, h=1e-6):
        q, v = np.asarray(q, float), np.asarray(v, float)
        fp, fm = self.frames(q + h * v), self.frames(q - h * v)
        energy = 0.0
        for b, (m, inertia, cm) in self.mass.items():
            vel = (self.world(fp, b, cm) - self.world(fm, b, cm)) / (2 * h)
            omega = (fp[b][1] - fm[b][1]) / (2 * h)
            energy += 0.5 * m * abs(vel) ** 2 + 0.5 * inertia * omega ** 2
        return energy

    def gravity_potential(self, q, g):
        f = self.frames(q)
        return sum(m * g * self.world(f, b, cm).imag for b, (m, _i, cm) in self.mass.items())


_CACHE = {}


def independent():
    if "kinematics" not in _CACHE:
        _CACHE["kinematics"] = RecordKinematics()
    return _CACHE["kinematics"]


def stroke_states():
    """States through the position drive's stroke (scenarios.py), integrator
    B at 10 µs, from rest and then every millisecond to 45 ms: the stroke, the
    toe meeting the button, let-off and the strike."""
    if "stroke" not in _CACHE:
        model = mdl.Model()
        state, _it, _res = sc.rest_state(model)
        drive = sc.make_drive("position", state.q)
        out = [state.copy()]

        def keep(_before, after, _info):
            if abs(after.t * 1e3 - round(after.t * 1e3)) < 1e-6:
                out.append(after.copy())

        itg.simulate(model, state, 1e-5, 4500, drive, "B", observe=keep)
        _CACHE["stroke"] = out
    return _CACHE["stroke"]


@check
def independent_derivation_from_the_record():
    """The model against RecordKinematics (no code shared with model.py or
    script 10): every gap at script 10's closure, at 300 poses around it and
    at the 46 states of a key stroke; the mass matrix at 40 poses, polarized
    from the independent kinetic energy; the (wippen, jack, lever) block
    against its closed forms with the record's masses (`docs/FOUNDATIONS.md` > Relative coordinates for the jack and the repetition lever:
    coupling I_cm + m(|u|² + r·u), own diagonal I_cm + m|u|², share of the
    wippen's diagonal I_cm + m|r + u|², jack and lever uncoupled); every
    analytic gradient against central differences of the independent gap at
    61 poses and at the stroke's states."""
    ind = independent()
    rng = np.random.default_rng(SEED + 10)
    stroke = [s.q for s in stroke_states()]
    poses = [closure_pose()] + random_poses(rng, 300) + stroke
    worst_gap = max(float(np.max(np.abs(mdl.Model.gaps(q) - ind.gaps(q)))) for q in poses)
    assert worst_gap < 1e-12, f"gaps differ from the independent derivation by {worst_gap:.1e} m"
    worst_m, worst_cf = 0.0, 0.0
    eye = np.eye(mdl.N_Q)
    for q in [closure_pose()] + random_poses(rng, 39, spread=(1.0,) * 5):
        m_mat = np.array(mdl.Model.mass_matrix(mdl.Pose(q)))
        mine = np.array([[0.5 * (ind.kinetic_energy(q, eye[i] + eye[j])
                                 - ind.kinetic_energy(q, eye[i] - eye[j]))
                          for j in range(mdl.N_Q)] for i in range(mdl.N_Q)])
        scale = np.sqrt(np.outer(np.diag(m_mat), np.diag(m_mat)))
        worst_m = max(worst_m, float(np.max(np.abs(m_mat - mine) / scale)))
        mw, iw, cw = ind.mass["w"]
        diag_w = iw + mw * abs(cw) ** 2
        for child, idx in (("j", mdl.J), ("r", mdl.R)):
            m, inertia, cm = ind.mass[child]
            u = cmath.exp(1j * q[idx]) * cm  # the child's center of mass from its pin, wippen frame
            r = ind.pins[child]  # the child's pin, wippen frame
            coupling = inertia + m * (abs(u) ** 2 + ind.dot(r, u))
            own = inertia + m * abs(u) ** 2
            diag_w += inertia + m * abs(r + u) ** 2
            worst_cf = max(worst_cf, abs(m_mat[mdl.W, idx] - coupling) / coupling,
                           abs(m_mat[idx, idx] - own) / own)
        worst_cf = max(worst_cf, abs(m_mat[mdl.W, mdl.W] - diag_w) / diag_w)
        assert m_mat[mdl.J, mdl.R] == 0.0, "the jack and the lever must have no mass coupling"
    assert worst_m < 1e-8, f"M differs from the independent kinetic energy by {worst_m:.1e}"
    assert worst_cf < 1e-12, f"the wippen block differs from its closed forms by {worst_cf:.1e}"
    worst_grad, compared = 0.0, 0
    h = 1e-6
    for q in [closure_pose()] + random_poses(rng, 60) + stroke:
        kin = mdl.Model.contact_kinematics(q)
        for i in range(mdl.N_Q):
            e = np.zeros(mdl.N_Q)
            e[i] = h
            gp, gm = ind.gaps(q + e), ind.gaps(q - e)
            for c in mdl.CONTACTS:
                spec = mdl.SPECS[c]
                if spec.kind == "hc" and min(gp[c - 1], gm[c - 1]) <= -(spec.arc_radius + spec.radius):
                    continue  # inside W, where the normal switches (model.py A9)
                fd = (gp[c - 1] - gm[c - 1]) / (2 * h)
                an = kin[c][3][i]
                err = abs(fd - an)
                worst_grad = max(worst_grad, err / (1e-3 + abs(an)))
                compared += 1
                assert err < 1e-8 + 1e-6 * abs(an), (
                    f"contact {c}, coordinate {i}: analytic {an:.9e}, independent {fd:.9e}")
    return (f"gaps at {len(poses)} poses within {worst_gap:.0e} m; M within {worst_m:.1e} of the"
            f" independent kinetic energy at 40 poses, the wippen block within {worst_cf:.0e} of"
            f" its closed forms; {compared} gradients within {worst_grad:.1e} of central"
            f" differences of the independent gap")


@check
def normals_points_and_friction_through_a_stroke():
    """At script 10's closure, at the frictionless equilibrium and at the 46
    states of a key stroke (stroke_states), every contact whose circle is
    outside the hybrid's wedge W: (a) the unit normal points from body B to
    body A: moving A's frame along n opens the gap, at the rate of the printed
    normal's norm for the four lines of model.py A10; (b) the contact point
    lies on the common normal, midway between the two surfaces; (c) the
    normal force's generalized force is the virtual work of +f n on A and
    -f n on B at that point. At every one of those states where a frictional
    contact is loaded: the slip is the relative speed of the two bodies'
    material points along t = ẑ × n; μ is Eq. 2.10 with its bracket at that
    slip; the generalized friction force is the virtual work of -μ f_n t on A
    and +μ f_n t on B; and its power is never positive."""
    model = mdl.Model()
    states = ([itg.State(0.0, closure_pose(), np.zeros(mdl.N_Q)), sc.rest_state(model)[0]]
              + stroke_states())
    eps, h = 1e-7, 1e-7
    worst_dir = worst_point = worst_vw = worst_slip = worst_ft = 0.0
    frictional = 0
    for st in states:
        q, v = st.q, st.v
        pose = mdl.Pose(q)
        kin = mdl.Model.contact_kinematics(q)
        ev = model.evaluate(q, v)
        for c in mdl.CONTACTS:
            spec = mdl.SPECS[c]
            g, (nx, ny), (px, py), grad, jt, _branch = kin[c]
            if spec.kind == "hc" and g <= -(spec.arc_radius + spec.radius):
                continue
            # (a) the normal's direction: move A along n (or the ground's partner B along -n)
            moved, sign = (spec.a, 1.0) if spec.a != "g" else (spec.b, -1.0)
            shifted_gaps = []
            for s_ in (eps, -eps):
                shifted = mdl.Pose(q)
                ox, oy, cs, sn = shifted.frame[moved]
                shifted.frame[moved] = (ox + sign * s_ * nx, oy + sign * s_ * ny, cs, sn)
                shifted_gaps.append(mdl.contact_geometry(spec, shifted)[0])
            rate = (shifted_gaps[0] - shifted_gaps[1]) / (2 * eps)
            worst_dir = max(worst_dir, abs(rate - spec.gap_scale))
            assert abs(rate - spec.gap_scale) < 1e-6, (
                f"contact {c}: n does not point from B to A (the gap opens at {rate:+.6f})")
            # (b) the point: the circle's surface point facing the other body, plus half the gap
            true_gap = g / spec.gap_scale
            if spec.kind in ("lc", "hc"):  # circle on B, facing A along +n
                cx, cy = pose.to_world(spec.b, *spec.circle)
                d = spec.radius + 0.5 * true_gap
                ex, ey = cx + nx * d, cy + ny * d
            elif spec.kind == "cl":  # circle on A, facing B along -n
                cx, cy = pose.to_world(spec.a, *spec.circle)
                d = spec.radius + 0.5 * true_gap
                ex, ey = cx - nx * d, cy - ny * d
            else:
                cx, cy = pose.to_world(spec.b, *spec.circle_b)
                d = spec.radius_b + 0.5 * g
                ex, ey = cx + nx * d, cy + ny * d
            off = math.hypot(px - ex, py - ey)
            worst_point = max(worst_point, off)
            # the printed norms' share (model.py A10): the point uses the gap as projected
            assert off <= 1e-12 + 0.5 * abs(g - true_gap), f"contact {c}: point {off:.2e} m off midway"
            # (c) the normal force by virtual work, from the motion of material points
            local_a, local_b = pose.to_local(spec.a, px, py), pose.to_local(spec.b, px, py)
            rel = []
            for i in range(mdl.N_Q):
                e = np.zeros(mdl.N_Q)
                e[i] = h
                pp, pm = mdl.Pose(q + e), mdl.Pose(q - e)
                va = np.array(pp.to_world(spec.a, *local_a)) - np.array(pm.to_world(spec.a, *local_a))
                vb = np.array(pp.to_world(spec.b, *local_b)) - np.array(pm.to_world(spec.b, *local_b))
                rel.append((va - vb) / (2 * h))
            n, t = np.array([nx, ny]), np.array([-ny, nx])
            q_vw = np.array([n @ r for r in rel])
            err = float(np.max(np.abs(q_vw - np.array(grad) / spec.gap_scale)))
            worst_vw = max(worst_vw, err)
            assert err < 1e-8, f"contact {c}: normal force differs from its virtual work by {err:.1e}"
            if spec.friction is None or ev.fn[c - 1] <= 0.0:
                continue
            frictional += 1
            slip = float(sum((t @ rel[i]) * v[i] for i in range(mdl.N_Q)))
            worst_slip = max(worst_slip, abs(slip - ev.slip[c - 1]))
            assert abs(slip - ev.slip[c - 1]) < 1e-9 + 1e-6 * abs(slip), f"contact {c}: slip"
            a_, b1, b2 = spec.friction
            x = ev.slip[c - 1] / mdl.FRICTION_THRESHOLD_SPEED_M_S
            mu = a_ * (math.tanh(x) + b1 * x / (1.0 + b2 * x ** 4))
            assert abs(ev.ft[c - 1] - mu * ev.fn[c - 1]) <= 1e-12 * ev.fn[c - 1], (
                f"contact {c}: friction is not Eq. 2.10")
            q_model = -ev.ft[c - 1] * np.array(jt)
            q_mine = np.array([-ev.ft[c - 1] * (t @ r) for r in rel])
            err = float(np.max(np.abs(q_model - q_mine)))
            worst_ft = max(worst_ft, err)
            assert err < 1e-8 * (1.0 + abs(ev.ft[c - 1])), f"contact {c}: friction force"
            assert q_model @ v <= 0.0, f"contact {c}: friction does positive work"
    assert frictional > 0
    return (f"{len(states)} states: n from B to A within {worst_dir:.0e}; the point midway within"
            f" {worst_point:.1e} m (the printed norms' share); normal force by virtual work within"
            f" {worst_vw:.0e} m/rad; {frictional} loaded frictional contacts: slip within"
            f" {worst_slip:.0e} m/s, Eq. 2.10 exact, friction by virtual work within {worst_ft:.0e},"
            f" never doing positive work")


@check
def gravity_points_down_in_the_record_frames():
    """The record's ground frame has y upwards (Figure A.1), so gravity is -y.
    Its generalized forces are -∇ of g Σ m y_cm computed by RecordKinematics,
    at 60 poses. At the frictionless equilibrium each body falls onto its own
    support: the key, front-heavy (its center of mass in front of the balance
    pin), turns its front down toward the front-rail punching (#2); the
    wippen turns down onto the capstan (#1); the hammer turns its head away
    from the string (#12) and its shank down onto the jack (#7) and its rest
    rail (#13); and the string lies above the hammer head."""
    ind = independent()
    model = mdl.Model(contacts=False, springs=False)
    rng = np.random.default_rng(SEED + 11)
    worst = 0.0
    g = mdl.GRAVITY_M_S2
    for q in [closure_pose()] + random_poses(rng, 59, spread=(0.3,) * 5):
        qg = model.evaluate(q, np.zeros(mdl.N_Q)).q_gravity
        fd = np.array([-(ind.gravity_potential(q + 1e-6 * e, g) - ind.gravity_potential(q - 1e-6 * e, g))
                       / 2e-6 for e in np.eye(mdl.N_Q)])
        err = float(np.max(np.abs(qg - fd)))
        worst = max(worst, err)
        assert err < 1e-9, f"gravity differs from -∇(g Σ m y_cm) by {err:.1e} N·m"
    q_eq = sc.rest_state()[0].q
    qg = model.evaluate(q_eq, np.zeros(mdl.N_Q)).q_gravity
    kin = mdl.Model.contact_kinematics(q_eq)
    f = ind.frames(q_eq)
    key_cm = ind.world(f, "k", ind.mass["k"][2])
    assert key_cm.real > ind.pins["k"].real and qg[mdl.K] < 0.0, "the key should be front-heavy"
    # contact: (coordinate, +1 if gravity opens the gap, -1 if it closes it)
    falls = {2: (mdl.K, -1), 1: (mdl.W, -1), 12: (mdl.H, +1), 7: (mdl.H, -1), 13: (mdl.H, -1)}
    for c, (i, sign) in falls.items():
        assert sign * qg[i] * kin[c][3][i] > 0.0, f"gravity moves contact {c} the wrong way"
    head = ind.world(f, "h", ind.points["Ph4"]["c"])
    assert ind.points["Pg9"]["c"].imag > head.imag + ind.points["Ph4"]["r"], "the string is not above"
    return (f"60 poses within {worst:.0e} N·m of -∇(g Σ m y_cm); at equilibrium gravity turns the"
            f" key front down ({qg[mdl.K]:+.4f} N·m), the wippen onto the capstan"
            f" ({qg[mdl.W]:+.5f}), the hammer head away from the string ({qg[mdl.H]:+.5f})")


@check
def repetition_spring_lifts_the_lever():
    """Eq. A.3 as the record prints it, T = -0.073 (θr - 0.832) N·m, is the
    model's spring torque, and it is positive (counter-clockwise) at script
    10's closure and at the frictionless equilibrium. A positive θr closes the
    lever's gaps to the knuckle (#10) and to the drop screw (#11) and presses
    its stop (#6): the spring lifts the lever's knuckle end toward the
    knuckle, and the stop limits it. At equilibrium, with #9, #10 and #11
    open, the stop carries the spring: f6 = (T + gravity on θr) / |∂g6/∂θr|."""
    rec = mdl.RECORD["springs"]["repetition_lever"]
    coeff, offset = rec["coefficient"]["value"], rec["angle_offset"]["value"]
    model = mdl.Model()
    out = []
    for label, q in (("closure", closure_pose()), ("equilibrium", sc.rest_state(model)[0].q)):
        torque = coeff * (q[mdl.R] - offset)
        ev = model.evaluate(q, np.zeros(mdl.N_Q))
        assert torque > 0.0, f"{label}: Eq. A.3 gives {torque:+.5f} N·m"
        assert ev.q_springs[mdl.R] == torque, f"{label}: the model's spring torque {ev.q_springs[mdl.R]}"
        kin = mdl.Model.contact_kinematics(q)
        for c in (6, 10, 11):
            assert kin[c][3][mdl.R] < 0.0, f"{label}: a positive θr does not close contact {c}"
        out.append(f"{label} T = {torque:+.4f} N·m")
    q_eq = sc.rest_state(model)[0].q
    ev = model.evaluate(q_eq, np.zeros(mdl.N_Q))
    kin = mdl.Model.contact_kinematics(q_eq)
    assert ev.fn[9 - 1] == ev.fn[10 - 1] == ev.fn[11 - 1] == 0.0
    f6 = (ev.q_springs[mdl.R] + ev.q_gravity[mdl.R]) / -kin[6][3][mdl.R]
    assert abs(ev.fn[6 - 1] - f6) < 1e-9, f"the stop carries {ev.fn[5]:.6f} N, the spring {f6:.6f} N"
    return ("; ".join(out) + f"; ∂g/∂θr: stop {kin[6][3][mdl.R] * 1e3:+.2f}, knuckle"
            f" {kin[10][3][mdl.R] * 1e3:+.2f}, drop screw {kin[11][3][mdl.R] * 1e3:+.2f} mm/rad;"
            f" at equilibrium the stop carries the spring's {ev.fn[5]:.3f} N, the lever"
            f" {ev.gap[9] * 1e3:.2f} mm below the knuckle")


@check
def axis_friction_holds_with_every_joint_loaded():
    """The assembled action, every force on, posed (Newton on the contact
    deflections) so that every joint carries 0.9 of its axis-friction bound
    at once, through at least six loaded contacts, in three patterns of signs:
    1000 steps of 10 µs in A and B leave every coordinate bit-identical and
    every speed exactly zero. Then one joint at a time at 1.05 of its bound,
    the others at 0.5: the first step moves that joint in the direction of its
    net torque, in A at exactly dt (F - A)/M_ii, and leaves every other
    joint's speed exactly zero, in A and B."""
    model = mdl.Model()
    q_eq = sc.rest_state(model)[0].q
    bound = mdl.AXIS_FRICTION_NM
    dt = 1e-5
    for signs in ((1, 1, 1, 1, 1), (-1, 1, -1, 1, -1), (1, -1, -1, 1, 1)):
        q = sc.pose_with_net_torques(model, 0.9 * bound * np.array(signs), q_eq)
        ev = model.evaluate(q, np.zeros(mdl.N_Q))
        assert np.allclose(ev.Q / bound, 0.9 * np.array(signs), rtol=0, atol=1e-6)
        assert np.sum(ev.fn > 0.0) >= 6
        for name in ("A", "B"):
            final, steps, finite = itg.simulate(model, itg.State(0.0, q, np.zeros(mdl.N_Q)), dt, 1000,
                                                itg.FreeDrive(), name)
            assert finite and np.array_equal(final.q, q) and not np.any(final.v), (
                f"{name}, signs {signs}: moved by {final.q - q}, speeds {final.v}")
    for i in range(mdl.N_Q):
        for sign in (1.0, -1.0):
            target = 0.5 * bound * np.array([1, -1, 1, -1, 1], dtype=float)
            target[i] = sign * 1.05 * bound[i]
            q = sc.pose_with_net_torques(model, target, q_eq)
            ev = model.evaluate(q, np.zeros(mdl.N_Q))
            others = [j for j in range(mdl.N_Q) if j != i]
            for name, step in (("A", itg.step_explicit), ("B", itg.step_implicit)):
                after, _info = step(model, itg.State(0.0, q, np.zeros(mdl.N_Q)), dt, itg.FreeDrive())
                assert after.v[i] * sign > 0.0, f"{name}: joint {i} did not start with its net torque"
                assert not np.any(after.v[others]), f"{name}: joint {i} unstuck others: {after.v}"
                if name == "A":
                    expected = dt * (ev.Q[i] - sign * bound[i]) / ev.M[i, i]
                    assert abs(after.v[i] - expected) <= 1e-9 * abs(expected), (
                        f"joint {i}: {after.v[i]:.9e} against {expected:.9e}")
    return ("every joint at 0.9 of its bound through loaded contacts: 3 sign patterns held"
            " bit-identical over 1000 steps in A and B; at 1.05 of its bound each joint starts"
            " with its net torque, the others exactly stuck")


@check
def energy_with_undamped_contacts():
    """Contacts on with their damping set to zero, contact and axis friction
    off, gravity and springs on: the action kicked from its equilibrium (key
    front down, hammer up) for 10 ms, through contact closures. The total
    energy (kinetic, gravity, springs, the contacts' elastic energy) departs
    boundedly in (A), the departure first order in the step; (B), whose
    implicit stiffness damps, never gains energy and loses it at first order.
    A sign error in a contact force, in a gradient or in (B)'s K pumps energy
    in."""
    saved = {c: mdl.SPECS[c].damping for c in mdl.CONTACTS}
    out = []
    try:
        for c in mdl.CONTACTS:
            mdl.SPECS[c].damping = 0.0
        model = mdl.Model(contact_friction=False, axis_friction=False)
        q0 = sc.rest_state()[0].q
        v0 = np.array([-3.0, 2.0, -5.0, 4.0, -20.0])
        e0 = model.kinetic_energy(q0, v0) + model.potential_energy(q0)
        for name in ("A", "B"):
            worst, gain, final = [], [], []
            for dt in (4e-6, 2e-6):
                rec = {"worst": 0.0, "gain": -np.inf, "last": 0.0}

                def watch(_before, after, _info, rec=rec):
                    de = model.kinetic_energy(after.q, after.v) + model.potential_energy(after.q) - e0
                    rec["worst"] = max(rec["worst"], abs(de))
                    rec["gain"] = max(rec["gain"], de)
                    rec["last"] = de

                _s, _steps, finite = itg.simulate(model, itg.State(0.0, q0, v0), dt,
                                                  int(round(0.01 / dt)), itg.FreeDrive(), name,
                                                  observe=watch)
                assert finite
                worst.append(rec["worst"])
                gain.append(rec["gain"])
                final.append(rec["last"])
            if name == "A":
                assert worst[0] < 1e-2 * e0 and worst[0] / worst[1] > 1.6, f"A: {worst}"
                out.append(f"A departs by {worst[0] / e0:.1e} and {worst[1] / e0:.1e}"
                           f" of {e0 * 1e3:.1f} mJ")
            else:
                assert max(gain) <= 1e-9 * e0, f"B gains energy: {gain}"
                assert final[0] / final[1] > 1.6, f"B: loss not first order {final}"
                out.append(f"B loses {-final[0] / e0:.1e} and {-final[1] / e0:.1e}, never gains")
    finally:
        for c, d in saved.items():
            mdl.SPECS[c].damping = d
    return "at 4 and 2 µs: " + "; ".join(out)


@check
def slow_position_drive_through_let_off():
    """The key driven down in 0.3 s to 0.3 mm into the front-rail punching
    (scenarios.py KeyStroke), then held, in A and B at 10 µs. The jack's toe
    meets the button; the jack then turns clockwise relative to the wippen and
    leaves the knuckle (contact 7 opens) before the key reaches the punching:
    let-off. Nothing passes through anything: the knuckle never sinks 0.1 mm
    into the jack, nor reaches the wedge W where the hybrid's normal switches;
    the toe never sinks 0.1 mm into the button; the hammer never reaches the
    string. At the end the hammer rests on the repetition lever (#10 loaded,
    #7 open) and the lever on the drop screw (#11 loaded). A and B agree on
    where let-off and the resting hammer are. Those two distances, and the
    key's dip at Pk2 from rest to the front-rail punching, are reported beside
    the record's regulation targets: they are outputs of the printed geometry,
    not checked against the targets."""
    model = mdl.Model()
    start, _it, _res = sc.rest_state(model)
    bottom = sc.key_angle_at_penetration(start.q, 0.3e-3)
    spec7 = mdl.SPECS[7]
    results = {}
    for name in ("A", "B"):
        stroke = sc.KeyStroke(float(start.q[mdl.K]), bottom, start_s=0.005, down_s=0.3, hold_s=1.0,
                              up_s=0.05)
        rec = {"met": None, "let_off": None, "least": np.full(13, np.inf), "g7": None}

        def watch(before, after, info, rec=rec):
            ev = info.evaluation
            np.minimum(rec["least"], ev.gap, out=rec["least"])
            if rec["met"] is None and ev.fn[8 - 1] > 0.0:
                rec["met"] = before.q.copy()
            gaps = mdl.Model.gaps(after.q)
            if (rec["met"] is not None and rec["let_off"] is None and rec["g7"] is not None
                    and rec["g7"] <= 0.0 < gaps[7 - 1]):
                rec["let_off"] = (after.t, after.q.copy(), gaps[12 - 1])
            rec["g7"] = gaps[7 - 1]

        final, _steps, finite = itg.simulate(model, start.copy(), 1e-5, 36500, itg.PositionDrive(stroke),
                                             name, observe=watch)
        assert finite
        assert rec["met"] is not None, f"{name}: the toe never met the button"
        assert rec["let_off"] is not None, f"{name}: no let-off"
        t_off, q_off, hammer_off = rec["let_off"]
        least = rec["least"]
        assert t_off < stroke.start_s + stroke.down_s, f"{name}: let-off after the key bottomed"
        assert q_off[mdl.J] < rec["met"][mdl.J], f"{name}: the jack did not turn clockwise to escape"
        assert least[7 - 1] > -1e-4, f"{name}: the knuckle {-least[6] * 1e3:.3f} mm into the jack"
        assert least[7 - 1] > -(spec7.arc_radius + spec7.radius), f"{name}: the knuckle reached W"
        assert least[8 - 1] > -1e-4, f"{name}: the toe {-least[7] * 1e3:.3f} mm into the button"
        assert least[12 - 1] > 0.0, f"{name}: the hammer reached the string"
        ev = model.evaluate(final.q, final.v)
        assert ev.gap[7 - 1] > 0.0 and ev.fn[10 - 1] > 0.0 and ev.fn[11 - 1] > 0.0, (
            f"{name}: not resting on the lever at the drop screw; gaps (mm) {ev.gap * 1e3}")
        results[name] = (hammer_off, ev.gap[12 - 1], least[7 - 1], least[8 - 1])
    (off_a, rest_a, *_), (off_b, rest_b, *_) = results["A"], results["B"]
    assert abs(off_a - off_b) < 5e-4 and abs(rest_a - rest_b) < 5e-5, f"A and B disagree: {results}"
    targets = {s["step"]: s["target"]["value"] for s in mdl.RECORD["regulation"]["steps"]
               if "target" in s}
    touching = start.q.copy()
    touching[mdl.K] = sc.key_angle_at_penetration(start.q, 0.0)
    dip = (mdl.Pose(start.q).to_world("k", *mdl.FINGER_POINT_LOCAL)[1]
           - mdl.Pose(touching).to_world("k", *mdl.FINGER_POINT_LOCAL)[1])
    knuckle = -min(results["A"][2], results["B"][2])
    toe = -min(results["A"][3], results["B"][3])
    return (f"let-off with the hammer {off_a * 1e3:.2f} mm (A) and {off_b * 1e3:.2f} mm (B) from the"
            f" string; at the bottom it rests on the lever, held by the drop screw, {rest_a * 1e3:.2f}"
            f" mm from it; the knuckle at most {knuckle * 1e3:.3f} mm into the jack, the toe"
            f" {toe * 1e3:.3f} mm into the button; key dip at Pk2 {dip * 1e3:.2f} mm. The record's"
            f" targets: key dip {targets[1]} mm, let-off {targets[5]} mm, drop {targets[6]} mm")


def main():
    failures = 0
    t_all = time.perf_counter()
    for fn in CHECKS:
        t0 = time.perf_counter()
        try:
            detail = fn()
            status = "PASS"
        except AssertionError as exc:
            detail = f"{exc}"
            status = "FAIL"
            failures += 1
        print(f"{status}  {fn.__name__}  [{time.perf_counter() - t0:.1f} s]\n      {detail}",
              flush=True)
    print(f"\n{len(CHECKS) - failures} of {len(CHECKS)} checks passed"
          f" in {time.perf_counter() - t_all:.0f} s")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
