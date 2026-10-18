"""The two fixed-step integrators of the one-note model (model.py).

THROWAWAY PROTOTYPE (README.md).

Both take one fixed step dt: no adaptive stepping, no sub-steps. Both treat
axis friction as a set (model.py, A6): at velocity level, one impulse per joint,
bounded by ±A·dt, such that each joint either sticks (its speed exactly zero,
the impulse inside the box) or slides (the impulse on the bound, opposing the
joint's new speed). That is a box-constrained problem on the joint speeds,

    v⁺ = v_free + W λ,   |λ_i| <= A_i dt,   complementarity as above,

W the inverse of the step's system matrix restricted to the coordinates being
solved. It is solved by projected Gauss-Seidel (PGS) in a fixed joint order,
warm-started from the previous step, to a tolerance under an iteration cap;
the iteration count, the final residual and whether the cap was hit are
reported every step. A finishing pass then solves the linear system of the
active set PGS found (sticking joints at zero speed, sliding ones at their
bound) and, if that solution is consistent (every sticking impulse inside its
box, every sliding joint moving against its impulse), takes it and sets the
sticking joints' speeds to exactly zero, so a joint held by friction does not
creep by rounding. If it is not consistent, the PGS answer stands and the step
says so.

(A) step_explicit: explicit penalty, semi-implicit (symplectic) Euler.
        v⁺ = v + dt M(q)⁻¹ (Q(q, v, t) - h(q, v)) + M(q)⁻¹ λ
        q⁺ = q + dt v⁺
    Every force, contact normal and contact friction included, comes from the
    current state; W = M⁻¹.

(B) step_implicit: linearly implicit at velocity level. One Newton step of
    backward Euler on the stiff part:
        (M + dt C + dt² K) Δv = dt (Q - h - dt K v) + λ,   v⁺ = v + Δv,
        q⁺ = q + dt v⁺,
    K = -∂Q/∂q and C = -∂Q/∂q̇ of the contact laws about the current state:
    each penetrating contact's stiffness k = ∂f_n/∂δ and damping c = ∂f_n/∂δ̇
    along its normal (k ∇g∇gᵀ, c ∇g∇gᵀ, both >= 0 since every cubic rises and
    the clamp zeroes both), and the repetition spring's 0.073 N·m/rad. So the
    matrix is symmetric positive definite and W = (M + dt C + dt² K)⁻¹.
    Left explicit: the geometric stiffness f_n ∂²g/∂q², gravity's stiffness,
    the velocity terms h, and the whole of the contact friction (Eq. 2.10).
    A contact open at the start of a step enters at the next one.

    Why the contact friction stays explicit in (B), measured, not assumed:
    Eq. 2.10 rises from zero slip with the slope A (1 + B1) / v_t, about 5e4
    per m/s at the jack/knuckle. Linearized into C, the friction becomes
    linear in the new slip and unbounded, so a single step can hold a contact
    that Eq. 2.10 would let slide, above the law's own peak. On the forte-like
    force drive the head's speed at the strike then reads 2.2642, 2.2580,
    2.1791, 2.2518 and 2.2571 m/s at dt = 10, 5, 2, 1 and 0.5 µs: not
    converging. Left explicit, the law saturates at its peak and (B) reads
    2.2635, 2.2639, 2.2643, 2.2643 and 2.2643 m/s over the same steps, (A)
    2.2638 to 2.2643. `python3 -I scenarios.py slope-study` reruns the
    comparison; the linearized variant is Model(implicit_friction_slope=True).

Drives (model.py, A8):
    FreeDrive        nothing applied.
    TorqueDrive      constant generalized torques, N·m (a test harness).
    ForceDrive       a force profile F(t), N, at Pk2, vertical, downward > 0.
    PositionDrive    the key's angle θk(t) prescribed. The key is then not
                     integrated: θk follows the profile at every step time and
                     its speed is the profile's backward difference, so
                     q_k⁺ = q_k + dt v_k⁺ holds as for every other coordinate.
                     The output is the finger's force at Pk2, from the key's own
                     row of the step: the force that gives the key the motion
                     prescribed, given every other force on it. While the key's
                     new speed is zero its axis friction is a set, and the
                     finger's force is then an interval of half-width A_k/arm;
                     while it moves, the interval is a point.
"""

from __future__ import annotations

import math

import numpy as np

from model import K, N_Q

FREE_OF_KEY = np.array([1, 2, 3, 4])  # wippen, jack, lever, hammer


# =============================================================================
# Drives
# =============================================================================


class FreeDrive:
    kind = "free"

    def generalized_force(self, model, t, q):
        return np.zeros(N_Q)


class TorqueDrive:
    """Constant generalized torques on the coordinates, N·m."""

    kind = "torque"

    def __init__(self, torques_Nm):
        self.torques_Nm = np.array(torques_Nm, dtype=float)

    def generalized_force(self, model, t, q):
        return self.torques_Nm.copy()


class ForceDrive:
    """A finger force F(t) at Pk2, newtons, downward positive (model.py A8)."""

    kind = "force"

    def __init__(self, force_N):
        self.force_N = force_N

    def generalized_force(self, model, t, q):
        out = np.zeros(N_Q)
        out[K] = -self.force_N(t) * model.finger_arm(q)
        return out


class PositionDrive:
    """The key's angle prescribed, θk(t) in radians."""

    kind = "position"

    def __init__(self, key_angle_rad):
        self.key_angle_rad = key_angle_rad

    def generalized_force(self, model, t, q):
        return np.zeros(N_Q)


# =============================================================================
# State and step report
# =============================================================================


class State:
    """t (s), q (rad), v (rad/s), and the last step's axis-friction impulses
    (N·m·s), kept to warm-start the next friction solve."""

    __slots__ = ("t", "q", "v", "impulse")

    def __init__(self, t, q, v, impulse=None):
        self.t = float(t)
        self.q = np.array(q, dtype=float)
        self.v = np.array(v, dtype=float)
        self.impulse = np.zeros(N_Q) if impulse is None else np.array(impulse, dtype=float)

    def copy(self):
        return State(self.t, self.q, self.v, self.impulse)


class StepInfo:
    """What one step reports.

    iterations     PGS sweeps of the axis-friction solve
    converged      PGS reached its tolerance before its cap
    residual       natural residual of the box problem after the finishing pass,
                   relative to each joint's bound (0 when every joint is exactly
                   stuck or exactly sliding)
    polished       the finishing pass's exact active-set solution was taken
    sticking       which coordinates' joints were held by axis friction
    axis_friction_Nm   the friction torque on each joint over the step
    finger_force_N     position drive only: (low, high) finger force at Pk2
    evaluation     the model's evaluation at the start of the step
    """

    __slots__ = ("iterations", "converged", "residual", "polished", "sticking",
                 "axis_friction_Nm", "finger_force_N", "evaluation")


# =============================================================================
# Axis friction as a set: the box-constrained problem
# =============================================================================


def solve_axis_friction(w_mat, v_free, bound, warm, tol=1e-13, cap=200):
    """Box-constrained problem of the module docstring, by PGS then the
    finishing pass. w_mat (n×n, symmetric positive definite), v_free (n), bound
    (n, >= 0, an impulse), warm (n). Returns (impulse, v, report) with report =
    (iterations, converged, residual, polished, sticking)."""
    n = len(v_free)
    w = [[float(x) for x in row] for row in np.asarray(w_mat)]
    vf = [float(x) for x in v_free]
    b = [float(x) for x in bound]
    lam = [min(max(float(x), -bi), bi) for x, bi in zip(warm, b)]
    rows = [i for i in range(n) if b[i] > 0.0]
    iterations, converged = 0, True
    if rows:
        converged = False
        for sweep in range(1, cap + 1):
            change = 0.0
            for i in rows:
                wi = w[i]
                r = vf[i]
                for j in range(n):
                    r += wi[j] * lam[j]
                new = lam[i] - r / wi[i]
                if new > b[i]:
                    new = b[i]
                elif new < -b[i]:
                    new = -b[i]
                d = abs(new - lam[i]) / b[i]
                if d > change:
                    change = d
                lam[i] = new
            iterations = sweep
            if change <= tol:
                converged = True
                break

    def speeds(impulse):
        return [vf[i] + sum(w[i][j] * impulse[j] for j in range(n)) for i in range(n)]

    v = speeds(lam)
    stick = [i for i in rows if abs(lam[i]) < b[i] * (1.0 - 1e-9)]
    polished = False
    if stick:
        slide = [i for i in range(n) if i not in stick]
        w_ss = np.array([[w[i][j] for j in stick] for i in stick])
        rhs = np.array([-(vf[i] + sum(w[i][j] * lam[j] for j in slide)) for i in stick])
        lam_s = np.linalg.solve(w_ss, rhs)
        trial = list(lam)
        for k, i in enumerate(stick):
            trial[i] = float(lam_s[k])
        v_trial = speeds(trial)
        ok = all(abs(trial[i]) <= b[i] * (1.0 + 1e-12) for i in stick)
        ok = ok and all(v_trial[i] * trial[i] <= 0.0 for i in slide if b[i] > 0.0)
        if ok:
            lam, v, polished = trial, v_trial, True
            for i in stick:
                v[i] = 0.0  # stuck: zero by the complementarity, not by rounding
    residual = 0.0
    for i in rows:
        target = min(max(lam[i] - v[i] / w[i][i], -b[i]), b[i])
        residual = max(residual, abs(lam[i] - target) / b[i])
    sticking = tuple(i for i in stick) if polished else tuple(
        i for i in rows if abs(lam[i]) < b[i] * (1.0 - 1e-9))
    return np.array(lam), np.array(v), (iterations, converged, residual, polished, sticking)


def _key_friction_interval(model, v_key):
    """The key's axis-friction torque when its speed is known (position drive):
    a point while it moves, the whole set while it is still."""
    a = float(model.axis_friction_bound_Nm[K])
    if v_key > 0.0:
        return -a, -a
    if v_key < 0.0:
        return a, a
    return -a, a


def _finger_force(model, q, key_need_Nm, v_key):
    """Finger force interval at Pk2 from the torque the key's row needs beyond
    every modeled force, key_need = τ_finger + τ_axis_friction."""
    lo, hi = _key_friction_interval(model, v_key)
    arm = model.finger_arm(q)
    return (lo - key_need_Nm) / arm, (hi - key_need_Nm) / arm


# =============================================================================
# The two steps
# =============================================================================


def _step(model, state, dt, drive, system, tol, cap):
    """Shared body of both steps. `system(ev, force, v, dt)` returns (A, b):
    the step is A Δv = b + λ (A = M, b = dt (Q - h) for the explicit step)."""
    t, q, v = state.t, state.q, state.v
    implicit = system is _implicit_system
    ev = model.evaluate(q, v, jacobians=implicit)
    force = ev.Q - ev.h + drive.generalized_force(model, t, q)
    a_mat, b_vec = system(ev, force, v, dt)
    bound = model.axis_friction_bound_Nm * dt
    info = StepInfo()
    info.evaluation = ev
    info.finger_force_N = None
    if drive.kind == "position":
        free = FREE_OF_KEY
        q_key = float(drive.key_angle_rad(t + dt))
        v_key = (q_key - q[K]) / dt
        dv_key = v_key - v[K]
        a_ff = a_mat[np.ix_(free, free)]
        w_mat = np.linalg.inv(a_ff)
        v_free = v[free] + w_mat @ (b_vec[free] - a_mat[free, K] * dv_key)
        lam_f, v_f, report = solve_axis_friction(w_mat, v_free, bound[free], state.impulse[free],
                                                 tol, cap)
        v_next = np.empty(N_Q)
        v_next[free] = v_f
        v_next[K] = v_key
        impulse = np.zeros(N_Q)
        impulse[free] = lam_f
        key_need = (a_mat[K] @ (v_next - v) - b_vec[K]) / dt
        info.finger_force_N = _finger_force(model, q, key_need, v_key)
        q_next = q + dt * v_next
        q_next[K] = q_key
        sticking = tuple(int(free[i]) for i in report[4])
    else:
        w_mat = np.linalg.inv(a_mat)
        v_free = v + w_mat @ b_vec
        impulse, v_next, report = solve_axis_friction(w_mat, v_free, bound, state.impulse, tol, cap)
        q_next = q + dt * v_next
        sticking = report[4]
    info.iterations, info.converged, info.residual, info.polished, _ = report
    info.sticking = sticking
    info.axis_friction_Nm = impulse / dt
    return State(t + dt, q_next, v_next, impulse), info


def _explicit_system(ev, force, v, dt):
    return ev.M, dt * force


def _implicit_system(ev, force, v, dt):
    return ev.M + dt * ev.C + (dt * dt) * ev.K, dt * (force - dt * (ev.K @ v))


def step_explicit(model, state, dt, drive, tol=1e-13, cap=200):
    """Integrator (A): explicit penalty, semi-implicit Euler (module docstring)."""
    return _step(model, state, dt, drive, _explicit_system, tol, cap)


def step_implicit(model, state, dt, drive, tol=1e-13, cap=200):
    """Integrator (B): linearly implicit at velocity level (module docstring)."""
    return _step(model, state, dt, drive, _implicit_system, tol, cap)


INTEGRATORS = {"A": step_explicit, "B": step_implicit}


def is_finite(state):
    return bool(np.all(np.isfinite(state.q)) and np.all(np.isfinite(state.v)))


def simulate(model, state, dt, n_steps, drive, integrator="B", observe=None, tol=1e-13, cap=200):
    """n_steps fixed steps. observe(state_before, state_after, info) is called
    after every step. Stops early, keeping the last finite state, if a state
    stops being finite. Returns (state, steps_taken, finite)."""
    step = INTEGRATORS[integrator]
    for k in range(n_steps):
        new, info = step(model, state, dt, drive, tol, cap)
        if not is_finite(new) or not all(math.isfinite(x) for x in info.evaluation.Q):
            return state, k, False
        if observe is not None:
            observe(state, new, info)
        state = new
    return state, n_steps, True
