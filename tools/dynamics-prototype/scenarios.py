"""Scenarios for the one-note model: the initial state, the two drives, a runner.

THROWAWAY PROTOTYPE (README.md).

Initial state: script 10's closure pose is geometric, not an equilibrium (its
lever overlaps its stop by 0.88 mm, about 16 N the repetition spring cannot
hold), so every scenario starts from the frictionless static equilibrium found
from it (model.static_equilibrium), at rest. Axis friction holds that state
exactly, its net torques being zero.

Drives (integrators.py; model.py A8 for the finger's point and direction):

  position   The key's angle prescribed by a quintic smoothstep
             s(u) = 10u³ - 15u⁴ + 6u⁵ (zero speed and acceleration at both
             ends): at rest, down in `down_s`, held `hold_s`, up in `up_s`.
             The bottom is set by how far the front-rail punching is compressed
             (`bottom_penetration_mm`, ASSUMED 0.3 mm by default), solved on
             contact 2's gap, which depends on θk alone. Output: the finger's
             force at Pk2.
  forte, piano
             Force profiles at Pk2, piecewise linear. They are SHAPED AFTER
             Hirschkorn's descriptions, not his inputs, whose measured tables
             are not printed: forte, 0 N until 0.05 s, rising to 10 N at 0.11 s,
             a spike to about 34 N at the key bottom, held about 0.20 s, then
             released (§4.1, p. 44); piano, rising to about 1.5 N, spiking to
             about 3 N at the key bottom, about three times longer to reach it
             (§3.2.1, p. 35; §4.2.1, p. 48). The spike's timing here is fixed in
             time, where his followed the key bottom.

Observables, read off the trajectory (a first pass, not the trade's protocols):
let-off as the hammer-to-string distance at the instant the jack/knuckle gap
opens, after the jack's toe has met the button; the strike as the instant the
hammer/string gap closes; both interpolated linearly within the step, with the
hammer head's speed towards the string at that instant; and the
hammer-to-string distance at the first step the back check catches the tail.
Diagnostics: the friction solver's sweeps and residual, the contact-friction
sign reversals that count chatter, the largest change of the finger force
between consecutive steps before the toe meets the button, and the finger
force over the hold (Recorder).

Run: python3 -I scenarios.py position|forte|piano|rest [--integrator A|B]
     [--dt seconds] [--t-end seconds]
     python3 -I scenarios.py slope-study   (integrators.py, why (B) keeps the
     contact friction explicit; about two minutes)
     python3 -I scenarios.py letoff-geometry   (where the printed geometry
     puts let-off, rigid and quasi-static)
     python3 -I scenarios.py chatter-study   (Eq. 2.10's explicit step limit
     at the back check in the hold, and at the jack/knuckle once the jack
     slides; a few seconds)
     python3 -I scenarios.py capstan-study   (the capstan's chatter in the
     finger force during the stroke; a few seconds)
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # python3 -I drops it
sys.dont_write_bytecode = True  # no __pycache__ beside the prototype

import numpy as np  # noqa: E402

import integrators as itg  # noqa: E402
import model as mdl  # noqa: E402

_EQUILIBRIUM = {}


def rest_state(model=None):
    """The frictionless static equilibrium found from script 10's closure, at
    rest, with its solver's report. Computed once per model configuration."""
    model = model or mdl.Model()
    key = (model.contacts, model.contact_friction, model.springs, model.gravity)
    if key not in _EQUILIBRIUM:
        q_closure, _start = mdl.script_10_rest_pose()
        q, iterations, residual = mdl.static_equilibrium(model, q_closure)
        _EQUILIBRIUM[key] = (q, iterations, residual)
    q, iterations, residual = _EQUILIBRIUM[key]
    return itg.State(0.0, q, np.zeros(mdl.N_Q)), iterations, residual


def smoothstep(u):
    """Quintic smoothstep on [0, 1]: C2, zero speed and acceleration at both ends."""
    u = min(max(u, 0.0), 1.0)
    return u * u * u * (10.0 + u * (-15.0 + 6.0 * u))


def key_angle_at_penetration(q_rest, penetration_m):
    """The key angle at which the front-rail punching (contact 2) is compressed
    by `penetration_m`; contact 2's gap depends on θk alone. Newton on θk."""
    spec = mdl.SPECS[2]
    q = np.array(q_rest, dtype=float)
    for _ in range(100):
        pose = mdl.Pose(q)
        gap, nx, ny, px, py, _ = mdl.contact_geometry(spec, pose)
        grad, _jt = mdl.lever_arms(spec, pose, nx, ny, px, py)
        step = (-penetration_m - gap) / grad[mdl.K]
        q[mdl.K] += step
        if abs(step) < 1e-15:
            break
    return float(q[mdl.K])


class KeyStroke:
    """Position profile: rest until `start_s`, down, hold, up, rest."""

    def __init__(self, angle_rest, angle_bottom, start_s=0.01, down_s=0.05, hold_s=0.10,
                 up_s=0.05):
        self.angle_rest, self.angle_bottom = angle_rest, angle_bottom
        self.start_s, self.down_s, self.hold_s, self.up_s = start_s, down_s, hold_s, up_s

    def __call__(self, t):
        travel = self.angle_bottom - self.angle_rest
        t_down_end = self.start_s + self.down_s
        t_up_start = t_down_end + self.hold_s
        if t <= self.start_s:
            return self.angle_rest
        if t <= t_down_end:
            return self.angle_rest + travel * smoothstep((t - self.start_s) / self.down_s)
        if t <= t_up_start:
            return self.angle_bottom
        return self.angle_bottom - travel * smoothstep((t - t_up_start) / self.up_s)


class PiecewiseLinear:
    def __init__(self, points):
        self.t = np.array([p[0] for p in points], dtype=float)
        self.f = np.array([p[1] for p in points], dtype=float)

    def __call__(self, t):
        return float(np.interp(t, self.t, self.f))


# Shaped after Hirschkorn's descriptions (module docstring); not his inputs.
FORTE_LIKE_N = PiecewiseLinear([(0.0, 0.0), (0.05, 0.0), (0.11, 10.0), (0.12, 34.0),
                                (0.31, 34.0), (0.35, 0.0), (1.0, 0.0)])
PIANO_LIKE_N = PiecewiseLinear([(0.0, 0.0), (0.05, 0.0), (0.23, 1.5), (0.24, 3.0),
                                (0.43, 3.0), (0.47, 0.0), (1.0, 0.0)])


def make_drive(name, q_rest, bottom_penetration_mm=0.3):
    if name == "position":
        bottom = key_angle_at_penetration(q_rest, bottom_penetration_mm * 1e-3)
        return itg.PositionDrive(KeyStroke(float(q_rest[mdl.K]), bottom))
    if name == "forte":
        return itg.ForceDrive(FORTE_LIKE_N)
    if name == "piano":
        return itg.ForceDrive(PIANO_LIKE_N)
    if name == "rest":
        return itg.FreeDrive()
    raise ValueError(name)


class Recorder:
    """Keeps a thinned time series and the events the summary reports, and two
    diagnostics. Reversals: per frictional contact, the steps at which its
    friction force changes sign from one step to the next while the contact
    stays loaded. At a contact that slides they are rare; at one that sticks
    under load they count the step-to-step chatter of an explicitly integrated
    Eq. 2.10, whose stable step there is f_n A (1 + B1) / v_t against the
    tangential mass, nanoseconds at the back check under 90 N. Jump: the
    largest change of the finger force between consecutive steps before the
    toe meets the button, which shows the capstan's chatter in the output
    where its slip passes through zero (README.md). Hold: the finger force's
    mean, least and greatest over a window (the position drive's hold), which
    shows the back check's."""

    def __init__(self, model, every, hold_window=None):
        self.model = model
        self.every = every
        self.hold_window = hold_window
        self.hold_forces = []
        self.reversals = {c: 0 for c in sorted(mdl.FRICTION_LAW)}
        self.previous_ft = {c: 0.0 for c in sorted(mdl.FRICTION_LAW)}
        self.k = 0
        self.rows = []
        self.max_iterations = 0
        self.unconverged = 0
        self.unpolished = 0
        self.max_residual = 0.0
        self.button_met = False
        self.let_off = None
        self.strike = None
        self.check = None
        self.peak_finger_N = None
        self.y_finger_rest = None
        self.previous_finger_N = None
        self.finger_jump_N = 0.0

    def finger_height(self, q):
        return mdl.Pose(q).to_world("k", *mdl.FINGER_POINT_LOCAL)[1]

    @staticmethod
    def gap_and_rate(c, q, v):
        """One contact's gap and its rate ∇g·q̇."""
        spec = mdl.SPECS[c]
        pose = mdl.Pose(q)
        gap, nx, ny, px, py, _branch = mdl.contact_geometry(spec, pose)
        grad, _jt = mdl.lever_arms(spec, pose, nx, ny, px, py)
        return gap, float(np.dot(grad, v))

    def __call__(self, before, after, info):
        ev = info.evaluation
        self.max_iterations = max(self.max_iterations, info.iterations)
        self.unconverged += 0 if info.converged else 1
        self.unpolished += 0 if (info.polished or not info.sticking) else 1
        self.max_residual = max(self.max_residual, info.residual)
        if self.y_finger_rest is None:
            self.y_finger_rest = self.finger_height(before.q)
        fn = ev.fn
        if fn[8 - 1] > 0.0:
            self.button_met = True
        g7a, _r7a = self.gap_and_rate(7, before.q, before.v)
        g7b, _r7b = self.gap_and_rate(7, after.q, after.v)
        g12a, r12a = self.gap_and_rate(12, before.q, before.v)
        g12b, r12b = self.gap_and_rate(12, after.q, after.v)
        head_speed = -r12a  # towards the string, m/s
        dt = after.t - before.t
        # events at the zero crossing of a gap, interpolated linearly in the step
        if self.button_met and self.let_off is None and g7a <= 0.0 < g7b:
            u = -g7a / (g7b - g7a)
            self.let_off = (before.t + u * dt, g12a + u * (g12b - g12a), -(r12a + u * (r12b - r12a)))
        if self.strike is None and g12a > 0.0 >= g12b:
            u = g12a / (g12a - g12b)
            self.strike = (before.t + u * dt, -(r12a + u * (r12b - r12a)))
        if self.strike is not None and self.check is None and fn[4 - 1] > 0.0:
            self.check = (before.t, ev.gap[12 - 1])
        for c in self.reversals:
            ft = ev.ft[c - 1]
            if ft * self.previous_ft[c] < 0.0:
                self.reversals[c] += 1
            self.previous_ft[c] = ft
        if info.finger_force_N is not None:
            if not self.button_met:
                mid = 0.5 * (info.finger_force_N[0] + info.finger_force_N[1])
                if self.previous_finger_N is not None:
                    self.finger_jump_N = max(self.finger_jump_N, abs(mid - self.previous_finger_N))
                self.previous_finger_N = mid
            hi = info.finger_force_N[1]
            if self.peak_finger_N is None or hi > self.peak_finger_N[1]:
                self.peak_finger_N = (before.t, hi)
            if self.hold_window and self.hold_window[0] <= before.t < self.hold_window[1]:
                self.hold_forces.append(0.5 * (info.finger_force_N[0] + hi))
        if self.k % self.every == 0:
            travel_mm = (self.y_finger_rest - self.finger_height(before.q)) * 1e3
            finger = info.finger_force_N if info.finger_force_N is not None else (np.nan, np.nan)
            self.rows.append((before.t, travel_mm, finger[0], finger[1], ev.gap[12 - 1] * 1e3,
                              head_speed, fn[1 - 1], fn[2 - 1], fn[7 - 1], fn[10 - 1],
                              info.iterations))
        self.k += 1


def run(name, integrator="B", dt=2e-6, t_end=0.25, every=None, quiet=False):
    model = mdl.Model()
    state, eq_iterations, eq_residual = rest_state(model)
    drive = make_drive(name, state.q)
    n_steps = int(round(t_end / dt))
    hold = None
    if name == "position":
        stroke = drive.key_angle_rad
        hold = (stroke.start_s + stroke.down_s + 0.005, stroke.start_s + stroke.down_s + stroke.hold_s)
    rec = Recorder(model, every or max(1, int(round(0.002 / dt))), hold_window=hold)
    t0 = time.perf_counter()
    final, steps, finite = itg.simulate(model, state, dt, n_steps, drive, integrator, observe=rec)
    wall = time.perf_counter() - t0
    if not quiet:
        print(f"scenario {name}, integrator {integrator}, dt {dt:.3g} s, {steps} steps"
              f" of {n_steps}, finite {finite}, {wall:.1f} s wall,"
              f" {wall / max(steps, 1) * 1e6:.0f} us/step")
        print(f"  initial state: frictionless equilibrium in {eq_iterations} iterations,"
              f" residual {eq_residual:.1e} N·m")
        ev0 = model.evaluate(state.q, state.v)
        for c in mdl.CONTACTS:
            print(f"    #{c:2d} {mdl.NAME[c]:32s} gap {ev0.gap[c - 1] * 1e3:+8.3f} mm"
                  f"  force {ev0.fn[c - 1]:7.4f} N")
        print("  t (ms)  travel@Pk2 (mm)  finger lo/hi (N)  hammer-string (mm)  head speed (m/s)"
              "  f1 f2 f7 f10 (N)  PGS")
        for r in rec.rows:
            print(f"  {r[0] * 1e3:6.1f}  {r[1]:7.3f}  {r[2]:7.3f} {r[3]:7.3f}  {r[4]:8.3f}  {r[5]:+7.3f}"
                  f"  {r[6]:7.3f} {r[7]:7.3f} {r[8]:7.3f} {r[9]:7.3f}  {r[10]:3d}")
        if rec.let_off:
            t, gap, speed = rec.let_off
            print(f"  let-off at {t * 1e3:.2f} ms: hammer {gap * 1e3:.2f} mm from the string,"
                  f" head at {speed:+.3f} m/s")
        else:
            print("  no let-off")
        if rec.strike:
            print(f"  strike at {rec.strike[0] * 1e3:.2f} ms, head at {rec.strike[1]:+.3f} m/s")
        if rec.check:
            print(f"  back check catches at {rec.check[0] * 1e3:.2f} ms, hammer"
                  f" {rec.check[1] * 1e3:.2f} mm from the string")
        if rec.peak_finger_N:
            print(f"  peak finger force {rec.peak_finger_N[1]:.3f} N at"
                  f" {rec.peak_finger_N[0] * 1e3:.2f} ms")
            print(f"  finger force before the toe meets the button: largest change between"
                  f" consecutive steps {rec.finger_jump_N:.3f} N")
        if rec.hold_forces:
            f = np.array(rec.hold_forces)
            print(f"  finger force over the hold ({hold[0] * 1e3:.0f} to {hold[1] * 1e3:.0f} ms,"
                  f" key still, midpoint of its friction interval): mean {f.mean():.2f} N,"
                  f" least {f.min():.2f} N, greatest {f.max():.2f} N")
        print("  contact friction sign reversals (chatter where a contact sticks under load): "
              + ", ".join(f"#{c} {n}" for c, n in rec.reversals.items()))
        print(f"  axis friction: max PGS sweeps {rec.max_iterations}, unconverged steps"
              f" {rec.unconverged}, steps with a stuck joint left unpolished {rec.unpolished},"
              f" max residual {rec.max_residual:.1e}")
    return final, rec, finite


def strike_on_forte(integrator, dt, implicit_friction_slope=False, t_end=0.104):
    """The forte-like force drive up to just past the strike: (let-off, strike)
    as the Recorder reports them."""
    model = mdl.Model(implicit_friction_slope=implicit_friction_slope)
    state, _iterations, _residual = rest_state(model)
    rec = Recorder(model, 10**9)
    itg.simulate(model, state, dt, int(round(t_end / dt)), make_drive("forte", state.q),
                 integrator, observe=rec)
    return rec.let_off, rec.strike


def slope_study():
    """The comparison integrators.py cites for leaving the contact friction
    explicit in (B): the forte-like drive's let-off and strike at five steps, for
    (A), for (B), and for (B) with Eq. 2.10's slope linearised into C."""
    print("integrator  friction slope  dt (us)  let-off (ms)  hammer-string (mm)"
          "  strike (ms)  head at strike (m/s)")
    for integrator, slope in (("A", False), ("B", False), ("B", True)):
        for dt in (1e-5, 5e-6, 2e-6, 1e-6, 5e-7):
            let_off, strike = strike_on_forte(integrator, dt, slope)
            print(f"    {integrator}       {'implicit' if slope else 'explicit':8s}    {dt * 1e6:5.1f}"
                  f"    {let_off[0] * 1e3:8.3f}      {let_off[1] * 1e3:8.3f}        "
                  f"{strike[0] * 1e3:8.3f}      {strike[1]:8.4f}", flush=True)


def letoff_geometry():
    """Where the printed geometry puts let-off, without dynamics: from script
    10's closure the key is turned with the capstan on the heel, the jack on its
    rest and the knuckle on the jack all held closed (contacts 1, 5 and 7 at
    zero gap, rigid), until the jack's toe meets the button (contact 8);
    returns the hammer-to-string distance then (m) and the key's rotation."""
    q0, _start = mdl.script_10_rest_pose()

    def lifted(key_angle):
        q = q0.copy()
        q[mdl.K] = key_angle
        for _ in range(50):
            kin = mdl.Model.contact_kinematics(q)
            r = np.array([kin[c][0] for c in (1, 5, 7)])
            jac = np.array([[kin[c][3][i] for i in (mdl.W, mdl.J, mdl.H)] for c in (1, 5, 7)])
            step = np.linalg.solve(jac, -r)
            q[[mdl.W, mdl.J, mdl.H]] += step
            if np.max(np.abs(step)) < 1e-14:
                break
        return q

    lo, hi = q0[mdl.K], q0[mdl.K] - 0.08
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if mdl.Model.gaps(lifted(mid))[8 - 1] > 0.0:
            lo = mid
        else:
            hi = mid
    q = lifted(0.5 * (lo + hi))
    gap12 = mdl.Model.gaps(q)[12 - 1]
    print(f"toe meets button with the hammer {gap12 * 1e3:.2f} mm from the string,"
          f" key turned {(q[mdl.K] - q0[mdl.K]) * 1e3:+.2f} mrad (rigid, quasi-static)")
    return gap12, q[mdl.K] - q0[mdl.K]


def pose_with_net_torques(model, target, q0):
    """A pose near q0 where the modelled forces at rest, Q(q, 0), equal
    `target` (N·m per coordinate), by Newton with a central-difference
    Jacobian: there the axis friction must supply exactly -target."""
    q = np.array(q0, dtype=float)

    def force(x):
        return model.evaluate(x, np.zeros(mdl.N_Q)).Q

    for _ in range(12):
        jac = np.zeros((mdl.N_Q, mdl.N_Q))
        for j in range(mdl.N_Q):
            e = np.zeros(mdl.N_Q)
            e[j] = 1e-9
            jac[:, j] = (force(q + e) - force(q - e)) / 2e-9
        q = q - np.linalg.solve(jac, force(q) - target)
    return q


def chatter_study(at_s=0.070, window_s=4e-6):
    """The position drive with (B) at 2 µs up to `at_s`, in its hold, then
    restarted with (A) at 10, 2 and 1 ns for `window_s`: friction sign
    reversals at the back check (contact 4) and the finger force over the
    window's second half. Shows the explicit stability limit of Eq. 2.10 at a
    contact that sticks under load. Then the same limit at the jack/knuckle,
    loaded at rest by the hammer: the action posed so that the jack carries
    1.05 times its axis-friction bound and every other joint half of its own,
    with two patterns of signs, released from rest for 10 ms at 10 and 1 µs in
    (A) and (B)."""
    model = mdl.Model()
    state, _iterations, _residual = rest_state(model)
    drive = make_drive("position", state.q)
    dt = 2e-6
    held, _steps, _finite = itg.simulate(model, state, dt, int(round(at_s / dt)), drive, "B")
    print(f"restart at {held.t * 1e3:.1f} ms, in the hold; integrator A")
    for fine in (1e-8, 2e-9, 1e-9):
        start = held.copy()
        start.impulse = start.impulse * (fine / dt)
        record = {"flips": 0, "previous": 0.0, "finger": [], "ft": []}

        def watch(_before, _after, info, record=record):
            ft = info.evaluation.ft[4 - 1]
            if ft * record["previous"] < 0.0:
                record["flips"] += 1
            record["previous"] = ft
            record["finger"].append(0.5 * sum(info.finger_force_N))
            record["ft"].append(ft)

        n = int(round(window_s / fine))
        itg.simulate(model, start, fine, n, drive, "A", observe=watch)
        finger, ft = np.array(record["finger"][n // 2:]), np.array(record["ft"][n // 2:])
        print(f"  dt {fine * 1e9:4.0f} ns: {n} steps, {record['flips']} sign reversals at the"
              f" back check; second half: finger {finger.min():.2f} to {finger.max():.2f} N,"
              f" back-check friction {ft.min():+.2f} to {ft.max():+.2f} N")
    bound = mdl.AXIS_FRICTION_NM
    print("the jack at 1.05 times its axis-friction bound (net torque positive), each other joint"
          " at half of its own, from rest, over 10 ms:")
    for signs in ((1, -1, 1, -1, 1), (1, 1, 1, 1, 1)):
        target = 0.5 * bound * np.array(signs, dtype=float)
        target[mdl.J] = 1.05 * bound[mdl.J]
        posed = pose_with_net_torques(model, target, state.q)
        print(f"  the other joints' torques of signs {signs[:2] + signs[3:]} (key, wippen, lever,"
              f" hammer)")
        for integrator in ("A", "B"):
            for fine in (1e-5, 1e-6):
                record = {"flips": 0, "previous": 0.0}

                def watch_jack(_before, _after, info, record=record):
                    ft = info.evaluation.ft[7 - 1]
                    if ft * record["previous"] < 0.0:
                        record["flips"] += 1
                    record["previous"] = ft

                final, _steps, _finite = itg.simulate(model, itg.State(0.0, posed, np.zeros(mdl.N_Q)),
                                                      fine, int(round(0.01 / fine)), itg.FreeDrive(),
                                                      integrator, observe=watch_jack)
                moved = (final.q - posed) * 1e6
                print(f"    {integrator} at {fine * 1e6:2.0f} us: jack {moved[mdl.J]:+.3f} µrad, wippen"
                      f" {moved[mdl.W]:+.3f}, hammer {moved[mdl.H]:+.3f}; {record['flips']} sign"
                      f" reversals at the jack/knuckle in {int(round(0.01 / fine))} steps")


def capstan_study(at_s=0.0286, window_s=0.0012):
    """The position drive with (B) at 2 µs up to `at_s`, in the stroke where
    the capstan's slip passes through zero, then restarted at 2 and 1 µs with
    (B) and at 0.2 and 0.05 µs with (A) for `window_s`: the capstan friction's
    sign reversals, the largest change of the finger force between
    consecutive steps, and the finger force averaged over each 0.1 ms. The
    first 20 µs after the restart are left out of all three, since the key's
    speed, a backward difference of its profile, changes estimate there."""
    model = mdl.Model()
    state, _iterations, _residual = rest_state(model)
    drive = make_drive("position", state.q)
    dt = 2e-6
    start, _steps, _finite = itg.simulate(model, state, dt, int(round(at_s / dt)), drive, "B")
    t0 = start.t + 2e-5
    edges = t0 + 1e-4 * np.arange(int((window_s - 2e-5) / 1e-4) + 1)
    print(f"restart at {start.t * 1e3:.1f} ms; finger force averaged over each 0.1 ms from"
          f" {t0 * 1e3:.2f} ms")
    for integrator, fine in (("B", 2e-6), ("B", 1e-6), ("A", 2e-7), ("A", 5e-8)):
        restart = start.copy()
        restart.impulse = restart.impulse * (fine / dt)
        rows = []

        def watch(before, _after, info, rows=rows):
            rows.append((before.t, 0.5 * sum(info.finger_force_N), info.evaluation.ft[1 - 1]))

        itg.simulate(model, restart, fine, int(round(window_s / fine)), drive, integrator,
                     observe=watch)
        r = np.array(rows)
        r = r[r[:, 0] >= t0]
        jump = float(np.max(np.abs(np.diff(r[:, 1]))))
        flips = int(np.sum(r[1:, 2] * r[:-1, 2] < 0.0))
        means = [r[(r[:, 0] >= a) & (r[:, 0] < b), 1].mean() for a, b in zip(edges[:-1], edges[1:])]
        print(f"  {integrator} at {fine * 1e6:4.2f} us: {flips:4d} capstan friction reversals,"
              f" largest step-to-step change {jump:.3f} N; means (N) "
              + " ".join(f"{m:.3f}" for m in means), flush=True)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("scenario", choices=("position", "forte", "piano", "rest", "slope-study",
                                        "letoff-geometry", "chatter-study", "capstan-study"))
    p.add_argument("--integrator", choices=("A", "B"), default="B")
    p.add_argument("--dt", type=float, default=2e-6)
    p.add_argument("--t-end", type=float, default=0.25)
    a = p.parse_args(argv)
    if a.scenario == "slope-study":
        slope_study()
    elif a.scenario == "letoff-geometry":
        letoff_geometry()
    elif a.scenario == "chatter-study":
        chatter_study()
    elif a.scenario == "capstan-study":
        capstan_study()
    else:
        run(a.scenario, a.integrator, a.dt, a.t_end)


if __name__ == "__main__":
    main()
