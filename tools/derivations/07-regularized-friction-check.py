"""Friction and damping laws: what each one does, checked by arithmetic.

1. A regularized joint friction (tanh with threshold speed w_t) does not erase
   the down weight / up weight threshold in practice: under a sub-threshold
   excess it creeps, slowly. What it costs is a non-physical parameter, w_t,
   and, only if it is integrated explicitly, a tiny time step.
2. Thorin's additive contact damping b_d d^2 d' against Hunt-Crossley's
   multiplicative f(d)(1 + D d'), with one constant b_d. Both forms turn
   attractive when the surfaces separate fast enough, and the separation speed
   at which the additive form does so varies with the penetration; the clamp
   max(0, f) is the remedy for both.
3. Hirschkorn's damping factor D, calibrated on a forte blow, against the bias
   of dropping it.
4. Static and kinetic friction at the four sliding contacts of Hirschkorn's
   Table A.8, mu(x) = A (tanh(x) + B1 x / (1 + B2 x^4)), x = s_t / v_t (his
   Eq. 2.10, p. 25: A multiplies the whole bracket, which a plain-text
   extraction of the thesis drops): A is the kinetic coefficient and the static
   one is the maximum of mu over x > 0. The coefficients are read from
   data/params/hirschkorn-boston-gp178-note52.json.

Inputs: Hirschkorn 2004, Tables A.8 and A.9, and the inertias about the pivots
from script 01.

Pure arithmetic: import math and nothing else. Run with python3 -I.
"""
import json
import math
from pathlib import Path

print("=== 1. Regularized joint friction at the key: creep, and the explicit step ===")
Ik = 3.0972e-3   # key inertia about its pivot, kg m^2 (script 01)
cd = 0.0122      # key axis friction, N m (Table A.9)
Lk = 0.2175      # key arm to the front punching, m (Table A.3, Pk4)
g = 9.81
print(f"  key: I_pivot = {Ik:.4e} kg m^2, c_d = {cd} N m, front arm = {Lk} m")
for wt in (1e-3, 1e-6, 1e-9):
    dtau = 1e-3 * g * Lk          # 1 g of excess at the front
    u = dtau / cd
    w = wt * math.atanh(min(u, 0.999999))
    tau = Ik * wt / cd
    print(f"   w_t = {wt:7.0e} rad/s: 1 g of excess creeps at {w:.3e} rad/s,"
          f" {w * 10 * Lk * 1e6:10.4f} um of key front in 10 s;"
          f" explicit integration would need dt < {2 * tau * 1e6:.4g} us")
Ij, cdj = 1.787e-6, 3.97e-4
print(f"  jack: I_pivot = {Ij:.3e} kg m^2, c_d = {cdj} N m -> explicit integration would need"
      f" dt < {2 * Ij * 1e-6 / cdj * 1e9:.1f} ns at w_t = 1e-6 rad/s")
tau = Ik * 1e-6 / cd
print("  implicit (backward Euler) on the linearised law: amplification 1/(1 + dt/tau) < 1 for every dt;")
for dt in (1e-5, 1e-4, 1e-3):
    print(f"   key, w_t = 1e-6 rad/s, dt = {dt * 1e6:6.0f} us: amplification {1 / (1 + dt / tau):.2e}, stable")
print("  => the threshold survives regularization to within micrometres; the cost of the")
print("     law is the parameter w_t, which enters every sensitivity. The step limit holds")
print("     only for explicit integration: treated implicitly, a velocity-only law sets none.")

print("\n=== 2. Additive damping with ONE b_d, against Hunt-Crossley, contact 7 ===")
a, b, c, D = 3.92e11, 1.10e7, 3.23e4, 4.694   # Table A.8, jack/knuckle


def elastic(d):
    return a * d**3 + b * d * d + c * d


d0 = 5.82e-4   # penetration at 100 N
bd = D * elastic(d0) / d0**2
print(f"  b_d calibrated once, so that b_d d^2 = D f(d) at d0 = {d0 * 1e3:.3f} mm: b_d = {bd:.4e} N s/m^3")
print(f"  Hunt-Crossley turns attractive at d' = -1/D = {-1 / D:.4f} m/s, at every penetration")
for d in (1e-4, 3e-4, d0):
    print(f"   d = {d * 1e3:.3f} mm: f = {elastic(d):8.3f} N; the additive form turns attractive"
          f" at d' = -f/(b_d d^2) = {-elastic(d) / (bd * d * d):+.4f} m/s")
for d, v in ((3e-4, -0.2), (d0, -0.2), (d0, -0.5)):
    hc = elastic(d) * (1 + D * v)
    th = elastic(d) + bd * d * d * v
    print(f"   at d = {d * 1e3:.3f} mm, d' = {v:+.2f} m/s: Hunt-Crossley {hc:+9.2f} N, additive {th:+9.2f} N")
print("  => with one b_d the two forms agree only at the penetration it was calibrated at.")
print("     Both need the clamp max(0, f); the clamp, not the form, removes the pull.")
print("  Entry stiffness at d = 0+:")
for v in (0.02, 0.05, 0.2):
    print(f"   d' = {v:.2f} m/s: Hunt-Crossley c (1 + D d') = {c * (1 + D * v):6.0f} N/m;"
          f" additive c = {c:.0f} N/m (ratio {1 + D * v:.2f})")
print("  => the jump at touchdown is c itself, in both forms.")

print("\n=== 3. Dropping Hirschkorn's damping factor: bias on the loading force ===")
print("  D = (f_loading(x_max) / f_fit(x_max) - 1) / x'_ave (thesis Eq. 2.7), calibrated on a forte blow.")
for v in (0.01, 0.02, 0.05, 0.1):
    print(f"   contact 7, x'_ave = {v:.2f} m/s -> the elastic branch alone under-reads by {D * v * 100:5.1f} %")
print("  Hirschkorn 4.2.2: recalibrated for a piano blow, the damping factors come out four to five")
print("  times larger, so one damping constant cannot serve from pianissimo to fortissimo.")

print("\n=== 4. Static against kinetic friction at the sliding contacts (Table A.8) ===")


def mu(x, A, B1, B2):
    return A * (math.tanh(x) + B1 * x / (1 + B2 * x**4))


_RECORD = json.loads(
    (Path(__file__).resolve().parents[2] / "data" / "params" / "hirschkorn-boston-gp178-note52.json").read_text(
        encoding="utf-8"
    )
)
_ROWS = _RECORD["contacts"]["rows"]
_NAMES = {"1": "1 capstan / heel", "4": "4 back check", "7": "7 jack / knuckle", "10": "10 lever / knuckle"}
_SLIDING = [
    (_NAMES[k], *(_ROWS[k]["friction"][c]["value"] for c in ("A", "B1", "B2"))) for k in ("1", "4", "7", "10")
]

for name, A, B1, B2 in _SLIDING:
    xs = [i * 1e-4 for i in range(1, 100001)]       # x from 0.0001 to 10
    x_best = max(xs, key=lambda x: mu(x, A, B1, B2))
    peak = mu(x_best, A, B1, B2)
    if peak <= A:
        print(f"   contact {name:18s} mu_k = A = {A:.3f}: no static peak (B1 = 0), mu_s = mu_k")
    else:
        print(f"   contact {name:18s} mu_k = A = {A:.3f}, B1 = {B1:.2f}: mu_s = {peak:.3f}"
              f" at s/v_t = {x_best:.3f}, mu_s / mu_k = {peak / A:.2f}")
print("  Table A.9's axis friction is kinetic only: static friction was not included in the joints.")
