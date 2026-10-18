"""The studies that settle `docs/FOUNDATIONS.md` > The time step and > One dynamic
solver, run on the one-note model, and the report they produce: RESULTS.md.

THROWAWAY PROTOTYPE (README.md). Every number in RESULTS.md is computed by this file;
nothing in it is typed by hand.

Run:  python3 -I tools/dynamics-prototype/study.py [--jobs N] [--save FILE | --load FILE]

The runs are independent and spread over N worker processes (default: every core);
about 9 minutes on 20 cores, several hours on one. RESULTS.md is rewritten at the end.
--save also pickles the raw results; --load rewrites RESULTS.md from them without running.

What it does, in order:
  1. Step convergence. Four scenarios (a slow weighing-like press, a forte and a piano
     press, all driven in position, and a forte driven in force), each run by four
     integrators at fixed steps of 1 to 500 µs, and some below 1 µs (SUB_RUNS). The
     observables are read as the trade reads them and judged at the trade's resolution
     (OBS, and RESULTS.md). A and A-set also run at steps between the sweep's
     (STABILITY_US), to find where they first diverge.
  2. The descent study: why the forte press descends in 30 ms and the piano in 160 ms.
  3. The stiffest linearized modes along each scenario, and the explicit step's linear
     stability limit there.
  4. Operation counts per step, from what each step actually did.
  5. Consistency checks of the variant below.

The four integrators. model.py and integrators.py are used unchanged:
  A, B         integrators.py's explicit and linearly implicit steps, as verified:
               contact friction is Eq. 2.10 (v_t = 1e-5 m/s) integrated explicitly;
               axis friction is a set.
  A-set, B-set the same two steps with the contact friction of contacts 1, 4, 7 and 10
               made a set like the axis friction (step_set below): sliding,
               |f_t| = A f_n opposing the slip; stuck, any |f_t| <= A p f_n, p Eq. 2.10's
               peak over A (1.000, 1.686, 1.725, 1.128). That is Eq. 2.10's limit as v_t
               goes to zero, which has no threshold speed left. It is a variant tested here
               to inform a decision the owner owes (whether contact friction is a set or a
               regularized law); it is not a change of the model.
"""

from __future__ import annotations

import argparse
import math
import os
import sys
import textwrap
import time
from pathlib import Path

for _var in ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS"):
    os.environ.setdefault(_var, "1")  # one thread per worker process

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))  # python3 -I drops it
sys.dont_write_bytecode = True  # no __pycache__ beside the prototype

import multiprocessing as mp  # noqa: E402

import numpy as np  # noqa: E402

import integrators as itg  # noqa: E402
import model as mdl  # noqa: E402
import scenarios as sc  # noqa: E402

K, W, J, R, H = mdl.K, mdl.W, mdl.J, mdl.R, mdl.H
FRICTIONAL = tuple(sorted(mdl.FRICTION_LAW))  # contacts 1, 4, 7, 10
GRAM_N = 9.80665e-3  # one gram-force

# =============================================================================
# Steps, resolutions, scenarios
# =============================================================================

STEPS_US = (1, 2, 5, 10, 20, 50, 100, 200, 500)
SUB_US = (0.5, 0.25)  # below 1 µs: A, B and A-set on three scenarios, B-set on two (all_jobs)
SUB_RUNS = tuple((name, v) for name in ("forte", "piano", "forte_force") for v in ("A", "B", "A-set")) + \
    (("forte", "B-set"), ("forte_force", "B-set"))
# between the sweep's steps, for A and A-set only: where each first diverges (the sweep alone
# brackets that step by a factor of 2 to 2.5)
STABILITY_US = (120, 140, 160, 180, 250, 300, 400)
STABILITY_VARIANTS = ("A", "A-set")
# the piano press as built (A) with Eq. 2.10's threshold speed raised, where its explicit
# integration can be resolved: is the as-built model's difference from the set the law's,
# or its chatter's? (v_t m/s, step µs)
THRESHOLD_RUNS = ((1e-4, 1), (1e-4, 0.25), (3e-4, 1), (3e-4, 0.25))
VARIANTS = ("A", "B", "A-set", "B-set")
MODEL_OF = {"A": "as built", "B": "as built", "A-set": "set", "B-set": "set"}
WINDOW_S = 2e-4  # Hirschkorn's load cell rate, 5 kHz
TRAVEL_BIN_MM = 0.08  # the thinnest punching paper
BOTTOM_PENETRATION_M = 0.3e-3  # scenarios.py's assumption: the key bottoms 0.3 mm into the punching
HOLD_READ_S = 0.02  # checking and drop are read over the last 20 ms of the hold

RES_FORCE_SLOW_N = GRAM_N
RES_FORCE_BLOW_N = 0.1
RES_MM = 0.08
RES_SPEED = 0.01  # relative


class Scenario:
    def __init__(self, name, title, drive, **kw):
        self.name, self.title, self.drive = name, title, drive
        self.__dict__.update(kw)


SCENARIOS = {
    "slow": Scenario("slow", "slow press", "position", start_s=0.005, down_s=1.0, ramp_s=0.1,
                     hold_s=0.2, up_s=1.0, settle_s=0.2),
    "forte": Scenario("forte", "forte press", "position", start_s=0.005, down_s=0.030, hold_s=0.1,
                      settle_s=0.35),
    "piano": Scenario("piano", "piano press", "position", start_s=0.005, down_s=0.160, hold_s=0.1,
                      settle_s=0.35),
    "forte_force": Scenario("forte_force", "forte by force", "force", t_end=0.8, hold_end_s=0.30),
}
DESCENTS_MS = (25, 30, 35, 40, 50, 60, 80, 100, 130, 160, 200)


class Cruise:
    """Key angle for the slow press: rest until start_s, then down at constant speed with
    smooth ramps, held, up the same way. Over a move of duration T with ramps t_r the
    travel fraction is x = V t_r S(t/t_r) on the first ramp, V (t - t_r/2) in the cruise,
    1 - V t_r S((T - t)/t_r) on the last ramp, V = 1/(T - t_r), S(u) = u^4 (5/2 - 3u + u^2)
    the integral of the quintic smoothstep: the speed is C2 and constant between ramps."""

    def __init__(self, a_rest, a_bottom, start_s, move_s, ramp_s, hold_s):
        self.a_rest, self.a_bottom = a_rest, a_bottom
        self.start_s, self.move_s, self.ramp_s, self.hold_s = start_s, move_s, ramp_s, hold_s

    def fraction(self, t):
        big_t, tr = self.move_s, self.ramp_s
        v = 1.0 / (big_t - tr)

        def s(u):
            return u ** 4 * (2.5 - 3.0 * u + u * u)
        if t <= 0.0:
            return 0.0
        if t < tr:
            return v * tr * s(t / tr)
        if t <= big_t - tr:
            return v * (t - 0.5 * tr)
        if t < big_t:
            return 1.0 - v * tr * s((big_t - t) / tr)
        return 1.0

    def __call__(self, t):
        t_up = self.start_s + self.move_s + self.hold_s
        if t <= t_up:
            x = self.fraction(t - self.start_s)
        else:
            x = 1.0 - self.fraction(t - t_up)
        return self.a_rest + (self.a_bottom - self.a_rest) * x


def build(scn, q_rest, down_s=None, hold_s=None, release=True):
    """Phases [(drive, end time)], the key profile (position drives), and the times the
    observables are read at."""
    if scn.drive == "force":
        t_end = scn.t_end
        return dict(phases=[(itg.ForceDrive(sc.FORTE_LIKE_N), t_end)], profile=None, t_end=t_end,
                    hold_window=(scn.hold_end_s - HOLD_READ_S, scn.hold_end_s), down=None, up=None)
    bottom = sc.key_angle_at_penetration(q_rest, BOTTOM_PENETRATION_M)
    down_s = scn.down_s if down_s is None else down_s
    hold_s = scn.hold_s if hold_s is None else hold_s
    t_bottom = scn.start_s + down_s
    t_hold_end = t_bottom + hold_s
    if scn.name == "slow":
        profile = Cruise(float(q_rest[K]), bottom, scn.start_s, down_s, scn.ramp_s, hold_s)
        t_end = t_hold_end + scn.up_s + scn.settle_s
        phases = [(itg.PositionDrive(profile), t_end)]
        up = (t_hold_end, t_hold_end + scn.up_s)
    else:
        profile = sc.KeyStroke(float(q_rest[K]), bottom, start_s=scn.start_s, down_s=down_s,
                               hold_s=1e9, up_s=1.0)
        if release:
            t_end = t_hold_end + scn.settle_s
            phases = [(itg.PositionDrive(profile), t_hold_end), (itg.FreeDrive(), t_end)]
        else:
            t_end = t_hold_end
            phases = [(itg.PositionDrive(profile), t_hold_end)]
        up = None
    return dict(phases=phases, profile=profile, t_end=t_end, hold_window=(t_hold_end - HOLD_READ_S, t_hold_end),
                down=(scn.start_s, t_bottom), up=up)


# =============================================================================
# The set-valued contact-friction variant
# =============================================================================


def eq210_peak(law):
    """Eq. 2.10's largest value over A: its static coefficient over its kinetic one."""
    a, b1, b2 = law
    x = np.linspace(0.0, 30.0, 300001)
    return float(np.max(np.tanh(x) + b1 * x / (1.0 + b2 * x ** 4)))


PEAK = {c: eq210_peak(mdl.FRICTION_LAW[c]) for c in FRICTIONAL}
CHECKPOINTS = (1, 2, 4, 8, 16, 32, 64, 128)


def exact_pass(w, u_free, bound, lam0, rows):
    """The box problem solved exactly by pivoting on its active set, from the one the
    impulses lam0 show: each row stuck (speed exactly zero, impulse inside its box) or
    sliding (impulse on a bound). The stuck rows' impulses are the exact solution of least
    utilization sum (lam_i/bound_i)^2, unique even where stuck rows are redundant (a joint
    and a contact holding the same body), so the split between them cannot drift from
    step to step. Each pivot moves the row with the largest natural residual
    |lam_i - clip(lam_i - u_i/w_ii)| / bound_i to the other state; when every residual is
    below 1e-9 the impulses meet every condition of the problem and are its solution.
    Returns (impulses or None, the stuck-set size of each solve)."""
    n = len(u_free)
    state = np.zeros(n, dtype=int)
    for i in rows:
        if abs(lam0[i]) >= bound[i] * (1.0 - 1e-9):
            state[i] = 1 if lam0[i] > 0.0 else -1
    sizes = []
    for _pivot in range(3 * len(rows) + 1):
        lam = np.where(state != 0, state * bound, 0.0)
        stick = [i for i in rows if state[i] == 0]
        if stick:
            slide = [i for i in range(n) if i not in stick]
            b_s = bound[stick]
            rhs = -(u_free[stick] + w[np.ix_(stick, slide)] @ lam[slide])
            lam[stick] = b_s * np.linalg.lstsq(w[np.ix_(stick, stick)] * b_s, rhs, rcond=None)[0]
            sizes.append(len(stick))
        u = u_free + w @ lam
        worst, worst_i = 0.0, -1
        for i in rows:
            r = abs(lam[i] - min(max(lam[i] - u[i] / w[i, i], -bound[i]), bound[i])) / bound[i]
            if r > worst:
                worst, worst_i = r, i
        if worst <= 1e-9:
            return lam, sizes
        i = worst_i
        if state[i] != 0:
            state[i] = 0  # sliding the wrong way: stick it
        elif abs(lam[i]) > bound[i]:
            state[i] = 1 if lam[i] > 0.0 else -1  # stuck beyond its box: slide on that bound
        else:
            state[i] = -1 if u[i] > 0.0 else 1  # cannot be held still: slide against its speed
    return None, sizes


def solve_box(w, u_free, bound, warm, y, tol_speed=1e-12, cap=200):
    """u = u_free + w lam, |lam_i| <= bound_i, lam_i opposing u_i where it is on its bound,
    u_i = 0 where it is inside; y maps the impulses onto the generalized speeds
    (v = v_free + y lam). Projected Gauss-Seidel sweeps in row order from the warm start
    guess the active set; after sweeps 1, 2, 4, ..., 128, exact_pass pivots from that
    guess to the exact solution and the solve stops when it succeeds. Between those, PGS
    also stops once a sweep moves no generalized speed by more than tol_speed (rad/s): the
    speeds are unique even where rows are redundant, the impulses are not. Returns
    (impulses, stuck rows, (sweeps, converged, exact, stuck-set sizes solved))."""
    n = len(u_free)
    lam = np.clip(np.asarray(warm, dtype=float), -bound, bound)
    rows = [i for i in range(n) if bound[i] > 0.0]
    iterations, converged, exact, attempts = 0, True, False, []
    if rows:
        converged = False
        for sweep in range(1, cap + 1):
            before = lam.copy()
            for i in rows:
                r = u_free[i] + float(w[i] @ lam)
                new = lam[i] - r / w[i, i]
                b = bound[i]
                if new > b:
                    new = b
                elif new < -b:
                    new = -b
                lam[i] = new
            iterations = sweep
            if sweep in CHECKPOINTS or sweep == cap:
                found, sizes = exact_pass(w, u_free, bound, lam, rows)
                attempts += sizes
                if found is not None:
                    lam, converged, exact = found, True, True
                    break
            if float(np.max(np.abs(y @ (lam - before)))) <= tol_speed:
                converged = True
                break
    stick = [i for i in rows if abs(lam[i]) < bound[i] * (1.0 - 1e-9)]
    return lam, stick, (iterations, converged, exact, attempts)


class SetState(itg.State):
    """integrators.State plus the variant's hidden state, per frictional contact: its last
    impulse (the warm start) and whether it ended the last step stuck (which bound applies)."""

    __slots__ = ("contact_impulse", "contact_stuck")

    def __init__(self, t, q, v, impulse=None, contact_impulse=None, contact_stuck=None):
        super().__init__(t, q, v, impulse)
        self.contact_impulse = dict(contact_impulse or {c: 0.0 for c in FRICTIONAL})
        self.contact_stuck = dict(contact_stuck or {c: False for c in FRICTIONAL})

    def copy(self):
        return SetState(self.t, self.q, self.v, self.impulse, self.contact_impulse, self.contact_stuck)


class Info:
    """What a step reports: integrators.StepInfo's fields, plus the counts the cost model
    needs and, for the set variant, each contact row's impulse and final slip."""

    def __init__(self, **kw):
        self.__dict__.update(kw)


def step_set(model, state, dt, drive, implicit, cap=200, contact_friction=True):
    """Integrator A (implicit=False) or B (implicit=True) of integrators.py, with the contact
    friction of contacts 1, 4, 7 and 10 a set, solved in one box problem with the axis
    friction: rows = the free joints (bound A_i dt), then each loaded frictional contact
    (Jacobian its slip row, bound mu f_n dt with f_n at the start of the step, mu = A p if
    it ended the last step stuck, A if sliding). `model` has contact_friction=False, so its
    Q holds no contact friction. The system matrix, the drives and the finger force are
    integrators.py's; the key's contact-friction reaction enters the finger force."""
    t, q, v = state.t, state.q, state.v
    ev = model.evaluate(q, v, jacobians=implicit)
    force = ev.Q - ev.h + drive.generalized_force(model, t, q)
    if implicit:
        a_mat = ev.M + dt * ev.C + (dt * dt) * ev.K
        b_vec = dt * (force - dt * (ev.K @ v))
    else:
        a_mat = ev.M
        b_vec = dt * force
    pose = ev.pose
    crow = []
    if contact_friction:
        for c in FRICTIONAL:
            fn = ev.fn[c - 1]
            if fn > 0.0:
                spec = mdl.SPECS[c]
                _g, nx, ny, px, py, _b = mdl.contact_geometry(spec, pose)
                _grad, jt = mdl.lever_arms(spec, pose, nx, ny, px, py)
                mu = mdl.FRICTION_LAW[c][0] * (PEAK[c] if state.contact_stuck[c] else 1.0)
                crow.append((c, np.array(jt), mu * fn * dt))
    position = drive.kind == "position"
    if position:
        free = [W, J, R, H]
        q_key = float(drive.key_angle_rad(t + dt))
        v_key = (q_key - q[K]) / dt
        dv_key = v_key - v[K]
    else:
        free = [K, W, J, R, H]
        v_key = dv_key = 0.0
    nf = len(free)
    a_ff = a_mat[np.ix_(free, free)]
    a_inv = np.linalg.inv(a_ff)
    if position:
        v_free = v[free] + a_inv @ (b_vec[free] - a_mat[free, K] * dv_key)
    else:
        v_free = v + a_inv @ b_vec
    n_rows = nf + len(crow)
    jac = np.zeros((n_rows, nf))
    jac[:nf, :nf] = np.eye(nf)
    u_off = np.zeros(n_rows)
    bound = np.empty(n_rows)
    bound[:nf] = model.axis_friction_bound_Nm[free] * dt
    warm = np.empty(n_rows)
    warm[:nf] = state.impulse[free]
    for k, (c, jt, b) in enumerate(crow):
        jac[nf + k] = jt[free]
        if position:
            u_off[nf + k] = jt[K] * v_key
        bound[nf + k] = b
        warm[nf + k] = state.contact_impulse[c]
    y = a_inv @ jac.T
    w = jac @ y
    u_free = jac @ v_free + u_off
    lam, stick, (iterations, converged, exact, attempts) = solve_box(w, u_free, bound, warm, y, cap=cap)
    v_f = v_free + y @ lam
    if exact:
        for i in stick:
            if i < nf:
                v_f[i] = 0.0  # a stuck joint: zero by the solution, not by rounding
    v_next = np.empty(mdl.N_Q)
    v_next[free] = v_f
    impulse = np.zeros(mdl.N_Q)
    impulse[free] = lam[:nf]
    contact_impulse = {c: 0.0 for c in FRICTIONAL}
    contact_stuck = {c: False for c in FRICTIONAL}
    ft = np.zeros(13)
    if position:
        v_next[K] = v_key
    rows_out = []
    for k, (c, jt, b) in enumerate(crow):
        contact_impulse[c] = float(lam[nf + k])
        contact_stuck[c] = (nf + k) in stick
        ft[c - 1] = -lam[nf + k] / dt  # model.py's sign: f_t along +slip, Q -= f_t jt
        rows_out.append((c, float(lam[nf + k]), float(jt @ v_next), b))
    ev.ft = ft
    finger = None
    if position:
        friction_on_key = sum(jt[K] * lam[nf + k] for k, (c, jt, b) in enumerate(crow))
        key_need = (a_mat[K] @ (v_next - v) - b_vec[K] - friction_on_key) / dt
        finger = itg._finger_force(model, q, key_need, v_key)
        q_next = q + dt * v_next
        q_next[K] = q_key
    else:
        q_next = q + dt * v_next
    info = Info(evaluation=ev, finger_force_N=finger, iterations=iterations, converged=converged,
                exact=exact, attempts=attempts, sticking=tuple(free[i] for i in stick if i < nf),
                n_free=nf, n_rows=n_rows, contact_rows=rows_out)
    return SetState(t + dt, q_next, v_next, impulse, contact_impulse, contact_stuck), info


def make_model(variant):
    return mdl.Model(contact_friction=False) if variant.endswith("-set") else mdl.Model()


def stepper(variant):
    implicit = variant.startswith("B")
    if variant.endswith("-set"):
        def step(model, state, dt, drive):
            return step_set(model, state, dt, drive, implicit)
        return step
    base = itg.step_implicit if implicit else itg.step_explicit

    def step(model, state, dt, drive):
        new, i = base(model, state, dt, drive)
        nf = 4 if drive.kind == "position" else 5
        info = Info(evaluation=i.evaluation, finger_force_N=i.finger_force_N, iterations=i.iterations,
                    converged=i.converged, exact=i.polished, attempts=[len(i.sticking)] if i.sticking else [],
                    sticking=i.sticking, n_free=nf, n_rows=nf, contact_rows=None)
        return new, info
    return step


# =============================================================================
# Operation counts
# =============================================================================

COST_TERMS = (
    ("pose: 5 angles, the two pins the wippen carries", 16),
    ("13 signed gaps: 10 line/circle (32 each), 1 circle/circle (30), 2 hybrids (40 each)", 430),
    ("mass matrix: 5 centers of mass, 7 Jacobian columns, 11 entries", 90),
    ("velocity terms, gravity, springs, the finger's arm", 100),
)
COMMON_FLOPS = sum(f for _, f in COST_TERMS)
PER_LOADED_FLOPS = 70  # lever arms (<= 4 pivots), gap rate, Hunt-Crossley law, Q accumulation
PER_FRICTION_EQ210_FLOPS = 25  # explicit Eq. 2.10: slip, the bracketed law, Q accumulation
B_ASSEMBLY_FLOPS = 125  # M + dt C + dt^2 K (15 entries), dt K v, the right-hand side
B_PER_LOADED_FLOPS = 30  # rank-one updates of K and C (<= 3 coordinates per contact)
TRANSCENDENTALS = "5 sin, 5 cos, 3 sqrt, 2 atan2"
TRANSCENDENTAL_CALLS = 15


def step_flops(implicit, set_friction, n_free, n_loaded, n_friction, n_rows, sweeps, attempts):
    """Floating-point operations of one step as an engine would do it (each add, multiply,
    divide or compare counted once), from the counts the step reports. Transcendentals are
    counted apart: TRANSCENDENTALS per step, plus one tanh per loaded frictional contact
    in the as-built model."""
    n = n_free
    f = COMMON_FLOPS + PER_LOADED_FLOPS * n_loaded
    if not set_friction:
        f += PER_FRICTION_EQ210_FLOPS * n_friction
    if implicit:
        f += B_ASSEMBLY_FLOPS + B_PER_LOADED_FLOPS * n_loaded
    f += (2 * n ** 3) // 3 + 2 * n * n  # Cholesky of the n x n system, the free speeds
    f += n_rows * 2 * n * n + 6 * n_rows * n_rows  # Y = A^-1 J^T, W = J Y (J sparse)
    f += sweeps * (n_rows * (2 * n_rows + 5) + 2 * n * n_rows)  # PGS sweeps and their stopping test
    for s in attempts:
        f += (2 * s ** 3) // 3 + 4 * s * s + 2 * n_rows * n_rows  # exact pass: solve and check
    f += 2 * n * n_rows + 4 * mdl.N_Q + 20  # speeds, positions, the finger's force
    return f


# =============================================================================
# The probe: observables and diagnostics, read while a run goes
# =============================================================================


class Probe:
    def __init__(self, dt, t_end, hold_window, variant, modal_every=0):
        self.dt = dt
        self.n = 0
        self.n_win = int(math.ceil(t_end / WINDOW_S - 1e-9)) + 1
        self.win_impulse = np.zeros(self.n_win)
        self.win_still = np.zeros(self.n_win, dtype=bool)
        self.prev = None
        self.button = False
        self.let_off = None
        self.strike = None
        self.catch = None
        self.check_before_let_off_N = 0.0
        self.check_before_strike_N = 0.0
        self.max_pen = np.zeros(13)
        self.max_fn = np.zeros(13)
        # steps before the toe meets the button with a contact penetrating and its force clamped to 0
        self.clamped = np.zeros(13, dtype=int)
        self.stick_onsets = []  # set variant: (t, contact) where a contact row goes from sliding to stuck
        self.stuck_rows = set()
        self.flips = {c: 0 for c in FRICTIONAL}
        self.prev_ft = {c: 0.0 for c in FRICTIONAL}
        self.hold_window = hold_window
        self.hold_g12 = []
        self.hold_finger = []
        self.hold_fn = np.zeros(13)
        self.hold_v = 0.0
        self.y_rest = None
        self.set_friction = variant.endswith("-set")
        self.implicit = variant.startswith("B")
        self.hist_loaded = {}
        self.hist_rows = {}
        self.hist_sweeps = {}
        self.flops_sum = 0
        self.flops_max = 0
        self.unconverged = 0
        self.exact_steps = 0
        self.positive_power = -1.0
        self.modal_every = modal_every
        self.modal = []
        self.modal_model = mdl.Model() if modal_every else None

    @staticmethod
    def rate12(q, v):
        pose = mdl.Pose(q)
        spec = mdl.SPECS[12]
        _g, nx, ny, px, py, _ = mdl.contact_geometry(spec, pose)
        grad, _ = mdl.lever_arms(spec, pose, nx, ny, px, py)
        return float(np.dot(grad, v))

    def travel(self, q):
        return self.y_rest - mdl.Pose(q).to_world("k", *mdl.FINGER_POINT_LOCAL)[1]

    def __call__(self, before, after, info):
        ev = info.evaluation
        dt = self.dt
        t = self.n * dt
        self.n += 1
        if self.y_rest is None:
            self.y_rest = mdl.Pose(before.q).to_world("k", *mdl.FINGER_POINT_LOCAL)[1]
        gap, fn = ev.gap, ev.fn
        np.maximum(self.max_pen, -gap, out=self.max_pen)
        np.maximum(self.max_fn, fn, out=self.max_fn)
        if not self.button:
            self.clamped += (gap < 0.0) & (fn == 0.0)
        # cost counts
        n_loaded = int(np.count_nonzero(fn))
        n_friction = sum(1 for c in FRICTIONAL if fn[c - 1] > 0.0)
        self.hist_loaded[n_loaded] = self.hist_loaded.get(n_loaded, 0) + 1
        self.hist_rows[info.n_rows] = self.hist_rows.get(info.n_rows, 0) + 1
        self.hist_sweeps[info.iterations] = self.hist_sweeps.get(info.iterations, 0) + 1
        flops = step_flops(self.implicit, self.set_friction, info.n_free, n_loaded, n_friction, info.n_rows,
                           info.iterations, info.attempts)
        self.flops_sum += flops
        self.flops_max = max(self.flops_max, flops)
        self.unconverged += 0 if info.converged else 1
        self.exact_steps += 1 if info.exact else 0
        if info.contact_rows is not None:
            stuck_now = {c for c, lam, _slip, b in info.contact_rows if b > 0.0 and abs(lam) < b * (1.0 - 1e-9)}
            if len(self.stick_onsets) < 1000:
                self.stick_onsets += [(t, c) for c in sorted(stuck_now - self.stuck_rows)]
            self.stuck_rows = stuck_now
        if info.contact_rows:
            for _c, lam, slip, b in info.contact_rows:
                # contact friction's power lam * slip / dt, over the largest it could have:
                # -1 when sliding, 0 when stuck, never positive
                if b > 0.0 and abs(slip) > 1e-9:
                    self.positive_power = max(self.positive_power, lam * slip / (b * abs(slip)))
        # the finger's force, into 0.2 ms windows
        if info.finger_force_N is not None:
            lo, hi = info.finger_force_N
            f, still = 0.5 * (lo + hi), lo != hi
        else:
            f, still = 0.0, True
        t0, t1 = t, t + dt
        i0 = int(t0 / WINDOW_S + 1e-9)
        while t0 < t1 - 1e-15 and i0 < self.n_win:
            seg_end = min((i0 + 1) * WINDOW_S, t1)
            self.win_impulse[i0] += f * (seg_end - t0)
            if still:
                self.win_still[i0] = True
            t0 = seg_end
            i0 += 1
        for c in FRICTIONAL:
            ft = ev.ft[c - 1]
            if ft * self.prev_ft[c] < 0.0:
                self.flips[c] += 1
            self.prev_ft[c] = ft
        # events, between the previous step's start and this one's
        g4, g7, g12 = gap[3], gap[6], gap[11]
        if self.prev is not None:
            tp, qp, vp, g4p, g7p, g12p = self.prev
            if self.button and self.let_off is None and g7p <= 0.0 < g7:
                u = -g7p / (g7 - g7p)
                r0, r1 = self.rate12(qp, vp), self.rate12(before.q, before.v)
                x0, x1 = self.travel(qp), self.travel(before.q)
                self.let_off = dict(t=tp + u * (t - tp), hammer=g12p + u * (g12 - g12p),
                                    speed=-(r0 + u * (r1 - r0)), travel=x0 + u * (x1 - x0))
            if self.strike is None and g12p > 0.0 >= g12:
                u = g12p / (g12p - g12)
                r0, r1 = self.rate12(qp, vp), self.rate12(before.q, before.v)
                self.strike = dict(t=tp + u * (t - tp), speed=-(r0 + u * (r1 - r0)))
            if self.let_off is not None and self.catch is None and g4p > 0.0 >= g4:
                u = g4p / (g4p - g4)
                self.catch = dict(t=tp + u * (t - tp), hammer=g12p + u * (g12 - g12p))
        if fn[7] > 0.0:
            self.button = True
        if self.let_off is None:
            self.check_before_let_off_N = max(self.check_before_let_off_N, fn[3])
            if self.strike is None:
                self.check_before_strike_N = max(self.check_before_strike_N, fn[3])
        half = 0.5 * dt  # step times are n dt: compare with half a step's margin
        if self.hold_window and self.hold_window[0] - half <= t < self.hold_window[1] - half:
            self.hold_g12.append(g12)
            if info.finger_force_N is not None:
                self.hold_finger.append(0.5 * sum(info.finger_force_N))
            if t + dt >= self.hold_window[1] - half:
                self.hold_v = float(np.max(np.abs(after.v[[W, J, R, H]])))
                self.hold_fn = fn.copy()
        if self.modal_every and (self.n - 1) % self.modal_every == 0:
            free = [W, J, R, H] if info.finger_force_N is not None else [K, W, J, R, H]
            self.modal.append(modal_sample(self.modal_model, before.q, before.v, free, t))
        self.prev = (t, before.q.copy(), before.v.copy(), g4, g7, g12)


def simulate(step, model, state, dt, n_steps, drive, probe):
    for k in range(n_steps):
        new, info = step(model, state, dt, drive)
        if not itg.is_finite(new) or not all(math.isfinite(x) for x in info.evaluation.Q):
            return state, k, False
        probe(state, new, info)
        state = new
    return state, n_steps, True


_REST = {}


def rest():
    """The frictionless static equilibrium every run starts from (scenarios.rest_state)."""
    if "q" not in _REST:
        state, iterations, residual = sc.rest_state(mdl.Model())
        _REST.update(q=state.q.copy(), iterations=iterations, residual=residual)
    return _REST["q"].copy()


def run(name, variant, dt, down_s=None, hold_s=None, release=True, modal_every=0):
    """One scenario, one integrator, one fixed step. Returns a picklable summary."""
    scn = SCENARIOS[name]
    q0 = rest()
    plan = build(scn, q0, down_s, hold_s, release)
    model = make_model(variant)
    state = SetState(0.0, q0, np.zeros(mdl.N_Q)) if variant.endswith("-set") else itg.State(0.0, q0, np.zeros(mdl.N_Q))
    step = stepper(variant)
    probe = Probe(dt, plan["t_end"], plan["hold_window"], variant, modal_every)
    finite, steps = True, 0
    for drive, t_phase_end in plan["phases"]:
        n = int(round(t_phase_end / dt)) - steps
        state, k, finite = simulate(step, model, state, dt, n, drive, probe)
        steps += k
        if not finite:
            break
    ev_end = mdl.Model().evaluate(state.q, state.v)
    return dict(name=name, variant=variant, dt=dt, finite=finite, steps=steps, t_end=plan["t_end"],
                let_off=probe.let_off, strike=probe.strike, catch=probe.catch,
                check_before_let_off_N=probe.check_before_let_off_N,
                check_before_strike_N=probe.check_before_strike_N,
                hold_g12=(float(np.mean(probe.hold_g12)) if probe.hold_g12 else None), hold_v=probe.hold_v,
                hold_fn=probe.hold_fn, hold_finger=float(np.mean(probe.hold_finger)) if probe.hold_finger else None,
                end_g12=float(ev_end.gap[11]), end_g7=float(ev_end.gap[6]), end_v=float(np.max(np.abs(state.v))),
                max_pen=probe.max_pen, max_fn=probe.max_fn, clamped=probe.clamped,
                stick_onsets=probe.stick_onsets, flips=probe.flips, force=probe.win_impulse / WINDOW_S,
                still=probe.win_still, hist_loaded=probe.hist_loaded, hist_rows=probe.hist_rows,
                hist_sweeps=probe.hist_sweeps, flops_mean=probe.flops_sum / max(probe.n, 1),
                flops_max=probe.flops_max, unconverged=probe.unconverged, exact_steps=probe.exact_steps,
                positive_power=probe.positive_power, modal=probe.modal,
                down=plan["down"], up=plan["up"], down_s=down_s or getattr(scn, "down_s", None))


# =============================================================================
# Linearized modes
# =============================================================================


def modal_sample(model, q, v, free, t):
    """At one state: the largest generalized eigenvalue of (K, M) over the free coordinates,
    K = -dQ/dq of the contact normal laws and the repetition spring at the forces the run
    produces (model.Evaluation.K), as a frequency; which loaded contacts carry that mode
    (their share of its stiffness); and the explicit step's linear stability limit with
    K and the contact damping C both explicit, as integrator A has them."""
    ev = model.evaluate(q, v, jacobians=True)
    m = ev.M[np.ix_(free, free)]
    k = ev.K[np.ix_(free, free)]
    c = ev.C[np.ix_(free, free)]
    lc = np.linalg.cholesky(m)
    li = np.linalg.inv(lc)
    s = li @ k @ li.T
    wv, y = np.linalg.eigh(0.5 * (s + s.T))
    lam = max(float(wv[-1]), 0.0)
    phi = li.T @ y[:, -1]
    total = float(phi @ k @ phi)
    share = {}
    pose = ev.pose
    for cc in mdl.CONTACTS:
        if ev.fn[cc - 1] > 0.0 and total > 0.0:
            spec = mdl.SPECS[cc]
            _g, nx, ny, px, py, _b = mdl.contact_geometry(spec, pose)
            grad, _jt = mdl.lever_arms(spec, pose, nx, ny, px, py)
            g = np.array(grad)
            _f, kk, _c = mdl.normal_force(spec, -ev.gap[cc - 1], -float(g @ v))
            share[cc] = kk * float(g[free] @ phi) ** 2 / total
    return dict(t=t, f=math.sqrt(lam) / (2 * math.pi), share=share, free=len(free),
                limit=explicit_limit(m, k, c), fn={cc: float(ev.fn[cc - 1]) for cc in share})


def explicit_limit(m, k, c, h_hi=2e-3, tol=1e-6):
    """Largest step h at which v+ = v + h M^-1 (-K x - C v), x+ = x + h v+ is stable:
    spectral radius of its amplification matrix at most 1 + tol, by bisection. The
    coordinates no contact holds give a defective eigenvalue 1, which rounding moves by
    about 1e-8; tol keeps that from passing for an instability."""
    n = len(m)
    a = np.linalg.solve(m, k)
    b = np.linalg.solve(m, c)
    eye = np.eye(n)

    def radius(h):
        g = np.block([[eye - h * h * a, h * (eye - h * b)], [-h * a, eye - h * b]])
        return float(np.max(np.abs(np.linalg.eigvals(g))))

    if radius(h_hi) <= 1.0 + tol:
        return h_hi
    lo, hi = 0.0, h_hi
    for _ in range(50):
        mid = 0.5 * (lo + hi)
        if radius(mid) <= 1.0 + tol:
            lo = mid
        else:
            hi = mid
    return lo


def rest_modes():
    """At rest: the top mode with the contacts the equilibrium loads, key free and key
    held; and script 10's starting estimates (`docs/FOUNDATIONS.md` > The time step) recomputed by script 10's
    own function, at its closure, at its assumed fortissimo forces."""
    q0 = rest()
    model = mdl.Model()
    out = {"free": modal_sample(model, q0, np.zeros(mdl.N_Q), [K, W, J, R, H], 0.0),
           "held": modal_sample(model, q0, np.zeros(mdl.N_Q), [W, J, R, H], 0.0)}
    s10 = mdl.S10
    q_closure = list(mdl.script_10_rest_pose()[0])
    m_mat = s10.mass_matrix(q_closure)
    m_inv = s10.inverse(m_mat)
    out["s10"] = [(label, s10.top_mode_hz(m_mat, m_inv, cs, q_closure)) for label, cs in
                  (("capstan, front punching, jack/knuckle", [1, 2, 7]),
                   ("the same with the lever on the knuckle and the drop screw", [1, 2, 7, 10, 11]),
                   ("all thirteen at once", list(range(1, 14))))]
    return out


# =============================================================================
# Observables
# =============================================================================


def key_travel_mm(profile, t):
    q = rest()
    q[K] = profile(t)
    y0 = mdl.Pose(rest()).to_world("k", *mdl.FINGER_POINT_LOCAL)[1]
    return (y0 - mdl.Pose(q).to_world("k", *mdl.FINGER_POINT_LOCAL)[1]) * 1e3


_TRAVEL_CACHE = {}


def window_travel(name, n_win):
    """Key travel at Pk2 (mm) at the middle of each 0.2 ms window, from the profile."""
    key = (name, n_win)
    if key not in _TRAVEL_CACHE:
        plan = build(SCENARIOS[name], rest())
        tm = (np.arange(n_win) + 0.5) * WINDOW_S
        _TRAVEL_CACHE[key] = np.array([key_travel_mm(plan["profile"], t) for t in tm])
    return _TRAVEL_CACHE[key]


def bottom_travel_mm(name):
    plan = build(SCENARIOS[name], rest())
    return key_travel_mm(plan["profile"], plan["down"][1])


def slow_curves(res):
    """The slow press's force against key travel, in 0.08 mm bins, down and up: each bin
    the mean of the 0.2 ms windows whose middle falls in it while the key moves."""
    f = res["force"]
    n = len(f)
    x = window_travel("slow", n)
    tm = (np.arange(n) + 0.5) * WINDOW_S
    out = {}
    for label, (a, b) in (("down", res["down"]), ("up", res["up"])):
        m = (tm > a) & (tm < b) & ~res["still"][:n]
        bins = np.floor(x[m] / TRAVEL_BIN_MM).astype(int)
        vals = {}
        for i in np.unique(bins):
            vals[int(i)] = float(np.mean(f[m][bins == i]))
        out[label] = vals
    return out


def touchweights(curves):
    """The slow press's mean force over key travel 2 to 6 mm, going down and coming up: the
    dynamic counterparts of the down weight D and the up weight U at this press's speed,
    with BW = (D + U)/2 and F = (D - U)/2 after Stanwood. Grams. They are read off a
    position-driven press, not by the trade's protocol (a quasi-static search for the
    weight that starts the key moving); 2 to 6 mm keeps clear of the ramps, of the toe on
    the button, and of the hammer landing on its rail at the end of the up stroke."""
    lo, hi = int(round(2.0 / TRAVEL_BIN_MM)), int(round(6.0 / TRAVEL_BIN_MM))
    down = [v for i, v in curves["down"].items() if lo <= i < hi]
    up = [v for i, v in curves["up"].items() if lo <= i < hi]
    d, u = float(np.mean(down)) / GRAM_N, float(np.mean(up)) / GRAM_N
    return d, u, 0.5 * (d + u), 0.5 * (d - u)


def blow_curve(res):
    """The forte or piano press's force over the descent: {window index: mean force, N}
    for the 0.2 ms windows in which the key moves."""
    f = res["force"]
    n = len(f)
    a, b = res["down"]
    e = np.arange(n + 1) * WINDOW_S
    m = (e[:-1] >= a - 1e-12) & (e[1:] <= b + 1e-12) & ~res["still"][:n]
    return {int(i): float(f[i]) for i in np.nonzero(m)[0]}


OBS = {
    # name: (unit, kind, resolution, scenarios, title)
    "force": ("g", "curve", None, ("slow", "forte", "piano"), "key force"),
    "D": ("g", "abs", RES_FORCE_SLOW_N / GRAM_N, ("slow",), "down weight D"),
    "U": ("g", "abs", RES_FORCE_SLOW_N / GRAM_N, ("slow",), "up weight U"),
    "let-off": ("mm", "abs", RES_MM, ("slow", "forte", "piano", "forte_force"), "let-off"),
    "aftertouch": ("mm", "abs", RES_MM, ("slow", "forte", "piano"), "aftertouch"),
    "travel": ("mm", "abs", RES_MM, ("forte_force",), "key travel at let-off"),
    "v let-off": ("m/s", "rel", RES_SPEED, ("slow", "forte", "piano", "forte_force"), "hammer at let-off"),
    "v string": ("m/s", "rel", RES_SPEED, ("forte", "piano", "forte_force"), "hammer at the string"),
    "drop": ("mm", "abs", RES_MM, ("slow",), "drop"),
    "checking": ("mm", "abs", RES_MM, ("forte", "piano", "forte_force"), "checking"),
    "rest": ("mm", "abs", RES_MM, ("slow", "forte", "piano", "forte_force"), "rest after release"),
}


def curve_resolution_g(name):
    return (RES_FORCE_SLOW_N if name == "slow" else RES_FORCE_BLOW_N) / GRAM_N


def observe(res):
    """Every observable of a run, as the trade reads it (OBSERVABLES in RESULTS.md)."""
    name = res["name"]
    o = {}
    if not res["finite"]:
        return None
    lo = res["let_off"]
    if lo is not None:
        o["let-off"] = lo["hammer"] * 1e3
        o["v let-off"] = lo["speed"]
        if name == "forte_force":
            o["travel"] = lo["travel"] * 1e3
        else:
            o["aftertouch"] = bottom_travel_mm(name) - lo["travel"] * 1e3
    if res["strike"] is not None and name != "slow":
        o["v string"] = res["strike"]["speed"]
    if res["hold_g12"] is not None:
        o["drop" if name == "slow" else "checking"] = res["hold_g12"] * 1e3
    o["rest"] = res["end_g12"] * 1e3
    if name == "slow":
        curves = slow_curves(res)
        o["force"] = curves
        o["D"], o["U"], o["BW"], o["F"] = touchweights(curves)
    elif name in ("forte", "piano"):
        o["force"] = blow_curve(res)
    return o


def curve_deviation_g(name, a, b, after_let_off_window=None):
    """Largest difference between two force curves where both exist, in grams; for the
    blows also split at let-off: (all, before let-off, after)."""
    if name == "slow":
        devs = []
        for label in ("down", "up"):
            common = set(a[label]) & set(b[label])
            devs.append(max(abs(a[label][i] - b[label][i]) for i in common) / GRAM_N if common else math.inf)
        return max(devs), devs[0], devs[1]
    common = sorted(set(a) & set(b))
    if not common:
        return math.inf, math.inf, math.inf
    d = {i: abs(a[i] - b[i]) / GRAM_N for i in common}
    pre = [d[i] for i in common if after_let_off_window is None or i < after_let_off_window]
    post = [d[i] for i in common if after_let_off_window is not None and i >= after_let_off_window]
    return max(d.values()), max(pre) if pre else 0.0, max(post) if post else 0.0


def deviations(name, o, ref, ref_res):
    """Each observable's deviation from the reference over its resolution (<= 1 within)."""
    if o is None:
        return None
    out = {}
    for key, (unit, kind, res_, scns, _title) in OBS.items():
        if name not in scns:
            continue
        if key not in ref:
            continue
        if key not in o:
            out[key] = math.inf
            continue
        if kind == "curve":
            lw = None
            if name in ("forte", "piano") and ref_res["let_off"] is not None:
                lw = int(ref_res["let_off"]["t"] / WINDOW_S)
            dev = curve_deviation_g(name, o[key], ref[key], lw)
            out[key] = dev[0] / curve_resolution_g(name)
            out["force parts"] = dev
        elif kind == "abs":
            out[key] = abs(o[key] - ref[key]) / res_
        else:
            out[key] = abs(o[key] - ref[key]) / (res_ * abs(ref[key]))
    return out


def worst(dev):
    return max(v for k, v in dev.items() if k != "force parts") if dev else math.inf


# =============================================================================
# Consistency checks of the variant
# =============================================================================


def check_zero_friction():
    """With every contact-friction bound zero, A-set must be integrator A on the model with
    contact friction off: a forte press of 60 ms at 5 µs, compared state by state."""
    scn = SCENARIOS["forte"]
    q0 = rest()
    plan = build(scn, q0, down_s=0.060, hold_s=0.02, release=False)
    drive, t_end = plan["phases"][0]
    dt = 5e-6
    model = mdl.Model(contact_friction=False)
    a = itg.State(0.0, q0, np.zeros(mdl.N_Q))
    b = SetState(0.0, q0, np.zeros(mdl.N_Q))
    worst_q = worst_v = 0.0
    worst_f = 0.0
    for _ in range(int(round(t_end / dt))):
        a, ia = itg.step_explicit(model, a, dt, drive)
        b, ib = step_set(model, b, dt, drive, False, contact_friction=False)
        worst_q = max(worst_q, float(np.max(np.abs(a.q - b.q))))
        worst_v = max(worst_v, float(np.max(np.abs(a.v - b.v))))
        worst_f = max(worst_f, abs(sum(ia.finger_force_N) - sum(ib.finger_force_N)) / 2)
    return worst_q, worst_v, worst_f, int(round(t_end / dt))


def check_box_solver(n_problems=300, seed=20041):
    """solve_box against an exhaustive enumeration of active sets, on random problems of
    the variant's shape: four free speeds, the four joints' rows and two contact rows, so
    W has rank four and its rows are redundant. The generalized speeds are unique and are
    compared; the enumeration takes every split of rows into stuck, sliding up and sliding
    down, solves it, and keeps the one meeting every condition."""
    rng = np.random.default_rng(seed)
    worst_dv = 0.0
    most_sweeps = 0
    import itertools
    for _ in range(n_problems):
        nf = 4
        a = rng.normal(size=(nf, nf))
        a_mat = a @ a.T + 0.1 * np.eye(nf)
        a_inv = np.linalg.inv(a_mat)
        jac = np.vstack([np.eye(nf), rng.normal(size=(2, nf))])
        y = a_inv @ jac.T
        w = jac @ y
        v_free = rng.normal(size=nf)
        u_free = jac @ v_free
        bound = rng.uniform(0.05, 1.5, size=len(u_free))
        lam, _stick, rep = solve_box(w, u_free, bound, np.zeros(len(u_free)), y)
        most_sweeps = max(most_sweeps, rep[0])
        v = v_free + y @ lam
        best = None
        for states in itertools.product((0, 1, -1), repeat=len(u_free)):
            stick = [i for i, s in enumerate(states) if s == 0]
            fixed = np.array([s * bound[i] if s else 0.0 for i, s in enumerate(states)])
            if stick:
                w_ss = w[np.ix_(stick, stick)]
                rhs = -(u_free[stick] + w[:, [i for i in range(len(states)) if i not in stick]][stick] @
                        fixed[[i for i in range(len(states)) if i not in stick]])
                sol = np.linalg.lstsq(w_ss, rhs, rcond=None)[0]
                fixed[stick] = sol
            u = u_free + w @ fixed
            ok = all(abs(fixed[i]) <= bound[i] * (1 + 1e-9) and abs(u[i]) < 1e-8 for i in stick)
            ok = ok and all(u[i] * states[i] <= 1e-12 for i in range(len(states)) if states[i] != 0)
            if ok:
                best = v_free + y @ fixed
                break
        assert best is not None
        worst_dv = max(worst_dv, float(np.max(np.abs(best - v))))
    return worst_dv, most_sweeps, n_problems


# =============================================================================
# Running everything
# =============================================================================


def job(spec):
    kind = spec[0]
    if kind == "sweep":
        _k, name, variant, dt = spec
        return spec, run(name, variant, dt)
    if kind == "descent":
        _k, down_ms = spec
        return spec, run("forte", "A-set", 5e-6, down_s=down_ms * 1e-3, hold_s=0.05, release=False)
    if kind == "modal":
        _k, name = spec
        every = 20 if name == "slow" else 4
        return spec, run(name, "A-set", 5e-6, modal_every=every)
    if kind == "threshold":
        _k, v_t, dt = spec
        saved = mdl.FRICTION_THRESHOLD_SPEED_M_S
        mdl.FRICTION_THRESHOLD_SPEED_M_S = v_t  # model.friction_coefficient reads it at each call
        try:
            return spec, run("piano", "A", dt)
        finally:
            mdl.FRICTION_THRESHOLD_SPEED_M_S = saved
    if kind == "check-zero":
        return spec, check_zero_friction()
    if kind == "check-box":
        return spec, check_box_solver()
    if kind == "rest-modes":
        return spec, rest_modes()
    raise ValueError(spec)


def job_cost(spec):
    """A rough relative cost, to start the longest jobs first."""
    kind = spec[0]
    durations = {"slow": 2.405, "forte": 0.485, "piano": 0.615, "forte_force": 0.8}
    if kind == "sweep":
        _k, name, variant, dt = spec
        return durations[name] / dt * (1.6 if variant.endswith("set") else 1.0)
    if kind == "modal":
        return durations[spec[1]] / 5e-6 * 4.0
    if kind == "descent":
        return 0.3 / 5e-6 * 1.6
    if kind == "threshold":
        return durations["piano"] / spec[2]
    return 1e5


def all_jobs():
    jobs = [("sweep", name, v, us * 1e-6) for name in SCENARIOS for v in VARIANTS for us in STEPS_US]
    jobs += [("sweep", name, v, us * 1e-6) for name, v in SUB_RUNS for us in SUB_US]
    jobs += [("sweep", name, v, us * 1e-6) for name in SCENARIOS for v in STABILITY_VARIANTS for us in STABILITY_US]
    jobs += [("descent", ms) for ms in DESCENTS_MS]
    jobs += [("threshold", v_t, us * 1e-6) for v_t, us in THRESHOLD_RUNS]
    jobs += [("modal", name) for name in SCENARIOS]
    jobs += [("check-zero",), ("check-box",), ("rest-modes",)]
    return sorted(jobs, key=job_cost, reverse=True)


def run_all(n_jobs):
    rest()  # computed once, before the workers fork
    jobs = all_jobs()
    out = {}
    t0 = time.perf_counter()
    with mp.get_context("fork").Pool(n_jobs) as pool:
        for k, (spec, result) in enumerate(pool.imap_unordered(job, jobs), 1):
            out[spec] = result
            print(f"  [{k:3d}/{len(jobs)}] {spec}  ({time.perf_counter() - t0:.0f} s)", flush=True)
    return out


# =============================================================================
# Analysis
# =============================================================================


def analyze(out):
    sweep = {(s[1], s[2], round(s[3] * 1e6, 3)): r for s, r in out.items() if s[0] == "sweep"}
    A = {}
    for name in SCENARIOS:
        obs = {(v, us): observe(sweep[(name, v, us)]) for v in VARIANTS for us in STEPS_US}
        for v in VARIANTS:
            for us in SUB_US:
                if (name, v, us) in sweep:
                    obs[(v, us)] = observe(sweep[(name, v, us)])
        refs = {}
        for model_name, pair in (("as built", ("A", "B")), ("set", ("A-set", "B-set"))):
            self_diff = {}
            for v in pair:
                d = deviations(name, obs[(v, 2)], obs[(v, 1)], sweep[(name, v, 1)])
                self_diff[v] = worst(d)
            best = min(pair, key=lambda v: self_diff[v])
            other = pair[1] if best == pair[0] else pair[0]
            cross = deviations(name, obs[(other, 1)], obs[(best, 1)], sweep[(name, best, 1)])
            refs[model_name] = dict(ref=best, other=other, self_diff=self_diff, cross=cross)
        devs = {}
        for (v, us), o in obs.items():
            r = refs[MODEL_OF[v]]
            devs[(v, us)] = deviations(name, o, obs[(r["ref"], 1)], sweep[(name, r["ref"], 1)])
        largest, limit, first_fail = {}, {}, {}
        for v in VARIANTS:
            ok = None
            for us in STEPS_US:
                d = devs[(v, us)]
                if d is None or worst(d) > 1.0:
                    break
                ok = us
            largest[v] = ok
            grid = sorted(set(STEPS_US) | (set(STABILITY_US) if v in STABILITY_VARIANTS else set()))
            limit[v] = max(us for us in grid if all(sweep[(name, v, s)]["finite"] for s in grid if s <= us))
            first_fail[v] = next((us for us in grid if not sweep[(name, v, us)]["finite"]), None)
        per_obs_largest = {}
        for v in VARIANTS:
            per = {}
            for key in list(OBS):
                if name not in OBS[key][3]:
                    continue
                ok = None
                for us in STEPS_US:
                    d = devs[(v, us)]
                    if d is None or d.get(key, math.inf) > 1.0:
                        break
                    ok = us
                per[key] = ok
            per_obs_largest[v] = per
        A[name] = dict(obs=obs, devs=devs, refs=refs, largest=largest, limit=limit, first_fail=first_fail,
                       per_obs=per_obs_largest)
    return sweep, A


# =============================================================================
# The report
# =============================================================================


TITLE = {"slow": "slow press", "forte": "forte press", "piano": "piano press", "forte_force": "forte by force"}
OBS_TITLE = {"force": "key force", "D": "D", "U": "U", "let-off": "let-off", "aftertouch": "aftertouch",
             "travel": "key travel at let-off", "v let-off": "hammer at let-off",
             "v string": "hammer at the string", "drop": "drop", "checking": "checking",
             "rest": "rest after release"}
COLUMNS = {
    "slow": [("force", "key force Δ down / up (g)"), ("D", "D (g)"), ("U", "U (g)"), ("let-off", "let-off (mm)"),
             ("aftertouch", "aftertouch (mm)"), ("v let-off", "hammer at let-off (m/s)"), ("drop", "drop (mm)"),
             ("rest", "rest after release (mm)")],
    "forte": [("force", "key force Δ to / after let-off (g)"), ("let-off", "let-off (mm)"),
              ("aftertouch", "aftertouch (mm)"), ("v let-off", "hammer at let-off (m/s)"),
              ("v string", "hammer at the string (m/s)"), ("checking", "checking (mm)"),
              ("rest", "rest after release (mm)")],
    "forte_force": [("let-off", "let-off (mm)"), ("travel", "key travel at let-off (mm)"),
                    ("v let-off", "hammer at let-off (m/s)"), ("v string", "hammer at the string (m/s)"),
                    ("checking", "checking (mm)"), ("rest", "rest after release (mm)")],
}
COLUMNS["piano"] = COLUMNS["forte"]
DIGITS = {"D": 1, "U": 1, "let-off": 3, "aftertouch": 3, "travel": 3, "v let-off": 4, "v string": 4,
          "drop": 3, "checking": 3, "rest": 3}
ASSUMED_N = mdl.S10.F_ASSUMED  # script 10's assumed fortissimo forces, behind `docs/FOUNDATIONS.md` > The time step's estimates
GFLOPS_PER_CORE = 1.0  # an ASSUMED sustained rate for this scalar, branchy code; to be measured (`docs/FOUNDATIONS.md` > The 88-note budget)


def md_table(header, rows, align=None):
    align = align or ["r"] * len(header)
    out = ["| " + " | ".join(header) + " |",
           "|" + "|".join(":---" if a == "l" else "---:" for a in align) + "|"]
    out += ["| " + " | ".join(str(x) for x in row) + " |" for row in rows]
    return out


def _cell(o, d, key):
    val = o.get(key)
    flagged = d is not None and d.get(key, 0.0) > 1.0
    text = "none" if val is None else f"{val:.{DIGITS[key]}f}"
    return text + " *" if flagged else text


def _force_cell(d, is_ref):
    if is_ref:
        return "ref"
    parts = d.get("force parts")
    text = f"{parts[1]:.1f} / {parts[2]:.1f}"
    return text + " *" if d.get("force", 0.0) > 1.0 else text


def convergence_table(A, name, variants=VARIANTS, steps=STEPS_US):
    a = A[name]
    cols = COLUMNS[name]
    rows = []
    for v in variants:
        ref = a["refs"][MODEL_OF[v]]["ref"]
        for us in steps:
            if (v, us) not in a["obs"]:
                continue
            o, d = a["obs"][(v, us)], a["devs"][(v, us)]
            is_ref = v == ref and us == 1
            step = f"{us:g}" + (" (ref)" if is_ref else "")
            if o is None:
                rows.append([v, step, "diverges"] + ["–"] * (len(cols) - 1))
                continue
            rows.append([v, step] + [_force_cell(d, is_ref) if key == "force" else
                                     _cell(o, None if is_ref else d, key) for key, _t in cols])
    return md_table(["integrator", "step (µs)"] + [t for _k, t in cols], rows, ["l", "r"] + ["r"] * len(cols))


def binding_at(A, name, v, us):
    """The observable furthest outside its resolution at step us, or the blow-up."""
    d = A[name]["devs"].get((v, us))
    if d is None:
        return "diverges"
    key = max((k for k in d if k != "force parts"), key=lambda k: d[k])
    return OBS_TITLE[key]


def summary_table(A):
    """Per integrator and scenario, the largest step within resolution and what fails next;
    and the stability limit over all four scenarios."""
    rows = []
    for v in VARIANTS:
        row = [v]
        for name in SCENARIOS:
            ok = A[name]["largest"][v]
            nxt = [us for us in STEPS_US if ok is None or us > ok]
            if ok is None:
                row.append(f"none ({binding_at(A, name, v, 1)} at 1 µs)")
            elif nxt:
                row.append(f"{ok:g} µs ({binding_at(A, name, v, nxt[0])} at {nxt[0]:g})")
            else:
                row.append(f"{ok:g} µs (the whole sweep)")
        lim = min(A[n]["limit"][v] for n in SCENARIOS)
        fail = [A[n]["first_fail"][v] for n in SCENARIOS if A[n]["first_fail"][v] is not None]
        if fail:
            nxt = min(fail)
            fails = [TITLE[n] for n in SCENARIOS if A[n]["first_fail"][v] == nxt]
            row.append(f"{lim:g} µs (diverges at {nxt:g}: {', '.join(fails)})")
        else:
            row.append(f"{lim:g} µs or more (the whole sweep)")
        rows.append(row)
    return md_table(["integrator"] + [TITLE[n] for n in SCENARIOS] + ["stability limit"], rows,
                    ["l"] * (len(SCENARIOS) + 2))


def per_observable_table(A, variants):
    rows = []
    for v in variants:
        for key in OBS:
            cells = [("none" if A[n]["per_obs"][v][key] is None else f"{A[n]['per_obs'][v][key]:g}")
                     if key in A[n]["per_obs"][v] else "" for n in SCENARIOS]
            if any(cells):
                rows.append([v, OBS_TITLE[key]] + cells)
    return md_table(["integrator", "observable"] + [f"{TITLE[n]} (µs)" for n in SCENARIOS], rows,
                    ["l", "l"] + ["r"] * len(SCENARIOS))


def largest_excluding(A, v, exclude):
    """Largest sweep step at which every observable but those in `exclude` (scenario, key)
    stays within its resolution in every scenario, at that step and every smaller one."""
    best = None
    for us in STEPS_US:
        for name in SCENARIOS:
            d = A[name]["devs"][(v, us)]
            if d is None or any(val > 1.0 for key, val in d.items()
                                if key != "force parts" and (name, key) not in exclude):
                return best
        best = us
    return best


def force_part_largest(A, name, v, part, limit_g):
    """Largest sweep step at which one part of the key-force curve (1: to let-off, or going
    down; 2: after let-off, or coming up) stays within limit_g grams of the reference."""
    best = None
    for us in STEPS_US:
        d = A[name]["devs"][(v, us)]
        if d is None or d["force parts"][part] > limit_g:
            break
        best = us
    return best


def one_norm(res, ref):
    """Thorin's J between two force curves over the descent: sum |F - F_ref| / sum |F_ref|
    over the 0.2 ms windows where the key moves in both."""
    a, b = blow_curve(res), blow_curve(ref)
    common = sorted(set(a) & set(b))
    return sum(abs(a[i] - b[i]) for i in common) / sum(abs(b[i]) for i in common)


def cost_row(sweep, name, v, us):
    r = sweep[(name, v, us)]
    n = sum(r["hist_sweeps"].values())

    def mean(h):
        return sum(k * c for k, c in h.items()) / n

    def pct(h, q):
        acc = 0
        for k in sorted(h):
            acc += h[k]
            if acc >= q * n:
                return k
        return max(h)
    steps_per_s = 1e6 / us
    mflop = r["flops_mean"] * steps_per_s / 1e6
    return dict(loaded=(mean(r["hist_loaded"]), max(r["hist_loaded"])), rows=(mean(r["hist_rows"]), max(r["hist_rows"])),
                sweeps=(mean(r["hist_sweeps"]), pct(r["hist_sweeps"], 0.999), max(r["hist_sweeps"])),
                exact=r["exact_steps"] / n, flops=(r["flops_mean"], r["flops_max"]), mflop=mflop,
                unconverged=r["unconverged"], steps=n)


def write_results(out, sweep, A, wall_s, n_jobs):
    lines = []
    w = lines.append

    def para(text, width=92):
        """A paragraph or a list of bullets, re-wrapped at `width` once the numbers are in."""
        items, current = [], None
        for line in text.strip("\n").split("\n"):
            if line.startswith("- ") or current is None:
                current = [line]
                items.append(current)
            else:
                current.append(line.strip())
        for item in items:
            bullet = item[0].startswith("- ")
            joined = " ".join(x.strip() for x in item)
            if bullet:
                joined = joined[2:]
            lines.extend(textwrap.wrap(joined, width, initial_indent="- " if bullet else "",
                                       subsequent_indent="  " if bullet else "",
                                       break_long_words=False, break_on_hyphens=False))
        w("")

    def table(rows):
        lines.extend(rows)
        w("")

    # ---------------------------------------------------------------- numbers
    q0 = rest()
    rest_hammer = mdl.Model().evaluate(q0, np.zeros(mdl.N_Q)).gap[11] * 1e3
    dip = bottom_travel_mm("forte")
    slow = SCENARIOS["slow"]
    cruise = dip / (slow.down_s - slow.ramp_s)
    rm = out[("rest-modes",)]
    zero_q, zero_v, zero_f, zero_n = out[("check-zero",)]
    box_dv, box_sweeps, box_n = out[("check-box",)]
    stab = {v: min(A[n]["limit"][v] for n in SCENARIOS) for v in VARIANTS}
    aset_fail = min(A[n]["first_fail"]["A-set"] for n in SCENARIOS if A[n]["first_fail"]["A-set"] is not None)
    aset_fail_name = next(n for n in SCENARIOS if A[n]["first_fail"]["A-set"] == aset_fail)
    deepest = max((r["max_pen"].max(), n, v, us) for (n, v, us), r in sweep.items() if r["finite"])
    largest = {v: {n: A[n]["largest"][v] for n in SCENARIOS} for v in VARIANTS}
    overall = {v: None if any(x is None for x in largest[v].values()) else min(largest[v].values()) for v in VARIANTS}
    trade_step = largest_excluding(A, "A-set", {("forte", "force")})
    forte_force_step = A["forte"]["per_obs"]["A-set"]["force"]
    forte_stroke_step = force_part_largest(A, "forte", "A-set", 1, RES_FORCE_BLOW_N / GRAM_N)
    forte_after_step = force_part_largest(A, "forte", "A-set", 2, RES_FORCE_BLOW_N / GRAM_N)
    ref_forte = sweep[("forte", "A-set", 1)]
    # The deviations are from the 1 µs reference, whose own error is first order too: a
    # first-order error c h shows against it as c (h - 1 µs). So the order is fitted on
    # h - 1 µs (fitting on h would read a first-order error as order 1.2 over 2 to 50 µs),
    # and c is the least-squares slope through zero.
    def order_and_rate(part):
        hs = np.array([us for us in STEPS_US if 2 <= us <= trade_step], dtype=float)
        dev = np.array([A["forte"]["devs"][("A-set", us)]["force parts"][part] for us in hs])
        order = float(np.polyfit(np.log(hs - 1.0), np.log(dev), 1)[0])
        rate = float(np.sum(dev * (hs - 1.0)) / np.sum((hs - 1.0) ** 2))
        return order, rate
    stroke_order, stroke_rate = order_and_rate(1)
    after_order, after_rate = order_and_rate(2)
    j_forte = {us: one_norm(sweep[("forte", "A-set", us)], ref_forte) for us in STEPS_US
               if sweep[("forte", "A-set", us)]["finite"]}
    # where the forte key force's deviation sits at the recommended step: its largest window
    # before and after let-off, and the largest more than 1 ms away from each
    dev_curve, ref_curve = blow_curve(sweep[("forte", "A-set", trade_step)]), blow_curve(ref_forte)
    lw_forte = int(ref_forte["let_off"]["t"] / WINDOW_S)
    devw = {i: abs(dev_curve[i] - ref_curve[i]) / GRAM_N for i in set(dev_curve) & set(ref_curve)}
    i_pre = max((i for i in devw if i < lw_forte), key=devw.get)
    i_post = max((i for i in devw if i >= lw_forte), key=devw.get)
    away = int(round(1e-3 / WINDOW_S))
    else_pre = max(devw[i] for i in devw if i < lw_forte and abs(i - i_pre) > away)
    else_post = max(devw[i] for i in devw if i >= lw_forte and abs(i - i_post) > away)
    t_pre, t_post = (i_pre + 0.5) * WINDOW_S, (i_post + 0.5) * WINDOW_S
    capstan_stick = min((t for t, c in ref_forte["stick_onsets"] if c == 1), key=lambda t: abs(t - t_pre))
    # the forte key force against the 10 N scale behind its 0.1 N resolution
    fmax_forte = max(ref_curve.values())
    fmax_piano = max(blow_curve(sweep[("piano", "A-set", 1)]).values())
    pct_g = 0.01 * fmax_forte / GRAM_N
    forte_pct_step = min(force_part_largest(A, "forte", "A-set", part, pct_g) or 0 for part in (1, 2))
    a_check = {us: A["forte"]["obs"][("A", us)]["checking"] for us in (0.25, 0.5, 1, 2)}
    flips = {(n, v): sum(sweep[(n, v, 1)]["flips"].values()) for n in SCENARIOS for v in VARIANTS}
    steps_1us = {n: sweep[(n, "A", 1)]["steps"] for n in SCENARIOS}
    held = max(sweep[(n, v, 1)]["hold_v"] for n in ("forte", "piano", "forte_force") for v in ("A-set", "B-set"))
    end_still = max(sweep[(n, v, 1)]["end_v"] for n in SCENARIOS for v in ("A-set", "B-set"))
    end_asbuilt = max(sweep[(n, v, 1)]["end_v"] for n in SCENARIOS for v in ("A", "B"))
    # as built, below 1 µs: successive differences of checking as the step halves, and the
    # first-order extrapolation 2 F(h/2) - F(h) from the two finest steps
    asb = {}
    for n in ("forte", "forte_force"):
        for v in ("A", "B"):
            c1, c05, c025 = (A[n]["obs"][(v, us)]["checking"] for us in (1, 0.5, 0.25))
            asb[(n, v)] = dict(d1=c05 - c1, d2=c025 - c05, limit=2 * c025 - c05,
                               paper_us=RES_MM / (abs(c025 - c05) / 0.25))
    aset_sub = max((worst(A[n]["devs"][("A-set", us)]), n, us) for n in ("forte", "piano", "forte_force")
                   for us in SUB_US)
    thr = {(s_[1], round(s_[2] * 1e6, 3)): observe(r) for s_, r in out.items() if s_[0] == "threshold"}
    thr_flips = {(s_[1], round(s_[2] * 1e6, 3)): sum(r["flips"].values()) for s_, r in out.items()
                 if s_[0] == "threshold"}
    catch = {(v, us): sweep[("forte", v, us)]["catch"]["t"] for v in ("A-set", "B-set")
             for us in (0.25, 0.5) + STEPS_US if ("forte", v, us) in sweep and sweep[("forte", v, us)]["finite"]
             and sweep[("forte", v, us)]["catch"] is not None}
    catch_ref = catch[("A-set", 1)]
    delay_per_us = (catch[("B-set", 1)] - catch_ref) / 1e-6
    modal = {n: out[("modal", n)]["modal"] for n in SCENARIOS}
    top = {n: max(modal[n], key=lambda x: x["f"]) for n in SCENARIOS}
    low = {n: min(modal[n], key=lambda x: x["limit"]) for n in SCENARIOS}
    top_all = max(top.values(), key=lambda x: x["f"])
    top_all_name = next(n for n in SCENARIOS if top[n] is top_all)
    lim_all = min(low.values(), key=lambda x: x["limit"])
    lim_all_name = next(n for n in SCENARIOS if low[n] is lim_all)
    # how long the forte press's linear limit stays below a step, from its samples
    forte_samples = modal["forte"]
    sample_s = forte_samples[1]["t"] - forte_samples[0]["t"]

    def unstable_ms(h_us):
        return 1e3 * sample_s * sum(1 for x in forte_samples if x["limit"] < h_us * 1e-6)
    clamped7 = {us: int(sweep[("forte", "A-set", us)]["clamped"][6]) for us in (trade_step, 100, stab["A-set"])}
    pos_power = max(r["positive_power"] for s, r in out.items()
                    if s[0] == "sweep" and s[2].endswith("-set") and r["finite"])
    cost_step = trade_step
    costs = {(n, v): cost_row(sweep, n, v, us) for n in SCENARIOS for v in VARIANTS
             for us in [A[n]["largest"][v] or 1]}
    aset_cost = {n: cost_row(sweep, n, "A-set", cost_step) for n in SCENARIOS}
    aset_cost_fine = {n: cost_row(sweep, n, "A-set", forte_force_step) for n in SCENARIOS}
    flops_note = max(c["flops"][0] for c in aset_cost.values())
    mflop_note = flops_note * 1e6 / cost_step / 1e6
    mflop_note_fine = max(c["flops"][0] for c in aset_cost_fine.values()) * 1e6 / forte_force_step / 1e6

    # ---------------------------------------------------------------- claims the text makes
    assert overall["A"] is None or overall["A"] <= 1
    assert overall["B"] is None or overall["B"] <= 2
    assert trade_step is not None and trade_step >= 20, trade_step
    assert stab["A-set"] < max(STEPS_US) and stab["B-set"] == max(STEPS_US)
    assert forte_force_step is not None and forte_force_step <= 5
    assert forte_stroke_step is not None and forte_stroke_step < trade_step
    assert stab["A-set"] >= trade_step
    assert pos_power <= -1.0 + 1e-9
    assert end_still < 1e-6  # "the action back at rest" at the end of the references
    assert all(catch[("B-set", us)] > catch[("A-set", 1)] for us in STEPS_US if ("B-set", us) in catch)
    assert abs(a_check[0.25] - a_check[1]) > RES_MM
    v1, v025 = A["piano"]["obs"][("A", 1)]["v string"], A["piano"]["obs"][("A", 0.25)]["v string"]
    assert abs(v025 / v1 - 1) < 0.001  # "does not move between 1 and 0.25 µs as built"
    assert trade_step * 1e-6 < min(min(x["limit"] for x in out[("modal", n)]["modal"]) for n in SCENARIOS)
    assert deepest[0] < 3e-3  # "every run that stays finite keeps every penetration under 3 mm"
    assert abs(capstan_stick - t_pre) < 2 * WINDOW_S  # "where the capstan sticks"
    assert 0.0 < t_post - ref_forte["catch"]["t"] < 1.5e-3  # "just after the back check's catch"
    assert devw[i_pre] > 2 * else_pre and devw[i_post] > 2 * else_post  # "gathered there"
    assert 3 * trade_step < aset_fail  # "under a third of the step at which A-set first diverges"
    assert fmax_piano < 10.0 < fmax_forte and forte_pct_step > forte_force_step
    assert all(0.0 < x["d2"] / x["d1"] < 0.6 for x in asb.values())  # "first order: the difference halves"
    assert end_asbuilt > 1e-6 > end_still
    assert aset_sub[0] <= 1.0  # the set model's reference is converged
    assert unstable_ms(trade_step) == 0.0 < unstable_ms(100)
    assert clamped7[trade_step] == 0 < clamped7[100]
    # "set by the contact damping": without it, the limit for that mode would be 1/(pi f)
    assert 2.0 * lim_all["limit"] < 1.0 / (np.pi * lim_all["f"])
    assert all(0.8 < x < 1.2 for x in (stroke_order, after_order))  # "first order in the step"
    for v_t in sorted({k[0] for k in thr}):  # resolving the law moves the strike speed toward the set's
        assert thr[(v_t, 0.25)]["v string"] < thr[(v_t, 1)]["v string"] < A["piano"]["obs"][("A", 1)]["v string"]

    def us_list(xs):
        return ", ".join(f"{x:g}" for x in xs)

    def sci(x):
        mantissa, exponent = f"{x:.0e}".split("e")
        return f"{mantissa}e{int(exponent)}"

    def and_list(xs):
        xs = list(xs)
        return xs[0] if len(xs) == 1 else ", ".join(xs[:-1]) + " and " + xs[-1]

    def share_text(sample):
        parts = sorted(((s, c) for c, s in sample["share"].items() if s >= 0.1), reverse=True)
        return ", ".join(f"#{c} {mdl.NAME[c]} {100 * s:.0f} % ({sample['fn'][c]:.2g} N)" for s, c in parts)

    a_forte = A["forte"]["obs"]
    set_flips_forte = flips[("forte", "A-set")]
    w("# Dynamics prototype: step convergence, stiffest modes, cost per step")
    w("")
    para(f"""
Generated by `study.py` (`python3 -I tools/dynamics-prototype/study.py`): every number in
this file is printed by it, from runs of `model.py` and `integrators.py` used unchanged. It
settles `docs/FOUNDATIONS.md` > The time step and > One dynamic solver on the
one-note model of Hirschkorn's action, and says what can and cannot yet be said about
> The felt law and > Rigid bodies. What the model assumes is in `README.md`. With the
geometry as printed, let-off comes about 13 mm from the string where Hirschkorn regulated
1 mm (`README.md`, "Reading its output"): that moves the scenarios' let-off, drop and
checking, not the way they converge. Every result here is a consistency check in the sense
of `docs/FOUNDATIONS.md` > What a test proves; drive in position, compare forces: it says how the model behaves under its integrators, not that the
model is right.
""")

    # ------------------------------------------------------------------ in short
    w("## In short")
    w("")
    asbuilt_best = max(x for v in ("A", "B") for x in largest[v].values() if x is not None)
    b_vs_a = "; ".join(TITLE[n] + " " + ("none" if largest["B-set"][n] is None else f"{largest['B-set'][n]:g} µs")
                       + f" against {largest['A-set'][n]:g} µs" for n in SCENARIOS)
    para(f"""
- **Contact friction decides how small the step must be.** As built, Eq. 2.10 is integrated
  explicitly at contacts 1, 4, 7 and 10 with its threshold speed at 1e-5 m/s, and wherever
  a loaded contact sticks its friction changes sign from one step to the next:
  {flips[("forte", "A")]:,} reversals in the {steps_1us["forte"]:,} steps of the forte press at 1 µs.
  Its 1 µs references are then not converged: in the forte press checking moves by
  {abs(a_check[0.25] - a_check[1]):.3f} mm between 1 and 0.25 µs, at first order, toward the set
  model's value, and the piano press's hammer speed at the string sits on a plateau the chatter
  sets, which a step of 0.25 µs does not leave (the section on contact friction below). A and
  B hold every observable within resolution of those references up to {asbuilt_best} µs at
  most, in any scenario. With contact friction a set, as the axes already are (A-set,
  B-set), the reversals fall to {set_flips_forte} and the observables converge. An implicit
  Eq. 2.10 with its saturation would stop the chatter too (`docs/FOUNDATIONS.md` > Set-valued dry friction, at the axes and at sliding contacts); it is not
  built here.
- **The step is {trade_step} µs.** With A-set, every trade observable (let-off, aftertouch,
  drop, checking, the rest regained after release, D and U), the hammer's speeds within
  1 %, the slow press's key force within 1 g and the piano press's within 0.1 N hold up to
  {trade_step} µs in all four scenarios. A-set stays finite up to {stab["A-set"]} µs in every
  scenario and first diverges at {aset_fail} µs, in the {TITLE[aset_fail_name]}. The forte
  press's key force is the one curve that needs a finer step to stay within 0.1 N:
  {forte_force_step} µs. Its deviation is first order in the step, about {stroke_rate:.1f} g per µs
  of step before let-off and {after_rate:.1f} g after it ({forte_stroke_step} µs suffice before
  let-off, {forte_after_step} µs after), and it is gathered where the capstan sticks and just
  after the back check's catch. At {trade_step} µs that curve's 1-norm deviation from the
  reference, Thorin's J, is {100 * j_forte[trade_step]:.2f} %.
- **The solver is A, explicit, with both frictions as one set.** B is stable at every step
  tried, but its linearly implicit Euler damps every contact mode and loses energy at each
  impact in proportion to the step: the back check catches the hammer
  {delay_per_us:.0f} µs late per µs of step in the forte press, and checking drifts with the
  step. B-set's largest step within resolution, against A-set's: {b_vs_a}. Its stability
  buys nothing here: A-set's limit lies above every step its accuracy allows.
- **The stiffest mode met is {top_all["f"]:.0f} Hz**, in the {TITLE[top_all_name]}:
  {share_text(top_all)}. `docs/FOUNDATIONS.md` > The time step's starting estimates, recomputed by script 10 at
  its assumed fortissimo forces, are {rm["s10"][0][1]:.0f}, {rm["s10"][1][1]:.0f} and
  {rm["s10"][2][1]:.0f} Hz; the largest forces the scenarios produce are tabulated against
  those assumed below.
- **A note costs about {flops_note:.0f} floating-point operations a step** with A-set, plus
  {TRANSCENDENTAL_CALLS} transcendental calls: {mflop_note:.0f} Mflop per simulated second at
  {trade_step} µs, {88 * mflop_note / 1000:.1f} GFLOP/s if all 88 notes moved at once.
""")

    # ------------------------------------------------------------------ integrators
    w("## The four integrators")
    w("")
    table(md_table(
        ["integrator", "contact normal laws", "contact friction (contacts 1, 4, 7, 10)", "axis friction", "code"],
        [["A", "explicit: semi-implicit Euler", "Eq. 2.10, explicit, v_t = 1e-5 m/s", "a set", "`integrators.py`"],
         ["B", "linearly implicit: one Newton step of backward Euler on their stiffness and damping",
          "Eq. 2.10, explicit", "a set", "`integrators.py`"],
         ["A-set", "as A", "a set, solved with the axes", "a set", "`study.py`, `step_set`"],
         ["B-set", "as B", "a set, solved with the axes", "a set", "`study.py`, `step_set`"]],
        ["l", "l", "l", "l", "l"]))
    peaks = ", ".join(f"{PEAK[c]:.3f}" for c in FRICTIONAL)
    para(f"""
A-set and B-set are a variant built for this study, to measure what the owner's choice of
contact-friction law changes; `model.py` and `integrators.py` are not changed. Their contact
friction is Eq. 2.10's limit as v_t goes to zero, which leaves no threshold speed: a contact
that slides carries A f_n against its slip; one that sticks carries any force up to
A p f_n, p the law's peak over A ({peaks} at contacts 1, 4, 7, 10). f_n is the normal force
at the start of the step, and which bound applies depends on whether the contact ended the
previous step stuck, hidden state the step carries. Each step solves one box problem at
velocity level: one row per free joint (bound A_i dt) and one per loaded frictional contact
(its slip row, bound mu f_n dt). A projected Gauss-Seidel sweep guesses which rows stick, and
an exact active-set pass, pivoting from that guess, finishes the solve. Where stuck rows are
redundant (a joint and a contact holding the same body) it takes the split of least
utilization, sum (lambda_i / bound_i)^2, so the split, and the finger force that depends on
it, does not drift from step to step. The checks at the end of this file test it.
""")

    # ------------------------------------------------------------------ observables
    w("## Observables and the resolution each is judged at")
    w("")
    table(md_table(
        ["observable", "read as", "resolution", "why that resolution"],
        [["key force, slow press",
          "the force at Pk2, vertical, that drives the key through the profile (the key's own row of each "
          "step); its mean in 0.08 mm bins of key travel, going down and coming up, compared bin by bin",
          "1 g", "touchweights are read to the gram (Stanwood: down and up weight \"to the nearest gram\")"],
         ["D, U (slow press)",
          "the mean of that force over key travel 2 to 6 mm, going down and coming up: this press's "
          "counterparts of the down and up weight at its speed, not the trade's protocol (a quasi-static "
          "search for the weight that starts the key), which is the quasi-static solver's", "1 g", "as above"],
         ["key force, forte and piano presses",
          "the same force in 0.2 ms windows (5 kHz, the rate of Hirschkorn's load cell) over the descent, "
          "compared window by window, split at let-off", "0.1 N",
          "the trade has no reading of force in a blow; the measured curve it will be compared with, "
          "Thorin's position-driven key force, comes from a sensor specified within ±1 % of its usual 10 N "
          "scale (`docs/research/A2-scientific-literature.md` > Thorin 2013/2017: the set richest in FORCES), so a step moving the curve by less cannot change that comparison. "
          f"The piano press's key force peaks at {fmax_piano:.1f} N, within that scale; the forte press's at "
          f"{fmax_forte:.0f} N, past it, so 0.1 N is stricter there than that sensor"],
         ["let-off",
          "the hammer-to-string distance at the instant the jack leaves the knuckle (contact 7 opens after "
          "the toe has met the button), interpolated within the step (`docs/FOUNDATIONS.md` > The acceptance criterion)", "0.08 mm",
          "the thinnest punching paper, the finest regulation quantum the trade uses anywhere in the action "
          "(`docs/FOUNDATIONS.md` > The acceptance criterion); the trade specifies hammer-side distances to 1/32\" (0.79 mm) at best "
          "(`docs/research/A3-nomenclature-and-regulation.md` > Master table of the keyboard and action settings), so it is a strict choice"],
         ["aftertouch (the let-off instant)",
          f"the key travel at Pk2 from the let-off instant to the bottom ({dip:.2f} mm, the key 0.3 mm into "
          "the front punching); in the forte by force, whose bottom is not prescribed, the key travel at "
          "let-off", "0.08 mm", "a punching paper: aftertouch is set through the key dip, with papers"],
         ["hammer at let-off, at the string",
          "the speed of the hammer's striking point toward the string (the rate of contact 12's gap) at "
          "the let-off instant and at the instant the gap closes, interpolated", "1 %",
          "the trade has no reading of it; the reproducing piano that records it, the Bösendorfer SE, counts "
          "the time over the last 5 mm at 25.6 kHz, a quantum of v²/128 m/s: 0.8 % at 1 m/s, 1.6 % at 2 m/s "
          "(`docs/research/A5-numerical-formulation.md` > MIDI velocity 0–127 mapping)"],
         ["drop (slow press)",
          "the hammer-to-string distance with the key held at the bottom after let-off, the hammer resting on "
          "the repetition lever, mean over the last 20 ms of the hold", "0.08 mm", "as let-off"],
         ["checking",
          "the hammer-to-string distance with the key held at the bottom after the blow, the hammer caught by "
          "the back check, mean over the last 20 ms of the hold", "0.08 mm", "as let-off"],
         ["rest after release",
          f"the hammer-to-string distance at the end of the run, the key released and the action back at rest "
          f"(the largest joint speed at the end of the set model's 1 µs runs is {end_still:.1e} rad/s; as built, "
          f"where contact friction chatters, {end_asbuilt:.1e} rad/s; the runs start from the frictionless "
          f"equilibrium, the hammer {rest_hammer:.2f} mm from the string)",
          "0.08 mm", "as let-off"]],
        ["l", "l", "r", "l"]))
    para("""
A deviation is measured from the reference run of the same model (below); a value outside its
resolution is marked `*` in the tables. The key force while the key is still is an interval
(the key's own axis friction) and is not judged.
""")

    # ------------------------------------------------------------------ scenarios
    w("## Scenarios")
    w("")
    fs = SCENARIOS["forte"]
    ps = SCENARIOS["piano"]
    table(md_table(
        ["scenario", "drive", "profile", "where the profile comes from"],
        [["slow press", "key position",
          f"rest {1e3 * slow.start_s:g} ms; down {slow.down_s:g} s at a constant {cruise:.1f} mm/s at Pk2 between "
          f"smooth {1e3 * slow.ramp_s:g} ms ramps; held {slow.hold_s:g} s; up the same way; "
          f"{slow.settle_s:g} s at rest",
          "a weighing-like press: slow, controlled motion of the key, read for D and U; the release is "
          "the up stroke"],
         ["forte press", "key position, then released",
          f"quintic smoothstep down in {1e3 * fs.down_s:g} ms; held {fs.hold_s:g} s; then the finger lets go and "
          f"the key is free for {fs.settle_s:g} s",
          "a forte keystroke reaches the key bottom in about 25 ms (Askenfelt and Jansson, cited by Goebl et al. "
          "2005, p. 1154); a 25 ms smoothstep drives the back check into the hammer's tail before let-off with "
          "this geometry (descent table), 30 ms is the fastest in 5 ms steps that does not"],
         ["piano press", "key position, then released",
          f"the same, down in {1e3 * ps.down_s:g} ms",
          "about 160 ms to the key bottom (the same source); Hirschkorn's piano blow takes about three times "
          "his forte's 0.06 s (§4.2.1, p. 48)"],
         ["forte by force", "force at Pk2", f"`scenarios.py`'s FORTE_LIKE_N for {SCENARIOS['forte_force'].t_end:g} s; "
          "the force falls to zero by 0.35 s, the release",
          "shaped after Hirschkorn's forte (§4.1, p. 44): 10 N reached at 0.11 s, then about 34 N at the key "
          "bottom, held about 0.2 s; not his measured input, which is not printed"]],
        ["l", "l", "l", "l"]))
    para("""
Every run starts from the frictionless equilibrium of `scenarios.py` (rest_state). The key's
bottom, 0.3 mm into the front-rail punching, is `scenarios.py`'s assumption. The descent
study runs A-set at 5 µs with descents from 25 to 200 ms (held 50 ms): the largest finger
force and back-check force before let-off, where let-off falls, and the hammer's speed at
the string. A back-check force before let-off means the back check met the hammer's tail
before the jack escaped.
""")
    rows = []
    for ms in DESCENTS_MS:
        r = out[("descent", ms)]
        lo, st = r["let_off"], r["strike"]
        f = r["force"]
        n = len(f)
        e = np.arange(n + 1) * WINDOW_S
        t_lo = lo["t"] if lo else math.inf
        m = (e[1:] <= t_lo) & ~r["still"][:n]
        note = ""
        if lo is None and r["check_before_strike_N"] > 0.0:
            note = "the back check reaches the hammer's tail while the jack still holds the knuckle"
        elif lo is None and st is not None:
            note = "the hammer thrown off the jack: it strikes, then falls into the back check, without let-off"
        rows.append([f"{ms}", f"{f[m].max():.1f}" if m.any() else "–", f"{r['check_before_let_off_N']:.1f}",
                     f"{lo['hammer'] * 1e3:.2f}" if lo else "none", f"{st['speed']:.3f}" if st else "none", note])
    table(md_table(["descent (ms)", "finger force before let-off, peak (N)", "back check before let-off, peak (N)",
                    "let-off (mm)", "hammer at the string (m/s)", "what happens"], rows,
                   ["r", "r", "r", "r", "r", "l"]))
    para(f"""
A key descending in a smoothstep slows down over its second half. With let-off about 13 mm
from the string, a descent of 80 or 100 ms throws the hammer off the jack before the toe
reaches the button; the forte and piano presses ({1e3 * fs.down_s:g} and
{1e3 * ps.down_s:g} ms) let off normally.
""")

    # ------------------------------------------------------------------ convergence
    w("## Step convergence")
    w("")
    para("""
Each scenario runs at fixed steps of 1, 2, 5, 10, 20, 50, 100, 200 and 500 µs under each
integrator. The reference of a model is the 1 µs run of its more accurate integrator, the one
whose 2 µs run departs less from its own 1 µs run, in resolutions; the other integrator's 1 µs
run is the cross-check. As built and with the set are two models, each judged against its
own reference.
""")
    rows = []
    for name in SCENARIOS:
        for model_name in ("as built", "set"):
            r = A[name]["refs"][model_name]
            cross = r["cross"]
            key = max((k for k in cross if k != "force parts"), key=lambda k: cross[k])
            rows.append([TITLE[name], model_name, f"{r['ref']} at 1 µs",
                         f"{r['self_diff'][r['ref']]:.2f} ({r['other']}: {r['self_diff'][r['other']]:.2f})",
                         f"{r['other']} at 1 µs: {cross[key]:.2f}, on {OBS_TITLE[key]}"])
    table(md_table(["scenario", "model", "reference", "2 µs against 1 µs, in resolutions",
                    "cross-check: largest deviation, in resolutions"], rows, ["l", "l", "l", "r", "l"]))
    cross_fail = [(n, m) for n in SCENARIOS for m in ("as built", "set") if worst(A[n]["refs"][m]["cross"]) > 1.0]
    assert cross_fail == [("forte", "as built"), ("forte", "set")], cross_fail
    para("""
The cross-check fails in the forte press only, for each model, and on the key force. As
built, the 1 µs runs are not converged: contact friction chatters (the section after the
tables). With the set, B-set departs from A-set at 1 µs on the key force after let-off only:
B's numerical damping delays the back check's catch, and the force at the finger rises by
tens of newtons within a millisecond there. Below 1 µs, B-set closes on A-set's reference
(the second table after the convergence tables).
""")
    for name in SCENARIOS:
        w(f"### {TITLE[name].capitalize()}")
        w("")
        table(convergence_table(A, name))
    o_ref = A["slow"]["obs"][("A-set", 1)]
    o_asb = A["slow"]["obs"][("A", 1)]
    para(f"""
Key force Δ: the largest deviation from the reference over the bins (slow press: going
down / coming up) or the windows (forte and piano: to let-off / after it), in grams. Read the
trade's way, the slow press's reference gives D = {o_ref["D"]:.1f} g and U = {o_ref["U"]:.1f} g,
so a balance weight BW = (D + U)/2 = {o_ref["BW"]:.1f} g and a friction F = (D - U)/2 =
{o_ref["F"]:.1f} g (as built: D = {o_asb["D"]:.1f}, U = {o_asb["U"]:.1f}, BW = {o_asb["BW"]:.1f},
F = {o_asb["F"]:.1f} g); these are the printed geometry's, at this press's speed.
""")

    w("### Below 1 µs")
    w("")
    bset = [A["forte"]["devs"][("B-set", us)]["force parts"][2] for us in (1, 0.5, 0.25)]
    para(f"""
A and B as built, A-set, and B-set, at 0.5 and 0.25 µs, against the same references. A-set
stays within {aset_sub[0]:.2f} of a resolution of its 1 µs reference at both steps in every
scenario (the most in the {TITLE[aset_sub[1]]} at {aset_sub[2]:g} µs): the set model's reference
is converged. In the forte press B-set's key force after let-off departs from it by
{bset[0]:.0f}, {bset[1]:.0f} and {bset[2]:.0f} g at 1, 0.5 and 0.25 µs: it halves with the step, as
B's first-order damping would make it, so the two integrators close on the same curve, and
B-set would need about {RES_FORCE_BLOW_N / GRAM_N / (bset[2] / 0.25):.2f} µs to come within 0.1 N.
""")
    for name in ("forte", "piano", "forte_force"):
        variants = [v for v in VARIANTS if (v, 0.25) in A[name]["obs"]]
        w(f"{TITLE[name].capitalize()}:")
        w("")
        table(convergence_table(A, name, variants, (0.25, 0.5, 1, 2)))

    w("### Largest step within resolution, and stability limit")
    w("")
    para(f"""
The largest step of the sweep at which, and at every smaller step of which, every observable
of the scenario stays within its resolution of the reference; in brackets, what fails at the
next step. The stability limit is the largest step at which every scenario's run stays
finite, and in brackets the step at which one first diverges; for A and A-set it is found on
a finer grid, with {us_list(STABILITY_US)} µs added to the sweep's steps. Every run that stays
finite keeps every penetration under 3 mm (the deepest, {1e3 * deepest[0]:.2f} mm, in the
{TITLE[deepest[1]]} under {deepest[2]} at {deepest[3]:g} µs).
""")
    table(summary_table(A))
    para("""
Each observable's own largest step within resolution (blank where the scenario has no such
observable; none: not even at 1 µs):
""")
    table(per_observable_table(A, VARIANTS))

    # ------------------------------------------------------------------ chatter and the law
    w("## Contact friction as built: chatter, and what the law changes")
    w("")
    para("""
Sign reversals of the friction force at the four frictional contacts, step to step while the
contact stays loaded, over each run at 1 µs. Where a loaded contact slides, its friction
seldom reverses; where it sticks, explicit Eq. 2.10 reverses at almost every step, since its
stable step there is a few nanoseconds (`README.md`, `scenarios.py chatter-study`).
""")
    rows = []
    for name in SCENARIOS:
        rows.append([TITLE[name], f"{steps_1us[name]:,}"] + [f"{flips[(name, v)]:,}" for v in VARIANTS])
    table(md_table(["scenario", "steps"] + list(VARIANTS), rows, ["l", "r", "r", "r", "r", "r"]))
    paper = [x["paper_us"] for x in asb.values()]
    para(f"""
As the step falls below 1 µs the as-built model's checking keeps moving toward the set
model's: {a_check[1]:.3f}, {a_check[0.5]:.3f} and {a_check[0.25]:.3f} mm at 1, 0.5 and 0.25 µs
in the forte press under A, against {a_forte[("A-set", 1)]["checking"]:.3f} mm with A-set
(tables above). It converges, slowly: chatter lets the hammer held by the back check creep
toward the string through the hold, by an amount proportional to the step (`README.md`,
"Reading its output"), so each halving of the step moves checking by about half as much as
the last ({asb[("forte", "A")]["d1"]:+.3f} then {asb[("forte", "A")]["d2"]:+.3f} mm under A,
{asb[("forte", "B")]["d1"]:+.3f} then {asb[("forte", "B")]["d2"]:+.3f} mm under B). Extrapolated at
first order, A and B as built reach {asb[("forte", "A")]["limit"]:.3f} and
{asb[("forte", "B")]["limit"]:.3f} mm in the forte press, against the set model's
{a_forte[("A-set", 1)]["checking"]:.3f} mm, and {asb[("forte_force", "A")]["limit"]:.3f} and
{asb[("forte_force", "B")]["limit"]:.3f} mm in the forte by force, against
{A["forte_force"]["obs"][("A-set", 1)]["checking"]:.3f} mm. As built, checking within a punching
paper of that limit needs a step of {min(paper):.1f} to {max(paper):.1f} µs; with A-set,
{A["forte"]["per_obs"]["A-set"]["checking"]} µs holds it.
""")
    rows = []
    lw_ref = {n: int(sweep[(n, "A-set", 1)]["let_off"]["t"] / WINDOW_S) for n in ("forte", "piano")}
    for name in SCENARIOS:
        sub = 0.25 if ("A", 0.25) in A[name]["obs"] else 1
        o_b, o_s = A[name]["obs"][("A", sub)], A[name]["obs"][("A-set", 1)]
        if "force" in o_b and "force" in o_s:
            parts = curve_deviation_g(name, o_b["force"], o_s["force"], lw_ref.get(name))
            dev = parts[0] / curve_resolution_g(name)
            split = "down / up" if name == "slow" else "to / after let-off"
            rows.append([TITLE[name], f"key force, largest difference {split} (g)",
                         f"{parts[1]:.1f} / {parts[2]:.1f} (A, {sub:g} µs)", "–",
                         f"{dev:.2f}" + (" *" if dev > 1.0 else ""), ""])
        for key in OBS:
            if key == "force" or key not in o_b or key not in o_s:
                continue
            unit, kind, res_, _scns, _t = OBS[key]
            dev = abs(o_b[key] - o_s[key]) / (res_ * abs(o_s[key]) if kind == "rel" else res_)
            rows.append([TITLE[name], OBS_TITLE[key], f"{o_b[key]:.{DIGITS[key]}f} (A, {sub:g} µs)",
                         f"{o_s[key]:.{DIGITS[key]}f}", f"{dev:.2f}" + (" *" if dev > 1.0 else ""),
                         f", {100 * (o_b[key] / o_s[key] - 1):+.1f} %" if kind == "rel" else ""])
    para("""
What the contact-friction law changes, each model at its smallest step: as built (A at
0.25 µs, or 1 µs in the slow press) against the set (A-set at 1 µs), in resolutions:
""")
    table(md_table(["scenario", "observable", "as built", "set", "difference (resolutions)"],
                   [r[:5] for r in rows], ["l", "l", "r", "r", "r"]))
    o_b, o_s = A["piano"]["obs"][("A", 0.25)], A["piano"]["obs"][("A-set", 1)]
    piano_diff = 100 * (o_b["v string"] / o_s["v string"] - 1)
    beyond = [r for r in rows if r[4].endswith("*") and not r[1].startswith("key force")]
    beyond_text = "; ".join(f"the {r[0]}'s {r[1]}, {r[2].split(' ')[0]} against {r[3]} ({r[4][:-2]} resolutions{r[5]})"
                            for r in beyond)
    force_beyond = [r[0] for r in rows if r[1].startswith("key force") and r[4].endswith("*")]
    assert any(r[0] == "piano press" and r[1] == "hammer at the string" for r in beyond) and force_beyond
    v_set = o_s["v string"]
    thr_v = {k: thr[k]["v string"] for k in thr}
    v_ts = sorted({k[0] for k in thr})
    assert all(thr_v[(v_t, 0.25)] < v_set < o_b["v string"] for v_t in v_ts)  # "below the set model's"
    assert all(thr_v[(a, 0.25)] < thr_v[(b, 0.25)] for a, b in zip(v_ts[1:], v_ts[:-1]))  # "more as it rises"
    assert all(thr_flips[(v_t, 0.25)] < thr_flips[(v_t, 1)] for v_t in v_ts)
    thr_text = "; ".join(f"with {sci(v_t)} m/s, from {thr_v[(v_t, 1)]:.4f} to {thr_v[(v_t, 0.25)]:.4f} m/s "
                         f"({thr_flips[(v_t, 1)]:,} and {thr_flips[(v_t, 0.25)]:,} reversals)" for v_t in v_ts)
    para(f"""
The two models differ beyond resolution on: {beyond_text}; and on the key force in the
{and_list(force_beyond)}. At the steps tried, these are differences between the set and an
explicit Eq. 2.10 that chatters, not yet between two laws. The piano press's hammer at the string shows it: as built
it differs by {piano_diff:+.1f} % and barely moves between 1 and 0.25 µs, a plateau rather than
a converged value. With the threshold speed raised to where a step of 0.25 µs begins to
resolve the law, the same refinement from 1 to 0.25 µs moves it a long way: {thr_text}, against
the set model's {v_set:.4f} m/s. So the {piano_diff:+.1f} % is the explicit integration's, not
the law's: resolved, the law gives a strike speed below the set model's, the further below
the higher its threshold speed, and approaches the set model's as that speed falls. Whether
the slow press's differences are the law's or the chatter's, the sweep does not say. The
piano blow is where Hirschkorn's own model fails (`docs/FOUNDATIONS.md` > The felt law), so the friction law
belongs in that test alongside the felt law, integrated so that it converges.
""")

    # ------------------------------------------------------------------ B's damping
    w("## Why B needs a smaller step than A")
    w("")
    rows = []
    for us in (0.25, 0.5) + STEPS_US:
        cells = [f"{us:g}"]
        for v in ("A-set", "B-set"):
            if (v, us) in catch:
                cells.append(f"{1e3 * catch[(v, us)]:.3f}")
                cells.append(f"{A['forte']['obs'][(v, us)]['checking']:.3f}")
            else:
                cells += ["–", "–"]
        rows.append(cells)
    para(f"""
B solves the contact laws by one Newton step of backward Euler. That is stable at any step,
but backward Euler damps an undamped mode of frequency omega by 1/sqrt(1 + (omega dt)^2) per
step, so an impact loses energy in proportion to the step. In the forte press the hammer
leaves the string slower and returns later, and the back check catches it
{delay_per_us:.0f} µs late per µs of step:
""")
    table(md_table(["step (µs)", "A-set: catch (ms)", "A-set: checking (mm)", "B-set: catch (ms)",
                    "B-set: checking (mm)"], rows))
    a_catch_spread = max(abs(catch[("A-set", us)] - catch_ref) for us in STEPS_US
                         if us <= trade_step and ("A-set", us) in catch)
    para(f"""
A (semi-implicit Euler) keeps an undamped contact mode's energy bounded instead: up to
{trade_step} µs its catch stays within {1e6 * a_catch_spread:.0f} µs of the reference. B's step
is therefore set by accuracy long before stability matters, and A-set stays stable up to
{stab["A-set"]} µs, above the step its own accuracy allows.
""")

    # ------------------------------------------------------------------ modes
    w("## The stiffest linearized modes")
    w("")
    para(f"""
Along each scenario's A-set run at 5 µs (every 20 µs; every 100 µs in the slow press), the
largest generalized eigenvalue of (K, M): K the stiffness of the loaded contact laws and the
repetition spring at the forces the run produces (`model.py`, `Evaluation.K`), M the mass
matrix, over the coordinates free at that instant (the key's own excluded while it is driven
in position). The contacts carrying the mode are those holding at least 10 % of its
stiffness. The explicit step's linear stability limit is the largest step at which integrator
A's step, linearized there with the contact damping C as explicit as A has it, keeps every
mode bounded, the contact damping included.
""")
    rows = []
    for name in SCENARIOS:
        t, l = top[name], low[name]
        rows.append([TITLE[name], f"{t['f']:.0f}", f"{1e3 * t['t']:.2f}", share_text(t),
                     f"{1e6 * l['limit']:.0f}", f"{1e3 * l['t']:.2f}", share_text(l)])
    table(md_table(["scenario", "highest mode (Hz)", "at (ms)", "carried by", "least explicit limit (µs)",
                    "at (ms)", "carried by"], rows, ["l", "r", "r", "l", "r", "r", "l"]))
    top_c = max(top_all["share"], key=lambda c: top_all["share"][c])
    s10 = [x[1] for x in rm["s10"]]
    where = ("between" if s10[0] < top_all["f"] < s10[-1] else "below" if top_all["f"] <= s10[0] else "above")
    para(f"""
At rest the highest mode is {rm["free"]["f"]:.0f} Hz with the key free and {rm["held"]["f"]:.0f} Hz
with it held ({share_text(rm["free"])}). `docs/FOUNDATIONS.md` > The time step starts from script 10's estimates at
its closure with contacts loaded at assumed fortissimo forces, recomputed here by script 10's
own function: {s10[0]:.0f} Hz ({rm["s10"][0][0]}), {s10[1]:.0f} Hz ({rm["s10"][1][0]}) and
{s10[2]:.0f} Hz ({rm["s10"][2][0]}). The highest mode met in the four scenarios,
{top_all["f"]:.0f} Hz, lies {where} {"them" if where == "between" else "them all"}; it is carried
by #{top_c} {mdl.NAME[top_c]} at {top_all["fn"][top_c]:.1f} N, a contact the estimates load at
an assumed {ASSUMED_N[top_c]} N. The largest force each contact meets in the references:
""")
    rows = []
    for c in mdl.CONTACTS:
        met = max(sweep[(n, "A-set", 1)]["max_fn"][c - 1] for n in SCENARIOS)
        where = max(SCENARIOS, key=lambda n: sweep[(n, "A-set", 1)]["max_fn"][c - 1])
        rows.append([f"#{c} {mdl.NAME[c]}", f"{met:.1f}", TITLE[where], f"{ASSUMED_N[c]}"])
    table(md_table(["contact", "largest force met (N)", "in", "script 10's assumed force (N)"], rows,
                   ["l", "r", "l", "r"]))
    met = {c: max(sweep[(n, "A-set", 1)]["max_fn"][c - 1] for n in SCENARIOS) for c in mdl.CONTACTS}
    above = [f"#{c}" for c in mdl.CONTACTS if met[c] > ASSUMED_N[c]]
    below = [f"#{c}" for c in mdl.CONTACTS if met[c] <= ASSUMED_N[c]]
    para(f"""
The scenarios press harder than the estimates assume at {", ".join(above) or "no contact"}, and
less hard at {", ".join(below) or "none"}. The explicit step's least linear limit is
{1e6 * lim_all["limit"]:.0f} µs, in the {TITLE[lim_all_name]} at {1e3 * lim_all["t"]:.2f} ms
({share_text(lim_all)}). The stiffest mode at that instant is only {lim_all["f"]:.0f} Hz: what
sets the limit there is the contact damping, which A integrates explicitly and which grows
with the contact force (Hunt and Crossley's f(δ) D δ̇), not the stiffness. So the limit falls
as a blow gets harder, and the forte press is the hardest blow run here. A-set's runs stay
finite up to {stab["A-set"]} µs in every scenario, although in the forte press the linear limit
stays below 100 µs for {unstable_ms(100):.1f} ms and below {stab["A-set"]} µs for
{unstable_ms(stab["A-set"]):.1f} ms; they first diverge at {aset_fail} µs, in the
{TITLE[aset_fail_name]}. Through such a stretch the explicit damping drives an oscillation of
the jack/knuckle's force that the clamp max(0, f) bounds: before the toe meets the button, the
clamp zeroes that force while the knuckle still presses into the jack on {clamped7[100]} steps at
100 µs and {clamped7[stab["A-set"]]} at {stab["A-set"]} µs, and on none at {trade_step} µs, where the
linear limit is never exceeded. Any change to the felt laws moves these modes and these
limits, and the study reruns in minutes.
""")

    # ------------------------------------------------------------------ cost
    w("## Cost per step")
    w("")
    terms = "; ".join(f"{t} ({f})" for t, f in COST_TERMS)
    para(f"""
Operations an engine would do per step, counted from what each step of these runs did (each
add, multiply, divide or compare counted once): {terms}; then {PER_LOADED_FLOPS} per loaded
contact (lever arms, rate, law, generalized force), {PER_FRICTION_EQ210_FLOPS} per loaded
frictional contact for explicit Eq. 2.10 (plus one tanh), and for B {B_ASSEMBLY_FLOPS} to
assemble its system plus {B_PER_LOADED_FLOPS} per loaded contact; a Cholesky factorization of
the 4 x 4 or 5 x 5 system, the box problem's matrix (rows x rows), each PGS sweep, each exact
solve on a stuck set of size s ((2/3) s^3 + 4 s^2, plus a check), and the updates. Every step
also calls {TRANSCENDENTALS}. Each integrator at its largest step within resolution (1 µs where
it has none):
""")
    rows = []
    for v in VARIANTS:
        for name in SCENARIOS:
            us = A[name]["largest"][v] or 1
            c = costs[(name, v)]
            rows.append([v, TITLE[name], f"{us:g}", f"{c['loaded'][0]:.1f} / {c['loaded'][1]}",
                         f"{c['rows'][0]:.1f} / {c['rows'][1]}",
                         f"{c['sweeps'][0]:.2f} / {c['sweeps'][1]} / {c['sweeps'][2]}",
                         f"{100 * c['exact']:.0f}", f"{c['flops'][0]:.0f} / {c['flops'][1]}",
                         f"{c['mflop']:,.0f}", f"{88 * c['mflop'] / 1000:,.1f}"])
    table(md_table(["integrator", "scenario", "step (µs)", "loaded contacts, mean / max", "box rows, mean / max",
                    "PGS sweeps, mean / 99.9 % / max", "exact pass (% of steps)", "flops per step, mean / max",
                    "Mflop per simulated s", "88 notes moving (GFLOP/s)"], rows,
                   ["l", "l", "r", "r", "r", "r", "r", "r", "r", "r"]))
    rows = []
    for us in (trade_step, forte_force_step):
        for name in SCENARIOS:
            c = cost_row(sweep, name, "A-set", us)
            rows.append([f"{us:g}", TITLE[name], f"{c['flops'][0]:.0f}", f"{c['mflop']:,.0f}",
                         f"{88 * c['mflop'] / 1000:,.1f}", f"{88 * c['mflop'] / 1000 / GFLOPS_PER_CORE:,.1f}"])
    para(f"""
A-set at the recommended step and at the step the forte key force needs, every scenario. The
last column divides by an assumed {GFLOPS_PER_CORE:g} GFLOP/s sustained per core for this
scalar, branching code: an assumption to be replaced by `docs/FOUNDATIONS.md` > The 88-note budget's measurement,
natively and in browsers. Python's wall time is no evidence of either.
""")
    table(md_table(["step (µs)", "scenario", "flops per step", "Mflop per simulated s",
                    "88 notes moving (GFLOP/s)", "cores at the assumed rate"], rows,
                   ["r", "l", "r", "r", "r", "r"]))
    para(f"""
Every scenario's contacts are all evaluated every step (13 gaps); four to five are loaded on
average. With contact friction a set, a single PGS sweep followed by the exact pass solves
nearly every step; as built, PGS takes about two sweeps on the axes alone. A note costs about
{mflop_note:.0f} Mflop per simulated second at {trade_step} µs and {mflop_note_fine:,.0f} at
{forte_force_step} µs. A note at rest need not be stepped at all; `docs/FOUNDATIONS.md` > The 88-note budget counts on far
fewer than 88 notes moving at once, and the count over a corpus of MIDI files is its test.
""")

    # ------------------------------------------------------------------ recommendation
    hold = sweep[("forte", "A-set", 1)]
    hold_fn4 = hold["hold_fn"][3]
    punching_n = mdl.normal_force(mdl.SPECS[2], BOTTOM_PENETRATION_M, 0.0)[0]
    stroke_ms = 1e3 * (ref_forte["let_off"]["t"] - SCENARIOS["forte"].start_s)
    parts = A["forte"]["devs"][("A-set", trade_step)]["force parts"]
    b_largest = {n: A[n]["largest"]["B-set"] for n in SCENARIOS}
    w("## Recommendation")
    w("")
    w("### The time step")
    w("")
    para(f"""
- **One fixed step of {trade_step} µs, with A-set.** Every trade observable in all four
  scenarios is within its resolution there: let-off, aftertouch, drop, checking, the rest
  regained after release, D and U, and the hammer's speed at let-off and at the string within
  1 %. So are the slow press's key force (1 g) and the piano press's (0.1 N). It is under a
  third of the step at which A-set first diverges ({aset_fail} µs), and below the least linear
  limit met ({1e6 * lim_all["limit"]:.0f} µs). That limit is set by the contact damping and falls
  as a blow gets harder, so the margin is the forte press's, not the step's: a harder blow,
  which the printed geometry jams (the 25 ms descent), is to be run once the model is
  regulated.
- **The one curve outside its resolution at {trade_step} µs** is the forte press's key force
  under position drive: {parts[1]:.0f} g before let-off and {parts[2]:.0f} g after it, against
  10 g, with J = {100 * j_forte[trade_step]:.2f} % over the descent. Read within 0.1 N it needs
  {forte_force_step} µs: a run of the same solver at a finer fixed step, done offline when the
  bench compares a forte key force with a measured one ({mflop_note_fine:,.0f} Mflop per
  simulated second for the note). Read within 1 % of its own peak, {fmax_forte:.0f} N, it holds
  to {forte_pct_step} µs.
- **No sub-steps on the real-time path.** No observable but that curve asks for a finer step
  anywhere. That curve's error is gathered at events: before let-off at {1e3 * t_pre:.1f} ms,
  where the capstan sticks ({1e3 * capstan_stick:.2f} ms), {devw[i_pre]:.0f} g at {trade_step} µs
  against at most {else_pre:.0f} g elsewhere before let-off; after let-off at
  {1e3 * t_post:.1f} ms, {1e3 * (t_post - ref_forte["catch"]["t"]):.1f} ms after the back check's
  catch, {devw[i_post]:.0f} g against at most {else_post:.0f} g elsewhere. Sub-steps at those
  events might keep it within 0.1 N at a larger step than {forte_force_step} µs; that is not
  tested here.
- The keyboard drives its keys by force (`docs/FOUNDATIONS.md` > The 88-note runtime), as the forte by force does, and
  that scenario holds every observable up to {largest["A-set"]["forte_force"]} µs with A-set.
""")
    w("### One dynamic solver")
    w("")
    para(f"""
- **Integrator A, explicit, with the axis and the contact friction in one box problem at
  velocity level (A-set).** Its accuracy sets the step; B's stability is never needed below
  A's limit, and B's numerical damping costs it a far smaller step for the same
  observables: B-set holds
  {", ".join(("none" if b_largest[n] is None else f"{b_largest[n]:g} µs") + f" ({TITLE[n]})" for n in SCENARIOS)},
  where A-set holds {", ".join(f"{largest['A-set'][n]:g} µs" for n in SCENARIOS)}.
- That revises the hypothesis as `docs/FOUNDATIONS.md` > One dynamic solver states it, compliant contacts treated
  implicitly: measured, the contact laws are better explicit than under backward Euler, the
  implicit step tried here; an implicit step without its numerical damping is not tested (what
  remains open, below). The quasi-static solver is then not the dynamic step's static limit:
  it stays a separate solve on the same contacts and the same friction sets, inertia dropped,
  as `docs/FOUNDATIONS.md` > One dynamic solver describes it.
- The box problem has 4 to 8 rows. One PGS sweep and an exact active-set pass solve almost
  every step; for the real-time path a fixed budget of a sweep and a bounded number of pivots
  keeps the cost per step bounded (`docs/FOUNDATIONS.md` > Determinism, stated correctly).
""")
    w("### The decision the owner owes: contact friction")
    w("")
    para(f"""
- As built, Eq. 2.10 is integrated explicitly, and at the steps tried it either converges
  slowly, at first order (checking within a punching paper needs a step of {min(paper):.1f} to
  {max(paper):.1f} µs), or sits on a plateau its chatter sets (the piano press's hammer at the
  string, barely moving from 1 to 0.25 µs, {piano_diff:+.1f} % from the set model's). That says
  nothing against the law itself: integrated implicitly with its saturation, which this
  prototype does not build, a velocity-only law sets no step limit (`docs/FOUNDATIONS.md` > Set-valued dry friction, at the axes and at sliding contacts). What
  counts against it is its threshold speed, which has no physical value and would stay in
  every sensitivity, as `docs/FOUNDATIONS.md` > Set-valued dry friction, at the axes and at sliding contacts argues for the axes; where the law is resolved, that speed moves
  the piano press's strike speed (the section on contact friction).
- Its limit as the threshold speed goes to zero, a set with static coefficient A p and
  kinetic A, converges at {trade_step} µs{" and holds a body it sticks exactly still" if held == 0.0 else ""}.
  Against the as-built model at its smallest step it differs beyond resolution on
  {" and ".join(f"the {r[0]}'s {r[1]} ({r[4][:-2]} resolutions{r[5]})" for r in beyond)},
  and on the key force in the {and_list(force_beyond)}; at those steps the as-built model
  has not converged, so these differences are not yet the law's, and the piano press's is
  the chatter's.
- Recommended: contact friction as a set, like the axes, for that reason, and because it
  is the variant measured here. The choice is the owner's.
""")
    w("### What remains open")
    w("")
    para(f"""
- **The felt law.** Its test is each law against Hirschkorn's measured forte and piano
  blows, which exist only as figures in his thesis (Figs. 3.9 to 3.13 and 4.1 to 4.15) and are
  not digitized: no law has been compared. What this study does say: the cubic with its damping
  and the clamp runs stably with A-set up to {stab["A-set"]} µs; its tangent stiffness sets the
  stiffest modes and its damping the least explicit limit, so a stiffer law, a more damped
  one, or one with steep unloading branches, moves the step: this study reruns with each law.
  Without hysteresis, the hammer caught by the back check keeps the catch's elastic energy: in
  the forte press the back check holds it with {hold_fn4:.0f} N through the hold, and the finger
  pushes {hold["hold_finger"]:.0f} N to
  keep the key down, {punching_n:.0f} N of it on the front punching. A rate-independent law with
  loading and unloading branches would release most of that. The piano press uses Table A.8's
  forte-calibrated damping factors; Hirschkorn's piano recalibration is not printed. As built,
  the contact friction's chatter moves the piano press's hammer speed at the string by
  {piano_diff:+.1f} % at every step tried, so the friction law has to be integrated so that it
  converges before that test reads the piano blow.
- **Rigid bodies.** Not tested: the model has no degree of freedom between head and
  shank, and the record gives no shank stiffness. Thorin's hammer has its first mode at 39 Hz
  (`docs/FOUNDATIONS.md` > Rigid bodies); the forte press's stroke from rest to let-off lasts {stroke_ms:.0f} ms
  here, so (f tau)^-1 = {1 / (39 * stroke_ms / 1e3):.1f}: the mode is not fast against the stroke,
  and a flexible shank should show in the key force under position drive. The added coordinate
  brings a 39 Hz mode, far below the stiffest contact mode met ({top_all["f"]:.0f} Hz), but it
  also splits the hammer's mass, so the string and the knuckle see less of it: the stiffest
  modes and the explicit limit have to be recomputed with it, which this study does by rerunning.
  The system grows from 5 x 5 to 6 x 6.
- **The regulation.** With the geometry as printed let-off comes about 13 mm from the string.
  Regulated to Hirschkorn's 1 mm, the hammer leaves the jack about 1 mm from the string, so
  let-off, the strike and the lever's stop on the drop screw come much closer together in time.
  Rerun this study once the model is regulated, with a blow harder than the forte press: the
  least explicit limit falls as the force at the jack/knuckle rises.
- **A second-order explicit step** (the velocity-Verlet form of A) could relax the forte key
  force's step; it is not tested here. Nor is an implicit step without numerical damping
  (theta = 1/2), which would matter only where A's limit falls below the step accuracy
  allows: a stiffer law, or a harder blow, since the least limit is the contact damping's.
- **One note.** Note 52 only: heavier bass hammers and lighter treble parts move the modes.
- **The profiles.** The presses are smoothsteps, zero speed at both ends, not measured key
  motions; the forte by force is shaped from Hirschkorn's description.
""")

    # ------------------------------------------------------------------ checks
    w("## Checks of the set variant")
    w("")

    para(f"""
- With every contact-friction bound zero, A-set reproduces integrator A on the model without
  contact friction: {zero_n} steps of a 60 ms forte press at 5 µs, positions
  {"identical" if zero_q == 0.0 else f"within {zero_q:.1e} rad"}, speeds within {zero_v:.1e} rad/s,
  finger force within {zero_f:.1e} N.
- The box solver against an exhaustive enumeration of active sets, on {box_n} random problems
  of its shape (four free speeds, six rows, rank four): generalized speeds within
  {box_dv:.1e}, at most {box_sweeps} PGS sweep{"s" if box_sweeps > 1 else ""} before the exact pass.
- Contact friction never does positive work: over every step of every A-set and B-set run
  that stays finite, the largest ratio of a sliding contact's friction power to its bound is
  {pos_power:.6f} (-1 when sliding against its bound).
- A body held by friction stays still: at the end of the hold, the largest joint speed in the
  A-set and B-set references of the forte, piano and forte-by-force scenarios is
  {"exactly zero" if held == 0.0 else f"{held:.1e} rad/s"}.
""")
    w("## Reproducing")
    w("")
    para(f"""
`python3 -I tools/dynamics-prototype/study.py` reruns every run and rewrites this file; here the
runs took {wall_s:.0f} s on {n_jobs} worker processes. `--save FILE` also keeps the raw results,
and `--load FILE` rewrites this file from them without running. The runs use numpy and the
platform's math functions, so another machine may differ in the last digits (`README.md`).
""")
    return lines


def main(argv=None):
    p = argparse.ArgumentParser(description="The studies that settle `docs/FOUNDATIONS.md` > The time step and `docs/FOUNDATIONS.md` > One dynamic solver on "
                                            "the one-note model; writes RESULTS.md beside this file.")
    p.add_argument("--jobs", type=int, default=os.cpu_count() or 1,
                   help="worker processes (default: every core)")
    p.add_argument("--save", help="also pickle the raw results to this file")
    p.add_argument("--load", help="rewrite RESULTS.md from results pickled by --save, without running")
    a = p.parse_args(argv)
    t0 = time.perf_counter()
    if a.load:
        import pickle
        with open(a.load, "rb") as fh:
            out, wall_runs, n_jobs = pickle.load(fh)
    else:
        n_jobs = max(1, a.jobs)
        out = run_all(n_jobs)
        wall_runs = time.perf_counter() - t0
        if a.save:
            import pickle
            with open(a.save, "wb") as fh:
                pickle.dump((out, wall_runs, n_jobs), fh)
    sweep, analysis = analyze(out)
    lines = write_results(out, sweep, analysis, wall_runs, n_jobs)
    (HERE / "RESULTS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"RESULTS.md written; the runs took {wall_runs:.0f} s on {n_jobs} worker processes")


if __name__ == "__main__":
    main()
